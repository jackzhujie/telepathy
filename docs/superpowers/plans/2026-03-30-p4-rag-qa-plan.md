# P4: RAG 检索与问答 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现完整的 RAG 管道：用户提问 → 向量检索 → 语义重排 → LLM 生成带引用的回答 → 流式输出 → 多轮对话。

**Architecture:** RAG 引擎串联 embedder、vector search、prompt assembly，通过 Ollama 流式生成回答。对话历史持久化到 SQLite。

**Tech Stack:** reqwest (Ollama API), rusqlite, serde_json, Tauri Events, Naive UI, Vue 3

---

### Task 1: 对话管理数据库

**Files:**
- Create: `src-tauri/src/db/conversations.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: 创建对话表结构**

在 `src-tauri/src/db/conversations.rs` 中定义:

```rust
#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sources: Option<String>,
    pub created_at: String,
}

pub fn init_conversations_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            sources TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);"
    )
    .map_err(|e| AppError::Internal(format!("Failed to create conversations tables: {}", e)))?;
    Ok(())
}

pub fn create_conversation(conn: &Connection, id: &str, title: Option<&str>) -> Result<Conversation, AppError> {
    conn.execute(
        "INSERT INTO conversations (id, title) VALUES (?1, ?2)",
        params![id, title],
    ).map_err(|e| AppError::Internal(format!("Failed to create conversation: {}", e)))?;
    Ok(Conversation {
        id: id.to_string(),
        title: title.map(|s| s.to_string()),
        created_at: String::new(),
    })
}

pub fn get_conversations(conn: &Connection) -> Result<Vec<Conversation>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, title, created_at FROM conversations ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;
    let rows = stmt.query_map([], |row| {
        Ok(Conversation { id: row.get(0)?, title: row.get(1)?, created_at: row.get(2)? })
    }).map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_messages(conn: &Connection, conv_id: &str) -> Result<Vec<ChatMessage>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, conversation_id, role, content, sources, created_at FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;
    let rows = stmt.query_map(params![conv_id], |row| {
        Ok(ChatMessage {
            id: row.get(0)?, conversation_id: row.get(1)?, role: row.get(2)?,
            content: row.get(3)?, sources: row.get(4)?, created_at: row.get(5)?,
        })
    }).map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn insert_message(conn: &Connection, msg: &ChatMessage) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, sources, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![msg.id, msg.conversation_id, msg.role, msg.content, msg.sources, msg.created_at],
    ).map_err(|e| AppError::Internal(format!("Failed to insert message: {}", e)))?;
    Ok(())
}

pub fn delete_conversation(conn: &Connection, conv_id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM messages WHERE conversation_id = ?1", params![conv_id])
        .map_err(|e| AppError::Internal(format!("Failed to delete messages: {}", e)))?;
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![conv_id])
        .map_err(|e| AppError::Internal(format!("Failed to delete conversation: {}", e)))?;
    Ok(())
}
```

- [ ] **Step 2: 更新 db 模块导出**

在 `src-tauri/src/db/mod.rs` 中添加 `pub mod conversations;`

- [ ] **Step 3: 编译验证 + 测试**

```bash
cd src-tauri && cargo check && cargo test
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/db/conversations.rs src-tauri/src/db/mod.rs
git commit -m "feat(backend): add conversation and message storage for multi-turn RAG"
```

---

### Task 2: RAG 引擎与 Prompt 组装

**Files:**
- Create: `src-tauri/src/services/rag.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: 实现 RAG 引擎**

在 `src-tauri/src/services/rag.rs` 中实现:

