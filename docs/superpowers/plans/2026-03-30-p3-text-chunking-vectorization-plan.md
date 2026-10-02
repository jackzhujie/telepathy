# P3: 文本分块与向量化 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现语义分块和本地嵌入向量化，将解析后的文本转换为可检索的向量索引。

**Architecture:** 使用段落级语义分块引擎切分文本，通过 Ollama Embeddings API 调用专用嵌入模型生成向量，存储到 SQLite + sqlite-vss 扩展中。

**Tech Stack:** rusqlite (bundled + vss), reqwest, serde_json, tokio, Naive UI

---

### Task 1: SQLite 向量扩展集成

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/db/vectors.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: 添加 sqlite-vss 依赖**

在 `src-tauri/Cargo.toml` 中确认 `rusqlite` 已启用 `bundled` feature，然后添加 `sqlite-vss` 支持。

```toml
# Cargo.toml - 确认已有
rusqlite = { version = "0.31", features = ["bundled"] }

# sqlite-vss 通过 rusqlite 的 loadable extension 支持
```

- [ ] **Step 2: 创建向量数据库模块**

在 `src-tauri/src/db/vectors.rs` 中实现向量表初始化和查询操作：

```rust
#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

pub fn init_vector_tables(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunks (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            content TEXT NOT NULL,
            metadata TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create chunks table: {}", e)))?;

    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS chunk_vectors USING vss0(embedding(768))"
    )
    .map_err(|e| AppError::Internal(format!("Failed to create vector table: {}", e)))?;

    Ok(())
}

pub fn insert_chunk(conn: &Connection, chunk: &Chunk) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO chunks (id, document_id, chunk_index, content, metadata, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            chunk.id,
            chunk.document_id,
            chunk.chunk_index,
            chunk.content,
            chunk.metadata,
            chunk.created_at,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert chunk: {}", e)))?;
    Ok(())
}

pub fn get_chunks_by_document(conn: &Connection, doc_id: &str) -> Result<Vec<Chunk>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, document_id, chunk_index, content, metadata, created_at FROM chunks WHERE document_id = ?1 ORDER BY chunk_index")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map(rusqlite::params![doc_id], |row| {
            Ok(Chunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_index: row.get(2)?,
                content: row.get(3)?,
                metadata: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query chunks: {}", e)))?;

    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }
    Ok(chunks)
}

pub fn insert_chunk_vector(
    conn: &Connection,
    chunk_id: &str,
    embedding: &[f32],
) -> Result<(), AppError> {
    let embedding_json = serde_json::to_string(embedding)
        .map_err(|e| AppError::Internal(format!("Failed to serialize embedding: {}", e)))?;

    conn.execute(
        "INSERT OR REPLACE INTO chunk_vectors (rowid, embedding)
         VALUES ((SELECT rowid FROM chunks WHERE id = ?1), ?2)",
        rusqlite::params![chunk_id, embedding_json],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert vector: {}", e)))?;
    Ok(())
}

pub fn delete_chunks_by_document(conn: &Connection, doc_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM chunk_vectors WHERE rowid IN (SELECT rowid FROM chunks WHERE document_id = ?1)",
        rusqlite::params![doc_id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to delete vectors: {}", e)))?;

    conn.execute(
        "DELETE FROM chunks WHERE document_id = ?1",
        rusqlite::params![doc_id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to delete chunks: {}", e)))?;

    Ok(())
}
```

- [ ] **Step 3: 更新 db 模块导出**

在 `src-tauri/src/db/mod.rs` 中添加 `pub mod vectors;`

- [ ] **Step 4: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/db/vectors.rs src-tauri/src/db/mod.rs src-tauri/Cargo.toml
git commit -m "feat(backend): add sqlite vector storage with chunks and embeddings tables"
```

---

### Task 2: 语义分块引擎

**Files:**
- Create: `src-tauri/src/services/chunker.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: 实现语义分块逻辑**

在 `src-tauri/src/services/chunker.rs` 中实现段落级语义分块：

