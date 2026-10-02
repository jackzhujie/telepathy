<script setup lang="ts">
import { Milkdown, useEditor } from '@milkdown/vue';
import { Editor, rootCtx, defaultValueCtx, editorViewCtx, serializerCtx } from '@milkdown/core';
import { gfm } from '@milkdown/preset-gfm';
import { commonmark } from '@milkdown/preset-commonmark';
import { listener, listenerCtx } from '@milkdown/plugin-listener';
import { history } from '@milkdown/plugin-history';
import { prism } from '@milkdown/plugin-prism';
import { nord } from '@milkdown/theme-nord';
import { replaceAll } from '@milkdown/utils';
import { watch, ref } from 'vue';

const props = defineProps<{
  modelValue: string;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const isInternalUpdate = ref(false);
const isReady = ref(false);

const { get } = useEditor((root) => {
  console.log('Milkdown: Initializing editor...');
  return Editor.make()
    .config((ctx) => {
      ctx.set(rootCtx, root);
      ctx.set(defaultValueCtx, props.modelValue);
      
      const listener = ctx.get(listenerCtx);
      
      listener.markdownUpdated((_ctx, markdown, prevMarkdown) => {
        const trimmedMarkdown = markdown.trimEnd();
        if (markdown !== prevMarkdown && !isInternalUpdate.value) {
          isInternalUpdate.value = true;
          emit('update:modelValue', trimmedMarkdown);
          setTimeout(() => {
            isInternalUpdate.value = false;
          }, 50);
        }
      });

      // 当编辑器挂载完成时标记为就绪
      listener.mounted(() => {
        console.log('Milkdown: Editor mounted and ready.');
        isReady.value = true;
      });
    })
    .config(nord)
    .use(commonmark)
    .use(gfm)
    .use(listener)
    .use(history)
    .use(prism);
});

// 支持外部 modelValue 变化时同步到编辑器
watch([() => props.modelValue, isReady], ([newValue, ready]) => {
  if (!ready || isInternalUpdate.value) return;
  
  const editor = get();
  if (!editor) return;
  
  // 获取编辑器当前内容的 Markdown
  const currentMarkdown = editor.action((ctx) => {
    try {
      const view = ctx.get(editorViewCtx);
      const serializer = ctx.get(serializerCtx);
      return serializer(view.state.doc);
    } catch (e) {
      return null;
    }
  });

  // 只有当内容真正发生变化时（忽略末尾换行差异）才更新编辑器
  if (currentMarkdown !== null && newValue.trimEnd() !== currentMarkdown.trimEnd()) {
    isInternalUpdate.value = true;
    editor.action(replaceAll(newValue));
    // 稍微延迟重置状态，给编辑器留出处理时间
    setTimeout(() => {
      isInternalUpdate.value = false;
    }, 50);
  }
});
</script>

<template>
  <div class="milkdown-editor-wrapper prose prose-sm dark:prose-invert max-w-none">
    <Milkdown />
  </div>
</template>

<style>
/* Milkdown 基础样式调整 */
.milkdown {
  margin-left: auto;
  margin-right: auto;
  background: transparent !important;
  box-shadow: none !important;
}

.milkdown .editor {
  padding: 1rem 0 !important;
  outline: none !important;
  color: var(--text-primary, #ffffff) !important;
  min-height: 200px;
}

/* 强制重置 Milkdown 内部的所有文字颜色 */
.milkdown .editor *, 
.milkdown .editor *::placeholder {
  color: inherit !important;
}

/* 针对特定标签的样式加强 */
.milkdown .editor h1,
.milkdown .editor h2,
.milkdown .editor h3,
.milkdown .editor h4,
.milkdown .editor strong {
  color: var(--text-primary, #ffffff) !important;
  font-weight: 600 !important;
}

.milkdown .editor p,
.milkdown .editor li {
  color: var(--text-primary, #e2e8f0) !important;
  line-height: 1.6 !important;
}

.milkdown .editor blockquote {
  border-left: 4px solid var(--brand, #3b82f6) !important;
  background: rgba(255, 255, 255, 0.05) !important;
  padding: 0.5rem 1rem !important;
  margin: 1rem 0 !important;
}

.milkdown .editor code {
  background: rgba(255, 255, 255, 0.1) !important;
  padding: 0.2rem 0.4rem !important;
  border-radius: 4px !important;
  color: var(--brand, #60a5fa) !important;
}

/* 修复光标颜色 */
.milkdown .editor .ProseMirror-cursor {
  border-left: 2px solid var(--brand, #3b82f6) !important;
}

/* 沉浸式体验：隐藏 Milkdown 自带的一些 UI 元素，通过自定义实现 */
.milkdown .slash-menu {
  @apply bg-panel-bg border border-border-main/50 shadow-xl rounded-lg overflow-hidden;
}
</style>
