# 知识库界面 UI 优化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 优化知识库界面（`KnowledgeBase.vue`）的视觉风格，使其具备高级极客毛玻璃质感，完全支持动态主题色配置。

**Architecture:** 更新页面主容器、搜索栏、文档列表卡片及文本块详情区域样式，通过 Tailwind 的透明通道和 Backdrop filter 实现优雅的毛玻璃界面。

**Tech Stack:** Vue 3.5 + TypeScript + Tailwind CSS

---

### Task 1: 整体框架与背景及标题搜索区域优化
**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 优化主体框架容器与标题样式**
  - 将背景改用 `bg-app-bg/40 backdrop-blur-md`，并将标题改为渐变色文本。
- [ ] **Step 2: 优化下拉框与搜索输入框**
  - 改用毛玻璃边框和半透明圆角样式 `bg-panel-bg/45 backdrop-blur-md border border-border-main/35 rounded-xl hover:border-brand/40`。
- [ ] **Step 3: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 4: 提交变更**
  - `git add src/views/KnowledgeBase.vue && git commit -m "feat(ui): optimize knowledge base frame, title, and search panel"`

---

### Task 2: 文档列表卡片毛玻璃化
**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 重构文档列表卡片样式**
  - 未选中卡片：使用 `bg-panel-bg/45 backdrop-blur-md border border-border-main/35 rounded-xl hover:border-brand/35 hover:shadow-md hover:-translate-y-0.5 transition-all duration-300`。
  - 选中卡片：使用 `bg-brand/10 backdrop-blur-sm border-2 border-brand/50 rounded-xl shadow-md font-semibold hover:-translate-y-0.5`。
- [ ] **Step 2: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 3: 提交变更**
  - `git add src/views/KnowledgeBase.vue && git commit -m "feat(ui): optimize document list cards with glassmorphism"`

---

### Task 3: 详情面板容器与文本块卡片优化
**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 升级右侧文本块详情卡片**
  - 右侧详情容器加入 `bg-app-bg/40 backdrop-blur-md border-l border-border-main/25`。
  - 文本块项使用 `bg-panel-bg/35 backdrop-blur-sm border border-border-main/25 rounded-xl hover:border-brand/30 hover:shadow-md transition-all duration-300`。
- [ ] **Step 2: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 3: 提交变更**
  - `git add src/views/KnowledgeBase.vue && git commit -m "feat(ui): optimize chunk list cards with glassmorphism"`
