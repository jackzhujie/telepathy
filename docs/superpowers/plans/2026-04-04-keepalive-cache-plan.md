# KeepAlive 页面缓存实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Chat、Documents、Settings 三个页面添加 Vue KeepAlive 缓存，保留用户操作状态。

**Architecture:** 在 AppLayout.vue 中使用 `<KeepAlive :include="['Chat', 'Documents', 'Settings']">` 包裹 `<router-view>`，并为三个视图组件通过 `defineOptions` 定义 `name` 属性以匹配 include 列表。

**Tech Stack:** Vue 3 Composition API, Vue Router

---

### Task 1: 为路由添加 meta.keepAlive 标记

**Files:**
- Modify: `src/router/index.ts`

- [ ] **Step 1: 修改路由配置，为 Chat、Documents、Settings 添加 meta**

修改 `src/router/index.ts`，为三个路由添加 `meta: { keepAlive: true }`:

```typescript
import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'KnowledgeBase', component: () => import('@/views/KnowledgeBase.vue') },
  { path: '/chat', name: 'Chat', component: () => import('@/views/Chat.vue'), meta: { keepAlive: true } },
  { path: '/documents', name: 'Documents', component: () => import('@/views/Documents.vue'), meta: { keepAlive: true } },
  { path: '/settings', name: 'Settings', component: () => import('@/views/Settings.vue'), meta: { keepAlive: true } }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
```

- [ ] **Step 2: 验证修改**

确认文件保存无语法错误。

---

### Task 2: 为三个视图组件添加 name 属性

KeepAlive 的 `include` 匹配的是组件的 `name`，需要通过 `defineOptions` 定义。

**Files:**
- Modify: `src/views/Chat.vue`
- Modify: `src/views/Documents.vue`
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: Chat.vue 添加 defineOptions**

在 `src/views/Chat.vue` 的 `<script setup>` 第一行添加:

```typescript
defineOptions({ name: 'Chat' });
```

修改后文件开头为:

```vue
<script setup lang="ts">
defineOptions({ name: 'Chat' });
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
// ... 其余不变
```

- [ ] **Step 2: Documents.vue 添加 defineOptions**

在 `src/views/Documents.vue` 的 `<script setup>` 第一行添加:

```typescript
defineOptions({ name: 'Documents' });
```

修改后文件开头为:

```vue
<script setup lang="ts">
defineOptions({ name: 'Documents' });
import { ref, onMounted } from 'vue';
// ... 其余不变
```

- [ ] **Step 3: Settings.vue 添加 defineOptions**

在 `src/views/Settings.vue` 的 `<script setup>` 第一行添加:

```typescript
defineOptions({ name: 'Settings' });
```

修改后文件开头为:

```vue
<script setup lang="ts">
defineOptions({ name: 'Settings' });
import { ref, onMounted, computed } from 'vue';
// ... 其余不变
```

---

### Task 3: AppLayout.vue 添加 KeepAlive 包裹

**Files:**
- Modify: `src/components/layout/AppLayout.vue`

- [ ] **Step 1: 修改 AppLayout.vue，用 KeepAlive 包裹 router-view**

完整文件内容:

```vue
<script setup lang="ts">
import Sidebar from './Sidebar.vue';
import { NLayout } from 'naive-ui';
</script>

<template>
  <n-layout has-sider style="height: 100vh;">
    <Sidebar />
    <n-layout>
      <div style="padding: 24px; height: 100%; box-sizing: border-box;">
        <KeepAlive :include="['Chat', 'Documents', 'Settings']">
          <router-view />
        </KeepAlive>
      </div>
    </n-layout>
  </n-layout>
</template>
```

- [ ] **Step 2: 验证修改**

确认文件保存无语法错误。

---

### Task 4: 验证功能

- [ ] **Step 1: 启动开发服务器**

```bash
pnpm dev
```

- [ ] **Step 2: 手动验证以下行为**

1. 进入 Chat 页面，输入一些内容或切换对话，切到其他页面再切回来，状态应保持
2. 进入 Documents 页面，切到其他页面再切回来，列表状态应保持
3. 进入 Settings 页面，切到其他页面再切回来，页面状态应保持
4. 进入 KnowledgeBase 页面，切到其他页面再切回来，页面应重新加载（不缓存）

---
