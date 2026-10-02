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
         ORDER BY created_at DESC",
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

pub fn get_notifications_paginated(
    conn: &Connection,
    limit: i32,
    offset: i32,
) -> Result<Vec<AppNotification>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, title, content, route_path, is_read, created_at
         FROM notifications
         ORDER BY created_at DESC
         LIMIT ?1 OFFSET ?2",
    )?;
    let notifications = stmt.query_map(params![limit, offset], |row| {
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

pub fn get_total_count(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM notifications")?;
    stmt.query_row([], |row| row.get(0))
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
