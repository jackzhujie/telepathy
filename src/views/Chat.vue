<script setup lang="ts">
defineOptions({ name: 'Chat' });
import { ref, onMounted, onUnmounted, onActivated, nextTick, computed, watch } from 'vue';
import { useChatStore } from '@/stores/chat';
import { useProjectsStore } from '@/stores/projects';
import { useSettingsStore } from '@/stores/settings';
import { useVirtualizer } from '@tanstack/vue-virtual';
import { useThrottleFn } from '@vueuse/core';
import {
  SelectContent, SelectIcon, SelectItem, SelectItemText,
  SelectPortal, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
} from 'reka-ui';
import ConversationSidebar from '@/components/chat/ConversationSidebar.vue';
import MessageItem from '@/components/chat/MessageItem.vue';
import ChatInput from '@/components/chat/ChatInput.vue';
import 'highlight.js/styles/github-dark.min.css';

const chatStore = useChatStore();
const projectsStore = useProjectsStore();
const settingsStore = useSettingsStore();

const sidebarCollapsed = ref(false);

const hasVisionModel = computed(() => {
  return !!settingsStore.settings.vision_model;
});

// 虚拟滚动配置
const messagesRef = computed(() => chatStore.messages);
const isLoadingMore = ref(false);
const isUserScrollingUp = ref(false);

const messageListRef = ref<HTMLElement | null>(null);

const virtualizerOptions = computed(() => ({
  count: messagesRef.value.length,
  getScrollElement: () => messageListRef.value,
  estimateSize: () => 100, // 更细的默认颗粒度
  overscan: 10,
}));

const virtualizer = useVirtualizer(virtualizerOptions);

// 加载更多消息
async function loadMoreMessages() {
  if (!isLoadingMore.value && chatStore.messagesHasMore) {
    isLoadingMore.value = true;
    const previousHeight = messageListRef.value?.scrollHeight || 0;
    
    try {
      await chatStore.loadMoreMessages();
      
      // 保持滚动位不受新消息加入带来的推挤影响
      await nextTick();
      if (messageListRef.value) {
        const newHeight = messageListRef.value.scrollHeight;
        messageListRef.value.scrollTop += (newHeight - previousHeight);
      }
    } catch (e) {
      console.error('Failed to load history messages:', e);
    } finally {
      isLoadingMore.value = false;
    }
  }
}

const isAutoScrolling = ref(false);

// 监听滚动事件，设置了200ms节流
const handleScroll = useThrottleFn((event: Event) => {
  if (isAutoScrolling.value) return;

  const target = event.target as HTMLElement;
  
  // 判断用户是否正在向上看历史消息（而不是停留在最底部），距离底部超过15px判定为非底部
  const distanceToBottom = target.scrollHeight - target.scrollTop - target.clientHeight;
  isUserScrollingUp.value = distanceToBottom > 15;

  if (target.scrollTop < 150) {
    loadMoreMessages();
  }
}, 200);

const projectOptions = computed(() => {
  return projectsStore.projects.map((p) => ({ label: p.name, value: p.id }));
});

const scrollToBottom = async (force = false) => {
  await nextTick();
  if (force) {
    isUserScrollingUp.value = false;
    isAutoScrolling.value = true;
  }
  if (messageListRef.value) {
    const el = messageListRef.value;
    const distanceToBottom = el.scrollHeight - el.scrollTop - el.clientHeight;

    // 若用户正在看历史记录且不是强制滚动，则绝不干扰和覆盖用户的视线！
    if (!force && distanceToBottom > 15) {
      return;
    }

    if (force || !isUserScrollingUp.value) {
      messageListRef.value.scrollTo({
        top: messageListRef.value.scrollHeight,
        behavior: force ? 'smooth' : 'auto'
      });
    }
  }
  if (force) {
    setTimeout(() => {
      isAutoScrolling.value = false;
    }, 800);
  }
};

// 数组长度变化时(通常是新发了一条或新建会话)，激进地平滑滚到底部
watch(() => chatStore.messages.length, () => scrollToBottom(true));

