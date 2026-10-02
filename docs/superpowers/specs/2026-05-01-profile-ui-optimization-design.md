# 用户信息界面 UI 优化设计方案 (Profile UI Optimization)

## 1. 目标与设计理念
为了使用户信息页面（`Profile.vue`）具备一致的高级 **极客/毛玻璃质感风格（Glassmorphism）**，并增强视觉质感。支持深色与浅色动态主题色配置。

## 2. 详细设计规范

### A. 整体框架与容器
- **基础背景**：`max-w-[680px] mx-auto min-h-full px-1 pb-4 bg-app-bg/40 backdrop-blur-md select-text`。
- **大标题**：`bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary font-black text-xl tracking-tight`。

### B. 表单区块 Section
- **面板卡片**：`border border-border-main/35 bg-panel-bg/45 backdrop-blur-md rounded-2xl p-4 transition-all duration-300 shadow-sm select-none`。
- **区块小标题**：带精致微发光图标或色彩渐变标识。

### C. 输入框与下拉选择框
- **文本输入框**：`w-full px-3 py-2 border border-border-main/35 rounded-xl bg-panel-bg/45 backdrop-blur-sm text-text-primary focus:outline-none focus:ring-2 focus:ring-brand/30 transition-all text-xs hover:border-border-main/60 shadow-sm select-text`。
- **下拉选择框 (reka-ui select)**：`inline-flex w-full items-center justify-between rounded-xl px-3 py-2 bg-panel-bg/45 backdrop-blur-sm border border-border-main/35 text-text-primary hover:bg-surface-bg/60 transition-all hover:border-border-main/60 shadow-sm`。
- **下拉内容框**：`bg-panel-bg/60 backdrop-blur-md rounded-xl shadow-xl border border-border-main/35 overflow-hidden z-[100] animate-fade-in`。

### D. 兴趣偏好标签区
- **兴趣卡片标签**：`inline-flex items-center gap-1.5 px-3 py-1 bg-brand/15 border border-brand/25 text-brand rounded-xl text-xs font-bold transition-all shadow-sm`。
- **添加按钮与输入框**：`bg-gradient-to-br from-brand to-brand-600 rounded-xl hover:shadow-lg transition-all`。
