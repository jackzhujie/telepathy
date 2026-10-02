<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useVirtualizer } from '@tanstack/vue-virtual';
import { useKnowledgeStore } from '@/stores/knowledge';
import { useProjectsStore } from '@/stores/projects';
import { searchChunks, deleteDocument } from '@/api/tauri';
import {
  SelectContent,
  SelectIcon,
  SelectItem,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from 'reka-ui';
import type { SearchResult } from '@/types/knowledge';

const projectsStore = useProjectsStore();
const knowledgeStore = useKnowledgeStore();

const selectedDocId = ref<string | null>(null);
const searchKeyword = ref('');
const searchResults = ref<SearchResult[]>([]);
const isSearching = ref(false);
const showDeleteModal = ref(false);
const docIdToDelete = ref<string | null>(null);
const activeTab = ref<'list' | 'search'>('list');
const selectedProjectId = ref<string | null>(null);

// 虚拟滚动相关容器引用
const docListContainer = ref<HTMLElement | null>(null);
const chunkListContainer = ref<HTMLElement | null>(null);

const projectOptions = computed(() => {
  return projectsStore.projects.map((p) => ({ label: p.name, value: p.id }));
});

const selectedDocument = computed(() =>
  knowledgeStore.documents.find(d => d.id === selectedDocId.value)
);

// --- 虚拟滚动：文档列表 ---
const docVirtualizer = useVirtualizer({
  get count() { return knowledgeStore.documents.length },
  getScrollElement: () => docListContainer.value,
  estimateSize: () => 120,
  overscan: 5,
});

// --- 虚拟滚动：文本块列表 ---
const chunkVirtualizer = useVirtualizer({
  get count() { return knowledgeStore.chunks.length },
  getScrollElement: () => chunkListContainer.value,
  estimateSize: () => 150,
  overscan: 10,
});

const loadMoreTrigger = ref<HTMLElement | null>(null);
const chunkLoadMoreTrigger = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;
let chunkObserver: IntersectionObserver | null = null;

async function handleProjectChange() {
  selectedDocId.value = null;
  knowledgeStore.chunks = [];
  await knowledgeStore.fetchDocuments(selectedProjectId.value || undefined, true);
}

async function selectDocument(docId: string) {
  selectedDocId.value = docId;
  await knowledgeStore.fetchChunks(docId, true);
}

function handleDeleteDoc(docId: string) {
  docIdToDelete.value = docId;
  showDeleteModal.value = true;
}

async function confirmDeleteDoc() {
  if (!docIdToDelete.value) return;
  try {
    await deleteDocument(docIdToDelete.value);
    knowledgeStore.documents = knowledgeStore.documents.filter(d => d.id !== docIdToDelete.value);
    if (selectedDocId.value === docIdToDelete.value) {
      selectedDocId.value = null;
      knowledgeStore.chunks = [];
    }
  } catch (e) {
    console.error('删除文档失败:', e);
  } finally {
    showDeleteModal.value = false;
    docIdToDelete.value = null;
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

// --- 虚拟滚动：搜索结果列表 ---
const searchListContainer = ref<HTMLElement | null>(null);
const searchVirtualizer = useVirtualizer({
  get count() { return searchResults.value.length },
  getScrollElement: () => searchListContainer.value,
  estimateSize: () => 150,
  overscan: 10,
});

function highlightKeyword(content: string, keyword: string): string {
  if (!keyword) return content;
  const regex = new RegExp(`(${keyword})`, 'gi');
  return content.replace(regex, '<mark>$1</mark>');
}

onMounted(async () => {
  await projectsStore.fetchProjects();
  
  if (projectsStore.projects.length > 0 && !selectedProjectId.value) {
    selectedProjectId.value = projectsStore.projects[0].id;
    await knowledgeStore.fetchDocuments(selectedProjectId.value, true);
  } else if (selectedProjectId.value) {
    await knowledgeStore.fetchDocuments(selectedProjectId.value, true);
  } else {
    await knowledgeStore.fetchDocuments(undefined, true);
  }

  // 初始化侧边栏观察器
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && knowledgeStore.docHasMore && !knowledgeStore.isDocsLoading) {
      console.log('[API Call] Loading next page via Observer:', knowledgeStore.docPage);
      knowledgeStore.loadMoreDocuments(selectedProjectId.value || undefined);
    }
  }, { 
    root: docListContainer.value,
    threshold: 0.01,
    rootMargin: '100px' 
  });

  // 初始化详情列表观察器
  chunkObserver = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && knowledgeStore.chunkHasMore && !knowledgeStore.isChunksLoading && selectedDocId.value) {
      console.log('[API Call] Loading next chunk page:', knowledgeStore.chunkPage);
      knowledgeStore.loadMoreChunks();
    }
  }, {
    root: chunkListContainer.value,
    threshold: 0.01,
    rootMargin: '100px'
  });

  if (loadMoreTrigger.value) observer.observe(loadMoreTrigger.value);
  if (chunkLoadMoreTrigger.value) chunkObserver.observe(chunkLoadMoreTrigger.value);
});

