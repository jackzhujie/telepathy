<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { useReindexStore } from '@/stores/reindex';
import { getDocuments, reindexAllDocuments } from '@/api/tauri';
import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogOverlay,
  AlertDialogPortal,
  AlertDialogRoot,
  AlertDialogTitle,
  SliderRange,
  SliderRoot,
  SliderThumb,
  SliderTrack,
} from 'reka-ui';
import ModelManager from '@/components/settings/ModelManager.vue';

defineOptions({ name: 'Models' });

const store = useSettingsStore();
const reindexStore = useReindexStore();

import { useNotificationsStore } from '@/stores/notifications';
const notificationsStore = useNotificationsStore();

// UI 状态
const activeTab = ref<'manager' | 'advanced'>('manager');
const saving = ref<Record<string, boolean>>({});
const toast = ref<{ message: string; visible: boolean }>({ message: '', visible: false });

const showToast = (message: string) => {
  toast.value.message = message;
  toast.value.visible = true;
  setTimeout(() => {
    toast.value.visible = false;
  }, 3000);
};

// 模型选择逻辑
const embeddingModel = computed(() => store.settings.embedding_model || 'bge-large-zh');
const showEmbeddingConfirm = ref(false);
const pendingEmbeddingModel = ref('');

async function handleChatModelChange(v: string) {
  saving.value.chat_model = true;
  try {
    await store.saveSetting('chat_model', v);
    showToast('默认对话模型已更新');
  } catch (e) {
    console.error('保存失败:', e);
  } finally {
    saving.value.chat_model = false;
  }
}

async function handleEmbeddingModelChange(newModel: string) {
  if (newModel === embeddingModel.value) return;

  const documents = await getDocuments();
  const hasIndexedDocs = documents.some(d => d.status === 'indexed' || d.status === 'done');

  if (hasIndexedDocs) {
    pendingEmbeddingModel.value = newModel;
    showEmbeddingConfirm.value = true;
  } else {
    await saveEmbeddingModel(newModel);
  }
}

async function saveEmbeddingModel(model: string) {
  saving.value.embedding_model = true;
  try {
    await store.saveSetting('embedding_model', model);
    showToast('语义检索模型已更新');
  } catch (e) {
    console.error('保存失败:', e);
  } finally {
    saving.value.embedding_model = false;
  }
}

async function handleVisionModelChange(v: string) {
  saving.value.vision_model = true;
  try {
    await store.saveSetting('vision_model', v);
    showToast('视觉理解模型已更新');
    
    // Auto-set mmproj if binding exists
    const currentBindings = store.settings.vision_model_bindings ? JSON.parse(store.settings.vision_model_bindings) : {};
    if (currentBindings[v]) {
        await store.saveSetting('vision_mmproj', currentBindings[v]);
    }
  } catch (e) {
    console.error('保存失败:', e);
  } finally {
    saving.value.vision_model = false;
  }
}

async function handleVisionMmprojChange(v: string) {
  saving.value.vision_mmproj = true;
  try {
    await store.saveSetting('vision_mmproj', v);
    showToast('视觉投影器 (mmproj) 已更新');
  } catch (e) {
    console.error('保存失败:', e);
  } finally {
    saving.value.vision_mmproj = false;
  }
}

async function confirmEmbeddingChange() {
  showEmbeddingConfirm.value = false;
  reindexStore.startReindex();

  try {
    await saveEmbeddingModel(pendingEmbeddingModel.value);

    const result = await reindexAllDocuments();
    reindexStore.finishReindex(result);
    if (result.success) {
      showToast('嵌入模型已更新，所有文档已重新索引');
      notificationsStore.playDebouncedSound();
    } else {
      showToast(`索引完成但有${result.failed_count}个文档失败`);
    }
  } catch (e) {
    console.error('重新索引失败:', e);
    reindexStore.finishReindex({ success: false, total_documents: 0, indexed_count: 0, failed_count: 0, errors: [String(e)] });
    showToast('重新索引失败');
  }
}

// 高级参数逻辑
const topK = ref<number[]>([parseInt(store.settings.top_k || '5')]);
const similarityThreshold = ref<number[]>([parseFloat(store.settings.similarity_threshold || '0.3')]);

