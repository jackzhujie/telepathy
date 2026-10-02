# Telepathy 暗色现代风 UI 重设计

## 概述

将 Telepathy 全部页面从当前浅色主题重新设计为暗色现代风格，以「深空知识」为设计概念，蓝色系品牌色，传达 AI 知识库的科技感与专业感。

## 设计方向

**概念**: 深空知识 — 以深色太空为灵感的暗色主题，蓝色系品牌色。

**约束**:
- 仅修改视觉层（CSS/模板），不改变功能逻辑
- 保持 reka-ui 组件库，通过 Tailwind 覆盖样式
- 保持现有 Tauri IPC 事件系统不变
- 保持所有 Store、Router、API 层不变

## 设计语言

### 色彩系统

| 语义 | 色值 | 用途 |
|------|------|------|
| 底色 | `#0a0a0f` | 页面最深背景 |
| 面板色 | `#111118` | 侧边栏、卡片背景 |
| 悬浮层 | `#1a1a24` | 弹窗、下拉菜单、hover 态 |
| 边框色 | `#1e1e2e` | 卡片边框、分隔线 |
| 边框亮色 | `#2a2a3a` | hover 态边框 |
| 品牌主色 | `#3b82f6` | CTA、活跃态、链接 |
| 品牌辅色 | `#6366f1` | 渐变终点、次要强调 |
| 品牌渐变 | `#3b82f6 → #6366f1` | 按钮、进度条 |
| 成功色 | `#22c55e` | 完成状态 |
| 警告色 | `#f97316` | 进行中状态 |
| 错误色 | `#ef4444` | 错误状态 |
| 主文字 | `#e2e8f0` | 标题、正文 |
| 次文字 | `#94a3b8` | 描述、元信息 |
| 弱文字 | `#64748b` | 占位符、禁用态 |

### 圆角系统

| 层级 | 值 | 用途 |
|------|-----|------|
| sm | 6px | 标签、小按钮 |
| md | 8px | 按钮、输入框 |
| lg | 12px | 卡片、面板 |
| xl | 16px | 弹窗、对话框 |

### 字体

- 显示字体: `Plus Jakarta Sans`（Google Fonts）
- 正文回退: `-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`
- 代码: `JetBrains Mono` 或 `Fira Code`

### 动效规范

- 过渡时长: 150ms（微交互）、300ms（页面切换）
- 缓动函数: `cubic-bezier(0.4, 0, 0.2, 1)`
- 卡片 hover: `translateY(-2px)` + 阴影加深
- 页面切换: `opacity` 淡入
- 按钮 hover: 渐变背景微位移

### 玻璃效果

- 侧边栏: `background: rgba(17, 17, 24, 0.8); backdrop-filter: blur(12px);`
- 顶部导航: `background: rgba(17, 17, 24, 0.6); backdrop-filter: blur(12px);`
- 浮动面板: `background: rgba(26, 26, 36, 0.9); backdrop-filter: blur(16px);`

## 各页面设计

### 1. 侧边栏 (Sidebar.vue)

- 深色半透明背景 + `backdrop-blur(12px)`
- 右侧 1px 边框 `#1e1e2e`
- 品牌名 "Telepathy": 顶部，`Plus Jakarta Sans` 粗体，白色
- 导航项: 图标 + 文字，圆角 8px，padding 10px 12px
  - 默认态: 透明背景，文字 `#94a3b8`
  - Hover 态: 背景 `rgba(59, 130, 246, 0.08)`，文字 `#e2e8f0`
  - Active 态: 背景 `rgba(59, 130, 246, 0.12)`，文字 `#3b82f6`，左侧 3px 蓝色竖条
- 折叠态: 宽度 64px，仅显示图标，品牌名显示首字母 "T"
- 折叠按钮: 底部，圆形，hover 蓝色微光

### 2. 顶部导航 (AppLayout.vue header)

- 深色半透明背景 + `backdrop-blur(12px)`
- 底部 1px 边框 `#1e1e2e`
- macOS 窗口控制按钮: 保持原生样式
- 搜索框: 背景 `#1a1a24`，边框 `#2a2a3a`，聚焦时边框 `#3b82f6` + 蓝色外发光
- 通知图标: 白色，未读角标蓝色小圆点
- 设置图标: 白色，hover 变蓝

### 3. 知识库页面 (KnowledgeBase.vue)

- 页面标题: 大号 `Plus Jakarta Sans` 粗体，白色
- 项目选择器: 深色下拉框，选中项蓝色高亮
- 搜索框: 同顶部搜索框样式
- 文档卡片列表:
  - 背景 `#111118`，边框 `#1e1e2e`，圆角 12px
  - Hover: 边框 `#3b82f6`，`translateY(-2px)`，阴影 `0 8px 24px rgba(59, 130, 246, 0.1)`
  - 文件名: 白色粗体
  - 文件类型标签: 彩色编码
    - `.md` → 蓝色 `rgba(59, 130, 246, 0.15)` 文字 `#60a5fa`
    - `.pdf` → 红色 `rgba(239, 68, 68, 0.15)` 文字 `#f87171`
    - `.txt` → 灰色 `rgba(148, 163, 184, 0.15)` 文字 `#94a3b8`
    - `.json` → 绿色 `rgba(34, 197, 94, 0.15)` 文字 `#4ade80`
    - 其他 → 紫色 `rgba(139, 92, 246, 0.15)` 文字 `#a78bfa`
  - 文本块数: 蓝色数字
  - 时间: `#64748b` 弱文字
- 搜索结果高亮: `rgba(59, 130, 246, 0.25)` 背景
- 空状态: 居中图标 + 引导文字

### 4. AI 对话页面 (Chat.vue)

