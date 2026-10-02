<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { useNotificationsStore } from '@/stores/notifications';
import { useRouter } from 'vue-router';

const emit = defineEmits<{ close: [] }>();
const store = useNotificationsStore();
const router = useRouter();

const typeIconMap: Record<string, string> = {
  model_download_complete: '📦',
  app_update_available: '🔄',
  document_index_complete: '📄',
  system: 'ℹ️',
};


function handleNotificationClick(notification: any) {
  store.markNotificationRead(notification.id);
  
  if (notification.routePath) {
    router.push(notification.routePath);
  } else if (notification.type === 'document_index_complete' || 
             notification.content?.toLowerCase().includes('document') || 
             notification.content?.includes('文档') || 
             notification.content?.includes('索引')) {
    router.push('/documents');
  } else if (notification.type === 'model_download_complete' || 
             notification.content?.toLowerCase().includes('model') || 
             notification.content?.includes('模型')) {
    router.push('/models');
  }
  
  emit('close');
}

function handleMarkAllRead() {
  store.markAllAsRead();
}

const loadMoreTrigger = ref<HTMLElement | null>(null);
const scrollContainer = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

onMounted(async () => {
  await store.fetchNotificationsPaginated(true);
  
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && store.hasMore && !store.isLoading) {
      store.loadMore();
    }
  }, { 
    root: scrollContainer.value,
    threshold: 0.1 
  });

  if (loadMoreTrigger.value) observer.observe(loadMoreTrigger.value);
});

watch(loadMoreTrigger, (el) => {
  if (el) observer?.observe(el);
});

onUnmounted(() => {
  if (observer) observer.disconnect();
});

function formatTime(dateStr: string): string {
  if (!dateStr) return '未知时间';
  const date = new Date(dateStr);
  if (isNaN(date.getTime())) return '未知时间';
  
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  if (hours < 24) return `${hours} 小时前`;
  if (days < 7) return `${days} 天前`;
  return date.toLocaleDateString('zh-CN');
}
</script>

<template>
  <div
    class="w-80 bg-panel-bg rounded-md border border-border-main/50 shadow-2xl overflow-hidden flex flex-col"
    style="max-height: 500px;"
    v-motion
    :initial="{ opacity: 0, x: 20, scale: 0.95 }"
    :enter="{ opacity: 1, x: 0, scale: 1 }"
    :leave="{ opacity: 0, x: 20, scale: 0.95 }"
  >
    <!-- Header -->
    <div class="flex justify-between items-center px-4 py-3 border-b border-border-main/50 bg-surface-bg/30 backdrop-blur-sm sticky top-0 z-10">
      <div class="flex items-center gap-2">
        <h3 class="text-xs font-bold text-text-primary tracking-tight">通知中心</h3>
        <span v-if="store.unreadCount > 0" class="px-1.5 py-0.5 bg-brand text-[10px] text-white rounded-full font-bold">
          {{ store.unreadCount }}
        </span>
      </div>
      <div class="flex gap-2 items-center">
        <button
          v-if="store.unreadCount > 0"
          class="text-[10px] text-brand hover:text-brand-300 font-medium transition-colors"
          @click="handleMarkAllRead"
        >
          全部已读
        </button>
        <button
          class="p-1 text-text-muted hover:text-text-primary transition-colors"
          @click="emit('close')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>

    <!-- Content -->
    <div ref="scrollContainer" class="flex-1 overflow-y-auto custom-scrollbar">
      <div v-if="store.notifications.length === 0 && !store.isLoading" class="flex flex-col items-center justify-center py-16 px-4">
        <div class="text-3xl mb-3 opacity-20">🔔</div>
        <div class="text-text-muted text-xs font-medium">暂无通知</div>
      </div>

      <div class="divide-y divide-border-main/30">
        <div
          v-for="notification in store.notifications"
          :key="notification.id"
          class="group flex items-start gap-3 p-3.5 cursor-pointer transition-all duration-200 hover:bg-surface-bg/60 relative"
          :class="{ 'bg-brand/[0.03]': !notification.isRead }"
          @click="handleNotificationClick(notification)"
        >
          <div class="text-base flex-shrink-0 mt-0.5 transition-transform group-hover:scale-110 duration-200">
            {{ typeIconMap[notification.type] || 'ℹ️' }}
          </div>
          <div class="flex-1 min-w-0">
            <div class="font-bold text-[11px] text-text-primary mb-1 tracking-tight truncate">{{ notification.title }}</div>
            <div class="text-[10px] text-text-secondary line-clamp-2 leading-relaxed opacity-90">{{ notification.content }}</div>
            <div class="text-[9px] text-text-muted mt-1.5 flex items-center gap-1.5 font-medium">
              <span>{{ formatTime(notification.createdAt) }}</span>
            </div>
          </div>
          <div v-if="!notification.isRead" class="w-1.5 h-1.5 rounded-full bg-brand shadow-[0_0_8px_rgba(99,102,241,0.6)] flex-shrink-0 mt-1.5"></div>
        </div>

        <!-- Pagination Trigger -->
        <div v-if="store.hasMore" ref="loadMoreTrigger" class="py-4 flex justify-center items-center">
          <div class="animate-spin rounded-full h-3 w-3 border-t-2 border-b-2 border-brand/50"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.max-h-\[320px\] {
  scrollbar-width: thin;
  scrollbar-color: #2a2a3a transparent;
}

.max-h-\[320px\]::-webkit-scrollbar {
  width: 4px;
}

.max-h-\[320px\]::-webkit-scrollbar-track {
  background: transparent;
}

.max-h-\[320px\]::-webkit-scrollbar-thumb {
  background-color: #2a2a3a;
  border-radius: 2px;
}
</style>
