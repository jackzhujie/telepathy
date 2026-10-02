# 设计规格：文档多模态视觉解析 (Vision Document Parsing)

**日期**: 2026-05-19  
**模块**: 知识库摄入与多模态解析  
**状态**: 提案已批准  

---

## 1. 背景与设计原则 (Background & Design Principles)

### 1.1 背景
随着 Telepathy 支持了本地多模态 Vision 模型（如 LLaVA，Qwen2-VL 等），应用已具备直接理解图像内容的能力。目前，知识库文档解析（Ingestion）模块在处理图片和 PDF 时仍属于传统方式：
* **图片文件**：暂不支持索引，尝试索引时会直接报错。
* **PDF 文件**：采用纯文本提取库 `pdf_extract`，无法提取扫描版 PDF（纯图片 PDF）的内容，也难以处理含有图表、复杂表格的 PDF 页面。

### 1.2 设计原则
1. **防爆/资源受限保护**：本地运行 Vision 模型对 CPU/GPU 内存要求极高，逐页解析大文件十分耗时。必须增加全局开关，且在未配置视觉模型时置灰禁用，同时在 UI 中提供明确的解析速度变慢警示。
2. **轻量与解耦**：Rust 主程序本身不直接集成庞大的 C/C++ PDF 渲染动态库（如 PDFium/Poppler）。使用原有的 **Advanced Sidecar（高级解析器插件）** 进行 PDF 页面转图片（JPEG/PNG），由 Rust 的 `VisionAdapter` 调度 Vision 模型进行推理，避免两端重复加载模型崩溃。
3. **优雅降级**：若未开启视觉解析或未安装高级 Sidecar 插件，对 PDF 解析时自动降级回轻量级的 `pdf_extract` 纯文本提取；对图片文件进行索引时提示用户在设置中开启该功能。

---

## 2. 详细交互设计 (UI/UX Specification)

### 2.1 设置项增加
在 `src/views/Models.vue` 的 **高级推理与检索参数 (advanced)** 页签中，新增一个独立的卡片：**“文档多模态视觉解析”**。

* **配置项键名**：`use_vision_parser` (SQLite 数据库 `settings` 表，默认为 `"false"`)。
* **开关状态控制逻辑**：
  * 定义计算属性 `isVisionConfigured`：检测是否存在 `settings.vision_model` 且存在 `settings.vision_mmproj`。
  * 若 `isVisionConfigured === false`：
    * 开关组件强制置灰禁用（Disabled）。
    * 下方显示高亮警告：“⚠️ 请先在『模型市场与本地管理』中配置视觉模型与视觉投影器 (mmproj)。”
  * 若 `isVisionConfigured === true`：
    * 开关组件可用，允许用户启用或关闭。
    * 下方显示提示：“💡 提示：使用多模态模型解析图片和 PDF 会显著降低文档索引速度，并占用较多 CPU/GPU 内存。”
* **自动关闭联动**：当 `isVisionConfigured` 变更为 `false`（例如用户删除了视觉模型路径配置）时，若当前 `use_vision_parser` 为 `"true"`，应自动将其写回 `"false"`。

---

## 3. 技术实现架构 (Technical Architecture)

### 3.1 总体数据流

```
用户点击“重新索引/导入文档”
        │
        ▼
后端: index_document(doc_id)
        │
        ▼
异步文档解析: parse_document_async(path)
        │
        ├──【格式为 TXT/MD/JSON/CSV】：
        │     └── 运行同步 core::parse_text_file
        │
        ├──【格式为 PNG/JPG/JPEG等图片】：
        │     ├── use_vision_parser == true => 调用本地 VisionModel 进行 OCR/图片理解
        │     └── use_vision_parser == false => 返回友好错误：“请在系统设置中启用视觉解析”
        │
        └──【格式为 PDF】：
              ├── use_vision_parser == true 且高级 Sidecar 已安装:
              │     ├── 1. 调用 Sidecar 将 PDF 逐页转为图片输出到 temp 目录
              │     ├── 2. 逐页调用本地 VisionModel 进行文字与版面提取
              │     └── 3. 拼接各页的 Markdown，清理临时文件并返回
              │
              └── 否则:
                    └── 降级调用 core::parse_pdf 纯文本提取
```

