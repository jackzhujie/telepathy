# 2026-05-02 知识库文档删除弹窗重构设计文档

## 1. 概述
在 `KnowledgeBase.vue` 页面中，删除文档确认弹框目前是使用纯 Tailwind CSS + `v-if` 手写实现的。为了统一项目弹框技术栈，提高交互专业度（如支持 ESC 退出、Focus Trap），计划将其重构为基于 `reka-ui` 组件库的模式。

## 2. 解决方案：使用 reka-ui 的 DialogRoot 组件

### 涉及文件：
- `src/views/KnowledgeBase.vue`

### 脚本修改计划：
1. 从 `reka-ui` 中导入以下弹窗组件：
   ```typescript
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

### 模板修改计划：
将第 440 到 466 行的原生弹窗 `div` 代码替换为 `reka-ui` 的组件结构：

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

## 3. 验收标准
- 使用 `pnpm tsc --noEmit` 检查无异常。
- 新模态框完全重现原模态框的视觉风格。
- 支持点击“取消”关闭，支持按 ESC 键自动关闭。
