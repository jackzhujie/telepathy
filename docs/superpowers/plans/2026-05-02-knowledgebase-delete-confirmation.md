# 知识库文档删除弹窗重构 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 `KnowledgeBase.vue` 的原生 `div` 弹窗重构为 `reka-ui` 组件弹框。

**Tech Stack:** Vue 3.5 + TypeScript + Vite + reka-ui

---

### Task 1: 引入 reka-ui 组件并替换原生弹框模板

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 引入 `reka-ui` 组件**

  在 `src/views/KnowledgeBase.vue` 的头部引入弹窗所需的组件：
  ```ts
  import {
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogOverlay,
    DialogPortal,
    DialogRoot,
    DialogTitle,
  } from 'reka-ui';
  ```

- [ ] **Step 2: 替换原生删除确认弹框模板**

  将原有的弹窗结构：
  ```html
  <div v-if="showDeleteModal" class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center animate-fade-in">
    ...
  </div>
  ```
  替换为基于 `reka-ui` 组件库的新结构：
  ```html
      <!-- 知识库文档删除确认模态框 -->
      <DialogRoot v-model:open="showDeleteModal">
        <DialogPortal>
          <DialogOverlay class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 animate-fade-in" />
          <DialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-panel-bg border border-border-main/60 p-6 rounded-2xl max-w-sm w-full shadow-2xl flex flex-col gap-4 z-50 animate-fade-in select-none">
            <div class="flex flex-col gap-1.5">
              <DialogTitle class="font-bold text-text-primary text-base">确定要删除此文档吗？</DialogTitle>
              <DialogDescription class="text-text-muted text-xs select-text">此操作不可恢复，该文档的所有向量索引和分片都将被永久删除。</DialogDescription>
            </div>
            <div class="flex justify-end gap-3 mt-2">
              <DialogClose class="px-4 py-2 border border-border-main/50 text-text-secondary hover:bg-surface-bg rounded-xl font-bold text-xs transition-colors">
                取消
              </DialogClose>
              <button 
                class="px-4 py-2 bg-danger-500 hover:bg-danger-600 text-white rounded-xl font-bold text-xs transition-colors shadow-md shadow-danger-500/20 active:scale-98"
                @click="confirmDeleteDoc"
              >
                确定删除
              </button>
            </div>
          </DialogContent>
        </DialogPortal>
      </DialogRoot>
  ```

- [ ] **Step 3: 使用 `tsc` 验证重构后前端编译是否正常**

  Run: `pnpm tsc --noEmit`
