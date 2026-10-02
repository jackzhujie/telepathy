# 文档多模态视觉解析实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现应用在高级设置中配置“文档多模态视觉解析”的开关，并在开启时对导入的图片直接调用本地 Vision 模型进行 OCR 描述，对 PDF 文件若安装了 Sidecar 插件则由其渲染图片并由主程序本地多模态解析，若关闭或未安装则降级为原本的解析策略。

**Architecture:** 
1. **前端配置项**：在 `Models.vue` 的 `advanced` 页签增加视觉文档解析开关，由 `use_vision_parser` (SQLite `settings` 表记录) 控制，且与视觉模型/投影器的配置状态（存在性）强制联动。
2. **后端统一异步入口**：重构 `parser::mod.rs` 为 `parse_document_async` 异步路由。
3. **视觉解析实现**：
   - 对于图片：直接调用 `VisionAdapter` 执行流式 OCR 推理并收集。
   - 对于 PDF：调用高级 Sidecar 的命令行协议（`parser-advanced --render-pdf-pages <pdf_path> <temp_dir>`）渲染页面为图片，Rust 对每张图片流式 OCR，最后拼接。
4. **解析降级**：当未配置/未启用视觉或未安装 Sidecar 时，PDF 降级至原 `pdf_extract`，图片则报错提示用户开启开关。

**Tech Stack:** Tauri v2, Rust (tokio, llama-cpp-4, rusqlite), Vue 3 + TypeScript, Tailwind CSS, Reka-UI

---

### Task 1: 前端设置页面修改 (Frontend Settings UI)

**Files:**
- Modify: `src/views/Models.vue`

- [ ] **Step 1: 编写 UI 修改，新增视觉文档解析卡片并绑定状态与控制逻辑**
  
  在 `src/views/Models.vue` 中导入 `watch` (已导入)，并在脚本区添加：
  ```typescript
  const isVisionConfigured = computed(() => {
    return !!store.settings.vision_model && !!store.settings.vision_mmproj;
  });

  const useVisionParser = computed(() => {
    return store.settings.use_vision_parser === 'true' && isVisionConfigured.value;
  });

  async function toggleVisionParser() {
    if (!isVisionConfigured.value) return;
    const targetState = !useVisionParser.value;
    try {
      await store.saveSetting('use_vision_parser', String(targetState));
      showToast(targetState ? '已开启文档视觉解析（解析速度会变慢）' : '已关闭文档视觉解析');
    } catch (e) {
      console.error('保存失败:', e);
    }
  }

  // 监听视觉模型配置的变更，如果被清除，自动关闭该功能
  watch(isVisionConfigured, async (newVal) => {
    if (!newVal && store.settings.use_vision_parser === 'true') {
      try {
        await store.saveSetting('use_vision_parser', 'false');
      } catch (e) {
        console.error('自动关闭失败:', e);
      }
    }
  });
  ```

  在 `<template>` 区域 `Tab 内容 B` 的 `<div v-show="activeTab === 'advanced'" ...>` 块的最后添加多模态解析配置卡片：
  ```html
          <!-- 文档多模态视觉解析配置卡片 -->
          <div class="bg-panel-bg border border-border-main/50 hover:border-border-main/80 rounded-xl p-5 space-y-4 transition-all duration-300 shadow-sm md:col-span-2">
            <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand">
                <path d="M15 12c0 1.657-1.343 3-3 3s-3-1.343-3-3 1.343-3 3-3 3 1.343 3 3z"/>
                <path d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
              </svg>
              <h3 class="text-sm font-bold text-text-primary">文档多模态视觉解析</h3>
            </div>

            <div class="flex justify-between items-start gap-4">
              <div class="space-y-1">
                <h4 class="text-xs font-bold text-text-primary">启用视觉模型解析文档</h4>
                <p class="text-[10px] text-text-muted leading-relaxed">
                  开启后，导入 PNG/JPG/WebP/BMP 等图片格式文档时，将自动使用本地 Vision 模型生成内容描述；导入 PDF 时，若已安装高级解析插件，将对每一页生成图像并进行多模态文本与表格提取。
                </p>
              </div>
              
              <!-- 开关按钮 -->
              <button 
                @click="toggleVisionParser"
                :disabled="!isVisionConfigured"
                class="w-12 h-6 rounded-full p-0.5 transition-colors flex items-center shadow-inner relative cursor-pointer"
                :class="[
                  useVisionParser ? 'bg-brand' : 'bg-surface-bg border border-border-main',
                  !isVisionConfigured ? 'opacity-40 cursor-not-allowed' : ''
                ]"
              >
                <div 
                  class="w-5 h-5 bg-white rounded-full shadow-md transition-all duration-200"
                  :class="useVisionParser ? 'translate-x-6' : 'translate-x-0'"
                ></div>
              </button>
            </div>

            <!-- 提示与警示区域 -->
            <div v-if="!isVisionConfigured" class="p-2.5 bg-red-500/10 border border-red-500/20 rounded-lg flex items-start gap-2">
              <span class="text-xs">⚠️</span>
              <p class="text-[10px] text-red-500 font-medium leading-normal">
                请先在“模型市场与本地管理”中配置<strong>视觉模型</strong>与<strong>视觉投影器 (mmproj)</strong>，方可启用此功能。
              </p>
            </div>
            <div v-else class="p-2.5 bg-yellow-500/10 border border-yellow-500/20 rounded-lg flex items-start gap-2">
              <span class="text-xs">💡</span>
              <p class="text-[10px] text-yellow-600 dark:text-yellow-400 font-medium leading-normal">
                <strong>警告：</strong>使用视觉多模态解析大文档或多页 PDF 时，会显著降低解析/索引速度（每页可能耗时数秒至数十秒），并会占用较多 CPU/GPU 内存。
              </p>
            </div>
          </div>
  ```