// 模型打字过程中，内容变长，使用瞬间滚动（避免平滑效果导致卡顿拖影）
watch(
  () => {
    const msgs = chatStore.messages;
    if (msgs.length > 0) return msgs[msgs.length - 1].content;
    return '';
  },
  () => scrollToBottom(false)
);

async function handleCopy(index: number) {
  const text = chatStore.copyMessage(index);
  try { await navigator.clipboard.writeText(text); } catch {}
}

async function handleCopyRaw(index: number) {
  const text = chatStore.copyRawMessage(index);
  try { await navigator.clipboard.writeText(text); } catch {}
}

function handleEdit(index: number) {
  // Clear all other editing states first
  chatStore.messages.forEach((m, i) => {
    if (i !== index) m.isEditing = false;
  });
  
  if (index === -1) return;
  
  const msg = chatStore.messages[index];
  if (msg) msg.isEditing = true;
}

async function handleSubmitEdit(index: number, content: string) {
  await chatStore.editAndResend(index, content);
}

const chatInputRef = ref<any>(null);

function focusInput() {
  nextTick(() => {
    chatInputRef.value?.focus();
  });
}

watch(() => chatStore.currentConversationId, () => {
  focusInput();
});

import { useNotificationsStore } from '@/stores/notifications';
const notificationsStore = useNotificationsStore();

watch(() => chatStore.isGenerating, (newVal, oldVal) => {
  if (oldVal && !newVal) {
    notificationsStore.playNotificationSound();
  }
});

onMounted(async () => {
  await projectsStore.fetchProjects();
  await chatStore.initListeners();
  focusInput();
});

onActivated(async () => {
  // 必须始终重新加载侧边栏会话列表
  await chatStore.fetchConversationsPaginated(true);

  // 如果当前正在生成，切回来时不要重载和清空对话列表，否则会导致 Token 渲染错位
  if (chatStore.isGenerating) {
    await scrollToBottom();
    focusInput();
    return;
  }

  if (chatStore.currentConversationId) {
    await chatStore.loadConversation(chatStore.currentConversationId);
    await scrollToBottom();
  }
  focusInput();
});

onUnmounted(async () => {
  await chatStore.cleanup();
});</script>

