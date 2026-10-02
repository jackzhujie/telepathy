# 暗色现代风 UI 重设计 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Telepathy 全部页面从浅色主题重新设计为暗色现代风格（「深空知识」概念，蓝色系品牌色）

**Architecture:** 仅修改视觉层（Tailwind 配置、全局样式、组件模板中的 CSS class），不改变任何功能逻辑、Store、Router、API 层。通过 Tailwind 自定义色板 + CSS 变量统一管理暗色主题。

**Tech Stack:** Vue 3 + Tailwind CSS + reka-ui + highlight.js

---

### Task 1: 扩展 Tailwind 配置 + 全局暗色样式

**Files:**
- Modify: `tailwind.config.js`
- Modify: `src/style.css`
- Modify: `index.html`

- [ ] **Step 1: 更新 tailwind.config.js**

将 `tailwind.config.js` 的完整内容替换为：

```js
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // 暗色主题色板
        'dark': {
          'bg': '#0a0a0f',
          'panel': '#111118',
          'surface': '#1a1a24',
          'border': '#1e1e2e',
          'border-light': '#2a2a3a',
        },
        // 品牌色
        'brand': {
          DEFAULT: '#3b82f6',
          'indigo': '#6366f1',
        },
        // 语义色
        'success': '#22c55e',
        'warning': '#f97316',
        'danger': '#ef4444',
        // 文字色
        'text': {
          'primary': '#e2e8f0',
          'secondary': '#94a3b8',
          'muted': '#64748b',
        },
      },
      fontFamily: {
        'display': ['"Plus Jakarta Sans"', 'sans-serif'],
        'body': ['-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'sans-serif'],
        'mono': ['"JetBrains Mono"', '"Fira Code"', 'monospace'],
      },
      borderRadius: {
        'sm': '6px',
        'md': '8px',
        'lg': '12px',
        'xl': '16px',
      },
      backdropBlur: {
        'glass': '12px',
      },
    },
  },
  plugins: [],
}
```

- [ ] **Step 2: 更新 src/style.css**

将 `src/style.css` 的完整内容替换为：

```css
@import url('https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@300;400;500;600;700;900&display=swap');

@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  --dark-bg: #0a0a0f;
  --dark-panel: #111118;
  --dark-surface: #1a1a24;
  --dark-border: #1e1e2e;
  --dark-border-light: #2a2a3a;
  --brand: #3b82f6;
  --brand-indigo: #6366f1;
  --text-primary: #e2e8f0;
  --text-secondary: #94a3b8;
  --text-muted: #64748b;
}

@layer base {
  body {
    @apply font-body text-text-primary bg-dark-bg min-h-screen w-screen m-0 p-0;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  h1, h2, h3, h4, h5, h6 {
    @apply font-display;
  }

  #app {
    @apply w-full h-full;
  }

  /* 全局滚动条 */
  ::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }

  ::-webkit-scrollbar-track {
    background: transparent;
  }

  ::-webkit-scrollbar-thumb {
    background: #2a2a3a;
    border-radius: 3px;
  }

  ::-webkit-scrollbar-thumb:hover {
    background: #3a3a4a;
  }

  /* Firefox 滚动条 */
  * {
    scrollbar-width: thin;
    scrollbar-color: #2a2a3a transparent;
  }
}

@layer components {
  /* 玻璃效果 */
  .glass {
    background: rgba(17, 17, 24, 0.8);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }

  .glass-light {
    background: rgba(17, 17, 24, 0.6);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }

  /* 品牌渐变 */
  .brand-gradient {
    background: linear-gradient(135deg, #3b82f6, #6366f1);
  }

  /* 暗色输入框 */
  .dark-input {
    @apply w-full px-4 py-2 bg-dark-surface border border-dark-border-light rounded-md text-text-primary placeholder-text-muted focus:outline-none focus:border-brand focus:ring-1 focus:ring-brand/30 transition-all duration-200;
  }

  /* 暗色卡片 */
  .dark-card {
    @apply bg-dark-panel border border-dark-border rounded-lg;
  }

  /* 暗色按钮 */
  .btn-brand {
    @apply px-4 py-2 brand-gradient text-white rounded-md font-medium hover:opacity-90 transition-all duration-200;
  }

  .btn-ghost {
    @apply px-4 py-2 border border-dark-border-light text-text-secondary rounded-md hover:bg-dark-surface hover:text-text-primary transition-all duration-200;
  }
}
```

