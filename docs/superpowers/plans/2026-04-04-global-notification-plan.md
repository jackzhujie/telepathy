# 全局通知系统实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现系统级通知 + 前端通知中心的双通道通知机制，支持 SQLite 持久化存储。

**Architecture:** 后端在关键事件触发时发送 Tauri Event 到前端并调用系统通知 API；前端通过 Pinia store 管理通知状态，Sidebar 添加铃铛图标，点击弹出通知面板。通知数据持久化到 SQLite。

**Tech Stack:** Tauri v2, Rust (rusqlite), Vue 3, Pinia, Naive UI

---

### Task 0: 添加 tauri-plugin-notification 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 添加 Cargo 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加:

```toml
tauri-plugin-notification = "2"
```

- [ ] **Step 2: 注册插件**

修改 `src-tauri/src/lib.rs`，在 `.plugin(tauri_plugin_dialog::init())` 后添加:

```rust
.plugin(tauri_plugin_notification::init())
```

- [ ] **Step 3: 添加通知权限到 default capability**

修改 `src-tauri/capabilities/default.json`，在 `permissions` 数组中添加:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "notification:default"
  ]
}
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/capabilities/default.json
git commit -m "feat: add tauri-plugin-notification dependency"
```

---

### Task 1: 通知类型定义 (前端)

**Files:**
- Create: `src/types/notification.ts`

- [ ] **Step 1: 创建通知类型定义文件**

创建 `src/types/notification.ts`:

```typescript
export type NotificationType = 
  | 'model_download_complete'
  | 'ollama_install_complete'
  | 'app_update_available'
  | 'system';

export interface AppNotification {
  id: string;
  type: NotificationType;
  title: string;
  content: string;
  routePath?: string;
  isRead: boolean;
  createdAt: string;
}

export interface NotificationPayload {
  id: string;
  type: NotificationType;
  title: string;
  content: string;
  routePath?: string;
}
```

- [ ] **Step 2: 提交**

```bash
git add src/types/notification.ts
git commit -m "feat: add notification type definitions"
```

---

### Task 2: 通知数据库层 (后端 Rust)

**Files:**
- Create: `src-tauri/src/db/notifications.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: 创建 notifications.rs 数据库模块**

创建 `src-tauri/src/db/notifications.rs`:

```rust
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppNotification {
    pub id: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub content: String,
    pub route_path: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

pub fn init_notifications_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notifications (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            route_path TEXT,
            is_read INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn insert_notification(conn: &Connection, notification: &AppNotification) -> Result<()> {
    conn.execute(
        "INSERT INTO notifications (id, type, title, content, route_path, is_read, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            notification.id,
            notification.notification_type,
            notification.title,
            notification.content,
            notification.route_path,
            if notification.is_read { 1 } else { 0 },
            notification.created_at,
        ],
    )?;
    Ok(())
}

pub fn get_notifications(conn: &Connection) -> Result<Vec<AppNotification>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, title, content, route_path, is_read, created_at
         FROM notifications
         ORDER BY created_at DESC"
    )?;
    let notifications = stmt.query_map([], |row| {
        Ok(AppNotification {
            id: row.get(0)?,
            notification_type: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            route_path: row.get(4)?,
            is_read: row.get::<_, i32>(5)? != 0,
            created_at: row.get(6)?,
        })
    })?;
    notifications.collect()
}

pub fn mark_as_read(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE notifications SET is_read = 1 WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn get_unread_count(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM notifications WHERE is_read = 0")?;
    stmt.query_row([], |row| row.get(0))
}

pub fn create_notification(
    notification_type: &str,
    title: &str,
    content: &str,
    route_path: Option<&str>,
) -> AppNotification {
    AppNotification {
        id: Uuid::new_v4().to_string(),
        notification_type: notification_type.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        route_path: route_path.map(String::from),
        is_read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}
```

