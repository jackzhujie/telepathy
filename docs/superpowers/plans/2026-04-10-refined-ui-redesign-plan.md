# 凝光 UI 重设计 — 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Telepathy UI 升级为精致紧凑暗色主题，实现 macOS 风格圆角窗口、轻边框、紧凑间距。

**Architecture:** 分层修改 — 先改基础设施（Tauri 配置、Tailwind 配置、全局样式），再改布局骨架（AppLayout、Sidebar），最后逐页调整视图组件。每层改动独立可验证。

**Tech Stack:** Tauri v2 (transparent window), Vue 3 + TypeScript, Tailwind CSS, Inter font (Google Fonts CDN)

---

### Task 1: 基础设施 — Tauri 窗口透明配置

**Files:**
- Modify: `src-tauri/tauri.conf.json:13-21`

- [ ] **Step 1: 添加 transparent: true 到窗口配置**

在 `src-tauri/tauri.conf.json` 的 windows[0] 中，在 `"decorations": false` 后添加 `"transparent": true`：

```json
{
  "title": "Telepathy",
  "width": 1200,
  "height": 800,
  "minWidth": 900,
  "minHeight": 600,
  "decorations": false,
  "transparent": true
}
```

- [ ] **Step 2: 提交**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat(window): enable transparent window for rounded corners"
```

---

### Task 2: 基础设施 — Tailwind 配置更新

**Files:**
- Modify: `tailwind.config.js`

- [ ] **Step 1: 更新字体、颜色、圆角配置**

修改 `tailwind.config.js` 中的以下部分：

**字体** — 将 `fontFamily.plus-jakarta` 改为 `fontFamily.inter`：
```js
fontFamily: {
  'inter': ['"Inter"', 'system-ui', '-apple-system', 'sans-serif'],
},
```

**颜色** — 微调 `dark-border` 和 `dark-surface`：
```js
'dark-border': '#1A1D28',     // 原 #252836，更柔和
'dark-surface': '#181B25',    // 原 #1A1D28，压暗增加对比
```

**圆角** — 无需额外修改，使用 Tailwind 默认值即可。

- [ ] **Step 2: 提交**

```bash
git add tailwind.config.js
git commit -m "style(config): update tailwind - Inter font, lighter borders"
```

---

### Task 3: 基础设施 — 全局样式更新

**Files:**
- Modify: `src/style.css`

- [ ] **Step 1: 更新字体引入**

将 `@import` 行从 Plus Jakarta Sans 改为 Inter：
```css
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap');
```

- [ ] **Step 2: 更新 body 字体类**

将 body 的 `font-plus-jakarta` 改为 `font-inter`：
```css
body {
  @apply font-inter bg-dark-bg text-text-primary min-h-screen w-screen m-0 p-0 antialiased;
  color-scheme: dark;
}
```

- [ ] **Step 3: 添加窗口圆角和阴影**

在 `:root` 或 body 样式后添加根容器样式（用于 `#app`）：
```css
#app {
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5), 0 0 0 0.5px rgba(255, 255, 255, 0.05);
  height: 100vh;
  width: 100vw;
}
```

- [ ] **Step 4: 更新边框为 0.5px**

在全局样式中添加轻边框工具类，替换所有 `border` 为更轻的版本：
```css
/* 轻量边框 - 几乎隐形 */
.border-subtle {
  border-width: 0.5px;
  border-style: solid;
  border-color: #1A1D28;
}
```

同时更新 `.glass` 和 `.dark-card` 等组件类中的边框：
- `.glass`: `border border-dark-border/50` → `border border-dark-border/30`（更轻）
- `.dark-card`: `border border-dark-border` → `border border-dark-border/50`（更轻）
- `.dark-input`: `border border-dark-border` → `border border-dark-border/50`（更轻）

- [ ] **Step 5: 移除毛玻璃效果**

将 `.glass` 类从 `backdrop-blur` 改为纯色（透明窗口下毛玻璃性能差）：
```css
.glass {
  @apply bg-dark-panel border border-dark-border/30;
}
```

移除 `.glass-light` 类中的 `backdrop-blur-sm`：
```css
.glass-light {
  @apply bg-dark-panel/60 border border-dark-border/20;
}
```

- [ ] **Step 6: 更新标题字号**

