use crate::db::{documents, vectors};
use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedDocument {
    pub id: String,
    pub name: String,
    pub file_type: String,
    pub chunk_count: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkDetail {
    pub id: String,
    pub document_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: ChunkDetail,
    pub document_name: String,
}

#[tauri::command]
pub async fn get_indexed_documents(
    project_id: Option<String>,
    app_handle: AppHandle,
) -> Result<Vec<IndexedDocument>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&conn)?;
    vectors::init_vector_tables(&conn)?;

    let documents: Vec<IndexedDocument>;

    if let Some(ref pid) = project_id {
        let mut stmt = conn
            .prepare(
                "SELECT d.id, d.name, d.file_type, d.created_at,
                        (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) as chunk_count
                 FROM documents d
                 WHERE d.status = 'indexed' AND d.project_id = ?1
                 ORDER BY d.created_at DESC",
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![pid], |row| {
                Ok(IndexedDocument {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_type: row.get(2)?,
                    created_at: row.get(3)?,
                    chunk_count: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

        documents = rows.filter_map(|r| r.ok()).collect();
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT d.id, d.name, d.file_type, d.created_at,
                        (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) as chunk_count
                 FROM documents d
                 WHERE d.status = 'indexed'
                 ORDER BY d.created_at DESC",
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(IndexedDocument {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_type: row.get(2)?,
                    created_at: row.get(3)?,
                    chunk_count: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

        documents = rows.filter_map(|r| r.ok()).collect();
    };

    Ok(documents)
}

#[tauri::command]
pub async fn get_document_chunks_detail(
    doc_id: String,
    app_handle: AppHandle,
) -> Result<Vec<ChunkDetail>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    vectors::init_vector_tables(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, document_id, chunk_index, content, created_at
             FROM chunks WHERE document_id = ?1 ORDER BY chunk_index",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(rusqlite::params![doc_id], |row| {
            Ok(ChunkDetail {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_index: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let chunks: Vec<ChunkDetail> = rows.filter_map(|r| r.ok()).collect();

    Ok(chunks)
}

#[tauri::command]
pub async fn search_chunks(
    keyword: String,
    app_handle: AppHandle,
) -> Result<Vec<SearchResult>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&conn)?;
    vectors::init_vector_tables(&conn)?;

    let search_pattern = format!("%{}%", keyword);

    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.document_id, c.chunk_index, c.content, c.created_at, d.name
             FROM chunks c
             JOIN documents d ON c.document_id = d.id
             WHERE c.content LIKE ?1 AND d.status = 'indexed'
             ORDER BY c.created_at DESC
             LIMIT 50",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(rusqlite::params![search_pattern], |row| {
            Ok(SearchResult {
                chunk: ChunkDetail {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    chunk_index: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                },
                document_name: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let results: Vec<SearchResult> = rows.filter_map(|r| r.ok()).collect();

    Ok(results)
}

#[tauri::command]
pub async fn get_indexed_documents_paginated(
    project_id: Option<String>,
    page: i32,
    page_size: i32,
    app_handle: AppHandle,
) -> Result<crate::models::PaginatedResult<IndexedDocument>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&conn)?;
    vectors::init_vector_tables(&conn)?;

    let offset = (page - 1) * page_size;
    let documents: Vec<IndexedDocument>;
    let total: i64;

    if let Some(ref pid) = project_id {
        // 获取总数
        let mut count_stmt = conn
            .prepare(
                "SELECT COUNT(*) FROM documents d WHERE d.status = 'indexed' AND d.project_id = ?1",
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;
        total = count_stmt
            .query_row(rusqlite::params![pid], |row| row.get(0))
            .map_err(|e| AppError::Internal(format!("Failed to count documents: {}", e)))?;

        // 获取分页数据
        let mut stmt = conn
            .prepare(
                "SELECT d.id, d.name, d.file_type, d.created_at,
                        (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) as chunk_count
                 FROM documents d
                 WHERE d.status = 'indexed' AND d.project_id = ?1
                 ORDER BY d.created_at DESC
                 LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![pid, page_size, offset], |row| {
                Ok(IndexedDocument {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_type: row.get(2)?,
                    created_at: row.get(3)?,
                    chunk_count: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

        documents = rows.filter_map(|r| r.ok()).collect();
    } else {
        // 获取总数
        let mut count_stmt = conn
            .prepare("SELECT COUNT(*) FROM documents d WHERE d.status = 'indexed'")
            .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;
        total = count_stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| AppError::Internal(format!("Failed to count documents: {}", e)))?;

        // 获取分页数据
        let mut stmt = conn
            .prepare(
                "SELECT d.id, d.name, d.file_type, d.created_at,
                        (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) as chunk_count
                 FROM documents d
                 WHERE d.status = 'indexed'
                 ORDER BY d.created_at DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![page_size, offset], |row| {
                Ok(IndexedDocument {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_type: row.get(2)?,
                    created_at: row.get(3)?,
                    chunk_count: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

        documents = rows.filter_map(|r| r.ok()).collect();
    };

    Ok(crate::models::PaginatedResult::new(
        documents, total, page, page_size,
    ))
}

#[tauri::command]
pub async fn get_document_chunks_detail_paginated(
    doc_id: String,
    page: i32,
    page_size: i32,
    app_handle: AppHandle,
) -> Result<crate::models::PaginatedResult<ChunkDetail>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    vectors::init_vector_tables(&conn)?;

    let offset = (page - 1) * page_size;

    // 获取总数
    let mut count_stmt = conn
        .prepare("SELECT COUNT(*) FROM chunks WHERE document_id = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare count query: {}", e)))?;
    let total = count_stmt
        .query_row(rusqlite::params![doc_id], |row| row.get(0))
        .map_err(|e| AppError::Internal(format!("Failed to count chunks: {}", e)))?;

    // 获取分页数据
    let mut stmt = conn
        .prepare(
            "SELECT id, document_id, chunk_index, content, created_at
             FROM chunks WHERE document_id = ?1 ORDER BY chunk_index LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;

    let rows = stmt
        .query_map(rusqlite::params![doc_id, page_size, offset], |row| {
            Ok(ChunkDetail {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_index: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;

    let chunks: Vec<ChunkDetail> = rows.filter_map(|r| r.ok()).collect();

    Ok(crate::models::PaginatedResult::new(
        chunks, total, page, page_size,
    ))
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
