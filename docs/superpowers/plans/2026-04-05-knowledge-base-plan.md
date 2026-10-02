# 知识库页面实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现知识库页面，展示已索引的文档和文本块，支持简单的搜索功能

**Architecture:** 前端 Vue 页面调用后端 Rust API 获取数据，使用 Naive UI 组件展示文档列表和文本块详情

**Tech Stack:** Vue 3, TypeScript, Naive UI, Pinia, Tauri Commands

---

## 功能设计

### 核心功能
1. **已索引文档列表** - 显示所有 `status = 'indexed'` 的文档
2. **文档详情** - 点击文档显示其所有文本块（chunks）
3. **文本搜索** - 在文本块中搜索关键词（使用 SQLite LIKE 查询，不依赖 embedding）

### 页面布局
- 左侧：已索引文档列表（可搜索/过滤）
- 右侧：选中文档的文本块列表
- 支持点击展开查看文本块详情

---

## 文件变更清单

### 新建文件
- `src/types/knowledge.ts` - 知识库相关类型定义

### 修改文件
- `src/views/KnowledgeBase.vue` - 实现知识库页面 UI
- `src/api/tauri.ts` - 添加 `search_chunks` API
- `src-tauri/src/commands/knowledge.rs` - 新建后端命令
- `src-tauri/src/commands/mod.rs` - 导出 knowledge 模块
- `src-tauri/src/lib.rs` - 注册 knowledge 命令

---

### Task 1: 后端 - 添加知识库命令

**Files:**
- Create: `src-tauri/src/commands/knowledge.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 knowledge.rs**

```rust
use crate::db::{documents, vectors};
use crate::errors::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedDocument {
    pub id: String,
    pub name: String,
    pub file_type: String,
    pub chunk_count: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkDetail {
    pub id: String,
    pub document_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: ChunkDetail,
    pub document_name: String,
}

#[tauri::command]
pub async fn get_indexed_documents(
    app_handle: AppHandle,
) -> Result<Vec<IndexedDocument>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&conn)?;
    vectors::init_vector_tables(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.name, d.file_type, d.created_at,
                    (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) as chunk_count
             FROM documents d
             WHERE d.status = 'indexed'
             ORDER BY d.created_at DESC",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(IndexedDocument {
                id: row.get(0)?,
                name: row.get(1)?,
                file_type: row.get(2)?,
                created_at: row.get(3)?,
                chunk_count: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let documents: Vec<IndexedDocument> = rows
        .filter_map(|r| r.ok())
        .collect();

    Ok(documents)
}

