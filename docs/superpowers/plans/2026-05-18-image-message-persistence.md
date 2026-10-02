# 聊天图片消息持久化实现计划 (Image Message Persistence Plan)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现聊天界面中 Base64 图片数据的 SQLite 数据库持久化存储与加载，解决切换窗口/视图时图片丢失的问题。

**Architecture:** 在 SQLite 的 `messages` 表中新增可选列 `images TEXT`，在 Rust 层的 `ChatMessage` 实体中增加 `Option<String>` 属性。消息发送时将图片数组序列化为 JSON 字符串存入数据库，加载历史会话时由前端 Pinia Store 进行反序列化并渲染。

**Tech Stack:** Tauri v2, Rust (rusqlite, serde_json), Vue 3, TypeScript, Pinia

---

### Task 1: 升级数据库 Schema 与 Rust 数据模型

**Files:**
- Modify: `src-tauri/src/db/conversations.rs`
- Modify: `src-tauri/src/models/chat.rs`

- [ ] **Step 1: 在 Rust `ChatMessage` 数据模型中增加 `images` 属性**

在 `src-tauri/src/db/conversations.rs` 中：
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sources: Option<String>,
    pub images: Option<String>, // 新增
    pub created_at: String,
}
```

在 `src-tauri/src/models/chat.rs` 中同步增加：
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sources: Option<String>,
    pub images: Option<String>, // 新增
    pub created_at: String,
}
```

- [ ] **Step 2: 升级 `init_conversations_table` 建表语句与动态迁移策略**

修改 `src-tauri/src/db/conversations.rs` 中的建表及升级逻辑：
```rust
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
            images TEXT, -- 新增
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);",
    )
    .map_err(|e| AppError::Internal(format!("Failed to create conversations tables: {}", e)))?;

    // 动态添加字段支持热升级
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN images TEXT", []);

    Ok(())
}
```

- [ ] **Step 3: 运行 `cargo check` 验证代码是否存在编译错误**

执行：
```bash
cargo check --manifest-path src-tauri/Cargo.toml
```
Expected: 编译报错，提示在测试模块 `tests` 以及数据库访问方法的 `ChatMessage` 实例化中缺少 `images` 字段或列映射不匹配。

- [ ] **Step 4: Commit 本步骤更改**

```bash
git add src-tauri/src/db/conversations.rs src-tauri/src/models/chat.rs
git commit -m "db: add images column and update ChatMessage data structures"
```

---

### Task 2: 升级数据库存取接口与单元测试

**Files:**
- Modify: `src-tauri/src/db/conversations.rs`

- [ ] **Step 1: 升级数据库查询与写入语句以适配 `images` 列**

修改 `src-tauri/src/db/conversations.rs` 中的数据库操作函数：

```rust
pub fn get_messages(conn: &Connection, conv_id: &str) -> Result<Vec<ChatMessage>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, sources, images, created_at
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;
    let rows = stmt
        .query_map(params![conv_id], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                sources: row.get(4)?,
                images: row.get(5)?, // 新增
                created_at: row.get(6)?, // 索引后移一位
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn insert_message(conn: &Connection, msg: &ChatMessage) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, sources, images, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            msg.id,
            msg.conversation_id,
            msg.role,
            msg.content,
            msg.sources,
            msg.images, // 新增
            msg.created_at,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert message: {}", e)))?;
    Ok(())
}

pub fn get_message_by_id(
    conn: &Connection,
    message_id: &str,
) -> Result<ChatMessage, rusqlite::Error> {
    conn.query_row(
        "SELECT id, conversation_id, role, content, sources, images, created_at FROM messages WHERE id = ?1",
        rusqlite::params![message_id],
        |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                sources: row.get(4)?,
                images: row.get(5)?, // 新增
                created_at: row.get(6)?, // 索引后移一位
            })
        },
    )
}

pub fn get_messages_paginated(
    conn: &Connection,
    conv_id: &str,
    page: i32,
    page_size: i32,
) -> Result<(Vec<ChatMessage>, i64), AppError> {
    let offset = (page - 1) * page_size;

    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, sources, images, created_at
             FROM (
                SELECT id, conversation_id, role, content, sources, images, created_at
                FROM messages
                WHERE conversation_id = ?1
                ORDER BY created_at DESC
                LIMIT ?2 OFFSET ?3
             )
             ORDER BY created_at ASC",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(params![conv_id, page_size, offset], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                sources: row.get(4)?,
                images: row.get(5)?, // 新增
                created_at: row.get(6)?, // 索引后移一位
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let messages: Vec<ChatMessage> = rows.filter_map(|r| r.ok()).collect();

    let total = get_messages_count(conn, conv_id)?;

    Ok((messages, total))
}
```

- [ ] **Step 2: 在单元测试模块中为 `ChatMessage` 实例化补全 `images` 属性**

