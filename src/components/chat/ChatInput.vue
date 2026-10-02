<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted, computed } from 'vue';

const props = defineProps<{
  isGenerating: boolean;
  hasVisionModel?: boolean;
}>();

const emit = defineEmits<{
  send: [content: string, images?: string[]];
  stop: [];
}>();

const isSending = ref(false);
const input = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const fileInputRef = ref<HTMLInputElement | null>(null);
const images = ref<string[]>([]);
const previewUrls = ref<string[]>([]);
const visionMode = ref(true);

watch(visionMode, (newVal) => {
  if (!newVal && images.value.length > 0) {
    clearImages();
  }
});

const DEFAULT_IMAGE_PROMPT = '请描述这张图片的内容';

const placeholder = computed(() => {
  if (images.value.length > 0) {
    return '输入描述，或直接发送让 AI 解释图片内容...';
  }
  return '输入您的问题，支持粘贴图片 (Enter 发送，Shift+Enter 换行)...';
});

function handleSend() {
  if (!input.value.trim() && images.value.length === 0) return;
  if (props.isGenerating || isSending.value) return;

  isSending.value = true;
  const currentImages = [...images.value];
  let currentInput = input.value.trim();
  if (!currentInput && currentImages.length > 0) {
    currentInput = DEFAULT_IMAGE_PROMPT;
  }

  emit('send', currentInput, currentImages.length > 0 ? currentImages : undefined);
  input.value = '';
  clearImages();

  nextTick(() => {
    autoResize();
    setTimeout(() => {
      isSending.value = false;
    }, 200);
  });
}

function clearImages() {
  previewUrls.value.forEach(url => URL.revokeObjectURL(url));
  images.value = [];
  previewUrls.value = [];
}

function removeImage(index: number) {
  URL.revokeObjectURL(previewUrls.value[index]);
  images.value.splice(index, 1);
  previewUrls.value.splice(index, 1);
}

function handleFileSelect(event: Event) {
  const target = event.target as HTMLInputElement;
  const files = target.files;
  if (files) {
    addFiles(Array.from(files));
  }
  target.value = '';
}

function addFiles(files: File[]) {
  for (const file of files) {
    if (!file.type.startsWith('image/')) continue;
    if (images.value.length >= 9) break;

    const reader = new FileReader();
    reader.onload = (e) => {
      const result = e.target?.result as string;
      images.value.push(result);
      previewUrls.value.push(URL.createObjectURL(file));
    };
    reader.readAsDataURL(file);
  }
}

function handlePaste(event: ClipboardEvent) {
  if (!visionMode.value) return;
  const items = event.clipboardData?.items;
  if (!items) return;

  for (const item of items) {
    if (item.type.startsWith('image/')) {
      event.preventDefault();
      const file = item.getAsFile();
      if (file) {
        addFiles([file]);
      }
      break;
    }
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleSend();
  }
}

function autoResize() {
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = 'auto';
  el.style.height = Math.min(el.scrollHeight, 200) + 'px';
}

watch(input, () => { nextTick(autoResize); });

onMounted(() => {
  document.addEventListener('paste', handlePaste);
});

onUnmounted(() => {
  document.removeEventListener('paste', handlePaste);
  clearImages();
});

defineExpose({ focus: () => textareaRef.value?.focus() });
</script>

<template>
  <div class="flex flex-col gap-2 pt-3 bg-app-bg/10 backdrop-blur-md border-t border-border-main/25">
    <!-- Image previews -->
    <div v-if="previewUrls.length > 0" class="flex gap-2 px-3 overflow-x-auto pb-1">
      <div
        v-for="(url, index) in previewUrls"
        :key="index"
        class="relative flex-shrink-0 w-16 h-16 rounded-lg overflow-hidden bg-panel-bg border border-border-main/40 shadow-sm"
      >
        <img :src="url" class="w-full h-full object-cover" />
        <button
          @click="removeImage(index)"
          class="absolute -top-1 -right-1 w-5 h-5 bg-danger text-white rounded-full text-[10px] font-bold flex items-center justify-center shadow-md hover:bg-danger/80 transition-colors"
        >
          ×
        </button>
      </div>
    </div>

    <!-- Input row -->
    <div class="flex items-end gap-2 px-3">
      <div class="flex-1 bg-panel-bg/45 backdrop-blur-md border border-border-main/40 p-1.5 rounded-xl shadow-md flex items-end gap-1.5 focus-within:border-brand/50 focus-within:ring-2 focus-within:ring-brand/15 transition-all duration-300 hover:border-border-main/60">
        <!-- Vision Mode Toggle -->
        <button
          @click="visionMode = !visionMode"
          class="p-2 transition-colors rounded-lg flex items-center justify-center"
          :class="visionMode ? 'text-brand hover:bg-brand/10' : 'text-text-muted hover:bg-surface-bg'"
          :title="visionMode ? '图片识别已开启 (点击关闭并禁用上传)' : '图片识别已关闭 (点击开启)'"
        >
          <svg v-if="visionMode" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"></path>
            <circle cx="12" cy="12" r="3"></circle>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"></path>
            <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"></path>
            <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"></path>
            <line x1="2" y1="2" x2="22" y2="22"></line>
          </svg>
        </button>

        <!-- Image upload button -->
        <button
          v-if="visionMode"
          @click="fileInputRef?.click()"
          class="p-2 text-text-muted hover:text-brand transition-colors rounded-lg hover:bg-brand/10"
          title="添加图片"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
            <circle cx="8.5" cy="8.5" r="1.5"></circle>
            <polyline points="21 15 16 10 5 21"></polyline>
          </svg>
        </button>
        <input
          ref="fileInputRef"
          type="file"
          accept="image/*"
          multiple
          class="hidden"
          @change="handleFileSelect"
        />

        <textarea
          ref="textareaRef"
          v-model="input"
          @keydown="handleKeydown"
          class="flex-1 min-h-[50px] max-h-[160px] px-3 py-1.5 border-0 focus:outline-none focus:ring-0 transition-all resize-none bg-transparent text-text-primary placeholder-text-muted/60 text-xs leading-relaxed"
          :placeholder="placeholder"
          rows="2"
        ></textarea>
      </div>

      <button
        v-if="isGenerating"
        class="px-3.5 py-3 bg-danger/10 hover:bg-danger/20 text-danger border border-danger/25 rounded-xl font-bold text-xs transition-all duration-300 flex items-center justify-center gap-1.5 shadow-sm hover:scale-105 active:scale-95"
        @click="emit('stop')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="6" width="12" height="12" rx="1"></rect>
        </svg>
        <span>停止</span>
      </button>
      <button
        v-else
        :disabled="(!input.trim() && images.length === 0) || isGenerating || isSending"
        class="px-3.5 py-3 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 text-white rounded-xl font-bold text-xs transition-all duration-300 flex items-center justify-center gap-1.5 disabled:opacity-35 disabled:grayscale disabled:cursor-not-allowed disabled:hover:scale-100 hover:scale-105 active:scale-95 shadow-md"
        @click="handleSend"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="22" y1="2" x2="11" y2="13"></line>
          <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
        </svg>
        <span>发送</span>
      </button>
    </div>
  </div>
</template>
