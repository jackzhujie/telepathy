# 知识库虚拟化与滚动加载实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 优化知识库页面的文档列表和文本块详情页，引入虚拟滚动和分页加载，支持海量数据的高性能展示。

**Architecture:** 使用 Pinia Store (`knowledge.ts`) 统一管理分页状态，在 `KnowledgeBase.vue` 中集成 `@tanstack/vue-virtual` 实现动态高度虚拟列表。

**Tech Stack:** Vue 3, Pinia, @tanstack/vue-virtual, Tauri (Rust backend already supports pagination)

---

### Task 1: 创建 Knowledge Store

**Files:**
- Create: `src/stores/knowledge.ts`
- Modify: `src/api/tauri.ts` (ensure exports are correct)

- [ ] **Step 1: 编写 Knowledge Store 基础结构**

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { IndexedDocument, ChunkDetail } from '@/types/knowledge';
import { getIndexedDocumentsPaginated, getDocumentChunksDetailPaginated } from '@/api/tauri';

export const useKnowledgeStore = defineStore('knowledge', () => {
  // 文档列表状态
  const documents = ref<IndexedDocument[]>([]);
  const docPage = ref(1);
  const docHasMore = ref(true);
  const isDocsLoading = ref(false);
  const docTotal = ref(0);

  // 文本块详情状态
  const chunks = ref<ChunkDetail[]>([]);
  const chunkPage = ref(1);
  const chunkHasMore = ref(true);
  const isChunksLoading = ref(false);
  const chunkTotal = ref(0);
  const currentDocId = ref<string | null>(null);

  // Actions: 文档加载
  async function fetchDocuments(projectId?: string, reset: boolean = true) {
    if (isDocsLoading.value) return;
    isDocsLoading.value = true;
    try {
      if (reset) {
        docPage.value = 1;
        docHasMore.value = true;
      }
      const result = await getIndexedDocumentsPaginated(projectId, docPage.value, 20);
      if (reset) {
        documents.value = result.items;
      } else {
        documents.value = [...documents.value, ...result.items];
      }
      docTotal.value = result.total;
      docHasMore.value = result.has_more;
      docPage.value++;
    } finally {
      isDocsLoading.value = false;
    }
  }

  // Actions: 文本块加载
  async function fetchChunks(docId: string, reset: boolean = true) {
    if (isChunksLoading.value) return;
    isChunksLoading.value = true;
    currentDocId.value = docId;
    try {
      if (reset) {
        chunkPage.value = 1;
        chunkHasMore.value = true;
        chunks.value = [];
      }
      const result = await getDocumentChunksDetailPaginated(docId, chunkPage.value, 50);
      if (reset) {
        chunks.value = result.items;
      } else {
        chunks.value = [...chunks.value, ...result.items];
      }
      chunkTotal.value = result.total;
      chunkHasMore.value = result.has_more;
      chunkPage.value++;
    } finally {
      isChunksLoading.value = false;
    }
  }

  return {
    documents, docHasMore, isDocsLoading, docTotal,
    chunks, chunkHasMore, isChunksLoading, chunkTotal, currentDocId,
    fetchDocuments, fetchChunks
  };
});
```

- [ ] **Step 2: 验证 Store 逻辑**
可以通过临时导入到 App.vue 或其他组件中调用并观察 console。

- [ ] **Step 3: Commit**

```bash
git add src/stores/knowledge.ts
git commit -m "feat: add knowledge store for paginated loading"
```

---

### Task 2: 重构 KnowledgeBase.vue 引入虚拟化

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 引入依赖和 Store**

```typescript
import { useVirtualizer } from '@tanstack/vue-virtual';
import { useKnowledgeStore } from '@/stores/knowledge';
// 替换原有的本地 ref
const knowledgeStore = useKnowledgeStore();
```

- [ ] **Step 2: 实现左侧文档虚拟列表**
- 使用 `useVirtualizer` 绑定文档列表。
- 实现 `scrollToIndex` 逻辑（如果切换时需要重置）。

- [ ] **Step 3: 实现右侧文本块虚拟列表（动态高度）**
- 使用 `useVirtualizer` 的 `estimateSize` 和测量机制。
- 确保在切换文档时重置虚拟化状态。

- [ ] **Step 4: 添加滚动到底部触发加载更多逻辑**

- [ ] **Step 5: Commit**

```bash
git add src/views/KnowledgeBase.vue
git commit -m "feat: implement virtualization and infinite scroll in KnowledgeBase"
```

---

### Task 3: UI 美化与加载状态

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 添加“加载更多”指示器**
当 `isChunksLoading` 为 true 且已存在部分数据时，在列表底部显示一个简单的 Spinner 或文字提示。

- [ ] **Step 2: 处理空状态与边界**
完善 `total === 0` 或搜索无结果时的展示。

- [ ] **Step 3: Commit**

```bash
git add src/views/KnowledgeBase.vue
git commit -m "style: add loading indicators and improve empty states"
```
