<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';

const windowMinimize = async () => {
  try {
    await invoke('window_minimize');
  } catch (error) {
    console.error('最小化窗口失败:', error);
  }
};

const windowMaximize = async () => {
  try {
    await invoke('window_maximize');
  } catch (error) {
    console.error('最大化窗口失败:', error);
  }
};

const windowClose = async () => {
  try {
    await invoke('window_close');
  } catch (error) {
    console.error('关闭窗口失败:', error);
  }
};
</script>

<template>
  <div class="flex items-center gap-1.5 group select-none">
    <button class="w-3 h-3 rounded-full bg-[#FF5F57] hover:brightness-110 transition-all flex items-center justify-center relative shadow-[inset_0_0_1px_rgba(0,0,0,0.2)]" @click="windowClose">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-2 h-2 opacity-0 group-hover:opacity-100 text-amber-950/70 transition-opacity absolute" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
        <path d="M3 3l6 6M9 3L3 9"></path>
      </svg>
    </button>
    <button class="w-3 h-3 rounded-full bg-[#FEBC2E] hover:brightness-110 transition-all flex items-center justify-center relative shadow-[inset_0_0_1px_rgba(0,0,0,0.2)]" @click="windowMinimize">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-2 h-2 opacity-0 group-hover:opacity-100 text-amber-950/70 transition-opacity absolute" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
        <path d="M2.5 6h7"></path>
      </svg>
    </button>
    <button class="w-3 h-3 rounded-full bg-[#28C840] hover:brightness-110 transition-all flex items-center justify-center relative shadow-[inset_0_0_1px_rgba(0,0,0,0.2)]" @click="windowMaximize">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-2 h-2 opacity-0 group-hover:opacity-100 text-green-950/70 transition-opacity absolute" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
        <path d="M6 3v6M3 6h6"></path>
      </svg>
    </button>
  </div>
</template>
