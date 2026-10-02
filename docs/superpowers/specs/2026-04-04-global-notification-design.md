# 全局通知系统设计

## 目标
实现系统级通知 + 前端通知中心的双通道通知机制，支持持久化存储，当前阶段仅实现系统事件通知。

## 通知类型
- `model_download_complete` - 模型下载完成
- `ollama_install_complete` - Ollama 安装完成
- `app_update_available` - 应用更新可用
- `system` - 其他系统通知

## 架构

### 后端 (Rust/Tauri)
- 在关键事件触发时，通过 Tauri Event 发送通知到前端
- 调用 Tauri Notification API 发送系统级通知

### 前端
- 创建 `notifications` Pinia store 管理通知列表、未读计数
- Sidebar 添加铃铛图标 + 未读徽标，点击弹出通知面板
- 通知数据持久化到 SQLite

## 数据模型

```typescript
interface AppNotification {
  id: string;
  type: 'model_download_complete' | 'ollama_install_complete' | 'app_update_available' | 'system';
  title: string;
  content: string;
  routePath?: string;  // 点击通知后的跳转路径
  isRead: boolean;
  createdAt: string;   // ISO 时间字符串
}
```

## 文件变更清单

### 新建文件
1. `src/types/notification.ts` - 通知类型定义
2. `src/stores/notifications.ts` - 通知 store（状态管理 + 持久化）
3. `src/components/layout/NotificationPanel.vue` - 通知面板组件

### 修改文件
1. `src/api/tauri.ts` - 添加通知相关的 Tauri 命令和事件类型
2. `src/components/layout/Sidebar.vue` - 添加铃铛图标和通知面板入口
3. `src/stores/settings.ts` - 在模型下载/安装完成时触发通知
4. `src-tauri/src/lib.rs` - 在关键事件发送 Tauri Event + 系统通知

## 交互流程

1. 后端事件触发 → 发送 Tauri Event 到前端
2. 前端 store 接收事件 → 创建通知 → 写入 SQLite
3. 后端同时触发系统级通知（macOS 原生弹窗）
4. 用户点击 Sidebar 铃铛查看通知列表
5. 点击单条通知 → 标记已读 → 跳转到关联页面

## 持久化策略

- 通知创建时立即写入 SQLite
- 应用启动时从 SQLite 加载所有未读通知（已读通知也保留，供用户回顾）
- 通知标记为已读时更新数据库
