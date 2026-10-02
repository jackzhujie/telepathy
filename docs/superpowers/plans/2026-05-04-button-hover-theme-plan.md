# 按钮悬浮状态主题化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 改造 Telepathy 项目中的按钮 hover 状态，将其与主题色动态关联，完美自适应各种色彩及暗色/亮色主题。

**Architecture:** A+C 结合方案。通过在 `style.css` 中提取、升级按钮组件类，并利用带有 Alpha 通道的 `bg-color/alpha` 替换页面中写死的特定色阶（如 `hover:bg-brand-600`、`hover:bg-danger-600`）。

**Tech Stack:** Tailwind CSS, CSS Custom Properties

---

### Task 1: 改造 `style.css` 中的全局基础按钮类

**Files:**
- Modify: `src/style.css:185-217`

- [ ] **Step 1: 升级基础按钮类为带有 Alpha 通道的自适应属性**

在 `src/style.css` 中找到 `.btn-brand`、`.btn-ghost`、`.btn-danger`、`.btn-success`，将硬编码悬浮属性替换为 Alpha 映射，并追加通用的 `.btn-secondary` 按钮类。

```css
  /* Buttons */
  .btn-brand {
    @apply px-4 py-2 bg-brand text-white font-medium rounded-lg
           hover:bg-brand/90 active:bg-brand/80
           focus:outline-none focus:ring-2 focus:ring-brand/50
           transition-all duration-200
           flex items-center justify-center gap-2;
  }

  .btn-ghost {
    @apply px-4 py-2 text-text-secondary font-medium rounded-lg
           hover:bg-surface-bg/70 hover:text-text-primary
           active:bg-hover-bg/60
           focus:outline-none focus:ring-2 focus:ring-border-main
           transition-all duration-200
           flex items-center justify-center gap-2;
  }

  .btn-danger {
    @apply px-4 py-2 bg-danger text-white font-medium rounded-lg
           hover:bg-danger/90 active:bg-danger/80
           focus:outline-none focus:ring-2 focus:ring-danger/50
           transition-all duration-200
           flex items-center justify-center gap-2;
  }

  .btn-success {
    @apply px-4 py-2 bg-success text-white font-medium rounded-lg
           hover:bg-success/90 active:bg-success/80
           focus:outline-none focus:ring-2 focus:ring-success/50
           transition-all duration-200
           flex items-center justify-center gap-2;
  }

  .btn-secondary {
    @apply px-4 py-2 bg-surface-bg text-text-primary font-medium rounded-lg
           hover:bg-hover-bg active:bg-border-main
           focus:outline-none focus:ring-2 focus:ring-border-main/50
           transition-all duration-200
           flex items-center justify-center gap-2;
  }
```

- [ ] **Step 2: 验证编译测试**

Run: `npm run dev` 确保项目能正常启动并且没有任何 CSS 语法警告。
Expected: PASS

---

### Task 2: 替换 Vue 页面中的硬编码悬浮背景色

**Files:**
- Modify: `src/views/Documents.vue`
- Modify: `src/views/KnowledgeBase.vue`
- Modify: `src/components/layout/AppLayout.vue`
- Modify: `src/components/profile/UserProfilePanel.vue`

- [ ] **Step 1: 修正 `Documents.vue` 中的删除按钮 hover 色阶**

打开 `src/views/Documents.vue` 将两处删除确认框中的 `hover:bg-danger-600` 改为 `hover:bg-danger/90`：

```vue
<!-- 替换前 -->
<button type="button" class="px-3.5 py-2 bg-danger-500 bg-danger-500/90 hover:bg-danger-600 hover:shadow-lg hover:shadow-danger-500/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md" @click="handleDeleteDoc">

<!-- 替换后 -->
<button type="button" class="px-3.5 py-2 bg-danger-500 bg-danger-500/90 hover:bg-danger/90 hover:shadow-lg hover:shadow-danger-500/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md" @click="handleDeleteDoc">
```

- [ ] **Step 2: 修正 `KnowledgeBase.vue` 中的静态 hover 色阶**

打开 `src/views/KnowledgeBase.vue`：
1. 将 273 行的 `hover:bg-brand-600` 修改为 `hover:bg-brand/90`。
2. 将 460 行的 `hover:bg-danger-600` 修改为 `hover:bg-danger/90`。

- [ ] **Step 3: 修正 `AppLayout.vue` 中的静态 hover 色阶**

打开 `src/components/layout/AppLayout.vue`：
1. 将 543 行处的 `hover:bg-brand-600` 修改为 `hover:bg-brand/90`。

- [ ] **Step 4: 修正 `UserProfilePanel.vue` 中的静态 hover 色阶**

打开 `src/components/profile/UserProfilePanel.vue`：
1. 将 199 行处的 `hover:bg-brand-600` 修改为 `hover:bg-brand/90`。

- [ ] **Step 5: 启动本地前端开发服务器，并进行回归验证**

1. 启动 `npm run dev`。
2. 点击应用内的各种按钮，测试其悬浮状态及点击回馈在亮色和暗色主题下是否完美协调。
3. 如果通过测试，提交修改。

```bash
git add .
git commit -m "style: make button hover colors adapt to themes via alpha opacity"
```