<template>
  <div class="flex h-full gap-0 bg-app-bg">
    <ConversationSidebar
      :conversations="chatStore.conversations"
      :has-more="chatStore.conversationsHasMore"
      :is-loading="chatStore.isConversationsLoading"
      :current-conversation-id="chatStore.currentConversationId"
      :collapsed="sidebarCollapsed"
      :is-generating="chatStore.isGenerating"
      @select="chatStore.loadConversation"
      @new="chatStore.newConversation"
      @delete="chatStore.deleteConversation"
      @load-more="chatStore.loadMoreConversations"
    />

    <div class="flex-1 flex flex-col max-w-4xl mx-auto px-4 py-3 w-full bg-app-bg/40 backdrop-blur-md">

      <!-- Header -->
      <div class="flex items-center justify-between gap-3 pb-3 border-b border-border-main/35">
        <button
          class="p-2 hover:bg-surface-bg/50 rounded-lg transition-all duration-300 hover:scale-105"
          @click="sidebarCollapsed = !sidebarCollapsed"
        >
          <svg v-if="sidebarCollapsed" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-secondary">
            <line x1="3" y1="12" x2="21" y2="12"></line>
            <line x1="3" y1="6" x2="21" y2="6"></line>
            <line x1="3" y1="18" x2="21" y2="18"></line>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-secondary">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        <h2 class="text-sm font-bold bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary flex-1 tracking-tight">
          RAG 知识问答
        </h2>
        <SelectRoot v-model="chatStore.selectedProjectId">
          <SelectTrigger class="w-44 px-3 py-1.5 border border-border-main/40 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand/30 transition-all bg-panel-bg/40 backdrop-blur-md text-text-primary flex items-center justify-between text-xs overflow-hidden hover:border-brand/40">
            <div class="truncate flex-1 text-left">
              <SelectValue placeholder="选择项目" />
            </div>
            <SelectIcon class="ml-1 flex-shrink-0">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </SelectIcon>
          </SelectTrigger>
          <SelectPortal>
            <SelectContent class="bg-panel-bg/60 backdrop-blur-md rounded-lg shadow-xl border border-border-main/40 overflow-hidden z-50 animate-fade-in" position="popper" :side-offset="5">
              <SelectViewport class="p-1">
                <SelectItem
                  v-for="option in projectOptions"
                  :key="option.value"
                  :value="option.value"
                  class="relative flex items-center px-3 py-2 rounded-md text-[11px] cursor-pointer text-text-primary hover:bg-brand/10 data-[highlighted]:bg-brand/10 transition-all"
                >
                  <SelectItemText>{{ option.label }}</SelectItemText>
                </SelectItem>
              </SelectViewport>
            </SelectContent>
          </SelectPortal>
        </SelectRoot>
      </div>

      <!-- Messages -->
      <div ref="messageListRef" class="flex-1 overflow-y-auto overflow-x-hidden py-3 relative scroll-smooth" @scroll="handleScroll">
        <!-- 加载更多指示器 -->
        <div v-if="isLoadingMore" class="py-2 flex justify-center items-center w-full z-10 opacity-70">
          <div class="flex items-center gap-2 text-text-muted text-xs">
            <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-brand" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            正在回溯历史记录...
          </div>
        </div>

        <div v-if="chatStore.messages.length === 0" class="text-center text-text-muted mt-16 leading-relaxed">
          <div class="text-lg font-bold mb-2 tracking-tight">有什么我可以帮您的吗？</div>
          <div class="text-xs opacity-80">基于您的知识库文档进行智能问答</div>
        </div>
        <div v-else class="relative" :style="{ height: virtualizer.getTotalSize() + 'px' }">
          <div
            v-for="virtualItem in virtualizer.getVirtualItems()"
            :key="messagesRef[virtualItem.index].id"
            :data-index="virtualItem.index"
            :ref="(el) => (virtualizer.measureElement(el as any))"
            :style="{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              transform: `translateY(${virtualItem.start}px)`,
            }"
            class="pb-4 pr-2"
          >
            <MessageItem
              :message="messagesRef[virtualItem.index]"
              :index="virtualItem.index"
              :is-generating="chatStore.isGenerating"
              @copy="handleCopy"
              @copy-raw="handleCopyRaw"
              @edit="handleEdit"
              @submit-edit="handleSubmitEdit"
              @regenerate="(i) => chatStore.regenerateMessage(i)"
            />
          </div>
        </div>

      </div>

      <!-- 回到最新消息按钮（悬浮在输入框正上方，水平居中） -->
      <div class="relative w-full flex justify-center h-0 z-20">
        <Transition name="fade">
          <button 
            v-if="isUserScrollingUp"
            @click="scrollToBottom(true)"
            class="absolute bottom-3.5 flex items-center justify-center p-2.5 bg-panel-bg/95 border border-border-main/70 hover:border-brand/40 shadow-xl rounded-full text-brand hover:bg-surface-bg transition-all duration-300 hover:scale-105 active:scale-95 group backdrop-blur-md cursor-pointer select-none"
          >
            <div class="relative flex items-center justify-center w-5 h-5">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" class="transition-transform duration-300 group-hover:translate-y-0.5">
                <path d="M7 13l5 5 5-5M7 6l5 5 5-5"/>
              </svg>
              <div v-if="chatStore.isGenerating" class="absolute -top-1 -right-1 w-2.5 h-2.5 bg-brand rounded-full animate-ping"></div>
            </div>
          </button>
        </Transition>
      </div>

      <!-- Input -->
      <ChatInput
        ref="chatInputRef"
        :is-generating="chatStore.isGenerating"
        :has-vision-model="hasVisionModel"
        @send="chatStore.sendMessage"
        @stop="chatStore.stopGeneration"
      />
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>
