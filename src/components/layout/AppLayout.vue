<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useNotificationsStore } from '@/stores/notifications';
import { useSettingsStore } from '@/stores/settings';
import { useReindexStore } from '@/stores/reindex';
import { useBackgroundTaskStore } from '@/stores/backgroundTask';
import { useAppStore } from '@/stores/app';
import Sidebar from './Sidebar.vue';
import NotificationPanel from './NotificationPanel.vue';
import WindowControls from './WindowControls.vue';
import { 
  TooltipProvider, 
  TooltipRoot, 
  TooltipTrigger, 
  TooltipPortal, 
  TooltipContent, 
  TooltipArrow 
} from 'reka-ui';

const appWindow = getCurrentWindow();
const router = useRouter();
const notificationsStore = useNotificationsStore();
const settingsStore = useSettingsStore();
const reindexStore = useReindexStore();
const backgroundTaskStore = useBackgroundTaskStore();
const appStore = useAppStore();
const showNotifications = ref(false);
const isBackgroundTaskCollapsed = ref(false);
const bgTaskStyle = ref<{ left?: string; top?: string; transform?: string }>({});
const hasDraggedBgTask = ref(false);
const isDraggingBgTask = ref(false);
const bgTaskStartOffset = ref({ x: 0, y: 0 });
const bgTaskStartClient = ref({ x: 0, y: 0 });

const isReindexCollapsed = ref(false);
const reindexStyle = ref<{ left?: string; top?: string; transform?: string }>({});
const hasDraggedReindex = ref(false);
const isDraggingReindex = ref(false);
const reindexStartOffset = ref({ x: 0, y: 0 });
const reindexStartClient = ref({ x: 0, y: 0 });

const isModelPullCollapsed = ref(false);
const modelPullStyle = ref<{ left?: string; top?: string; transform?: string }>({});
const hasDraggedModelPull = ref(false);
const isDraggingModelPull = ref(false);
const modelPullStartOffset = ref({ x: 0, y: 0 });
const modelPullStartClient = ref({ x: 0, y: 0 });
const wasJustDraggedModelPull = ref(false);

const DRAG_THRESHOLD = 3;
const wasJustDraggedReindex = ref(false);
const wasJustDraggedBgTask = ref(false);