- [ ] **Step 3: 更新 index.html**

将 `index.html` 的完整内容替换为：

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Telepathy - AI 知识库</title>
  </head>

  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 4: 提交**

```bash
git add tailwind.config.js src/style.css index.html
git commit -m "style: 配置暗色主题 Tailwind 色板和全局样式"
```

---

### Task 2: 侧边栏暗色化

**Files:**
- Modify: `src/components/layout/Sidebar.vue`

- [ ] **Step 1: 重写 Sidebar.vue 模板样式**

将 `<template>` 部分替换为以下内容（`<script setup>` 保持不变）：

```vue
<template>
  <div
    :class="['sidebar', { 'collapsed': appStore.isSidebarCollapsed }]"
    class="glass border-r border-dark-border transition-all duration-300 flex flex-col h-full"
    :style="{ width: appStore.isSidebarCollapsed ? '72px' : '240px' }"
    v-motion
    :initial="{ opacity: 0, x: -20 }"
    :enter="{ opacity: 1, x: 0 }"
  >
    <div
      class="flex items-center px-6 py-5 border-b border-dark-border"
      v-motion
      :initial="{ opacity: 0 }"
      :enter="{ opacity: 1 }"
      :delay="100"
    >
      <span class="font-black text-xl text-text-primary truncate tracking-tight font-display">
        {{ appStore.isSidebarCollapsed ? 'T' : 'Telepathy' }}
      </span>
    </div>

    <nav class="flex-1 overflow-y-auto min-h-0 p-4">
      <ul class="space-y-1">
        <li
          v-for="(item, index) in menuItems"
          :key="item.key"
          v-motion
          :initial="{ opacity: 0, x: -20 }"
          :enter="{ opacity: 1, x: 0 }"
          :delay="150 + index * 50"
        >
          <button
            class="flex items-center gap-3 px-4 py-3 w-full justify-start rounded-md transition-all duration-200 relative"
            :class="route.name === item.key
              ? 'bg-brand/10 text-brand'
              : 'text-text-secondary hover:bg-dark-surface hover:text-text-primary'"
            @click="handleMenuClick(item.key)"
          >
            <!-- Active 指示条 -->
            <div
              v-if="route.name === item.key"
              class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-5 bg-brand rounded-r-full"
            ></div>
            <span class="text-xl flex-shrink-0">
              <svg v-if="item.icon === 'book'" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
              </svg>
              <svg v-else-if="item.icon === 'chat'" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
              <svg v-else-if="item.icon === 'document'" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
                <polyline points="10 9 9 9 8 9"/>
              </svg>
              <svg v-else-if="item.icon === 'user'" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
              </svg>
            </span>
            <span v-if="!appStore.isSidebarCollapsed" class="font-medium text-sm">
              {{ item.label }}
            </span>
          </button>
        </li>
      </ul>
    </nav>

    <div class="p-4 border-t border-dark-border">
      <button
        class="flex items-center justify-center gap-3 px-4 py-3 w-full rounded-md text-text-secondary hover:bg-dark-surface hover:text-text-primary transition-all duration-200"
        @click="toggleSidebar"
        v-motion
        :initial="{ opacity: 0 }"
        :enter="{ opacity: 1 }"
        :delay="300"
      >
        <svg v-if="appStore.isSidebarCollapsed" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
        <span v-if="!appStore.isSidebarCollapsed" class="text-sm font-medium">
          收起
        </span>
      </button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "style: 侧边栏暗色化 - 玻璃效果 + 蓝色活跃指示条"
```

---

### Task 3: 顶部导航 + 主布局暗色化

**Files:**
- Modify: `src/components/layout/AppLayout.vue`

- [ ] **Step 1: 更新 AppLayout.vue 模板中的样式 class**

在 `<template>` 中进行以下替换（`<script setup>` 部分完全不动）：

1. 根 div: `class="flex h-screen w-screen bg-gray-50"` → `class="flex h-screen w-screen bg-dark-bg"`

2. header: `class="h-20 border-b border-gray-200 bg-white flex items-center justify-between px-8"` → `class="h-16 border-b border-dark-border glass-light flex items-center justify-between px-8"`

