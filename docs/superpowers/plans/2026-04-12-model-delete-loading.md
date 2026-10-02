# 模型删除 Loading 状态实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为模型管理界面添加删除 Loading 状态，提升大模型删除时的用户体验。

**Architecture:** 在 `ModelManager.vue` 中引入 `isDeletingInProgress` 状态，结合 `try...finally` 确保删除期间 UI 提供明确反馈并防止重复操作。

**Tech Stack:** Vue 3 (Composition API), Tailwind CSS, Lucide-like SVG icons.

---

### Task 1: 更新删除逻辑与状态

**Files:**
- Modify: `src/components/settings/ModelManager.vue`

- [ ] **Step 1: 添加新的响应式状态**
  在 `deletingModel` 变量下方添加 `isDeletingInProgress`。

```typescript
const deletingModel = ref<string | null>(null);
const isDeletingInProgress = ref<string | null>(null); // 新增
```

- [ ] **Step 2: 重构 handleDelete 方法**
  使用 `try...finally` 结构包裹删除逻辑，确保状态正确流转。

```typescript
const handleDelete = async (fullName: string) => {
  isDeletingInProgress.value = fullName;
  try {
    await store.removeModel(fullName);
  } catch (error) {
    console.error('删除模型失败:', error);
  } finally {
    isDeletingInProgress.value = null;
    deletingModel.value = null;
  }
};
```

- [ ] **Step 3: 提交更改**

```bash
git add src/components/settings/ModelManager.vue
git commit -m "feat: add isDeletingInProgress state and refactor handleDelete"
```

---

### Task 2: 更新 UI 表现层 (Spinner 与 禁用状态)

**Files:**
- Modify: `src/components/settings/ModelManager.vue`

- [ ] **Step 1: 修改“确定删除”按钮以包含 Loading 效果**
  替换原有的“确定删除”按钮，添加 Spinner SVG 和动态状态。

```html
<button 
  class="px-2 py-0.5 text-[10px] text-danger-500 hover:text-white bg-danger-500/10 hover:bg-danger-500 rounded border border-danger-500/20 transition-all flex items-center gap-1 disabled:opacity-50 disabled:cursor-not-allowed"
  @click="handleDelete(model.full_name)"
  :disabled="isDeletingInProgress === model.full_name"
  title="确定删除"
>
  <svg v-if="isDeletingInProgress === model.full_name" class="animate-spin h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
  </svg>
  {{ isDeletingInProgress === model.full_name ? '正在删除...' : '确定删除' }}
</button>
```

- [ ] **Step 2: 禁用“取消”按钮**
  在删除进行时禁用取消按钮。

```html
<button 
  class="px-2 py-0.5 text-[10px] text-text-muted hover:text-text-primary bg-dark-panel/50 hover:bg-dark-surface rounded border border-dark-border/50 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
  @click="cancelDelete"
  :disabled="isDeletingInProgress === model.full_name"
  title="取消"
>
  取消
</button>
```

- [ ] **Step 3: 提交并验证**

```bash
git add src/components/settings/ModelManager.vue
git commit -m "feat: implement delete loading UI and disabled states"
```
