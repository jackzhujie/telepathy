import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { 
  getSettings, 
  updateSetting, 
  getSystemInfo as getSystemInfoCmd, 
  cancelPullModel as cancelPullModelCmd, 
  searchModels as searchModelsCmd, 
  type SystemInfo 
} from '@/api/tauri';
import {
  listRegistryModels,
  listInstalledModels,
  getModelRecommendations,
  installModel as installModelApi,
  deleteModel as deleteModelApi,
  getModelHub,
} from '@/api/tauri';
import type { RegistryModel, InstalledModel, ModelRecommendation, ModelPullProgress, ModelHubResponse } from '@/types/models';

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Record<string, string>>({});
  const models = computed(() => installedModels.value.map(m => m.full_name));
  const isLoading = ref(false);
  
  const systemInfo = ref<SystemInfo | null>(null);
  const pullGlobalNotification = ref<{ message: string; type: 'success' | 'error' | 'info'; timestamp: number } | null>(null);

  const registryModels = ref<RegistryModel[]>([]);
  const installedModels = ref<InstalledModel[]>([]);
  const modelRecommendations = ref<ModelRecommendation[]>([]);
  const activeDownloads = ref<Record<string, ModelPullProgress>>({});
  
  const installingModel = computed(() => {
    const keys = Object.keys(activeDownloads.value);
    return keys.length > 0 ? keys[0] : null;
  });

  const installProgress = computed(() => {
    const keys = Object.keys(activeDownloads.value);
    return keys.length > 0 ? activeDownloads.value[keys[0]] : null;
  });

  const modelHubData = ref<ModelHubResponse | null>(null);
  const isSyncingHub = ref(false);
  const searchResultModels = ref<any[]>([]);
  const isSearching = ref(false);
  const searchTimestamp = ref(0);
  const lastSearchQuery = ref('');
  const bestMirrorPrefix = ref('');
  const isMirrorProbing = ref(false);
  const isAppUpdateReady = ref(false); // App update
  const appUpdateInfo = ref<any>(null);
  const isAppUpdateDownloading = ref(false);
  const appUpdateProgress = ref(0);
  const currentTheme = ref<'light' | 'dark' | 'system'>((localStorage.getItem('theme') as any) || 'dark');
  const currentBrandColor = ref('#6366F1');

  const chatModels = computed(() =>
    installedModels.value.filter(m => m.model_type === 'chat')
  );
  const embeddingModels = computed(() =>
    installedModels.value.filter(m => m.model_type === 'embedding')
  );
  const visionModels = computed(() =>
    installedModels.value.filter(m => m.model_type === 'vision')
  );

  async function fetchSettings() {
    isLoading.value = true;
    try {
      settings.value = await getSettings();
      // 加载并应用主题
      if (settings.value.theme) {
        currentTheme.value = settings.value.theme as any;
        localStorage.setItem('theme', currentTheme.value);
      }
      if (settings.value.brand_color) {
        currentBrandColor.value = settings.value.brand_color;
      }
      applyTheme();
      applyBrandColor();

      // 监听系统主题变化
      window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
        if (currentTheme.value === 'system') {
          applyTheme();
        }
      });
    } finally {
      isLoading.value = false;
    }
  }

  function applyTheme() {
    const root = document.documentElement;
    let effectiveTheme = currentTheme.value;
    
    if (effectiveTheme === 'system') {
      effectiveTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    
    if (effectiveTheme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }

  function applyBrandColor() {
    const root = document.documentElement;
    root.style.setProperty('--color-brand', currentBrandColor.value);
    
    // 计算 RGB 组件以支持 Tailwind 颜色透明度 (rgba(var(--brand-rgb), 0.5))
    const color = currentBrandColor.value;
    if (color.startsWith('#')) {
      const r = parseInt(color.slice(1, 3), 16);
      const g = parseInt(color.slice(3, 5), 16);
      const b = parseInt(color.slice(5, 7), 16);
      root.style.setProperty('--brand-rgb', `${r} ${g} ${b}`);
      root.style.setProperty('--color-brand-glow', `rgba(${r}, ${g}, ${b}, 0.15)`);
    }
  }

  async function setTheme(theme: 'light' | 'dark' | 'system') {
    currentTheme.value = theme;
    localStorage.setItem('theme', theme);
    await saveSetting('theme', theme);
    applyTheme();
  }

  async function setBrandColor(color: string) {
    currentBrandColor.value = color;
    await saveSetting('brand_color', color);
    applyBrandColor();
  }

  async function saveSetting(key: string, value: string) {
    await updateSetting(key, value);
    settings.value[key] = value;
  }

  async function fetchSystemInfo() {
    try {
      systemInfo.value = await getSystemInfoCmd();
    } catch (e: any) {
      console.error('[SystemInfo] Error:', e);
    }
  }

  async function relaunchApp() {
    const { relaunch } = await import('@tauri-apps/plugin-process');
    await relaunch();
  }

  async function fetchRegistryModels(modelType?: string) {
    try {
      registryModels.value = await listRegistryModels(modelType);
    } catch (e: any) {
      console.error('[Registry] Error:', e);
    }
  }

  async function fetchInstalledModels() {
    try {
      installedModels.value = await listInstalledModels();
    } catch (e: any) {
      console.error('[InstalledModels] Error:', e);
    }
  }

  async function fetchModelRecommendations() {
    try {
      modelRecommendations.value = await getModelRecommendations();
    } catch (e: any) {
      console.error('[Recommendations] Error:', e);
    }
  }

  async function fetchModelHub(forceRefresh: boolean = false) {
    isSyncingHub.value = true;
    try {
      modelHubData.value = await getModelHub(forceRefresh);
      console.log('[ModelHub] Synced', modelHubData.value);
    } catch (e: any) {
      console.error('[ModelHub] Error:', e);
    } finally {
      isSyncingHub.value = false;
    }
  }
  
  async function searchHubModels(query: string) {
    const trimmed = query?.trim();
    if (!trimmed || trimmed.length <= 1) return;
    
    if (trimmed === lastSearchQuery.value) {
      return;
    }

    const now = Date.now();
    searchTimestamp.value = now;
    lastSearchQuery.value = trimmed;
    isSearching.value = true;
    searchResultModels.value = [];
    
    try {
      const results = await searchModelsCmd(trimmed);
      if (searchTimestamp.value === now) {
        searchResultModels.value = results;
      }
    } catch (e: any) {
      console.error('[Search] Error:', e);
      if (searchTimestamp.value === now) {
        searchResultModels.value = [];
      }
    } finally {
      if (searchTimestamp.value === now) {
        isSearching.value = false;
      }
    }
  }

  async function installNewModel(modelName: string, variant: string) {
    const finalModelId = `${modelName}:${variant}`;
    
    // 初始化占位状态
    activeDownloads.value[finalModelId] = {
      model: finalModelId,
      status: 'Preparing...',
      percentage: 0,
      completed: 0,
      total: 0,
      stage: 'preparing'
    };

    try {
      await installModelApi(modelName, variant);
    } catch (e: any) {
      console.error('[Install] Error:', e);
      delete activeDownloads.value[finalModelId];
    }
  }

  async function removeModel(modelName: string) {
    try {
      await deleteModelApi(modelName);
      
      // Auto-delete bound mmproj if it exists
      const currentBindings = settings.value.vision_model_bindings ? JSON.parse(settings.value.vision_model_bindings) : {};
      const boundMmproj = currentBindings[modelName];
      if (boundMmproj) {
         try {
           await deleteModelApi(boundMmproj);
         } catch(e) {
           console.warn('[Delete] Failed to delete bound mmproj:', e);
         }
      }
      
      await fetchInstalledModels();
    } catch (e: any) {
      console.error('[Delete] Error:', e);
      throw e;
    }
  }

  async function cancelInstall(modelId?: string) {
    try {
      if (modelId) {
        delete activeDownloads.value[modelId];
        await cancelPullModelCmd(modelId);
      } else {
        const keys = Object.keys(activeDownloads.value);
        activeDownloads.value = {};
        for (const k of keys) {
          await cancelPullModelCmd(k);
        }
      }
    } catch (e) {
      console.error('[Cancel] Error:', e);
    }
  }

  const numThread = computed({
    get: () => parseInt(settings.value.num_thread || '6'),
    set: (v: number) => saveSetting('num_thread', v.toString())
  });

  const numCtx = computed({
    get: () => parseInt(settings.value.num_ctx || '4096'),
    set: (v: number) => saveSetting('num_ctx', v.toString())
  });

  const temperature = computed({
    get: () => parseFloat(settings.value.temperature || '0.7'),
    set: (v: number) => saveSetting('temperature', v.toFixed(1))
  });

  const autoUpdate = computed({
    get: () => settings.value.auto_update !== 'false', // 默认开启
    set: (v: boolean) => saveSetting('auto_update', String(v))
  });

  const enableOfficeParser = computed({
    get: () => settings.value.enable_office_parser === 'true',
    set: (v: boolean) => saveSetting('enable_office_parser', String(v))
  });

  const wizardCompleted = computed({
    get: () => settings.value.wizard_completed === 'true',
    set: (v: boolean) => saveSetting('wizard_completed', String(v))
  });

  const showSetupWizard = computed(() => {
    return !wizardCompleted.value && installedModels.value.length === 0;
  });

  async function downloadPrimaryPresets() {
    const recs = modelHubData.value?.primary_recommendations;
    if (!recs) return;

    if (recs.chat) {
      installNewModel(recs.chat.model.name, recs.chat.variant.tag).catch(console.error);
    }
    if (recs.embedding) {
      installNewModel(recs.embedding.model.name, recs.embedding.variant.tag).catch(console.error);
    }
    wizardCompleted.value = true;
  }

  function skipSetupWizard() {
    wizardCompleted.value = true;
  }

  // 注册全局多任务下载监听器
  listen<any>('model-pull-progress', (event) => {
    const { message, percentage, completed = 0, total = 0, stage, model } = event.payload;
    if (!model) return;
    
    if (stage === 'cancelled') {
      delete activeDownloads.value[model];
      return;
    }

    activeDownloads.value[model] = {
      model,
      status: message || 'Downloading...',
      percentage,
      completed,
      total,
      stage
    };
  });

  listen<{ model: string; error: string }>('model-pull-error', (event) => {
    const { model, error } = event.payload;
    delete activeDownloads.value[model];
    
    pullGlobalNotification.value = {
      message: `模型 ${model} 下载失败: ${error}`,
      type: 'error',
      timestamp: Date.now()
    };
    setTimeout(() => {
      pullGlobalNotification.value = null;
    }, 5000);
  });

  listen<{ model: string }>('model-pull-done', async (event) => {
    const { model } = event.payload;
    delete activeDownloads.value[model];
    
    pullGlobalNotification.value = {
      message: `模型 ${model} 下载并安装完成！`,
      type: 'success',
      timestamp: Date.now()
    };
    setTimeout(() => {
      pullGlobalNotification.value = null;
    }, 4000);

    await fetchInstalledModels();

    import('./notifications').then(({ useNotificationsStore }) => {
      const notificationsStore = useNotificationsStore();
      notificationsStore.playDebouncedSound();
    }).catch(err => console.error('播放下载声音失败:', err));
  });

  return {
    settings,
    models,
    isLoading,
    systemInfo,
    pullGlobalNotification,
    registryModels,
    installedModels,
    modelRecommendations,
    installingModel,
    installProgress,
    activeDownloads,
    chatModels,
    embeddingModels,
    visionModels,
    fetchRegistryModels,
    fetchInstalledModels,
    fetchModelRecommendations,
    fetchModelHub,
    installNewModel,
    cancelInstall,
    removeModel,
    numThread,
    numCtx,
    temperature,
    autoUpdate,
    enableOfficeParser,
    modelHubData,
    isSyncingHub,
    searchResultModels,
    isSearching,
    searchHubModels,
    lastSearchQuery,
    bestMirrorPrefix,
    isMirrorProbing,
    isAppUpdateReady,
    appUpdateInfo,
    isAppUpdateDownloading,
    appUpdateProgress,
    currentTheme,
    currentBrandColor,
    setTheme,
    setBrandColor,
    fetchSettings,
    saveSetting,
    fetchSystemInfo,
    relaunchApp,
    wizardCompleted,
    showSetupWizard,
    downloadPrimaryPresets,
    skipSetupWizard,
  };
});