在 `src-tauri/src/db/conversations.rs` 底部的 `mod tests` 中修改 `test_insert_and_get_messages`：
```rust
    #[test]
    fn test_insert_and_get_messages() {
        let conn = setup_db();
        create_conversation(&conn, "conv-1", None).unwrap();
        let msg = ChatMessage {
            id: "msg-1".to_string(),
            conversation_id: "conv-1".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            sources: None,
            images: Some("[\"data:image/png;base64,...\"]".to_string()), // 修改此处增加测试数据
            created_at: "2026-01-01".to_string(),
        };
        insert_message(&conn, &msg).unwrap();
        let messages = get_messages(&conn, "conv-1").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[0].images, Some("[\"data:image/png;base64,...\"]".to_string())); // 验证 images 持久化
    }
```

- [ ] **Step 3: 运行 `cargo test` 验证单元测试**

执行：
```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib db::conversations::tests
```
Expected: 所有测试通过 (PASS)

- [ ] **Step 4: Commit 本步骤更改**

```bash
git add src-tauri/src/db/conversations.rs
git commit -m "db: update sqlite CRUD methods and pass conversation unit tests"
```

---

### Task 3: 升级 RAG 接口逻辑序列化保存图片

**Files:**
- Modify: `src-tauri/src/commands/rag.rs`

- [ ] **Step 1: 在 `rag_query` 逻辑中将图片序列化存盘，并适配占位符逻辑**

在 `src-tauri/src/commands/rag.rs` 中的 `rag_query` 函数中：

修改第 192-202 行的 `user_msg` 构造逻辑：
```rust
    // Save user message (skip if regenerating)
    let user_msg_id = if skip_user_insert.unwrap_or(false) {
        let last_user_msg = history.iter().rev().find(|m| m.role == "user");
        last_user_msg
            .map(|m| m.id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string())
    } else {
        // 序列化前端传入的 Base64 数组为 JSON
        let images_json = images.as_ref().map(|imgs| serde_json::to_string(imgs).unwrap_or_default());
        
        let user_msg = conversations::ChatMessage {
            id: Uuid::new_v4().to_string(),
            conversation_id: conv_id.clone(),
            role: "user".to_string(),
            content: final_query.clone(),
            sources: None,
            images: images_json, // 存入数据库
            created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        };
        conversations::insert_message(&conn, &user_msg)?;
        user_msg.id
    };
```

修改第 214-222 行的 `assistant_msg_placeholder` 构造逻辑：
```rust
    // --- 占位符先行策略 ---
    // 立即向数据库插入一条空的助手机器人消息，确保持久化锚点已建立
    let assistant_msg_placeholder = conversations::ChatMessage {
        id: assistant_msg_id.clone(),
        conversation_id: conv_id.clone(),
        role: "assistant".to_string(),
        content: String::new(),
        sources: sources_json.clone(),
        images: None, // 机器人回复无图片输入，保持 None
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };
    conversations::insert_message(&conn, &assistant_msg_placeholder)?;
```

- [ ] **Step 2: 验证 Rust 后端编译**

执行：
```bash
cargo build --manifest-path src-tauri/Cargo.toml
```
Expected: 编译完全通过，无任何 Warning 与 Error。

- [ ] **Step 3: Commit 本步骤更改**

```bash
git add src-tauri/src/commands/rag.rs
git commit -m "feat: serialize and save image list inside rag_query"
```

---

### Task 4: 升级前端接口类型声明与 Pinia Store 数据反序列化

**Files:**
- Modify: `src/types/chat.ts`
- Modify: `src/stores/chat.ts`

- [ ] **Step 1: 前端 `ChatMessage` 接口增加 `images` 声明**

修改 `src/types/chat.ts` 的 `ChatMessage` 接口定义：
```typescript
export interface ChatMessage {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  sources: string | null;
  images: string | null; // 新增，后端以 TEXT(JSON) 返回
  created_at: string;
}
```

- [ ] **Step 2: 在 Pinia Store 的 `loadConversationPaginated` 中反序列化图片数据**

修改 `src/stores/chat.ts` 中的 `loadConversationPaginated` 函数（约第 255-271 行）：
```typescript
      const newMessages = result.items.map((msg: ChatMessage, index: number) => {
        const isLastAssistant = msg.role === 'assistant' && index === result.items.length - 1;
        const isCurrentGenerating = (id === generatingConversationId.value) || (isGenerating.value && id === currentConversationId.value);
        const status: 'done' | 'streaming' | 'sending' | 'error' = (isLastAssistant && isCurrentGenerating) 
          ? (msg.content ? 'streaming' : 'sending') 
          : 'done';

        return {
          id: msg.id,
          role: msg.role as 'user' | 'assistant',
          content: msg.content,
          sources: msg.sources ? JSON.parse(msg.sources) : undefined,
          images: msg.images ? JSON.parse(msg.images) : undefined, // 反序列化图片 Base64 数组
          status,
          timestamp: new Date(msg.created_at).getTime(),
        };
      });
```

- [ ] **Step 3: 运行 TypeScript 语法检查**

执行：
```bash
pnpm tsc --noEmit
```
Expected: 没有任何 TS 编译错误或警告。

- [ ] **Step 4: Commit 本步骤更改**

```bash
git add src/types/chat.ts src/stores/chat.ts
git commit -m "feat: frontend support deserializing images and integrate type schema"
```
