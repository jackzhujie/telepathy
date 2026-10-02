use crate::db::snaps::{self, Snap};
use crate::errors::AppError;
use crate::models::PaginatedResult;
use chrono::Local;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

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

#[tauri::command]
pub fn get_snaps_paginated(
    page: i32,
    page_size: i32,
    app: AppHandle,
) -> Result<PaginatedResult<Snap>, AppError> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    snaps::init_snaps_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init snaps table: {}", e)))?;

    let offset = (page - 1) * page_size;
    let total = snaps::get_total_count(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to get total count: {}", e)))?
        as i64;

    let items = snaps::get_snaps_paginated(&conn, page_size, offset)
        .map_err(|e| AppError::Internal(format!("Failed to get items: {}", e)))?;

    Ok(PaginatedResult::new(items, total, page, page_size))
}

#[tauri::command]
pub async fn create_snap(
    content: String,
    tags: Option<String>,
    app: AppHandle,
) -> Result<Snap, AppError> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    snaps::init_snaps_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init snaps table: {}", e)))?;

    let id = Uuid::new_v4().to_string();
    let now = Local::now().to_rfc3339();

    let snap = Snap {
        id,
        content: content.clone(),
        tags,
        is_pinned: false,
        created_at: now.clone(),
        updated_at: now,
    };

    snaps::insert_snap(&conn, &snap)
        .map_err(|e| AppError::Internal(format!("Failed to create snap: {}", e)))?;

    // 背景向量化任务
    let snap_clone = snap.clone();
    let db_path_clone = db_path.clone();
    tokio::spawn(async move {
        if let Ok(conn) = rusqlite::Connection::open(&db_path_clone) {
            let embedding_model = crate::db::settings::get_setting(&conn, "embedding_model")
                .unwrap_or_default()
                .unwrap_or_else(|| "bge-m3:latest".to_string());

            let app_data_dir = app.path().app_data_dir().unwrap_or_default();
            let embedding_model_path = app_data_dir.join("downloads").join(&embedding_model);
            let embedder =
                crate::services::embedder::Embedder::new(&embedding_model_path.to_string_lossy());
            if let Ok(vector) = embedder.embed(&snap_clone.content).await {
                let _ = snaps::insert_snap_embedding(&conn, &snap_clone.id, &vector);
            }
        }
    });

    Ok(snap)
}

#[tauri::command]
pub async fn update_snap(
    id: String,
    content: String,
    tags: Option<String>,
    is_pinned: bool,
    app: AppHandle,
) -> Result<Snap, AppError> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    snaps::init_snaps_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init snaps table: {}", e)))?;

    let now = Local::now().to_rfc3339();

    let snap = Snap {
        id: id.clone(),
        content: content.clone(),
        tags,
        is_pinned,
        created_at: "".to_string(),
        updated_at: now,
    };

    snaps::update_snap(&conn, &snap)
        .map_err(|e| AppError::Internal(format!("Failed to update snap: {}", e)))?;

    // 背景更新向量索引
    let db_path_clone = db_path.clone();
    tokio::spawn(async move {
        if let Ok(conn) = rusqlite::Connection::open(&db_path_clone) {
            let embedding_model = crate::db::settings::get_setting(&conn, "embedding_model")
                .unwrap_or_default()
                .unwrap_or_else(|| "bge-large-zh".to_string());

            let app_data_dir = app.path().app_data_dir().unwrap_or_default();
            let embedding_model_path = app_data_dir.join("downloads").join(&embedding_model);
            let embedder =
                crate::services::embedder::Embedder::new(&embedding_model_path.to_string_lossy());
            if let Ok(vector) = embedder.embed(&content).await {
                let _ = snaps::insert_snap_embedding(&conn, &id, &vector);
            }
        }
    });

    Ok(snap)
}

#[tauri::command]
pub fn delete_snap(id: String, app: AppHandle) -> Result<(), AppError> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    snaps::init_snaps_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init snaps table: {}", e)))?;

    snaps::delete_snap(&conn, &id)
        .map_err(|e| AppError::Internal(format!("Failed to delete snap: {}", e)))
}