- [ ] **Step 2: 更新 db/mod.rs 导出 notifications 模块**

修改 `src-tauri/src/db/mod.rs`:

```rust
pub mod conversations;
pub mod documents;
pub mod notifications;
pub mod settings;
pub mod vectors;
```

**注意**: 本项目采用每个命令单独打开数据库连接的模式（见 `settings.rs`），不需要在 main.rs 中全局初始化。notifications 表会在首次插入/查询时通过 `init_notifications_table` 自动创建。在 Task 3 的 `get_notifications` 命令中，打开连接后先调用 `init_notifications_table` 确保表存在。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/db/notifications.rs src-tauri/src/db/mod.rs src-tauri/src/main.rs
git commit -m "feat: add notifications database layer"
```

---

### Task 3: 通知 Tauri 命令 (后端 Rust)

**Files:**
- Create: `src-tauri/src/commands/notifications.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 notifications.rs 命令模块**

创建 `src-tauri/src/commands/notifications.rs`:

```rust
use crate::db::notifications::{self, AppNotification};
use crate::errors::AppError;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Serialize, Clone)]
pub struct NotificationPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub content: String,
    pub route_path: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

fn get_db_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
    }
    Ok(app_data_dir.join("telepathy.db"))
}

pub fn emit_notification(app: &AppHandle, notification: AppNotification) {
    // 写入数据库
    let db_path = match get_db_path(app) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get db path for notification: {}", e);
            return;
        }
    };
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let _ = notifications::init_notifications_table(&conn);
        let _ = notifications::insert_notification(&conn, &notification);
    }
    // 发送前端事件
    let _ = app.emit("notification-created", &notification);
    // 发送系统通知
    #[cfg(not(target_os = "linux"))]
    {
        let _ = app
            .notification()
            .builder()
            .title(&notification.title)
            .body(&notification.content)
            .show();
    }
}

#[tauri::command]
pub fn get_notifications(app: AppHandle) -> Result<Vec<NotificationPayload>, AppError> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    notifications::init_notifications_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init notifications table: {}", e)))?;
    let items = notifications::get_notifications(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to get notifications: {}", e)))?;
    Ok(items.into_iter().map(|n| NotificationPayload {
        id: n.id,
        notification_type: n.notification_type,
        title: n.title,
        content: n.content,
        route_path: n.route_path,
        is_read: n.is_read,
        created_at: n.created_at,
    }).collect())
}

#[tauri::command]
pub fn mark_notification_read(app: AppHandle, id: String) -> Result<(), AppError> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    notifications::init_notifications_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init notifications table: {}", e)))?;
    notifications::mark_as_read(&conn, &id)
        .map_err(|e| AppError::Internal(format!("Failed to mark as read: {}", e)))
}

#[tauri::command]
pub fn get_unread_count(app: AppHandle) -> Result<i32, AppError> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    notifications::init_notifications_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init notifications table: {}", e)))?;
    notifications::get_unread_count(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to get unread count: {}", e)))
}
```

- [ ] **Step 2: 更新 commands/mod.rs**

修改 `src-tauri/src/commands/mod.rs`:

```rust
pub mod chat;
pub mod document;
pub mod greet;
pub mod indexing;
pub mod installer;
pub mod notifications;
pub mod rag;
pub mod settings;
```

- [ ] **Step 3: 注册 Tauri 命令**

修改 `src-tauri/src/lib.rs`，在 `invoke_handler` 中添加:

```rust
commands::notifications::get_notifications,
commands::notifications::mark_notification_read,
commands::notifications::get_unread_count,
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/notifications.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add notification Tauri commands"
```

---

### Task 4: 通知 Pinia Store (前端)

**Files:**
- Create: `src/stores/notifications.ts`
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 添加通知 API 函数**

在 `src/api/tauri.ts` 末尾添加:

