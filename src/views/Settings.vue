<script setup lang="ts">
defineOptions({ name: 'Settings' });
import { computed, onMounted, ref } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { getVersion } from '@tauri-apps/api/app';
import { useUpdater } from '@/hooks/useUpdater';

const store = useSettingsStore();
const appVersion = ref('');
const { checkForUpdates, isChecking } = useUpdater();

onMounted(async () => {
  try {
    appVersion.value = await getVersion();
  } catch (e) {
    appVersion.value = '0.1.0';
  }
});

const isSoundEnabled = computed(() => store.settings.sound_enabled !== 'false');

async function toggleSound() {
  await store.saveSetting('sound_enabled', String(!isSoundEnabled.value));
}

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

      <!-- 通知音效设置卡片 -->
      <div class="border border-border-main/50 rounded-xl bg-panel-bg p-5 hover:border-border-main/80 transition-all duration-300 shadow-sm space-y-4">
        <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand">
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/>
            <path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>
          </svg>
          <h3 class="text-sm font-bold text-text-primary">通知音效设置</h3>
        </div>
        <div class="flex justify-between items-center">
          <div>
            <h4 class="text-xs font-bold text-text-primary">核心操作声音提示</h4>
            <p class="text-[10px] text-text-muted mt-1">在回答完毕、索引完成、模型下载完成时播放声音</p>
          </div>
          <button 
            @click="toggleSound"
            class="w-12 h-6 rounded-full p-0.5 transition-colors flex items-center shadow-inner relative cursor-pointer"
            :class="isSoundEnabled ? 'bg-brand' : 'bg-surface-bg border border-border-main'"
          >
            <div 
              class="w-5 h-5 bg-white rounded-full shadow-md transition-all duration-200 flex items-center justify-center text-[10px]"
              :class="isSoundEnabled ? 'translate-x-6' : 'translate-x-0'"
            >
              {{ isSoundEnabled ? '🔔' : '🔕' }}
            </div>
          </button>
        </div>
      </div>

      <!-- 高级功能扩展卡片 -->
      <div class="border border-border-main/50 rounded-xl bg-panel-bg p-5 hover:border-border-main/80 transition-all duration-300 shadow-sm space-y-4">
        <div class="flex items-center gap-2 border-b border-border-main/40 pb-2 mb-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-brand">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/>
          </svg>
          <h3 class="text-sm font-bold text-text-primary">高级功能扩展</h3>
        </div>
        <div class="flex justify-between items-center">
          <div>
            <h4 class="text-xs font-bold text-text-primary">高级 Office 文档解析扩展</h4>
            <p class="text-[10px] text-text-muted mt-1">开启后支持原生解析 Word (.docx)、Excel (.xlsx/.xls)、PPT (.pptx) 等格式，目前全功能免费使用</p>
          </div>
          <button 
            @click="store.enableOfficeParser = !store.enableOfficeParser"
            class="w-12 h-6 rounded-full p-0.5 transition-colors flex items-center shadow-inner relative cursor-pointer"
            :class="store.enableOfficeParser ? 'bg-brand' : 'bg-surface-bg border border-border-main'"
          >
            <div 
              class="w-5 h-5 bg-white rounded-full shadow-md transition-all duration-200"
              :class="store.enableOfficeParser ? 'translate-x-6' : 'translate-x-0'"
            ></div>
          </button>
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
          <div class="flex flex-col gap-1">
            <div class="flex items-center gap-2">
              <span class="text-text-secondary font-semibold text-xs">软件版本</span>
              <span class="text-text-primary font-bold text-xs">v{{ appVersion }}</span>
            </div>
            <div v-if="store.appUpdateInfo" class="flex items-center gap-1.5 animate-pulse">
              <div class="w-1.5 h-1.5 bg-red-500 rounded-full"></div>
              <span class="text-[10px] text-red-500 font-bold">发现新版本 v{{ store.appUpdateInfo.version }}</span>
            </div>
          </div>
          <button 
            @click="checkForUpdates()"
            class="px-4 py-1.5 bg-surface-bg hover:bg-border-main text-text-primary text-xs font-bold rounded-lg transition-all border border-border-main flex items-center gap-2"
            :disabled="isChecking"
          >
            <div v-if="isChecking" class="animate-spin rounded-full h-3 w-3 border-2 border-brand border-t-transparent"></div>
            {{ isChecking ? '正在检查...' : '检查更新' }}
          </button>
        </div>
        
        <!-- 自动更新开关 -->
        <div class="flex justify-between items-center pt-2 border-t border-border-main/30">
          <div>
            <h4 class="text-xs font-bold text-text-primary">自动下载更新</h4>
            <p class="text-[10px] text-text-muted mt-1">发现新版本时自动在后台下载，准备就绪后将在下次启动时自动应用</p>
          </div>
          <button 
            @click="store.autoUpdate = !store.autoUpdate"
            class="w-10 h-5 rounded-full p-0.5 transition-colors flex items-center shadow-inner relative cursor-pointer"
            :class="store.autoUpdate ? 'bg-brand' : 'bg-surface-bg border border-border-main'"
          >
            <div 
              class="w-4 h-4 bg-white rounded-full shadow-md transition-all duration-200"
              :class="store.autoUpdate ? 'translate-x-5' : 'translate-x-0'"
            ></div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
