<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useSnapStore } from '@/stores/snap';
import MilkdownEditor from '@/components/editor/MilkdownEditor.vue';
import { MilkdownProvider } from '@milkdown/vue';

const snapStore = useSnapStore();
const newSnapContent = ref('');
const newSnapTags = ref('');
const viewMode = ref<'zen' | 'split'>('zen'); // zen: WYSIWYG, split: side-by-side
const isHistoryCollapsed = ref(false);

onMounted(() => {
  snapStore.fetchSnaps(true);
});

async function handleAddSnap() {
  if (!newSnapContent.value.trim()) return;
  try {
    await snapStore.createSnap(newSnapContent.value, newSnapTags.value);
    newSnapContent.value = '';
    newSnapTags.value = '';
  } catch (e) {
    alert('保存失败');
  }
}

const sortedSnaps = computed(() => {
  return [...snapStore.snaps].sort((a, b) => {
    if (a.isPinned && !b.isPinned) return -1;
    if (!a.isPinned && b.isPinned) return 1;
    const timeA = a.createdAt ? new Date(a.createdAt).getTime() : 0;
    const timeB = b.createdAt ? new Date(b.createdAt).getTime() : 0;
    return timeB - timeA;
  });
});

function formatDate(dateStr: string) {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  return date.toLocaleString();
}
</script>

