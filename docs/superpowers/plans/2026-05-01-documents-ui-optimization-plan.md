# 文档管理界面 UI 优化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 优化文档管理界面（`Documents.vue`）的视觉风格，使其具备高级极客毛玻璃质感，全面支持动态主题色配置。

**Architecture:** 更新主体容器、顶部操作区域、警告与提示横幅、拖拽区、虚拟滚动列表以及模态框的整体外观，使其拥有现代、轻盈的玻璃质感。

**Tech Stack:** Vue 3.5 + TypeScript + Tailwind CSS

---

### Task 1: 整体框架背景与顶部控制区域优化
**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 优化页面主体容器与标题样式**
  - 主容器背景：`bg-app-bg/40 backdrop-blur-md px-1 select-text`。
  - 标题：“文档管理”增加渐变 `bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary tracking-tight`。
- [ ] **Step 2: 重构下拉框选择、按钮及操作按钮样式**
  - 触发器：`bg-panel-bg/45 backdrop-blur-md border border-border-main/40 rounded-xl hover:border-brand/40`。
  - 按钮：“新建项目”、“导入文档”换上渐变 `bg-gradient-to-br from-brand to-brand-600 rounded-xl hover:shadow-lg transition-all hover:scale-[1.02] active:scale-98`。
  - “编辑”、“删除”按钮适配玻璃边框形态。
- [ ] **Step 3: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 4: 提交变更**
  - `git add src/views/Documents.vue && git commit -m "feat(ui): optimize documents frame, title, and action controls"`

---

### Task 2: 警告横幅、拖拽区域与项目信息卡片优化
**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 优化警告与提示横幅**
  - 将提示和警告卡片更新为通透的玻璃底座 `bg-brand/10 backdrop-blur-sm border border-brand/25 rounded-xl` 或警告模式 `bg-warning-500/10 backdrop-blur-sm border border-warning-500/25 rounded-xl`。
- [ ] **Step 2: 优化项目详情面板与拖拽上传区**
  - 项目基本详情面板：`bg-panel-bg/35 backdrop-blur-sm border border-border-main/25 rounded-xl`。
  - 拖拽导入区：`border border-dashed border-border-main/35 bg-panel-bg/25 backdrop-blur-sm rounded-2xl hover:border-brand/50 hover:bg-brand/5`。
- [ ] **Step 3: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 4: 提交变更**
  - `git add src/views/Documents.vue && git commit -m "feat(ui): optimize warning banners, project panel, and drag-and-drop area"`

---

### Task 3: 文档列表表格与模态框优化
**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 重构文档表格容器与虚拟列表行**
  - 表格容器：`bg-panel-bg/45 backdrop-blur-md border border-border-main/25 rounded-xl overflow-hidden`。
  - 表格行：`hover:bg-brand/5 transition-colors border-b border-border-main/25`。
  - 标签与操作按钮（解析、索引、删除）微调。
- [ ] **Step 2: 重构模态框（DialogContent）**
  - `bg-panel-bg/65 backdrop-blur-md border border-border-main/35 rounded-2xl`，输入框使用 `bg-panel-bg/45 rounded-xl`。
- [ ] **Step 3: 运行并验证 TypeScript**
  - 命令：`pnpm tsc --noEmit`
- [ ] **Step 4: 提交变更**
  - `git add src/views/Documents.vue && git commit -m "feat(ui): optimize documents table list and modaldialogs"`
