# 个人随笔界面 UI 优化设计方案 (SnapNote UI Optimization)

## 1. 目标与设计理念
为了使个人随笔页面（`SnapNote.vue`）与全局一致，并提供高效的 **极客/毛玻璃质感风格（Glassmorphism）**，我们将全面重构其整体容器、编辑区域和底部的历史随笔卡片。支持深色与浅色动态主题色配置。

## 2. 详细设计规范

### A. 页面基础容器
- **主体背景**：应用全屏 `bg-app-bg/40 backdrop-blur-md px-1 select-text h-full flex flex-col p-6`。
- **大标题**：`bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary tracking-tight`。

### B. 模式切换
- **模式切换开关面板**：
  - `bg-panel-bg/45 backdrop-blur-md p-1 rounded-xl border border-border-main/35 shadow-sm`。
  - 选中项：`bg-gradient-to-br from-brand to-brand-600 shadow-md text-white rounded-lg font-bold`。

### C. Markdown 编辑区容器
- **外部容器**：
  - `bg-panel-bg/45 backdrop-blur-md rounded-2xl border border-border-main/35 shadow-2xl overflow-hidden`。
- **源码编辑和底部控制栏**：
  - 源码编辑框（双栏）：`w-1/2 border-r border-border-main/20 p-6 overflow-y-auto bg-black/5`。
  - 底部控制栏：`p-4 border-t border-border-main/25 bg-panel-bg/25 backdrop-blur-sm flex items-center justify-between`。
  - 标签输入框与发布按钮：`bg-panel-bg/45 border border-border-main/35 rounded-xl`，发布按钮升级为渐变按钮。

### D. 历史记录卡片区
- **历史容器**：
  - `bg-panel-bg/25 border border-border-main/25 backdrop-blur-md rounded-2xl p-4 mt-auto shadow-sm`。
- **随记卡片**：
  - `bg-panel-bg/45 backdrop-blur-md rounded-xl border border-border-main/35 hover:border-brand/40 hover:shadow-md hover:-translate-y-0.5 transition-all duration-300`。
