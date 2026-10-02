<script setup lang="ts">
import type { SearchSource } from '@/types/chat';

defineProps<{
  sources: SearchSource[];
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <div v-if="visible && sources.length > 0" class="border-t border-border-main/50 rounded-t-md bg-panel-bg p-2.5">
    <div class="flex justify-between items-center mb-2">
      <h3 class="text-xs font-bold text-text-primary">引用来源</h3>
      <button class="text-text-muted text-sm hover:text-text-secondary" @click="emit('close')">×</button>
    </div>
    <div class="max-h-48 overflow-y-auto">
      <div
        v-for="(src, i) in sources"
        :key="i"
        class="mb-2 p-2.5 rounded-md bg-surface-bg border border-border-main/50 shadow-sm transition-all duration-300 hover:shadow-md"
      >
        <div class="flex justify-between items-center mb-1.5">
          <span class="px-2 py-0.5 bg-brand/15 text-brand text-[10px] rounded-full font-bold">{{ src.document_name }}</span>
          <span class="text-[10px] text-text-secondary font-semibold">相似度 {{ (src.score * 100).toFixed(1) }}%</span>
        </div>
        <div class="text-[10px] text-text-secondary leading-relaxed">{{ src.content }}</div>
      </div>
    </div>
  </div>
</template>
