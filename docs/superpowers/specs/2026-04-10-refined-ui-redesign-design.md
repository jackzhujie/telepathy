# 凝光 — 精致紧凑 UI 重设计

## 概述

将 Telepathy 的 UI 从当前的「宽松暗色主题」升级为「精致紧凑暗色主题」，参考 Arc Browser、Linear、Notion 等 macOS 原生质感应用，实现圆角窗口、轻边框、紧凑间距、精致字体的整体视觉提升。

## 设计目标

1. **macOS 风格圆角窗口** — 透明窗口 + CSS 圆角，真正的原生窗口质感
2. **轻量边框** — 边框几乎隐形，用背景色差分层而非线条
3. **紧凑精致** — 缩小字体、间距、组件尺寸，提高信息密度但不牺牲可读性
4. **统一圆角** — 全局一致的圆角体系

## 设计规格

### 1. 窗口系统

- **Tauri 配置**: `transparent: true`，保持 `decorations: false`
- **CSS 圆角**: 根容器 `border-radius: 10px` + `overflow: hidden`
- **窗口阴影**: `box-shadow: 0 20px 60px rgba(0,0,0,0.5)` 模拟 macOS 窗口投影
- **性能注意**: 透明窗口下移除 `backdrop-filter` 毛玻璃效果，改为纯色背景

### 2. 字体系统

| 用途 | 当前 | 调整后 |
|------|------|--------|
| 全局字体 | Plus Jakarta Sans | **Inter** |
| Logo | text-xl (20px) font-black | text-sm (14px) font-bold |
| 页面标题 | text-xl (20px) font-bold | text-lg (18px) font-semibold |
| 正文/菜单 | text-sm (14px) | text-xs (12px) |
| 辅助文字 | text-sm (14px) | text-[11px] (11px) |
| 搜索框 | text-sm (14px) | text-xs (12px) |

字体引入方式：Google Fonts CDN `@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap')`

### 3. 间距系统

| 区域 | 当前 | 调整后 |
|------|------|--------|
| 侧边栏内边距 | p-5 (20px) | p-3 (12px) |
| 导航栏高度 | h-16 (64px) | h-11 (44px) |
| 导航栏内边距 | px-8 (32px) | px-4 (16px) |
| 内容区内边距 | p-6 (24px) | p-4 (16px) |
| 菜单项间距 | space-y-3 (12px) | space-y-1 (4px) |
| 菜单按钮内边距 | px-3 py-3 | px-2.5 py-1.5 |
| 组件间距 | gap-3/gap-4 | gap-2 (8px) |

### 4. 侧边栏

| 属性 | 当前 | 调整后 |
|------|------|--------|
| 展开宽度 | 240px | 220px |
| 折叠宽度 | 72px | 56px |
| Logo 区 | p-5, text-xl | p-3, text-sm |
| 菜单图标 | 20x20 | 16x16 |
| 底部按钮区 | p-4 | p-2 |

### 5. 边框与分割

| 属性 | 当前 | 调整后 |
|------|------|--------|
| 边框色 | #252836 | #1A1D28（更暗，几乎隐形） |
| 边框宽度 | 1px | 0.5px（CSS border-width: 0.5px） |
| 侧边栏右边框 | 保留 | 保留但更轻 |
| 导航栏底边框 | 保留 | 保留但更轻 |

### 6. 圆角统一

| 元素 | 当前 | 调整后 |
|------|------|--------|
| 按钮/输入框 | rounded-lg (8px) | rounded-md (6px) |
| 卡片/面板 | rounded-xl (12px) | rounded-lg (8px) |
| 浮动面板 | rounded-xl (12px) | rounded-lg (8px) |
| 窗口控制按钮 | rounded-full | rounded-full（保持） |

### 7. 颜色微调

保持现有暗色体系，仅微调：

| Token | 当前值 | 调整后 | 说明 |
|-------|--------|--------|------|
| dark-border | #252836 | #1A1D28 | 更柔和的分割线 |
| dark-surface | #1A1D28 | #181B25 | 压暗增加面板对比度 |

### 8. 窗口控制按钮

保持 macOS 风格红黄绿三色按钮，但缩小尺寸：
- 当前: w-8 h-8 (32x32)
- 调整后: w-3 h-3 (12x12)，间距 gap-1.5 (6px)

## 涉及文件

| 文件 | 改动内容 |
|------|----------|
| src-tauri/tauri.conf.json | 添加 transparent: true |
| src/style.css | 字体换 Inter、边框 0.5px、根容器圆角阴影、全局字号调整 |
| tailwind.config.js | 字体 family、颜色微调 |
| src/components/layout/AppLayout.vue | 导航栏紧凑化、窗口圆角容器、间距缩小、控制按钮缩小 |
| src/components/layout/Sidebar.vue | 宽度缩小、间距紧凑、图标缩小 |
| src/components/layout/NotificationPanel.vue | 间距紧凑 |
| src/views/KnowledgeBase.vue | 间距紧凑、字号缩小 |
| src/views/Chat.vue | 间距紧凑、字号缩小 |
| src/views/Documents.vue | 间距紧凑、字号缩小 |
| src/views/Profile.vue | 间距紧凑、字号缩小 |
| src/views/Settings.vue | 间距紧凑、字号缩小 |
| src/components/profile/UserProfilePanel.vue | 间距紧凑 |
| src/components/batch/BatchImportDialog.vue | 间距紧凑 |

## 不变的部分

- 品牌色体系（Indigo #6366F1）保持不变
- 语义色（success/warning/danger）保持不变
- 文本色阶（primary/secondary/muted）保持不变
- 所有功能逻辑和交互行为保持不变
- 暗色模式保持不变（已是暗色主题）
