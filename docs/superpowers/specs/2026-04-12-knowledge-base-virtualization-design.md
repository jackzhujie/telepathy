# 设计文档：知识库页面虚拟滚动与滚动加载优化

## 1. 背景与目标
随着知识库中文档数量和文本块（Chunks）数量的增加，传统的全量渲染方式会导致内存占用过高和界面卡顿。
本设计的目标是实现高性能的虚拟滚动和按需加载，支持海量数据的平滑展示。

## 2. 系统架构

### 2.1 状态管理 (`src/stores/knowledge.ts`)
新增一个专门的 Store 处理知识库相关的分页数据。

**State:**
- `documents`: `IndexedDocument[]` (侧边栏文档)
- `docPagination`: `{ page: number, hasMore: boolean, loading: boolean }`
- `chunks`: `ChunkDetail[]` (选中文档的文本块)
- `chunkPagination`: `{ page: number, hasMore: boolean, loading: boolean, currentDocId: string | null }`

**Actions:**
- `fetchDocuments(reset: boolean)`: 调用 `get_indexed_documents_paginated`
- `loadMoreDocuments()`: 加载下一页文档
- `fetchChunks(docId: string, reset: boolean)`: 调用 `get_document_chunks_detail_paginated`
- `loadMoreChunks()`: 加载下一页文本块

### 2.2 前端视图 (`src/views/KnowledgeBase.vue`)
重构现有模板，集成虚拟滚动。

**组件结构:**
- **Sidebar List**: 使用 `useVirtualizer` 渲染文档卡片。
  - `estimateSize`: 120px
  - 监听列表末尾滚动事件触发 `loadMoreDocuments`。
- **Detail List**: 使用 `useVirtualizer` 渲染文本块卡片。
  - `estimateSize`: 150px
  - **动态高度支持**: 使用虚拟滚动库的动态测量，并监听 DOM 尺寸变化。

## 3. 关键逻辑

### 3.1 分页加载逻辑
- 初始加载页码为 1。
- 每次加载完成后，根据返回的 `total` 和 `page_size` 计算 `hasMore`。
- 滚动到列表底部 20% 位置时，预触发下一页加载。

### 3.2 动态高度测量
- 每个文本块卡片渲染后，使用 `observerElement` 进行测量。
- 缓存测量结果，避免滚动时的布局抖动。

### 3.3 交互逻辑
- 切换文档时，清空当前 `chunks` 数组，重置 `chunkPagination`，并立即加载第一页。
- 搜索结果目前保持全量搜索（后端限制前 50 条），但若后期量大，也可接入此套分页逻辑。

## 4. 异常处理
- 加载失败时显示重试按钮。
- 网络延迟时在列表底部显示 Loading 占位符。

## 5. 验收标准
- 侧边栏支持流畅滚动 100+ 文档。
- 详情页支持流畅滚动 5000+ 文本块。
- 切换文档时 UI 响应时间 < 200ms。
