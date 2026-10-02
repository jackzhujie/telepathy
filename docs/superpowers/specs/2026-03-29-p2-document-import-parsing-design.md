# P2: 文档导入与解析 - 设计文档

## 1. 项目背景 (Project Context)
Telepathy 作为一个本地智能知识库，需要具备强大的文档摄入能力。P2 阶段的目标是实现从本地文件到结构化文本的转换，为后续的向量化 (P3) 和检索问答 (P4) 提供高质量的数据源。

## 2. 核心目标 (Core Goals)
- **多格式支持**: 支持 Markdown, TXT, PDF, Word, Excel, PowerPoint 以及常见图片格式。
- **深度视觉理解**: 利用本地 Ollama 视觉模型解析图片和复杂 PDF。
- **插件化架构**: 保持应用体积轻量，高级解析功能作为可选插件下载。
- **本地优先管理**: 采用 Managed Library 模式，确保知识库的稳定性和独立性。

## 3. 架构设计 (Architecture)

### 3.1 插件化解析调度 (Modular Parser Dispatcher)
- **Core Parser (Built-in)**: 
  - 语言: Rust
  - 职责: 处理 `.md`, `.txt`, `.json` 以及基础流式 `.pdf`。
- **Advanced Sidecar (Optional)**:
  - 语言: Python (Bundled via Tauri Sidecar)
  - 职责: 处理 `.docx`, `.xlsx`, `.pptx` 以及复杂表格识别。
  - 获取方式: 用户在应用内点击“下载高级解析器”后动态安装。
- **Vision Engine (Local Ollama)**:
  - 协议: HTTP REST API
  - 模型: `qwen2-vl`, `llava` 或 `minicpm-v`。
  - 职责: 提取图片文字并生成图片语义描述。

### 3.2 文档存储模型 (Storage Model)
- **存储位置**: 用户数据目录下的 `library/` 文件夹。
- **数据库 (SQLite)**:
  - `documents` 表: 记录 `id`, `name`, `original_path`, `cloned_path`, `file_type`, `size`, `status` (Pending/Parsing/Success/Failed), `error_msg`, `created_at` 等元数据。

## 4. 界面设计 (UI/UX)
- **文档中心 (Document Center)**: 
  - 采用 Naive UI `n-data-table` 展示文档列表。
  - 支持批量选择、删除、重试解析。
- **交互方式**:
  - 支持全屏拖拽文件导入。
  - 顶部导航提供“解析任务队列”和“插件中心”。
- **状态反馈**:
  - 实时显示解析进度条。
  - 针对缺少插件或加密文件的明确错误提示。

## 5. 错误处理与性能 (Error Handling & Performance)
- **并发控制**: Rust 后端使用信号量限制同时运行的解析任务数量（默认 2-4）。
- **重试机制**: 
  - 手动单个/批量重试。
  - 安装高级解析器后自动重试失败任务。
- **大文件处理**: 针对超过 100MB 的文件进行性能提醒，并优先采用流式解析。

## 6. 验收标准 (Success Criteria)
- [ ] 成功导入并克隆文件至管理库。
- [ ] Markdown 和纯文本解析无误且速度极快。
- [ ] 图片能通过 Ollama 获取文本描述。
- [ ] 用户可以动态下载并启用高级解析插件（Sidecar）。
- [ ] 文档中心能实时反映解析状态。