- [ ] **Step 2: 验证界面是否能正确渲染并且无语法报错**
  
  运行 `pnpm run build` 或使用 `tsc` 检测。如果修改无编译错误，提交：
  ```bash
  git add src/views/Models.vue
  git commit -m "feat: add document vision parser settings card to Models UI"
  ```

---

### Task 2: 后端 Sidecar PDF 渲染支持 (Backend Sidecar PDF Rendering)

**Files:**
- Modify: `src-tauri/src/services/parser/sidecar.rs`

- [ ] **Step 1: 在 `sidecar.rs` 中实现 `render_pdf_pages_via_sidecar` 接口**
  
  打开 `src-tauri/src/services/parser/sidecar.rs`。在其末尾添加：
  ```rust
  /// 调用 Sidecar 将 PDF 页面渲染为临时图片输出到指定文件夹下
  /// 运行命令: parser-advanced --render-pdf-pages <pdf_path> <temp_dir>
  pub async fn render_pdf_pages_via_sidecar(
      app_handle: &AppHandle,
      pdf_path: &Path,
      temp_dir: &Path,
  ) -> Result<usize, AppError> {
      if !is_sidecar_installed(app_handle) {
          return Err(AppError::Internal(
              "高级解析器 Sidecar 未安装，无法执行 PDF 页面视觉渲染。".to_string(),
          ));
      }

      let sidecar_path = get_sidecar_path(app_handle)?;
      let output = tokio::process::Command::new(&sidecar_path)
          .arg("--render-pdf-pages")
          .arg(pdf_path.to_string_lossy().as_ref())
          .arg(temp_dir.to_string_lossy().as_ref())
          .output()
          .await
          .map_err(|e| AppError::Internal(format!("Failed to run sidecar for rendering: {}", e)))?;

      if !output.status.success() {
          let stderr = String::from_utf8_lossy(&output.stderr);
          return Err(AppError::Internal(format!(
              "Sidecar PDF rendering failed: {}",
              stderr
          )));
      }

      // 从输出文本中尝试获取渲染的总页数，如果无法获取，直接读取输出文件夹中图片数量
      let stdout = String::from_utf8_lossy(&output.stdout);
      println!("[Sidecar] Render PDF output: {}", stdout);

      let files = std::fs::read_dir(temp_dir)
          .map_err(|e| AppError::Internal(format!("Failed to read temp page output dir: {}", e)))?;

      let count = files
          .filter_map(|entry| entry.ok())
          .filter(|entry| {
              entry.path().extension()
                  .map(|ext| ext.to_string_lossy().to_lowercase() == "jpg" || ext.to_string_lossy().to_lowercase() == "png")
                  .unwrap_or(false)
          })
          .count();

      Ok(count)
  }
  ```