将 h1-h6 的字号缩小：
```css
h1 { @apply font-inter font-semibold text-text-primary text-lg; }
h2 { @apply font-inter font-semibold text-text-primary text-base; }
h3 { @apply font-inter font-medium text-text-primary text-sm; }
h4 { @apply font-inter font-medium text-text-primary text-xs; }
h5 { @apply font-inter font-medium text-text-primary text-xs; }
h6 { @apply font-inter font-medium text-text-primary text-xs; }
```

- [ ] **Step 7: 提交**

```bash
git add src/style.css
git commit -m "style(global): Inter font, rounded window, lighter borders, compact typography"
```

---

### Task 4: 布局骨架 — AppLayout.vue 紧凑化

**Files:**
- Modify: `src/components/layout/AppLayout.vue`

- [ ] **Step 1: 缩小导航栏**

将 header 从 `h-16` 改为 `h-11`，内边距从 `px-8` 改为 `px-4`：
```html
<header
  class="h-11 border-b border-dark-border/50 bg-dark-panel flex items-center justify-between px-4 select-none"
  @mousedown="onHeaderMouseDown"
>
```

- [ ] **Step 2: 缩小窗口控制按钮**

将红黄绿按钮从 `w-8 h-8` 改为 `w-3 h-3`，间距从 `gap-2` 改为 `gap-1.5`，图标从 `h-4 w-4` 改为内部 SVG：
```html
<div class="flex items-center gap-1.5 mr-4">
  <button
    class="w-3 h-3 rounded-full bg-[#FF5F57] hover:brightness-110"
    @click="windowClose"
  ></button>
  <button
    class="w-3 h-3 rounded-full bg-[#FEBC2E] hover:brightness-110"
    @click="windowMinimize"
  ></button>
  <button
    class="w-3 h-3 rounded-full bg-[#28C840] hover:brightness-110"
    @click="windowMaximize"
  ></button>
</div>
```

- [ ] **Step 3: 缩小 Logo 字号**

将 `text-xl` 改为 `text-sm`：
```html
<h1 class="text-sm font-bold text-text-primary tracking-tight font-inter">Telepathy</h1>
```

- [ ] **Step 4: 缩小搜索框**

将搜索框从 `w-64` 改为 `w-48`，`px-4 py-2` 改为 `px-3 py-1`，`text-sm` 改为 `text-xs`，图标 `h-5 w-5` 改为 `h-3.5 w-3.5`：
```html
<input
  type="text"
  placeholder="搜索..."
  class="w-48 px-3 py-1 bg-dark-surface border border-dark-border/50 rounded-md text-text-primary placeholder-text-muted focus:outline-none focus:border-brand-500 focus:ring-1 focus:ring-brand-500/30 text-xs transition-all duration-300 pl-8"
>
<div class="absolute left-2.5 top-1/2 transform -translate-y-1/2 text-text-muted">
  <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
  </svg>
</div>
```

- [ ] **Step 5: 缩小导航图标按钮**

将通知和设置按钮从 `p-2` 改为 `p-1`，图标从 `h-5 w-5` 改为 `h-4 w-4`：
```html
<button class="p-1 relative hover:bg-dark-surface rounded-md transition-colors" @click="toggleNotifications">
  <svg ... class="h-4 w-4 text-text-secondary" ...>
```
```html
<button class="p-1 hover:bg-dark-surface rounded-md transition-colors" @click="() => router.push({ name: 'Settings' })">
  <svg ... class="h-4 w-4 text-text-secondary" ...>
```

未读圆点从 `top-2 right-2 w-3 h-3` 改为 `top-0.5 right-0.5 w-2 h-2`。

- [ ] **Step 6: 缩小内容区内边距**

将内容区从 `p-6` 改为 `p-4`：
```html
<div class="p-4 h-full box-border transition-all duration-300">
```

- [ ] **Step 7: 缩小浮动面板间距**

重新索引浮动面板（展开态）：
- `px-6 py-4` → `px-4 py-2.5`
- `min-w-[280px]` → `min-w-[240px]`
- spinner `h-6 w-6` → `h-4 w-4`
- `gap-3` → `gap-2`
- `text-sm` → `text-xs`
- 进度条 `h-1.5` → `h-1`
- 收起按钮 `px-4 py-2` → `px-3 py-1.5`，spinner `h-4 w-4` → `h-3 w-3`，`text-sm` → `text-[11px]`

