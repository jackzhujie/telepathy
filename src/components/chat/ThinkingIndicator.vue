<script setup lang="ts">
</script>

<template>
  <div class="flex items-center gap-2 py-2 text-text-muted">
    <div class="flex gap-1.5 px-1 py-1">
      <span class="w-1.5 h-1.5 rounded-full bg-brand animate-big-bounce" style="animation-delay: 0ms"></span>
      <span class="w-1.5 h-1.5 rounded-full bg-brand animate-big-bounce" style="animation-delay: 150ms"></span>
      <span class="w-1.5 h-1.5 rounded-full bg-brand animate-big-bounce" style="animation-delay: 300ms"></span>
    </div>
    <span class="text-xs">正在思考...</span>
  </div>
</template>

<style scoped>
@keyframes big-bounce {
  0%, 100% {
    transform: translateY(0);
    animation-timing-function: cubic-bezier(0.8, 0, 1, 1);
  }
  50% {
    transform: translateY(-80%);
    animation-timing-function: cubic-bezier(0, 0, 0.2, 1);
  }
}
.animate-big-bounce {
  animation: big-bounce 0.8s infinite;
}
</style>
