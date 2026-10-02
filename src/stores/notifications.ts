import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { getNotifications, getNotificationsPaginated, markNotificationRead as markReadApi } from '@/api/tauri';
import type { AppNotification } from '@/types/notification';

import { useDebounceFn } from '@vueuse/core';

export const useNotificationsStore = defineStore('notifications', () => {
  const notifications = ref<AppNotification[]>([]);
  const unreadCount = ref(0);
  const isLoading = ref(false);
  const page = ref(1);
  const hasMore = ref(true);

  let unlistenNotification: UnlistenFn | null = null;

  const playDebouncedSound = useDebounceFn(() => {
    playNotificationSound();
  }, 1000);

  function playNotificationSound() {
    try {
      import('./settings').then(({ useSettingsStore }) => {
        const settingsStore = useSettingsStore();
        if (settingsStore.settings.sound_enabled === 'false') {
          return;
        }
        
        const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
        if (ctx.state === 'suspended') {
          ctx.resume();
        }
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();

        osc.type = 'sine';
        osc.frequency.setValueAtTime(523.25, ctx.currentTime);
        osc.frequency.exponentialRampToValueAtTime(659.25, ctx.currentTime + 0.15);

        gain.gain.setValueAtTime(0.12, ctx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + 0.3);

        osc.connect(gain);
        gain.connect(ctx.destination);

        osc.start();
        osc.stop(ctx.currentTime + 0.3);
      }).catch(err => console.error('音频合成失败:', err));
    } catch (e) {
      console.error('播放通知声音失败:', e);
    }
  }

  async function initListener() {
    if (unlistenNotification) return;
    unlistenNotification = await listen<AppNotification>('notification-created', (event) => {
      const newNotification = event.payload;
      notifications.value.unshift(newNotification);
      if (!newNotification.isRead) {
        unreadCount.value++;
        playNotificationSound();
      }
    });
  }

  async function fetchNotifications() {
    isLoading.value = true;
    try {
      notifications.value = await getNotifications();
      unreadCount.value = notifications.value.filter((n) => !n.isRead).length;
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchNotificationsPaginated(reset: boolean = true) {
    if (isLoading.value) return;
    isLoading.value = true;
    try {
      if (reset) {
        page.value = 1;
        hasMore.value = true;
      }
      
      const result = await getNotificationsPaginated(page.value, 15);
      if (reset) {
        notifications.value = result.items;
      } else {
        notifications.value = [...notifications.value, ...result.items];
      }
      hasMore.value = result.hasMore;
      page.value++;
      
      // Calculate unread count (this might be better handled by a separate API or local filter)
      unreadCount.value = notifications.value.filter((n) => !n.isRead).length;
    } finally {
      isLoading.value = false;
    }
  }

  async function loadMore() {
    if (!hasMore.value || isLoading.value) return;
    await fetchNotificationsPaginated(false);
  }

  async function markNotificationRead(id: string) {
    await markReadApi(id);
    const notification = notifications.value.find((n) => n.id === id);
    if (notification && !notification.isRead) {
      notification.isRead = true;
      unreadCount.value = Math.max(0, unreadCount.value - 1);
    }
  }

  async function markAllAsRead() {
    for (const n of notifications.value.filter((n) => !n.isRead)) {
      await markReadApi(n.id);
      n.isRead = true;
    }
    unreadCount.value = 0;
  }

  return {
    notifications,
    unreadCount,
    isLoading,
    hasMore,
    initListener,
    fetchNotifications,
    fetchNotificationsPaginated,
    loadMore,
    markNotificationRead,
    markAllAsRead,
    playNotificationSound,
    playDebouncedSound,
  };
});
