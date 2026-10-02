use crate::db::projects;
use crate::errors::AppError;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateProjectInput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn create_project(
    input: CreateProjectInput,
    app_handle: AppHandle,
) -> Result<projects::Project, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    projects::init_projects_table(&conn)?;

    let project = projects::Project {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        description: input.description,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    projects::insert_project(&conn, &project)?;
    Ok(project)
}

#[tauri::command]
pub async fn list_projects(app_handle: AppHandle) -> Result<Vec<projects::Project>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    projects::init_projects_table(&conn)?;
    projects::get_all_projects(&conn)
}

#[tauri::command]
pub async fn update_project(
    input: UpdateProjectInput,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    projects::update_project(&conn, &input.id, &input.name, input.description.as_deref())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_project(id: String, app_handle: AppHandle) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    projects::delete_project(&conn, &id)?;
    Ok(())
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
