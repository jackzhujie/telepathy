# 聊天界面 UI 优化设计规格书 (2026-05-01)

## 1. 目标与设计理念
为了提升 Telepathy 应用在聊天和问答过程中的沉浸感、科技感和高级感，我们将对聊天界面进行全面的 **极客/毛玻璃质感风格（Glassmorphism）** 升级。所有重构设计都将支持主题色配置（通过 CSS 变量动态适配深色/浅色和自定义配色），不采用任何硬编码色值。

## 2. 详细设计规范

### A. 页面基础容器与动效
- **主体背景**：应用全屏 `bg-app-bg` 结合 `backdrop-blur-md` 提供底层深度。
- **页面框架 (Chat.vue)**：
  - 聊天面板宽度放宽：由原本的 `max-w-3xl` 调整为 `max-w-4xl px-4 py-3`，扩展单行展示宽度。
  - 渐变微光动效：对正在生成和载入的消息加入 `animate-pulse-glass` 呼吸感毛玻璃反馈。
  - 新消息滑入动效：新加载的消息应用 `animate-message-appear`（向右上浮进场，300ms 缓动）。
- **Header 优化**：
  - 侧边栏折叠切换按钮：增加 `hover:bg-surface-bg/50` 柔和圆角卡片。
  - 页面标题：字体采用现代粗体并启用 `.brand-gradient-text`（或通过主题色控制）。
  - 下拉项目选择组件（SelectRoot）：改用 `bg-panel-bg/40 backdrop-blur-md border border-border-main/50` 面板，悬停时边框带有主题色微发光特效。

### B. 聊天消息气泡 (MessageItem.vue)
- **用户消息（右侧）**：
  - **背景**：采用基于主题色的半透明渐变（例如：`bg-brand/85 backdrop-blur-md`）。
  - **描边与阴影**：带有细腻的边缘描边（`border border-white/10 dark:border-white/5`），悬浮阴影（`shadow-md hover:shadow-brand/25`）。
- **助手消息（左侧）**：
  - **背景**：采用 `bg-panel-bg/45 backdrop-blur-md border border-border-main/40`。
  - **排版与行高**：正文使用 `text-text-primary`，文本行高提升为 `1.75`，字号 `text-xs`（12px）。
- **来源引用卡片 (SourcesPanel)**：
  - 收起/展开：设计成嵌入式卡片面板。
  - 单个来源卡片背景：`bg-surface-bg/40 backdrop-blur-sm border border-border-main/30`。

### C. 输入框 (ChatInput.vue)
- **Island 风格输入仓**：
  - 输入框整体包装为独立的悬浮毛玻璃底座。
  - 样式：`bg-panel-bg/45 backdrop-blur-md border border-border-main/40 p-2 rounded-xl shadow-lg focus-within:border-brand/50 focus-within:ring-2 focus-within:ring-brand/20 transition-all duration-300`。
- **发送按钮**：圆形或大圆角毛玻璃矩形，使用主题色渐变。

### D. 对话历史侧边栏 (ConversationSidebar.vue)
- **侧边栏面板**：`bg-panel-bg/35 backdrop-blur-md border-r border-border-main/35 transition-all duration-300`。
- **列表项 (Item)**：
  - 未选中：`hover:bg-surface-bg/30` 的柔和高光。
  - 选中：`bg-brand/10 backdrop-blur-sm border-l-2 border-brand text-brand`。

## 3. 验收标准与测试流程
1. **视觉保真度**：聊天面板、输入框、消息气泡在浅色模式和深色模式下均无硬编码色块，毛玻璃效果通透、层级清晰。
2. **主题色动态绑定**：切换自定义主题色（改变 `--color-brand` 等变量）后，气泡和按钮边框应自动跟随变化。
3. **滚动与性能**：TanStack Virtualized 虚拟滚动列表在加载 100+ 条毛玻璃气泡时，滑动依然保持每秒 60fps 的流畅。
