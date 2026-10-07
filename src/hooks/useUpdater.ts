import { ref, toRef } from 'vue';
import { useSettingsStore } from '@/stores/settings';

// Updater is disabled in the open-source build.
// This stub keeps the call sites (App.vue, Settings.vue) compiling
// without depending on @tauri-apps/plugin-updater.
// To re-enable: install @tauri-apps/plugin-updater, register the plugin
// in src-tauri/src/lib.rs, and restore the real implementation.

const isCheckingLocal = ref(false);

export function useUpdater() {
  const store = useSettingsStore();
  const error = ref<string | null>(null);

  const checkForUpdates = async (_silent = false) => {
    console.log('[updater] auto-update is disabled in this build');
    return null;
  };

  const installUpdate = async () => {
    console.log('[updater] auto-update is disabled in this build');
  };

  const relaunchApp = async () => {
    console.log('[updater] relaunch is disabled in this build');
  };

  return {
    isChecking: isCheckingLocal,
    updateInfo: toRef(store, 'appUpdateInfo'),
    downloadProgress: toRef(store, 'appUpdateProgress'),
    isDownloading: toRef(store, 'isAppUpdateDownloading'),
    isDownloaded: toRef(store, 'isAppUpdateReady'),
    error,
    checkForUpdates,
    installUpdate,
    relaunchApp
  };
}
