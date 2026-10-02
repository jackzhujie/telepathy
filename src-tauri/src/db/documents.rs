#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::{params, Connection};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub original_path: String,
    pub library_path: String,
    pub file_type: String,
    pub size: i64,
    pub status: String,
    pub error_msg: Option<String>,
    pub project_id: Option<String>,
    pub created_at: String,
}

pub fn init_documents_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            original_path TEXT NOT NULL,
            library_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            error_msg TEXT,
            project_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create documents table: {}", e)))?;

    // 迁移：添加 project_id 列（如果不存在）
    let mut stmt = conn
        .prepare("PRAGMA table_info(documents)")
        .map_err(|e| AppError::Internal(format!("Failed to prepare PRAGMA: {}", e)))?;

    let column_names: Vec<String> = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            Ok(name)
        })
        .map_err(|e| AppError::Internal(format!("Failed to query table info: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect column names: {}", e)))?;

    let has_project_id = column_names.iter().any(|name| name == "project_id");

    if !has_project_id {
        conn.execute("ALTER TABLE documents ADD COLUMN project_id TEXT", [])
            .map_err(|e| AppError::Internal(format!("Failed to add project_id column: {}", e)))?;
    }

    Ok(())
}

pub fn insert_document(conn: &Connection, doc: &Document) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO documents (id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            doc.id,
            doc.name,
            doc.original_path,
            doc.library_path,
            doc.file_type,
            doc.size,
            doc.status,
            doc.error_msg,
            doc.project_id,
            doc.created_at,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert document: {}", e)))?;
    Ok(())
}

pub fn get_all_documents(conn: &Connection) -> Result<Vec<Document>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at FROM documents ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                original_path: row.get(2)?,
                library_path: row.get(3)?,
                file_type: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                error_msg: row.get(7)?,
                project_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query documents: {}", e)))?;

    let mut documents = Vec::new();
    for row in rows {
        documents.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }
    Ok(documents)
}

pub fn get_documents_by_project(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<Document>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at 
                  FROM documents 
                  WHERE project_id = ?1 
                  ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map(params![project_id], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                original_path: row.get(2)?,
                library_path: row.get(3)?,
                file_type: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                error_msg: row.get(7)?,
                project_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query documents: {}", e)))?;

    let mut documents = Vec::new();
    for row in rows {
        documents.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }
    Ok(documents)
}

pub fn get_document(conn: &Connection, id: &str) -> Result<Option<Document>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at FROM documents WHERE id = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                original_path: row.get(2)?,
                library_path: row.get(3)?,
                file_type: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                error_msg: row.get(7)?,
                project_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query document: {}", e)))?;

    match rows.next() {
        Some(row) => Ok(Some(row.map_err(|e| {
            AppError::Internal(format!("Failed to read row: {}", e))
        })?)),
        None => Ok(None),
    }
}

pub fn update_document_status(
    conn: &Connection,
    id: &str,
    status: &str,
    error_msg: Option<&str>,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE documents SET status = ?1, error_msg = ?2 WHERE id = ?3",
        params![status, error_msg, id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to update document status: {}", e)))?;
    Ok(())
}

pub fn delete_document(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM documents WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(format!("Failed to delete document: {}", e)))?;
    Ok(())
}

pub fn get_documents_paginated(
    conn: &Connection,
    project_id: Option<&str>,
    page: i32,
    page_size: i32,
) -> Result<(Vec<Document>, i64), AppError> {
    let offset = (page - 1) * page_size;
    let mut documents: Vec<Document> = Vec::new();

    if let Some(_project_id) = project_id {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at 
                 FROM documents 
                 WHERE project_id = ?1 
                 ORDER BY created_at DESC 
                 LIMIT ?2 OFFSET ?3"
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map(params![_project_id, page_size, offset], |row| {
                Ok(Document {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    original_path: row.get(2)?,
                    library_path: row.get(3)?,
                    file_type: row.get(4)?,
                    size: row.get(5)?,
                    status: row.get(6)?,
                    error_msg: row.get(7)?,
                    project_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| AppError::Internal(format!("Failed to query documents: {}", e)))?;

        for row in rows {
            documents
                .push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at 
                 FROM documents 
                 ORDER BY created_at DESC 
                 LIMIT ?1 OFFSET ?2"
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map(params![page_size, offset], |row| {
                Ok(Document {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    original_path: row.get(2)?,
                    library_path: row.get(3)?,
                    file_type: row.get(4)?,
                    size: row.get(5)?,
                    status: row.get(6)?,
                    error_msg: row.get(7)?,
                    project_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| AppError::Internal(format!("Failed to query documents: {}", e)))?;

        for row in rows {
            documents
                .push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
        }
    }

    let total = get_documents_count(conn, project_id)?;

    Ok((documents, total))
}

pub fn get_documents_count(conn: &Connection, project_id: Option<&str>) -> Result<i64, AppError> {
    let query = if project_id.is_some() {
        "SELECT COUNT(*) FROM documents WHERE project_id = ?1"
    } else {
        "SELECT COUNT(*) FROM documents"
    };

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;

    let count = if let Some(project_id_val) = project_id {
        stmt.query_row(params![project_id_val], |row| row.get(0))
    } else {
        stmt.query_row([], |row| row.get(0))
    }
    .map_err(|e| AppError::Internal(format!("Failed to count documents: {}", e)))?;

    Ok(count)
}
