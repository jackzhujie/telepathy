<script setup lang="ts">
import { ref, nextTick, watch } from 'vue';
import { renderMarkdown } from '@/utils/markdown';
import MessageActions from './MessageActions.vue';
import ThinkingIndicator from './ThinkingIndicator.vue';
import type { Message } from '@/types/chat';

const props = defineProps<{
  message: Message;
  index: number;
  isGenerating: boolean;
}>();

const emit = defineEmits<{
  copy: [index: number];
  copyRaw: [index: number];
  edit: [index: number];
  'submit-edit': [index: number, content: string];
  regenerate: [index: number];
}>();

const showLocalSources = ref(false);
const editContent = ref('');
const isEditing = ref(false);
const editRef = ref<HTMLTextAreaElement | null>(null);

watch(() => props.message.isEditing, (val) => {
  if (val) {
    editContent.value = props.message.content;
    isEditing.value = true;
    nextTick(() => { editRef.value?.focus(); });
  } else {
    isEditing.value = false;
  }
});

function startEdit() {
  emit('edit', props.index);
}

async function submitEdit() {
  if (!editContent.value.trim()) return;
  isEditing.value = false;
  emit('submit-edit', props.index, editContent.value);
}

function cancelEdit() {
  isEditing.value = false;
  // Notify parent to reset store state if needed
  emit('edit', -1); 
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    submitEdit();
  } else if (e.key === 'Escape') {
    cancelEdit();
  }
}
</script>

<template>
  <div :class="['flex w-full items-start group relative py-3 animate-message-appear', message.role === 'user' ? 'justify-end' : 'justify-start']">
    <div :class="[
      'max-w-[82%] rounded-xl shadow-sm transition-all duration-300 relative',
      message.role === 'user'
        ? 'bg-gradient-to-br from-brand/90 to-brand/70 backdrop-blur-md text-white px-4 py-3 rounded-tr-none border border-white/10 shadow-md hover:shadow-brand/20 hover:-translate-y-0.5'
        : 'bg-panel-bg/45 backdrop-blur-md border border-border-main/40 px-4 py-3 rounded-tl-none shadow-sm hover:shadow-md hover:-translate-y-0.5',
    ]">
      <!-- 发件人圆角微型标签 -->
      <span class="absolute -top-4 font-semibold opacity-70 tracking-tight text-[10px] whitespace-nowrap" :class="message.role === 'user' ? 'right-1 text-brand-300' : 'left-1 text-text-muted'">
        {{ message.role === 'user' ? '您' : 'AI 助手' }}
      </span>

      <!-- Message Actions -->
      <div
        v-if="message.status === 'done' && !isEditing"
        class="absolute -top-3 z-10 opacity-0 group-hover:opacity-100 transition-opacity duration-200"
        :class="message.role === 'user' ? 'left-0' : 'right-0'"
      >
        <MessageActions
          :role="message.role"
          :is-generating="isGenerating"
          @copy="emit('copy', index)"
          @copy-raw="emit('copyRaw', index)"
          @edit="startEdit"
          @regenerate="emit('regenerate', index)"
        />
      </div>

      <!-- User message -->
      <template v-if="message.role === 'user'">
        <!-- Images -->
        <div v-if="message.images && message.images.length > 0" class="mb-2 flex flex-wrap gap-2">
          <img
            v-for="(img, i) in message.images"
            :key="i"
            :src="img"
            class="max-w-[200px] max-h-[150px] rounded-lg object-cover border border-white/20 shadow-sm"
          />
        </div>
        <div v-if="!isEditing" class="text-xs font-medium whitespace-pre-wrap leading-relaxed select-text">{{ message.content }}</div>
        <div v-else class="flex flex-col gap-2 min-w-[200px]">
          <textarea
            ref="editRef"
            v-model="editContent"
            @keydown="handleKeydown"
            class="w-full min-h-[80px] bg-white/10 text-white border border-white/20 rounded-lg px-3 py-2 text-xs resize-none focus:outline-none focus:ring-2 focus:ring-white/30 placeholder:text-white/40 leading-relaxed"
            placeholder="重新编辑消息..."
          ></textarea>
          <div class="flex justify-end gap-2">
            <button @click="cancelEdit" class="px-2 py-1 text-[10px] bg-white/10 hover:bg-white/20 rounded transition-colors">取消</button>
            <button @click="submitEdit" class="px-2 py-1 text-[10px] bg-white text-brand font-bold rounded shadow-sm hover:bg-white/90 transition-all">发送</button>
          </div>
        </div>
      </template>

      <!-- Assistant message -->
      <template v-else>
        <ThinkingIndicator v-if="message.status === 'sending' && !message.content" />
        <div class="inline">
          <div v-if="message.content" class="markdown-body text-xs text-text-primary leading-relaxed select-text inline" v-html="renderMarkdown(message.content)"></div>
          <span v-if="message.status === 'streaming'" class="inline-block w-1.5 h-3.5 bg-brand animate-pulse ml-0.5 align-middle"></span>
        </div>
        <div v-if="message.status === 'error'" class="mt-2 flex items-center gap-2 text-danger text-xs">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="15" y1="9" x2="9" y2="15"></line>
            <line x1="9" y1="9" x2="15" y2="15"></line>
          </svg>
          {{ message.error || '生成失败' }}
        </div>
        <div v-if="message.sources && message.sources.length > 0" class="mt-3 pt-2 border-t border-border-main/20">
          <button class="text-[10px] text-brand font-bold hover:underline flex items-center gap-1" @click="showLocalSources = !showLocalSources">
            <svg v-if="!showLocalSources" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"></polyline></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg>
            {{ showLocalSources ? '收起来源 (' + message.sources.length + ')' : '查看来源 (' + message.sources.length + ')' }}
          </button>
          
          <div v-if="showLocalSources" class="mt-2 text-xs bg-surface-bg/25 backdrop-blur-sm rounded-lg border border-border-main/30 p-2 max-h-48 overflow-y-auto space-y-2">
            <div
              v-for="(src, i) in message.sources"
              :key="i"
              class="p-2 rounded-md bg-panel-bg/35 border border-border-main/25 shadow-sm transition-all duration-300 hover:shadow-md hover:border-brand/30 last:mb-0"
            >
              <div class="flex justify-between items-center mb-1 gap-2">
                <span class="px-1.5 py-0.5 bg-brand/10 text-brand text-[10px] rounded focus:outline-none focus:ring-1 focus:ring-brand/30 truncate flex-1 min-w-0" :title="src.document_name">{{ src.document_name }}</span>
                <span class="text-[10px] text-text-muted font-medium shrink-0">相似度 {{ (src.score * 100).toFixed(1) }}%</span>
              </div>
              <div class="text-[10px] text-text-secondary leading-relaxed line-clamp-3 select-text" :title="src.content">{{ src.content }}</div>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
:deep(.markdown-body) {
  display: inline;
}
:deep(.markdown-body > p) {
  display: inline;
}
</style>
