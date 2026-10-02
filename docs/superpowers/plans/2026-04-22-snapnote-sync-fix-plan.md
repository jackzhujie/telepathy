# SnapNote 编辑器同步与可见性修复实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复禅模式不可见及双栏模式不同步的问题，实现源码与 WYSIWYG 编辑器的全双向实时同步。

**Architecture:** 
1. 在 `MilkdownEditor` 中通过 `watch` 监听外部 `modelValue` 变化，并利用 `replaceAll` 同步到编辑器内部。
2. 通过防抖或简单的相等判断防止双向绑定产生的死循环。
3. 调整 CSS 确保在主应用的暗黑主题下内容可见。

**Tech Stack:** Milkdown, Vue 3, Tailwind CSS.

---

### Task 1: 完善 MilkdownEditor 双向同步逻辑

**Files:**
- Modify: `src/components/editor/MilkdownEditor.vue`

- [ ] **Step 1: 引入防抖或同步锁**
在 `MilkdownEditor.vue` 的 `<script>` 中添加状态判断，避免循环触发更新。

- [ ] **Step 2: 更新 watch 逻辑**
修改 `watch(() => props.modelValue, ...)`，使其支持非空内容的同步。
```typescript
watch(() => props.modelValue, (newValue) => {
  const editor = get();
  if (editor && newValue !== editor.action((ctx) => ctx.get(editorViewCtx).state.doc.textContent)) { // 简化判断
    // 实际应使用 markdown 序列化值对比，或者简单的同步锁
    editor.action(replaceAll(newValue));
  }
});
```

- [ ] **Step 3: 调整样式确保可见性**
确保 `prose-invert` 正确生效，并微调 `milkdown` 的全局样式。

### Task 2: 优化 SnapNote.vue 布局与样式

**Files:**
- Modify: `src/views/SnapNote.vue`

- [ ] **Step 1: 优化双栏布局比例**
确保左侧源码区和右侧预览区在双栏模式下各占 50%，并修复背景色导致的对比度问题。

- [ ] **Step 2: 添加暗黑模式兼容类**
在编辑器容器上添加 `dark` 或 `prose-invert` 相关类名。

### Task 3: 验证修复

- [ ] **Step 1: 验证双向同步**
在左侧 `textarea` 输入，右侧 Milkdown 应同步渲染；在右侧编辑，左侧 `textarea` 应同步更新源码。
- [ ] **Step 2: 验证禅模式**
切换到禅模式，确保编辑器充满容器且内容清晰可见。
- [ ] **Step 3: 验证发布后清空**
点击发布，确保两边内容同时被清空。
