# ollama-rs 迁移收尾计划 (Settings & Vision)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 彻底移除 `src-tauri` 中残留的手动 Ollama API 调用，统一使用 `ollama-rs` SDK。

**Architecture:** 
1. 重构 `settings.rs` 以使用 `ollama-rs` 获取模型列表。
2. 重构 `vision.rs` 以使用 `ollama-rs` 进行视觉解析，并调整调用链以传递 `AppHandle`。
3. 清理不再需要的手动 JSON 结构体和 `reqwest` 调用。

**Tech Stack:** Rust, tauri v2, ollama-rs.

---

### Task 1: 重构 Settings 模块

**Files:**
- Modify: `src-tauri/src/commands/settings.rs`

- [ ] **Step 1: 移除手动 API 调用逻辑**
修改 `list_ollama_models` 函数，移除 `reqwest::Client` 和手动 URL 构造。使用 `ollama::get_ollama_client` 获取客户端，并调用 `list_local_models`。

- [ ] **Step 2: 删除冗余结构体**
删除 `OllamaModelsResponse` 和 `OllamaModel` 结构体定义。

- [ ] **Step 3: 编译验证**
运行 `cargo check`。

---

### Task 2: 重构 Vision Parser (接口调整)

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`
- Modify: `src-tauri/src/services/parser/mod.rs`
- Modify: `src-tauri/src/commands/document.rs`

- [ ] **Step 1: 修改 `vision.rs` 中的 `parse_image` 签名**
使其接受 `app_handle: &AppHandle`：
```rust
pub async fn parse_image(path: &Path, app_handle: &AppHandle) -> Result<String, AppError>
```

- [ ] **Step 2: 更新 `parser/mod.rs` 转发函数**
更新 `parse_image_async` 以接受并传递 `AppHandle`。

- [ ] **Step 3: 更新 `document.rs` 调用处**
在 `parse_document` 命令中传入 `app_handle`。

---

### Task 4: 重构 Vision Parser (核心逻辑)

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`

- [ ] **Step 1: 使用 `ollama-rs` 发送视觉请求**
使用 `client.send_chat_messages`。构造 `ChatMessage` 时使用 `images` 字段添加 base64 数据。示例：
```rust
let message = ChatMessage::user(prompt.to_string()).add_images(vec![base64_image]);
```

- [ ] **Step 2: 移除旧的手动逻辑**
删除 `is_ollama_running` 和 `start_ollama`（这些逻辑应由 `installer.rs` 或全局服务负责，parser 模块应专注于解析），或者将其改为通过 `ollama-rs` 探测。

- [ ] **Step 3: 最终清理**
移除 `vision.rs` 中不再使用的 `reqwest`, `serde_json` 引用。

---

### Task 5: 最终验证

- [ ] **Step 1: 全局搜索检查**
运行 `grep -r "/api/" src-tauri/src`，确认无手动 URL。

- [ ] **Step 2: 编译并测试**
确保项目能够正常运行且模型列表预览、图片解析功能正常。
