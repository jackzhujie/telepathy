# 聊天界面 UI 优化实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 全面重构和优化聊天界面 UI，采用极致悬浮的毛玻璃风格（Glassmorphism），并完美支持深浅色模式与主题色切换。

**Architecture:** 
1. 通过 Tailwind + CSS 变量实现完全不含硬编码色值的极客质感毛玻璃气泡。
2. 扩宽聊天面板最大宽度以提升长文本阅读体验。
3. 输入框升级为 Island 风格悬浮毛玻璃底座。

**Tech Stack:** Vue 3, Tailwind CSS (w/ dynamic variables), Reka UI

---

### Task 1: 对话历史侧边栏毛玻璃升级 (ConversationSidebar.vue)

**Files:**
- Modify: `src/components/chat/ConversationSidebar.vue`

- [ ] **Step 1: 升级侧边栏容器样式为毛玻璃效果**
- [ ] **Step 2: 验证侧边栏编译正确**
- [ ] **Step 3: Commit 提交变更**

---

### Task 2: 聊天面板框架与 Header 毛玻璃化设计 (Chat.vue)

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: 升级 Chat.vue 页面主体框架最大宽度与毛玻璃 Header**
- [ ] **Step 2: 验证 Chat.vue 编译正确**
- [ ] **Step 3: Commit 提交变更**

---

### Task 3: 聊天消息气泡与 Markdown 容器悬浮升级 (MessageItem.vue)

**Files:**
- Modify: `src/components/chat/MessageItem.vue`

- [ ] **Step 1: 将 MessageItem.vue 整体气泡更新为悬浮毛玻璃**
- [ ] **Step 2: 验证编译正确**
- [ ] **Step 3: Commit 提交变更**

---

### Task 4: 输入框升级为 Island 风格悬浮底座 (ChatInput.vue)

**Files:**
- Modify: `src/components/chat/ChatInput.vue`

- [ ] **Step 1: 将输入组件的 TextArea 容器更新为圆角 Island 底座**
- [ ] **Step 2: 验证编译正确并测试**
- [ ] **Step 3: Commit 提交变更**