#[tauri::command]
pub async fn get_document_chunks_detail(
    doc_id: String,
    app_handle: AppHandle,
) -> Result<Vec<ChunkDetail>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    vectors::init_vector_tables(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, document_id, chunk_index, content, created_at
             FROM chunks WHERE document_id = ?1 ORDER BY chunk_index",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(rusqlite::params![doc_id], |row| {
            Ok(ChunkDetail {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_index: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let chunks: Vec<ChunkDetail> = rows
        .filter_map(|r| r.ok())
        .collect();

    Ok(chunks)
}

#[tauri::command]
pub async fn search_chunks(
    keyword: String,
    app_handle: AppHandle,
) -> Result<Vec<SearchResult>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&conn)?;
    vectors::init_vector_tables(&conn)?;

    let search_pattern = format!("%{}%", keyword);

    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.document_id, c.chunk_index, c.content, c.created_at, d.name
             FROM chunks c
             JOIN documents d ON c.document_id = d.id
             WHERE c.content LIKE ?1 AND d.status = 'indexed'
             ORDER BY c.created_at DESC
             LIMIT 50",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(rusqlite::params![search_pattern], |row| {
            Ok(SearchResult {
                chunk: ChunkDetail {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    chunk_index: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                },
                document_name: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let results: Vec<SearchResult> = rows
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
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

- [ ] **Step 2: 更新 commands/mod.rs**

在 `src-tauri/src/commands/mod.rs` 中添加:

```rust
pub mod knowledge;
```

- [ ] **Step 3: 更新 lib.rs**

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中添加:

```rust
commands::knowledge::get_indexed_documents,
commands::knowledge::get_document_chunks_detail,
commands::knowledge::search_chunks,
```

- [ ] **Step 4: 运行 cargo check**

Run: `cd /Users/mac/project/telepathy && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/commands/knowledge.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(backend): add knowledge base commands for indexed documents and search"
```

---

### Task 2: 前端 - 添加类型定义和 API

**Files:**
- Create: `src/types/knowledge.ts`
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 创建类型定义**

创建 `src/types/knowledge.ts`:

```typescript
export interface IndexedDocument {
  id: string;
  name: string;
  file_type: string;
  chunk_count: number;
  created_at: string;
}

export interface ChunkDetail {
  id: string;
  document_id: string;
  chunk_index: number;
  content: string;
  created_at: string;
}

export interface SearchResult {
  chunk: ChunkDetail;
  document_name: string;
}
```

- [ ] **Step 2: 添加 API 函数**

在 `src/api/tauri.ts` 末尾添加:

```typescript
export async function getIndexedDocuments(): Promise<IndexedDocument[]> {
  return invoke<IndexedDocument[]>('get_indexed_documents');
}

export async function getDocumentChunksDetail(docId: string): Promise<ChunkDetail[]> {
  return invoke<ChunkDetail[]>('get_document_chunks_detail', { docId });
}

export async function searchChunks(keyword: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('search_chunks', { keyword });
}
```

- [ ] **Step 3: 提交**

```bash
git add src/types/knowledge.ts src/api/tauri.ts
git commit -m "feat(frontend): add knowledge base types and API"
```

---

### Task 3: 前端 - 实现知识库页面

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 实现完整的知识库页面**

将 `src/views/KnowledgeBase.vue` 替换为:

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { NInput, NCard, NEmpty, NSpin, NTag, NButton } from 'naive-ui';
import { getIndexedDocuments, getDocumentChunksDetail, searchChunks } from '@/api/tauri';
import type { IndexedDocument, ChunkDetail, SearchResult } from '@/types/knowledge';

const documents = ref<IndexedDocument[]>([]);
const selectedDocId = ref<string | null>(null);
const chunks = ref<ChunkDetail[]>([]);
const searchKeyword = ref('');
const searchResults = ref<SearchResult[]>([]);
const isLoading = ref(false);
const isSearching = ref(false);
const activeTab = ref<'list' | 'search'>('list');

const selectedDocument = computed(() =>
  documents.value.find(d => d.id === selectedDocId.value)
);

async function loadDocuments() {
  isLoading.value = true;
  try {
    documents.value = await getIndexedDocuments();
  } catch (e) {
    console.error('Failed to load documents:', e);
  } finally {
    isLoading.value = false;
  }
}

async function selectDocument(docId: string) {
  selectedDocId.value = docId;
  isLoading.value = true;
  try {
    chunks.value = await getDocumentChunksDetail(docId);
  } catch (e) {
    console.error('Failed to load chunks:', e);
    chunks.value = [];
  } finally {
    isLoading.value = false;
  }
}

async function handleSearch() {
  if (!searchKeyword.value.trim()) {
    searchResults.value = [];
    return;
  }
  isSearching.value = true;
  try {
    searchResults.value = await searchChunks(searchKeyword.value.trim());
    activeTab.value = 'search';
  } catch (e) {
    console.error('Failed to search:', e);
    searchResults.value = [];
  } finally {
    isSearching.value = false;
  }
}

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleString('zh-CN');
  } catch {
    return dateStr;
  }
}

function highlightKeyword(content: string, keyword: string): string {
  if (!keyword) return content;
  const regex = new RegExp(`(${keyword})`, 'gi');
  return content.replace(regex, '<mark>$1</mark>');
}

onMounted(() => {
  loadDocuments();
});
</script>

<template>
  <div class="knowledge-base">
    <div class="kb-header">
      <h2>知识库</h2>
      <div class="search-box">
        <NInput
          v-model:value="searchKeyword"
          placeholder="搜索知识库..."
          style="width: 300px"
          @keyup.enter="handleSearch"
        />
        <NButton type="primary" @click="handleSearch" :loading="isSearching">
          搜索
        </NButton>
      </div>
    </div>

    <div class="kb-content">
      <NSpin :show="isLoading">
        <div v-if="activeTab === 'list'" class="documents-panel">
          <div v-if="documents.length === 0" class="empty-state">
            <NEmpty description="暂无已索引的文档">
              <template #extra>
                <NButton size="small" type="primary" @click="$router.push('/documents')">
                  去导入文档
                </NButton>
              </template>
            </NEmpty>
          </div>
          <div v-else class="document-list">
            <NCard
              v-for="doc in documents"
              :key="doc.id"
              :class="{ selected: doc.id === selectedDocId }"
              class="document-card"
              hoverable
              @click="selectDocument(doc.id)"
            >
              <div class="doc-info">
                <div class="doc-name">{{ doc.name }}</div>
                <div class="doc-meta">
                  <NTag size="tiny" :bordered="false">.{{ doc.file_type }}</NTag>
                  <span class="chunk-count">{{ doc.chunk_count }} 个文本块</span>
                  <span class="doc-date">{{ formatDate(doc.created_at) }}</span>
                </div>
              </div>
            </NCard>
          </div>
        </div>

        <div v-else class="search-panel">
          <div v-if="searchResults.length === 0" class="empty-state">
            <NEmpty :description="`未找到"${searchKeyword}"相关结果`" />
          </div>
          <div v-else class="search-results">
            <div class="results-header">
              找到 {{ searchResults.length }} 个相关文本块
              <NButton size="small" quaternary @click="activeTab = 'list'">返回列表</NButton>
            </div>
            <div
              v-for="result in searchResults"
              :key="result.chunk.id"
              class="result-item"
            >
              <div class="result-doc">{{ result.document_name }}</div>
              <div
                class="result-content"
                v-html="highlightKeyword(result.chunk.content, searchKeyword)"
              ></div>
            </div>
          </div>
        </div>
      </NSpin>

      <div v-if="selectedDocId && activeTab === 'list'" class="chunks-panel">
        <h3>文档详情</h3>
        <div class="selected-doc-name">{{ selectedDocument?.name }}</div>
        <div v-if="chunks.length === 0" class="empty-chunks">
          <NEmpty description="暂无文本块" />
        </div>
        <div v-else class="chunk-list">
          <div v-for="(chunk, idx) in chunks" :key="chunk.id" class="chunk-item">
            <div class="chunk-header">
              <NTag size="small" type="info">块 {{ chunk.chunk_index + 1 }}</NTag>
            </div>
            <div class="chunk-content">{{ chunk.content }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.knowledge-base {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.kb-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.kb-header h2 {
  margin: 0;
}

.search-box {
  display: flex;
  gap: 8px;
}

.kb-content {
  flex: 1;
  display: flex;
  gap: 24px;
  overflow: hidden;
}

.documents-panel {
  flex: 1;
  overflow-y: auto;
  min-width: 300px;
}

.document-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.document-card {
  cursor: pointer;
  transition: all 0.2s;
}

.document-card.selected {
  border-color: var(--primary-color, #18a058);
  background: rgba(24, 160, 88, 0.05);
}

.doc-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.doc-name {
  font-weight: 500;
  font-size: 14px;
}

.doc-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #666;
}

.chunk-count {
  color: #18a058;
}

.chunks-panel {
  flex: 1;
  overflow-y: auto;
  border-left: 1px solid #eee;
  padding-left: 24px;
  min-width: 400px;
}

.chunks-panel h3 {
  margin: 0 0 8px 0;
}

.selected-doc-name {
  color: #666;
  margin-bottom: 16px;
}

.empty-state {
  padding: 48px;
}

.empty-chunks {
  padding: 24px;
}

.chunk-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.chunk-item {
  border: 1px solid #eee;
  border-radius: 8px;
  padding: 12px;
}

.chunk-header {
  margin-bottom: 8px;
}

.chunk-content {
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  color: #333;
}

.search-panel {
  flex: 1;
  overflow-y: auto;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  color: #666;
}

.result-item {
  border: 1px solid #eee;
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}

.result-doc {
  font-weight: 500;
  margin-bottom: 8px;
  color: #18a058;
}

.result-content {
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
}

.result-content :deep(mark) {
  background: #fef3c7;
  padding: 0 2px;
  border-radius: 2px;
}
</style>
```

- [ ] **Step 2: 验证文件语法**

Run: `cd /Users/mac/project/telepathy && pnpm type-check 2>&1 | head -30`
Expected: No TypeScript errors

- [ ] **Step 3: 提交**

```bash
git add src/views/KnowledgeBase.vue
git commit -m "feat(frontend): implement knowledge base page with documents and search"
```

---

### Task 4: 验证功能

- [ ] **Step 1: 启动开发服务器**

```bash
pnpm tauri dev
```

- [ ] **Step 2: 手动验证以下行为**

1. 进入知识库页面，应该能看到已索引的文档列表
2. 点击文档，应该能看到其文本块列表
3. 在搜索框输入关键词并点击搜索，应该能看到搜索结果
4. 搜索结果中的关键词应该高亮显示

---

## 验收标准

| # | 验收项 | 验证方式 |
|---|--------|----------|
| 1 | 知识库页面能显示已索引的文档列表 | 导入并索引文档后进入知识库页面 |
| 2 | 点击文档能显示其文本块 | 点击列表中的文档 |
| 3 | 搜索功能能找到包含关键词的文本块 | 输入关键词搜索 |
| 4 | 搜索结果中高亮显示匹配的关键词 | 查看搜索结果 |
| 5 | `cargo check` 无错误 | 终端 |
| 6 | `pnpm type-check` 无错误 | 终端 |