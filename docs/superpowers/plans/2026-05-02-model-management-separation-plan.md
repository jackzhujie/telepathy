# 模型管理功能独立化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将模型下载管理、核心模型配置、高级推理配置等功能整合到一个全新的顶级页面“模型管理 (Models)”。

**Architecture:** 创建顶级页面组件 `Models.vue`，集成原 `ModelManager.vue` 及高级推理/检索配置参数；精简 `Settings.vue` 仅保留外观与应用信息；添加路由及边栏顶级入口。

**Tech Stack:** Vue 3.5, TypeScript, Vite, reka-ui, Tailwind CSS

---

### Task 1: 导航与路由配置

**Files:**
- Modify: `src/router/index.ts`
- Modify: `src/components/layout/Sidebar.vue`

- [ ] **Step 1: 在路由表中新增 `/models` 路由**
在 `src/router/index.ts` 的 `routes` 数组中添加 `/models`：
```typescript
  { path: '/models', name: 'Models', component: () => import('@/views/Models.vue'), meta: { keepAlive: true } },
```

- [ ] **Step 2: 在侧边栏新增“模型管理”顶级导航入口**
修改 `src/components/layout/Sidebar.vue` 的 `menuItems` 数组，将“模型管理”添加到“AI 对话”下方：
```typescript
const menuItems = [
  { label: '知识库', key: 'KnowledgeBase', icon: 'book' },
  { label: 'AI 对话', key: 'Chat', icon: 'chat' },
  { label: '模型管理', key: 'Models', icon: 'cpu' },
  { label: '文档管理', key: 'Documents', icon: 'document' },
  { label: '个人随记', key: 'SnapNote', icon: 'zap' },
  { label: '用户信息', key: 'Profile', icon: 'user' }
];
```

- [ ] **Step 3: 添加 `cpu` 图标 SVG 的渲染条件**
在 `Sidebar.vue` 中 `svg v-else-if="item.icon === 'zap'"` 旁增加：
```html
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
```

- [ ] **Step 4: 提交 Task 1 变更**
```bash
git add src/router/index.ts src/components/layout/Sidebar.vue
git commit -m "feat: add top-level navigation entry for model management"
```

---

### Task 2: 构建新顶级页面 `Models.vue`

**Files:**
- Create: `src/views/Models.vue`

- [ ] **Step 1: 新建 `src/views/Models.vue`**
将原 `ModelManager.vue` 完整的功能（为您推荐、全量探索、本地模型列表、排行榜）与高级推理参数（Top-K、相似度、线程数、上下文、温度、对话/检索模型选择）合并，提供完美、高度专业的一体化页面。
具体代码：
```vue
<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
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

async function confirmEmbeddingChange() {
  showEmbeddingConfirm.value = false;
  reindexStore.startReindex();

  try {
    await saveEmbeddingModel(pendingEmbeddingModel.value);

    const result = await reindexAllDocuments();
    reindexStore.finishReindex(result);
    if (result.success) {
      showToast('嵌入模型已更新，所有文档已重新索引');
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
      <div v-if="activeTab === 'manager'" class="animate-in fade-in duration-300">
        <ModelManager 
          @select-chat-model="handleChatModelChange"
          @select-embedding-model="handleEmbeddingModelChange"
        />
      </div>

      <!-- Tab 内容 B：高级推理与检索参数 -->
      <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-6 animate-in fade-in duration-300">
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
```

- [ ] **Step 2: 提交 Task 2 变更**
```bash
git add src/views/Models.vue
git commit -m "feat: implement brand new top-level Models.vue page"
```

---

