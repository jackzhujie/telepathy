#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: String,
    pub title: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sources: Option<String>,
    pub images: Option<String>,
    pub created_at: String,
}

pub fn init_conversations_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            sources TEXT,
            images TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);",
    )
    .map_err(|e| AppError::Internal(format!("Failed to create conversations tables: {}", e)))?;

    // 动态添加 images 列，确保在已有的数据库中进行平滑升级而不奔溃
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN images TEXT", []);

    Ok(())
}

pub fn create_conversation(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
) -> Result<Conversation, AppError> {
    conn.execute(
        "INSERT INTO conversations (id, title) VALUES (?1, ?2)",
        params![id, title],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create conversation: {}", e)))?;
    Ok(Conversation {
        id: id.to_string(),
        title: title.map(|s| s.to_string()),
        created_at: String::new(),
    })
}

pub fn get_conversations(conn: &Connection) -> Result<Vec<Conversation>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, title, created_at FROM conversations ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_messages(conn: &Connection, conv_id: &str) -> Result<Vec<ChatMessage>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, sources, images, created_at
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;
    let rows = stmt
        .query_map(params![conv_id], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                sources: row.get(4)?,
                images: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn insert_message(conn: &Connection, msg: &ChatMessage) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, sources, images, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            msg.id,
            msg.conversation_id,
            msg.role,
            msg.content,
            msg.sources,
            msg.images,
            msg.created_at,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert message: {}", e)))?;
    Ok(())
}

pub fn update_message_content(
    conn: &Connection,
    message_id: &str,
    content: &str,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE messages SET content = ?1 WHERE id = ?2",
        params![content, message_id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to update message content: {}", e)))?;
    Ok(())
}

pub fn delete_conversation(conn: &Connection, conv_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conv_id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to delete messages: {}", e)))?;
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![conv_id])
        .map_err(|e| AppError::Internal(format!("Failed to delete conversation: {}", e)))?;
    Ok(())
}

pub fn get_message_by_id(
    conn: &Connection,
    message_id: &str,
) -> Result<ChatMessage, rusqlite::Error> {
    conn.query_row(
        "SELECT id, conversation_id, role, content, sources, images, created_at FROM messages WHERE id = ?1",
        rusqlite::params![message_id],
        |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                sources: row.get(4)?,
                images: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    )
}

pub fn delete_messages_from(
    conn: &Connection,
    conversation_id: &str,
    created_at: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1 AND created_at >= ?2",
        rusqlite::params![conversation_id, created_at],
    )?;
    Ok(())
}

pub fn get_conversations_paginated(
    conn: &Connection,
    page: i32,
    page_size: i32,
) -> Result<(Vec<Conversation>, i64), AppError> {
    let offset = (page - 1) * page_size;

    let mut stmt = conn
        .prepare("SELECT id, title, created_at FROM conversations ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(params![page_size, offset], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let conversations: Vec<Conversation> = rows.filter_map(|r| r.ok()).collect();

    let total = get_conversations_count(conn)?;

    Ok((conversations, total))
}

pub fn get_conversations_count(conn: &Connection) -> Result<i64, AppError> {
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM conversations")
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;

    let count = stmt
        .query_row([], |row| row.get(0))
        .map_err(|e| AppError::Internal(format!("Failed to count conversations: {}", e)))?;

    Ok(count)
}

pub fn get_messages_paginated(
    conn: &Connection,
    conv_id: &str,
    page: i32,
    page_size: i32,
) -> Result<(Vec<ChatMessage>, i64), AppError> {
    let offset = (page - 1) * page_size;

    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, sources, images, created_at
             FROM (
                SELECT id, conversation_id, role, content, sources, images, created_at
                FROM messages
                WHERE conversation_id = ?1
                ORDER BY created_at DESC
                LIMIT ?2 OFFSET ?3
             )
             ORDER BY created_at ASC",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(params![conv_id, page_size, offset], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                sources: row.get(4)?,
                images: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let messages: Vec<ChatMessage> = rows.filter_map(|r| r.ok()).collect();

    let total = get_messages_count(conn, conv_id)?;

    Ok((messages, total))
}

pub fn get_messages_count(conn: &Connection, conv_id: &str) -> Result<i64, AppError> {
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM messages WHERE conversation_id = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;

    let count = stmt
        .query_row(params![conv_id], |row| row.get(0))
        .map_err(|e| AppError::Internal(format!("Failed to count messages: {}", e)))?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_conversations_table(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_conversations() {
        let conn = setup_db();
        create_conversation(&conn, "conv-1", Some("Test Chat")).unwrap();
        let convs = get_conversations(&conn).unwrap();
        assert_eq!(convs.len(), 1);
        assert_eq!(convs[0].title, Some("Test Chat".to_string()));
    }

    #[test]
    fn test_insert_and_get_messages() {
        let conn = setup_db();
        create_conversation(&conn, "conv-1", None).unwrap();
        let msg = ChatMessage {
            id: "msg-1".to_string(),
            conversation_id: "conv-1".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            sources: None,
            images: Some("[\"data:image/png;base64,...\"]".to_string()),
            created_at: "2026-01-01".to_string(),
        };
        insert_message(&conn, &msg).unwrap();
        let messages = get_messages(&conn, "conv-1").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[0].images, Some("[\"data:image/png;base64,...\"]".to_string()));
    }

    #[test]
    fn test_delete_conversation() {
        let conn = setup_db();
        create_conversation(&conn, "conv-1", None).unwrap();
        delete_conversation(&conn, "conv-1").unwrap();
        let convs = get_conversations(&conn).unwrap();
        assert!(convs.is_empty());
    }
}