后台任务浮动面板（展开态）：
- 同上比例缩小
- `min-w-[320px]` → `min-w-[260px]`

- [ ] **Step 8: 更新 scoped 样式**

移除或更新响应式媒体查询中的无效选择器：
```css
<style scoped>
@media (max-width: 768px) {
  .p-4 { padding: 0.75rem; }
}
</style>
```

- [ ] **Step 9: 提交**

```bash
git add src/components/layout/AppLayout.vue
git commit -m "style(layout): compact header, smaller controls, tighter spacing"
```

---

### Task 5: 布局骨架 — Sidebar.vue 紧凑化

**Files:**
- Modify: `src/components/layout/Sidebar.vue`

- [ ] **Step 1: 缩小侧边栏宽度**

将展开/折叠宽度从 `240px/72px` 改为 `220px/56px`：
```html
:style="{ width: appStore.isSidebarCollapsed ? '56px' : '220px' }"
```

- [ ] **Step 2: 缩小 Logo 区域**

将 Logo 区 `p-5` 改为 `p-3`，字号 `text-xl` 改为 `text-sm`，`font-black` 改为 `font-bold`：
```html
<div class="flex items-center p-3 border-b border-dark-border/50">
  <span class="font-bold text-sm text-text-primary font-inter truncate tracking-tight">
    {{ appStore.isSidebarCollapsed ? 'T' : 'Telepathy' }}
  </span>
</div>
```

- [ ] **Step 3: 缩小导航区域**

将导航区 `p-5` 改为 `p-2`，菜单项间距 `space-y-3` 改为 `space-y-0.5`：
```html
<nav class="flex-1 overflow-y-auto min-h-0 p-2">
  <ul class="space-y-0.5">
```

- [ ] **Step 4: 缩小菜单按钮**

将菜单按钮 `px-3 py-3` 改为 `px-2 py-1.5`，`gap-3` 改为 `gap-2`，图标 `text-xl` 改为 `text-sm`（内含 SVG `width="20" height="20"` 改为 `width="16" height="16"`），菜单文字 `text-sm` 改为 `text-xs`：
```html
<button
  class="relative flex items-center gap-2 px-2 py-1.5 w-full rounded-md transition-colors"
  :class="[
    route.name === item.key ? 'bg-brand-500/10 text-brand-400' : 'text-text-secondary hover:bg-dark-surface hover:text-text-primary',
    appStore.isSidebarCollapsed ? 'justify-center' : 'justify-start'
  ]"
  @click="handleMenuClick(item.key)"
>
  <span v-if="route.name === item.key && !appStore.isSidebarCollapsed" class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-4 bg-brand-500 rounded-r-full"></span>
  <span class="text-sm flex-shrink-0">
    <svg ... width="16" height="16" ...>
```

菜单文字：
```html
<span v-if="!appStore.isSidebarCollapsed" class="font-medium text-xs text-text-secondary">
```

- [ ] **Step 5: 缩小底部折叠按钮**

将底部区域 `p-4` 改为 `p-2`，按钮 `px-4 py-2.5` 改为 `px-2 py-1.5`，`gap-3` 改为 `gap-2`，图标 SVG `width="20" height="20"` 改为 `width="16" height="16"`，文字 `text-sm` 改为 `text-xs`：
```html
<div class="p-2 border-t border-dark-border/50">
  <button
    class="flex items-center justify-center gap-2 px-2 py-1.5 w-full rounded-md hover:bg-dark-surface transition-colors"
    @click="toggleSidebar"
  >
    <svg ... width="16" height="16" ...>
```

- [ ] **Step 6: 移除毛玻璃类**

将容器上的 `glass` 类改为 `bg-dark-panel`（透明窗口下不用毛玻璃）：
```html
<div
  :class="['sidebar', { 'collapsed': appStore.isSidebarCollapsed }]"
  class="bg-dark-panel border-r border-dark-border/50 transition-all duration-300 flex flex-col h-full"
```

