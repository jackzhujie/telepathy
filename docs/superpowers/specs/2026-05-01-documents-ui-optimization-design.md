# 文档管理界面 UI 优化设计方案 (Documents UI Optimization)

## 1. 目标与设计理念
为了提升“文档管理”页面（`Documents.vue`）的沉浸感与整体视觉高级感，我们将其全面升级至 **极客/毛玻璃质感风格（Glassmorphism）**。该重构全面支持动态主题色配置，保证在深浅色模式下均拥有优良的通透度和体验。

## 2. 详细设计规范

### A. 页面基础容器
- **主体背景**：应用全屏 `bg-app-bg/40 backdrop-blur-md px-1 select-text`。
- **大标题**：采用现代粗体，配合渐变色 `bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary tracking-tight`。

### B. 操作工具栏（项目与文档管理）
- **下拉框触发器 (SelectTrigger)**：
  - `bg-panel-bg/45 backdrop-blur-md border border-border-main/40 rounded-xl hover:border-brand/40`。
- **项目模态框 (DialogContent)**：
  - `bg-panel-bg/65 backdrop-blur-md border border-border-main/35 rounded-2xl`。
- **按钮控件**：
  - 核心按钮（新建/导入）：`bg-gradient-to-br from-brand to-brand-600 rounded-xl hover:shadow-lg hover:scale-[1.02] active:scale-98 transition-all`。
  - 编辑/删除/取消等辅按钮：`bg-panel-bg/35 border border-border-main/35 rounded-xl`。

### C. 警告与详情卡片
- **Sidecar/无项目警告与索引提示**：
  - 改用毛玻璃微光底座：`bg-brand/10 backdrop-blur-sm border border-brand/25 rounded-xl`。
- **项目基本详情面板**：
  - `bg-panel-bg/35 backdrop-blur-sm border border-border-main/25 rounded-xl`。

### D. 导入区与文档列表
- **拖拽区**：
  - `border border-dashed border-border-main/35 bg-panel-bg/25 backdrop-blur-sm rounded-2xl hover:border-brand/50 hover:bg-brand/5`。
- **文档表格列表**：
  - 表格容器：`bg-panel-bg/45 backdrop-blur-md border border-border-main/25 rounded-xl overflow-hidden`。
  - 单个表格行：`hover:bg-brand/5 transition-colors border-b border-border-main/25`。
  - 标签与操作按钮：文件类型 Tag、解析/索引/删除按钮均微调为精致小圆角/毛玻璃描边微发光形态。
