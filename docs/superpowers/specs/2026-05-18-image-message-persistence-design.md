# 2026-05-18 图片消息持久化设计文档 (Image Message Persistence Design)

## 1. 概述
在 Telepathy（本地 AI 知识库桌面应用）中，用户发送图片消息并在切换窗口或重新聚焦时，原本保存在内存中的图片会因为触发会话重新加载从数据库中覆盖拉取而消失，只有文本得以留存。
本规范定义了为图片消息（多模态输入）添加完整持久化支持的设计，确保用户上传的 Base64 图片在会话重载或切换时依旧完美渲染。

## 2. 需求分析
1. **数据持久化**：用户发送的图片（Base64 字符串数组）需要保存在 SQLite 数据库中。
2. **平滑迁移**：更新数据库建表语句，并提供自动 `ALTER TABLE` 逻辑，在保证新用户直接建表的同时，老用户在升级后也能够无缝加载（不丢失老数据且不发生奔溃）。
3. **级联删除**：当删除会话时，图片对应的字段随消息行一同从 SQLite 级联清除，不留下任何脏数据。
4. **前后端序列化对齐**：
   - 后端 (Rust) 统一以 `Option<String>` 字段接收并返回序列化后的 JSON 字符串（例如 `["data:image/png;base64,..."]`）。
   - 前端 (TypeScript) 将该 JSON 字符串反序列化回 Base64 字符串数组以供 Vue 组件直接渲染。

## 3. 详细设计

### 3.1 数据库结构变更 (SQLite Schema)
在 `messages` 表中新增可选列 `images`，其类型为 `TEXT`：
```sql
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    sources TEXT,
    images TEXT, -- 新增的图片存储列
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
```

#### 数据库初始化与平滑兼容
在 `init_conversations_table` 时，若表已建立但缺少 `images` 列，我们将动态执行：
```rust
let _ = conn.execute("ALTER TABLE messages ADD COLUMN images TEXT", []);
```
此句报错（如字段已存在）会自动被忽略，从而实现完美零成本的前向兼容。

### 3.2 后端变更 (Rust)

#### 数据结构变更
1. **`db::conversations::ChatMessage`** 结构体新增：
   ```rust
   pub images: Option<String>,
   ```
2. **`models::chat::ChatMessage`** 结构体同步新增：
   ```rust
   pub images: Option<String>,
   ```

#### 数据库操作方法升级 (`db::conversations`)
- `insert_message`: 在 `INSERT INTO messages` 语句中新增映射 `images`。
- `get_messages`: 查询语句新增 `images` 列，返回时装入结构体，并将后面的列索引向后推移。
- `get_message_by_id`: 查询语句新增 `images` 列，映射装入结构体。
- `get_messages_paginated`: 在嵌套子查询及外层查询中均引入 `images` 列，以确保分页也能正常载入图片。

#### 核心命令逻辑变更 (`commands::rag::rag_query`)
在保存用户发送的消息时：
```rust
let images_json = images.as_ref().map(|imgs| serde_json::to_string(imgs).unwrap_or_default());
```
然后将 `images_json` 作为 `images` 属性赋值给 `conversations::ChatMessage` 进行插入。

---

### 3.3 前端变更 (TypeScript / Vue)

#### 接口类型更新 (`src/types/chat.ts`)
更新 `ChatMessage` 接口，新增：
```typescript
images: string | null;
```

#### 状态管理器升级 (`src/stores/chat.ts`)
在 `loadConversationPaginated` 的数据解包映射过程中：
```typescript
images: msg.images ? JSON.parse(msg.images) : undefined,
```

---

## 4. 测试与验证标准
1. **图片发送与加载**：上传 1-2 张截图发送，页面立即正常渲染。
2. **重新聚焦与页面切换**：切换到别的操作系统窗口再切回，或者在左侧对话历史列表之间进行切换，图片**均持久完整，依然显示在原有位置**。
3. **级联删除校验**：删除会话，对应数据库 `messages` 物理数据清空，磁盘无图片文件残留。
4. **编译与单测**：
   - 运行前端 `tsc --noEmit` 成功。
   - 运行后端 `cargo test` 成功。
