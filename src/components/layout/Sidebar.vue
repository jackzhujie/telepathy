<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { useAppStore } from '@/stores/app';

import WindowControls from './WindowControls.vue';

const router = useRouter();
const route = useRoute();
const appStore = useAppStore();

const menuItems = [
  { label: '知识库', key: 'KnowledgeBase', icon: 'book' },
  { label: 'AI 对话', key: 'Chat', icon: 'chat' },
  { label: '模型管理', key: 'Models', icon: 'cpu' },
  { label: '文档管理', key: 'Documents', icon: 'document' },
  { label: '个人随记', key: 'SnapNote', icon: 'zap' },
  { label: '用户信息', key: 'Profile', icon: 'user' }
];

function handleMenuClick(key: string) {
  router.push({ name: key });
}

function toggleSidebar() {
  appStore.isSidebarCollapsed = !appStore.isSidebarCollapsed;
}
</script>

<template>
  <div
    :class="['sidebar', { 'collapsed': appStore.isSidebarCollapsed }]"
    class="bg-panel-bg border-r border-border-main/50 transition-all duration-300 flex flex-col h-full"
    :style="{ width: appStore.isSidebarCollapsed ? '56px' : '170px' }"
    v-motion
    :initial="{ opacity: 0, x: -20 }"
    :enter="{ opacity: 1, x: 0 }"
  >
    <div
      class="flex items-center h-11 border-b border-border-main/50 px-4 select-none"
      :class="appStore.isSidebarCollapsed ? 'justify-center' : 'justify-between'"
      v-motion
      :initial="{ opacity: 0 }"
      :enter="{ opacity: 1 }"
      :delay="100"
    >
      <WindowControls v-if="!appStore.isSidebarCollapsed" />
      <span class="font-bold text-sm text-text-primary font-inter truncate tracking-tight">
        {{ appStore.isSidebarCollapsed ? 'T' : 'Telepathy' }}
      </span>
    </div>

    <nav class="flex-1 overflow-y-auto min-h-0 p-2">
      <ul class="space-y-0.5">
        <li
          v-for="(item, index) in menuItems"
          :key="item.key"
          v-motion
          :initial="{ opacity: 0, x: -20 }"
          :enter="{ opacity: 1, x: 0 }"
          :delay="150 + index * 50"
        >
          <button
            class="relative flex items-center gap-2 px-2 py-1.5 w-full rounded-md transition-colors"
            :class="[
              route.name === item.key ? 'bg-brand/10 text-brand' : 'text-text-secondary hover:bg-surface-bg hover:text-text-primary',
              appStore.isSidebarCollapsed ? 'justify-center' : 'justify-start'
            ]"
            @click="handleMenuClick(item.key)"
          >
            <span v-if="route.name === item.key && !appStore.isSidebarCollapsed" class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-4 bg-brand rounded-r-full"></span>
            <span class="text-sm flex-shrink-0">
              <svg v-if="item.icon === 'book'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
              </svg>
              <svg v-else-if="item.icon === 'chat'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
              <svg v-else-if="item.icon === 'document'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
                <polyline points="10 9 9 9 8 9"/>
              </svg>
              <svg v-else-if="item.icon === 'user'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
              </svg>
              <svg v-else-if="item.icon === 'cpu'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="4" y="4" width="16" height="16" rx="2" ry="2"></rect>
                <rect x="9" y="9" width="6" height="6"></rect>
                <line x1="9" y1="1" x2="9" y2="4"></line>
                <line x1="15" y1="1" x2="15" y2="4"></line>
                <line x1="9" y1="20" x2="9" y2="23"></line>
                <line x1="15" y1="20" x2="15" y2="23"></line>
                <line x1="20" y1="9" x2="23" y2="9"></line>
                <line x1="20" y1="15" x2="23" y2="15"></line>
                <line x1="1" y1="9" x2="4" y2="9"></line>
                <line x1="1" y1="15" x2="4" y2="15"></line>
              </svg>
              <svg v-else-if="item.icon === 'zap'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
              </svg>
            </span>
            <span v-if="!appStore.isSidebarCollapsed" class="font-medium text-xs text-text-secondary">
              {{ item.label }}
            </span>
          </button>
        </li>
      </ul>
    </nav>

    <div class="p-2 border-t border-border-main/50">
      <button
        class="flex items-center justify-center gap-2 px-2 py-1.5 w-full rounded-md hover:bg-surface-bg transition-colors"
        @click="toggleSidebar"
        v-motion
        :initial="{ opacity: 0 }"
        :enter="{ opacity: 1 }"
        :delay="300"
      >
        <svg v-if="appStore.isSidebarCollapsed" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
        <span v-if="!appStore.isSidebarCollapsed" class="text-xs font-medium text-text-secondary">
          {{ appStore.isSidebarCollapsed ? '展开' : '收起' }}
        </span>
      </button>
    </div>
  </div>
</template>