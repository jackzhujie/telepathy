# 分页与无限滚动功能设计文档

**日期：** 2026-04-10
**作者：** AI Assistant
**版本：** 1.0

## 1. 概述

本设计文档描述了为 Telepathy 项目添加分页和无限滚动功能的完整方案，旨在优化大量数据场景下的性能表现。

## 2. 目标

- 减少内存占用，只加载必要的数据
- 提升列表渲染性能，结合虚拟滚动
- 提供流畅的用户体验
- 保持与现有代码架构的兼容性

## 3. 技术方案

### 3.1 整体架构

```
前端组件 → Pinia Store → Tauri API → Rust 后端 → SQLite 数据库
```

### 3.2 核心概念

- **分页（Pagination）**：将数据按页分割，每次请求一页数据
- **无限滚动（Infinite Scroll）**：滚动到列表底部时自动加载下一页
- **虚拟滚动（Virtual Scrolling）**：只渲染可见区域的 DOM 元素

## 4. 后端设计（Rust）

### 4.1 分页通用类型

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_more: bool,
}
```

### 4.2 数据库层修改

#### 4.2.1 documents.rs
- 新增 `get_documents_paginated(project_id: Option<&str>, page: i32, page_size: i32) -> Result<PaginatedResult<Document>, AppError>`
- 新增 `get_documents_by_project_paginated(project_id: &str, page: i32, page_size: i32) -> Result<PaginatedResult<Document>, AppError>`

#### 4.2.2 conversations.rs
- 新增 `get_conversations_paginated(page: i32, page_size: i32) -> Result<PaginatedResult<Conversation>, AppError>`
- 新增 `get_messages_paginated(conv_id: &str, page: i32, page_size: i32) -> Result<PaginatedResult<ChatMessage>, AppError>`

#### 4.2.3 knowledge 相关
- 新增 `get_indexed_documents_paginated(project_id: Option<&str>, page: i32, page_size: i32) -> Result<PaginatedResult<IndexedDocument>, AppError>`
- 新增 `get_document_chunks_detail_paginated(doc_id: &str, page: i32, page_size: i32) -> Result<PaginatedResult<ChunkDetail>, AppError>`

### 4.3 Tauri 命令

新增以下命令：
- `get_documents_paginated`
- `get_conversations_paginated`
- `get_messages_paginated`
- `get_indexed_documents_paginated`
- `get_document_chunks_detail_paginated`

## 5. 前端设计（Vue + TypeScript）

### 5.1 API 层（api/tauri.ts）

#### 5.1.1 类型定义
```typescript
export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}
```

#### 5.1.2 API 函数
- `getDocumentsPaginated(projectId?: string, page?: number, pageSize?: number): Promise<PaginatedResult<Document>>`
- `getConversationsPaginated(page?: number, pageSize?: number): Promise<PaginatedResult<Conversation>>`
- `getMessagesPaginated(conversationId: string, page?: number, pageSize?: number): Promise<PaginatedResult<ChatMessage>>`
- `getIndexedDocumentsPaginated(projectId?: string, page?: number, pageSize?: number): Promise<PaginatedResult<IndexedDocument>>`
- `getDocumentChunksDetailPaginated(docId: string, page?: number, pageSize?: number): Promise<PaginatedResult<ChunkDetail>>`

### 5.2 Store 层修改

#### 5.2.1 documents.ts
新增状态：
- `documentsPaginated: Document[]`
- `documentsPage: number`
- `documentsPageSize: number`
- `documentsTotal: number`
- `documentsHasMore: boolean`
- `isLoadingMore: boolean`

新增方法：
- `fetchDocumentsPaginated(page?: number, reset?: boolean): Promise<void>`
- `loadMoreDocuments(): Promise<void>`

#### 5.2.2 chat.ts
新增状态：
- `conversationsPaginated: Conversation[]`
- `conversationsPage: number`
- `conversationsHasMore: boolean`
- `messagesPaginated: Message[]`
- `messagesPage: number`
- `messagesHasMore: boolean`

新增方法：
- `fetchConversationsPaginated(page?: number, reset?: boolean): Promise<void>`
- `loadMoreConversations(): Promise<void>`
- `loadConversationPaginated(id: string): Promise<void>`
- `loadMoreMessages(): Promise<void>`

### 5.3 通用 Hook - useInfiniteScroll

创建 `src/hooks/useInfiniteScroll.ts`：

```typescript
import { ref, onMounted, onUnmounted, type Ref } from 'vue';

interface UseInfiniteScrollOptions {
  container: Ref<HTMLElement | null>;
  loadMore: () => Promise<void>;
  hasMore: Ref<boolean>;
  isLoading: Ref<boolean>;
  threshold?: number;
}

export function useInfiniteScroll(options: UseInfiniteScrollOptions) {
  const { container, loadMore, hasMore, isLoading, threshold = 200 } = options;

  async function handleScroll() {
    if (!container.value || isLoading.value || !hasMore.value) return;

    const { scrollTop, scrollHeight, clientHeight } = container.value;
    const distanceToBottom = scrollHeight - scrollTop - clientHeight;

    if (distanceToBottom < threshold) {
      await loadMore();
    }
  }

  onMounted(() => {
    container.value?.addEventListener('scroll', handleScroll, { passive: true });
  });

  onUnmounted(() => {
    container.value?.removeEventListener('scroll', handleScroll);
  });

  return {};
}
```

### 5.4 页面组件修改

#### 5.4.1 Documents.vue
- 使用 `useInfiniteScroll` Hook
- 显示加载更多指示器
- 保持虚拟滚动

#### 5.4.2 KnowledgeBase.vue
- 三个列表都应用无限滚动
- 文档列表、搜索结果、文本块详情都支持分页

#### 5.4.3 Chat.vue
- 消息列表支持分页（从旧到新加载）
- 会话侧边栏支持分页

#### 5.4.4 ConversationSidebar.vue
- 使用无限滚动加载会话列表

## 6. UI/UX 设计

### 6.1 加载状态

- **首次加载**：显示骨架屏
- **加载更多**：列表底部显示加载指示器（旋转动画）
- **加载失败**：显示错误信息和重试按钮

### 6.2 空状态

- 暂无数据时显示友好的插画和提示文字
- 提供操作引导（如"去导入文档"）

### 6.3 错误处理

- 网络错误时显示 Toast 通知
- 提供重试机制
- 记录错误日志

## 7. 技术参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `page_size` | 20 | 每页数据条数 |
| `threshold` | 200px | 无限滚动触发阈值（距离底部） |
| `overscan` | 5 | 虚拟滚动预渲染数量 |

## 8. 兼容性

- 保留原有的一次性加载 API 作为备用
- 新的分页 API 与现有功能兼容
- 渐进式采用，可逐步迁移

## 9. 测试计划

- 单元测试：分页函数、Store 方法
- 集成测试：完整的数据加载流程
- 性能测试：大数据量场景下的表现
- 用户体验测试：滚动流畅度、加载反馈

## 10. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 后端修改影响现有功能 | 高 | 保持原有 API 不变，新增分页 API |
| 分页数据排序问题 | 中 | 统一按 created_at DESC 排序 |
| 重复加载 | 低 | 使用 loading 状态防止重复请求 |
