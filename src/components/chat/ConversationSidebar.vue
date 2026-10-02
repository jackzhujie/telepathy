<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import type { Conversation } from '@/types/chat';
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from 'reka-ui';

const props = defineProps<{
  conversations: Conversation[];
  hasMore: boolean;
  isLoading: boolean;
  currentConversationId: string | undefined;
  collapsed: boolean;
  isGenerating?: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  new: [];
  delete: [id: string];
  'load-more': [];
}>();

const loadMoreTrigger = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

const showDeleteConfirm = ref(false);
const convIdToDelete = ref<string | null>(null);

function requestDelete(id: string) {
  convIdToDelete.value = id;
  showDeleteConfirm.value = true;
}

function confirmDelete() {
  if (convIdToDelete.value) {
    emit('delete', convIdToDelete.value);
    showDeleteConfirm.value = false;
    convIdToDelete.value = null;
  }
}

onMounted(() => {
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && props.hasMore && !props.isLoading) {
      emit('load-more');
    }
  }, { threshold: 0.1 });

  if (loadMoreTrigger.value) observer.observe(loadMoreTrigger.value);
});

watch(loadMoreTrigger, (el) => {
  if (el) observer?.observe(el);
});

onUnmounted(() => {
  if (observer) observer.disconnect();
});
</script>

<template>
  <div
    class="border-r border-border-main/35 flex flex-col transition-all duration-300 bg-panel-bg/35 backdrop-blur-md"
    :class="collapsed ? 'w-0 overflow-hidden border-r-0' : 'w-48'"
  >
    <div class="p-3 border-b border-border-main/30">
      <button
        class="w-full px-3 py-2.5 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 text-white rounded-lg font-bold text-xs transition-all duration-300 hover:scale-[1.02] flex items-center justify-center gap-1.5 disabled:opacity-50 disabled:grayscale disabled:cursor-not-allowed disabled:hover:scale-100 disabled:hover:shadow-none"
        :disabled="isGenerating"
        @click="emit('new')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        <span>新对话</span>
      </button>
    </div>
    <div class="overflow-y-auto flex-1 custom-scrollbar px-2 py-3 space-y-1.5">
      <div v-for="conv in conversations" :key="conv.id">
        <div
          class="flex items-center justify-between w-full px-3 py-2.5 transition-all duration-200 rounded-lg group select-none"
          :class="[
            conv.id === currentConversationId 
              ? 'bg-brand/10 backdrop-blur-sm border-l-2 border-brand text-brand shadow-sm font-semibold' 
              : 'hover:bg-surface-bg/40 text-text-secondary hover:text-text-primary',
            isGenerating ? 'opacity-50 cursor-not-allowed pointer-events-none' : 'cursor-pointer'
          ]"
          @click="isGenerating ? undefined : emit('select', conv.id)"
        >
          <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-xs font-medium" :title="conv.title || '新对话'">{{ conv.title || '新对话' }}</span>
          <button
            class="opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-danger/10 rounded disabled:pointer-events-none"
            :disabled="isGenerating"
            @click.stop="requestDelete(conv.id)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-danger hover:text-danger-600">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
      
      <!-- 加载更多触发器 -->
      <div v-if="hasMore" ref="loadMoreTrigger" class="py-4 flex justify-center items-center h-10">
        <div class="animate-spin rounded-full h-3 w-3 border-t-2 border-b-2 border-brand"></div>
      </div>

      <div v-if="conversations.length === 0" class="text-center text-text-muted py-6 px-2 text-[10px]">
        暂无对话记录
      </div>
    </div>
    
    <!-- 删除对话确认模态框 -->
    <DialogRoot v-model:open="showDeleteConfirm">
      <DialogPortal>
        <DialogOverlay class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 animate-fade-in" />
        <DialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-panel-bg border border-border-main/60 p-6 rounded-2xl max-w-sm w-full shadow-2xl flex flex-col gap-4 z-50 animate-fade-in select-none">
          <div class="flex flex-col gap-1.5">
            <DialogTitle class="font-bold text-text-primary text-base select-none">确定要删除这段历史对话吗？</DialogTitle>
            <DialogDescription class="text-text-muted text-xs select-text">此操作不可恢复，该对话的所有消息记录将被永久删除。</DialogDescription>
          </div>
          <div class="flex justify-end gap-3 mt-2">
            <DialogClose class="px-4 py-2 border border-border-main/50 text-text-secondary hover:bg-surface-bg rounded-xl font-bold text-xs transition-colors cursor-pointer select-none">
              取消
            </DialogClose>
            <button 
              class="px-4 py-2 bg-danger-500 hover:bg-danger/90 text-white rounded-xl font-bold text-xs transition-colors shadow-md shadow-danger-500/20 active:scale-98 cursor-pointer select-none"
              @click="confirmDelete"
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
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--color-border-main);
  border-radius: 2px;
}
</style>
