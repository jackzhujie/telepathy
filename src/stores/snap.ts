import { defineStore } from 'pinia';
import { ref } from 'vue';
import { getSnapsPaginated, createSnap as createSnapApi, updateSnap as updateSnapApi, deleteSnap as deleteSnapApi } from '@/api/tauri';
import type { Snap } from '@/types/snap';

export const useSnapStore = defineStore('snap', () => {
  const snaps = ref<Snap[]>([]);
  const total = ref(0);
  const page = ref(1);
  const pageSize = ref(20);
  const isLoading = ref(false);
  const hasMore = ref(true);

  async function fetchSnaps(reset = false) {
    if (isLoading.value) return;
    if (reset) {
      page.value = 1;
      snaps.value = [];
      hasMore.value = true;
    }
    if (!hasMore.value) return;

    isLoading.value = true;
    try {
      const result = await getSnapsPaginated(page.value, pageSize.value);
      snaps.value = reset ? result.items : [...snaps.value, ...result.items];
      total.value = result.total;
      hasMore.value = snaps.value.length < total.value;
      if (hasMore.value) {
        page.value++;
      }
    } catch (e) {
      console.error('Failed to fetch snaps:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function createSnap(content: string, tags?: string) {
    try {
      const newSnap = await createSnapApi(content, tags);
      snaps.value.unshift(newSnap);
      total.value++;
      return newSnap;
    } catch (e) {
      console.error('Failed to create snap:', e);
      throw e;
    }
  }

  async function updateSnap(id: string, content: string, tags?: string, isPinned?: boolean) {
    try {
      const updated = await updateSnapApi(id, content, tags, isPinned);
      const index = snaps.value.findIndex(s => s.id === id);
      if (index !== -1) {
        // We only update the fields that changed, keeping the original createdAt
        const original = snaps.value[index];
        snaps.value[index] = {
          ...original,
          content: updated.content,
          tags: updated.tags,
          isPinned: updated.isPinned,
          updatedAt: updated.updatedAt,
        };
      }
      return updated;
    } catch (e) {
      console.error('Failed to update snap:', e);
      throw e;
    }
  }

  async function deleteSnap(id: string) {
    try {
      await deleteSnapApi(id);
      snaps.value = snaps.value.filter(s => s.id !== id);
      total.value--;
    } catch (e) {
      console.error('Failed to delete snap:', e);
      throw e;
    }
  }

  return {
    snaps,
    total,
    isLoading,
    hasMore,
    fetchSnaps,
    createSnap,
    updateSnap,
    deleteSnap,
  };
});