// 监听 trigger 元素的动态出现
watch(loadMoreTrigger, (newEl, oldEl) => {
  if (oldEl) observer?.unobserve(oldEl);
  if (newEl) observer?.observe(newEl);
});

watch(chunkLoadMoreTrigger, (newEl, oldEl) => {
  if (oldEl) chunkObserver?.unobserve(oldEl);
  if (newEl) chunkObserver?.observe(newEl);
});

onUnmounted(() => {
  if (observer) observer.disconnect();
  if (chunkObserver) chunkObserver.disconnect();
});

// 当文档切换时，重置文本块虚拟列表位置
watch(selectedDocId, () => {
  chunkVirtualizer.value?.scrollToOffset(0);
});

// 记录刚刚成功复制的 Chunk ID，用于展示复制成功提示
const copiedChunkId = ref<string | null>(null);

async function copyChunkContent(chunkId: string, text: string) {
  try {
    await navigator.clipboard.writeText(text);
    copiedChunkId.value = chunkId;
    setTimeout(() => {
      if (copiedChunkId.value === chunkId) copiedChunkId.value = null;
    }, 2000);
  } catch (err) {
    console.error('Failed to copy text:', err);
  }
}

// 获取各种文件类型的彩色图标和样式
const getFileIcon = (fileType: string) => {
  const ext = fileType.toLowerCase();
  switch (ext) {
    case 'pdf':
      return {
        bgClass: 'bg-red-500/10 text-red-500 border-red-500/20',
        path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6'
      };
    case 'md':
    case 'txt':
      return {
        bgClass: 'bg-indigo-500/10 text-indigo-500 border-indigo-500/20',
        path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M16 13H8 M16 17H8 M10 9H8'
      };
    case 'doc':
    case 'docx':
      return {
        bgClass: 'bg-blue-500/10 text-blue-500 border-blue-500/20',
        path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M16 12H8 M16 16H8'
      };
    case 'csv':
    case 'xlsx':
    case 'xls':
      return {
        bgClass: 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20',
        path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M8 13h2v2H8zm4 0h2v2h-2zm4 0h2v2h-2zm-8 4h2v2H8zm4 0h2v2h-2zm4 0h2v2h-2z'
      };
    case 'json':
      return {
        bgClass: 'bg-amber-500/10 text-amber-500 border-amber-500/20',
        path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M8 12h8 M8 16h8'
      };
    default:
      return {
        bgClass: 'bg-text-muted/10 text-text-muted border-text-muted/20',
        path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6'
      };
  }
};
</script>