- 左侧对话列表:
  - 背景 `#111118`，边框右侧 `#1e1e2e`
  - 对话项: hover 背景 `#1a1a24`，active 背景 `rgba(59, 130, 246, 0.12)`
  - 新建对话按钮: 蓝色渐变背景，白色文字
- 右侧聊天区域:
  - 消息气泡:
    - 用户消息: 蓝色渐变背景 `linear-gradient(135deg, #3b82f6, #6366f1)`，白色文字，右对齐
    - AI 消息: 背景 `#111118`，边框 `#1e1e2e`，左对齐
  - 流式输出: 末尾蓝色闪烁光标 `@keyframes blink`
  - 引用来源面板: 可折叠，蓝色左边框，深色卡片
  - 输入框: 底部固定，深色背景，蓝色聚焦环，发送按钮蓝色渐变
  - Markdown 渲染: 代码块深色背景 `#0a0a0f`，语法高亮使用暗色主题

### 5. 文档管理页面 (Documents.vue)

- 项目 CRUD: 深色 Dialog 弹窗，输入框同搜索框样式
- 文档表格:
  - 表头: `#64748b` 弱文字，底部 `#1e1e2e` 边框
  - 行: 交替背景 `transparent` / `rgba(255,255,255,0.02)`
  - Hover: 背景 `rgba(59, 130, 246, 0.05)`
  - 状态标签 (pill):
    - pending → 灰色 `#64748b`
    - parsing → 蓝色 `#3b82f6`
    - done → 绿色 `#22c55e`
    - error → 红色 `#ef4444`
    - indexed → 靛蓝 `#6366f1`
- 操作按钮: 图标按钮，hover 蓝色
- 拖拽导入区: 蓝色虚线边框 `#3b82f6`，拖入时背景 `rgba(59, 130, 246, 0.05)`
- 批量导入 Dialog: 同项目 CRUD Dialog 样式

### 6. 用户信息页面 (Profile.vue)

- 分区卡片: 背景 `#111118`，边框 `#1e1e2e`，圆角 12px
- 区块标题: 白色粗体，底部蓝色渐变分隔线
- 表单输入: 深色背景 `#1a1a24`，边框 `#2a2a3a`
- 兴趣标签: 蓝色半透明胶囊 `rgba(59, 130, 246, 0.15)` 文字 `#60a5fa`，删除按钮 hover 变红
- 添加标签: 虚线边框胶囊，点击变输入框

### 7. 设置页面 (Settings.vue)

- 手风琴面板: reka-ui Accordion，深色样式覆盖
  - 面板头: hover 背景 `#1a1a24`，展开箭头旋转动画
  - 面板内容: 内边距 16px
- 滑块 (Slider): 轨道 `#2a2a3a`，填充蓝色渐变
- 模型推荐卡片: 背景 `#111118`，边框 `#1e1e2e`，hover 微上移
- 进度条: 蓝色渐变填充，动画
- 应用信息: 底部，弱文字

### 8. 浮动面板 (AppLayout.vue)

- 重新索引: 橙色保持，但背景改为深色半透明 + `backdrop-blur`
  - 收起态: 深色圆形，橙色图标
  - 展开态: 深色半透明卡片，橙色进度条
- 批量索引: 蓝色保持，同上深色半透明处理
- 拖拽、收起/展开交互保持不变

### 9. 通知面板 (NotificationPanel.vue)

- 深色半透明背景 + `backdrop-blur`
- 通知项: 深色卡片，未读项左侧蓝色竖条
- 图标颜色按类型区分
- 时间: `#64748b` 弱文字

## 技术实现

### Tailwind 配置扩展

在 `tailwind.config.js` 中扩展:
- `colors`: 添加暗色主题色板 (dark-bg, dark-panel, dark-surface, dark-border 等)
- `borderRadius`: 添加 sm/md/lg/xl
- `backdropBlur`: 添加 glass 效果值
- `fontFamily`: 添加 display 字体

### 全局样式

在 `src/style.css` 中:
- `body` 背景设为 `#0a0a0f`
- 引入 Google Fonts `Plus Jakarta Sans`
- 定义 CSS 变量供非 Tailwind 场景使用
- 覆盖 reka-ui 组件的默认样式（Accordion、Dialog、Select、Toast 等）

### reka-ui 样式覆盖

通过 Tailwind 的 `@layer components` 覆盖 reka-ui 组件样式:
- Dialog: 深色背景 + 蓝色边框
- Select: 深色触发器 + 深色下拉面板
- Accordion: 深色面板 + 动画
- Toast: 深色背景 + 蓝色进度条
- Tooltip: 深色背景 + 蓝色箭头
- Progress: 蓝色渐变填充

### 文件修改清单

| 文件 | 改动类型 |
|------|---------|
| `tailwind.config.js` | 扩展暗色主题配置 |
| `src/style.css` | 全局暗色样式 + CSS 变量 + 字体引入 |
| `index.html` | 引入 Google Fonts |
| `src/components/layout/Sidebar.vue` | 暗色样式重写 |
| `src/components/layout/AppLayout.vue` | 暗色样式重写（导航 + 浮动面板） |
| `src/components/layout/NotificationPanel.vue` | 暗色样式重写 |
| `src/components/batch/BatchImportDialog.vue` | 暗色样式重写 |
| `src/components/profile/UserProfilePanel.vue` | 暗色样式重写 |
| `src/views/KnowledgeBase.vue` | 暗色样式重写 |
| `src/views/Chat.vue` | 暗色样式重写 |
| `src/views/Documents.vue` | 暗色样式重写 |
| `src/views/Profile.vue` | 暗色样式重写 |
| `src/views/Settings.vue` | 暗色样式重写 |
| `src/utils/markdown.ts` | 暗色代码高亮主题 |