```rust
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub min_length: usize,
    pub max_length: usize,
    pub merge_threshold: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            min_length: 50,
            max_length: 2000,
            merge_threshold: 500,
        }
    }
}

pub fn chunk_text(text: &str, config: &ChunkConfig) -> Vec<String> {
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut chunks = Vec::new();
    let mut buffer = String::new();

    for paragraph in paragraphs {
        if buffer.is_empty() {
            buffer = paragraph.to_string();
        } else if buffer.len() + paragraph.len() + 2 <= config.merge_threshold {
            buffer.push_str("\n\n");
            buffer.push_str(paragraph);
        } else {
            if buffer.len() > config.max_length {
                chunks.extend(split_long_chunk(&buffer, config.max_length));
            } else if buffer.len() >= config.min_length {
                chunks.push(buffer.clone());
            }
            buffer = paragraph.to_string();
        }
    }

    if !buffer.is_empty() {
        if buffer.len() > config.max_length {
            chunks.extend(split_long_chunk(&buffer, config.max_length));
        } else if buffer.len() >= config.min_length {
            chunks.push(buffer);
        }
    }

    chunks
}

fn split_long_chunk(text: &str, max_length: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let sentences: Vec<&str> = text
        .split(|c| c == '。' || c == '.' || c == '！' || c == '!' || c == '？' || c == '?')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut buffer = String::new();
    for sentence in sentences {
        if buffer.is_empty() {
            buffer = sentence.to_string();
        } else if buffer.len() + sentence.len() + 1 <= max_length {
            buffer.push('。');
            buffer.push_str(sentence);
        } else {
            chunks.push(buffer.clone());
            buffer = sentence.to_string();
        }
    }
    if !buffer.is_empty() {
        chunks.push(buffer);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_chunking() {
        let text = "第一段内容。\n\n第二段内容。\n\n第三段内容。";
        let config = ChunkConfig::default();
        let chunks = chunk_text(text, &config);
        assert!(!chunks.is_empty());
        assert!(chunks.iter().all(|c| c.len() >= config.min_length));
    }

    #[test]
    fn test_merge_short_paragraphs() {
        let text = "短文1\n\n短文2\n\n短文3";
        let config = ChunkConfig {
            min_length: 5,
            ..Default::default()
        };
        let chunks = chunk_text(text, &config);
        assert!(chunks.len() <= 3);
    }

    #[test]
    fn test_split_long_chunk() {
        let long_text = "这是一个很长的句子。" .repeat(100);
        let config = ChunkConfig::default();
        let chunks = chunk_text(&long_text, &config);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.len() <= config.max_length));
    }
}
```

- [ ] **Step 2: 更新 services 模块导出**

在 `src-tauri/src/services/mod.rs` 中添加 `pub mod chunker;`

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/services/chunker.rs src-tauri/src/services/mod.rs
git commit -m "feat(backend): implement semantic text chunking engine"
```

---

### Task 3: 向量化引擎

**Files:**
- Create: `src-tauri/src/services/embedder.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: 实现 Ollama Embeddings API 客户端**

在 `src-tauri/src/services/embedder.rs` 中实现向量嵌入：

```rust
#![allow(dead_code)]
use crate::errors::AppError;
use reqwest::Client;
use serde_json::json;

pub struct Embedder {
    client: Client,
    model: String,
    base_url: String,
}

impl Embedder {
    pub fn new(model: &str) -> Self {
        Self {
            client: Client::new(),
            model: model.to_string(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let payload = json!({
            "model": self.model,
            "prompt": text
        });

        let response = self
            .client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to Ollama: {}", e)))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse response: {}", e)))?;

        let embedding = body
            .get("embedding")
            .and_then(|e| e.as_array())
            .ok_or_else(|| AppError::Internal("No embedding in response".to_string()))?;

        let vector: Vec<f32> = embedding
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        if vector.is_empty() {
            return Err(AppError::Internal("Empty embedding returned".to_string()));
        }

        Ok(vector)
    }

    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    pub async fn check_model_available(&self) -> Result<bool, AppError> {
        let response = self
            .client
            .post(format!("{}/api/show", self.base_url))
            .json(&json!({ "name": self.model }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to Ollama: {}", e)))?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedder_creation() {
        let embedder = Embedder::new("bge-large-zh");
        assert_eq!(embedder.model, "bge-large-zh");
        assert_eq!(embedder.base_url, "http://localhost:11434");
    }
}
```

- [ ] **Step 2: 更新 services 模块导出**

在 `src-tauri/src/services/mod.rs` 中添加 `pub mod embedder;`

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/services/embedder.rs src-tauri/src/services/mod.rs
git commit -m "feat(backend): implement Ollama embeddings API client"
```

---

### Task 4: 索引命令与数据库集成

**Files:**
- Create: `src-tauri/src/commands/indexing.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/document.rs`

- [ ] **Step 1: 创建索引命令**

在 `src-tauri/src/commands/indexing.rs` 中实现：

```rust
use crate::db::{documents, vectors};
use crate::errors::AppError;
use crate::services::{chunker, embedder};
use std::path::Path;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct IndexResult {
    pub chunk_count: usize,
    pub document_id: String,
}