<template>
  <div class="flex flex-col h-full bg-app-bg/40 backdrop-blur-md px-1 select-text">
    <div class="flex justify-between items-center mb-4 pb-4 border-b border-border-main/35">
      <h2 class="font-black text-xl bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary tracking-tight">知识库</h2>
      <div class="flex items-center gap-3 flex-wrap">
        <SelectRoot v-model="selectedProjectId" @update:model-value="handleProjectChange">
          <SelectTrigger
            class="w-48 px-3 py-2 border border-border-main/40 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand/30 transition-all duration-300 bg-panel-bg/45 backdrop-blur-md text-text-primary flex items-center justify-between hover:border-brand/40 shadow-sm text-xs"
          >
            <SelectValue placeholder="选择项目" />
            <SelectIcon>
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </SelectIcon>
          </SelectTrigger>
          <SelectPortal>
            <SelectContent class="bg-panel-bg/60 backdrop-blur-md rounded-xl shadow-xl border border-border-main/40 overflow-hidden z-50 animate-fade-in" position="popper" :side-offset="5">
              <SelectViewport class="p-1">
                <SelectItem
                  v-for="option in projectOptions"
                  :key="option.value"
                  :value="option.value"
                  class="relative flex items-center px-4 py-2 rounded-lg text-xs cursor-pointer text-text-primary hover:bg-brand/10 data-[highlighted]:bg-brand/10 transition-all duration-200"
                >
                  <SelectItemText>{{ option.label }}</SelectItemText>
                </SelectItem>
              </SelectViewport>
            </SelectContent>
          </SelectPortal>
        </SelectRoot>
        <div class="flex gap-2.5 items-center">
          <input
            v-model="searchKeyword"
            @keyup.enter="handleSearch"
            placeholder="搜索知识库..."
            class="px-3.5 py-2 border border-border-main/40 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand/30 transition-all duration-300 bg-panel-bg/45 backdrop-blur-md text-text-primary placeholder:text-text-muted/60 text-xs shadow-sm hover:border-border-main/60"
          />
          <button 
            class="px-3.5 py-2.5 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 text-white rounded-xl font-bold text-xs transition-all duration-300 hover:scale-[1.02] active:scale-98 flex items-center justify-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed shadow-md"
            :disabled="isSearching"
            @click="handleSearch"
          >
            <span v-if="isSearching" class="animate-spin rounded-full h-3 w-3 border-t-2 border-b-2 border-white inline-block"></span>
            搜索
          </button>
        </div>
      </div>
    </div>


    <div class="flex flex-1 gap-4 overflow-hidden min-h-0 h-0">
      <div class="flex flex-col flex-1 min-h-0 min-w-0">
        <div v-if="activeTab === 'list'" class="flex-1 min-h-0 flex flex-col min-w-[320px] max-w-2xl relative">
          <div v-if="knowledgeStore.isDocsLoading && knowledgeStore.documents.length === 0" class="flex-1 flex justify-center items-center py-8 border border-brand/20">
            <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-brand"></div>
          </div>
          
          <div v-else-if="knowledgeStore.documents.length === 0" class="flex flex-col items-center justify-center py-8 flex-1 border border-brand/20">
            <div class="text-text-muted text-3xl mb-3">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </div>
            <p class="text-text-secondary mb-3">暂无已索引的文档</p>
            <button 
              class="px-3 py-2 bg-brand hover:bg-brand/90 text-white rounded-md font-semibold transition-colors"
              @click="$router.push('/documents')"
            >
              去导入文档
            </button>
          </div>
          
          <div v-else ref="docListContainer" class="flex-1 overflow-y-auto">
            <div :style="{ height: `${docVirtualizer.getTotalSize()}px` }" class="relative w-full">
              <div
                v-for="virtualRow in docVirtualizer.getVirtualItems()"
                :key="virtualRow.index"
                :data-index="virtualRow.index"
                :ref="el => { if (el) docVirtualizer.measureElement(el as HTMLElement) }"
                :style="{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  transform: `translateY(${virtualRow.start}px)`,
                }"
                class="px-1 py-1.5"
              >
                <div
                  :class="knowledgeStore.documents[virtualRow.index].id === selectedDocId ? 'border border-brand/60 bg-brand/5 backdrop-blur-md shadow-md font-semibold -translate-y-[1px]' : 'border border-border-main/35 bg-panel-bg/40 backdrop-blur-md hover:border-brand/30 hover:shadow-md hover:-translate-y-[1px]'"
                  class="relative cursor-pointer transition-all duration-300 rounded-2xl h-full shadow-sm group overflow-hidden"
                  @click="selectDocument(knowledgeStore.documents[virtualRow.index].id)"
                >
                  <!-- Selected indicator stripe -->
                  <div
                    v-if="knowledgeStore.documents[virtualRow.index].id === selectedDocId"
                    class="absolute left-0 top-0 bottom-0 w-[3px] bg-gradient-to-b from-brand to-brand-600 rounded-r"
                  ></div>
                  
                  <div class="p-3.5 pl-4">
                    <div class="flex flex-col gap-2.5">
                      <div class="flex items-center justify-between gap-2.5">
                        <div class="flex items-center gap-2.5 min-w-0 flex-1">
                          <!-- Color Document Icon badge -->
                          <div
                            class="p-2 border rounded-xl flex items-center justify-center flex-shrink-0"
                            :class="getFileIcon(knowledgeStore.documents[virtualRow.index].file_type).bgClass"
                          >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                              <path :d="getFileIcon(knowledgeStore.documents[virtualRow.index].file_type).path" />
                            </svg>
                          </div>
                          <div class="font-bold text-sm text-text-primary truncate flex-1 leading-normal">{{ knowledgeStore.documents[virtualRow.index].name }}</div>
                        </div>
                        <button
                          class="p-1.5 text-text-muted hover:text-danger hover:bg-danger/10 rounded-lg transition-colors duration-200 opacity-0 group-hover:opacity-100 focus:opacity-100"
                          title="删除此文档"
                          @click.stop="handleDeleteDoc(knowledgeStore.documents[virtualRow.index].id)"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-4v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                          </svg>
                        </button>
                      </div>
                      <div class="flex items-center gap-2.5 flex-wrap pl-[38px]">
                        <span class="text-brand font-bold text-xs select-none">{{ knowledgeStore.documents[virtualRow.index].chunk_count }} 个块</span>
                        <span class="w-1 h-1 bg-border-main/50 rounded-full select-none"></span>
                        <span class="text-text-muted text-[10px] select-none leading-none">{{ formatDate(knowledgeStore.documents[virtualRow.index].created_at) }}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <!-- 滚动加载指示器 -->
            <div v-if="knowledgeStore.docHasMore" ref="loadMoreTrigger" class="py-4 flex justify-center h-10 items-center">
              <div class="animate-spin rounded-full h-4 w-4 border-t-2 border-b-2 border-brand"></div>
            </div>
          </div>
        </div>

        <div v-else class="flex-1 flex flex-col min-h-0">
          <div v-if="searchResults.length === 0" class="flex-1 flex flex-col items-center justify-center py-8">
            <div class="text-text-muted text-3xl mb-3">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
            <p class="text-text-secondary">未找到 "{{ searchKeyword }}" 相关结果</p>
          </div>
          <div v-else class="flex-1 flex flex-col min-h-0">
            <div class="flex justify-between items-center mb-3 pb-2 border-b border-border-main/50">
              <span class="text-text-secondary font-medium">找到 {{ searchResults.length }} 个相关文本块</span>
              <button 
                class="px-3 py-1 border-2 border-border-main-light hover:bg-surface-bg text-text-secondary rounded-md text-xs transition-colors"
                @click="activeTab = 'list'"
              >
                返回列表
              </button>
            </div>
            <div ref="searchListContainer" class="flex-1 overflow-y-auto">
              <div :style="{ height: `${searchVirtualizer.getTotalSize()}px` }" class="relative w-full">
                <div
                  v-for="virtualRow in searchVirtualizer.getVirtualItems()"
                  :key="virtualRow.index"
                  :data-index="virtualRow.index"
                  :ref="el => { if (el) searchVirtualizer.measureElement(el as HTMLElement) }"
                  :style="{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    transform: `translateY(${virtualRow.start}px)`,
                  }"
                  class="pb-3 pr-2"
                >
                  <div class="border-2 border-border-main rounded-md hover:shadow-lg hover:shadow-brand/5 hover:border-brand/30 transition-all">
                    <div class="p-3">
                      <div class="font-semibold text-brand mb-2 truncate text-xs">{{ searchResults[virtualRow.index].document_name }}</div>
                      <div
                        class="text-text-secondary text-xs whitespace-pre-wrap leading-relaxed"
                        v-html="highlightKeyword(searchResults[virtualRow.index].chunk.content, searchKeyword)"
                      ></div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-h-0 border-l border-border-main/20 pl-6 min-w-[320px]">
        <div v-if="selectedDocId && activeTab === 'list'" class="flex-1 flex flex-col min-h-0">
          <h3 class="text-sm font-bold mb-2 flex items-center justify-between text-text-primary select-none">
            <span>文档详情</span>
            <span v-if="knowledgeStore.chunkTotal > 0" class="text-[10px] text-text-muted font-normal select-none">
              共 {{ knowledgeStore.chunkTotal }} 个文本块
            </span>
          </h3>
          <div class="text-text-secondary text-xs mb-3.5 pb-2 border-b border-border-main/20 truncate select-none font-medium" :title="selectedDocument?.name">
            {{ selectedDocument?.name }}
          </div>
          
          <div v-if="knowledgeStore.isChunksLoading && knowledgeStore.chunks.length === 0" class="flex-1 flex justify-center items-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-brand"></div>
          </div>
          
          <div v-else-if="knowledgeStore.chunks.length === 0" class="flex-1 flex flex-col items-center justify-center py-8">
            <div class="text-text-muted text-2xl mb-2">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </div>
            <p class="text-text-secondary">暂无文本块</p>
          </div>
          
          <div v-else ref="chunkListContainer" class="flex-1 overflow-y-auto">
            <div :style="{ height: `${chunkVirtualizer.getTotalSize()}px` }" class="relative w-full">
              <div
                v-for="virtualRow in chunkVirtualizer.getVirtualItems()"
                :key="virtualRow.index"
                :data-index="virtualRow.index"
                :ref="el => { if (el) chunkVirtualizer.measureElement(el as HTMLElement) }"
                :style="{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  transform: `translateY(${virtualRow.start}px)`,
                }"
                class="pb-3.5 pr-2"
              >
                <div class="border border-border-main/25 bg-panel-bg/30 backdrop-blur-sm rounded-2xl hover:shadow-md transition-all duration-300 hover:border-brand/30 hover:-translate-y-[1px] overflow-hidden">
                  <div class="p-4">
                    <div class="mb-3 flex justify-between items-center select-none">
                      <span class="px-2 py-0.5 bg-brand/10 text-brand text-[10px] font-bold rounded-lg border border-brand/20">
                        块 {{ knowledgeStore.chunks[virtualRow.index].chunk_index + 1 }}
                      </span>
                      <div class="flex items-center gap-2">
                        <span class="text-[9px] text-text-muted select-none">
                          ID: {{ knowledgeStore.chunks[virtualRow.index].id.split('-')[0] }}
                        </span>
                        <button
                          class="p-1 hover:bg-brand/10 hover:text-brand rounded-lg text-text-muted transition-colors flex items-center gap-1 text-[10px] font-bold cursor-pointer"
                          title="复制文本块内容"
                          @click="copyChunkContent(knowledgeStore.chunks[virtualRow.index].id, knowledgeStore.chunks[virtualRow.index].content)"
                        >
                          <svg v-if="copiedChunkId === knowledgeStore.chunks[virtualRow.index].id" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" class="text-brand">
                            <polyline points="20 6 9 17 4 12"></polyline>
                          </svg>
                          <svg v-else xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                          </svg>
                          <span>{{ copiedChunkId === knowledgeStore.chunks[virtualRow.index].id ? '已复制' : '复制' }}</span>
                        </button>
                      </div>
                    </div>
                    <div class="text-text-secondary text-xs md:text-sm whitespace-pre-wrap leading-relaxed select-text font-normal">
                      {{ knowledgeStore.chunks[virtualRow.index].content }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <!-- 详情列表底部加载指示器 -->
            <div v-if="knowledgeStore.chunkHasMore" ref="chunkLoadMoreTrigger" class="py-4 flex justify-center h-10 items-center">
              <div class="animate-spin rounded-full h-4 w-4 border-t-2 border-b-2 border-brand"></div>
            </div>
          </div>
        </div>
        
        <!-- Right Column Empty State when no document selected -->
        <div v-else-if="activeTab === 'list'" class="flex-1 flex flex-col items-center justify-center text-center p-6 select-none">
          <div class="relative w-20 h-20 mb-4 flex items-center justify-center rounded-full bg-brand/5 border border-brand/10">
            <div class="absolute inset-0 bg-brand/10 rounded-full blur-xl animate-pulse"></div>
            <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-brand/70 relative z-10" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
              <path stroke-linecap="round" stroke-linejoin="round" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          </div>
          <h4 class="font-bold text-sm text-text-primary mb-1">选择文档查看详情</h4>
          <p class="text-xs text-text-muted max-w-xs leading-relaxed">在左侧列表选择一个已索引的文档，这里将显示经过智能切片处理后的全部文本知识块。</p>
        </div>
      </div>
    </div>

    <!-- 知识库文档删除确认模态框 -->
    <DialogRoot v-model:open="showDeleteModal">
      <DialogPortal>
        <DialogOverlay class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 animate-fade-in" />
        <DialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-panel-bg border border-border-main/60 p-6 rounded-2xl max-w-sm w-full shadow-2xl flex flex-col gap-4 z-50 animate-fade-in select-none">
          <div class="flex flex-col gap-1.5">
            <DialogTitle class="font-bold text-text-primary text-base select-none">确定要删除此文档吗？</DialogTitle>
            <DialogDescription class="text-text-muted text-xs select-text">此操作不可恢复，该文档的所有向量索引和分片都将被永久删除。</DialogDescription>
          </div>
          <div class="flex justify-end gap-3 mt-2">
            <DialogClose class="px-4 py-2 border border-border-main/50 text-text-secondary hover:bg-surface-bg rounded-xl font-bold text-xs transition-colors cursor-pointer select-none">
              取消
            </DialogClose>
            <button 
              class="px-4 py-2 bg-danger-500 hover:bg-danger/90 text-white rounded-xl font-bold text-xs transition-colors shadow-md shadow-danger-500/20 active:scale-98 cursor-pointer select-none"
              @click="confirmDeleteDoc"
            >
              确定删除
            </button>
          </div>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>
  </div>
</template>

<style scoped>
:deep(mark) {
  background-color: rgba(99, 102, 241, 0.18) !important;
  color: var(--color-brand) !important;
  font-weight: 600;
  border-radius: 4px;
  padding: 1px 3px;
}
</style>