3. 标题: `class="text-2xl font-black text-gray-800 tracking-tight"` → `class="text-xl font-bold text-text-primary tracking-tight font-display"`

4. 搜索框: `class="w-64 px-4 py-2 border-2 border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all duration-300 pl-10"` → `class="w-64 px-4 py-2 bg-dark-surface border border-dark-border-light rounded-md text-text-primary placeholder-text-muted focus:outline-none focus:border-brand focus:ring-1 focus:ring-brand/30 transition-all duration-200 pl-10 text-sm"`

5. 搜索图标: `class="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-500"` → `class="absolute left-3 top-1/2 transform -translate-y-1/2 text-text-muted"`

6. 通知按钮: `class="p-2 relative hover:bg-gray-100 rounded-lg transition-colors"` → `class="p-2 relative hover:bg-dark-surface rounded-md transition-colors"`

7. 通知图标: `class="h-6 w-6 text-gray-600"` → `class="h-5 w-5 text-text-secondary"`

8. 设置按钮: `class="p-2 hover:bg-gray-100 rounded-lg transition-colors"` → `class="p-2 hover:bg-dark-surface rounded-md transition-colors"`

9. 设置图标: `class="h-6 w-6 text-gray-600"` → `class="h-5 w-5 text-text-secondary"`

10. 路由视图容器: `class="flex-1 overflow-auto"` 保持不变，内层 `class="p-8 h-full box-border transition-all duration-300"` → `class="p-6 h-full box-border transition-all duration-300"`

11. 重新索引展开态: `class="px-6 py-4 bg-orange-600 text-white rounded-xl shadow-2xl..."` → `class="px-5 py-3 bg-dark-panel/90 backdrop-blur-xl border border-dark-border text-white rounded-xl shadow-2xl..."`，内部文字颜色相应调整

12. 重新索引收起态: `class="px-4 py-2 bg-orange-600 text-white rounded-full..."` → `class="px-4 py-2 bg-dark-panel/90 backdrop-blur-xl border border-dark-border rounded-full..."`，添加橙色图标强调

13. 批量索引展开态: 同重新索引，蓝色改为品牌色

14. 批量索引收起态: 同重新索引

- [ ] **Step 2: 提交**

```bash
git add src/components/layout/AppLayout.vue
git commit -m "style: 顶部导航和浮动面板暗色化"
```

---

### Task 4: 通知面板暗色化

**Files:**
- Modify: `src/components/layout/NotificationPanel.vue`

- [ ] **Step 1: 重写 NotificationPanel.vue 模板样式**

替换所有浅色 class：
- 外层容器: `bg-white` → `bg-dark-panel border-dark-border`
- 标题: `text-gray-800` → `text-text-primary`
- 通知项 hover: `hover:bg-gray-50` → `hover:bg-dark-surface`
- 未读背景: `bg-blue-50` → `bg-brand/5`
- 文字颜色: `text-gray-800` → `text-text-primary`，`text-gray-600` → `text-text-secondary`，`text-gray-500` → `text-text-muted`
- 分隔线: `border-gray-200` → `border-dark-border`
- 滚动条: `#e5e7eb` → `#2a2a3a`

- [ ] **Step 2: 提交**

```bash
git add src/components/layout/NotificationPanel.vue
git commit -m "style: 通知面板暗色化"
```

---

### Task 5: 知识库页面暗色化

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 替换 KnowledgeBase.vue 中所有浅色 class**

