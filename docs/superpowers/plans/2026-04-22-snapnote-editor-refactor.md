# SnapNote 编辑器重构实施计划 (Milkdown Typora 模式)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 SnapNote 的编辑器从 md-editor-v3 切换到 Milkdown，实现 Typora 风格的即时渲染体验，并支持双栏切换。

**Architecture:** 
- 卸载旧编辑器依赖，安装 Milkdown 套件。
- 构建 `MilkdownEditor.vue` 包装组件。
- 重构 `SnapNote.vue` 业务逻辑，移除旧模式状态，添加新的视角切换逻辑。

**Tech Stack:** Vue 3, Milkdown, Tailwind CSS, Pinia

---

### Task 1: 依赖管理与环境准备

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 卸载旧依赖**
Run: `pnpm remove md-editor-v3`

- [ ] **Step 2: 安装 Milkdown 核心套件**
Run: `pnpm add @milkdown/core @milkdown/prose @milkdown/ctx @milkdown/transformer @milkdown/preset-commonmark @milkdown/preset-gfm @milkdown/vue @milkdown/plugin-listener @milkdown/plugin-history @milkdown/plugin-slash @milkdown/plugin-prism prismjs`

- [ ] **Step 3: 验证安装状态**
Run: `pnpm install`
Expected: 依赖安装成功，无冲突。

- [ ] **Step 4: Commit**
```bash
git add package.json pnpm-lock.yaml
git commit -m "chore: remove md-editor-v3 and install milkdown dependencies"
```

### Task 2: 构建 Milkdown 编辑器包装组件

**Files:**
- Create: `src/components/editor/MilkdownEditor.vue`

- [ ] **Step 1: 编写 MilkdownEditor 组件**
实现一个基础的 Milkdown 封装，支持内容初始化、变化监听和深色模式适配。

```vue
<script setup lang="ts">
import { Editor, rootCtx, defaultValueCtx } from '@milkdown/core';
import { nord } from '@milkdown/theme-nord'; // 暂时使用内置主题，后续可定制
import { commonmark } from '@milkdown/preset-commonmark';
import { gfm } from '@milkdown/preset-gfm';
import { Milkdown, useEditor } from '@milkdown/vue';
import { listener, listenerCtx } from '@milkdown/plugin-listener';
import { ref, watch } from 'vue';

const props = defineProps<{
  modelValue: string;
  readonly?: boolean;
}>();

const emit = defineEmits(['update:modelValue']);

const { get } = useEditor((root) =>
  Editor.make()
    .config((ctx) => {
      ctx.set(rootCtx, root);
      ctx.set(defaultValueCtx, props.modelValue);
      ctx.get(listenerCtx).markdownUpdated((ctx, markdown, prevMarkdown) => {
        if (markdown !== prevMarkdown) {
          emit('update:modelValue', markdown);
        }
      });
    })
    .config(nord)
    .use(commonmark)
    .use(gfm)
    .use(listener)
);
</script>

<template>
  <div class="milkdown-container h-full overflow-auto">
    <Milkdown />
  </div>
</template>

<style>
/* 覆盖 Milkdown 默认样式以匹配应用 UI */
.milkdown {
  @apply bg-transparent text-text-primary;
  max-width: none !important;
  box-shadow: none !important;
}
</style>
```

- [ ] **Step 2: Commit**
```bash
git add src/components/editor/MilkdownEditor.vue
git commit -m "feat: add MilkdownEditor wrapper component"
```

### Task 3: 重构 SnapNote.vue 核心逻辑

**Files:**
- Modify: `src/views/SnapNote.vue`

- [ ] **Step 1: 移除旧代码**
删除所有 `md-editor-v3` 的导入、样式导入以及相关的 `ref` (如 `htmlPreview`, `previewOnly`)。

- [ ] **Step 2: 引入 MilkdownEditor**
在 `SnapNote.vue` 中导入并使用 `MilkdownEditor`。

- [ ] **Step 3: 重构布局**
实现“沉浸模式”与“分栏模式”的条件渲染。

```html
<!-- 沉浸模式 -->
<div v-if="viewMode === 'zen'" class="flex-1 max-w-3xl mx-auto w-full">
  <MilkdownEditor v-model="newSnapContent" />
</div>

<!-- 分栏模式 -->
<div v-else class="flex-1 flex gap-4">
  <div class="flex-1 bg-surface-bg/30 rounded-xl p-4">
    <textarea v-model="newSnapContent" class="w-full h-full bg-transparent outline-none resize-none" />
  </div>
  <div class="flex-1 bg-surface-bg/30 rounded-xl p-4 overflow-auto">
    <MilkdownEditor :modelValue="newSnapContent" readonly />
  </div>
</div>
```

- [ ] **Step 4: Commit**
```bash
git add src/views/SnapNote.vue
git commit -m "refactor: replace old editor with milkdown and implement view modes"
```

### Task 5: 最终清理与验证

- [ ] **Step 1: 验证功能**
测试随笔的创建、保存、历史记录加载是否正常。

- [ ] **Step 2: 编译检查**
Run: `npm run build` (或者 `pnpm tauri build`)
Expected: 无类型错误，构建成功。

- [ ] **Step 3: Commit**
```bash
git commit -m "fix: final polish and verification of snapnote editor"
```
