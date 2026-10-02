# 计划文档：通知模块优化（已读状态管理与通知声音）

## 1. 概述
对通知模块进行深度优化，提升用户通知管理的灵活性和声音感知：
1. **保留已读状态管理**：取消在点击铃铛图标时自动将全部通知标记为已读的逻辑。
2. **支持点击单条通知标记已读及精准跳转**。
3. **增加声音反馈**：收到新通知时，通过 Web Audio API 合成一个柔和科技感的声音。

## 2. 代码修改计划

### 修改 `src/components/layout/AppLayout.vue`
删除 `toggleNotifications` 里的自动 `markAllAsRead` 调用，保持通知面板内真实的已读/未读状态。

```diff
const toggleNotifications = () => {
  showNotifications.value = !showNotifications.value;
- if (showNotifications.value) {
-   notificationsStore.markAllAsRead();
- }
};
```

### 修改 `src/stores/notifications.ts`
1. 引入 Web Audio API 声音合成函数 `playNotificationSound`。
2. 在 `initListener()` 监听收到新通知 `notification-created` 时，自动调用 `playNotificationSound()`。

### 修改 `src/components/layout/NotificationPanel.vue`
在 `handleNotificationClick(notification)` 里新增精准模块的路由跳转支持。例如：
- `model_download_complete` / `model_installed` -> 跳转到 `/models`
- `document_index_complete` -> 跳转到 `/documents`

## 3. 验证计划
1. 启动项目运行 `pnpm tsc --noEmit` 检查语法和 TS 编译。
2. 确保各通知状态正常。
