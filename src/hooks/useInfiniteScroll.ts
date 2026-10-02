import { ref, onMounted, onUnmounted, watch } from 'vue';

export interface UseInfiniteScrollOptions {
  loadMore: () => Promise<void>;
  isLoading: () => boolean;
  hasMore: () => boolean;
  rootMargin?: string;
}

export function useInfiniteScroll(options: UseInfiniteScrollOptions) {
  const { loadMore, isLoading, hasMore, rootMargin = '100px' } = options;
  const target = ref<HTMLElement | null>(null);
  let observer: IntersectionObserver | null = null;

  const handleIntersect = async (entries: IntersectionObserverEntry[]) => {
    const entry = entries[0];
    if (entry.isIntersecting && !isLoading() && hasMore()) {
      await loadMore();
    }
  };

  onMounted(() => {
    if (target.value) {
      observer = new IntersectionObserver(handleIntersect, {
        root: null,
        rootMargin,
        threshold: 0.1,
      });
      observer.observe(target.value);
    }
  });

  onUnmounted(() => {
    if (observer && target.value) {
      observer.unobserve(target.value);
      observer.disconnect();
    }
  });

  // 当目标元素变化时重新观察
  watch(target, (newTarget, oldTarget) => {
    if (observer) {
      if (oldTarget) {
        observer.unobserve(oldTarget);
      }
      if (newTarget) {
        observer.observe(newTarget);
      }
    }
  });

  return { target };
}