```rust
#![allow(dead_code)]
use crate::db::{conversations::ChatMessage, vectors::{self, ChunkWithScore}};
use crate::errors::AppError;
use crate::services::embedder::Embedder;
use rusqlite::Connection;
use serde::Serialize;

const SIMILARITY_THRESHOLD: f32 = 0.3;
const DEFAULT_TOP_K: usize = 5;

#[derive(Debug, Serialize, Clone)]
pub struct SearchSource {
    pub chunk_id: String,
    pub document_name: String,
    pub chunk_index: i32,
    pub content: String,
    pub score: f32,
}

pub fn search_context(
    query: &str,
    conn: &Connection,
    embedder: &Embedder,
    rt: &tokio::runtime::Runtime,
    top_k: usize,
) -> Result<Vec<SearchSource>, AppError> {
    let query_embedding = rt.block_on(embedder.embed(query))?;
    let results = vectors::search_similar(conn, &query_embedding, top_k * 2)?;

    let sources: Vec<SearchSource> = results
        .into_iter()
        .filter(|r| r.score >= SIMILARITY_THRESHOLD)
        .take(top_k)
        .map(|r| SearchSource {
            chunk_id: r.chunk.id.clone(),
            document_name: r.chunk.document_id.clone(),
            chunk_index: r.chunk.chunk_index,
            content: r.chunk.content,
            score: r.score,
        })
        .collect();

    Ok(sources)
}

pub fn build_rag_prompt(
    query: &str,
    sources: &[SearchSource],
    history: &[ChatMessage],
) -> String {
    let mut prompt = String::new();

    prompt.push_str("你是一个智能知识库助手。请基于以下检索到的文档内容回答用户问题。\n\n");

    if !sources.is_empty() {
        prompt.push_str("## 检索到的文档内容\n\n");
        for (i, source) in sources.iter().enumerate() {
            prompt.push_str(&format!(
                "[来源: {}, 块 #{}]\n{}\n\n",
                source.document_name,
                source.chunk_index + 1,
                source.content
            ));
        }
    }

    prompt.push_str("## 回答要求\n");
    prompt.push_str("1. 优先使用检索到的文档内容回答问题\n");
    prompt.push_str("2. 在相关段落末尾标注引用来源，格式：[来源: 文件名, 块 #N]\n");
    prompt.push_str("3. 如果检索到的内容无法完全回答问题，可以结合你的通用知识补充，但需标注[通用知识]\n");
    prompt.push_str("4. 如果完全没有检索到相关内容，直接用你的知识回答，标注[通用知识]\n\n");

    if !history.is_empty() {
        prompt.push_str("## 对话历史\n\n");
        for msg in history.iter().rev().take(6) {
            let role_label = if msg.role == "user" { "用户" } else { "助手" };
            prompt.push_str(&format!("{}: {}\n\n", role_label, msg.content));
        }
    }

    prompt.push_str(&format!("## 用户问题\n\n{}\n", query));

    prompt
}

pub fn build_no_context_prompt(query: &str, history: &[ChatMessage]) -> String {
    let mut prompt = String::new();
    prompt.push_str("你是一个智能助手。用户的问题在知识库中没有找到相关内容，请基于你的通用知识回答。\n");
    prompt.push_str("请在回答末尾标注[通用知识]。\n\n");

    if !history.is_empty() {
        prompt.push_str("## 对话历史\n\n");
        for msg in history.iter().rev().take(6) {
            let role_label = if msg.role == "user" { "用户" } else { "助手" };
            prompt.push_str(&format!("{}: {}\n\n", role_label, msg.content));
        }
    }

    prompt.push_str(&format!("## 用户问题\n\n{}\n", query));
    prompt
}

pub fn format_sources_for_storage(sources: &[SearchSource]) -> String {
    serde_json::to_string(sources).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rag_prompt() {
        let sources = vec![SearchSource {
            chunk_id: "c1".into(),
            document_name: "test.txt".into(),
            chunk_index: 0,
            content: "Hello world".into(),
            score: 0.9,
        }];
        let prompt = build_rag_prompt("What is this?", &sources, &[]);
        assert!(prompt.contains("test.txt"));
        assert!(prompt.contains("Hello world"));
        assert!(prompt.contains("What is this?"));
    }

    #[test]
    fn test_build_no_context_prompt() {
        let prompt = build_no_context_prompt("Unknown?", &[]);
        assert!(prompt.contains("通用知识"));
        assert!(prompt.contains("Unknown?"));
    }
}
```

