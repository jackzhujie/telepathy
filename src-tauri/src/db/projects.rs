#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::{params, Connection};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

pub fn init_projects_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create projects table: {}", e)))?;
    Ok(())
}

pub fn insert_project(conn: &Connection, project: &Project) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO projects (id, name, description, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            project.id,
            project.name,
            project.description,
            project.created_at,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert project: {}", e)))?;
    Ok(())
}

pub fn get_all_projects(conn: &Connection) -> Result<Vec<Project>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, name, description, created_at FROM projects ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query projects: {}", e)))?;

    let mut projects = Vec::new();
    for row in rows {
        projects.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }
    Ok(projects)
}

pub fn update_project(
    conn: &Connection,
    id: &str,
    name: &str,
    description: Option<&str>,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE projects SET name = ?1, description = ?2 WHERE id = ?3",
        params![name, description, id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to update project: {}", e)))?;
    Ok(())
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(format!("Failed to delete project: {}", e)))?;
    Ok(())
}
