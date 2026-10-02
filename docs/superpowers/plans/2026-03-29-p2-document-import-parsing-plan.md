# P2: 文档导入与解析 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现一个高效、插件化的本地文档导入与解析系统，支持多种格式并具备图片深度理解能力。

**Architecture:** 采用 Managed Library 模式存储文件，Rust 后端调度解析任务。解析分为核心内置解析器 (Rust) 和可选的高级 Sidecar 插件 (Python)，视觉解析通过 Ollama API 完成。

**Tech Stack:** Tauri v2, SQLite, reqwest, pdf-extract, encoding_rs, Ollama API, Naive UI.

---

### Task 1: 数据库与文件管理基础

**Files:**
- Create: `src-tauri/src/db/documents.rs`
- Modify: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/services/storage.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: 定义文档数据库表结构**
在 `src-tauri/src/db/documents.rs` 中定义 `Document` 模型和初始化 SQL（`documents` 表）。

- [ ] **Step 2: 实现存储服务**
在 `src-tauri/src/services/storage.rs` 中实现 `clone_to_library` 函数，将文件从任意位置复制到应用数据目录下的 `library` 文件夹，并生成唯一文件名。

- [ ] **Step 3: 编写数据库与存储单元测试**
验证文件克隆是否成功，元数据是否正确存入 SQLite。

- [ ] **Step 4: 提交变更**
```bash
git add src-tauri/src/db/ src-tauri/src/services/storage.rs
git commit -m "feat(backend): implement document database schema and managed library storage"
```

---

### Task 2: 基础解析器实现 (Core Parser)

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/services/parser/mod.rs`
- Create: `src-tauri/src/services/parser/core.rs`

- [ ] **Step 1: 添加解析依赖**
在 `src-tauri/Cargo.toml` 中添加 `pdf-extract` 和 `encoding_rs`。

- [ ] **Step 2: 实现纯文本与 Markdown 解析**
在 `core.rs` 中实现对 `.txt`, `.md`, `.json` 的 UTF-8 及多种编码支持的提取逻辑。

- [ ] **Step 3: 实现基础 PDF 提取**
集成 `pdf-extract` 提取 PDF 文本内容。

- [ ] **Step 4: 编写解析器测试**
使用模拟文件测试不同格式的文本提取效果。

- [ ] **Step 5: 提交变更**
```bash
git add src-tauri/Cargo.toml src-tauri/src/services/parser/
git commit -m "feat(backend): implement core parsers for text, markdown and pdf"
```

---

### Task 3: 视觉模型集成 (Vision Engine)

**Files:**
- Create: `src-tauri/src/services/parser/vision.rs`
- Modify: `src-tauri/src/services/parser/mod.rs`

- [ ] **Step 1: 实现 Ollama 视觉请求逻辑**
在 `vision.rs` 中封装调用 Ollama `/api/chat` 接口的函数，支持 Base64 图片上传。

- [ ] **Step 2: 编写图片解析 Prompt**
设计用于提取文字并描述图片语义的提示词（系统提示词）。

- [ ] **Step 3: 集成至解析中心**
在 `parser/mod.rs` 中根据文件类型自动分发至 `VisionParser`。

- [ ] **Step 4: 提交变更**
```bash
git add src-tauri/src/services/parser/vision.rs
git commit -m "feat(backend): integrate ollama vision engine for image analysis"
```

---

### Task 4: 文档管理前端实现 (UI/UX)

**Files:**
- Create: `src/stores/documents.ts`
- Create: `src/views/Documents.vue`
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 创建文档 Store**
在 Pinia 中管理文档列表、导入状态和解析任务队列。

- [ ] **Step 2: 实现导入命令后端 Command**
在 Rust 中暴露 `import_document` command 并在 `src/api/tauri.ts` 中封装。

- [ ] **Step 3: 实现 Documents.vue UI**
使用 Naive UI 列表展示文档，并实现全屏拖拽区域。

- [ ] **Step 4: 提交变更**
```bash
git add src/stores/documents.ts src/views/Documents.vue src/api/tauri.ts
git commit -m "feat(frontend): implement document management UI and state store"
```

---

### Task 5: 插件化架构预留与最终集成

**Files:**
- Create: `src-tauri/src/services/parser/sidecar.rs`
- Modify: `src-tauri/src/views/Documents.vue`

- [ ] **Step 1: 实现 Sidecar 检测逻辑**
检查应用目录下是否存在 Python 运行时目录。

- [ ] **Step 2: 实现 UI 插件安装引导**
在文档列表中针对不支持的格式显示“下载高级解析插件”按钮。

- [ ] **Step 3: 完成解析任务流串联**
确保从导入到克隆再到异步解析的整个流程闭环。

- [ ] **Step 4: 提交变更**
```bash
git add .
git commit -m "feat(p2): complete modular parser architecture and final integration"
```
