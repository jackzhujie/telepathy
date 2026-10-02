# 用户信息界面 UI 优化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 优化用户信息界面（`Profile.vue`）的视觉风格，使其具备一致的高级极客毛玻璃质感风格（Glassmorphism），完全支持动态主题色。

**Architecture:** 更新主体容器背景、大标题、表单 Section、文本输入框、下拉框以及兴趣标签的样式。

**Tech Stack:** Vue 3.5 + TypeScript + Tailwind CSS

---

### Task 1: 基础框架、基本信息和职业背景重构
**Files:**
- Modify: `src/views/Profile.vue`

- [ ] **Step 1: 升级主体容器与大标题**
  - 主容器：`max-w-[680px] mx-auto min-h-full px-1 pb-4 bg-app-bg/40 backdrop-blur-md select-text animate-fade-in`。
  - 标题：“用户信息”换上渐变文字。
- [ ] **Step 2: 重构基本信息与职业背景 Section**
  - Section：`border border-border-main/35 bg-panel-bg/45 backdrop-blur-md rounded-2xl p-4 transition-all duration-300 shadow-sm mb-4`。
  - 文本输入框：`w-full px-3 py-2 border border-border-main/35 rounded-xl bg-panel-bg/45 text-text-primary focus:outline-none focus:ring-2 focus:ring-brand/30 transition-all text-xs hover:border-border-main/60 shadow-sm select-text`。
  - 下拉框选择器：`inline-flex w-full items-center justify-between rounded-xl px-3 py-2 bg-panel-bg/45 border border-border-main/35 text-text-primary hover:bg-surface-bg/60 transition-all hover:border-border-main/60 shadow-sm`。
  - 下拉弹出面板：`bg-panel-bg/60 backdrop-blur-md rounded-xl shadow-xl border border-border-main/35 overflow-hidden z-[100] animate-fade-in`。
- [ ] **Step 3: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 4: 提交变更**
  - `git add src/views/Profile.vue && git commit -m "feat(ui): optimize profile frame and form cards"`

---

### Task 2: 兴趣偏好标签区域优化
**Files:**
- Modify: `src/views/Profile.vue`

- [ ] **Step 1: 重构兴趣卡片与添加输入框**
  - Section：`border border-border-main/35 bg-panel-bg/45 backdrop-blur-md rounded-2xl p-4 transition-all duration-300 shadow-sm mb-4`。
  - 兴趣卡片项：`inline-flex items-center gap-1.5 px-3 py-1.5 bg-brand/15 border border-brand/25 text-brand rounded-xl text-xs font-bold transition-all shadow-sm`。
  - 添加按钮：`px-4 py-2 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md`。
- [ ] **Step 2: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 3: 提交变更**
  - `git add src/views/Profile.vue && git commit -m "feat(ui): optimize interest cards and input area"`
