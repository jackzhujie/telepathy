# 知识库界面 UI 优化设计方案 (Knowledge Base UI Optimization)

## 1. 目标与设计理念
为了使知识库页面（`KnowledgeBase.vue`）与聊天页面、侧边栏保持一致的 **极客/毛玻璃质感风格（Glassmorphism）**，提升应用整体的科技感与沉浸体验，我们将对知识库页面进行全面重构。
该设计全面支持动态主题色配置，不使用任何硬编码色值。

## 2. 详细设计规范

### A. 整体框架与背景
- **页面全屏背景**：采用 `bg-app-bg` 加上柔和的 `backdrop-blur-md`。
- **标题区域**：
  - 渐变微光大标题：`bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary`。
- **项目选择与搜索区**：
  - 下拉框（SelectRoot）和输入框（Input）：采用 `bg-panel-bg/40 backdrop-blur-md border border-border-main/35 rounded-xl hover:border-brand/40`。
  - 搜索按钮：升级为渐变按钮 `bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg`。

### B. 列表区域（左侧）
- **选中卡片**：
  - `bg-brand/10 backdrop-blur-sm border-2 border-brand/50 shadow-md font-semibold hover:-translate-y-0.5 transition-all duration-300`。
- **未选中卡片**：
  - `bg-panel-bg/45 backdrop-blur-md border border-border-main/35 hover:border-brand/35 hover:shadow-md hover:-translate-y-0.5 transition-all duration-300`。
- **文件类型标签**：
  - `bg-surface-bg/50 backdrop-blur-sm text-text-secondary border border-border-main/20`。

### C. 详情区域（右侧）
- **详情容器**：
  - 增加与主面板一致的 `bg-app-bg/40 backdrop-blur-md`，左侧边缘使用细腻描边 `border-l border-border-main/25`。
- **文本块卡片**：
  - `bg-panel-bg/35 backdrop-blur-md border border-border-main/25 hover:border-brand/30 hover:shadow-md transition-all rounded-xl`。
  - 索引索引标：`bg-brand/10 text-brand font-bold`。
