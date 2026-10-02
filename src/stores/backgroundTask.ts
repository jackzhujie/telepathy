import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useDocumentsStore } from './documents';

export const useBackgroundTaskStore = defineStore('backgroundTask', () => {
  const isRunning = ref(false);
  const total = ref(0);
  const current = ref(0);
  const successCount = ref(0);
  const failedCount = ref(0);

  function startTask(count: number) {
    isRunning.value = true;
    total.value = count;
    current.value = 0;
    successCount.value = 0;
    failedCount.value = 0;
  }

  function incrementSuccess() {
    successCount.value++;
    current.value = successCount.value + failedCount.value;
  }

  function incrementFailed() {
    failedCount.value++;
    current.value = successCount.value + failedCount.value;
  }

  function completeTask() {
    isRunning.value = false;
  }

  function reset() {
    isRunning.value = false;
    total.value = 0;
    current.value = 0;
    successCount.value = 0;
    failedCount.value = 0;
  }

  const progress = computed(() => {
    if (total.value === 0) return 0;
    return ((successCount.value + failedCount.value) / total.value) * 100;
  });

  async function runBackgroundIndexing(docIds: string[]) {
    const documentsStore = useDocumentsStore();
    startTask(docIds.length);

    for (const docId of docIds) {
      try {
        await documentsStore.runIndex(docId);
        incrementSuccess();
      } catch (e) {
        console.error('后台索引失败:', e);
        incrementFailed();
      }
    }

    completeTask();
  }

  return {
    isRunning,
    total,
    current,
    successCount,
    failedCount,
    progress,
    startTask,
    incrementSuccess,
    incrementFailed,
    completeTask,
    reset,
    runBackgroundIndexing,
  };
});