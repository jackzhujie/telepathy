# 2026-05-02 文档删除增加二次确认设计文档

## 1. 概述
当前在“文档管理”页面（`Documents.vue`），点击文档右侧的“删除”按钮后会直接执行删除，没有任何二次确认模态框拦截。这容易导致误操作。为了提高安全性，需要引入二次确认模态框。

## 2. 解决方案：引入 reka-ui 模态框

为了在 `Documents.vue` 内部保持统一，我们将继续使用 `reka-ui` 的 `DialogRoot` 等组件来实现。

### 代码修改计划：
在 `src/views/Documents.vue` 的 `<script setup>` 中：
1. 引入相关响应式状态：
   ```typescript
   import type { Document } from '@/types/document';

   const showDeleteDocConfirm = ref(false);
   const docToDelete = ref<Document | null>(null);
   ```
2. 新增打开弹窗逻辑并重构原 `handleDelete` 逻辑：
   ```typescript
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

### 模板修改：
1. 找到文档列表中的“删除”按钮，将其 `@click="handleDelete(documentsRef[virtualItem.index].id)"` 替换为 `@click="confirmDeleteDoc(documentsRef[virtualItem.index])"`。
2. 在页面底部插入 `reka-ui` 的 `DialogRoot`：

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

## 3. 验收标准
- 点击“删除”文档按钮，页面不再直接删除，而是弹出确认模态框。
- 点击模态框中的“取消”，关闭模态框并不执行删除。
- 点击模态框中的“删除”，调用后端接口彻底删除文档。
