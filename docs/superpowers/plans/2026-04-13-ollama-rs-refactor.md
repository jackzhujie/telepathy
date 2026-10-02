# ollama-rs 迁移与重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 使用 `ollama-rs` 库全面重构 Telepathy 的 AI 交互层，统一模型管理逻辑，提升代码健壮性。

**Architecture:** 
1. 引入 `ollama-rs` 依赖。
2. 创建 `services/ollama.rs` 统一管理客户端实例。
3. 按照从外围（Embedder、模型管理）到核心（RAG 对话）的顺序进行重构。
4. 统一模型拉取逻辑，移除冗余的命令行执行代码。

**Tech Stack:** Rust, tauri v2, ollama-rs, tokio, rusqlite.

---

### Task 1: 基础设施与依赖准备

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/services/ollama.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: 添加 `ollama-rs` 依赖**
修改 `src-tauri/Cargo.toml`，添加：
```toml
ollama-rs = { version = "0.2.1", features = ["stream"] }
```

- [ ] **Step 2: 创建 `Ollama` 服务中心**
创建 `src-tauri/src/services/ollama.rs`，实现统一初始化：
```rust
use ollama_rs::Ollama;
use tauri::AppHandle;
use crate::db::settings;
use crate::errors::AppError;

pub async fn get_ollama_client(app_handle: &AppHandle) -> Result<Ollama, AppError> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    let db_path = app_data_dir.join("telepathy.db");
    
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    
    let ollama_url = settings::get_setting(&conn, "ollama_url")?
        .unwrap_or_else(|| "http://localhost:11434".to_string());
        
    // 解析 URL
    let url = url::Url::parse(&ollama_url)
        .map_err(|e| AppError::Internal(format!("Invalid Ollama URL: {}", e)))?;
        
    let host = url.host_str().unwrap_or("localhost").to_string();
    let port = url.port().unwrap_or(11434);
    let protocol = url.scheme().to_string();
    
    Ok(Ollama::new(format!("{}://{}", protocol, host), port))
}
```

- [ ] **Step 3: 导出模块**
在 `src-tauri/src/services/mod.rs` 中添加 `pub mod ollama;`。

- [ ] **Step 4: 编译验证**
运行 `cargo check` 确保依赖和新文件无误。

- [ ] **Step 5: Commit**
```bash
git add src-tauri/Cargo.toml src-tauri/src/services/ollama.rs src-tauri/src/services/mod.rs
git commit -m "refactor: add ollama-rs dependency and infrastructure service"
```

---

### Task 2: 重构 Embedder (向量化)

**Files:**
- Modify: `src-tauri/src/services/embedder.rs`

- [ ] **Step 1: 移除手动请求代码，改用 `ollama-rs`**
更新 `Embedder` 结构体和 `embed` 方法：
```rust
// ... 引入 ollama_rs 类型
impl Embedder {
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let ollama = crate::services::ollama::get_ollama_client(&self.app_handle).await?; // 需要传入 app_handle
        let res = ollama.generate_embeddings(self.model.clone(), text.to_string(), None).await
            .map_err(|e| AppError::Internal(format!("Ollama embedding error: {}", e)))?;
        Ok(res.embeddings.iter().map(|&x| x as f32).collect())
    }
}
```
*注意：Embedder 可能需要调整以持有 AppHandle 或在调用时传入。*

- [ ] **Step 2: 单元测试验证**
运行模型向量化测试。

- [ ] **Step 3: Commit**
```bash
git add src-tauri/src/services/embedder.rs
git commit -m "refactor: use ollama-rs for text embeddings"
```

---

### Task 3: 重构模型拉取与安装 (统一逻辑)

**Files:**
- Modify: `src-tauri/src/commands/models.rs`
- Modify: `src-tauri/src/commands/installer.rs`

- [ ] **Step 1: 在 `models.rs` 中实现统一拉取**
使用 `ollama.pull_model_stream` 替换原来的手动请求。

- [ ] **Step 2: 废弃 `installer::pull_model`**
将其标记为废弃或移除其命令行执行部分，引导前端调用 `models::install_model`。

- [ ] **Step 3: 验证进度流**
确保前端能收到 `model-pull-progress` 事件。

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/commands/models.rs src-tauri/src/commands/installer.rs
git commit -m "refactor: unify model pull logic using ollama-rs stream"
```

---

### Task 4: 重构 RAG 对话 (Chat)

**Files:**
- Modify: `src-tauri/src/commands/rag.rs`

- [ ] **Step 1: 简化流式对话代码**
移除 `while let Some(chunk) = stream.next().await` 中繁琐的文本解析逻辑。
示例：
```rust
let request = ChatMessageRequest::new(model, messages).options(options);
let mut stream = ollama.send_chat_messages_stream(request).await?;
while let Some(res) = stream.next().await {
    // 处理 res.message.content 即可
}
```

- [ ] **Step 2: 整合聊天参数**
将数据库读取的 `temperature` 等参数映射到 `GenerationOptions`。

- [ ] **Step 3: 测试完整对话流**
验证 RAG 问答是否依然正常，且响应速度和流式输出无卡顿。

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/commands/rag.rs
git commit -m "refactor: simplify RAG chat streaming with ollama-rs"
```

---

### Task 5: 清理与优化

- [ ] **Step 1: 扫描无用代码**
移除不再需要的 `reqwest` 手动结构体定义（如 `OllamaTagsResponse`）。

- [ ] **Step 2: 最终回归测试**
确保所有功能（文档上传、向量化对比、对话、模型下载）均正常。

- [ ] **Step 3: Commit**
```bash
git commit -m "refactor: final cleanup post ollama-rs migration"
```