- [ ] **Step 2: 更新 services 模块导出**

在 `src-tauri/src/services/mod.rs` 中添加 `pub mod rag;`

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/services/rag.rs src-tauri/src/services/mod.rs
git commit -m "feat(backend): implement RAG engine with prompt assembly and source citation"
```

---

### Task 3: RAG 查询 Command（流式）

**Files:**
- Create: `src-tauri/src/commands/rag.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 实现 RAG 查询命令**

在 `src-tauri/src/commands/rag.rs` 中实现流式 RAG 查询:

```rust
use crate::db::{conversations, documents, vectors};
use crate::errors::AppError;
use crate::services::{embedder, rag};
use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

#[derive(Serialize)]
pub struct RagSourcesPayload {
    pub sources: Vec<rag::SearchSource>,
}

#[tauri::command]
pub async fn rag_query(
    query: String,
    conversation_id: Option<String>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    // Ensure tables exist
    conversations::init_conversations_table(&conn)?;
    vectors::init_vector_tables(&conn)?;

    // Create or use conversation
    let conv_id = match conversation_id {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            let title = if query.len() > 20 { Some(&query[..20]) } else { Some(query.as_str()) };
            conversations::create_conversation(&conn, &id, title)?;
            id
        }
    };

    // Get conversation history
    let history = conversations::get_messages(&conn, &conv_id)?;

    // Search context
    let embedder = embedder::Embedder::new("bge-large-zh");
    let sources = rag::search_context(&query, &conn, &embedder, &tokio::runtime::Handle::current(), 5)?;

    // Build prompt
    let prompt = if sources.is_empty() {
        rag::build_no_context_prompt(&query, &history)
    } else {
        // Emit sources to frontend
        let _ = app_handle.emit("rag-sources", RagSourcesPayload { sources: sources.clone() });
        rag::build_rag_prompt(&query, &sources, &history)
    };

    // Save user message
    let user_msg = conversations::ChatMessage {
        id: Uuid::new_v4().to_string(),
        conversation_id: conv_id.clone(),
        role: "user".to_string(),
        content: query,
        sources: None,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    conversations::insert_message(&conn, &user_msg)?;

    // Stream LLM response
    let sources_json = if sources.is_empty() { None } else { Some(rag::format_sources_for_storage(&sources)) };

    let assistant_msg_id = Uuid::new_v4().to_string();
    let conv_id_clone = conv_id.clone();
    let app_handle_clone = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        use crate::services::ollama;
        use tauri::Emitter;

        // Call Ollama with the RAG prompt
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "model": "qwen2.5",
            "messages": [
                { "role": "user", "content": prompt }
            ],
            "stream": true
        });

        match client.post("http://localhost:11434/api/chat").json(&payload).send().await {
            Ok(resp) => {
                let mut stream = resp.bytes_stream();
                let mut full_content = String::new();
                let mut buffer = Vec::new();

                while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
                    if let Ok(bytes) = chunk {
                        buffer.extend_from_slice(&bytes);
                        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                            let line_bytes = buffer.drain(..=pos).collect::<Vec<u8>>();
                            if let Ok(text) = String::from_utf8(line_bytes) {
                                let trimmed = text.trim();
                                if trimmed.is_empty() { continue; }
                                if let Ok(resp) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                    if let Some(content) = resp.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                                        full_content.push_str(content);
                                        let _ = app_handle_clone.emit("chat-token", serde_json::json!({ "token": content }));
                                    }
                                    if resp.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                                        let _ = app_handle_clone.emit("chat-done", ());
                                    }
                                }
                            }
                        }
                    }
                }

                // Save assistant message
                let db_path = get_db_path(&app_handle_clone).unwrap_or_default();
                if let Ok(conn) = Connection::open(&db_path) {
                    let msg = conversations::ChatMessage {
                        id: assistant_msg_id,
                        conversation_id: conv_id_clone,
                        role: "assistant".to_string(),
                        content: full_content,
                        sources: sources_json,
                        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    };
                    let _ = conversations::insert_message(&conn, &msg);
                }
            }
            Err(e) => {
                let _ = app_handle_clone.emit("chat-error", serde_json::json!({ "error": format!("Ollama error: {}", e) }));
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn get_conversations(app_handle: AppHandle) -> Result<Vec<conversations::Conversation>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    conversations::init_conversations_table(&conn)?;
    conversations::get_conversations(&conn)
}

#[tauri::command]
pub async fn get_messages(
    conversation_id: String,
    app_handle: AppHandle,
) -> Result<Vec<conversations::ChatMessage>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    conversations::init_conversations_table(&conn)?;
    conversations::get_messages(&conn, &conversation_id)
}

#[tauri::command]
pub async fn delete_conversation_cmd(
    conversation_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    conversations::delete_conversation(&conn, &conversation_id)
}

fn get_db_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
    }
    Ok(app_data_dir.join("telepathy.db"))
}
```

