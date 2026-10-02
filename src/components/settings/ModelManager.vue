<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { watchDebounced } from '@vueuse/core';
import { useSettingsStore } from '@/stores/settings';
import { getModelVariants } from '@/api/tauri';

import {
  SelectContent,
  SelectIcon,
  SelectItem,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from 'reka-ui';

const emit = defineEmits<{
  'select-chat-model': [model: string],
  'select-embedding-model': [model: string],
  'select-vision-model': [model: string],
  'select-vision-mmproj': [model: string]
}>();

const store = useSettingsStore();

// UI 状态
const activeTab = ref<'recommend' | 'all' | 'chat' | 'embedding' | 'vision'>('recommend');
const searchQuery = ref('');
const expandedSearchModel = ref<string | null>(null); // 搜索结果展开状态
const activeRankingTab = ref<'leaderboard' | 'trending'>('leaderboard'); // 排行榜当前选中的 Tab
const deletingModel = ref<string | null>(null);

// 新增：记录推荐模型的展开折叠状态
const expandedRecommendModels = ref<Record<string, boolean>>({});

const toggleRecommendExpand = (modelName: string) => {
  expandedRecommendModels.value[modelName] = !expandedRecommendModels.value[modelName];
};

// 懒加载变体缓存 & 状态
const variantsCache = ref<Record<string, any[]>>({});
const loadingVariants = ref<Record<string, boolean>>({});

/**
 * 效率评分：分数越高，变体越适合当前硬件运行。
 */
const efficiencyScore = (sizeBytes: number): number => {
  const hw = store.modelHubData?.hardware;
  if (!hw || sizeBytes === 0) return 2; // 未知
  const { vram_total, ram_total, disk_free } = hw;
  if (disk_free < sizeBytes * 2) return 0;               // 磁盘不足
  if (vram_total > 0 && sizeBytes < vram_total) return 5; // 极速/显卡
  if (sizeBytes < ram_total) return (vram_total === 0) ? 4 : 3; // 内存运行 or 内存溢出
  return 1;                                               // 硬件不足
};

/**
 * 将变体列表按可运行性降序排列（分数高的在前）。
 */
const sortVariantsByEfficiency = (variants: any[]): any[] => {
  return [...variants]
    .filter(v => !v.tag.toLowerCase().includes('mmproj') && !v.params.toLowerCase().includes('mmproj'))
    .sort((a, b) => efficiencyScore(b.size) - efficiencyScore(a.size));
};

/**
 * 展开搜索结果行，并懒加载变体（命中缓存则跳过网络请求）。
 */
const toggleSearchExpand = async (modelName: string) => {
  if (expandedSearchModel.value === modelName) {
    expandedSearchModel.value = null;
    return;
  }
  expandedSearchModel.value = modelName;

  // 已缓存则直接使用
  if (variantsCache.value[modelName] !== undefined) return;

  // 懒加载
  loadingVariants.value[modelName] = true;
  try {
    const variants = await getModelVariants(modelName);
    variantsCache.value[modelName] = variants;
    // 同步回 searchResultModels 以供安装按钮读取 size
    const model = store.searchResultModels.find((m: any) => m.name === modelName);
    if (model) model.variants = variants;
  } catch (e) {
    console.error('[Variants] Failed to fetch:', e);
    variantsCache.value[modelName] = [];
  } finally {
    loadingVariants.value[modelName] = false;
  }
};


// 初始化数据
onMounted(async () => {
  await store.fetchInstalledModels();
  await store.fetchModelHub(); // 核心：同步云端枢纽与硬件探测
  await store.fetchRegistryModels();
});

// 全量探索 tab 的搜索防抖：仅在 'all' tab 激活时响应 searchQuery 变化
watchDebounced(
  searchQuery,
  (newQuery) => {
    if (activeTab.value !== 'all') return; // 仅全量探索 tab 触发在线检索
    const trimmed = newQuery?.trim();
    if (trimmed && trimmed.length > 1) {
      store.searchHubModels(trimmed);
    } else if (!trimmed) {
      // 清空搜索时同步清空结果和上次查询记录
      store.searchResultModels = [];
      store.lastSearchQuery = '';
    }
  },
  { debounce: 600 }
);

const handleEnterSearch = () => {
  const trimmed = searchQuery.value?.trim();
  if (!trimmed || trimmed.length <= 1) return;
  // 回车立即触发，跳过防抖等待；依赖 store 内部去重逻辑避免重复
  store.searchHubModels(trimmed);
};

// 计算属性：当前选中的引擎模型
const chatModel = computed({
  get: () => store.settings.chat_model || '',
  set: (v: string) => emit('select-chat-model', v),
});

const embeddingModel = computed({
  get: () => store.settings.embedding_model || '',
  set: (v: string) => emit('select-embedding-model', v),
});

const visionModel = computed({
  get: () => store.settings.vision_model || '',
  set: (v: string) => emit('select-vision-model', v),
});

// const visionMmproj = computed({
//   get: () => store.settings.vision_mmproj || '',
//   set: (v: string) => emit('select-vision-mmproj', v),
// });

// 计算属性：供下拉框选项使用
const chatModelOptions = computed(() => store.chatModels.map(m => ({ label: m.full_name, value: m.full_name })));
const embeddingModelOptions = computed(() => store.embeddingModels.map(m => ({ label: m.full_name, value: m.full_name })));
const visionModelOptions = computed(() => store.visionModels
  .filter(m => !m.full_name.toLowerCase().includes('mmproj'))
  .map(m => ({ label: m.full_name, value: m.full_name }))
);
const visibleInstalledModels = computed(() => {
  return store.installedModels.filter(m => !m.full_name.toLowerCase().includes('mmproj') && !m.full_name.toLowerCase().includes('projector'));
});

// 分类推荐逻辑
const chatRecommendations = computed(() => {
  if (!store.modelHubData?.recommendations) return [];
  const list: any[] = [];
  const seenFamily = new Set<string>();

  for (const r of store.modelHubData.recommendations) {
    if (r.model.category !== 'chat') continue;

    // 提炼模型家族/厂商进行去重，确保推荐的多样性
    const name = r.model.name.toLowerCase();
    let family = 'other';
    if (name.includes('qwen')) family = 'qwen';
    else if (name.includes('llama')) family = 'llama';
    else if (name.includes('deepseek')) family = 'deepseek';
    else if (name.includes('mistral')) family = 'mistral';
    else if (name.includes('gemma')) family = 'gemma';

    if (family !== 'other' && seenFamily.has(family)) {
      continue;
    }
    seenFamily.add(family);
    list.push(r);
    if (list.length >= 3) break;
  }
  return list;
});

const embeddingRecommendations = computed(() => {
  if (!store.modelHubData?.recommendations) return [];
  const list: any[] = [];
  const seenFamily = new Set<string>();

  for (const r of store.modelHubData.recommendations) {
    if (r.model.category !== 'embedding') continue;

    const name = r.model.name.toLowerCase();
    let family = 'other';
    if (name.includes('bge')) family = 'bge';
    else if (name.includes('gte')) family = 'gte';
    else if (name.includes('jina')) family = 'jina';

    if (family !== 'other' && seenFamily.has(family)) {
      continue;
    }
    seenFamily.add(family);
    list.push(r);
    if (list.length >= 3) break;
  }
  return list;
});

const visionRecommendations = computed(() => {
  if (!store.modelHubData?.recommendations) return [];
  const list: any[] = [];
  const seenFamily = new Set<string>();

  for (const r of store.modelHubData.recommendations) {
    if (r.model.category !== 'vision') continue;

    const name = r.model.name.toLowerCase();
    let family = 'other';
    if (name.includes('llava')) family = 'llava';
    else if (name.includes('qwen2-vl') || name.includes('qwen2.5-vl')) family = 'qwen-vl';
    else if (name.includes('moondream')) family = 'moondream';
    else if (name.includes('llama')) family = 'llama-vision';

    if (family !== 'other' && seenFamily.has(family)) {
      continue;
    }
    seenFamily.add(family);
    list.push(r);
    if (list.length >= 3) break;
  }
  return list;
});

const getModelFilename = (modelName: string, variantTag: string) => {
  // Extract a sanitized safe name for the file system matching Rust backend's naming
  const safeModelName = modelName.split('/').pop()?.replace("-", "_").replace(".", "_").toLowerCase() || modelName;
  const safeVariantName = variantTag.replace("/", "_").replace("\\", "_").replace(":", "_");
  const prefix = `${safeModelName}_`;
  const finalFilename = (safeVariantName.toLowerCase().startsWith(prefix) || safeVariantName.toLowerCase().startsWith(safeModelName))
    ? safeVariantName
    : `${safeModelName}_${safeVariantName}`;
  return finalFilename.toLowerCase().endsWith('.gguf') ? finalFilename : `${finalFilename}.gguf`;
};

const isModelInstalled = (modelName: string, variantTag: string) => {
  const filename = getModelFilename(modelName, variantTag);
  return store.installedModels.some(m => m.full_name.toLowerCase() === filename.toLowerCase());
};

const getDownloadProgress = (modelName: string, variantTag: string) => {
  const id = `${modelName}:${variantTag}`;
  return store.activeDownloads[id];
};

const selectChatModel = async (modelName: string, variantTag: string) => {
  try {
    const filename = getModelFilename(modelName, variantTag);
    await store.saveSetting('chat_model', filename);
    emit('select-chat-model', filename);
    await store.fetchInstalledModels();
  } catch (e: any) {
    console.error('[SelectModel] Failed:', e);
    store.pullGlobalNotification = {
      message: `设置默认模型失败: ${e.message || e}`,
      type: 'error',
      timestamp: Date.now()
    };
    setTimeout(() => {
      store.pullGlobalNotification = null;
    }, 4000);
  }
};

const selectEmbeddingModel = async (modelName: string, variantTag: string) => {
  try {
    const filename = getModelFilename(modelName, variantTag);
    await store.saveSetting('embedding_model', filename);
    emit('select-embedding-model', filename);
    await store.fetchInstalledModels();
  } catch (e: any) {
    console.error('[SelectModel] Failed:', e);
    store.pullGlobalNotification = {
      message: `设置默认模型失败: ${e.message || e}`,
      type: 'error',
      timestamp: Date.now()
    };
    setTimeout(() => {
      store.pullGlobalNotification = null;
    }, 4000);
  }
};

const selectVisionModel = async (modelName: string, variantTag: string) => {
  try {
    const filename = getModelFilename(modelName, variantTag);
    await store.saveSetting('vision_model', filename);
    emit('select-vision-model', filename);
    await store.fetchInstalledModels();
  } catch (e: any) {
    console.error('[SelectModel] Failed:', e);
    store.pullGlobalNotification = {
      message: `设置默认模型失败: ${e.message || e}`,
      type: 'error',
      timestamp: Date.now()
    };
    setTimeout(() => {
      store.pullGlobalNotification = null;
    }, 4000);
  }
};

const handleConfirmDelete = async () => {
  if (!deletingModel.value) return;
  try {
    await store.removeModel(deletingModel.value);
    deletingModel.value = null;
  } catch (error) {
    console.error('Failed to delete model:', error);
  }
};

const getModelTypeColor = (type: string) => {
  switch(type) {
    case 'chat': return 'bg-brand/20 text-brand border border-brand/30';
    case 'embedding': return 'bg-success-500/20 text-success-500 border border-success-500/30';
    case 'vision': return 'bg-purple-500/20 text-purple-500 border border-purple-500/30';
    default: return 'bg-gray-500/20 text-gray-500 border border-gray-500/30';
  }
};

const getModelTypeName = (type: string) => {
  switch(type) {
    case 'chat': return '对话';
    case 'embedding': return '嵌入';
    case 'vision': return '视觉';
    default: return '未知';
  }
};

const getCapabilityBadgeStyle = (cap: string) => {
  const c = cap.toLowerCase();
  if (c.includes('vision')) return 'bg-purple-500/10 text-purple-400 border border-purple-500/30';
  if (c.includes('tool')) return 'bg-blue-500/10 text-blue-400 border border-blue-500/30';
  if (c.includes('think')) return 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/30';
  if (c.includes('audio')) return 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30';
  if (c.includes('cloud')) return 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30';
  return 'bg-surface-bg-lighter text-text-muted border border-border-main';
};

const translateCapability = (cap: string) => {
  const c = cap.toLowerCase();
  if (c.includes('vision')) return '视觉';
  if (c.includes('tool')) return '工具';
  if (c.includes('think')) return '思维';
  if (c.includes('audio')) return '音频';
  if (c.includes('cloud')) return '云端';
  return cap;
};

const isInstalled = (modelName: string) => {
  return store.installedModels.some(m => m.full_name.startsWith(modelName));
};

const formatSize = (bytes: number) => {
  if (!bytes) return '未知';
  const gb = bytes / (1024 * 1024 * 1024);
  return gb.toFixed(1) + ' GB';
};

/**
 * 根据规格文件大小和用户硬件计算运行效率标签。
 * 返回 { label, icon, color, description } 供模板渲染。
 * size = 0 表示搜索结果未获取到文件大小，返回 unknown。
 */
const getVariantEfficiency = (sizeBytes: number) => {
  const hw = store.modelHubData?.hardware;
  if (!hw || sizeBytes === 0) {
    return { label: '未知', icon: '❓', color: 'text-text-muted bg-surface-bg border-border-main/50', description: '无法评估' };
  }

  const isAppleSilicon = hw.is_apple_silicon;
  const vram = hw.vram_total;
  const ram = hw.ram_total;
  const diskFree = hw.disk_free;

  // 磁盘不足（文件大小 × 2 保留空间）
  if (diskFree < sizeBytes * 2) {
    return { label: '磁盘不足', icon: '💾', color: 'text-red-400 bg-red-500/10 border-red-500/30', description: '磁盘剩余空间不足，建议清理后重试' };
  }

  // GPU/显存完全装载 → 最优
  if (vram > 0 && sizeBytes < vram) {
    if (isAppleSilicon) {
      return { label: '极速推荐', icon: '⚡', color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30', description: 'Apple Silicon 统一内存，GPU全速运行' };
    }
    return { label: '显卡加速', icon: '🎮', color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30', description: '可完全载入显存，推理极速' };
  }

  // 内存可以容纳（CPU模式）
  if (sizeBytes < ram) {
    return { label: '内存运行', icon: '🖥️', color: 'text-blue-400 bg-blue-500/10 border-blue-500/30', description: '可载入系统内存，CPU 模式运行' };
  }

  // 超出内存
  return { label: '硬件不足', icon: '🔴', color: 'text-red-400 bg-red-500/10 border-red-500/30', description: '超出内存上限，运行压力极大' };
};

const pendingInstall = ref<{name: string, variant: string} | null>(null);
const showHardwareConfirm = ref(false);



const checkEfficiencyAndInstall = async (modelName: string, variantTag: string, sizeBytes: number) => {
  if (variantTag.toLowerCase().includes('mmproj')) {
    proceedWithHardwareCheck(modelName, variantTag, sizeBytes);
    return;
  }

  let model: any = null;
  if (store.modelHubData?.recommendations) {
    model = store.modelHubData.recommendations.find((r: any) => r.model.name === modelName)?.model;
  }
  if (!model && store.modelHubData?.all_models) {
    model = store.modelHubData.all_models.find((m: any) => m.name === modelName);
  }
  if (!model && store.searchResultModels) {
    model = store.searchResultModels.find((m: any) => m.name === modelName);
  }
  if (!model && store.registryModels) {
    model = store.registryModels.find((m: any) => m.name === modelName);
  }

  if (model?.category === 'vision') {
    const allVars = variantsCache.value[modelName] ?? model.variants;
    const mmprojVars = allVars?.filter((v: any) => v.tag.toLowerCase().includes('mmproj') || v.params?.toLowerCase().includes('mmproj')) || [];
    const mmprojVariant = mmprojVars.find((v: any) => v.tag.toLowerCase().includes('f16')) || mmprojVars[0];
    
    if (mmprojVariant) {
      // Auto-bind
      const mainFilename = getModelFilename(modelName, variantTag);
      const mmprojFilename = getModelFilename(modelName, mmprojVariant.tag);
      const currentBindings = store.settings.vision_model_bindings ? JSON.parse(store.settings.vision_model_bindings) : {};
      currentBindings[mainFilename] = mmprojFilename;
      store.saveSetting('vision_model_bindings', JSON.stringify(currentBindings));
      
      // Auto-download mmproj silently
      setTimeout(() => {
          store.installNewModel(modelName, mmprojVariant.tag);
      }, 500);
    }
  }

  proceedWithHardwareCheck(modelName, variantTag, sizeBytes);
};

const proceedWithHardwareCheck = async (modelName: string, variantTag: string, sizeBytes: number) => {
  const eff = getVariantEfficiency(sizeBytes);
  if (eff.label === '硬件不足' || eff.label === '内存溢出') {
    pendingInstall.value = { name: modelName, variant: variantTag };
    showHardwareConfirm.value = true;
  } else {
    await store.installNewModel(modelName, variantTag);
  }
};



const confirmHardwareInstall = async () => {
  if (pendingInstall.value) {
    showHardwareConfirm.value = false;
    await store.installNewModel(pendingInstall.value.name, pendingInstall.value.variant);
    pendingInstall.value = null;
  }
};

const cancelHardwareInstall = () => {
  showHardwareConfirm.value = false;
  pendingInstall.value = null;
};

const handleInstall = async (str: string, sizeBytes: number = 0) => {
  const parts = str.split(':');
  const name = parts[0];
  const variant = parts[1] || 'latest';
  await checkEfficiencyAndInstall(name, variant, sizeBytes);
};

const handleInstallDirect = async (str: string, sizeBytes: number = 0) => {
  const parts = str.split(':');
  const name = parts[0];
  const variant = parts[1] || 'latest';
  await checkEfficiencyAndInstall(name, variant, sizeBytes);
};

const getModelSizeRange = (variants: any[]) => {
  if (!variants || variants.length === 0) return '大小待定';
  
  // 过滤掉投影器 (projector/mmproj) 变体，仅计算主模型的大小范围
  const mainVariants = variants.filter(v => 
    !v.tag.toLowerCase().includes('mmproj') && 
    !v.tag.toLowerCase().includes('projector') &&
    !(v.params && v.params.toLowerCase().includes('mmproj')) &&
    !(v.params && v.params.toLowerCase().includes('projector'))
  );
  
  const targetVariants = mainVariants.length > 0 ? mainVariants : variants;
  const sizes = targetVariants.map(v => v.size).filter(s => s > 0);
  if (sizes.length === 0) return '大小待定';
  
  if (sizes.length === 1) return formatSize(sizes[0]);
  
  const minSize = Math.min(...sizes);
  const maxSize = Math.max(...sizes);
  return `${formatSize(minSize)} ~ ${formatSize(maxSize)}`;
};

</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- 区域 B：核心模型配置 -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="bg-panel-bg border border-border-main/50 rounded-xl p-4 space-y-3">
        <label class="text-xs font-bold text-text-muted uppercase tracking-wider">默认对话模型 (Default Chat)</label>
        <SelectRoot v-model="chatModel">
          <SelectTrigger class="inline-flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm bg-surface-bg border border-border-main/50 text-text-primary hover:bg-surface-bg/80 transition-colors overflow-hidden">
            <div class="truncate flex-1 text-left">
              <SelectValue placeholder="请选择对话模型..." />
            </div>
            <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
          </SelectTrigger>
          <SelectPortal>
            <SelectContent class="z-[100] bg-panel-bg border border-border-main/50 rounded-lg shadow-2xl p-1 min-w-[240px]">
              <SelectViewport>
                <SelectItem v-for="opt in chatModelOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-2 text-sm rounded-md text-text-secondary cursor-pointer hover:bg-brand hover:text-white outline-none overflow-hidden">
                  <SelectItemText class="truncate">{{ opt.label }}</SelectItemText>
                </SelectItem>
              </SelectViewport>
            </SelectContent>
          </SelectPortal>
        </SelectRoot>
      </div>

      <div class="bg-panel-bg border border-border-main/50 rounded-xl p-4 space-y-3">
        <label class="text-xs font-bold text-text-muted uppercase tracking-wider">语义检索模型 (Embedding)</label>
        <SelectRoot v-model="embeddingModel">
          <SelectTrigger class="inline-flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm bg-surface-bg border border-border-main/50 text-text-primary hover:bg-surface-bg/80 transition-colors overflow-hidden">
            <div class="truncate flex-1 text-left">
              <SelectValue placeholder="请选择向量模型..." />
            </div>
            <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
          </SelectTrigger>
          <SelectPortal>
            <SelectContent class="z-[100] bg-panel-bg border border-border-main/50 rounded-lg shadow-2xl p-1 min-w-[240px]">
              <SelectViewport>
                <SelectItem v-for="opt in embeddingModelOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-2 text-sm rounded-md text-text-secondary cursor-pointer hover:bg-brand hover:text-white outline-none overflow-hidden">
                  <SelectItemText class="truncate">{{ opt.label }}</SelectItemText>
                </SelectItem>
              </SelectViewport>
            </SelectContent>
          </SelectPortal>
        </SelectRoot>
      </div>
    </div>

      <div class="bg-panel-bg border border-border-main/50 rounded-xl p-4 space-y-3">
        <div class="mb-3">
          <label class="text-xs font-bold text-text-muted uppercase tracking-wider">视觉理解模型 (Vision)</label>
          <p class="text-[10px] text-text-muted mt-1 leading-relaxed">
            多模态图文问答引擎。系统会自动在后台管理配套的投影器文件，您只需选择主模型即可。
          </p>
        </div>
        <SelectRoot v-model="visionModel">
          <SelectTrigger class="inline-flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm bg-surface-bg border border-border-main/50 text-text-primary hover:bg-surface-bg/80 transition-colors overflow-hidden">
            <div class="truncate flex-1 text-left">
              <SelectValue placeholder="请选择视觉模型..." />
            </div>
            <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
          </SelectTrigger>
          <SelectPortal>
            <SelectContent class="z-[100] bg-panel-bg border border-border-main/50 rounded-lg shadow-2xl p-1 min-w-[240px]">
              <SelectViewport>
                <SelectItem v-for="opt in visionModelOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-2 text-sm rounded-md text-text-secondary cursor-pointer hover:bg-brand hover:text-white outline-none overflow-hidden">
                  <SelectItemText class="truncate">{{ opt.label }}</SelectItemText>
                </SelectItem>
              </SelectViewport>
            </SelectContent>
          </SelectPortal>
        </SelectRoot>
      </div>

    <!-- 区域 C：已安装模型概览 -->
    <div v-if="visibleInstalledModels.length > 0" class="space-y-4">
      <div class="flex items-center justify-between">
        <h5 class="text-xs font-bold text-text-muted uppercase tracking-widest">本地已就绪 ({{ visibleInstalledModels.length }} 套模型)</h5>
      </div>
      <div class="flex gap-3 overflow-x-auto pb-4 scrollbar-hide min-w-0">
        <div 
          v-for="model in visibleInstalledModels" 
          :key="model.full_name"
          class="flex-shrink-0 w-52 bg-surface-bg border border-border-main-light rounded-xl p-4 group hover:border-brand/30 transition-all shadow-md relative"
        >
          <div class="flex flex-col gap-3">
            <div class="flex justify-between items-start pr-6 overflow-hidden">
              <span class="text-xs font-bold text-text-primary truncate leading-tight flex-1" :title="model.display_name">{{ model.display_name }}</span>
              <div class="w-2 h-2 rounded-full bg-success-500 absolute right-4 top-4"></div>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-[9px] px-1.5 py-0.5 bg-panel-bg rounded text-text-muted font-mono border border-border-main/50">{{ model.tag }}</span>
              <span class="text-[9px] text-text-muted">{{ formatSize(model.size_bytes) }}</span>
            </div>
            <div class="flex justify-between items-center mt-1">
              <div v-if="model.description" class="text-[9px] text-text-muted truncate w-24">{{ model.description }}</div>
              <div v-else class="text-[9px] text-brand/50 italic">本地就绪</div>
              <button 
                @click="deletingModel = model.full_name"
                class="opacity-0 group-hover:opacity-100 p-1.5 text-error-500 hover:bg-error-500/10 rounded-md transition-all absolute right-2 bottom-3"
                title="删除模型"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 主 Tab 切换 -->
    <div class="flex flex-col gap-4 w-full min-w-0">
      <div class="flex items-center justify-between border-b border-border-main/50 pb-2 w-full">
        <div class="flex gap-2 text-xs font-semibold flex-nowrap overflow-x-auto scrollbar-hide">
          <button 
            @click="activeTab = 'recommend'" 
            class="px-4 py-2 transition-all relative"
            :class="activeTab === 'recommend' ? 'text-brand' : 'text-text-secondary hover:text-text-primary'"
          >
            为您推荐
            <div v-if="activeTab === 'recommend'" class="absolute bottom-[-9px] left-0 w-full h-0.5 bg-brand"></div>
          </button>
          <button 
            @click="activeTab = 'all'" 
            class="px-4 py-2 transition-all relative"
            :class="activeTab === 'all' ? 'text-brand' : 'text-text-secondary hover:text-text-primary'"
          >
            全量探索
            <div v-if="activeTab === 'all'" class="absolute bottom-[-9px] left-0 w-full h-0.5 bg-brand"></div>
          </button>
        </div>

        <div class="flex items-center gap-3">
          <!-- 数据源指引 -->
          <div v-if="store.modelHubData" class="flex items-center gap-1.5 px-2 py-1 bg-surface-bg border border-border-main/50 rounded-md">
            <span class="text-[10px] text-text-muted uppercase font-bold tracking-tighter">当前源:</span>
            <span class="text-[10px] text-brand font-bold uppercase">{{ store.modelHubData.is_china ? 'ModelScope' : 'HuggingFace' }}</span>
          </div>

          <button 
            @click="store.fetchModelHub(true)" 
            class="flex items-center gap-1.5 px-3 py-1 bg-surface-bg/50 border border-border-main/50 rounded-md text-[11px] font-bold text-text-secondary hover:text-text-primary hover:bg-border-main hover:border-border-main transition-all cursor-pointer group"
            :disabled="store.isSyncingHub"
            :class="{ 'opacity-50 cursor-not-allowed': store.isSyncingHub }"
            title="强制从服务器获取最新模型排行与规格"
          >
            <svg 
              xmlns="http://www.w3.org/2000/svg" 
              class="w-3.5 h-3.5 group-hover:text-brand transition-colors" 
              :class="{ 'animate-spin text-brand': store.isSyncingHub }"
              viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"
            >
              <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path>
              <path d="M3 3v5h5"></path>
              <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"></path>
              <path d="M16 21v-5h5"></path>
            </svg>
            强制刷新
          </button>
        </div>
      </div>

      <!-- 全局加载状态 -->
      <div v-if="store.isSyncingHub" class="flex flex-col items-center justify-center py-32 text-center space-y-4 animate-fadeIn">
        <div class="relative w-14 h-14 flex items-center justify-center">
          <div class="absolute inset-0 rounded-full border-[3px] border-border-main/60"></div>
          <div class="absolute inset-0 rounded-full border-[3px] border-brand border-t-transparent animate-spin"></div>
          <svg class="w-6 h-6 text-brand animate-pulse" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 0 1-9 9m9-9a9 9 0 0 0-9-9m9 9H3m9 9a9 9 0 0 1-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 0 1 9-9"></path></svg>
        </div>
        <div>
          <h3 class="text-sm font-bold text-text-primary">正在同步模型知识库</h3>
          <p class="text-xs text-text-muted mt-2 max-w-[280px] mx-auto leading-relaxed">
            正在同步云端模型库排行榜与规格列表，并检测您的硬件评估运行效率。此过程可能需要几秒钟，请稍候...
          </p>
        </div>
      </div>

      <template v-else>
        <!-- Tab 内容 1：为您推荐 -->
        <div v-if="activeTab === 'recommend'" class="space-y-8 animate-fadeIn">
          <!-- 官方首选推荐组合 (置顶看板) -->
          <div v-if="store.modelHubData?.primary_recommendations" class="bg-indigo-950/10 border border-indigo-900/20 rounded-2xl p-5 mb-8 shadow-sm">
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-3 mb-4">
              <div class="flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 3.138 3.42 3.42 0 00.806 1.946 3.42 3.42 0 010 4.438 3.42 3.42 0 00-.806 1.946 3.42 3.42 0 01-3.138 3.138 3.42 3.42 0 00-1.946.806 3.42 3.42 0 01-4.438 0 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-3.138-3.138 3.42 3.42 0 00-.806-1.946 3.42 3.42 0 010-4.438 3.42 3.42 0 00.806-1.946 3.42 3.42 0 013.138-3.138z" />
                </svg>
                <h4 class="text-sm font-bold text-text-primary">官方首选推荐 (根据本机配置自动匹配)</h4>
              </div>
              <div v-if="store.modelHubData.hardware" class="text-[10px] bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 px-2 py-0.5 rounded font-medium">
                本机配置: {{ store.modelHubData.hardware.os }} · {{ Math.round(store.modelHubData.hardware.ram_total / 1_073_741_824) }}GB RAM
              </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <!-- Chat Rec Card -->
              <div v-if="store.modelHubData.primary_recommendations.chat" class="bg-surface-bg border border-border-main-light/40 rounded-xl p-4 flex flex-col gap-3 relative hover:border-brand/40 transition-all">
                <div class="flex justify-between items-start gap-2">
                  <div class="flex flex-col gap-0.5">
                    <div class="flex items-center gap-1.5">
                      <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
                      <span class="text-[10px] text-text-muted font-semibold uppercase tracking-wider">对话推理模型</span>
                    </div>
                    <h5 class="text-sm font-bold text-text-primary mt-1">{{ store.modelHubData.primary_recommendations.chat.model.name }}</h5>
                  </div>
                  <span class="text-[9px] px-1.5 py-0.5 rounded bg-brand/10 text-brand border border-brand/20 font-mono">{{ store.modelHubData.primary_recommendations.chat.variant.params }}</span>
                </div>
                <p class="text-xs text-text-secondary line-clamp-2 leading-relaxed flex-1">{{ store.modelHubData.primary_recommendations.chat.model.description }}</p>
                
                <div class="flex justify-between items-center mt-2 pt-2 border-t border-border-main/20">
                  <span class="text-[10px] text-text-muted">大小: {{ (store.modelHubData.primary_recommendations.chat.variant.size / 1_073_741_824).toFixed(2) }} GB</span>
                  
                  <div class="flex gap-2">
                    <!-- Progress bar if downloading -->
                    <div v-if="getDownloadProgress(store.modelHubData.primary_recommendations.chat.model.name, store.modelHubData.primary_recommendations.chat.variant.tag)" class="text-xs font-semibold text-indigo-400 flex items-center gap-1">
                      <svg class="animate-spin h-3.5 w-3.5 text-indigo-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      <span>{{ Math.round(getDownloadProgress(store.modelHubData.primary_recommendations.chat.model.name, store.modelHubData.primary_recommendations.chat.variant.tag).percentage) }}%</span>
                    </div>

                    <!-- Installed State -->
                    <template v-else-if="isModelInstalled(store.modelHubData.primary_recommendations.chat.model.name, store.modelHubData.primary_recommendations.chat.variant.tag)">
                      <button 
                        v-if="store.settings.chat_model === getModelFilename(store.modelHubData.primary_recommendations.chat.model.name, store.modelHubData.primary_recommendations.chat.variant.tag)"
                        disabled
                        class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-green-500/10 text-green-400 border border-green-500/20"
                      >
                        已启用作为默认
                      </button>
                      <button 
                        v-else
                        @click="selectChatModel(store.modelHubData.primary_recommendations.chat.model.name, store.modelHubData.primary_recommendations.chat.variant.tag)"
                        class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-indigo-500/10 text-indigo-400 hover:bg-indigo-500/20 border border-indigo-500/30 transition-colors"
                      >
                        设为默认
                      </button>
                    </template>

                    <!-- Install Button -->
                    <button 
                      v-else
                      @click="store.installNewModel(store.modelHubData.primary_recommendations.chat.model.name, store.modelHubData.primary_recommendations.chat.variant.tag)"
                      class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white shadow transition-colors"
                    >
                      一键下载
                    </button>
                  </div>
                </div>
              </div>

              <!-- Embedding Rec Card -->
              <div v-if="store.modelHubData.primary_recommendations.embedding" class="bg-surface-bg border border-border-main-light/40 rounded-xl p-4 flex flex-col gap-3 relative hover:border-brand/40 transition-all">
                <div class="flex justify-between items-start gap-2">
                  <div class="flex flex-col gap-0.5">
                    <div class="flex items-center gap-1.5">
                      <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
                      <span class="text-[10px] text-text-muted font-semibold uppercase tracking-wider">检索向量模型</span>
                    </div>
                    <h5 class="text-sm font-bold text-text-primary mt-1">{{ store.modelHubData.primary_recommendations.embedding.model.name }}</h5>
                  </div>
                  <span class="text-[9px] px-1.5 py-0.5 rounded bg-brand/10 text-brand border border-brand/20 font-mono">{{ store.modelHubData.primary_recommendations.embedding.variant.params }}</span>
                </div>
                <p class="text-xs text-text-secondary line-clamp-2 leading-relaxed flex-1">{{ store.modelHubData.primary_recommendations.embedding.model.description }}</p>
                
                <div class="flex justify-between items-center mt-2 pt-2 border-t border-border-main/20">
                  <span class="text-[10px] text-text-muted">大小: {{ (store.modelHubData.primary_recommendations.embedding.variant.size / 1_073_741_824).toFixed(2) }} GB</span>
                  
                  <div class="flex gap-2">
                    <!-- Progress bar if downloading -->
                    <div v-if="getDownloadProgress(store.modelHubData.primary_recommendations.embedding.model.name, store.modelHubData.primary_recommendations.embedding.variant.tag)" class="text-xs font-semibold text-indigo-400 flex items-center gap-1">
                      <svg class="animate-spin h-3.5 w-3.5 text-indigo-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      <span>{{ Math.round(getDownloadProgress(store.modelHubData.primary_recommendations.embedding.model.name, store.modelHubData.primary_recommendations.embedding.variant.tag).percentage) }}%</span>
                    </div>

                    <!-- Installed State -->
                    <template v-else-if="isModelInstalled(store.modelHubData.primary_recommendations.embedding.model.name, store.modelHubData.primary_recommendations.embedding.variant.tag)">
                      <button 
                        v-if="store.settings.embedding_model === getModelFilename(store.modelHubData.primary_recommendations.embedding.model.name, store.modelHubData.primary_recommendations.embedding.variant.tag)"
                        disabled
                        class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-green-500/10 text-green-400 border border-green-500/20"
                      >
                        已启用作为默认
                      </button>
                      <button 
                        v-else
                        @click="selectEmbeddingModel(store.modelHubData.primary_recommendations.embedding.model.name, store.modelHubData.primary_recommendations.embedding.variant.tag)"
                        class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-indigo-500/10 text-indigo-400 hover:bg-indigo-500/20 border border-indigo-500/30 transition-colors"
                      >
                        设为默认
                      </button>
                    </template>

                    <!-- Install Button -->
                    <button 
                      v-else
                      @click="store.installNewModel(store.modelHubData.primary_recommendations.embedding.model.name, store.modelHubData.primary_recommendations.embedding.variant.tag)"
                      class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white shadow transition-colors"
                    >
                      一键下载
                    </button>
                  </div>
                </div>
              </div>

              <!-- Vision Rec Card -->
              <div v-if="store.modelHubData.primary_recommendations.vision" class="bg-surface-bg border border-border-main-light/40 rounded-xl p-4 flex flex-col gap-3 relative hover:border-brand/40 transition-all">
                <div class="flex justify-between items-start gap-2">
                  <div class="flex flex-col gap-0.5">
                    <div class="flex items-center gap-1.5">
                      <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
                      <span class="text-[10px] text-text-muted font-semibold uppercase tracking-wider">多模态视觉模型</span>
                    </div>
                    <h5 class="text-sm font-bold text-text-primary mt-1">{{ store.modelHubData.primary_recommendations.vision.model.name }}</h5>
                  </div>
                  <span class="text-[9px] px-1.5 py-0.5 rounded bg-brand/10 text-brand border border-brand/20 font-mono">{{ store.modelHubData.primary_recommendations.vision.variant.params }}</span>
                </div>
                <p class="text-xs text-text-secondary line-clamp-2 leading-relaxed flex-1">{{ store.modelHubData.primary_recommendations.vision.model.description }}</p>
                
                <div class="flex justify-between items-center mt-2 pt-2 border-t border-border-main/20">
                  <span class="text-[10px] text-text-muted">大小: {{ (store.modelHubData.primary_recommendations.vision.variant.size / 1_073_741_824).toFixed(2) }} GB</span>
                  
                  <div class="flex gap-2">
                    <!-- Progress bar if downloading -->
                    <div v-if="getDownloadProgress(store.modelHubData.primary_recommendations.vision.model.name, store.modelHubData.primary_recommendations.vision.variant.tag)" class="text-xs font-semibold text-indigo-400 flex items-center gap-1">
                      <svg class="animate-spin h-3.5 w-3.5 text-indigo-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      <span>{{ Math.round(getDownloadProgress(store.modelHubData.primary_recommendations.vision.model.name, store.modelHubData.primary_recommendations.vision.variant.tag).percentage) }}%</span>
                    </div>

                    <!-- Installed State -->
                    <template v-else-if="isModelInstalled(store.modelHubData.primary_recommendations.vision.model.name, store.modelHubData.primary_recommendations.vision.variant.tag)">
                      <button 
                        v-if="store.settings.vision_model === getModelFilename(store.modelHubData.primary_recommendations.vision.model.name, store.modelHubData.primary_recommendations.vision.variant.tag)"
                        disabled
                        class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-green-500/10 text-green-400 border border-green-500/20"
                      >
                        已启用作为默认
                      </button>
                      <button 
                        v-else
                        @click="selectVisionModel(store.modelHubData.primary_recommendations.vision.model.name, store.modelHubData.primary_recommendations.vision.variant.tag)"
                        class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-indigo-500/10 text-indigo-400 hover:bg-indigo-500/20 border border-indigo-500/30 transition-colors"
                      >
                        设为默认
                      </button>
                    </template>

                    <!-- Install Button -->
                    <button 
                      v-else
                      @click="store.installNewModel(store.modelHubData.primary_recommendations.vision.model.name, store.modelHubData.primary_recommendations.vision.variant.tag)"
                      class="text-[10px] font-bold px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white shadow transition-colors"
                    >
                      一键下载
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 对话、向量、视觉并列 3 列推荐 -->
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
            <!-- ① 对话模型推荐 -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 pb-2 border-b border-border-main/20">
                <div class="w-1.5 h-4 bg-brand rounded-full"></div>
                <h5 class="text-xs font-bold text-text-primary uppercase tracking-wider">对话推荐 (Chat Models)</h5>
              </div>
              <div class="flex flex-col gap-4">
                <div 
                  v-for="rec in chatRecommendations" 
                  :key="rec.model.name"
                  class="bg-surface-bg border-2 border-border-main-light rounded-xl p-4 flex flex-col gap-3 relative group hover:border-brand/50 transition-all duration-300 shadow-md"
                >
                  <div class="flex justify-between items-start gap-2">
                    <div class="flex flex-col gap-1 overflow-hidden">
                      <h6 class="text-sm font-bold text-text-primary truncate" :title="rec.model.name">{{ rec.model.name }}</h6>
                      <div class="flex gap-1">
                        <span :class="getModelTypeColor(rec.model.category)" class="text-[9px] px-1.5 py-0.5 rounded uppercase font-bold">{{ getModelTypeName(rec.model.category) }}</span>
                      </div>
                    </div>
                    <span class="flex-shrink-0 text-[10px] bg-brand/10 text-brand px-1.5 py-0.5 rounded border border-brand/20 font-mono">{{ getModelSizeRange(rec.model.variants) }}</span>
                  </div>
                  <p class="text-xs text-text-secondary line-clamp-2 min-h-[2.5rem]">{{ rec.model.description }}</p>
                  
                  <!-- 折叠内容容器 -->
                  <div 
                    class="overflow-hidden transition-all duration-300 ease-in-out"
                    :class="expandedRecommendModels[rec.model.name] ? 'max-h-[500px] opacity-100 mt-2' : 'max-h-0 opacity-0'"
                  >
                    <div class="flex flex-col gap-1.5 pt-2 border-t border-border-main/30">
                      <label class="text-[9px] font-bold text-text-muted uppercase tracking-wider mb-1 block">可选规格 · 运行效率</label>
                      <div class="flex flex-col gap-1">
                        <div
                          v-for="v in sortVariantsByEfficiency(rec.model.variants)"
                          :key="v.tag"
                          class="flex items-center justify-between px-2 py-1.5 rounded-lg border transition-colors"
                          :class="getVariantEfficiency(v.size).color + ' border-opacity-40'"
                        >
                          <div class="flex items-center gap-1.5 min-w-0 flex-1 overflow-hidden">
                            <span class="text-[9px] flex-shrink-0" :title="getVariantEfficiency(v.size).description">{{ getVariantEfficiency(v.size).icon }}</span>
                            <span class="text-[10px] font-bold font-mono truncate flex-1" :title="v.params">{{ v.params }}</span>
                            <span class="text-[9px] opacity-70 flex-shrink-0">{{ v.size > 0 ? formatSize(v.size) : '' }}</span>
                          </div>
                          <div class="flex items-center gap-1">
                            <span
                              class="text-[8px] font-bold px-1 py-0.5 rounded"
                              :class="getVariantEfficiency(v.size).color"
                              :title="getVariantEfficiency(v.size).description"
                            >{{ getVariantEfficiency(v.size).label }}</span>
                            <button
                              v-if="!isInstalled(rec.model.name)"
                              @click.stop="handleInstall(`${rec.model.name}:${v.tag}`, v.size)"
                              class="p-1 rounded hover:bg-text-primary/10 transition-colors"
                              :title="`下载 ${rec.model.name}:${v.tag}`"
                            >
                              <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <div class="mt-auto pt-2 border-t border-border-main/30 flex justify-between items-center gap-2 overflow-hidden">
                    <span class="text-[10px] text-brand font-bold truncate flex-1 min-w-0" :title="rec.reason">{{ rec.reason }}</span>
                    <div v-if="isInstalled(rec.model.name)" class="text-success-500 font-bold text-xs uppercase tracking-tighter shrink-0">已就绪</div>
                  </div>

                  <!-- 折叠/展开控制按钮 -->
                  <button 
                    @click.stop="toggleRecommendExpand(rec.model.name)"
                    class="w-full flex items-center justify-center gap-1.5 py-1 px-3 bg-surface-bg border border-border-main/50 hover:bg-border-main/20 text-[10px] font-bold rounded-lg text-text-secondary hover:text-text-primary transition-all mt-2 cursor-pointer group"
                  >
                    <span>{{ expandedRecommendModels[rec.model.name] ? '收起规格' : '展开规格' }}</span>
                    <svg 
                      xmlns="http://www.w3.org/2000/svg" 
                      class="w-3.5 h-3.5 text-text-muted group-hover:text-text-primary transition-transform duration-300"
                      :class="{ 'rotate-180': expandedRecommendModels[rec.model.name] }"
                      viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
                    >
                      <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                  </button>
                </div>
              </div>
            </div>

            <!-- ② 向量检索推荐 -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 pb-2 border-b border-border-main/20">
                <div class="w-1.5 h-4 bg-success-500 rounded-full"></div>
                <h5 class="text-xs font-bold text-text-primary uppercase tracking-wider">向量推荐 (RAG Embedding)</h5>
              </div>
              <div class="flex flex-col gap-4">
                <div 
                  v-for="rec in embeddingRecommendations" 
                  :key="rec.model.name"
                  class="bg-surface-bg border-2 border-border-main-light rounded-xl p-4 flex flex-col gap-3 relative group hover:border-success-500/50 transition-all duration-300 shadow-md"
                >
                  <div class="flex justify-between items-start gap-2">
                    <div class="flex flex-col gap-1 overflow-hidden">
                      <h6 class="text-sm font-bold text-text-primary truncate" :title="rec.model.name">{{ rec.model.name }}</h6>
                      <div class="flex gap-1">
                        <span :class="getModelTypeColor(rec.model.category)" class="text-[9px] px-1.5 py-0.5 rounded uppercase font-bold">{{ getModelTypeName(rec.model.category) }}</span>
                      </div>
                    </div>
                    <span class="flex-shrink-0 text-[10px] bg-success-500/10 text-success-500 px-1.5 py-0.5 rounded border border-success-500/20 font-mono">{{ getModelSizeRange(rec.model.variants) }}</span>
                  </div>
                  <p class="text-xs text-text-secondary line-clamp-2 min-h-[2.5rem]">{{ rec.model.description }}</p>

                  <!-- 折叠内容容器 -->
                  <div 
                    class="overflow-hidden transition-all duration-300 ease-in-out"
                    :class="expandedRecommendModels[rec.model.name] ? 'max-h-[500px] opacity-100 mt-2' : 'max-h-0 opacity-0'"
                  >
                    <div class="flex flex-col gap-1.5 pt-2 border-t border-border-main/30">
                      <label class="text-[9px] font-bold text-text-muted uppercase tracking-wider mb-1 block">可选规格 · 运行效率</label>
                      <div class="flex flex-col gap-1">
                        <div
                          v-for="v in sortVariantsByEfficiency(rec.model.variants)"
                          :key="v.tag"
                          class="flex items-center justify-between px-2 py-1.5 rounded-lg border transition-colors"
                          :class="getVariantEfficiency(v.size).color + ' border-opacity-40'"
                        >
                          <div class="flex items-center gap-1.5 min-w-0 flex-1 overflow-hidden">
                            <span class="text-[9px] flex-shrink-0" :title="getVariantEfficiency(v.size).description">{{ getVariantEfficiency(v.size).icon }}</span>
                            <span class="text-[10px] font-bold font-mono truncate flex-1" :title="v.params">{{ v.params }}</span>
                            <span class="text-[9px] opacity-70 flex-shrink-0">{{ v.size > 0 ? formatSize(v.size) : '' }}</span>
                          </div>
                          <div class="flex items-center gap-1">
                            <span
                              class="text-[8px] font-bold px-1 py-0.5 rounded"
                              :class="getVariantEfficiency(v.size).color"
                              :title="getVariantEfficiency(v.size).description"
                            >{{ getVariantEfficiency(v.size).label }}</span>
                            <button
                              v-if="!isInstalled(rec.model.name)"
                              @click.stop="handleInstall(`${rec.model.name}:${v.tag}`, v.size)"
                              class="p-1 rounded hover:bg-text-primary/10 transition-colors"
                              :title="`下载 ${rec.model.name}:${v.tag}`"
                            >
                              <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="mt-auto pt-2 border-t border-border-main/30 flex justify-between items-center gap-2 overflow-hidden">
                    <span class="text-[10px] text-success-500 font-bold truncate flex-1 min-w-0" :title="rec.reason">{{ rec.reason }}</span>
                    <div v-if="isInstalled(rec.model.name)" class="text-success-500 font-bold text-xs uppercase tracking-tighter shrink-0">已就绪</div>
                  </div>

                  <!-- 折叠/展开控制按钮 -->
                  <button 
                    @click.stop="toggleRecommendExpand(rec.model.name)"
                    class="w-full flex items-center justify-center gap-1.5 py-1 px-3 bg-surface-bg border border-border-main/50 hover:bg-border-main/20 text-[10px] font-bold rounded-lg text-text-secondary hover:text-text-primary transition-all mt-2 cursor-pointer group"
                  >
                    <span>{{ expandedRecommendModels[rec.model.name] ? '收起规格' : '展开规格' }}</span>
                    <svg 
                      xmlns="http://www.w3.org/2000/svg" 
                      class="w-3.5 h-3.5 text-text-muted group-hover:text-text-primary transition-transform duration-300"
                      :class="{ 'rotate-180': expandedRecommendModels[rec.model.name] }"
                      viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
                    >
                      <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                  </button>
                </div>
              </div>
            </div>

            <!-- ③ 视觉理解推荐 -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 pb-2 border-b border-border-main/20">
                <div class="w-1.5 h-4 bg-purple-500 rounded-full"></div>
                <h5 class="text-xs font-bold text-text-primary uppercase tracking-wider">视觉推荐 (Vision Models)</h5>
              </div>
              <div class="flex flex-col gap-4">
                <div 
                  v-for="rec in visionRecommendations" 
                  :key="rec.model.name"
                  class="bg-surface-bg border-2 border-border-main-light rounded-xl p-4 flex flex-col gap-3 relative group hover:border-purple-500/50 transition-all duration-300 shadow-md"
                >
                  <div class="flex justify-between items-start gap-2">
                    <div class="flex flex-col gap-1 overflow-hidden">
                      <h6 class="text-sm font-bold text-text-primary truncate" :title="rec.model.name">{{ rec.model.name }}</h6>
                      <div class="flex gap-1">
                        <span :class="getModelTypeColor(rec.model.category)" class="text-[9px] px-1.5 py-0.5 rounded uppercase font-bold">{{ getModelTypeName(rec.model.category) }}</span>
                      </div>
                    </div>
                    <span class="flex-shrink-0 text-[10px] bg-purple-500/10 text-purple-500 px-1.5 py-0.5 rounded border border-purple-500/20 font-mono">{{ getModelSizeRange(rec.model.variants) }}</span>
                  </div>
                  <p class="text-xs text-text-secondary line-clamp-2 min-h-[2.5rem]">{{ rec.model.description }}</p>

                  <!-- 折叠内容容器 -->
                  <div 
                    class="overflow-hidden transition-all duration-300 ease-in-out"
                    :class="expandedRecommendModels[rec.model.name] ? 'max-h-[500px] opacity-100 mt-2' : 'max-h-0 opacity-0'"
                  >
                    <div class="flex flex-col gap-1.5 pt-2 border-t border-border-main/30">
                      <label class="text-[9px] font-bold text-text-muted uppercase tracking-wider mb-1 block">可选规格 · 运行效率</label>
                      <div class="flex flex-col gap-1">
                        <div
                          v-for="v in sortVariantsByEfficiency(rec.model.variants)"
                          :key="v.tag"
                          class="flex items-center justify-between px-2 py-1.5 rounded-lg border transition-colors"
                          :class="getVariantEfficiency(v.size).color + ' border-opacity-40'"
                        >
                          <div class="flex items-center gap-1.5 min-w-0 flex-1 overflow-hidden">
                            <span class="text-[9px] flex-shrink-0" :title="getVariantEfficiency(v.size).description">{{ getVariantEfficiency(v.size).icon }}</span>
                            <span class="text-[10px] font-bold font-mono truncate flex-1" :title="v.params">{{ v.params }}</span>
                            <span class="text-[9px] opacity-70 flex-shrink-0">{{ v.size > 0 ? formatSize(v.size) : '' }}</span>
                          </div>
                          <div class="flex items-center gap-1">
                            <span
                              class="text-[8px] font-bold px-1 py-0.5 rounded"
                              :class="getVariantEfficiency(v.size).color"
                              :title="getVariantEfficiency(v.size).description"
                            >{{ getVariantEfficiency(v.size).label }}</span>
                            <button
                              v-if="!isInstalled(rec.model.name)"
                              @click.stop="handleInstall(`${rec.model.name}:${v.tag}`, v.size)"
                              class="p-1 rounded hover:bg-text-primary/10 transition-colors"
                              :title="`下载 ${rec.model.name}:${v.tag}`"
                            >
                              <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="mt-auto pt-2 border-t border-border-main/30 flex justify-between items-center gap-2 overflow-hidden">
                    <span class="text-[10px] text-purple-500 font-bold truncate flex-1 min-w-0" :title="rec.reason">{{ rec.reason }}</span>
                    <div v-if="isInstalled(rec.model.name)" class="text-success-500 font-bold text-xs uppercase tracking-tighter shrink-0">已就绪</div>
                  </div>

                  <!-- 折叠/展开控制按钮 -->
                  <button 
                    @click.stop="toggleRecommendExpand(rec.model.name)"
                    class="w-full flex items-center justify-center gap-1.5 py-1 px-3 bg-surface-bg border border-border-main/50 hover:bg-border-main/20 text-[10px] font-bold rounded-lg text-text-secondary hover:text-text-primary transition-all mt-2 cursor-pointer group"
                  >
                    <span>{{ expandedRecommendModels[rec.model.name] ? '收起规格' : '展开规格' }}</span>
                    <svg 
                      xmlns="http://www.w3.org/2000/svg" 
                      class="w-3.5 h-3.5 text-text-muted group-hover:text-text-primary transition-transform duration-300"
                      :class="{ 'rotate-180': expandedRecommendModels[rec.model.name] }"
                      viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
                    >
                      <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>


          <!-- ③ 排行榜 -->
          <div v-if="store.modelHubData" class="bg-panel-bg border border-border-main/40 rounded-2xl overflow-hidden shadow-xl animate-in zoom-in-95 duration-500">
            <div class="px-5 py-4 border-b border-border-main/30 flex items-center justify-between bg-surface-bg/10">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-xl flex items-center justify-center text-sm shadow-inner" :class="activeRankingTab === 'leaderboard' ? 'bg-orange-500/20' : 'bg-cyan-500/20'">{{ activeRankingTab === 'leaderboard' ? '🏆' : '✨' }}</div>
                <div>
                  <h3 class="text-sm font-bold text-text-primary leading-none">{{ activeRankingTab === 'leaderboard' ? '全网最受欢迎榜' : '最新发布模型榜' }}</h3>
                  <p class="text-[10px] text-text-muted mt-1">{{ activeRankingTab === 'leaderboard' ? '基于社区总下载量排序' : '同步官方最新上架顺序' }}</p>
                </div>
              </div>
              <div class="flex p-1 bg-surface-bg rounded-lg border border-border-main/50">
                <button @click="activeRankingTab = 'leaderboard'" class="px-3 py-1.5 text-[10px] font-bold rounded-md transition-all" :class="activeRankingTab === 'leaderboard' ? 'bg-brand text-white shadow-lg' : 'text-text-muted hover:text-text-primary'">最受欢迎</button>
                <button @click="activeRankingTab = 'trending'" class="px-3 py-1.5 text-[10px] font-bold rounded-md transition-all" :class="activeRankingTab === 'trending' ? 'bg-cyan-600 text-white shadow-lg' : 'text-text-muted hover:text-text-primary'">最新发布</button>
              </div>
            </div>
            <div class="divide-y divide-border-main/20">
              <div v-for="(model, index) in (activeRankingTab === 'leaderboard' ? store.modelHubData.leaderboard : store.modelHubData.trending).slice(0, 100)" :key="model.name" class="group">
                <div class="px-5 py-4 flex items-center justify-between hover:bg-surface-bg/40 transition-colors">
                  <div class="flex items-center gap-4">
                    <span :class="index < 3 ? 'text-brand font-black italic scale-110' : 'text-text-muted font-mono'" class="text-sm w-4 flex-shrink-0">{{ index + 1 }}</span>
                    <div class="flex flex-col min-w-0 flex-1">
                      <div class="flex items-center gap-2 overflow-hidden">
                        <span class="text-xs font-bold text-text-primary truncate" :title="model.name">{{ model.name }}</span>
                        <span v-if="index === 0" class="flex-shrink-0 text-[8px] px-1.5 py-0.5 rounded-full bg-brand/10 text-brand border border-brand/20 font-bold uppercase">👑 No.1</span>
                        <span :class="getModelTypeColor(model.category)" class="flex-shrink-0 text-[8px] px-1.5 py-0.5 rounded font-bold">{{ getModelTypeName(model.category) }}</span>
                      </div>
                      <div class="flex items-center gap-3 mt-0.5">
                        <span class="text-[10px] text-text-muted flex items-center gap-1">{{ model.pulls }} pulls</span>
                        <span class="text-[10px] text-text-muted">•</span>
                        <span class="text-[10px] text-text-muted flex items-center gap-1">{{ getModelSizeRange(model.variants) }}</span>
                        <span class="text-[10px] text-text-muted">•</span>
                        <span class="text-[10px] text-text-muted">{{ model.updated }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button @click="toggleSearchExpand(model.name)" class="p-2 rounded-lg bg-surface-bg border border-border-main/50 text-text-muted hover:text-text-primary transition-all" :class="expandedSearchModel === model.name ? 'text-brand bg-brand/5 border-brand/30' : ''">
                      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" :class="expandedSearchModel === model.name ? 'rotate-180' : ''" class="transition-transform"><polyline points="6 9 12 15 18 9"></polyline></svg>
                    </button>
                    <button v-if="!isInstalled(model.name)" @click="handleInstall(`${model.name}:latest`, 0)" class="px-3 py-1.5 rounded-lg bg-brand text-white text-[10px] font-bold hover:bg-brand shadow-lg shadow-brand/20">闪电安装</button>
                    <div v-else class="px-3 py-1.5 rounded-lg bg-success-500/10 text-success-500 text-[10px] font-bold border border-success-500/20">已安装</div>
                  </div>
                </div>
                <!-- 排行榜规格展开 -->
                <div v-if="expandedSearchModel === model.name" class="px-5 py-4 bg-surface-bg/50 border-t border-border-main/20 animate-in slide-in-from-top-1 duration-200">
                  <label class="text-[10px] font-bold text-text-muted uppercase tracking-wider mb-3 block">可选规格 · 运行效率</label>
                  <div v-if="loadingVariants[model.name]" class="flex flex-col gap-2 py-1">
                    <div v-for="i in 3" :key="i" class="h-10 rounded-xl bg-border-main/20 animate-pulse"></div>
                  </div>
                  <div v-else class="flex flex-col gap-1.5">
                    <div v-for="variant in sortVariantsByEfficiency(variantsCache[model.name] ?? model.variants)" :key="variant.tag" class="flex items-center justify-between px-3 py-2 rounded-xl border transition-colors" :class="getVariantEfficiency(variant.size).color">
                      <div class="flex items-center gap-2">
                        <span class="text-sm">{{ getVariantEfficiency(variant.size).icon }}</span>
                        <div class="flex flex-col min-w-0 flex-1">
                          <span class="text-xs font-bold text-text-primary uppercase truncate">{{ variant.params }}</span>
                          <span class="text-[9px] text-text-muted">{{ variant.size > 0 ? formatSize(variant.size) : '大小待定' }}</span>
                        </div>
                      </div>
                      <button @click.stop="handleInstallDirect(`${model.name}:${variant.tag}`, variant.size)" class="p-1.5 bg-brand/10 text-brand border border-brand/30 rounded-lg hover:bg-brand hover:text-white transition-all flex-shrink-0">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Tab 内容 2：全量探索 -->
        <div v-else class="space-y-4 animate-fadeIn">
          <div class="text-[10px] text-text-muted italic px-2">提示：全量探索模式下您可以看到官网的所有历史模型元数据。</div>
          <div class="space-y-4">
            <!-- 搜索框 -->
            <div class="flex items-center gap-2">
              <div class="relative flex-1">
                <input 
                  v-model="searchQuery" 
                  type="text" 
                  placeholder="搜索模型名称或厂商 (输入后自动检索)..."
                  @keyup.enter="handleEnterSearch"
                  class="w-full pl-8 pr-3 py-2 text-xs bg-surface-bg border-2 border-border-main-light rounded-md text-text-primary focus:border-brand/50 outline-none transition-colors"
                />
                <svg xmlns="http://www.w3.org/2000/svg" class="absolute left-2.5 top-2.5 w-3.5 h-3.5 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <circle cx="11" cy="11" r="8"></circle>
                  <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                </svg>
              </div>
            </div>
            
            <!-- 搜索加载与结果展示 -->
            <div v-if="store.isSearching" class="space-y-4 mt-6">
              <div v-for="i in 3" :key="i" class="flex items-center justify-between p-4 bg-surface-bg/30 border border-border-main/20 rounded-xl relative overflow-hidden">
                <div class="flex items-center gap-4 w-full">
                  <div class="w-10 h-10 bg-panel-bg rounded-lg shimmer"></div>
                  <div class="flex-1 space-y-2">
                    <div class="h-4 bg-panel-bg rounded w-1/4 shimmer"></div>
                    <div class="h-3 bg-panel-bg rounded w-3/4 shimmer"></div>
                  </div>
                </div>
                <div class="w-20 h-8 bg-panel-bg rounded-full shimmer"></div>
              </div>
            </div>

            <div v-else-if="store.searchResultModels.length > 0" class="space-y-2 mt-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
              <div v-for="model in store.searchResultModels" :key="model.name" class="flex flex-col bg-surface-bg/40 border border-border-main/30 rounded-lg overflow-hidden group transition-all">
                <div class="flex items-center justify-between p-3 cursor-pointer hover:bg-surface-bg" @click="toggleSearchExpand(model.name)">
                  <div class="flex items-center gap-4">
                    <div class="p-2 bg-panel-bg rounded-lg">
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted"><path d="M12 2v20"></path><path d="M2 12h20"></path></svg>
                    </div>
                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-3 overflow-hidden">
                        <span class="text-sm font-bold text-text-primary truncate" :title="model.name">{{ model.name }}</span>
                        <div class="flex gap-1 items-center flex-wrap">
                          <span v-for="cap in model.capabilities" :key="cap" :class="getCapabilityBadgeStyle(cap)" class="text-[9px] px-1.5 py-0.5 rounded-sm font-bold">{{ translateCapability(cap) }}</span>
                          <span class="text-[10px] text-text-muted font-mono ml-1">{{ model.pulls }} pulls</span>
                          <span class="text-[10px] text-text-muted">•</span>
                          <span class="text-[10px] text-text-muted flex items-center gap-1">
                            <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                            {{ getModelSizeRange(model.variants) }}
                          </span>
                        </div>
                      </div>
                      <div class="text-xs text-text-secondary line-clamp-1 max-w-lg">{{ model.description || '暂无描述' }}</div>
                    </div>
                  </div>
                  <div class="flex items-center gap-3">
                    <div v-if="isInstalled(model.name)" class="text-[10px] font-bold text-success-500 bg-success-500/10 px-2 py-1 rounded-full">已就绪</div>
                    <button v-else class="p-1 text-text-muted hover:text-brand transition-colors">
                      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :class="expandedSearchModel === model.name ? 'rotate-180' : ''" class="transition-transform"><polyline points="6 9 12 15 18 9"></polyline></svg>
                    </button>
                  </div>
                </div>
                <!-- 详情展开 -->
                <div v-if="expandedSearchModel === model.name" class="p-3 border-t border-border-main/30 bg-panel-bg/30 animate-in slide-in-from-top-1 duration-200">
                  <label class="text-[10px] font-bold text-text-muted uppercase tracking-wider mb-3 block">可选下载规格 · 按尺寸安装</label>
                  <div v-if="loadingVariants[model.name]" class="flex flex-col gap-2 py-1">
                    <div v-for="i in 3" :key="i" class="h-10 rounded-xl bg-border-main/20 animate-pulse"></div>
                  </div>
                  <div v-else class="flex flex-col gap-1.5">
                    <div v-for="variant in sortVariantsByEfficiency(variantsCache[model.name] ?? model.variants)" :key="variant.tag" class="flex items-center justify-between px-3 py-2 rounded-xl border transition-colors" :class="getVariantEfficiency(variant.size).color">
                      <div class="flex items-center gap-2">
                        <span class="text-sm">{{ getVariantEfficiency(variant.size).icon }}</span>
                        <div class="flex flex-col min-w-0 flex-1">
                          <span class="text-xs font-bold text-text-primary uppercase truncate">{{ variant.params }}</span>
                          <span class="text-[9px] text-text-muted">{{ variant.size > 0 ? formatSize(variant.size) : '大小待定' }}</span>
                        </div>
                      </div>
                      <button @click.stop="handleInstallDirect(`${model.name}:${variant.tag}`, variant.size)" class="p-1.5 bg-brand/10 text-brand border border-brand/30 rounded-lg hover:bg-brand hover:text-white transition-all flex-shrink-0">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div v-else-if="searchQuery" class="flex flex-col items-center justify-center py-16 text-center space-y-4 animate-in zoom-in-95 duration-300">
              <div class="w-16 h-16 rounded-full bg-surface-bg flex items-center justify-center border border-border-main/50">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-text-muted opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <circle cx="11" cy="11" r="8"></circle>
                  <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                </svg>
              </div>
              <div class="space-y-1">
                <p class="text-sm font-bold text-text-primary">未找到相关模型</p>
                <p class="text-xs text-text-muted max-w-xs leading-relaxed">正在搜索或未找到匹配项。直接回车将触发在线全局检索。</p>
              </div>
            </div>
            <div v-else class="flex flex-col items-center justify-center py-12 border-2 border-dashed border-border-main/30 rounded-2xl bg-surface-bg/20">
              <p class="text-xs text-text-muted">已就绪模型可优先体验</p>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>

  <!-- 删除确认对话框 -->
  <div v-if="deletingModel" class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="bg-surface-bg border border-border-main rounded-2xl p-6 w-full max-w-sm shadow-2xl animate-in zoom-in-95 duration-200">
      <div class="flex flex-col gap-4">
        <div class="w-12 h-12 rounded-full bg-error-500/10 flex items-center justify-center text-error-500">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>
        </div>
        <div class="space-y-1">
          <h3 class="text-lg font-bold text-text-primary">确认删除模型？</h3>
          <p class="text-sm text-text-secondary leading-relaxed">您即将删除 <span class="text-text-primary font-mono font-bold">{{ deletingModel }}</span>。此操作将从磁盘移除所有相关模型文件且无法撤销。</p>
        </div>
        <div class="flex gap-3 mt-2">
          <button @click="deletingModel = null" class="flex-1 py-2 rounded-xl bg-panel-bg border border-border-main text-sm font-bold text-text-secondary hover:bg-border-main transition-colors">取消</button>
          <button @click="handleConfirmDelete" class="flex-1 py-2 rounded-xl bg-error-500 text-white text-sm font-bold hover:bg-error-600 transition-colors shadow-lg shadow-error-500/20">确认删除</button>
        </div>
      </div>
    </div>
  </div>

  <!-- 硬件不足确认二次弹窗 -->
  <div v-if="showHardwareConfirm" class="fixed inset-0 z-[150] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="bg-surface-bg border border-border-main rounded-2xl p-6 w-[400px] shadow-2xl flex flex-col gap-4 animate-in zoom-in-95 duration-200">
      <div class="flex items-center gap-3 text-amber-500">
        <div class="w-10 h-10 rounded-full bg-amber-500/10 flex items-center justify-center flex-shrink-0 border border-amber-500/20">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
        </div>
        <h3 class="text-lg font-bold">硬件规格异常警告</h3>
      </div>
      <div class="space-y-1">
        <p class="text-sm text-text-secondary leading-relaxed">您即将下载的 <span class="text-text-primary font-mono font-bold">{{ pendingInstall?.name }}:{{ pendingInstall?.variant }}</span> 尺寸超出了当前设备的硬件容量限制。这可能会导致推理缓慢、系统内存溢出或程序崩溃。</p>
      </div>
      <div class="flex gap-3 mt-2">
        <button @click="cancelHardwareInstall" class="flex-1 py-2 rounded-xl bg-panel-bg border border-border-main text-sm font-bold text-text-secondary hover:bg-border-main transition-colors">取消下载</button>
        <button @click="confirmHardwareInstall" class="flex-1 py-2 rounded-xl bg-amber-500 text-white text-sm font-bold hover:bg-amber-600 transition-colors shadow-lg shadow-amber-500/20">继续下载</button>
      </div>
    </div>
  </div>


</template>

<style scoped>
.animate-fadeIn {
  animation: fadeIn 0.4s ease-out;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

.shimmer {
  position: relative;
  background: rgba(255, 255, 255, 0.03);
  overflow: hidden;
}

.shimmer::after {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  transform: translateX(-100%);
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0,
    rgba(255, 255, 255, 0.03) 20%,
    rgba(255, 255, 255, 0.08) 60%,
    rgba(255, 255, 255, 0)
  );
  animation: shimmer-pulse 2.5s infinite linear;
}

@keyframes shimmer-pulse {
  100% {
    transform: translateX(100%);
  }
}

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}
.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