watch(topK, async (newVal) => {
  if (newVal[0] !== undefined) {
    saving.value.top_k = true;
    try {
      await store.saveSetting('top_k', String(newVal[0]));
    } catch (e) {
      console.error('保存失败:', e);
    } finally {
      saving.value.top_k = false;
    }
  }
});

watch(similarityThreshold, async (newVal) => {
  if (newVal[0] !== undefined) {
    saving.value.similarity_threshold = true;
    try {
      await store.saveSetting('similarity_threshold', newVal[0].toFixed(2));
    } catch (e) {
      console.error('保存失败:', e);
    } finally {
      saving.value.similarity_threshold = false;
    }
  }
});

const numThread = ref<number[]>([store.numThread]);
const numCtx = ref<number[]>([store.numCtx]);
const temperature = ref<number[]>([store.temperature]);

watch(numThread, (v) => { store.numThread = v[0]; });
watch(numCtx, (v) => { store.numCtx = v[0]; });
watch(temperature, (v) => { store.temperature = v[0]; });

import { invoke } from '@tauri-apps/api/core';

const isGpuSupported = ref(false);
const maxGpuLayers = ref(60);
const numGpu = ref<number[]>([parseInt(store.settings.num_gpu || '0')]);

onMounted(async () => {
  try {
    isGpuSupported.value = await invoke<boolean>('is_gpu_supported');
    try {
      const layers = await invoke<number>('get_current_model_max_layers');
      maxGpuLayers.value = layers;
      if (numGpu.value[0] > maxGpuLayers.value) {
        numGpu.value = [maxGpuLayers.value];
        await store.saveSetting('num_gpu', String(maxGpuLayers.value));
      }
    } catch (err) {
      maxGpuLayers.value = 60;
    }

    if (isGpuSupported.value && (!store.settings.num_gpu || store.settings.num_gpu === '0')) {
      const defaultGpuLayers = Math.min(24, maxGpuLayers.value);
      numGpu.value = [defaultGpuLayers];
      await store.saveSetting('num_gpu', String(defaultGpuLayers));
    }
  } catch (e) {
    isGpuSupported.value = false;
  }
});

watch(numGpu, async (v) => {
  if (v[0] !== undefined) {
    try {
      await store.saveSetting('num_gpu', String(v[0]));
    } catch (e) {
      console.error('保存失败:', e);
    }
  }
});

const isVisionConfigured = computed(() => {
  return !!store.settings.vision_model && !!store.settings.vision_mmproj;
});

const useVisionParser = computed(() => {
  return store.settings.use_vision_parser === 'true' && isVisionConfigured.value;
});

async function toggleVisionParser() {
  if (!isVisionConfigured.value) return;
  const targetState = !useVisionParser.value;
  try {
    await store.saveSetting('use_vision_parser', String(targetState));
    showToast(targetState ? '已开启文档视觉解析（解析速度会变慢）' : '已关闭文档视觉解析');
  } catch (e) {
    console.error('保存失败:', e);
  }
}

watch(isVisionConfigured, async (newVal) => {
  if (!newVal && store.settings.use_vision_parser === 'true') {
    try {
      await store.saveSetting('use_vision_parser', 'false');
    } catch (e) {
      console.error('自动关闭失败:', e);
    }
  }
});

</script>

