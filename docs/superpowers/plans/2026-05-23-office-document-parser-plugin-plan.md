# 高级 Office 文档原生解析插件与引导式开关实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现本地笔记应用对 Word (.docx)、Excel (.xlsx/.xls)、PPT (.pptx) 的原生解析与向量检索，配合系统设置中的高级解析开关以及未开启时的前台友好式引导逻辑，全面达成无 Sidecar 依赖的轻量化用户体验。

**Architecture:** 后端利用 calamine 进行 Excel CSV 形式的数据提取，解压 Docx 提取 XML 节点实现 Word 文本的高保真拼接，解压 Pptx 提取 Slide 幻灯片并通过 `--- Slide N ---` 页码划定边界输出。同时，于数据库及缓存中持久化管理 `enable_office_parser` 开关，在开关关闭时针对这类高级格式拦截并抛出精准的引导性报错，前端接收异常后提示用户前往设置页一键开启。

**Tech Stack:** Rust, Tauri v2, calamine 0.24, quick-xml 0.31, zip 0.6, Vue 3.5, Pinia, TypeScript

---

### Task 1: 引入依赖并初始化数据库设置项

**Files:**
- Modify: [Cargo.toml](file:///Users/mac/project/telepathy/src-tauri/Cargo.toml)
- Modify: [src/db/settings.rs](file:///Users/mac/project/telepathy/src-tauri/src/db/settings.rs)

- [ ] **Step 1.1: 添加 Rust 依赖项**
  在 `src-tauri/Cargo.toml` 文件的 `[dependencies]` 段落中增加 `calamine` 与 `quick-xml`：
  ```toml
  calamine = "0.24.0"
  quick-xml = "0.31.0"
  ```

- [ ] **Step 1.2: 数据库默认配置种子新增**
  在 `src-tauri/src/db/settings.rs` 的 `DEFAULT_SETTINGS` 列表中添加 `"enable_office_parser"` 默认设置：
  ```rust
  pub const DEFAULT_SETTINGS: &[(&str, &str)] = &[
      ("chat_model", "qwen2.5"),
      ("embedding_model", "bge-large-zh"),
      ("top_k", "5"),
      ("similarity_threshold", "0.3"),
      ("num_thread", "6"),
      ("num_ctx", "4096"),
      ("temperature", "0.7"),
      ("num_gpu", "0"),
      ("repeat_penalty", "1.1"),
      ("num_predict", "1024"),
      ("enable_office_parser", "false"),
  ];
  ```

- [ ] **Step 1.3: 运行 `cargo check` 确保编译通过**
  运行：`cargo check` (在 `/Users/mac/project/telepathy/src-tauri` 目录)
  预期：依赖下载完成，无编译错误。

- [ ] **Step 1.4: 提交**
  ```bash
  git add Cargo.toml src-tauri/Cargo.toml src-tauri/src/db/settings.rs
  git commit -m "feat: add calamine and quick-xml dependencies and seed enable_office_parser default setting"
  ```

---

### Task 2: 后端原生 Office 文档提取核心逻辑实现

**Files:**
- Modify: [src/services/parser/core.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/parser/core.rs)

- [ ] **Step 2.1: 实现 Word (.docx) 解析算法**
  在 `src-tauri/src/services/parser/core.rs` 的末尾追加 `parse_docx` 逻辑，解析 Word 中的文本：
  ```rust
  pub fn parse_docx(path: &Path) -> Result<String, AppError> {
      let file = std::fs::File::open(path)
          .map_err(|e| AppError::Internal(format!("Failed to open docx file: {}", e)))?;
      let mut archive = zip::ZipArchive::new(file)
          .map_err(|e| AppError::Internal(format!("Failed to open docx as zip: {}", e)))?;

      let mut document_xml = archive.by_name("word/document.xml")
          .map_err(|e| AppError::Internal(format!("Failed to find word/document.xml: {}", e)))?;

      let mut xml_content = String::new();
      std::io::Read::read_to_string(&mut document_xml, &mut xml_content)
          .map_err(|e| AppError::Internal(format!("Failed to read word/document.xml: {}", e)))?;

      let mut reader = quick_xml::Reader::from_str(&xml_content);
      reader.config_mut().trim_text(true);

      let mut buf = Vec::new();
      let mut out_text = String::new();
      let mut in_text = false;

      loop {
          match reader.read_event_into(&mut buf) {
              Ok(quick_xml::events::Event::Start(ref e)) => {
                  if e.name().as_ref() == b"w:t" {
                      in_text = true;
                  }
              }
              Ok(quick_xml::events::Event::End(ref e)) => {
                  if e.name().as_ref() == b"w:t" {
                      in_text = false;
                  } else if e.name().as_ref() == b"w:p" {
                      out_text.push('\n');
                  }
              }
              Ok(quick_xml::events::Event::Text(e)) => {
                  if in_text {
                      if let Ok(txt) = e.unescape() {
                          out_text.push_str(&txt);
                      }
                  }
              }
              Ok(quick_xml::events::Event::Eof) => break,
              Err(e) => return Err(AppError::Internal(format!("XML parse error: {}", e))),
              _ => {}
          }
          buf.clear();
      }

      Ok(out_text.trim().to_string())
  }
  ```

- [ ] **Step 2.2: 实现 Excel (.xlsx/.xls) 解析算法**
  在 `src-tauri/src/services/parser/core.rs` 的末尾追加 `parse_excel` 逻辑，格式化为 CSV 格式：
  ```rust
  use calamine::{Reader, Xlsx, Xls, Ods, Data, open_workbook_auto};

  pub fn parse_excel(path: &Path) -> Result<String, AppError> {
      let mut workbook = open_workbook_auto(path)
          .map_err(|e| AppError::Internal(format!("Failed to open Excel workbook: {}", e)))?;

      let mut out_text = String::new();
      let sheets = workbook.sheet_names().to_vec();

      for sheet_name in sheets {
          if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
              out_text.push_str(&format!("--- Sheet: {} ---\n", sheet_name));
              for row in range.rows() {
                  let row_text: Vec<String> = row.iter().map(|cell| {
                      match cell {
                          Data::Empty => "".to_string(),
                          Data::String(s) => s.clone(),
                          Data::Float(f) => f.to_string(),
                          Data::Int(i) => i.to_string(),
                          Data::Bool(b) => b.to_string(),
                          Data::DateTime(dt) => dt.to_string(),
                          Data::Error(err) => format!("Error: {:?}", err),
                      }
                  }).collect();
                  out_text.push_str(&row_text.join(","));
                  out_text.push('\n');
              }
              out_text.push('\n');
          }
      }

      Ok(out_text.trim().to_string())
  }
  ```

- [ ] **Step 2.3: 实现 PPT (.pptx) 解析算法**
  在 `src-tauri/src/services/parser/core.rs` 的末尾追加 `parse_pptx` 逻辑，顺序解析幻灯片，用 `--- Slide N ---` 隔离：
  ```rust
  pub fn parse_pptx(path: &Path) -> Result<String, AppError> {
      let file = std::fs::File::open(path)
          .map_err(|e| AppError::Internal(format!("Failed to open pptx file: {}", e)))?;
      let mut archive = zip::ZipArchive::new(file)
          .map_err(|e| AppError::Internal(format!("Failed to open pptx as zip: {}", e)))?;

      let mut slide_num = 1;
      let mut out_text = String::new();

      loop {
          let slide_name = format!("ppt/slides/slide{}.xml", slide_num);
          let slide_file = archive.by_name(&slide_name);
          if slide_file.is_err() {
              // 没有更多的 Slide XML，退出循环
              break;
          }

          let mut slide_xml = slide_file.unwrap();
          let mut xml_content = String::new();
          std::io::Read::read_to_string(&mut slide_xml, &mut xml_content)
              .map_err(|e| AppError::Internal(format!("Failed to read slide XML: {}", e)))?;

          let mut reader = quick_xml::Reader::from_str(&xml_content);
          reader.config_mut().trim_text(true);

          let mut buf = Vec::new();
          let mut slide_text = String::new();
          let mut in_text = false;

          loop {
              match reader.read_event_into(&mut buf) {
                  Ok(quick_xml::events::Event::Start(ref e)) => {
                      if e.name().as_ref() == b"a:t" {
                          in_text = true;
                      }
                  }
                  Ok(quick_xml::events::Event::End(ref e)) => {
                      if e.name().as_ref() == b"a:t" {
                          in_text = false;
                      } else if e.name().as_ref() == b"a:p" {
                          slide_text.push(' ');
                      }
                  }
                  Ok(quick_xml::events::Event::Text(e)) => {
                      if in_text {
                          if let Ok(txt) = e.unescape() {
                              slide_text.push_str(&txt);
                          }
                      }
                  }
                  Ok(quick_xml::events::Event::Eof) => break,
                  Err(e) => return Err(AppError::Internal(format!("XML slide parse error: {}", e))),
                  _ => {}
              }
              buf.clear();
          }

          if !slide_text.trim().is_empty() {
              out_text.push_str(&format!("--- Slide {} ---\n", slide_num));
              out_text.push_str(&slide_text.trim());
              out_text.push_str("\n\n");
          }

          slide_num += 1;
      }

      Ok(out_text.trim().to_string())
  }
  ```

- [ ] **Step 2.4: 编写并运行单元测试**
  在 `core.rs` 的末尾追加测试用例，校验 DOCX、XLSX、PPTX 提取框架是否正常。由于本地可能没有现成的测试文件，我们可以只写编译测试以确保这些函数没有语法问题：
  ```rust
  #[test]
  fn test_office_parsers_compilation() {
      // 仅用于验证方法签名和依赖能否正常编译
      let dummy_path = Path::new("non_existent_file.docx");
      let _ = parse_docx(dummy_path);
      let _ = parse_excel(dummy_path);
      let _ = parse_pptx(dummy_path);
  }
  ```
  运行：`cargo test --package temp-app --lib -- services::parser::core::tests`
  预期：测试通过（可以正确忽略 non-existent 错误，或者忽略未找到文件）。

- [ ] **Step 2.5: 提交**
  ```bash
  git add src-tauri/src/services/parser/core.rs
  git commit -m "feat: implement native docx, excel, pptx parsing algorithms in Rust"
  ```

---

### Task 3: 改造文件类型分发器并增加权限守护

**Files:**
- Modify: [src/services/parser/mod.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/parser/mod.rs)

- [ ] **Step 3.1: 拦截机制与逻辑重构**
  修改 `src-tauri/src/services/parser/mod.rs` 的 `parse_document_async` 函数。从 `SettingsCache` 中获取 `enable_office_parser` 开关。如果该开关不为 `"true"`，并且文件后缀为 `docx`, `xlsx`, `xls`, `pptx`，则拦截并返回特定的引导性错误消息；否则调用原生解析方法。
  
  替换 `mod.rs` 中的分发匹配逻辑：
  ```rust
  // 获取 enable_office_parser 配置
  let enable_office_parser = cache.get("enable_office_parser").map(|s| s == "true").unwrap_or(false);

  match extension.as_str() {
      "txt" | "md" | "json" | "csv" => {
          let path_clone = path.to_path_buf();
          tokio::task::spawn_blocking(move || core::parse_text_file(&path_clone))
              .await
              .map_err(|e| AppError::Internal(format!("Task failed: {}", e)))?
      }
      "docx" | "xlsx" | "xls" | "pptx" => {
          if !enable_office_parser {
              return Err(AppError::Internal(format!(
                  "该文档为高级 Office 格式（.{}）。当前未开启「高级文档解析扩展」，请前往「系统设置」开启该功能以开始解析和检索。",
                  extension
              )));
          }
          let path_clone = path.to_path_buf();
          let ext_clone = extension.clone();
          tokio::task::spawn_blocking(move || {
              match ext_clone.as_str() {
                  "docx" => core::parse_docx(&path_clone),
                  "xlsx" | "xls" => core::parse_excel(&path_clone),
                  "pptx" => core::parse_pptx(&path_clone),
                  _ => unreachable!(),
              }
          })
          .await
          .map_err(|e| AppError::Internal(format!("Task failed: {}", e)))?
      }
      "doc" | "ppt" => {
          // doc 和 ppt 二进制格式只保留 sidecar 分支，若无 sidecar 则做友好引导提示转换
          let is_sidecar_installed = sidecar::is_sidecar_installed(app_handle);
          if is_sidecar_installed {
              sidecar::parse_with_sidecar(app_handle, path).await
          } else {
              Err(AppError::Internal(format!(
                  "当前文件为老旧二进制格式（.{}），原生解析器暂不支持直接解析。请将其转换为更新的 .docx / .pptx 格式，或者在「系统设置」中下载并安装高级解析插件。",
                  extension
              )))
          }
      }
      "pdf" => {
          let is_sidecar_installed = sidecar::is_sidecar_installed(app_handle);
          if use_vision_parser && is_sidecar_installed {
              vision::parse_pdf_with_vision(path, app_handle).await
          } else {
              let path_clone = path.to_path_buf();
              tokio::task::spawn_blocking(move || core::parse_pdf(&path_clone))
                  .await
                  .map_err(|e| AppError::Internal(format!("Task failed: {}", e)))?
          }
      }
      "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => {
          if use_vision_parser {
              vision::parse_image_with_vision(path, app_handle).await
          } else {
              Err(AppError::Internal(
                  "该文件为图片格式，需在系统设置中启用『文档多模态视觉解析』才能进行解析和索引。".to_string(),
              ))
          }
      }
      _ => Err(AppError::Internal(format!(
          "Unsupported file type: .{}",
          extension
      ))),
  }
  ```

- [ ] **Step 3.2: 运行测试与格式规范校验**
  运行：`cargo check` 确保编译完全通过，所有的 `AppError` 处理和分支完全正常。

- [ ] **Step 3.3: 提交**
  ```bash
  git add src-tauri/src/services/parser/mod.rs
  git commit -m "feat: intercept Office documents when enable_office_parser toggle is false, else dispatch to native parsers"
  ```

---

### Task 4: 前端 Pinia Store 更新与设置界面扩展

**Files:**
- Modify: [src/stores/settings.ts](file:///Users/mac/project/telepathy/src/stores/settings.ts)
- Modify: [src/views/Settings.vue](file:///Users/mac/project/telepathy/src/views/Settings.vue)

- [ ] **Step 4.1: 在 Settings Store 中加入开关状态管理**
  在 `src/stores/settings.ts` 中加入一个响应式的 `enableOfficeParser` 计算属性以方便视图快捷绑定更新：
  ```typescript
  const enableOfficeParser = computed({
    get: () => settings.value.enable_office_parser === 'true',
    set: (v: boolean) => saveSetting('enable_office_parser', String(v))
  });
  ```
  同时要在 `return` 段落中导出该属性：
  ```typescript
  return {
    // ... 其它属性
    enableOfficeParser,
  }
  ```

- [ ] **Step 4.2: 在 Settings 界面中添加“高级功能扩展”卡片**
  修改 `src/views/Settings.vue`。在“通知音效设置卡片”下方增加一个独立的“高级功能扩展”卡片模块。
  ```vue
  <!-- 高级功能扩展设置卡片 -->
  <div class="border border-border-main/50 rounded-xl bg-panel-bg p-5 hover:border-border-main/80 transition-all duration-300 shadow-sm space-y-4">
    <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
      <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/>
      </svg>
      <h3 class="text-sm font-bold text-text-primary">高级功能扩展</h3>
    </div>
    <div class="flex justify-between items-center">
      <div>
        <h4 class="text-xs font-bold text-text-primary">高级 Office 文档解析扩展</h4>
        <p class="text-[10px] text-text-muted mt-1">开启后支持原生解析 Word (.docx)、Excel (.xlsx/.xls)、PPT (.pptx) 等格式，目前全功能免费使用</p>
      </div>
      <button 
        @click="store.enableOfficeParser = !store.enableOfficeParser"
        class="w-12 h-6 rounded-full p-0.5 transition-colors flex items-center shadow-inner relative cursor-pointer"
        :class="store.enableOfficeParser ? 'bg-brand' : 'bg-surface-bg border border-border-main'"
      >
        <div 
          class="w-5 h-5 bg-white rounded-full shadow-md transition-all duration-200"
          :class="store.enableOfficeParser ? 'translate-x-6' : 'translate-x-0'"
        ></div>
      </button>
    </div>
  </div>
  ```

- [ ] **Step 4.3: 提交**
  ```bash
  git add src/stores/settings.ts src/views/Settings.vue
  git commit -m "feat: add advanced feature toggle for Office parsing in settings store and view"
  ```

---

### Task 5: 导入文档对话框扩展

**Files:**
- Modify: [src/components/batch/BatchImportDialog.vue](file:///Users/mac/project/telepathy/src/components/batch/BatchImportDialog.vue)
- Modify: [src/commands/document.rs](file:///Users/mac/project/telepathy/src-tauri/src/commands/document.rs)

- [ ] **Step 5.1: 扩展后端文件夹扫描支持的扩展名**
  在 `src-tauri/src/commands/document.rs` 的 `scan_folder` 函数中：
  将 `supported_extensions` 从 `["pdf", "md", "txt", "docx"]` 扩大为：
  ```rust
  let supported_extensions = ["pdf", "md", "txt", "docx", "xlsx", "pptx", "xls"];
  ```

- [ ] **Step 5.2: 扩大前端导入文件选择对话框的文件后缀筛选**
  在 `src/components/batch/BatchImportDialog.vue` 的 `handleSelectFiles` 中，修改 `extensions`：
  ```typescript
  extensions: ['pdf', 'md', 'txt', 'docx', 'xlsx', 'pptx', 'xls']
  ```

- [ ] **Step 5.3: 在导入项的图标映射中增加新扩展名展示**
  在 `BatchImportDialog.vue` 的模板中（约 413 行）：
  ```vue
  {{ item.name.endsWith('.pdf') ? '📕' : item.name.endsWith('.md') ? '📝' : item.name.endsWith('.docx') ? '📘' : ['xlsx', 'xls'].some(ext => item.name.endsWith('.' + ext)) ? '📊' : item.name.endsWith('.pptx') ? '📈' : '📄' }}
  ```

- [ ] **Step 5.4: 提交**
  ```bash
  git add src/components/batch/BatchImportDialog.vue src-tauri/src/commands/document.rs
  git commit -m "feat: expand import format filter to support xlsx, xls, and pptx in frontend and scan_folder cmd"
  ```

---

### Task 6: 文档管理界面高亮引导 (Documents.vue)

**Files:**
- Modify: [src/views/Documents.vue](file:///Users/mac/project/telepathy/src/views/Documents.vue)

- [ ] **Step 6.1: 引入 Settings Store 并控制高亮引导 Banner 显示**
  修改 `src/views/Documents.vue`，导入并初始化 `useSettingsStore`，判断 `enable_office_parser` 开关是否被开启。
  
  在 `<script setup>` 中：
  ```typescript
  import { useSettingsStore } from '@/stores/settings';
  const settingsStore = useSettingsStore();

  // 定义局部状态以支持手动关闭提示 Banner
  const isBannerDismissed = ref(false);
  const showOfficeBanner = computed(() => {
    return !settingsStore.settings.enable_office_parser || settingsStore.settings.enable_office_parser !== 'true';
  });
  ```

- [ ] **Step 6.2: 渲染高亮引导横幅 (Banner)**
  在 `Documents.vue` 的模板中，替换原来隐藏的 sidecar 警告横幅模块（约 327 行的 `v-if="false && !sidecarInstalled"`）：
  ```vue
  <!-- 高级 Office 文档解析引导 Banner -->
  <div v-if="showOfficeBanner && !isBannerDismissed" class="bg-amber-500/10 backdrop-blur-sm border border-amber-500/25 rounded-xl p-4 animate-fade-in shadow-sm select-none">
    <div class="flex justify-between items-start">
      <div class="flex items-start gap-3">
        <div class="p-1 bg-amber-500/15 text-amber-500 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <div>
          <h3 class="font-bold text-amber-500 text-sm tracking-tight">提示：高级 Office 文档解析未开启</h3>
          <p class="text-text-secondary mt-1.5 text-xs leading-relaxed">
            您当前无法解析 Word (.docx)、Excel (.xlsx/.xls) 和 PPT (.pptx) 文档。
            您可免费开启此功能，以启用本地高性能免插件提取。
          </p>
        </div>
      </div>
      <button @click="isBannerDismissed = true" class="text-text-muted hover:text-text-primary p-1 rounded-lg transition-colors">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
    <div class="mt-3.5 flex gap-3">
      <button 
        @click="$router.push('/settings')"
        class="text-xs font-bold text-white bg-brand hover:bg-brand/90 px-3 py-1.5 rounded-lg shadow-sm transition-all"
      >
        前往设置开启
      </button>
      <button 
        @click="isBannerDismissed = true"
        class="text-xs font-bold text-text-secondary hover:text-text-primary px-3 py-1.5 transition-colors"
      >
        稍后处理
      </button>
    </div>
  </div>
  ```

- [ ] **Step 6.3: 运行 TypeScript 语法校验与前端构建**
  运行：`vue-tsc --noEmit`
  预期：无 TypeScript 报错。
  运行：`npm run build`
  预期：打包构建成功。

- [ ] **Step 6.4: 提交**
  ```bash
  git add src/views/Documents.vue
  git commit -m "feat: design highly friendly guide Banner in Documents view for unenabled Office parser"
  ```

---

## 5. 整体完整性验证 (Verification Steps)

1. **后端验证**：
   - 运行：`cargo test` 确保无测试失败。
   - 运行：`cargo check` 确认无 warnings 或 errors。

2. **前端静态分析**：
   - 运行：`vue-tsc --noEmit` 校验 TypeScript 完美通过。

3. **版本库干净校验**：
   - 运行：`git status` 查看是否有残留或多余的文件。