```typescript
import type { AppNotification } from '@/types/notification';

export async function getNotifications(): Promise<AppNotification[]> {
  return invoke<AppNotification[]>('get_notifications');
}

export async function markNotificationRead(id: string): Promise<void> {
  return invoke<void>('mark_notification_read', { id });
}

export async function getUnreadCount(): Promise<number> {
  return invoke<number>('get_unread_count');
}
```

- [ ] **Step 2: 创建 notifications store**

创建 `src/stores/notifications.ts`:

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { getNotifications, markNotificationRead as markReadApi, getUnreadCount } from '@/api/tauri';
import type { AppNotification } from '@/types/notification';

export const useNotificationsStore = defineStore('notifications', () => {
  const notifications = ref<AppNotification[]>([]);
  const unreadCount = ref(0);
  const isLoading = ref(false);

  let unlistenNotification: UnlistenFn | null = null;

  async function initListener() {
    if (unlistenNotification) return;
    unlistenNotification = await listen<AppNotification>('notification-created', (event) => {
      const newNotification = event.payload;
      notifications.value.unshift(newNotification);
      if (!newNotification.isRead) {
        unreadCount.value++;
      }
    });
  }

  async function fetchNotifications() {
    isLoading.value = true;
    try {
      notifications.value = await getNotifications();
      unreadCount.value = notifications.value.filter((n) => !n.isRead).length;
    } finally {
      isLoading.value = false;
    }
  }

  async function markNotificationRead(id: string) {
    await markReadApi(id);
    const notification = notifications.value.find((n) => n.id === id);
    if (notification && !notification.isRead) {
      notification.isRead = true;
      unreadCount.value = Math.max(0, unreadCount.value - 1);
    }
  }

  async function markAllAsRead() {
    for (const n of notifications.value.filter((n) => !n.isRead)) {
      await markReadApi(n.id);
      n.isRead = true;
    }
    unreadCount.value = 0;
  }

  return {
    notifications,
    unreadCount,
    isLoading,
    initListener,
    fetchNotifications,
    markNotificationRead,
    markAllAsRead,
  };
});
```

- [ ] **Step 3: 提交**

```bash
git add src/api/tauri.ts src/stores/notifications.ts
git commit -m "feat: add notifications store and API"
```

---

### Task 5: 通知面板组件 (前端)

**Files:**
- Create: `src/components/layout/NotificationPanel.vue`

- [ ] **Step 1: 创建通知面板组件**

创建 `src/components/layout/NotificationPanel.vue`:

```vue
<script setup lang="ts">
import { useNotificationsStore } from '@/stores/notifications';
import { useRouter } from 'vue-router';

const emit = defineEmits<{ close: [] }>();
const store = useNotificationsStore();
const router = useRouter();

const typeIconMap: Record<string, string> = {
  model_download_complete: '📦',
  ollama_install_complete: '⚙️',
  app_update_available: '🔄',
  system: 'ℹ️',
};

const typeColorMap: Record<string, string> = {
  model_download_complete: '#18a058',
  ollama_install_complete: '#2080f0',
  app_update_available: '#f0a020',
  system: '#666',
};

function handleNotificationClick(notification: any) {
  store.markNotificationRead(notification.id);
  if (notification.routePath) {
    router.push(notification.routePath);
  }
}

function handleMarkAllRead() {
  store.markAllAsRead();
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  if (hours < 24) return `${hours} 小时前`;
  if (days < 7) return `${days} 天前`;
  return date.toLocaleDateString('zh-CN');
}
</script>