#[tauri::command]
pub async fn index_document(
    doc_id: String,
    app_handle: AppHandle,
) -> Result<IndexResult, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&conn)?;
    vectors::init_vector_tables(&conn)?;

    let doc = documents::get_document(&conn, &doc_id)?
        .ok_or_else(|| AppError::Internal("Document not found".to_string()))?;

    vectors::delete_chunks_by_document(&conn, &doc_id)?;

    let text = std::fs::read_to_string(Path::new(&doc.library_path))
        .map_err(|e| AppError::Internal(format!("Failed to read document: {}", e)))?;

    let config = chunker::ChunkConfig::default();
    let text_chunks = chunker::chunk_text(&text, &config);

    let model = "bge-large-zh";
    let embedder = embedder::Embedder::new(model);

    for (i, chunk_content) in text_chunks.iter().enumerate() {
        let chunk = vectors::Chunk {
            id: Uuid::new_v4().to_string(),
            document_id: doc_id.clone(),
            chunk_index: i as i32,
            content: chunk_content.clone(),
            metadata: None,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        vectors::insert_chunk(&conn, &chunk)?;

        if let Ok(embedding) = embedder.embed(chunk_content).await {
            vectors::insert_chunk_vector(&conn, &chunk.id, &embedding)?;
        }
    }

    documents::update_document_status(&conn, &doc_id, "indexed", None)?;

    Ok(IndexResult {
        chunk_count: text_chunks.len(),
        document_id: doc_id,
    })
}

#[tauri::command]
pub async fn get_document_chunks(
    doc_id: String,
    app_handle: AppHandle,
) -> Result<Vec<vectors::Chunk>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    vectors::init_vector_tables(&conn)?;
    vectors::get_chunks_by_document(&conn, &doc_id)
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

更新 `src-tauri/src/commands/mod.rs`：
```rust
pub mod chat;
pub mod document;
pub mod greet;
pub mod indexing;
```

更新 `src-tauri/src/lib.rs` 的 invoke_handler：
```rust
commands::indexing::index_document,
commands::indexing::get_document_chunks,
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: 运行测试**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/commands/indexing.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(backend): add document indexing commands with chunking and embedding"
```

---

### Task 5: 前端分块预览与索引 UI

**Files:**
- Modify: `src/api/tauri.ts`
- Modify: `src/stores/documents.ts`
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 添加前端 API 封装**

在 `src/api/tauri.ts` 中添加：
```typescript
export async function indexDocument(docId: string): Promise<IndexResult> {
  return invoke<IndexResult>('index_document', { docId });
}

export async function getDocumentChunks(docId: string): Promise<Chunk[]> {
  return invoke<Chunk[]>('get_document_chunks', { docId });
}
```

- [ ] **Step 2: 添加类型定义**

在 `src/types/document.ts` 中添加：
```typescript
export interface Chunk {
  id: string;
  document_id: string;
  chunk_index: number;
  content: string;
  metadata: string | null;
  created_at: string;
}

export interface IndexResult {
  chunk_count: number;
  document_id: string;
}
```

- [ ] **Step 3: 更新文档 Store**

在 `src/stores/documents.ts` 中添加索引操作：
```typescript
async function indexDocument(docId: string) {
  const doc = documents.value.find((d) => d.id === docId);
  if (doc) doc.status = 'parsing';
  try {
    const result = await indexDoc(docId);
    if (doc) doc.status = 'indexed';
    return result;
  } catch (e) {
    if (doc) {
      doc.status = 'error';
      doc.error_msg = String(e);
    }
    throw e;
  }
}
```

- [ ] **Step 4: 更新 Documents.vue**

在操作列中添加"索引"按钮，支持对已解析文档执行索引操作。

- [ ] **Step 5: 编译验证**

```bash
cd /Users/mac/project/telepathy && npx vue-tsc --noEmit
```

- [ ] **Step 6: 提交**

```bash
git add src/api/tauri.ts src/types/document.ts src/stores/documents.ts src/views/Documents.vue
git commit -m "feat(frontend): add document indexing UI and chunk preview"
```
