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
      console.log('[Debug Store] API Result received:', result);
      
      if (reset) {
        documents.value = result.items;
      } else {
        documents.value = [...documents.value, ...result.items];
      }
      
      docTotal.value = result.total;
      // 这里的 hasMore 如果是 undefined，可能是命名不匹配
      docHasMore.value = result.hasMore === true; 
      console.log('[Debug Store] Updated state:', {
        total: docTotal.value,
        hasMore: docHasMore.value,
        page: docPage.value
      });
      docPage.value++;
    } catch (e) {
      console.error('Failed to fetch documents:', e);
    } finally {
      isDocsLoading.value = false;
    }
  }

  // 加载更多文档
  async function loadMoreDocuments(projectId?: string) {
    if (isDocsLoading.value || !docHasMore.value) return;
    await fetchDocuments(projectId, false);
  }

  // Actions: 文本块加载
  async function fetchChunks(docId: string, reset: boolean = true) {
    if (isChunksLoading.value) return;
    
    currentDocId.value = docId;
    isChunksLoading.value = true;
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
      chunkHasMore.value = result.hasMore;
      chunkPage.value++;
    } catch (e) {
      console.error('Failed to fetch chunks:', e);
    } finally {
      isChunksLoading.value = false;
    }
  }

  // 加载更多文本块
  async function loadMoreChunks() {
    if (isChunksLoading.value || !chunkHasMore.value || !currentDocId.value) return;
    await fetchChunks(currentDocId.value, false);
  }

  return {
    documents,
    docHasMore,
    isDocsLoading,
    docTotal,
    chunks,
    chunkHasMore,
    isChunksLoading,
    chunkTotal,
    currentDocId,
    docPage,
    chunkPage,
    fetchDocuments,
    loadMoreDocuments,
    fetchChunks,
    loadMoreChunks
  };
});