<template>
  <div class="max-w-[1200px] mx-auto min-h-full px-3 pb-4">
    <!-- 头部区域 -->
    <div class="pb-4 border-b border-border-main/50 flex items-center justify-between mb-4">
      <div>
        <h2 class="font-black text-xl text-text-primary tracking-tight">模型管理与配置</h2>
        <p class="text-xs text-text-muted mt-1 leading-relaxed">
          在这里下载、评估和删除大语言模型与嵌入模型，并精细化配置对话生成与向量检索参数。
        </p>
      </div>
    </div>

    <!-- Toast 通知 -->
    <div 
      v-if="toast.visible" 
      class="fixed bottom-6 right-6 px-4 py-2 bg-panel-bg border border-border-main/50 text-text-primary text-xs rounded-lg shadow-xl z-50 animate-in fade-in slide-in-from-bottom-2 duration-300"
    >
      {{ toast.message }}
    </div>

    <div v-if="store.isLoading" class="flex justify-center items-center py-8">
      <div class="animate-spin rounded-full h-10 w-12 border-t-4 border-b-4 border-brand"></div>
    </div>

    <div v-else class="space-y-6">
      <!-- 页面主要 Tab 切换 -->
      <div class="flex gap-2 text-xs font-semibold flex-nowrap border-b border-border-main/50 pb-2">
        <button 
          @click="activeTab = 'manager'" 
          class="px-4 py-2 transition-all relative"
          :class="activeTab === 'manager' ? 'text-brand font-bold' : 'text-text-secondary hover:text-text-primary'"
        >
          模型市场与本地管理
          <div v-if="activeTab === 'manager'" class="absolute bottom-[-9px] left-0 w-full h-0.5 bg-brand"></div>
        </button>
        <button 
          @click="activeTab = 'advanced'" 
          class="px-4 py-2 transition-all relative"
          :class="activeTab === 'advanced' ? 'text-brand font-bold' : 'text-text-secondary hover:text-text-primary'"
        >
          高级推理与检索参数
          <div v-if="activeTab === 'advanced'" class="absolute bottom-[-9px] left-0 w-full h-0.5 bg-brand"></div>
        </button>
      </div>

      <!-- Tab 内容 A：模型市场与管理 -->
      <div v-show="activeTab === 'manager'" class="animate-in fade-in duration-300">
        <ModelManager 
          @select-chat-model="handleChatModelChange"
          @select-embedding-model="handleEmbeddingModelChange"
          @select-vision-model="handleVisionModelChange"
          @select-vision-mmproj="handleVisionMmprojChange"
        />
      </div>

      <!-- Tab 内容 B：高级推理与检索参数 -->
      <div v-show="activeTab === 'advanced'" class="grid grid-cols-1 md:grid-cols-2 gap-6 animate-in fade-in duration-300">
        <!-- 检索参数模块 -->
        <div class="bg-panel-bg border border-border-main/50 hover:border-border-main/80 rounded-xl p-5 space-y-6 transition-all duration-300 shadow-sm">
          <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
            <h3 class="text-sm font-bold text-text-primary">语义检索参数配置</h3>
          </div>

          <div>
            <label class="block text-xs font-bold text-text-primary mb-2">Top-K</label>
            <SliderRoot
              v-model="topK"
              :min="1"
              :max="20"
              :step="1"
              class="relative flex items-center w-full h-4"
            >
              <SliderTrack class="relative grow h-2 bg-surface-bg rounded-full">
                <SliderRange class="absolute h-full bg-brand rounded-full" />
              </SliderTrack>
              <SliderThumb class="block w-4 h-4 bg-white border-2 border-brand rounded-full shadow-md focus:outline-none cursor-pointer" />
            </SliderRoot>
            <div class="text-text-secondary font-medium mt-2 text-[11px]">返回最相关的 {{ topK[0] }} 个文档块</div>
          </div>

          <div>
            <label class="block text-xs font-bold text-text-primary mb-2">相似度阈值</label>
            <SliderRoot
              v-model="similarityThreshold"
              :min="0"
              :max="1"
              :step="0.05"
              class="relative flex items-center w-full h-4"
            >
              <SliderTrack class="relative grow h-2 bg-surface-bg rounded-full">
                <SliderRange class="absolute h-full bg-brand rounded-full" />
              </SliderTrack>
              <SliderThumb class="block w-4 h-4 bg-white border-2 border-brand rounded-full shadow-md focus:outline-none cursor-pointer" />
            </SliderRoot>
            <div class="text-text-secondary font-medium mt-2 text-[11px]">
              过滤相似度低于 {{ similarityThreshold[0].toFixed(2) }} 的检索结果
            </div>
          </div>
        </div>

        <!-- LLM 推理参数模块 -->
        <div class="bg-panel-bg border border-border-main/50 hover:border-border-main/80 rounded-xl p-5 space-y-6 transition-all duration-300 shadow-sm">
          <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line x1="8" y1="21" x2="16" y2="21"></line><line x1="12" y1="17" x2="12" y2="21"></line></svg>
            <h3 class="text-sm font-bold text-text-primary">LLM 推理高级参数</h3>
          </div>

          <!-- 线程数 -->
          <div>
            <div class="flex justify-between mb-2">
              <label class="block text-xs font-bold text-text-primary">推理线程数 (num_thread)</label>
              <span class="text-[11px] font-mono text-brand">{{ numThread[0] }}</span>
            </div>
            <SliderRoot v-model="numThread" :min="1" :max="12" :step="1" class="relative flex items-center w-full h-4">
              <SliderTrack class="relative grow h-2 bg-surface-bg rounded-full">
                <SliderRange class="absolute h-full bg-brand rounded-full" />
              </SliderTrack>
              <SliderThumb class="block w-4 h-4 bg-white border-2 border-brand rounded-full shadow-md cursor-pointer" />
            </SliderRoot>
            <p class="mt-2 text-[10px] text-text-muted">建议设置为您的物理核心数。</p>
          </div>

          <!-- 上下文窗口 -->
          <div>
            <div class="flex justify-between mb-2">
              <label class="block text-xs font-bold text-text-primary">上下文窗口 (num_ctx)</label>
              <span class="text-[11px] font-mono text-brand">{{ numCtx[0] }}</span>
            </div>
            <SliderRoot v-model="numCtx" :min="1024" :max="16384" :step="1024" class="relative flex items-center w-full h-4">
              <SliderTrack class="relative grow h-2 bg-surface-bg rounded-full">
                <SliderRange class="absolute h-full bg-brand rounded-full" />
              </SliderTrack>
              <SliderThumb class="block w-4 h-4 bg-white border-2 border-brand rounded-full shadow-md cursor-pointer" />
            </SliderRoot>
            <p class="mt-2 text-[10px] text-text-muted">推理时最大上下文长度，较大的值会占用更多内存。</p>
          </div>

          <!-- 温度 -->
          <div>
            <div class="flex justify-between mb-2">
              <label class="block text-xs font-bold text-text-primary">生成温度 (temperature)</label>
              <span class="text-[11px] font-mono text-brand">{{ temperature[0].toFixed(1) }}</span>
            </div>
            <SliderRoot v-model="temperature" :min="0" :max="1" :step="0.1" class="relative flex items-center w-full h-4">
              <SliderTrack class="relative grow h-2 bg-surface-bg rounded-full">
                <SliderRange class="absolute h-full bg-brand rounded-full" />
              </SliderTrack>
              <SliderThumb class="block w-4 h-4 bg-white border-2 border-brand rounded-full shadow-md cursor-pointer" />
            </SliderRoot>
            <p class="mt-2 text-[10px] text-text-muted">较低的温度使回答更稳定。较高的温度更具创造性。</p>
          </div>

          <!-- GPU 加载层数 -->
          <div :class="{'opacity-50 pointer-events-none select-none': !isGpuSupported}">
            <div class="flex justify-between mb-2">
              <div class="flex items-center gap-1.5">
                <label class="block text-xs font-bold text-text-primary">GPU 加载层数 (num_gpu)</label>
                <span v-if="!isGpuSupported" class="px-1.5 py-0.5 bg-surface-bg border border-border-main/40 rounded text-[9px] text-text-muted">不支持显卡</span>
              </div>
              <span class="text-[11px] font-mono text-brand">{{ numGpu[0] }}</span>
            </div>
            <SliderRoot v-model="numGpu" :min="0" :max="maxGpuLayers" :step="1" :disabled="!isGpuSupported" class="relative flex items-center w-full h-4">
              <SliderTrack class="relative grow h-2 bg-surface-bg rounded-full">
                <SliderRange class="absolute h-full bg-brand rounded-full" />
              </SliderTrack>
              <SliderThumb class="block w-4 h-4 bg-white border-2 border-brand rounded-full shadow-md cursor-pointer" />
            </SliderRoot>
            <p class="mt-2 text-[10px] text-text-muted">
              {{ isGpuSupported ? `将模型全部或部分层加载到显卡（Metal 加载）中可极速提升生成速度。当前使用的 ${store.settings.chat_model || '聊天'} 模型最多支持 ${maxGpuLayers} 层。` : '您的电脑硬件当前不支持 GPU 显卡加速，该选项已禁用。' }}
            </p>
          </div>
        </div>

        <!-- 文档多模态视觉解析配置卡片 -->
        <div class="bg-panel-bg border border-border-main/50 hover:border-border-main/80 rounded-xl p-5 space-y-4 transition-all duration-300 shadow-sm md:col-span-2">
          <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand">
              <path d="M15 12c0 1.657-1.343 3-3 3s-3-1.343-3-3 1.343-3 3-3 3 1.343 3 3z"/>
              <path d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
            </svg>
            <h3 class="text-sm font-bold text-text-primary">文档多模态视觉解析</h3>
          </div>

          <div class="flex justify-between items-start gap-4">
            <div class="space-y-1">
              <h4 class="text-xs font-bold text-text-primary">启用视觉模型解析文档</h4>
              <p class="text-[10px] text-text-muted leading-relaxed">
                开启后，导入 PNG/JPG/WebP/BMP 等图片格式文档时，将自动使用本地 Vision 模型生成内容描述；导入 PDF 时，若已安装高级解析插件，将对每一页生成图像并进行多模态文本与表格提取。
              </p>
            </div>
            
            <!-- 开关按钮 -->
            <button 
              @click="toggleVisionParser"
              :disabled="!isVisionConfigured"
              class="w-12 h-6 rounded-full p-0.5 transition-colors flex items-center shadow-inner relative cursor-pointer"
              :class="[
                useVisionParser ? 'bg-brand' : 'bg-surface-bg border border-border-main',
                !isVisionConfigured ? 'opacity-40 cursor-not-allowed' : ''
              ]"
            >
              <div 
                class="w-5 h-5 bg-white rounded-full shadow-md transition-all duration-200"
                :class="useVisionParser ? 'translate-x-6' : 'translate-x-0'"
              ></div>
            </button>
          </div>

          <!-- 提示与警示区域 -->
          <div v-if="!isVisionConfigured" class="p-2.5 bg-red-500/10 border border-red-500/20 rounded-lg flex items-start gap-2">
            <span class="text-xs">⚠️</span>
            <p class="text-[10px] text-red-500 font-medium leading-normal">
              请先在“模型市场与本地管理”中配置<strong>视觉模型</strong>与<strong>视觉投影器 (mmproj)</strong>，方可启用此功能。
            </p>
          </div>
          <div v-else class="p-2.5 bg-yellow-500/10 border border-yellow-500/20 rounded-lg flex items-start gap-2">
            <span class="text-xs">💡</span>
            <p class="text-[10px] text-yellow-600 dark:text-yellow-400 font-medium leading-normal">
              <strong>警告：</strong>使用视觉多模态解析大文档或多页 PDF 时，会显著降低解析/索引速度（每页可能耗时数秒至数十秒），并会占用较多 CPU/GPU 内存。
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- 嵌入模型修改确认对话框 -->
    <AlertDialogRoot v-model:open="showEmbeddingConfirm">
      <AlertDialogPortal>
        <AlertDialogOverlay class="fixed inset-0 bg-black/50 z-40" />
        <AlertDialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-panel-bg border border-border-main/50 rounded-md p-4 sm:max-w-md w-full mx-4 z-50 shadow-xl animate-in zoom-in-95 duration-200">
          <AlertDialogTitle class="text-base font-bold text-warning-500 mb-3">
            ⚠️ 嵌入模型变更警告
          </AlertDialogTitle>
          <AlertDialogDescription class="text-text-secondary mb-4 text-sm leading-relaxed">
            更换嵌入模型会导致所有已索引的文档向量失效。
            <br><br>
            系统将自动使用新的嵌入模型重新解析和索引所有文档，这个过程可能需要较长时间。
            <br><br>
            <strong>确定要继续吗？</strong>
          </AlertDialogDescription>
          <div class="flex justify-end gap-2">
            <AlertDialogCancel class="px-3 py-1.5 border-2 border-border-main-light hover:bg-surface-bg text-text-secondary rounded-md transition-colors">
              取消
            </AlertDialogCancel>
            <AlertDialogAction
              class="px-3 py-1.5 bg-warning-500 hover:bg-warning-600 text-white rounded-md transition-colors font-medium"
              @click="confirmEmbeddingChange"
            >
              确定更换并重新索引
            </AlertDialogAction>
          </div>
        </AlertDialogContent>
      </AlertDialogPortal>
    </AlertDialogRoot>
  </div>
</template>