<template>
  <MilkdownProvider>
    <div class="h-full flex flex-col p-6 bg-app-bg/40 backdrop-blur-md px-1 select-text overflow-hidden animate-fade-in">
      <header class="mb-6 flex justify-between items-end select-none" v-motion :initial="{ opacity: 0, y: -20 }" :enter="{ opacity: 1, y: 0 }">
        <div class="flex items-center gap-2">
          <div class="p-2 bg-brand/15 backdrop-blur-md border border-brand/20 rounded-xl">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand">
              <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
            </svg>
          </div>
          <div>
            <h1 class="text-2xl font-black bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary tracking-tight">随笔工作台</h1>
            <p class="text-[11px] text-text-secondary mt-0.5">沉浸式 Markdown 创作空间</p>
          </div>
        </div>

        <!-- 模式切换 -->
        <div class="flex bg-panel-bg/45 backdrop-blur-md p-1 rounded-xl border border-border-main/35 shadow-sm">
          <button 
            @click="viewMode = 'zen'"
            :class="['px-3 py-1.5 text-xs rounded-lg transition-all font-bold select-none', viewMode === 'zen' ? 'bg-gradient-to-br from-brand to-brand-600 shadow-md text-white' : 'text-text-muted hover:text-text-primary hover:bg-surface-bg/50']"
          >
            禅模式
          </button>
          <button 
            @click="viewMode = 'split'"
            :class="['px-3 py-1.5 text-xs rounded-lg transition-all font-bold select-none', viewMode === 'split' ? 'bg-gradient-to-br from-brand to-brand-600 shadow-md text-white' : 'text-text-muted hover:text-text-primary hover:bg-surface-bg/50']"
          >
            双栏模式
          </button>
        </div>
      </header>

      <!-- 编辑区 -->
      <div 
        class="mb-3.5 flex-1 flex flex-col min-h-0 bg-panel-bg/45 backdrop-blur-md rounded-2xl border border-border-main/35 shadow-2xl overflow-hidden transition-all duration-300"
        v-motion :initial="{ opacity: 0, y: 20 }" :enter="{ opacity: 1, y: 0 }" :delay="100"
      >
        <div class="flex-1 flex min-h-0 overflow-hidden select-text">
          <!-- 左侧：源码编辑 (仅在双栏模式显示) -->
          <div v-if="viewMode === 'split'" class="w-1/2 border-r border-border-main/20 p-6 overflow-y-auto custom-scrollbar bg-surface-bg/30">
            <textarea
              v-model="newSnapContent"
              placeholder="在此输入 Markdown 源码..."
              class="w-full h-full bg-transparent border-none focus:ring-0 text-text-primary resize-none font-mono text-sm leading-relaxed placeholder:text-text-muted/40"
            ></textarea>
          </div>

          <!-- 右侧：Milkdown 编辑器 (禅模式下全屏) -->
          <div :class="[viewMode === 'split' ? 'w-1/2' : 'w-full']" class="p-8 overflow-y-auto custom-scrollbar relative">
            <MilkdownEditor 
              v-model="newSnapContent" 
              :read-only="false"
            />
          </div>
        </div>
      
      <!-- 底部控制栏 -->
      <div class="p-4 border-t border-border-main/20 bg-panel-bg/25 backdrop-blur-sm flex items-center justify-between select-none">
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-2 bg-panel-bg/45 px-3 py-1.5 rounded-xl border border-border-main/35 shadow-sm hover:border-border-main/60 transition-all duration-300">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
            <input 
              v-model="newSnapTags" 
              placeholder="标签 (逗号分隔)" 
              class="bg-transparent border-none focus:ring-0 text-xs text-text-primary w-48 p-0 placeholder-text-muted/50 select-text"
            />
          </div>
          <span class="text-[10px] text-text-muted hidden sm:inline-flex items-center gap-1">
             <kbd class="px-1.5 py-0.5 rounded bg-surface-bg/60 border border-border-main/20 font-sans">⌘</kbd>
             <kbd class="px-1.5 py-0.5 rounded bg-surface-bg/60 border border-border-main/20 font-sans">Enter</kbd>
             快速保存
          </span>
        </div>
        
        <button
          @click="handleAddSnap"
          class="px-6 py-2 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 active:scale-95 text-white rounded-xl text-sm font-bold transition-all disabled:opacity-50 shadow-md select-none"
          :disabled="!newSnapContent.trim()"
        >
          发布随记
        </button>
      </div>
    </div>

    <!-- 历史记录预览区 (紧凑型) -->
    <div 
      :class="[isHistoryCollapsed ? 'h-14 overflow-hidden' : 'h-48 overflow-y-auto']"
      class="pr-2 custom-scrollbar bg-panel-bg/25 border border-border-main/25 backdrop-blur-md rounded-2xl p-4 mt-auto shadow-sm select-none transition-all duration-300"
    >
      <div class="flex justify-between items-center mb-3">
        <div class="flex items-center gap-1.5 select-none">
          <span class="text-xs font-bold text-text-primary opacity-90">历史记录预览</span>
          <span class="text-[10px] bg-surface-bg/60 border border-border-main/20 px-1.5 py-0.5 rounded-md text-text-muted font-mono leading-none">{{ sortedSnaps.length }}</span>
        </div>
        <button 
          @click="isHistoryCollapsed = !isHistoryCollapsed"
          class="px-2 py-1 hover:bg-surface-bg/60 rounded-lg text-text-muted hover:text-text-primary transition-all duration-300 cursor-pointer flex items-center justify-center gap-1 select-none border border-transparent hover:border-border-main/30"
        >
          <span class="text-[10px]">{{ isHistoryCollapsed ? '展开' : '收起' }}</span>
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :class="isHistoryCollapsed ? 'rotate-180' : 'rotate-0'" class="transition-transform duration-300">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </button>
      </div>

      <div v-if="!isHistoryCollapsed">
        <div v-if="snapStore.isLoading && snapStore.snaps.length === 0" class="flex justify-center items-center h-28">
           <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-brand mr-2"></div>
           <span class="text-xs text-text-muted">加载记录...</span>
        </div>
        
        <div v-else-if="snapStore.snaps.length === 0" class="flex flex-col items-center justify-center h-28 text-text-muted/30">
          <p class="text-xs font-medium">暂无历史记录</p>
        </div>

        <div v-else class="flex gap-4 overflow-x-auto pb-2 custom-scrollbar-h select-none">
          <div
            v-for="snap in sortedSnaps"
            :key="snap.id"
            class="flex-shrink-0 w-64 bg-panel-bg/45 backdrop-blur-md p-4 rounded-xl border border-border-main/35 hover:border-brand/40 hover:shadow-md hover:-translate-y-0.5 transition-all duration-300 group relative shadow-sm"
          >
            <div class="flex justify-between items-center mb-2">
              <span class="text-[9px] font-mono text-text-muted">{{ formatDate(snap.createdAt) }}</span>
              <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-all">
                <button @click="snapStore.deleteSnap(snap.id)" class="text-text-muted hover:text-danger-500 transition-colors">
                  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
              </div>
            </div>
            <div class="text-xs text-text-primary line-clamp-3 opacity-80 group-hover:opacity-100 transition-opacity whitespace-pre-wrap select-text">
              {{ snap.content }}
            </div>
          </div>
        </div>
      </div>
    </div>
    </div>
  </MilkdownProvider>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(var(--border-main), 0.1);
  border-radius: 10px;
}

.custom-scrollbar-h::-webkit-scrollbar {
  height: 4px;
}
.custom-scrollbar-h::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar-h::-webkit-scrollbar-thumb {
  background: rgba(var(--border-main), 0.1);
  border-radius: 10px;
}
</style>
