pub mod conversations;
pub mod documents;
pub mod memories;
pub mod notifications;
pub mod profile;
pub mod projects;
pub mod settings;
pub mod settings_cache;
pub mod snaps;
pub mod vectors;

use crate::errors::AppError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Global database state managed by Tauri
pub struct DbState {
    pub db_path: PathBuf,
    pub connection: Arc<Mutex<Connection>>,
}

/// Helper: get a connection from the shared state
pub async fn open_connection(state: &DbState) -> Result<tokio::sync::MutexGuard<'_, Connection>, AppError> {
    Ok(state.connection.lock().await)
}

/// Initialize all database tables. Called once at app startup.
pub fn init_all_tables(conn: &Connection) -> Result<(), AppError> {
    conversations::init_conversations_table(conn)?;
    vectors::init_vector_tables(conn)?;
    settings::init_settings_table(conn)?;
    settings::seed_defaults(conn)?;
    documents::init_documents_table(conn)?;
    memories::init_memory_tables(conn)?;
    notifications::init_notifications_table(conn)
        .map_err(|e| AppError::Internal(format!("Failed to init notifications table: {}", e)))?;
    profile::init_profile_table(conn)
        .map_err(|e| AppError::Internal(format!("Failed to init profile table: {}", e)))?;
    projects::init_projects_table(conn)?;
    snaps::init_snaps_table(conn)
        .map_err(|e| AppError::Internal(format!("Failed to init snaps table: {}", e)))?;
    Ok(())
}