关键替换：
- 页面背景: `bg-white` → `bg-dark-bg`
- 标题: `text-primary` → `text-text-primary`
- Select 触发器: `bg-white border-gray-300` → `bg-dark-surface border-dark-border-light text-text-primary`
- Select 下拉: `bg-white border-gray-200` → `bg-dark-panel border-dark-border`
- Select 项: `hover:bg-blue-50 data-[highlighted]:bg-blue-100` → `hover:bg-dark-surface data-[highlighted]:bg-brand/10`
- 搜索框: 同 Select 触发器样式
- 搜索按钮: `bg-blue-600` → `brand-gradient`
- 文档卡片: `border-gray-200` → `border-dark-border`，`hover:shadow-md` → `hover:shadow-lg hover:shadow-brand/5 hover:border-brand/30 hover:-translate-y-0.5`
- 选中卡片: `border-blue-600 bg-blue-50` → `border-brand bg-brand/5`
- 文件名: `text-gray-800` → `text-text-primary`
- 文件类型标签: 使用彩色编码方案（.md 蓝、.pdf 红、.txt 灰、.json 绿）
- 文本块数: `text-blue-600` → `text-brand`
- 时间: `text-gray-600` → `text-text-muted`
- 空状态: `text-gray-400/500` → `text-text-muted/secondary`
- 文档详情区: `border-border` → `border-dark-border`，`text-gray-500` → `text-text-secondary`
- 文本块卡片: `border-gray-200` → `border-dark-border`，`text-gray-700` → `text-text-secondary`
- 块标签: `bg-blue-100 text-blue-800` → `bg-brand/15 text-brand`

- [ ] **Step 2: 提交**

```bash
git add src/views/KnowledgeBase.vue
git commit -m "style: 知识库页面暗色化"
```

---

### Task 6: AI 对话页面暗色化

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: 替换 Chat.vue 模板中的浅色 class**

关键替换：
- 页面背景: `bg-gray-50` → `bg-dark-bg`
- 对话列表侧边栏: `bg-white border-gray-200` → `bg-dark-panel border-dark-border`
- 新建对话按钮: `bg-blue-600` → `brand-gradient`
- 对话项 active: `bg-blue-50 border-l-4 border-blue-600` → `bg-brand/10 border-l-4 border-brand`
- 对话项 hover: `hover:bg-gray-100` → `hover:bg-dark-surface`
- 对话文字: `text-gray-800` → `text-text-primary`
- 聊天区域背景: `bg-gray-50` → `bg-dark-bg`
- 顶部分隔线: `border-gray-200` → `border-dark-border`
- 标题: `text-gray-800` → `text-text-primary`
- 侧边栏切换按钮: `hover:bg-gray-100` → `hover:bg-dark-surface`，图标 `text-gray-700` → `text-text-secondary`
- Select 组件: 同知识库页面暗色化
- 用户消息: `bg-blue-600 text-white` → `brand-gradient text-white`
- AI 消息: `bg-white border-gray-200` → `bg-dark-panel border-dark-border`
- 引用来源面板: `bg-white border-gray-200` → `bg-dark-panel border-dark-border`
- 来源卡片: `bg-white border-gray-200` → `bg-dark-surface border-dark-border`
- 来源标签: `bg-blue-50 text-blue-600` → `bg-brand/15 text-brand`
- 输入框: `border-gray-300 bg-white` → `border-dark-border-light bg-dark-surface text-text-primary placeholder-text-muted focus:border-brand`
- 发送按钮: `bg-blue-600` → `brand-gradient`
- 空状态: `text-gray-500` → `text-text-muted`

- [ ] **Step 2: 更新 Chat.vue 的 `<style scoped>` 部分**

替换 markdown-body 样式为暗色版本：
- `color: #1e293b` → `color: #e2e8f0`
- `background-color: rgba(0, 0, 0, 0.05)` → `background-color: rgba(255, 255, 255, 0.05)`
- `.hljs-code-block`: `background-color: rgba(0, 0, 0, 0.03); border: 2px solid #e2e8f0` → `background-color: #0a0a0f; border: 1px solid #1e1e2e`
- `blockquote border-left: 4px solid #2563eb; color: #64748b` → 保持蓝色边框，`color: #94a3b8`
- `th background-color: #f1f5f9` → `background-color: #1a1a24`
- `th, td border: 2px solid #e2e8f0` → `border: 1px solid #1e1e2e`
- `a color: #2563eb` → `color: #60a5fa`
- 滚动条: `background: #f1f5f9` → `transparent`，`background: #cbd5e1` → `#2a2a3a`，`hover: #94a3b8` → `#3a3a4a`

- [ ] **Step 3: 提交**

```bash
git add src/views/Chat.vue
git commit -m "style: AI 对话页面暗色化 - 含 markdown 暗色渲染"
```

---

### Task 7: 文档管理页面暗色化

**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 替换 Documents.vue 中所有浅色 class**