- [ ] **Step 2: 编译测试并提交**
  
  运行 `cargo check` 验证是否编译成功。成功后提交：
  ```bash
  git add src-tauri/src/services/parser/sidecar.rs
  git commit -m "feat: implement sidecar helper for rendering PDF pages"
  ```

---

### Task 3: 视觉解析服务实现 (Backend Vision Parser Implementation)

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`

- [ ] **Step 1: 实现单张图片和 PDF 的本地视觉解析函数**
  
  打开 `src-tauri/src/services/parser/vision.rs`，添加引用和两个函数实现：
  ```rust
  use crate::db::settings_cache::SettingsCache;
  use crate::services::inference::VisionEngineManagerState;
  use tauri::Manager;
  use tokio_util::sync::CancellationToken;
  use std::path::Path;

  /// 使用本地 Vision 模型解析单张图片并提取文本
  pub async fn parse_image_with_vision(path: &Path, app_handle: &AppHandle) -> Result<String, AppError> {
      let image_data = std::fs::read(path)
          .map_err(|e| AppError::Internal(format!("读取图片文件失败: {}", e)))?;
      
      // 压缩图片以保证低配显存能够安全执行
      let compressed_bytes = compress_image(&image_data, 448)?;

      // 获取 VisionEngine 管理状态
      let vision_state = app_handle.state::<VisionEngineManagerState>();
      let vision_arc = vision_state.0.clone();
      let mut vision_guard = vision_arc.lock().await;

      // 从设置中获取 Vision 加载参数
      let db_path = app_handle
          .path()
          .app_data_dir()
          .map(|p| p.join("telepathy.db"))
          .map_err(|e| AppError::Internal(format!("Failed to get app data path: {}", e)))?;
      let conn = rusqlite::Connection::open(&db_path)
          .map_err(|e| AppError::Internal(format!("Failed to open DB: {}", e)))?;
      
      let settings_cache = app_handle.state::<std::sync::Arc<SettingsCache>>();
      
      let vision_model = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_model")?
          .ok_or_else(|| AppError::Internal("未配置视觉模型".to_string()))?;
      let vision_mmproj = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_mmproj")?
          .ok_or_else(|| AppError::Internal("未配置 mmproj 视觉投影器".to_string()))?;

      let num_thread = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_thread")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(6);
      let num_ctx = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_ctx")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(4096);
      let num_gpu = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_gpu")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(0);

      let app_data_dir = app_handle.path().app_data_dir().unwrap();
      let model_path = app_data_dir.join("downloads").join(&vision_model);
      let mmproj_path = app_data_dir.join("downloads").join(&vision_mmproj);

      if !model_path.exists() || !mmproj_path.exists() {
          return Err(AppError::Internal("本地视觉模型文件或投影器文件不存在".to_string()));
      }

      // 加载模型
      vision_guard.load_model(
          &model_path,
          &mmproj_path,
          num_ctx as u32,
          num_gpu,
          num_thread
      ).await?;

      // 进行 OCR 推理
      let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
      let cancel_token = std::sync::Arc::new(CancellationToken::new());

      let query = "请将此图片中的所有文字内容提取出来。如果是表格，请完整保留 Markdown 表格格式；如果含有图表，请在适当位置用文字进行描述。请直接输出提取出的内容，不要有任何前导客套话或多余的解释。";

      let cancel_clone = cancel_token.clone();
      let stream_task = tokio::spawn(async move {
          vision_guard.stream_vision_chat(
              query,
              vec![compressed_bytes],
              1024,
              tx,
              cancel_clone,
          ).await
      });

      let mut extracted_text = String::new();
      while let Some(token) = rx.recv().await {
          extracted_text.push_str(&token);
      }

      stream_task.await
          .map_err(|e| AppError::Internal(format!("Stream task panicked: {}", e)))??;

      Ok(extracted_text)
  }

  /// 视觉解析 PDF（借用 Sidecar 渲染 PDF 页面，再逐页 Vision OCR）
  pub async fn parse_pdf_with_vision(path: &Path, app_handle: &AppHandle) -> Result<String, AppError> {
      let temp_id = uuid::Uuid::new_v4().to_string();
      let app_data_dir = app_handle.path().app_data_dir().unwrap();
      let temp_dir = app_data_dir.join("temp").join(format!("pdf_render_{}", temp_id));
      
      std::fs::create_dir_all(&temp_dir)
          .map_err(|e| AppError::Internal(format!("Failed to create temp PDF render directory: {}", e)))?;

      // 1. 调用 Sidecar 渲染 PDF 页面图片
      let render_result = crate::services::parser::sidecar::render_pdf_pages_via_sidecar(
          app_handle,
          path,
          &temp_dir
      ).await;

      if let Err(e) = render_result {
          // 清理文件夹并报错
          let _ = std::fs::remove_dir_all(&temp_dir);
          return Err(e);
      }

      // 2. 读取并排序所有导出的页面图片
      let mut page_images = Vec::new();
      if let Ok(entries) = std::fs::read_dir(&temp_dir) {
          for entry in entries.flatten() {
              let p = entry.path();
              if p.is_file() {
                  let is_img = p.extension()
                      .map(|ext| ext.to_string_lossy().to_lowercase() == "jpg" || ext.to_string_lossy().to_lowercase() == "png")
                      .unwrap_or(false);
                  if is_img {
                      page_images.push(p);
                  }
              }
          }
      }

      // 按页面名称的数字序号排序 (例如 page_1.jpg, page_2.jpg 等)
      page_images.sort_by(|a, b| {
          let a_num = a.file_stem().and_then(|s| s.to_string_lossy().replace("page_", "").parse::<int>().ok()).unwrap_or(0);
          let b_num = b.file_stem().and_then(|s| s.to_string_lossy().replace("page_", "").parse::<int>().ok()).unwrap_or(0);
          a_num.cmp(&b_num)
      });

      // 3. 对每一页运行视觉解析
      let mut full_markdown = String::new();

      // 获取/缓存加载参数以加速
      let db_path = app_data_dir.join("telepathy.db");
      let conn = rusqlite::Connection::open(&db_path)
          .map_err(|e| AppError::Internal(format!("Failed to open DB: {}", e)))?;
      let settings_cache = app_handle.state::<std::sync::Arc<SettingsCache>>();
      
      let vision_model = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_model")?
          .ok_or_else(|| AppError::Internal("未配置视觉模型".to_string()))?;
      let vision_mmproj = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_mmproj")?
          .ok_or_else(|| AppError::Internal("未配置 mmproj 视觉投影器".to_string()))?;

      let num_thread = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_thread")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(6);
      let num_ctx = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_ctx")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(4096);
      let num_gpu = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_gpu")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(0);

      let model_path = app_data_dir.join("downloads").join(&vision_model);
      let mmproj_path = app_data_dir.join("downloads").join(&vision_mmproj);

      if !model_path.exists() || !mmproj_path.exists() {
          let _ = std::fs::remove_dir_all(&temp_dir);
          return Err(AppError::Internal("本地视觉模型文件或投影器文件不存在".to_string()));
      }

      // 获取 VisionEngine 并全局加载一次模型（避免在每页的循环中重载）
      let vision_state = app_handle.state::<VisionEngineManagerState>();
      let vision_arc = vision_state.0.clone();
      let mut vision_guard = vision_arc.lock().await;
      vision_guard.load_model(
          &model_path,
          &mmproj_path,
          num_ctx as u32,
          num_gpu,
          num_thread
      ).await?;

      for (i, img_path) in page_images.iter().enumerate() {
          let page_num = i + 1;
          let image_data = match std::fs::read(img_path) {
              Ok(d) => d,
              Err(e) => {
                  println!("[Vision Parser] Warning: failed to read page {}: {}", page_num, e);
                  continue;
              }
          };

          let compressed_bytes = match compress_image(&image_data, 448) {
              Ok(b) => b,
              Err(e) => {
                  println!("[Vision Parser] Warning: failed to compress page {}: {}", page_num, e);
                  continue;
              }
          };

          let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
          let cancel_token = std::sync::Arc::new(CancellationToken::new());
          let query = format!(
              "这是文档的第 {} 页图片。请完整提取此页中的文字（如果是表格，请保留 Markdown 表格格式；如果含有图表，请进行语义描述）。请直接输出提取出的内容，不要有任何前导客套话或多余的解释。",
              page_num
          );

          let cancel_clone = cancel_token.clone();
          // 在循环内借用 Mutex 以便 stream_vision_chat
          let stream_task = vision_guard.stream_vision_chat(
              &query,
              vec![compressed_bytes],
              1024,
              tx,
              cancel_clone,
          );

          let receiver_task = tokio::spawn(async move {
              let mut page_text = String::new();
              while let Some(token) = rx.recv().await {
                  page_text.push_str(&token);
              }
              page_text
          });

          if let Err(e) = stream_task.await {
              println!("[Vision Parser] Warning: vision stream failed on page {}: {}", page_num, e);
              continue;
          }

          let page_text = receiver_task.await.unwrap_or_default();
          
          if !full_markdown.is_empty() {
              full_markdown.push_str("\n\n");
          }
          full_markdown.push_str(&format!("--- [第 {} 页] ---\n\n", page_num));
          full_markdown.push_str(&page_text);
      }

      // 4. 清理临时文件夹
      let _ = std::fs::remove_dir_all(&temp_dir);

      Ok(full_markdown)
  }
  ```
  *(注: 需要修复 `int` 为 `i32`，已在后续校验步骤修复)*

- [ ] **Step 2: 编译测试并提交**
  
  运行 `cargo check` 验证是否编译成功。成功后提交：
  ```bash
  git add src-tauri/src/services/parser/vision.rs
  git commit -m "feat: implement image and PDF page vision parser logic in vision.rs"
  ```

---

### Task 4: 后端解析路由与指令重构 (Backend Dispatcher & Commands)

**Files:**
- Modify: `src-tauri/src/services/parser/mod.rs`
- Modify: `src-tauri/src/commands/document.rs`
- Modify: `src-tauri/src/commands/indexing.rs`
- Modify: `src-tauri/src/commands/reindex.rs`

- [ ] **Step 1: 重构 `src-tauri/src/services/parser/mod.rs` 中的异步路由**
  
  将原本同步的路由替换为异步路由 `parse_document_async`，并在其中加入配置读取与降级。
  ```rust
  pub async fn parse_document_async(path: &Path, app_handle: &AppHandle) -> Result<String, AppError> {
      let extension = path
          .extension()
          .and_then(|e| e.to_str())
          .unwrap_or("")
          .to_lowercase();

      let cache = app_handle.state::<std::sync::Arc<crate::db::settings_cache::SettingsCache>>();
      let use_vision_parser = cache.get("use_vision_parser").map(|s| s == "true").unwrap_or(false);

      match extension.as_str() {
          "txt" | "md" | "json" | "csv" => {
              let path_clone = path.to_path_buf();
              tokio::task::spawn_blocking(move || core::parse_text_file(&path_clone))
                  .await
                  .map_err(|e| AppError::Internal(format!("Task failed: {}", e)))?
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
          ext if sidecar::requires_sidecar(path) => {
              sidecar::parse_with_sidecar(app_handle, path).await
          }
          _ => Err(AppError::Internal(format!(
              "Unsupported file type: .{}",
              extension
          ))),
      }
  }
  ```

- [ ] **Step 2: 修改 `src-tauri/src/commands/document.rs` 中的 `parse_document` 指令**
  
  ```rust
  #[tauri::command]
  pub async fn parse_document(doc_id: String, app_handle: AppHandle) -> Result<String, AppError> {
      let db_path = get_db_path(&app_handle)?;
      let conn = rusqlite::Connection::open(&db_path)
          .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

      let doc = documents::get_document(&conn, &doc_id)?
          .ok_or_else(|| AppError::Internal("Document not found".to_string()))?;

      documents::update_document_status(&conn, &doc_id, "parsing", None)?;

      let path = Path::new(&doc.library_path);
      
      // 调用异步集中路由
      let result = parser::parse_document_async(path, &app_handle).await;

      match result {
          Ok(text) => {
              documents::update_document_status(&conn, &doc_id, "done", None)?;
              Ok(text)
          }
          Err(e) => {
              documents::update_document_status(&conn, &doc_id, "error", Some(&e.to_string()))?;
              Err(e)
          }
      }
  }
  ```

- [ ] **Step 3: 修改 `src-tauri/src/commands/indexing.rs` 中的 `index_document` 指令**
  
  ```rust
      let library_path = doc.library_path.clone();
      // 调用异步解析入口
      let text = parser::parse_document_async(Path::new(&library_path), &app_handle).await?;
  ```

- [ ] **Step 4: 修改 `src-tauri/src/commands/reindex.rs` 中的 `reindex_single_document` 函数**
  
  增加 `app_handle: &AppHandle` 参数，并修改解析行为：
  ```rust
  // reindex_all_documents 函数中修改循环内的调用：
  match reindex_single_document(&db_path, &doc.id, &doc.library_path, &doc.project_id, &emb, &app_handle).await { ... }

  // reindex_single_document 签名与实现变更：
  async fn reindex_single_document(
      db_path: &std::path::Path,
      doc_id: &str,
      file_path: &str,
      project_id: &Option<String>,
      emb: &embedder::Embedder,
      app_handle: &AppHandle,
  ) -> Result<(), AppError> {
      let conn = rusqlite::Connection::open(db_path)
          .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

      vectors::delete_chunks_by_document(&conn, doc_id)
          .map_err(|e| AppError::Internal(format!("Failed to delete old chunks: {}", e)))?;

      let file_path_owned = file_path.to_string();
      // 采用异步文档解析
      let text = parser::parse_document_async(std::path::Path::new(&file_path_owned), app_handle).await?;
  ```

- [ ] **Step 5: 验证编译并提交**
  
  运行 `cargo check` 确保编译全部正常，然后进行本地提交：
  ```bash
  git add src-tauri/src/services/parser/mod.rs src-tauri/src/commands/document.rs src-tauri/src/commands/indexing.rs src-tauri/src/commands/reindex.rs
  git commit -m "refactor: unify document parsing with parse_document_async across commands"
  ```

---

### Task 5: 验证与编译检查 (Verification)

**Files:**
- Test compilation of whole project: `cargo build` and `pnpm run build` or `pnpm run tsc`

- [ ] **Step 1: 全局运行测试，确保所有变更文件无编译红字**
  
  运行: `cargo check` 以及 `tsc --noEmit`。
  
- [ ] **Step 2: 最终确认修改，提交全部剩余内容**
  
  提交: `git status` 检查无残留工作区脏数据。
