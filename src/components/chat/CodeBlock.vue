<script setup lang="ts">
import { ref } from 'vue';
import {
  TooltipRoot,
  TooltipTrigger,
  TooltipContent,
  TooltipPortal,
  TooltipProvider,
} from 'reka-ui';

const props = defineProps<{
  code: string;
  language?: string;
}>();

const copied = ref(false);

async function copyCode() {
  try {
    await navigator.clipboard.writeText(props.code);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch {
    // fallback
  }
}
</script>

<template>
  <TooltipProvider :delay-duration="300">
    <div class="relative group my-2">
      <div class="flex items-center justify-between px-3 py-1 bg-surface-bg/50 rounded-t-md border border-border-main/30 border-b-0">
        <span class="text-[10px] text-text-muted font-medium">{{ language || 'code' }}</span>
        <TooltipRoot>
          <TooltipTrigger as-child>
            <button class="p-1 rounded hover:bg-surface-bg transition-colors" @click="copyCode">
              <svg v-if="!copied" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-green-400">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            </button>
          </TooltipTrigger>
          <TooltipPortal>
            <TooltipContent class="px-2 py-1 bg-surface-bg text-text-primary text-[10px] rounded shadow-lg border border-border-main/50" :side-offset="5">
              {{ copied ? '已复制' : '复制代码' }}
            </TooltipContent>
          </TooltipPortal>
        </TooltipRoot>
      </div>
      <pre class="hljs-code-block !rounded-t-none !mt-0"><code v-html="code"></code></pre>
    </div>
  </TooltipProvider>
</template>