关键替换：
- 标题: `text-gray-800` → `text-text-primary`
- 分隔线: `border-gray-200` → `border-dark-border`
- Select 组件: 同其他页面暗色化
- 按钮组: `bg-blue-600` → `brand-gradient`，`border-gray-300 hover:bg-gray-100` → `border-dark-border-light hover:bg-dark-surface text-text-secondary`
- 删除按钮: `border-red-500 text-red-500 hover:bg-red-50` → `border-danger/50 text-danger hover:bg-danger/10`
- Sidecar 警告: `bg-blue-50 border-blue-200` → `bg-brand/5 border-brand/20`，文字 `text-blue-800/700/600` → `text-brand`
- 无项目警告: `bg-yellow-50 border-yellow-200` → `bg-warning/5 border-warning/20`，文字 `text-yellow-800/700` → `text-warning`
- 后台索引提示: `bg-blue-50 border-blue-200` → `bg-brand/5 border-brand/20`
- 项目信息卡片: `bg-white border-gray-200` → `bg-dark-panel border-dark-border`
- 拖拽区域: `border-gray-300 hover:border-gray-400 hover:bg-gray-50` → `border-dark-border-light hover:border-brand/50 hover:bg-brand/5`，拖入时 `border-blue-600 bg-blue-50` → `border-brand bg-brand/5`
- 表格: `bg-white border-gray-200` → `bg-dark-panel border-dark-border`
- 表头: `bg-gray-50 border-gray-200` → `bg-dark-surface border-dark-border`，文字 `text-gray-600` → `text-text-muted`
- 表格行: `hover:bg-gray-50` → `hover:bg-dark-surface/50`
- 表格文字: `text-gray-800` → `text-text-primary`
- 状态标签: 更新为暗色版本
  - pending: `bg-gray-100 text-gray-800` → `bg-dark-surface text-text-muted`
  - parsing: `bg-blue-100 text-blue-800` → `bg-brand/15 text-brand`
  - done: `bg-green-100 text-green-800` → `bg-success/15 text-success`
  - error: `bg-red-100 text-red-800` → `bg-danger/15 text-danger`
  - indexed: `bg-green-100 text-green-800` → `bg-brand-indigo/15 text-brand-indigo`
- 操作按钮: `border-gray-300 hover:bg-gray-100` → `border-dark-border-light hover:bg-dark-surface text-text-secondary`
- 索引按钮: `border-blue-200 text-blue-800 hover:bg-blue-100` → `border-brand/30 text-brand hover:bg-brand/10`
- 删除按钮: `border-red-200 text-red-800 hover:bg-red-100` → `border-danger/30 text-danger hover:bg-danger/10`
- Tooltip: `bg-gray-800` → `bg-dark-surface border border-dark-border`
- Toast: `bg-gray-800` → `bg-dark-panel border border-dark-border`
- Dialog 弹窗: `bg-white` → `bg-dark-panel border border-dark-border`
- Dialog 输入框: 同 dark-input 样式
- Card 组件: `bg-white border-gray-200` → `bg-dark-panel border-dark-border`
- 文件类型标签: `bg-gray-100 text-gray-800` → `bg-dark-surface text-text-secondary`

- [ ] **Step 2: 提交**

```bash
git add src/views/Documents.vue
git commit -m "style: 文档管理页面暗色化"
```

---

### Task 8: 用户信息 + 设置页面暗色化

**Files:**
- Modify: `src/views/Profile.vue`
- Modify: `src/views/Settings.vue`
- Modify: `src/components/profile/UserProfilePanel.vue`

- [ ] **Step 1: 替换 Profile.vue 中所有浅色 class**

关键替换：
- 标题: `text-primary` → `text-text-primary`
- 分隔线: `border-border` → `border-dark-border`
- 描述: `text-muted-foreground` → `text-text-secondary`
- 区块卡片: `border-gray-200 bg-white` → `border-dark-border bg-dark-panel`
- 区块标题: `text-gray-800` → `text-text-primary`
- 标签: `text-gray-700` → `text-text-secondary`
- 输入框: `border-input bg-background` → `bg-dark-surface border-dark-border-light text-text-primary placeholder-text-muted`
- 兴趣标签: `bg-secondary text-secondary-foreground` → `bg-brand/15 text-brand`
- 添加按钮: `bg-primary text-primary-foreground` → `brand-gradient text-white`

- [ ] **Step 2: 替换 Settings.vue 中所有浅色 class**