### 3.2 后端接口与文件变更

#### `src-tauri/src/services/parser/mod.rs`
* 新增异步文档解析入口函数：
  ```rust
  pub async fn parse_document_async(path: &Path, app_handle: &AppHandle) -> Result<String, AppError>
  ```
* 废弃原同步 `parse_file`。

#### `src-tauri/src/commands/indexing.rs`
* 修改 `index_document` 命令行函数，将原本同步的 `parser::parse_file` 改为调用异步的 `parser::parse_document_async`：
  ```rust
  let path = Path::new(&library_path);
  let text = parser::parse_document_async(path, &app_handle).await?;
  ```

#### `src-tauri/src/services/parser/vision.rs`
* 重构该文件，暴露以下两个核心解析函数：
  ```rust
  /// 使用本地 Vision 模型解析单张图片
  pub async fn parse_image_with_vision(path: &Path, app_handle: &AppHandle) -> Result<String, AppError>;

  /// 配合 Sidecar 渲染 PDF 页面为图片，并调用本地 Vision 模型进行逐页多模态提取
  pub async fn parse_pdf_with_vision(path: &Path, app_handle: &AppHandle) -> Result<String, AppError>;
  ```

#### `src-tauri/src/services/parser/sidecar.rs`
* 定义 Sidecar 的命令行交互协议：
  ```rust
  /// 调用 Sidecar 将 PDF 页面渲染为临时图片输出
  /// 运行命令: parser-advanced --render-pdf-pages <pdf_path> <temp_dir>
  pub async fn render_pdf_pages_via_sidecar(
      app_handle: &AppHandle,
      pdf_path: &Path,
      temp_dir: &Path
  ) -> Result<usize, AppError>;
  ```

---

## 4. 视觉解析 Prompts 与版面还原策略

### 4.1 单图片解析 Prompt
为了让 Vision 模型更精准地输出文档式结构，使用如下指令：
```
请将此图片中的所有文字内容提取出来。如果是表格，请完整保留 Markdown 表格格式；如果含有图表，请在适当位置用文字进行描述。请直接输出提取出的内容，不要有任何前导客套话或多余的解释。
```

### 4.2 PDF 逐页解析 Prompt 与分隔机制
在 PDF 视觉解析时，对每一页进行流式推理，Prompt 模板：
```
这是文档的第 {page_num} 页图片。请完整提取此页中的文字（如果是表格，请保留 Markdown 表格格式，如果含有图表请进行语义描述）。请直接输出提取出的内容，不要有任何前导客套话或多余的解释。
```
**合并规则**：每一页的输出文本被拼接进同一个大文本对象中，页与页之间使用如下分隔线，以便于 chunk 块切分时能够保留页码元数据：
```markdown


--- [第 {page_num} 页] ---

[页内文本...]
```

---

## 5. 验证方案 (Verification Plan)

1. **设置页面可用性与联动**：
   * 在未配置视觉模型时，验证 `use_vision_parser` 开关置灰且有错误红框提示。
   * 配置视觉模型后，开关解锁，显示警告提示；保存配置项并重新打开，状态保存正常。
   * 清除视觉模型配置后，`use_vision_parser` 自动变回关闭状态。
2. **单图片文件解析验证**：
   * 导入单张含有文本和表格的图片，在未开启开关时触发索引，验证返回明确的友好错误提示。
   * 开启开关后，验证模型成功被加载，图片压缩后被输入并返回还原的文本 and Markdown 表格。
3. **PDF 文件解析降级与视觉提取验证**：
   * 在未配置 Sidecar 或关闭开关时，导入扫描版 PDF，索引能正常完成（降级到 `pdf_extract`，此时由于无法提取图片文字，结果应为空或极少字符）。
   * 配置了 Sidecar 并开启视觉开关后，导入 PDF，验证系统在临时文件夹中生成了每页图片。
   * 验证 Rust 正确遍历了图片，逐页调用本地 Vision 推理引擎并按页拼接成最终文本。
   * 验证临时文件夹在解析成功或失败后都被正确物理删除。
