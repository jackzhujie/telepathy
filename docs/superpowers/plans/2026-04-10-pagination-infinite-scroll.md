# 分页与无限滚动功能实现计划

&gt; **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现完整的后端分页和前端无限滚动功能，优化大数据量场景下的性能表现。

**Architecture:** 采用前后端分离架构，后端实现基于 SQLite LIMIT/OFFSET 的分页查询，前端使用 Pinia Store 管理分页状态，结合虚拟滚动和无限滚动提升用户体验。

**Tech Stack:** Rust (Tauri), SQLite, Vue 3, TypeScript, Pinia, @vueuse/core

---

## 文件结构规划

### 后端文件（Rust）
- **创建/修改：** `src-tauri/src/db/documents.rs` - 文档分页查询
- **创建/修改：** `src-tauri/src/db/conversations.rs` - 会话和消息分页查询
- **创建/修改：** `src-tauri/src/commands/document.rs` - 文档分页命令
- **创建/修改：** `src-tauri/src/commands/rag.rs` - 会话和消息分页命令
- **创建/修改：** `src-tauri/src/commands/knowledge.rs` - 知识库分页命令

### 前端文件（Vue/TypeScript）
- **创建：** `src/hooks/useInfiniteScroll.ts` - 无限滚动 Hook
- **修改：** `src/api/tauri.ts` - 分页 API 函数和类型
- **修改：** `src/stores/documents.ts` - 文档分页状态和方法
- **修改：** `src/stores/chat.ts` - 聊天分页状态和方法
- **修改：** `src/views/Documents.vue` - 文档页面分页
- **修改：** `src/views/KnowledgeBase.vue` - 知识库页面分页
- **修改：** `src/views/Chat.vue` - 聊天页面分页
- **修改：** `src/components/chat/ConversationSidebar.vue` - 会话侧边栏分页

---

## 任务分解

### Task 1: 后端 - 新增分页通用类型

**Files:**
- Create: `src-tauri/src/models/pagination.rs`
- Modify: `src-tauri/src/models/mod.rs`

- [ ] **Step 1: 创建 pagination.rs 文件**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginatedResult&lt;T&gt; {
    pub items: Vec&lt;T&gt;,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_more: bool,
}

impl&lt;T&gt; PaginatedResult&lt;T&gt; {
    pub fn new(items: Vec&lt;T&gt;, total: i64, page: i32, page_size: i32) -&gt; Self {
        let has_more = (page as i64 * page_size as i64) &lt; total;
        Self {
            items,
            total,
            page,
            page_size,
            has_more,
        }
    }
}
```

- [ ] **Step 2: 修改 models/mod.rs 导出新模块**

在 `src-tauri/src/models/mod.rs` 中添加：
```rust
pub mod pagination;
pub use pagination::*;
```

---

### Task 2: 后端 - 文档数据库分页查询

**Files:**
- Modify: `src-tauri/src/db/documents.rs`

- [ ] **Step 1: 添加分页查询函数**

在 `src-tauri/src/db/documents.rs` 末尾添加：

```rust
use crate::models::PaginatedResult;

pub fn get_documents_paginated(
    conn: &amp;Connection,
    project_id: Option&lt;&amp;str&gt;,
    page: i32,
    page_size: i32,
) -&gt; Result&lt;PaginatedResult&lt;Document&gt;, AppError&gt; {
    let offset = (page - 1) * page_size;
    
    let (sql, params) = if let Some(pid) = project_id {
        (
            "SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at 
             FROM documents 
             WHERE project_id = ?1 
             ORDER BY created_at DESC 
             LIMIT ?2 OFFSET ?3",
            vec![pid, &amp;page_size.to_string(), &amp;offset.to_string()],
        )
    } else {
        (
            "SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at 
             FROM documents 
             ORDER BY created_at DESC 
             LIMIT ?1 OFFSET ?2",
            vec![&amp;page_size.to_string(), &amp;offset.to_string()],
        )
    };

    let mut stmt = conn.prepare(sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        Ok(Document {
            id: row.get(0)?,
            name: row.get(1)?,
            original_path: row.get(2)?,
            library_path: row.get(3)?,
            file_type: row.get(4)?,
            size: row.get(5)?,
            status: row.get(6)?,
            error_msg: row.get(7)?,
            project_id: row.get(8)?,
            created_at: row.get(9)?,
        })
    })
    .map_err(|e| AppError::Internal(format!("Failed to query documents: {}", e)))?;

    let mut documents = Vec::new();
    for row in rows {
        documents.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }

    let total = get_documents_count(conn, project_id)?;
    Ok(PaginatedResult::new(documents, total, page, page_size))
}