### Task 3: 精简 `Settings.vue`

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: 重写 `Settings.vue` 保留纯粹的外观和应用信息**
修改 `src/views/Settings.vue` 内容，完全去处原有的 Model 相关配置。
完整覆盖代码：
```vue
<script setup lang="ts">
defineOptions({ name: 'Settings' });
import { ref, watch } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import {
  SliderRange,
  SliderRoot,
  SliderThumb,
  SliderTrack,
} from 'reka-ui';

const store = useSettingsStore();

const presetColors = [
  { name: '经典蓝', value: '#6366F1' },
  { name: '极光绿', value: '#10B981' },
  { name: '胭脂红', value: '#F43F5E' },
  { name: '深海紫', value: '#8B5CF6' },
  { name: '芒果黄', value: '#F59E0B' },
  { name: '石墨灰', value: '#4B5563' },
];
</script>

<template>
  <div class="max-w-[720px] mx-auto min-h-full px-3 pb-4">
    <div class="pb-4 border-b border-border-main/50 mb-6">
      <h2 class="font-black text-xl text-text-primary tracking-tight">系统设置</h2>
    </div>

    <div v-if="store.isLoading" class="flex justify-center items-center py-8">
      <div class="animate-spin rounded-full h-10 w-12 border-t-4 border-b-4 border-brand"></div>
    </div>

    <div v-else class="space-y-6 animate-in fade-in duration-300">
      <!-- 外观设置卡片 -->
      <div class="border border-border-main/50 rounded-xl bg-panel-bg p-5 hover:border-border-main/80 transition-all duration-300 shadow-sm space-y-6">
        <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand">
            <path d="M12 21a9 9 0 1 1 0-18c4.97 0 9 4.03 9 9 0 4.97-4.03 9-9 9Z"/><path d="M12 3v18"/><path d="M12 7.5a4.5 4.5 0 0 0 0 9"/>
          </svg>
          <h3 class="text-sm font-bold text-text-primary">界面外观设置</h3>
        </div>

        <!-- 主题切换 -->
        <div>
          <label class="block text-xs font-bold text-text-primary mb-3 text-muted-foreground uppercase tracking-widest">显示主题</label>
          <div class="grid grid-cols-3 gap-3">
            <button 
              v-for="t in ['light', 'dark', 'system']" 
              :key="t"
              @click="store.setTheme(t as any)"
              class="flex flex-col items-center gap-2 p-3 rounded-lg border transition-all duration-200"
              :class="store.currentTheme === t ? 'border-brand bg-brand/5' : 'border-border-main hover:border-brand/30 bg-surface-bg/50'"
            >
              <div class="p-2 rounded-full" :class="store.currentTheme === t ? 'bg-brand text-white' : 'bg-surface-bg text-text-secondary'">
                <svg v-if="t === 'light'" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                </svg>
                <svg v-else-if="t === 'dark'" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                </svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>
                </svg>
              </div>
              <span class="text-xs font-bold capitalize">{{ t === 'light' ? '浅色' : t === 'dark' ? '深色' : '系统' }}</span>
            </button>
          </div>
        </div>

        <!-- 主题色配置 -->
        <div>
          <label class="block text-xs font-bold text-text-primary mb-3 text-muted-foreground uppercase tracking-widest">品牌主题色</label>
          <div class="flex flex-wrap gap-3">
            <button 
              v-for="color in presetColors" 
              :key="color.value"
              @click="store.setBrandColor(color.value)"
              class="w-10 h-10 rounded-full border-4 transition-all duration-200 shadow-sm relative group"
              :style="{ backgroundColor: color.value }"
              :class="store.currentBrandColor === color.value ? 'border-text-primary scale-110 shadow-md' : 'border-transparent hover:scale-105'"
              :title="color.name"
            >
              <div v-if="store.currentBrandColor === color.value" class="absolute inset-0 flex items-center justify-center text-white">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"></polyline></svg>
              </div>
            </button>
            
            <!-- 自定义颜色 -->
            <div class="relative group">
              <input 
                type="color" 
                :value="store.currentBrandColor"
                @input="(e) => store.setBrandColor((e.target as HTMLInputElement).value)"
                class="w-10 h-10 rounded-full border-4 border-transparent appearance-none bg-transparent cursor-pointer hover:scale-105 transition-transform"
              />
              <div class="absolute -top-1 -right-1 w-4 h-4 bg-surface-bg border border-border-main rounded-full flex items-center justify-center pointer-events-none">
                <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 应用信息卡片 -->
      <div class="border border-border-main/50 rounded-xl bg-panel-bg p-5 hover:border-border-main/80 transition-all duration-300 shadow-sm space-y-4">
        <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand">
            <circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line>
          </svg>
          <h3 class="text-sm font-bold text-text-primary">应用信息</h3>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-text-secondary font-semibold text-xs">软件版本</span>
          <span class="text-text-primary font-bold text-xs">v0.1.0</span>
        </div>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 提交 Task 3 变更**
```bash
git add src/views/Settings.vue
git commit -m "feat: simplify and polish Settings.vue"
```

---

### Task 4: 编译与测试验证

- [ ] **Step 1: 运行类型检查**
Run: `pnpm tsc --noEmit` 或项目自带的 TypeScript 校验命令，确保没有任何类型错误。

- [ ] **Step 2: 提交 Task 4 变更**
```bash
git commit --allow-empty -m "chore: verify and confirm changes pass type check"
```
