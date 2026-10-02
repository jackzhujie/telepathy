import { defineStore } from 'pinia';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ReindexProgress, ReindexResult } from '@/api/tauri';

export const useReindexStore = defineStore('reindex', {
  state: () => ({
    isReindexing: false,
    progress: null as ReindexProgress | null,
    lastResult: null as ReindexResult | null,
    _unlisten: null as UnlistenFn | null,
  }),
  actions: {
    async initListener() {
      if (this._unlisten) return;
      this._unlisten = await listen<ReindexProgress>('reindex-progress', (event) => {
        this.progress = event.payload;
      });
    },
    startReindex() {
      this.isReindexing = true;
      this.progress = null;
      this.lastResult = null;
    },
    finishReindex(result: ReindexResult) {
      this.isReindexing = false;
      this.lastResult = result;
    },
    cleanup() {
      if (this._unlisten) {
        this._unlisten();
        this._unlisten = null;
      }
    },
  },
});