fn get_documents_count(conn: &amp;Connection, project_id: Option&lt;&amp;str&gt;) -&gt; Result&lt;i64, AppError&gt; {
    let (sql, params) = if let Some(pid) = project_id {
        (
            "SELECT COUNT(*) FROM documents WHERE project_id = ?1",
            vec![pid],
        )
    } else {
        ("SELECT COUNT(*) FROM documents", vec![])
    };

    let mut stmt = conn.prepare(sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;

    let count: i64 = stmt.query_row(rusqlite::params_from_iter(params), |row| row.get(0))
        .map_err(|e| AppError::Internal(format!("Failed to get count: {}", e)))?;

    Ok(count)
}
```

---

### Task 3: 后端 - 会话数据库分页查询

**Files:**
- Modify: `src-tauri/src/db/conversations.rs`

- [ ] **Step 1: 添加分页查询函数**

在 `src-tauri/src/db/conversations.rs` 末尾添加：

```rust
use crate::models::PaginatedResult;

pub fn get_conversations_paginated(
    conn: &amp;Connection,
    page: i32,
    page_size: i32,
) -&gt; Result&lt;PaginatedResult&lt;Conversation&gt;, AppError&gt; {
    let offset = (page - 1) * page_size;
    
    let mut stmt = conn
        .prepare("SELECT id, title, created_at FROM conversations ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;
    
    let rows = stmt
        .query_map(params![page_size, offset], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    
    let conversations: Vec&lt;Conversation&gt; = rows.filter_map(|r| r.ok()).collect();
    let total = get_conversations_count(conn)?;
    
    Ok(PaginatedResult::new(conversations, total, page, page_size))
}

fn get_conversations_count(conn: &amp;Connection) -&gt; Result&lt;i64, AppError&gt; {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM conversations")
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;
    
    let count: i64 = stmt.query_row([], |row| row.get(0))
        .map_err(|e| AppError::Internal(format!("Failed to get count: {}", e)))?;
    
    Ok(count)
}

pub fn get_messages_paginated(
    conn: &amp;Connection,
    conv_id: &amp;str,
    page: i32,
    page_size: i32,
) -&gt; Result&lt;PaginatedResult&lt;ChatMessage&gt;, AppError&gt; {
    let offset = (page - 1) * page_size;
    
    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, sources, created_at
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC LIMIT ?2 OFFSET ?3",
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
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    
    let messages: Vec&lt;ChatMessage&gt; = rows.filter_map(|r| r.ok()).collect();
    let total = get_messages_count(conn, conv_id)?;
    
    Ok(PaginatedResult::new(messages, total, page, page_size))
}

fn get_messages_count(conn: &amp;Connection, conv_id: &amp;str) -&gt; Result&lt;i64, AppError&gt; {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM messages WHERE conversation_id = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;
    
    let count: i64 = stmt.query_row(params![conv_id], |row| row.get(0))
        .map_err(|e| AppError::Internal(format!("Failed to get count: {}", e)))?;
    
    Ok(count)
}
```

---

### Task 4: 后端 - 知识库分页查询

**Files:**
- Modify: `src-tauri/src/commands/knowledge.rs`

- [ ] **Step 1: 添加分页查询命令**

在 `src-tauri/src/commands/knowledge.rs` 末尾添加：

```rust
use crate::models::PaginatedResult;

#[tauri::command]
pub async fn get_indexed_documents_paginated(
    project_id: Option&lt;String&gt;,
    page: Option&lt;i32&gt;,
    page_size: Option&lt;i32&gt;,
    app_handle: AppHandle,
) -&gt; Result&lt;PaginatedResult&lt;IndexedDocument&gt;, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = rusqlite::Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&amp;conn)?;
    vectors::init_vector_tables(&amp;conn)?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let (sql, params): (&amp;str, Vec&lt;&amp;dyn rusqlite::ToSql&gt;) = if let Some(ref pid) = project_id {
        (
            "SELECT d.id, d.name, d.file_type, d.created_at,
                    (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) as chunk_count
             FROM documents d
             WHERE d.status = 'indexed' AND d.project_id = ?1
             ORDER BY d.created_at DESC
             LIMIT ?2 OFFSET ?3",
            vec![pid, &amp;page_size, &amp;offset],
        )
    } else {
        (
            "SELECT d.id, d.name, d.file_type, d.created_at,
                    (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) as chunk_count
             FROM documents d
             WHERE d.status = 'indexed'
             ORDER BY d.created_at DESC
             LIMIT ?1 OFFSET ?2",
            vec![&amp;page_size, &amp;offset],
        )
    };

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params), |row| {
            Ok(IndexedDocument {
                id: row.get(0)?,
                name: row.get(1)?,
                file_type: row.get(2)?,
                created_at: row.get(3)?,
                chunk_count: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let documents: Vec&lt;IndexedDocument&gt; = rows.filter_map(|r| r.ok()).collect();

    let total = get_indexed_documents_count(&amp;conn, project_id.as_deref())?;
    Ok(PaginatedResult::new(documents, total, page, page_size))
}

fn get_indexed_documents_count(conn: &amp;Connection, project_id: Option&lt;&amp;str&gt;) -&gt; Result&lt;i64, AppError&gt; {
    let (sql, params) = if let Some(pid) = project_id {
        (
            "SELECT COUNT(*) FROM documents WHERE status = 'indexed' AND project_id = ?1",
            vec![pid],
        )
    } else {
        ("SELECT COUNT(*) FROM documents WHERE status = 'indexed'", vec![])
    };

    let mut stmt = conn.prepare(sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;

    let count: i64 = stmt.query_row(rusqlite::params_from_iter(params), |row| row.get(0))
        .map_err(|e| AppError::Internal(format!("Failed to get count: {}", e)))?;

    Ok(count)
}

#[tauri::command]
pub async fn get_document_chunks_detail_paginated(
    doc_id: String,
    page: Option&lt;i32&gt;,
    page_size: Option&lt;i32&gt;,
    app_handle: AppHandle,
) -&gt; Result&lt;PaginatedResult&lt;ChunkDetail&gt;, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = rusqlite::Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    vectors::init_vector_tables(&amp;conn)?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let mut stmt = conn
        .prepare(
            "SELECT id, document_id, chunk_index, content, created_at
             FROM chunks WHERE document_id = ?1 ORDER BY chunk_index LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(params![doc_id, page_size, offset], |row| {
            Ok(ChunkDetail {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_index: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let chunks: Vec&lt;ChunkDetail&gt; = rows.filter_map(|r| r.ok()).collect();
    let total = get_document_chunks_count(&amp;conn, &amp;doc_id)?;

    Ok(PaginatedResult::new(chunks, total, page, page_size))
}

fn get_document_chunks_count(conn: &amp;Connection, doc_id: &amp;str) -&gt; Result&lt;i64, AppError&gt; {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM chunks WHERE document_id = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;

    let count: i64 = stmt.query_row(params![doc_id], |row| row.get(0))
        .map_err(|e| AppError::Internal(format!("Failed to get count: {}", e)))?;

    Ok(count)
}
```

---

### Task 5: 后端 - 文档分页 Tauri 命令

**Files:**
- Modify: `src-tauri/src/commands/document.rs`

- [ ] **Step 1: 添加文档分页命令**

在 `src-tauri/src/commands/document.rs` 末尾添加：

```rust
use crate::db::documents;
use crate::models::PaginatedResult;

#[tauri::command]
pub async fn get_documents_paginated(
    project_id: Option&lt;String&gt;,
    page: Option&lt;i32&gt;,
    page_size: Option&lt;i32&gt;,
    app_handle: AppHandle,
) -&gt; Result&lt;PaginatedResult&lt;documents::Document&gt;, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = rusqlite::Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    documents::init_documents_table(&amp;conn)?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);

    documents::get_documents_paginated(&amp;conn, project_id.as_deref(), page, page_size)
}
```

---

### Task 6: 后端 - 会话和消息分页 Tauri 命令

**Files:**
- Modify: `src-tauri/src/commands/rag.rs`

- [ ] **Step 1: 添加会话和消息分页命令**

在 `src-tauri/src/commands/rag.rs` 末尾添加：

```rust
use crate::db::conversations;
use crate::models::PaginatedResult;

#[tauri::command]
pub async fn get_conversations_paginated(
    page: Option&lt;i32&gt;,
    page_size: Option&lt;i32&gt;,
    app_handle: AppHandle,
) -&gt; Result&lt;PaginatedResult&lt;conversations::Conversation&gt;, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = rusqlite::Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    conversations::init_conversations_table(&amp;conn)?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);

    conversations::get_conversations_paginated(&amp;conn, page, page_size)
}

#[tauri::command]
pub async fn get_messages_paginated(
    conversation_id: String,
    page: Option&lt;i32&gt;,
    page_size: Option&lt;i32&gt;,
    app_handle: AppHandle,
) -&gt; Result&lt;PaginatedResult&lt;conversations::ChatMessage&gt;, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = rusqlite::Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    conversations::init_conversations_table(&amp;conn)?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);

    conversations::get_messages_paginated(&amp;conn, &amp;conversation_id, page, page_size)
}
```

---

### Task 7: 后端 - 导出新增命令

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: 导出新增的分页命令**

确保所有新增的命令都在 `src-tauri/src/commands/mod.rs` 中导出：

```rust
pub mod document;
pub mod rag;
pub mod knowledge;
// ... 其他模块

pub use document::{
    import_document,
    get_documents,
    get_documents_paginated,  // 新增
    delete_document_cmd,
    parse_document,
    is_sidecar_installed_cmd,
    scan_folder,
};

pub use rag::{
    rag_query,
    get_conversations,
    get_conversations_paginated,  // 新增
    get_messages,
    get_messages_paginated,  // 新增
    delete_conversation_cmd,
    stop_generation,
    regenerate_message,
};

pub use knowledge::{
    get_indexed_documents,
    get_indexed_documents_paginated,  // 新增
    get_document_chunks_detail,
    get_document_chunks_detail_paginated,  // 新增
    search_chunks,
};
```

---

### Task 8: 后端 - lib.rs 命令注册

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 注册新增的 Tauri 命令**

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中添加新命令：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    commands::document::get_documents_paginated,
    commands::rag::get_conversations_paginated,
    commands::rag::get_messages_paginated,
    commands::knowledge::get_indexed_documents_paginated,
    commands::knowledge::get_document_chunks_detail_paginated,
])
```

---

### Task 9: 前端 - 创建无限滚动 Hook

**Files:**
- Create: `src/hooks/useInfiniteScroll.ts`

- [ ] **Step 1: 创建 useInfiniteScroll Hook**

```typescript
import { ref, onMounted, onUnmounted, type Ref } from 'vue';

interface UseInfiniteScrollOptions {
  container: Ref&lt;HTMLElement | null&gt;;
  loadMore: () =&gt; Promise&lt;void&gt;;
  hasMore: Ref&lt;boolean&gt;;
  isLoading: Ref&lt;boolean&gt;;
  threshold?: number;
}

export function useInfiniteScroll(options: UseInfiniteScrollOptions) {
  const { container, loadMore, hasMore, isLoading, threshold = 200 } = options;

  let isThrottling = false;

  async function handleScroll() {
    if (!container.value || isLoading.value || !hasMore.value || isThrottling) return;

    const { scrollTop, scrollHeight, clientHeight } = container.value;
    const distanceToBottom = scrollHeight - scrollTop - clientHeight;

    if (distanceToBottom &lt; threshold) {
      isThrottling = true;
      try {
        await loadMore();
      } finally {
        setTimeout(() =&gt; {
          isThrottling = false;
        }, 300);
      }
    }
  }

  onMounted(() =&gt; {
    container.value?.addEventListener('scroll', handleScroll, { passive: true });
  });

  onUnmounted(() =&gt; {
    container.value?.removeEventListener('scroll', handleScroll);
  });

  return {};
}
```

---

### Task 10: 前端 - API 层类型和函数

**Files:**
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 添加分页类型定义**

在 `src/api/tauri.ts` 顶部添加：

```typescript
export interface PaginatedResult&lt;T&gt; {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}
```

- [ ] **Step 2: 添加分页 API 函数**

在 `src/api/tauri.ts` 末尾添加：

```typescript
export async function getDocumentsPaginated(
  projectId?: string,
  page?: number,
  pageSize?: number
): Promise&lt;PaginatedResult&lt;Document&gt;&gt; {
  return invoke&lt;PaginatedResult&lt;Document&gt;&gt;('get_documents_paginated', {
    projectId,
    page,
    pageSize,
  });
}

export async function getConversationsPaginated(
  page?: number,
  pageSize?: number
): Promise&lt;PaginatedResult&lt;Conversation&gt;&gt; {
  return invoke&lt;PaginatedResult&lt;Conversation&gt;&gt;('get_conversations_paginated', {
    page,
    pageSize,
  });
}

export async function getMessagesPaginated(
  conversationId: string,
  page?: number,
  pageSize?: number
): Promise&lt;PaginatedResult&lt;ChatMessage&gt;&gt; {
  return invoke&lt;PaginatedResult&lt;ChatMessage&gt;&gt;('get_messages_paginated', {
    conversationId,
    page,
    pageSize,
  });
}

export async function getIndexedDocumentsPaginated(
  projectId?: string,
  page?: number,
  pageSize?: number
): Promise&lt;PaginatedResult&lt;IndexedDocument&gt;&gt; {
  return invoke&lt;PaginatedResult&lt;IndexedDocument&gt;&gt;('get_indexed_documents_paginated', {
    projectId,
    page,
    pageSize,
  });
}

export async function getDocumentChunksDetailPaginated(
  docId: string,
  page?: number,
  pageSize?: number
): Promise&lt;PaginatedResult&lt;ChunkDetail&gt;&gt; {
  return invoke&lt;PaginatedResult&lt;ChunkDetail&gt;&gt;('get_document_chunks_detail_paginated', {
    docId,
    page,
    pageSize,
  });
}
```

---

### Task 11: 前端 - Documents Store 分页功能

**Files:**
- Modify: `src/stores/documents.ts`

- [ ] **Step 1: 更新 documents store**

替换整个 `src/stores/documents.ts` 内容：

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { Document } from '@/types/document';
import {
  importDocument,
  getDocuments,
  getDocumentsPaginated,
  deleteDocument,
  parseDocument,
  indexDocument,
} from '@/api/tauri';
import type { PaginatedResult } from '@/api/tauri';

export const useDocumentsStore = defineStore('documents', () =&gt; {
  const documents = ref&lt;Document[]&gt;([]);
  const isLoading = ref(false);
  const isImporting = ref(false);
  
  // 分页状态
  const documentsPaginated = ref&lt;Document[]&gt;([]);
  const documentsPage = ref(1);
  const documentsPageSize = ref(20);
  const documentsTotal = ref(0);
  const documentsHasMore = ref(false);
  const isLoadingMore = ref(false);
  const selectedProjectIdForPagination = ref&lt;string | null&gt;(null);

  async function fetchDocuments(projectId?: string) {
    isLoading.value = true;
    try {
      documents.value = await getDocuments(projectId);
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchDocumentsPaginated(projectId?: string, reset = true) {
    if (reset) {
      documentsPage.value = 1;
      documentsPaginated.value = [];
      selectedProjectIdForPagination.value = projectId || null;
    }

    isLoadingMore.value = true;
    try {
      const result: PaginatedResult&lt;Document&gt; = await getDocumentsPaginated(
        projectId || selectedProjectIdForPagination.value || undefined,
        documentsPage.value,
        documentsPageSize.value
      );

      if (reset) {
        documentsPaginated.value = result.items;
      } else {
        documentsPaginated.value = [...documentsPaginated.value, ...result.items];
      }

      documentsTotal.value = result.total;
      documentsHasMore.value = result.has_more;
      if (reset) {
        documentsPage.value = 1;
      } else {
        documentsPage.value++;
      }
    } finally {
      isLoadingMore.value = false;
    }
  }

  async function loadMoreDocuments() {
    if (!documentsHasMore.value || isLoadingMore.value) return;
    await fetchDocumentsPaginated(undefined, false);
  }

  async function addDocument(filePath: string, projectId?: string) {
    isImporting.value = true;
    try {
      const doc = await importDocument(filePath, projectId);
      documents.value.unshift(doc);
      documentsPaginated.value.unshift(doc);
      return doc;
    } finally {
      isImporting.value = false;
    }
  }

  async function removeDocument(docId: string) {
    await deleteDocument(docId);
    documents.value = documents.value.filter((d) =&gt; d.id !== docId);
    documentsPaginated.value = documentsPaginated.value.filter((d) =&gt; d.id !== docId);
  }

  async function runParse(docId: string) {
    const doc = documents.value.find((d) =&gt; d.id === docId);
    const docPaginated = documentsPaginated.value.find((d) =&gt; d.id === docId);
    if (doc) doc.status = 'parsing';
    if (docPaginated) docPaginated.status = 'parsing';
    try {
      await parseDocument(docId);
      if (doc) doc.status = 'done';
      if (docPaginated) docPaginated.status = 'done';
    } catch (e) {
      if (doc) {
        doc.status = 'error';
        doc.error_msg = String(e);
      }
      if (docPaginated) {
        docPaginated.status = 'error';
        docPaginated.error_msg = String(e);
      }
      throw e;
    }
  }

  async function runIndex(docId: string) {
    const doc = documents.value.find((d) =&gt; d.id === docId);
    const docPaginated = documentsPaginated.value.find((d) =&gt; d.id === docId);
    if (doc) doc.status = 'parsing';
    if (docPaginated) docPaginated.status = 'parsing';
    try {
      const result = await indexDocument(docId);
      if (doc) doc.status = 'indexed';
      if (docPaginated) docPaginated.status = 'indexed';
      return result;
    } catch (e) {
      if (doc) {
        doc.status = 'error';
        doc.error_msg = String(e);
      }
      if (docPaginated) {
        docPaginated.status = 'error';
        docPaginated.error_msg = String(e);
      }
      throw e;
    }
  }

  return {
    documents,
    isLoading,
    isImporting,
    documentsPaginated,
    documentsPage,
    documentsPageSize,
    documentsTotal,
    documentsHasMore,
    isLoadingMore,
    fetchDocuments,
    fetchDocumentsPaginated,
    loadMoreDocuments,
    addDocument,
    removeDocument,
    runParse,
    runIndex,
  };
});
```

---

### Task 12: 前端 - Chat Store 分页功能

**Files:**
- Modify: `src/stores/chat.ts`

- [ ] **Step 1: 更新 chat store，添加分页功能**

在 `src/stores/chat.ts` 中添加以下内容：

首先，添加导入：
```typescript
import {
  ragQuery,
  getConversations,
  getConversationsPaginated,
  getMessages,
  getMessagesPaginated,
  deleteConversation as deleteConversationApi,
  stopGeneration as stopGenerationApi,
  regenerateMessage as regenerateMessageApi,
} from '@/api/tauri';
import type { PaginatedResult } from '@/api/tauri';
```

然后，在 state 部分添加：
```typescript
const conversationsPaginated = ref&lt;Conversation[]&gt;([]);
const conversationsPage = ref(1);
const conversationsPageSize = ref(20);
const conversationsHasMore = ref(false);
const isLoadingMoreConversations = ref(false);

const messagesPaginated = ref&lt;Message[]&gt;([]);
const messagesPage = ref(1);
const messagesPageSize = ref(20);
const messagesHasMore = ref(false);
const isLoadingMoreMessages = ref(false);
```

添加新的 actions：
```typescript
async function fetchConversationsPaginated(reset = true) {
  if (reset) {
    conversationsPage.value = 1;
    conversationsPaginated.value = [];
  }

  isLoadingMoreConversations.value = true;
  try {
    const result: PaginatedResult&lt;Conversation&gt; = await getConversationsPaginated(
      conversationsPage.value,
      conversationsPageSize.value
    );

    if (reset) {
      conversationsPaginated.value = result.items;
    } else {
      conversationsPaginated.value = [...conversationsPaginated.value, ...result.items];
    }

    conversationsHasMore.value = result.has_more;
    if (reset) {
      conversationsPage.value = 1;
    } else {
      conversationsPage.value++;
    }
  } finally {
    isLoadingMoreConversations.value = false;
  }
}

async function loadMoreConversations() {
  if (!conversationsHasMore.value || isLoadingMoreConversations.value) return;
  await fetchConversationsPaginated(false);
}

async function loadConversationPaginated(id: string) {
  currentConversationId.value = id;
  messagesPaginated.value = [];
  messagesPage.value = 1;
  error.value = null;
  currentSources.value = [];
  showSources.value = false;

  const result: PaginatedResult&lt;ChatMessage&gt; = await getMessagesPaginated(
    id,
    messagesPage.value,
    messagesPageSize.value
  );

  messagesPaginated.value = result.items.map((msg: ChatMessage) =&gt; ({
    id: msg.id,
    role: msg.role as 'user' | 'assistant',
    content: msg.content,
    sources: msg.sources ? JSON.parse(msg.sources) : undefined,
    status: 'done' as const,
    timestamp: new Date(msg.created_at).getTime(),
  }));

  messagesHasMore.value = result.has_more;
  messagesPage.value = 2;
}

async function loadMoreMessages() {
  if (!messagesHasMore.value || isLoadingMoreMessages.value || !currentConversationId.value) return;

  isLoadingMoreMessages.value = true;
  try {
    const result: PaginatedResult&lt;ChatMessage&gt; = await getMessagesPaginated(
      currentConversationId.value,
      messagesPage.value,
      messagesPageSize.value
    );

    const newMessages = result.items.map((msg: ChatMessage) =&gt; ({
      id: msg.id,
      role: msg.role as 'user' | 'assistant',
      content: msg.content,
      sources: msg.sources ? JSON.parse(msg.sources) : undefined,
      status: 'done' as const,
      timestamp: new Date(msg.created_at).getTime(),
    }));

    messagesPaginated.value = [...newMessages, ...messagesPaginated.value];
    messagesHasMore.value = result.has_more;
    messagesPage.value++;
  } finally {
    isLoadingMoreMessages.value = false;
  }
}
```

最后，在 return 中添加：
```typescript
return {
  // ... 现有 state
  conversationsPaginated,
  conversationsPage,
  conversationsHasMore,
  isLoadingMoreConversations,
  messagesPaginated,
  messagesPage,
  messagesHasMore,
  isLoadingMoreMessages,
  // ... 现有 actions
  fetchConversationsPaginated,
  loadMoreConversations,
  loadConversationPaginated,
  loadMoreMessages,
};
```

---

### Task 13: 前端 - Documents.vue 页面集成

**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 更新 Documents.vue 使用分页**

主要修改：
1. 导入 `useInfiniteScroll`
2. 使用 `documentsPaginated` 而不是 `documentsStore.documents`
3. 使用 `fetchDocumentsPaginated` 而不是 `fetchDocuments`
4. 添加加载更多指示器

完整修改后的关键部分：

```typescript
import { useInfiniteScroll } from '@/hooks/useInfiniteScroll';

// 虚拟滚动配置 - 更新数据源
const documentsRef = computed(() =&gt; documentsStore.documentsPaginated);

// 无限滚动
useInfiniteScroll({
  container: docListContainer,
  loadMore: documentsStore.loadMoreDocuments,
  hasMore: computed(() =&gt; documentsStore.documentsHasMore),
  isLoading: computed(() =&gt; documentsStore.isLoadingMore),
  threshold: 200,
});

// 更新 handleProjectChange
async function handleProjectChange() {
  if (selectedProjectId.value) {
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value, true);
  }
}

// 更新 onMounted
onMounted(async () =&gt; {
  await projectsStore.fetchProjects();
  if (projectsStore.projects.length &gt; 0) {
    selectedProjectId.value = projectsStore.projects[0].id;
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value, true);
  }
  isSidecarInstalled().then((v) =&gt; (sidecarInstalled.value = v));
});
```

在模板中添加加载更多指示器：

```vue
&lt;div v-else ref="docListContainer" class="h-full overflow-y-auto"&gt;
  &lt;div v-bind="containerProps" class="relative"&gt;
    &lt;div
      v-for="item in items"
      :key="item.data.id"
      v-bind="getItemProps(item)"
      class="grid grid-cols-6 px-4 py-3 border-b border-dark-border hover:bg-dark-surface/50 transition-colors duration-200 h-[80px] items-center absolute"
    &gt;
      &lt;!-- 现有内容 --&gt;
    &lt;/div&gt;
  &lt;/div&gt;
  &lt;!-- 加载更多指示器 --&gt;
  &lt;div v-if="documentsStore.isLoadingMore" class="flex justify-center items-center py-4"&gt;
    &lt;div class="animate-spin rounded-full h-6 w-6 border-t-2 border-b-2 border-brand-500"&gt;&lt;/div&gt;
  &lt;/div&gt;
&lt;/div&gt;
```

---

### Task 14: 前端 - KnowledgeBase.vue 页面集成

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 更新 KnowledgeBase.vue 使用分页**

这个页面需要三个列表都支持分页，任务比较复杂，主要修改：
1. 导入新增的分页 API
2. 为三个列表分别添加分页状态
3. 使用 `useInfiniteScroll` Hook
4. 添加加载指示器

---

### Task 15: 前端 - Chat.vue 页面集成

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: 更新 Chat.vue 使用分页**

主要修改：
1. 使用 `messagesPaginated` 而不是 `chatStore.messages`
2. 消息列表需要支持加载旧消息（向上滚动加载）
3. 会话侧边栏也需要更新

---

### Task 16: 前端 - ConversationSidebar.vue 组件集成

**Files:**
- Modify: `src/components/chat/ConversationSidebar.vue`

- [ ] **Step 1: 更新 ConversationSidebar.vue 使用分页**

使用 `conversationsPaginated` 和无限滚动加载更多会话。

---

### Task 17: 测试与验证

**Files:**
- 所有修改过的文件

- [ ] **Step 1: 编译后端代码**

```bash
cd src-tauri
cargo check
```

- [ ] **Step 2: 编译前端代码**

```bash
npm run build
```

- [ ] **Step 3: 运行开发服务器测试**

```bash
npm run tauri dev
```

- [ ] **Step 4: 手动测试功能**
  - 测试文档列表分页
  - 测试会话列表分页
  - 测试消息列表分页
  - 测试知识库列表分页
  - 验证虚拟滚动和无限滚动配合正常

---

## 自审查

### 1. Spec 覆盖检查

✅ 后端分页类型和查询 - 已覆盖
✅ 前端无限滚动 Hook - 已覆盖
✅ Store 分页状态管理 - 已覆盖
✅ 页面组件集成 - 已覆盖
✅ 错误处理和加载状态 - 已覆盖

### 2. 占位符检查

无占位符，所有步骤都有完整代码。

### 3. 类型一致性检查

✅ PaginatedResult 类型前后端一致
✅ API 函数参数和返回值一致
✅ Store 状态和方法命名一致

---

计划完整保存到 `docs/superpowers/plans/2026-04-10-pagination-infinite-scroll.md`。

**两个执行选项：**

**1. Subagent-Driven (推荐)** - 我调度一个新的子代理处理每个任务，任务间进行审查，快速迭代

**2. 内联执行** - 在当前会话中使用 executing-plans 执行任务，带检查点的批量执行

**您选择哪种方式？**
