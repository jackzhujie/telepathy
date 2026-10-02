# 实现计划：SnapNote（随记）模块 - 第一阶段：核心基础

## 目标
建立随记模块的后端存储模型、基础 Tauri 通信链路以及前端状态管理。

## 详细步骤

### 1. 后端地基 (Rust)
- [ ] **数据库表创建**：修改 `src-tauri/src/db/notifications.rs` (或者新建 `db/snaps.rs`)。
    - 定义 `snaps` 表结构：`id`, `content`, `tags`, `is_pinned`, `created_at`, `updated_at`。
- [ ] **模型定义**：在 `src-tauri/src/models/` 下创建 `snap.rs`。
- [ ] **指令开发**：在 `src-tauri/src/commands/` 下创建 `snaps.rs`。
    - `get_snaps_paginated`: 支持分页获取随记。
    - `create_snap`: 保存新随记。
    - `update_snap`: 修改随记。
    - `delete_snap`: 删除。
- [ ] **指令注册**：在 `src-tauri/src/lib.rs` 中注册新命令。

### 2. 前端基础 (TypeScript & Pinia)
- [ ] **类型定义**：创建 `src/types/snap.ts`。
- [ ] **API 封装**：在 `src/api/tauri.ts` 中添加随记相关调用代码。
- [ ] **Store 建立**：创建 `src/stores/snap.ts`。
    - 实现列表加载、分页逻辑（复用我们之前的 Pagination 模式）。
    - 实现新建、搜索过滤功能。

### 3. UI 框架搭建
- [ ] **侧边栏更新**：修改 `Sidebar.vue` 添加“随记”图标和路由。
- [ ] **路由配置**：在 `src/router/index.ts` 中注册 `SnapNote.vue`。
- [ ] **基础视图**：创建 `src/views/SnapNote.vue`，实现简单的列表展示。

## 交付标准
1. 可以成功进入随记页面。
2. 可以通过控制台或简单的输入框创建一条随记并保存到数据库。
3. 刷新页面后随记依然存在且正确显示。