<template>
  <div class="notification-panel">
    <div class="notification-header">
      <h3>通知中心</h3>
      <div class="notification-actions">
        <n-button v-if="store.unreadCount > 0" size="tiny" quaternary type="primary" @click="handleMarkAllRead">
          全部已读
        </n-button>
        <n-button size="tiny" quaternary @click="emit('close')">✕</n-button>
      </div>
    </div>

    <n-scrollbar class="notification-list" style="max-height: 400px;">
      <div v-if="store.notifications.length === 0" class="empty-state">
        暂无通知
      </div>

      <div
        v-for="notification in store.notifications"
        :key="notification.id"
        class="notification-item"
        :class="{ unread: !notification.isRead }"
        @click="handleNotificationClick(notification)"
      >
        <div class="notification-icon" :style="{ color: typeColorMap[notification.type] || '#666' }">
          {{ typeIconMap[notification.type] || 'ℹ️' }}
        </div>
        <div class="notification-content">
          <div class="notification-title">{{ notification.title }}</div>
          <div class="notification-text">{{ notification.content }}</div>
          <div class="notification-time">{{ formatTime(notification.createdAt) }}</div>
        </div>
        <div v-if="!notification.isRead" class="unread-dot"></div>
      </div>
    </n-scrollbar>
  </div>
</template>

<style scoped>
.notification-panel {
  width: 360px;
  background-color: #1e1e1e;
  border: 1px solid rgba(255, 255, 255, 0.09);
  border-radius: 8px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.notification-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.09);
}

.notification-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
}

.notification-actions {
  display: flex;
  gap: 4px;
  align-items: center;
}

.notification-list {
  padding: 0;
}

.empty-state {
  text-align: center;
  color: #666;
  padding: 40px 16px;
  font-size: 13px;
}

.notification-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background-color 0.15s;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  position: relative;
}

.notification-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.notification-item.unread {
  background-color: rgba(32, 128, 240, 0.05);
}

.notification-icon {
  font-size: 20px;
  flex-shrink: 0;
  margin-top: 2px;
}

.notification-content {
  flex: 1;
  min-width: 0;
}

.notification-title {
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 4px;
}

.notification-text {
  font-size: 12px;
  color: #999;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.notification-time {
  font-size: 11px;
  color: #666;
  margin-top: 4px;
}

.unread-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #2080f0;
  flex-shrink: 0;
  margin-top: 6px;
}
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/layout/NotificationPanel.vue
git commit -m "feat: add notification panel component"
```

---

### Task 6: Sidebar 集成通知入口

**Files:**
- Modify: `src/components/layout/Sidebar.vue`

- [ ] **Step 1: 在 Sidebar 顶部添加铃铛图标**

修改 `src/components/layout/Sidebar.vue`:

```vue
<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { useAppStore } from '@/stores/app';
import { useNotificationsStore } from '@/stores/notifications';
import { NMenu, NLayoutSider, NBadge } from 'naive-ui';
import { ref } from 'vue';
import NotificationPanel from './NotificationPanel.vue';

const router = useRouter();
const route = useRoute();
const appStore = useAppStore();
const notificationsStore = useNotificationsStore();
const showNotifications = ref(false);

const menuOptions = [
  { label: '知识库', key: 'KnowledgeBase' },
  { label: 'AI 对话', key: 'Chat' },
  { label: '文档管理', key: 'Documents' },
  { label: '系统设置', key: 'Settings' }
];

function handleMenuClick(key: string) {
  router.push({ name: key });
  showNotifications.value = false;
}

function toggleNotifications() {
  showNotifications.value = !showNotifications.value;
}
</script>

<template>
  <n-layout-sider
    bordered
    collapse-mode="width"
    :collapsed-width="64"
    :width="200"
    :collapsed="appStore.isSidebarCollapsed"
    show-trigger
    @collapse="appStore.isSidebarCollapsed = true"
    @expand="appStore.isSidebarCollapsed = false"
  >
    <div class="sidebar-header">
      <span class="app-title">{{ appStore.isSidebarCollapsed ? 'T' : 'Telepathy' }}</span>
      <n-badge :value="notificationsStore.unreadCount" :max="99" :show="notificationsStore.unreadCount > 0">
        <n-button
          text
          size="small"
          class="notification-bell"
          :class="{ 'has-unread': notificationsStore.unreadCount > 0 }"
          @click="toggleNotifications"
        >
          🔔
        </n-button>
      </n-badge>
    </div>

    <n-menu
      :value="route.name as string"
      :options="menuOptions"
      @update:value="handleMenuClick"
    />

    <Teleport to="body">
      <div v-if="showNotifications" class="notification-overlay" @click="showNotifications = false"></div>
      <div v-if="showNotifications" class="notification-panel-wrapper">
        <NotificationPanel @close="showNotifications = false" />
      </div>
    </Teleport>
  </n-layout-sider>
