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

export const useDocumentsStore = defineStore('documents', () => {
  const documents = ref<Document[]>([]);
  const documentsPaginated = ref<Document[]>([]);
  const documentsPage = ref(1);
  const documentsHasMore = ref(true);
  const isLoading = ref(false);
  const isLoadingMore = ref(false);
  const isImporting = ref(false);

  async function fetchDocuments(projectId?: string) {
    isLoading.value = true;
    try {
      documents.value = await getDocuments(projectId);
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchDocumentsPaginated(projectId?: string, reset: boolean = true) {
    isLoading.value = true;
    try {
      if (reset) {
        documentsPage.value = 1;
        documentsHasMore.value = true;
      }
      
      const result = await getDocumentsPaginated(projectId, documentsPage.value, 20);
      if (reset) {
        documentsPaginated.value = result.items;
      } else {
        documentsPaginated.value = [...documentsPaginated.value, ...result.items];
      }
      documentsHasMore.value = result.hasMore;
      documentsPage.value++;
    } finally {
      isLoading.value = false;
    }
  }

  async function loadMoreDocuments(projectId?: string) {
    if (isLoadingMore.value || !documentsHasMore.value) return;
    
    isLoadingMore.value = true;
    try {
      await fetchDocumentsPaginated(projectId, false);
    } finally {
      isLoadingMore.value = false;
    }
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
    documents.value = documents.value.filter((d) => d.id !== docId);
    documentsPaginated.value = documentsPaginated.value.filter((d) => d.id !== docId);
  }

  async function runParse(docId: string) {
    const doc = documents.value.find((d) => d.id === docId);
    const docPaginated = documentsPaginated.value.find((d) => d.id === docId);
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
    const doc = documents.value.find((d) => d.id === docId);
    const docPaginated = documentsPaginated.value.find((d) => d.id === docId);
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
    documentsPaginated,
    documentsPage,
    documentsHasMore,
    isLoading,
    isLoadingMore,
    isImporting,
    fetchDocuments,
    fetchDocumentsPaginated,
    loadMoreDocuments,
    addDocument,
    removeDocument,
    runParse,
    runIndex,
  };
});