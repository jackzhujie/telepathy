use crate::db::memories;
use crate::errors::AppError;
use tauri::AppHandle;
use tauri::Manager;

#[tauri::command]
pub async fn get_memories(app_handle: AppHandle) -> Result<Vec<memories::Memory>, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    let db_path = app_data_dir.join("telepathy.db");

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    memories::get_all_memories(&conn)
}

#[tauri::command]
pub async fn delete_memory(id: String, app_handle: AppHandle) -> Result<(), AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    let db_path = app_data_dir.join("telepathy.db");

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    memories::delete_memory(&conn, &id)
}

#[tauri::command]
pub async fn manual_memory_extraction(
    conversation_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    let db_path = app_data_dir.join("telepathy.db");

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    let messages = crate::db::conversations::get_messages(&conn, &conversation_id)?;
    if messages.len() < 2 {
        return Err(AppError::Internal(
            "Not enough messages to extract memory".to_string(),
        ));
    }

    // Get last Q&A pair
    let ai_response = messages.iter().rev().find(|m| m.role == "assistant");
    let user_query = messages.iter().rev().find(|m| m.role == "user");

    if let (Some(u), Some(a)) = (user_query, ai_response) {
        let chat_model = crate::db::settings::get_setting(&conn, "chat_model")?
            .unwrap_or_else(|| "qwen3:1.7b".to_string());
        let embedding_model = crate::db::settings::get_setting(&conn, "embedding_model")?
            .unwrap_or_else(|| "bge-m3:latest".to_string());

        crate::services::memory_service::extract_and_save_memory(
            &app_handle,
            &u.content,
            &a.content,
            &chat_model,
            &embedding_model,
        )
        .await?;
    }

    Ok(())
}