</template>

<style scoped>
.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  font-weight: bold;
  font-size: 18px;
  white-space: nowrap;
  overflow: hidden;
}

.app-title {
  overflow: hidden;
  text-overflow: ellipsis;
}

.notification-bell {
  font-size: 16px;
  padding: 4px;
  transition: color 0.15s;
}

.notification-bell.has-unread {
  color: #2080f0;
}

.notification-overlay {
  position: fixed;
  inset: 0;
  z-index: 999;
}

.notification-panel-wrapper {
  position: fixed;
  top: 60px;
  left: 210px;
  z-index: 1000;
}
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "feat: integrate notification bell into sidebar"
```

---

### Task 7: 连接后端事件到通知系统

**Files:**
- Modify: `src-tauri/src/commands/installer.rs`

- [ ] **Step 1: 在 installer.rs 中导入通知模块**

在 `src-tauri/src/commands/installer.rs` 顶部添加导入:

```rust
use crate::commands::notifications::emit_notification;
use crate::db::notifications;
```

- [ ] **Step 2: 在 Ollama 安装完成时创建通知**

在 `download_ollama` 函数中，找到最后一个 `app.emit("ollama-download-progress", ...)` done 之后、`Ok(ollama_path)` 之前，添加:

```rust
    let notification = notifications::create_notification(
        "ollama_install_complete",
        "Ollama 安装完成",
        "Ollama 已成功安装到您的系统",
        Some("/settings"),
    );
    emit_notification(&app, notification);
```

- [ ] **Step 3: 在模型拉取完成时创建通知**

在 `pull_model` 函数中，找到最后一个 `app.emit("ollama-pull-progress", ...)` done 之后、`Ok(())` 之前，添加:

```rust
    let notification = notifications::create_notification(
        "model_download_complete",
        "模型下载完成",
        &format!("模型 {} 已成功下载", model),
        Some("/settings"),
    );
    emit_notification(&app, notification);
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/installer.rs
git commit -m "feat: wire up installer events to notification system"
```

---

### Task 8: 前端初始化通知监听

**Files:**
- Modify: `src/App.vue` 或 `src/main.ts`

- [ ] **Step 1: 在应用启动时初始化通知**

修改 `src/App.vue`:

```vue
<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider } from 'naive-ui';
import AppLayout from '@/components/layout/AppLayout.vue';
import { useNotificationsStore } from '@/stores/notifications';
import { onMounted } from 'vue';

const notificationsStore = useNotificationsStore();

onMounted(async () => {
  await notificationsStore.initListener();
  await notificationsStore.fetchNotifications();
});
</script>

<template>
  <n-config-provider style="height: 100vh;">
    <n-message-provider>
      <n-dialog-provider>
        <AppLayout />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
```

- [ ] **Step 2: 提交**

```bash
git add src/App.vue
git commit -m "feat: initialize notifications on app startup"
```

---

### Task 9: 验证和类型检查

- [ ] **Step 1: 运行 TypeScript 类型检查**

```bash
npx vue-tsc --noEmit
```

预期: 无错误输出

- [ ] **Step 2: 运行 Rust 编译检查**

```bash
cd src-tauri && cargo check
```

预期: 编译通过无错误

- [ ] **Step 3: 提交**

```bash
git add .
git commit -m "chore: fix type errors"
```
