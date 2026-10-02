<script setup lang="ts">
import AppLayout from '@/components/layout/AppLayout.vue';
import SetupWizard from '@/components/settings/SetupWizard.vue';
import { useNotificationsStore } from '@/stores/notifications';
import { useSettingsStore } from '@/stores/settings';
import { useUpdater } from '@/hooks/useUpdater';
import { onMounted, onUnmounted, watch } from 'vue';
import { TooltipProvider } from 'reka-ui';

const notificationsStore = useNotificationsStore();
const settingsStore = useSettingsStore();
const { checkForUpdates, updateInfo, isDownloading, isDownloaded, installUpdate } = useUpdater();

// 自动更新触发逻辑
watch(updateInfo, (val) => { 
  if (val && settingsStore.autoUpdate && !isDownloading.value && !isDownloaded.value) {
    installUpdate();
  }
});

let updateInterval: any = null;

onMounted(async () => {
  await notificationsStore.initListener();
  await notificationsStore.fetchNotifications();
  await settingsStore.fetchSettings();
  await settingsStore.fetchInstalledModels();

  // 启动时检查更新 (静默)
  checkForUpdates(true);

  // 每 30 分钟 (1800000 毫秒) 自动静默检测更新一次
  updateInterval = setInterval(() => {
    checkForUpdates(true);
  }, 1800000);

  // 监听首次用户交互来解锁音频限制
  const unlockAudio = () => {
    try {
      const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
      if (ctx.state === 'suspended') {
        ctx.resume();
      }
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      gain.gain.setValueAtTime(0, ctx.currentTime);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start();
      osc.stop(ctx.currentTime + 0.1);
      window.removeEventListener('click', unlockAudio);
    } catch (e) {
      console.error('解锁音频失败:', e);
    }
  };
  window.addEventListener('click', unlockAudio, { once: true });
});

onUnmounted(() => {
  if (updateInterval) {
    clearInterval(updateInterval);
  }
});
</script>

<template>
  <TooltipProvider :delay-duration="300">
    <div class="h-screen relative">
      <AppLayout />
      <SetupWizard v-if="settingsStore.showSetupWizard" />
    </div>
  </TooltipProvider>
</template>