function onBgTaskPointerDown(e: PointerEvent) {
  isDraggingBgTask.value = false;

  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const centerX = rect.left + rect.width / 2;
  const centerY = rect.top + rect.height / 2;
  bgTaskStartOffset.value = {
    x: centerX - e.clientX,
    y: centerY - e.clientY,
  };
  bgTaskStartClient.value = { x: e.clientX, y: e.clientY };

  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onBgTaskPointerMove(e: PointerEvent) {
  if (!bgTaskStartClient.value.x && !bgTaskStartClient.value.y) return;

  const dx = e.clientX - bgTaskStartClient.value.x;
  const dy = e.clientY - bgTaskStartClient.value.y;

  if (!isDraggingBgTask.value) {
    if (Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return;
    isDraggingBgTask.value = true;
    hasDraggedBgTask.value = true;

    // 首次超过阈值时设置内联样式
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;
    bgTaskStyle.value = {
      left: `${centerX}px`,
      top: `${centerY}px`,
      transform: 'translate(-50%, -50%)',
    };
  }

  const x = bgTaskStartOffset.value.x + e.clientX;
  const y = bgTaskStartOffset.value.y + e.clientY;

  bgTaskStyle.value = {
    left: `${x}px`,
    top: `${y}px`,
    transform: 'translate(-50%, -50%)',
  };
}

function onBgTaskPointerUp() {
  if (isDraggingBgTask.value) {
    wasJustDraggedBgTask.value = true;
    setTimeout(() => { wasJustDraggedBgTask.value = false; }, 0);
  }
  bgTaskStartClient.value = { x: 0, y: 0 };
  isDraggingBgTask.value = false;
}

function onReindexPointerDown(e: PointerEvent) {
  isDraggingReindex.value = false;

  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const centerX = rect.left + rect.width / 2;
  const centerY = rect.top + rect.height / 2;
  reindexStartOffset.value = {
    x: centerX - e.clientX,
    y: centerY - e.clientY,
  };
  reindexStartClient.value = { x: e.clientX, y: e.clientY };

  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onReindexPointerMove(e: PointerEvent) {
  if (!reindexStartClient.value.x && !reindexStartClient.value.y) return;

  const dx = e.clientX - reindexStartClient.value.x;
  const dy = e.clientY - reindexStartClient.value.y;

  if (!isDraggingReindex.value) {
    if (Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return;
    isDraggingReindex.value = true;
    hasDraggedReindex.value = true;

    // 首次超过阈值时设置内联样式
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;
    reindexStyle.value = {
      left: `${centerX}px`,
      top: `${centerY}px`,
      transform: 'translate(-50%, -50%)',
    };
  }

  const x = reindexStartOffset.value.x + e.clientX;
  const y = reindexStartOffset.value.y + e.clientY;

  reindexStyle.value = {
    left: `${x}px`,
    top: `${y}px`,
    transform: 'translate(-50%, -50%)',
  };
}

function onReindexPointerUp() {
  if (isDraggingReindex.value) {
    wasJustDraggedReindex.value = true;
    setTimeout(() => { wasJustDraggedReindex.value = false; }, 0);
  }
  reindexStartClient.value = { x: 0, y: 0 };
  isDraggingReindex.value = false;
}

function onModelPullPointerDown(e: PointerEvent) {
  isDraggingModelPull.value = false;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const centerX = rect.left + rect.width / 2;
  const centerY = rect.top + rect.height / 2;
  modelPullStartOffset.value = {
    x: centerX - e.clientX,
    y: centerY - e.clientY,
  };
  modelPullStartClient.value = { x: e.clientX, y: e.clientY };
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onModelPullPointerMove(e: PointerEvent) {
  if (!modelPullStartClient.value.x && !modelPullStartClient.value.y) return;
  const dx = e.clientX - modelPullStartClient.value.x;
  const dy = e.clientY - modelPullStartClient.value.y;
  if (!isDraggingModelPull.value) {
    if (Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return;
    isDraggingModelPull.value = true;
    hasDraggedModelPull.value = true;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;
    modelPullStyle.value = {
      left: `${centerX}px`,
      top: `${centerY}px`,
      transform: 'translate(-50%, -50%)',
    };
  }
  const x = modelPullStartOffset.value.x + e.clientX;
  const y = modelPullStartOffset.value.y + e.clientY;
  modelPullStyle.value = {
    left: `${x}px`,
    top: `${y}px`,
    transform: 'translate(-50%, -50%)',
  };
}

function onModelPullPointerUp() {
  if (isDraggingModelPull.value) {
    wasJustDraggedModelPull.value = true;
    setTimeout(() => { wasJustDraggedModelPull.value = false; }, 0);
  }
  modelPullStartClient.value = { x: 0, y: 0 };
  isDraggingModelPull.value = false;
}


const unreadNotifications = computed(() => {
  return notificationsStore.unreadCount;
});

const toggleNotifications = () => {
  showNotifications.value = !showNotifications.value;
};

const onHeaderMouseDown = async (e: MouseEvent) => {
  // 只响应鼠标左键，且目标不是交互元素
  if (e.button !== 0) return;
  const target = e.target as HTMLElement;
  if (target.closest('button') || target.closest('input') || target.closest('a')) return;
  await appWindow.startDragging();
};

const onHeaderDoubleClick = async (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (target.closest('button') || target.closest('input') || target.closest('a')) return;
  try {
    await appWindow.toggleMaximize();
  } catch (error) {
    console.error('切换最大化状态失败:', error);
  }
};

onMounted(async () => {
  await reindexStore.initListener();
});

onUnmounted(() => {
  reindexStore.cleanup();
});
</script>

<template>
  <div class="flex h-screen w-screen bg-app-bg">
    <!-- 侧边栏 -->
    <Sidebar />
    
    <!-- 主内容区 -->
    <main class="flex-1 flex flex-col overflow-hidden">
      <!-- 顶部导航栏 -->
      <header
        class="h-11 border-b border-border-main/50 bg-panel-bg flex items-center justify-between px-4 select-none"
        @mousedown="onHeaderMouseDown"
        @dblclick="onHeaderDoubleClick"
      >
        <div class="flex items-center gap-4">
          <WindowControls v-if="appStore.isSidebarCollapsed" />
          <h1 v-if="appStore.isSidebarCollapsed" class="text-sm font-bold text-text-primary tracking-tight font-inter">Telepathy</h1>
        </div>
        <div class="flex items-center gap-4 flex-shrink-0">
          <button 
            class="p-1 relative hover:bg-surface-bg rounded-md transition-colors"
            @click="toggleNotifications"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-text-secondary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
            </svg>
            <span 
              v-if="unreadNotifications > 0" 
              class="absolute top-0.5 right-0.5 w-2 h-2 bg-brand rounded-full"
            ></span>
          </button>
          <!-- 更新状态指示器 -->
          <div v-if="settingsStore.appUpdateInfo" class="flex items-center">
            <TooltipProvider :delay-duration="100">
              <TooltipRoot>
                <TooltipTrigger as-child>
                  <button 
                    v-if="settingsStore.isAppUpdateReady"
                    @click="settingsStore.relaunchApp()"
                    class="p-1.5 bg-brand/10 hover:bg-brand/20 text-brand rounded-md transition-all flex items-center gap-1.5 group border border-brand/20"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 animate-pulse" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    <span class="text-[10px] font-bold">重启更新</span>
                  </button>

                  <div v-else-if="settingsStore.isAppUpdateDownloading" class="relative w-8 h-8 flex items-center justify-center group cursor-help">
                    <svg class="w-6 h-6 transform -rotate-90">
                      <circle
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="2.5"
                        fill="transparent"
                        class="text-border-main/20"
                      />
                      <circle
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="2.5"
                        fill="transparent"
                        :stroke-dasharray="2 * Math.PI * 10"
                        :stroke-dashoffset="2 * Math.PI * 10 * (1 - settingsStore.appUpdateProgress / 100)"
                        class="text-brand transition-all duration-300"
                      />
                    </svg>
                    <div class="absolute inset-0 flex items-center justify-center">
                      <span class="text-[8px] font-mono font-bold text-brand">{{ settingsStore.appUpdateProgress }}%</span>
                    </div>
                  </div>

                  <div v-else class="w-2 h-2 bg-brand rounded-full animate-pulse cursor-help"></div>
                </TooltipTrigger>
                <TooltipPortal>
                  <TooltipContent
                    class="z-[150] px-3 py-2 text-[11px] bg-panel-bg/95 backdrop-blur-xl border border-border-main rounded-lg shadow-xl text-text-primary animate-in fade-in zoom-in duration-200"
                    side="bottom"
                    :side-offset="5"
                  >
                    <div class="space-y-1">
                      <div class="font-bold flex items-center gap-2">
                        <span class="w-1.5 h-1.5 bg-brand rounded-full"></span>
                        发现新版本 v{{ settingsStore.appUpdateInfo.version }}
                      </div>
                      <div v-if="settingsStore.isAppUpdateDownloading" class="text-text-muted">
                        正在下载... {{ settingsStore.appUpdateProgress }}%
                      </div>
                      <div v-else-if="settingsStore.isAppUpdateReady" class="text-success-500 font-medium">
                        更新已准备就绪，将在下次启动时自动应用。您也可以点击左侧按钮立即重启。
                      </div>
                      <div v-else class="text-text-muted">
                        等待开始下载...
                      </div>
                    </div>
                    <TooltipArrow class="fill-border-main" />
                  </TooltipContent>
                </TooltipPortal>
              </TooltipRoot>
            </TooltipProvider>
          </div>

          <button 
            class="p-1 hover:bg-surface-bg rounded-md transition-colors relative"
            @click="() => router.push({ name: 'Settings' })"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-text-secondary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <!-- App 更新红点 - 只有在没有显示指示器时才显示 -->
            <span 
              v-if="settingsStore.appUpdateInfo && !settingsStore.isAppUpdateDownloading && !settingsStore.isAppUpdateReady" 
              class="absolute top-0 right-0 w-2 h-2 bg-red-500 rounded-full border-2 border-panel-bg"
            ></span>
          </button>
        </div>
      </header>

      <!-- 下载进度 Toast 移除 -->
      <!-- App 升级提醒 Toast 移除 -->
      
      <!-- 路由视图 -->
      <div class="flex-1 overflow-auto">
        <div class="p-4 h-full box-border transition-all duration-300">
          <router-view v-slot="{ Component }">
            <KeepAlive :include="['Chat', 'Documents', 'Settings', 'Models']">
              <component :is="Component" />
            </KeepAlive>
          </router-view>
        </div>
      </div>
    </main>
    
    <!-- 通知面板 -->
    <NotificationPanel
      v-if="showNotifications"
      @close="showNotifications = false"
    />

    <!-- 重新索引进度提示 -->
    <div
      v-if="reindexStore.isReindexing"
      :style="reindexStyle"
      class="fixed z-50 cursor-move select-none"
      :class="hasDraggedReindex ? '' : 'bottom-6 right-6'"
      @pointerdown="onReindexPointerDown"
      @pointermove="onReindexPointerMove"
      @pointerup="onReindexPointerUp"
    >
      <div
        v-if="!isReindexCollapsed"
        class="px-4 py-2.5 bg-panel-bg/95 backdrop-blur-xl border border-border-main text-text-primary rounded-xl shadow-2xl flex items-center gap-2 min-w-[240px]"
      >
        <div class="animate-spin rounded-full h-4 w-4 border-2 border-orange-400 border-t-transparent flex-shrink-0"></div>
        <div class="flex-1">
          <div class="font-medium text-orange-400">正在重新索引文档...</div>
          <div class="text-xs text-text-muted mb-2">
            {{ reindexStore.progress ? `${reindexStore.progress.current}/${reindexStore.progress.total}` : '准备中...' }}
          </div>
          <div v-if="reindexStore.progress" class="text-[11px] text-text-muted truncate">
            {{ reindexStore.progress.document_name }}
          </div>
          <div class="mt-2 h-1 bg-surface-bg rounded-full overflow-hidden">
            <div
              class="h-full bg-orange-400 rounded-full transition-all duration-300"
              :style="{ width: reindexStore.progress ? `${(reindexStore.progress.current / reindexStore.progress.total) * 100}%` : '0%' }"
            ></div>
          </div>
        </div>
        <button
          @click="!wasJustDraggedReindex && (isReindexCollapsed = true)"
          class="text-text-muted hover:text-text-primary p-1"
          title="收起"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 15 12 9 18 15"></polyline>
          </svg>
        </button>
      </div>

      <button
        v-else
        @click="!wasJustDraggedReindex && (isReindexCollapsed = false)"
        class="inline-flex items-center gap-2 px-3 py-1.5 bg-panel-bg/95 backdrop-blur-xl border border-border-main text-text-primary rounded-full shadow-2xl hover:bg-surface-bg transition-colors cursor-pointer whitespace-nowrap"
      >
        <div class="animate-spin rounded-full h-3 w-3 border-2 border-orange-400 border-t-transparent flex-shrink-0"></div>
        <span class="text-[11px] text-text-muted">{{ reindexStore.progress ? `${reindexStore.progress.current}/${reindexStore.progress.total}` : '准备中' }}</span>
        <span class="text-[11px] text-orange-400">重新索引</span>
      </button>
    </div>

    <!-- 后台批量索引任务 -->
    <div
      v-if="backgroundTaskStore.isRunning"
      :style="bgTaskStyle"
      class="fixed z-50 cursor-move select-none"
      :class="hasDraggedBgTask ? '' : 'bottom-6 left-1/2 -translate-x-1/2'"
      @pointerdown="onBgTaskPointerDown"
      @pointermove="onBgTaskPointerMove"
      @pointerup="onBgTaskPointerUp"
    >
      <div
        v-if="!isBackgroundTaskCollapsed"
        class="px-4 py-2.5 bg-panel-bg/95 backdrop-blur-xl border border-border-main text-text-primary rounded-xl shadow-2xl flex items-center gap-2 min-w-[260px]"
      >
        <div class="animate-spin rounded-full h-4 w-4 border-2 border-blue-400 border-t-transparent flex-shrink-0"></div>
        <div class="flex-1">
          <div class="font-medium text-blue-400">正在批量索引...</div>
          <div class="text-xs text-text-muted mb-2">
            {{ backgroundTaskStore.current }} / {{ backgroundTaskStore.total }}
            <span class="text-green-400 ml-2">{{ backgroundTaskStore.successCount }} 成功</span>
            <span v-if="backgroundTaskStore.failedCount > 0" class="text-red-400 ml-1">{{ backgroundTaskStore.failedCount }} 失败</span>
          </div>
          <div class="mt-2 h-1 bg-surface-bg rounded-full overflow-hidden">
            <div
              class="h-full bg-blue-400 rounded-full transition-all duration-300"
              :style="{ width: `${backgroundTaskStore.progress}%` }"
            ></div>
          </div>
        </div>
        <button
          @click="!wasJustDraggedBgTask && (isBackgroundTaskCollapsed = true)"
          class="text-text-muted hover:text-text-primary p-1"
          title="收起"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 15 12 9 18 15"></polyline>
          </svg>
        </button>
      </div>

      <button
        v-else
        @click="!wasJustDraggedBgTask && (isBackgroundTaskCollapsed = false)"
        class="inline-flex items-center gap-2 px-3 py-1.5 bg-panel-bg/95 backdrop-blur-xl border border-border-main text-text-primary rounded-full shadow-2xl hover:bg-surface-bg transition-colors cursor-pointer whitespace-nowrap"
      >
        <div class="animate-spin rounded-full h-3 w-3 border-2 border-blue-400 border-t-transparent flex-shrink-0"></div>
        <span class="text-[11px] text-text-muted">{{ backgroundTaskStore.current }} / {{ backgroundTaskStore.total }}</span>
        <span class="text-[11px] text-blue-400">批量索引</span>
      </button>
    </div>

    <!-- 下载模型进度提示 -->
    <div
      v-if="Object.keys(settingsStore.activeDownloads).length > 0"
      :style="modelPullStyle"
      class="fixed z-50 cursor-move select-none"
      :class="hasDraggedModelPull ? '' : 'bottom-24 right-6'"
      @pointerdown="onModelPullPointerDown"
      @pointermove="onModelPullPointerMove"
      @pointerup="onModelPullPointerUp"
    >
      <div 
        v-if="!isModelPullCollapsed"
        class="px-3 py-2.5 bg-panel-bg/95 backdrop-blur-xl border border-brand/40 text-text-primary rounded-xl shadow-2xl flex flex-col gap-2.5 min-w-[240px] max-h-[300px] overflow-y-auto scrollbar-hide"
      >
        <div class="flex justify-between items-center border-b border-border-main/20 pb-1.5 flex-shrink-0">
          <span class="font-bold text-brand text-[11px] flex items-center gap-1.5">
            <span class="relative flex h-2 w-2">
              <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-brand opacity-75"></span>
              <span class="relative inline-flex rounded-full h-2 w-2 bg-brand"></span>
            </span>
            正在下载模型 ({{ Object.keys(settingsStore.activeDownloads).length }} 个任务)
          </span>
          <button
            @click="!wasJustDraggedModelPull && (isModelPullCollapsed = true)"
            class="text-text-muted hover:text-text-primary p-0.5"
            title="收起进度条"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 15 12 9 18 15"></polyline>
            </svg>
          </button>
        </div>

        <div class="flex flex-col gap-3 overflow-y-auto pr-1">
          <div 
            v-for="(progress, modelId) in settingsStore.activeDownloads" 
            :key="modelId"
            class="flex items-center gap-2 border-b border-border-main/5 pb-2 last:border-0 last:pb-0"
          >
            <div class="flex-1 min-w-0">
              <div class="flex justify-between items-center mb-0.5">
                <span class="font-semibold text-[10px] text-text-primary truncate max-w-[140px]" :title="modelId">
                  {{ modelId.split(':')[0] }}
                </span>
                <span class="text-[9px] font-mono font-bold text-brand">{{ progress.percentage || 0 }}%</span>
              </div>
              <div class="text-[8px] text-text-muted mb-1 truncate leading-tight">
                {{ progress.status || '准备中...' }}
              </div>
              <div class="h-1 bg-surface-bg rounded-full overflow-hidden">
                <div
                  class="h-full bg-brand rounded-full transition-all duration-300"
                  :style="{ width: `${progress.percentage || 0}%` }"
                ></div>
              </div>
            </div>
            <button
              @click="!wasJustDraggedModelPull && settingsStore.cancelInstall(modelId)"
              class="text-text-muted hover:text-danger-500 p-1 transition-colors flex-shrink-0"
              title="取消下载"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <button
        v-else
        @click="!wasJustDraggedModelPull && (isModelPullCollapsed = false)"
        class="inline-flex items-center gap-2 px-3 py-1.5 bg-brand/10 backdrop-blur border border-brand/40 text-brand rounded-full shadow-2xl hover:bg-brand/20 transition-colors pointer-events-auto"
      >
        <div class="animate-spin rounded-full h-3 w-3 border-2 border-brand border-t-transparent flex-shrink-0"></div>
        <span class="text-[10px] font-bold">
          <template v-if="Object.keys(settingsStore.activeDownloads).length === 1">
            {{ Object.values(settingsStore.activeDownloads)[0].percentage }}% 下载中
          </template>
          <template v-else>
            {{ Object.keys(settingsStore.activeDownloads).length }} 个模型下载中
          </template>
        </span>
      </button>
    </div>

    <!-- 全局消息通知 Toast -->
    <transition
      enter-active-class="transform ease-out duration-300 transition"
      enter-from-class="translate-y-2 opacity-0 sm:translate-y-0 sm:translate-x-2"
      enter-to-class="translate-y-0 opacity-100 sm:translate-x-0"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div 
        v-if="settingsStore.pullGlobalNotification" 
        class="fixed z-[100] top-12 left-1/2 transform -translate-x-1/2 pointer-events-none"
      >
        <div 
          class="max-w-sm w-full backdrop-blur border rounded-lg shadow-xl pointer-events-auto flex items-center gap-3 px-4 py-3"
          :class="settingsStore.pullGlobalNotification.type === 'error' ? 'bg-danger-500/10 border-danger-500/30' : 'bg-success-500/10 border-success-500/30'"
        >
          <svg v-if="settingsStore.pullGlobalNotification.type === 'success'" xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-success-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-danger-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-sm font-medium" :class="settingsStore.pullGlobalNotification.type === 'error' ? 'text-danger-500' : 'text-success-500'">
            {{ settingsStore.pullGlobalNotification.message }}
          </p>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
@media (max-width: 768px) {
  .p-4 { padding: 0.75rem; }
}
</style>
