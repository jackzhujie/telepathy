# KeepAlive 页面缓存设计

## 目标
为 Vue 3 路由添加页面缓存功能，保留用户操作状态（对话历史、下载进度等）。

## 缓存策略
- **缓存**: Chat, Documents, Settings
- **不缓存**: KnowledgeBase

## 实现方案

### 1. 路由配置 (`src/router/index.ts`)
为 Chat、Documents、Settings 路由添加 `meta: { keepAlive: true }`

### 2. AppLayout.vue 修改
使用 `<KeepAlive>` 包裹 `<router-view>`，通过 `include` 属性指定需要缓存的组件名列表。

```vue
<KeepAlive :include="['Chat', 'Documents', 'Settings']">
  <router-view />
</KeepAlive>
```

### 3. 组件命名
确保 Chat.vue、Documents.vue、Settings.vue 的 `<script setup>` 中定义 `name` 属性，与 KeepAlive 的 include 列表匹配。

使用 `defineOptions({ name: 'Chat' })` 语法。

## 文件变更清单
1. `src/router/index.ts` - 添加 meta.keepAlive
2. `src/components/layout/AppLayout.vue` - 添加 KeepAlive 包裹
3. `src/views/Chat.vue` - 添加 defineOptions name
4. `src/views/Documents.vue` - 添加 defineOptions name
5. `src/views/Settings.vue` - 添加 defineOptions name
