# 文档删除增加二次确认 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为“文档管理”页面的文档删除功能添加基于 `reka-ui` 的二次确认模态框。

**Architecture:** 
1. 在 `Documents.vue` 中定义 `showDeleteDocConfirm` 和 `docToDelete` 状态。
2. 重构删除逻辑：点击删除按钮时不直接触发删除，而是先打开弹窗。
3. 新增 `DialogRoot` 模态框，用户在其中确认后才真正调用后端删除。

**Tech Stack:** Vue 3.5 + TypeScript + Vite + reka-ui

---

### Task 1: 增加确认状态和处理逻辑，并替换原有删除按钮的点击事件

**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 新增响应式变量和方法**

  在 `Documents.vue` 中 `defineOptions` 后面，引入 `Document` 类型，并在合适的位置添加 `showDeleteDocConfirm` 和 `docToDelete`：
  ```ts
  import type { Document } from '@/types/document';

  const showDeleteDocConfirm = ref(false);
  const docToDelete = ref<Document | null>(null);

  function confirmDeleteDoc(doc: Document) {
    docToDelete.value = doc;
    showDeleteDocConfirm.value = true;
  }

  async function handleDeleteDoc() {
    if (!docToDelete.value) return;
    try {
      await documentsStore.removeDocument(docToDelete.value.id);
      showToast('已删除');
      showDeleteDocConfirm.value = false;
      docToDelete.value = null;
    } catch (e) {
      showToast(`删除失败: ${e}`);
    }
  }
  ```

- [ ] **Step 2: 替换文档列表中的“删除”按钮事件**

  在模板虚拟滚动的 `virtualItem` 渲染处，找到原来的删除按钮：
  ```html
  <button
    class="px-2.5 py-1.5 border border-danger-500/40 text-danger-500 bg-danger-500/5 hover:bg-danger-500/15 rounded-xl text-xs font-bold transition-all shadow-sm"
    @click="handleDelete(documentsRef[virtualItem.index].id)"
  >
    删除
  </button>
  ```
  将其修改为：
  ```html
  <button
    class="px-2.5 py-1.5 border border-danger-500/40 text-danger-500 bg-danger-500/5 hover:bg-danger-500/15 rounded-xl text-xs font-bold transition-all shadow-sm select-none"
    @click="confirmDeleteDoc(documentsRef[virtualItem.index])"
  >
    删除
  </button>
  ```

- [ ] **Step 3: 添加文档删除确认模态框的模板代码**

  在页面底部的 `<!-- 删除确认模态框 -->`（删除项目模态框）上方或下方插入新的模态框：

  ```html
      <!-- 文档删除确认模态框 -->
      <DialogRoot v-model:open="showDeleteDocConfirm">
        <DialogPortal>
          <DialogOverlay class="fixed inset-0 bg-black/40 backdrop-blur-sm z-40 animate-fade-in" />
          <DialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-panel-bg/65 backdrop-blur-md border border-border-main/35 rounded-2xl p-6 sm:max-w-lg w-full mx-4 z-50 shadow-2xl animate-fade-in select-none">
            <DialogTitle class="text-base font-black text-danger-500 mb-2 tracking-tight">确认删除</DialogTitle>
            <DialogDescription class="text-text-secondary mb-5 text-xs select-text">确定要删除文档"{{ docToDelete?.name }}"吗？此操作不可逆，该文档的所有向量索引和分片都将被永久删除。</DialogDescription>
            <Separator class="mb-4 pt-3 border-t border-border-main/25" />
            <div class="flex justify-end gap-2.5">
              <DialogClose class="px-3.5 py-2 border border-border-main/40 hover:bg-surface-bg/50 text-text-secondary rounded-xl text-xs font-bold transition-all shadow-sm">
                取消
              </DialogClose>
              <button type="button" class="px-3.5 py-2 bg-danger-500 bg-danger-500/90 hover:bg-danger-600 hover:shadow-lg hover:shadow-danger-500/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md" @click="handleDeleteDoc">
                删除
              </button>
            </div>
            <DialogClose class="absolute top-4 right-4 p-1.5 rounded-full hover:bg-surface-bg/50 transition-colors" aria-label="关闭">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </DialogClose>
          </DialogContent>
        </DialogPortal>
      </DialogRoot>
  ```

- [ ] **Step 4: 使用 `tsc` 验证前端代码是否编译正常**

  Run: `pnpm tsc --noEmit`

- [ ] **Step 5: 提交任务**
