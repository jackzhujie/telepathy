#![allow(dead_code)]
use crate::db::profile::{
    add_interest as db_add_interest, get_interests, get_profile_value, init_profile_table,
    remove_interest as db_remove_interest, set_profile_value, UserProfile,
};
use crate::errors::AppError;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

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

fn get_db_connection(db_path: &std::path::Path) -> Result<Connection, AppError> {
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    let _ = init_profile_table(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to init profile table: {}", e)));

    Ok(conn)
}

#[tauri::command]
pub async fn get_user_profile(app_handle: AppHandle) -> Result<UserProfile, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    let name = get_profile_value(&conn, "name")
        .map_err(|e| AppError::Internal(format!("Failed to get name: {}", e)))?
        .unwrap_or_default();
    let gender = get_profile_value(&conn, "gender")
        .map_err(|e| AppError::Internal(format!("Failed to get gender: {}", e)))?
        .unwrap_or_default();
    let age_group = get_profile_value(&conn, "age_group")
        .map_err(|e| AppError::Internal(format!("Failed to get age_group: {}", e)))?
        .unwrap_or_default();
    let occupation = get_profile_value(&conn, "occupation")
        .map_err(|e| AppError::Internal(format!("Failed to get occupation: {}", e)))?
        .unwrap_or_default();
    let industry = get_profile_value(&conn, "industry")
        .map_err(|e| AppError::Internal(format!("Failed to get industry: {}", e)))?
        .unwrap_or_default();
    let language = get_profile_value(&conn, "language")
        .map_err(|e| AppError::Internal(format!("Failed to get language: {}", e)))?
        .unwrap_or("auto".to_string());

    Ok(UserProfile {
        name,
        gender,
        age_group,
        occupation,
        industry,
        language,
    })
}

#[tauri::command]
pub async fn update_user_profile(
    app_handle: AppHandle,
    profile: UserProfile,
) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    set_profile_value(&conn, "name", &profile.name)
        .map_err(|e| AppError::Internal(format!("Failed to set name: {}", e)))?;
    set_profile_value(&conn, "gender", &profile.gender)
        .map_err(|e| AppError::Internal(format!("Failed to set gender: {}", e)))?;
    set_profile_value(&conn, "age_group", &profile.age_group)
        .map_err(|e| AppError::Internal(format!("Failed to set age_group: {}", e)))?;
    set_profile_value(&conn, "occupation", &profile.occupation)
        .map_err(|e| AppError::Internal(format!("Failed to set occupation: {}", e)))?;
    set_profile_value(&conn, "industry", &profile.industry)
        .map_err(|e| AppError::Internal(format!("Failed to set industry: {}", e)))?;
    set_profile_value(&conn, "language", &profile.language)
        .map_err(|e| AppError::Internal(format!("Failed to set language: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn get_user_interests(app_handle: AppHandle) -> Result<Vec<String>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    let interests = get_interests(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to get interests: {}", e)))?;

    Ok(interests)
}

#[tauri::command]
pub async fn add_user_interest(app_handle: AppHandle, interest: String) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    db_add_interest(&conn, &interest)
        .map_err(|e| AppError::Internal(format!("Failed to add interest: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn remove_user_interest(app_handle: AppHandle, interest: String) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    db_remove_interest(&conn, &interest)
        .map_err(|e| AppError::Internal(format!("Failed to remove interest: {}", e)))?;

    Ok(())
}