- [ ] **Step 2: 注册命令**

更新 `src-tauri/src/commands/mod.rs` 添加 `pub mod rag;`
更新 `src-tauri/src/lib.rs` invoke_handler:
```rust
commands::rag::rag_query,
commands::rag::get_conversations,
commands::rag::get_messages,
commands::rag::delete_conversation_cmd,
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/rag.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(backend): implement RAG query command with streaming and conversation management"
```

---

### Task 4: 前端 Chat 页面重构

**Files:**
- Modify: `src/views/Chat.vue`
- Modify: `src/api/tauri.ts`
- Modify: `src/types/chat.ts`

- [ ] **Step 1: 添加 RAG 相关类型**

在 `src/types/chat.ts` 中添加:

```typescript
export interface SearchSource {
  chunk_id: string;
  document_name: string;
  chunk_index: number;
  content: string;
  score: number;
}

export interface RagSourcesPayload {
  sources: SearchSource[];
}

export interface Conversation {
  id: string;
  title: string | null;
  created_at: string;
}

export interface ChatMessage {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  sources: string | null;
  created_at: string;
}
```

- [ ] **Step 2: 添加 RAG API 封装**

在 `src/api/tauri.ts` 中添加:

```typescript
export async function ragQuery(query: string, conversationId?: string): Promise<void> {
  return invoke<void>('rag_query', { query, conversationId });
}

export async function getConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>('get_conversations');
}

export async function getMessages(conversationId: string): Promise<ChatMessage[]> {
  return invoke<ChatMessage[]>('get_messages', { conversationId });
}

export async function deleteConversation(conversationId: string): Promise<void> {
  return invoke<void>('delete_conversation_cmd', { conversationId });
}
```

- [ ] **Step 3: 重构 Chat.vue**

重写 Chat.vue 支持：
- 左侧对话历史列表（可折叠）
- 右侧对话区域（流式响应 + 引用来源展示）
- 底部输入区域（Enter 发送）
- 复用 P1 的事件监听（chat-token, chat-done, chat-error）
- 新增 rag-sources 事件监听

- [ ] **Step 4: 编译验证**

```bash
cd /Users/mac/project/telepathy && npx vue-tsc --noEmit
```

- [ ] **Step 5: 提交**

```bash
git add src/views/Chat.vue src/api/tauri.ts src/types/chat.ts
git commit -m "feat(frontend): refactor Chat page with RAG support and conversation history"
```

---

### Task 5: 最终集成与冒烟测试

- [ ] **Step 1: 全量编译检查**

```bash
cd src-tauri && cargo check
cd /Users/mac/project/telepathy && npx vue-tsc --noEmit
```

- [ ] **Step 2: 运行全部测试**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 3: 提交最终状态**

```bash
git add -A && git commit -m "feat(p4): complete RAG pipeline integration"
```
