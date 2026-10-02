<script setup lang="ts">
import { ref } from 'vue';

defineProps<{
  role: 'user' | 'assistant';
  isGenerating: boolean;
}>();

const emit = defineEmits<{
  copy: [];
  copyRaw: [];
  edit: [];
  regenerate: [];
}>();

const copied = ref(false);

async function handleCopy() {
  emit('copy');
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 2000);
}

function handleCopyRaw() {
  emit('copyRaw');
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 2000);
}
</script>

<template>
  <div class="flex items-center gap-1 bg-surface-bg border border-border-main/50 rounded-full px-1.5 py-0.5 shadow-sm opacity-0 group-hover:opacity-100 transition-opacity duration-200">
    <!-- 复制按钮 -->
    <button 
      @click="handleCopy"
      class="p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 transition-colors text-text-muted hover:text-brand"
      :title="copied ? '已复制' : '复制内容'"
    >
      <svg v-if="!copied" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
      </svg>
      <svg v-else xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-green-500">
        <polyline points="20 6 9 17 4 12"></polyline>
      </svg>
    </button>

    <!-- 复制原始文本 (仅助手消息) -->
    <button 
      v-if="role === 'assistant'"
      @click="handleCopyRaw"
      class="p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 transition-colors text-text-muted hover:text-brand"
      title="复制 Markdown 源码"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="16 18 22 12 16 6"></polyline>
        <polyline points="8 6 2 12 8 18"></polyline>
      </svg>
    </button>

    <!-- 编辑按钮 (仅用户消息) -->
    <button 
      v-if="role === 'user'"
      @click="emit('edit')"
      class="p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 transition-colors text-text-muted hover:text-brand"
      title="编辑"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
      </svg>
    </button>

    <!-- 重新生成 (仅助手消息且非生成中) -->
    <button 
      v-if="role === 'assistant' && !isGenerating"
      @click="emit('regenerate')"
      class="p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 transition-colors text-text-muted hover:text-brand"
      title="重新生成"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="23 4 23 10 17 10"></polyline>
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
      </svg>
    </button>
  </div>
</template>
