use crate::db::notifications::{self, AppNotification};
use crate::errors::AppError;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
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
    let _ = app.emit("notification-created", &notification);
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
    Ok(items
        .into_iter()
        .map(|n| NotificationPayload {
            id: n.id,
            notification_type: n.notification_type,
            title: n.title,
            content: n.content,
            route_path: n.route_path,
            is_read: n.is_read,
            created_at: n.created_at,
        })
        .collect())
}

#[tauri::command]
pub fn get_notifications_paginated(
    page: i32,
    page_size: i32,
    app: AppHandle,
) -> Result<crate::models::PaginatedResult<NotificationPayload>, AppError> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    notifications::init_notifications_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init notifications table: {}", e)))?;

    let offset = (page - 1) * page_size;
    let total = notifications::get_total_count(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to get total count: {}", e)))?
        as i64;

    let items = notifications::get_notifications_paginated(&conn, page_size, offset)
        .map_err(|e| AppError::Internal(format!("Failed to get items: {}", e)))?;

    let items = items
        .into_iter()
        .map(|n| NotificationPayload {
            id: n.id,
            notification_type: n.notification_type,
            title: n.title,
            content: n.content,
            route_path: n.route_path,
            is_read: n.is_read,
            created_at: n.created_at,
        })
        .collect();

    Ok(crate::models::PaginatedResult::new(
        items, total, page, page_size,
    ))
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