关键替换：
- 标题: `text-primary` → `text-text-primary`
- 分隔线: `border-border` → `border-dark-border`
- 未安装状态卡片: `border-gray-200` → `border-dark-border`，`text-gray-800` → `text-text-primary`，`text-gray-600` → `text-text-secondary`
- 系统信息卡片: `border-gray-200` → `border-dark-border`
- 进度条: `bg-gray-200` → `bg-dark-surface`，填充 `bg-cta` → `brand-gradient`
- 安装按钮: `bg-blue-600` → `brand-gradient`
- 其他按钮: `border-gray-300 hover:bg-gray-100 text-gray-800` → `border-dark-border-light hover:bg-dark-surface text-text-secondary`
- 无模型状态: `border-amber-200 bg-amber-50/50` → `border-warning/20 bg-warning/5`
- 推荐模型卡片: `border-gray-200 hover:border-blue-600` → `border-dark-border hover:border-brand/50`
- Accordion 面板: `border-gray-200 bg-white hover:shadow-md` → `border-dark-border bg-dark-panel hover:border-dark-border-light`
- Accordion 头: `hover:bg-gray-50` → `hover:bg-dark-surface`
- Accordion 标题: `text-gray-800` → `text-text-primary`
- Select/Slider/Progress: 同其他页面暗色化
- 模型标签: `bg-gray-100 text-gray-800` → `bg-dark-surface text-text-secondary`
- AlertDialog: `bg-white` → `bg-dark-panel border border-dark-border`
- Toast: `bg-primary` → `bg-dark-panel border border-dark-border`
- 安装成功: `bg-green-50 border-green-200` → `bg-success/5 border-success/20`，文字 `text-green-700/600` → `text-success`
- 引导卡片: `bg-white border-gray-200` → `bg-dark-panel border-dark-border`

- [ ] **Step 3: 替换 UserProfilePanel.vue 中所有浅色 class**

同 Profile.vue 的替换规则，另外：
- AccordionTrigger: `text-muted-foreground` → `text-text-secondary`
- 输入框/标签: 同 Profile.vue

- [ ] **Step 4: 提交**

```bash
git add src/views/Profile.vue src/views/Settings.vue src/components/profile/UserProfilePanel.vue
git commit -m "style: 用户信息和设置页面暗色化"
```

---

### Task 9: 批量导入对话框暗色化

**Files:**
- Modify: `src/components/batch/BatchImportDialog.vue`

- [ ] **Step 1: 替换 BatchImportDialog.vue 中所有浅色 class**

关键替换：
- DialogOverlay: `bg-black/50` → 保持不变
- DialogContent: `bg-white` → `bg-dark-panel border border-dark-border`
- DialogTitle: 保持白色文字
- DialogDescription: `text-muted-foreground` → `text-text-secondary`
- 选择按钮: `border-input hover:bg-gray-50` → `border-dark-border-light hover:bg-dark-surface text-text-secondary`
- 文件列表容器: `border` → `border-dark-border`
- 文件项分隔线: `border-b` → `border-dark-border`
- 进度条: `bg-gray-200` → `bg-dark-surface`，填充 `bg-blue-500` → `brand-gradient`
- 底部按钮: 同其他页面
- 失败文字: `text-red-500` → `text-danger`

- [ ] **Step 2: 提交**

```bash
git add src/components/batch/BatchImportDialog.vue
git commit -m "style: 批量导入对话框暗色化"
```

---

### Task 10: Markdown 暗色代码高亮 + 最终验证

**Files:**
- Modify: `src/utils/markdown.ts`

- [ ] **Step 1: 确认 highlight.js 暗色主题已引入**

在 `src/views/Chat.vue` 中已有 `import 'highlight.js/styles/github-dark.min.css';`，确认无需额外修改。如果 `github-dark` 样式与暗色背景不协调，可替换为 `import 'highlight.js/styles/atom-one-dark.min.css';`。

- [ ] **Step 2: 构建验证**

```bash
cd /sessions/69d7be74a5234068df3dcc36/workspace && npm run build
```

Expected: 构建成功，无 TypeScript 错误。

- [ ] **Step 3: 提交**

```bash
git add -A
git commit -m "style: 完成暗色现代风 UI 重设计"
```
