<script setup lang="ts">
import { onMounted, computed, ref } from 'vue';
import { useSettingsStore } from '@/stores/settings';

const store = useSettingsStore();
const isLoadingHub = ref(false);

onMounted(async () => {
  if (!store.modelHubData) {
    isLoadingHub.value = true;
    try {
      await store.fetchModelHub();
    } finally {
      isLoadingHub.value = false;
    }
  }
});

const hardware = computed(() => store.modelHubData?.hardware);
const recs = computed(() => store.modelHubData?.primary_recommendations);

const ramGB = computed(() => {
  if (!hardware.value) return 0;
  return Math.round(hardware.value.ram_total / 1_073_741_824);
});

const formatSize = (bytes: number) => {
  if (!bytes) return '未知';
  return (bytes / 1_073_741_824).toFixed(2) + ' GB';
};

const handleInstall = async () => {
  await store.downloadPrimaryPresets();
};

const handleSkip = () => {
  store.skipSetupWizard();
};
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/65 backdrop-blur-md p-4 transition-all duration-300">
    <div class="w-full max-w-xl bg-slate-900/90 border border-slate-800 rounded-2xl shadow-2xl p-6 md:p-8 flex flex-col text-slate-100 max-h-[90vh] overflow-y-auto">
      <!-- Header -->
      <div class="text-center mb-6">
        <div class="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-indigo-500/10 text-indigo-400 mb-3 border border-indigo-500/20">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 animate-pulse" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
          </svg>
        </div>
        <h2 class="text-2xl font-bold tracking-tight bg-gradient-to-r from-indigo-200 to-indigo-400 bg-clip-text text-transparent">
          初始化您的本地 AI 空间 🪄
        </h2>
        <p class="text-sm text-slate-400 mt-2">
          Telepathy 运行在您本机的私有环境中，无须联网，数据百分百安全。
        </p>
      </div>

      <!-- Loading State -->
      <div v-if="isLoadingHub || !recs" class="flex flex-col items-center justify-center py-12">
        <svg class="animate-spin h-8 w-8 text-indigo-500 mb-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <span class="text-sm text-slate-400">正在探测您的系统硬件并匹配模型...</span>
      </div>

      <!-- Content -->
      <div v-else class="space-y-6">
        <!-- Hardware status tag -->
        <div v-if="hardware" class="bg-indigo-950/20 border border-indigo-900/30 rounded-xl p-3 flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-indigo-500/10 text-indigo-400 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
            </svg>
          </div>
          <div class="text-xs text-slate-300">
            检测到系统配置：
            <span class="text-indigo-300 font-semibold">{{ hardware.os }} ({{ hardware.is_apple_silicon ? 'Apple Silicon' : 'Intel/Other' }})</span>，
            内存大小约为 <span class="text-indigo-300 font-semibold">{{ ramGB }} GB</span>。
          </div>
        </div>

        <p class="text-xs text-slate-400 text-center font-medium">根据您的系统配置，我们为您自适应匹配了以下黄金首选模型组合：</p>

        <!-- Recommended Model Cards -->
        <div class="space-y-3">
          <!-- Chat Model Card -->
          <div v-if="recs.chat" class="bg-slate-800/40 border border-slate-700/30 rounded-xl p-4 flex gap-4 hover:border-slate-700/60 transition-all">
            <div class="w-10 h-10 rounded-lg bg-indigo-500/10 text-indigo-400 flex items-center justify-center flex-shrink-0 border border-indigo-500/20">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h4 class="font-semibold text-sm">{{ recs.chat.model.name }}</h4>
                <span class="text-[10px] px-1.5 py-0.5 rounded bg-indigo-500/10 text-indigo-400 font-medium">对话推理</span>
              </div>
              <p class="text-xs text-slate-400 mt-1 leading-relaxed">{{ recs.chat.model.description }}</p>
              <div class="flex items-center gap-4 mt-2 text-[10px] text-slate-400 font-medium">
                <span>参数大小: {{ recs.chat.variant.params }}</span>
                <span>磁盘占用: ~{{ formatSize(recs.chat.variant.size) }}</span>
              </div>
            </div>
          </div>

          <!-- Embedding Model Card -->
          <div v-if="recs.embedding" class="bg-slate-800/40 border border-slate-700/30 rounded-xl p-4 flex gap-4 hover:border-slate-700/60 transition-all">
            <div class="w-10 h-10 rounded-lg bg-indigo-500/10 text-indigo-400 flex items-center justify-center flex-shrink-0 border border-indigo-500/20">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h4 class="font-semibold text-sm">{{ recs.embedding.model.name }}</h4>
                <span class="text-[10px] px-1.5 py-0.5 rounded bg-indigo-500/10 text-indigo-400 font-medium">向量检索</span>
              </div>
              <p class="text-xs text-slate-400 mt-1 leading-relaxed">{{ recs.embedding.model.description }}</p>
              <div class="flex items-center gap-4 mt-2 text-[10px] text-slate-400 font-medium">
                <span>参数大小: {{ recs.embedding.variant.params }}</span>
                <span>磁盘占用: ~{{ formatSize(recs.embedding.variant.size) }}</span>
              </div>
            </div>
          </div>
        </div>

        <p class="text-[10px] text-slate-500 text-center leading-relaxed max-w-sm mx-auto">
          点击“一键配置并安装”后系统将在后台静默下载，您可以立刻开始熟悉和探索应用界面，并在下载完成后使用完整功能。
        </p>

        <!-- Actions -->
        <div class="flex flex-col md:flex-row gap-3 pt-2">
          <button 
            @click="handleSkip"
            class="flex-1 py-2.5 px-4 rounded-xl text-xs font-semibold text-slate-400 border border-slate-800 hover:border-slate-700 hover:text-slate-200 transition-all"
          >
            跳过引导，手动配置
          </button>
          <button 
            @click="handleInstall"
            class="flex-1 py-2.5 px-4 rounded-xl text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-500 active:bg-indigo-700 shadow-md shadow-indigo-600/15 transition-all"
          >
            一键配置并安装
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
