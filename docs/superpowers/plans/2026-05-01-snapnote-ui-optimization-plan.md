# 个人随笔界面 UI 优化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 优化个人随记界面（`SnapNote.vue`）的视觉风格，使其具备高级极客毛玻璃质感，完全支持动态主题色配置。

**Architecture:** 更新主容器、头部控制栏、Markdown 编辑框容器、底部控制项、发布随笔按钮以及随笔历史卡片区域的样式。

**Tech Stack:** Vue 3.5 + TypeScript + Tailwind CSS

---

### Task 1: 整体框架背景与模式切换栏优化
**Files:**
- Modify: `src/views/SnapNote.vue`

- [ ] **Step 1: 优化整体主体容器与大标题**
  - 主容器：`bg-app-bg/40 backdrop-blur-md px-1 select-text h-full flex flex-col p-6`。
  - 标题：“随笔工作台”增加渐变文字。
- [ ] **Step 2: 优化视图切换模式选择器**
  - 触发容器：`bg-panel-bg/45 backdrop-blur-md p-1 rounded-xl border border-border-main/35 shadow-sm`。
  - 按钮样式：使用 `bg-gradient-to-br from-brand to-brand-600 rounded-lg text-white font-bold`。
- [ ] **Step 3: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 4: 提交变更**
  - `git add src/views/SnapNote.vue && git commit -m "feat(ui): optimize snapnote frame, title, and view mode switch"`

---

### Task 2: 编辑器容器与底部控制栏优化
**Files:**
- Modify: `src/views/SnapNote.vue`

- [ ] **Step 1: 重构编辑器容器外部框样式**
  - 外框：`bg-panel-bg/45 backdrop-blur-md rounded-2xl border border-border-main/35 shadow-2xl overflow-hidden`。
- [ ] **Step 2: 优化源码编辑区域与底部控制栏**
  - 源码编辑：`w-1/2 border-r border-border-main/20 p-6 overflow-y-auto bg-black/5`。
  - 底部控制栏：`p-4 border-t border-border-main/25 bg-panel-bg/25 backdrop-blur-sm flex items-center justify-between`。
  - 发布随笔按钮：`bg-gradient-to-br from-brand to-brand-600 rounded-xl hover:shadow-lg transition-all hover:scale-[1.02] active:scale-98`。
- [ ] **Step 3: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 4: 提交变更**
  - `git add src/views/SnapNote.vue && git commit -m "feat(ui): optimize editor container and control bar"`

---

### Task 3: 底部随笔历史记录预览区域优化
**Files:**
- Modify: `src/views/SnapNote.vue`

- [ ] **Step 1: 升级历史卡片容器与项**
  - 历史外容器：`bg-panel-bg/25 border border-border-main/25 backdrop-blur-md rounded-2xl p-4 mt-auto shadow-sm`。
  - 单个历史随记卡片项：`bg-panel-bg/45 backdrop-blur-md rounded-xl border border-border-main/35 hover:border-brand/40 hover:shadow-md hover:-translate-y-0.5 transition-all duration-300 select-none flex-shrink-0 w-64 p-4`。
- [ ] **Step 2: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 3: 提交变更**
  - `git add src/views/SnapNote.vue && git commit -m "feat(ui): optimize historical note cards"`