- [ ] **Step 7: 提交**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "style(sidebar): compact width, tighter spacing, smaller icons"
```

---

### Task 6: 视图组件 — NotificationPanel.vue 紧凑化

**Files:**
- Modify: `src/components/layout/NotificationPanel.vue`

- [ ] **Step 1: 全局缩小间距和字号**

在该文件中进行以下替换：
- `p-4` → `p-3`（容器内边距）
- `gap-3` → `gap-2`（组件间距）
- `text-sm` → `text-xs`（正文）
- `text-lg` → `text-sm`（标题）
- `rounded-lg` → `rounded-md`（卡片圆角）
- `border-dark-border` → `border-dark-border/50`（边框更轻）
- `w-96` → `w-80`（面板宽度）
- `max-h-[400px]` → `max-h-[320px]`（最大高度）

- [ ] **Step 2: 提交**

```bash
git add src/components/layout/NotificationPanel.vue
git commit -m "style(notification): compact spacing and typography"
```

---

### Task 7: 视图组件 — KnowledgeBase.vue 紧凑化

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 缩小间距和字号**

在该文件中进行以下替换：
- `p-5` → `p-3`（卡片内边距）
- `p-4` → `p-3`（容器内边距）
- `px-5` → `px-3`
- `px-6` → `px-4`
- `py-8` → `py-4`
- `py-16` → `py-8`
- `gap-3` → `gap-2`
- `gap-4` → `gap-3`
- `gap-8` → `gap-4`
- `space-y-4` → `space-y-3`
- `space-y-5` → `space-y-3`
- `mb-3` → `mb-2`
- `mb-4` → `mb-3`
- `mb-6` → `mb-4`
- `mt-3` → `mt-2`
- `text-sm` → `text-xs`（正文和辅助文字）
- `text-base` → `text-sm`
- `text-lg` → `text-sm`
- `text-3xl` → `text-xl`（页面主标题）
- `text-4xl` → `text-2xl`
- `text-5xl` → `text-3xl`
- `rounded-lg` → `rounded-md`（按钮和输入框）
- `border-dark-border` → `border-dark-border/50`
- `min-w-[300px]` → `min-w-[240px]`
- `min-w-[400px]` → `min-w-[320px]`
- `max-w-[400px]` → `max-w-[320px]`

- [ ] **Step 2: 提交**

```bash
git add src/views/KnowledgeBase.vue
git commit -m "style(knowledge-base): compact spacing and typography"
```

---

### Task 8: 视图组件 — Chat.vue 紧凑化

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: 缩小间距和字号**

在该文件中进行以下替换：
- `p-4` → `p-3`
- `p-5` → `p-3`
- `p-6` → `p-4`
- `p-8` → `p-4`
- `px-3` → `px-2.5`
- `px-4` → `px-3`
- `px-5` → `px-3`
- `px-6` → `px-4`
- `px-8` → `px-4`
- `py-2` → `py-1.5`
- `py-3` → `py-2`
- `py-5` → `py-3`
- `py-8` → `py-4`
- `py-16` → `py-8`
- `pt-4` → `pt-3`
- `pt-6` → `pt-4`
- `gap-3` → `gap-2`
- `gap-6` → `gap-3`
- `space-y-2` → `space-y-1.5`
- `space-y-8` → `space-y-4`
- `mb-4` → `mb-3`
- `mt-3` → `mt-2`
- `mt-4` → `mt-3`
- `mt-32` → `mt-20`
- `text-sm` → `text-xs`
- `text-base` → `text-sm`
- `text-lg` → `text-sm`
- `text-2xl` → `text-lg`
- `text-3xl` → `text-xl`
- `text-5xl` → `text-2xl`
- `rounded-lg` → `rounded-md`（按钮和输入框）
- `rounded-xl` → `rounded-lg`（卡片和面板）
- `border-dark-border` → `border-dark-border/50`
- `w-64` → `w-52`
- `w-72` → `w-60`
- `max-h-[300px]` → `max-h-[240px]`
- `max-h-80` → `max-h-64`
- `max-w-4xl` → `max-w-3xl`
- `min-h-[100px]` → `min-h-[80px]`

- [ ] **Step 2: 提交**

```bash
git add src/views/Chat.vue
git commit -m "style(chat): compact spacing and typography"
```

---

### Task 9: 视图组件 — Documents.vue 紧凑化

**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 缩小间距和字号**

在该文件中进行以下替换：
- `p-2` → `p-1.5`
- `p-6` → `p-4`
- `p-12` → `p-8`
- `px-3` → `px-2.5`
- `px-4` → `px-3`
- `px-5` → `px-3`
- `px-6` → `px-4`
- `py-1` → 保持
- `py-1.5` → 保持
- `py-2` → `py-1.5`
- `py-4` → `py-3`
- `py-12` → `py-8`
- `pt-4` → `pt-3`
- `gap-3` → `gap-2`
- `gap-4` → `gap-3`
- `gap-6` → `gap-4`
- `space-y-6` → `space-y-4`
- `mb-2` → 保持
- `mb-3` → `mb-2`
- `mb-4` → `mb-3`
- `mb-6` → `mb-4`
- `mb-8` → `mb-5`
- `mt-2` → `mt-1.5`
- `mt-4` → `mt-3`
- `text-sm` → `text-xs`
- `text-lg` → `text-sm`
- `text-xl` → `text-base`
- `text-2xl` → `text-lg`
- `text-3xl` → `text-xl`
- `text-7xl` → `text-5xl`
- `rounded-lg` → `rounded-md`（按钮和输入框）
- `rounded-xl` → `rounded-lg`（对话框和拖拽区）
- `border-dark-border` → `border-dark-border/50`
- `w-48` → `w-40`

- [ ] **Step 2: 提交**

```bash
git add src/views/Documents.vue
git commit -m "style(documents): compact spacing and typography"
```

---

### Task 10: 视图组件 — Profile.vue 紧凑化

**Files:**
- Modify: `src/views/Profile.vue`

- [ ] **Step 1: 缩小间距和字号**

在该文件中进行以下替换：
- `p-6` → `p-4`
- `px-3` → `px-2.5`
- `px-5` → `px-3`
- `py-1.5` → 保持
- `py-2` → `py-1.5`
- `py-16` → `py-8`
- `pb-5` → `pb-4`
- `pb-6` → `pb-4`
- `gap-2` → 保持
- `gap-4` → `gap-3`
- `space-y-2` → 保持
- `space-y-8` → `space-y-4`
- `mb-2` → 保持
- `mb-3` → `mb-2`
- `mb-4` → `mb-3`
- `mt-2` → `mt-1.5`
- `mt-6` → `mt-4`
- `text-sm` → `text-xs`
- `text-lg` → `text-sm`
- `text-3xl` → `text-xl`
- `rounded-lg` → `rounded-md`
- `border-dark-border` → `border-dark-border/50`
- `w-12 h-12` → `w-10 h-10`（头像）
- `max-w-[800px]` → `max-w-[680px]`

- [ ] **Step 2: 提交**

```bash
git add src/views/Profile.vue
git commit -m "style(profile): compact spacing and typography"
```

---

### Task 11: 视图组件 — Settings.vue 紧凑化

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: 缩小间距和字号**

在该文件中进行以下替换：
- `p-5` → `p-3`
- `p-6` → `p-4`
- `p-7` → `p-4`
- `p-10` → `p-6`
- `px-4` → `px-3`
- `px-5` → `px-3`
- `px-6` → `px-4`
- `py-2` → `py-1.5`
- `py-3` → `py-2`
- `py-6` → `py-4`
- `py-16` → `py-8`
- `pb-5` → `pb-4`
- `pb-6` → `pb-4`
- `pt-4` → `pt-3`
- `gap-3` → `gap-2`
- `gap-4` → `gap-3`
- `gap-5` → `gap-3`
- `space-y-4` → `space-y-3`
- `mb-2` → 保持
- `mb-3` → `mb-2`
- `mb-4` → `mb-3`
- `mb-5` → `mb-3`
- `mb-6` → `mb-4`
- `mb-7` → `mb-4`
- `mb-8` → `mb-5`
- `mt-2` → `mt-1.5`
- `mt-3` → `mt-2`
- `mt-4` → `mt-3`
- `mt-6` → `mt-4`
- `mt-8` → `mt-5`
- `my-3` → `my-2`
- `text-sm` → `text-xs`
- `text-base` → `text-sm`
- `text-lg` → `text-sm`
- `text-xl` → `text-base`
- `text-2xl` → `text-lg`
- `text-3xl` → `text-xl`
- `text-5xl` → `text-3xl`
- `rounded-lg` → `rounded-md`（按钮和输入框）
- `rounded-xl` → `rounded-lg`（卡片和面板）
- `border-dark-border` → `border-dark-border/50`
- `w-5 h-5` → `w-4 h-4`（小图标）
- `w-8 h-8` → `w-6 h-6`
- `w-10 h-10` → `w-8 h-8`
- `h-2` → 保持（滑块）
- `h-3` → 保持
- `h-5` → `h-4`
- `h-8` → `h-6`
- `h-10` → `h-8`
- `h-12` → `h-10`
- `max-w-lg` → `max-w-md`
- `max-w-xl` → `max-w-lg`
- `max-w-[900px]` → `max-w-[720px]`

- [ ] **Step 2: 提交**

```bash
git add src/views/Settings.vue
git commit -m "style(settings): compact spacing and typography"
```

---

### Task 12: 子组件 — UserProfilePanel + BatchImportDialog 紧凑化

**Files:**
- Modify: `src/components/profile/UserProfilePanel.vue`
- Modify: `src/components/batch/BatchImportDialog.vue`

- [ ] **Step 1: UserProfilePanel.vue 缩小间距**

替换：
- `px-3` → `px-2.5`
- `py-4` → `py-3`
- `gap-4` → `gap-3`
- `space-y-4` → `space-y-3`
- `text-sm` → `text-xs`
- `text-lg` → `text-sm`

- [ ] **Step 2: BatchImportDialog.vue 缩小间距**

替换：
- `p-3` → `p-2.5`
- `p-6` → `p-4`
- `px-4` → `px-3`
- `py-2` → `py-1.5`
- `pt-4` → `pt-3`
- `gap-3` → `gap-2`
- `mb-2` → 保持
- `mb-4` → `mb-3`
- `mt-2` → `mt-1.5`
- `mt-4` → `mt-3`
- `mt-6` → `mt-4`
- `text-sm` → `text-xs`
- `text-lg` → `text-sm`
- `text-xl` → `text-base`
- `rounded-lg` → `rounded-md`
- `rounded-xl` → `rounded-lg`
- `border-dark-border` → `border-dark-border/50`
- `max-w-lg` → `max-w-md`
- `max-h-[80vh]` → `max-h-[70vh]`
- `max-h-64` → `max-h-56`

- [ ] **Step 3: 提交**

```bash
git add src/components/profile/UserProfilePanel.vue src/components/batch/BatchImportDialog.vue
git commit -m "style(components): compact UserProfilePanel and BatchImportDialog"
```

---

### Task 13: 验证 — TypeScript 检查

- [ ] **Step 1: 运行 TypeScript 类型检查**

```bash
cd /sessions/69d7be74a5234068df3dcc36/workspace && npx vue-tsc --noEmit
```

Expected: 无类型错误（样式改动不影响类型）

- [ ] **Step 2: 如有错误则修复并提交**

---

## 自检清单

| 设计规格 | 对应 Task | 状态 |
|----------|-----------|------|
| transparent: true 窗口配置 | Task 1 | ✅ |
| Inter 字体引入 | Task 2 + 3 | ✅ |
| 颜色微调 dark-border/dark-surface | Task 2 | ✅ |
| 根容器圆角 + 阴影 | Task 3 | ✅ |
| 边框 0.5px + 更轻 | Task 3 | ✅ |
| 移除毛玻璃 | Task 3 + 5 | ✅ |
| 导航栏 h-11 + px-4 | Task 4 | ✅ |
| 窗口控制按钮 w-3 h-3 | Task 4 | ✅ |
| 内容区 p-4 | Task 4 | ✅ |
| 侧边栏 220px/56px | Task 5 | ✅ |
| 菜单间距 space-y-0.5 | Task 5 | ✅ |
| 图标 16x16 | Task 5 | ✅ |
| KnowledgeBase 紧凑化 | Task 7 | ✅ |
| Chat 紧凑化 | Task 8 | ✅ |
| Documents 紧凑化 | Task 9 | ✅ |
| Profile 紧凑化 | Task 10 | ✅ |
| Settings 紧凑化 | Task 11 | ✅ |
| 子组件紧凑化 | Task 12 | ✅ |
