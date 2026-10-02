use crate::db::documents::{self, Document};
use crate::errors::AppError;
use crate::services::{parser, storage};
use std::path::Path;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[tauri::command]
pub async fn import_document(
    file_path: String,
    project_id: Option<String>,
    app_handle: AppHandle,
) -> Result<Document, AppError> {
    let source = Path::new(&file_path);

    let name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let extension = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let size = std::fs::metadata(source)
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    let library_path = storage::clone_to_library(&app_handle, source)?;

    let doc = Document {
        id: Uuid::new_v4().to_string(),
        name,
        original_path: file_path,
        library_path: library_path.to_string_lossy().to_string(),
        file_type: extension,
        size,
        status: "pending".to_string(),
        error_msg: None,
        project_id,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    documents::init_documents_table(&conn)?;
    documents::insert_document(&conn, &doc)?;

    Ok(doc)
}

#[tauri::command]
pub async fn get_documents(
    project_id: Option<String>,
    app_handle: AppHandle,
) -> Result<Vec<Document>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    documents::init_documents_table(&conn)?;

    match project_id {
        Some(pid) => documents::get_documents_by_project(&conn, &pid),
        None => documents::get_all_documents(&conn),
    }
}

#[tauri::command]
pub async fn get_documents_paginated(
    project_id: Option<String>,
    page: i32,
    page_size: i32,
    app_handle: AppHandle,
) -> Result<crate::models::PaginatedResult<Document>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    documents::init_documents_table(&conn)?;

    let (items, total) =
        documents::get_documents_paginated(&conn, project_id.as_deref(), page, page_size)?;
    Ok(crate::models::PaginatedResult::new(
        items, total, page, page_size,
    ))
}

#[tauri::command]
pub async fn delete_document_cmd(doc_id: String, app_handle: AppHandle) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    if let Some(doc) = documents::get_document(&conn, &doc_id)? {
        storage::remove_from_library(Path::new(&doc.library_path))?;
    }

    documents::delete_document(&conn, &doc_id)
}

#[tauri::command]
pub async fn parse_document(doc_id: String, app_handle: AppHandle) -> Result<String, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    let doc = documents::get_document(&conn, &doc_id)?
        .ok_or_else(|| AppError::Internal("Document not found".to_string()))?;

    documents::update_document_status(&conn, &doc_id, "parsing", None)?;

    let path = Path::new(&doc.library_path);
    let result = parser::parse_document_async(path, &app_handle).await;

    match result {
        Ok(text) => {
            documents::update_document_status(&conn, &doc_id, "done", None)?;
            Ok(text)
        }
        Err(e) => {
            documents::update_document_status(&conn, &doc_id, "error", Some(&e.to_string()))?;
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn is_sidecar_installed_cmd(app_handle: AppHandle) -> Result<bool, AppError> {
    Ok(parser::sidecar::is_sidecar_installed(&app_handle))
}

#[tauri::command]
pub async fn scan_folder(folder_path: String) -> Result<Vec<String>, AppError> {
    let supported_extensions = ["pdf", "md", "txt", "docx", "xlsx", "pptx", "xls", "ppt"];
    let mut files = Vec::new();

    fn scan_dir(dir: &Path, files: &mut Vec<String>, extensions: &[&str]) -> Result<(), AppError> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| AppError::Internal(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| AppError::Internal(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if path.is_dir() {
                scan_dir(&path, files, extensions)?;
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                if extensions.contains(&ext_str.as_str()) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
        Ok(())
    }

    scan_dir(Path::new(&folder_path), &mut files, &supported_extensions)?;
    Ok(files)
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
