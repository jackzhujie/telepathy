#![allow(dead_code)]
use crate::errors::AppError;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub fn get_library_dir(app_handle: &AppHandle) -> Result<PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;

    let library_dir = app_data_dir.join("library");
    if !library_dir.exists() {
        fs::create_dir_all(&library_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create library dir: {}", e)))?;
    }
    Ok(library_dir)
}

pub fn clone_to_library(app_handle: &AppHandle, source_path: &Path) -> Result<PathBuf, AppError> {
    let library_dir = get_library_dir(app_handle)?;

    let extension = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let new_filename = if extension.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        format!("{}.{}", Uuid::new_v4(), extension)
    };

    let dest_path = library_dir.join(&new_filename);

    fs::copy(source_path, &dest_path)
        .map_err(|e| AppError::Internal(format!("Failed to copy file to library: {}", e)))?;

    Ok(dest_path)
}

pub fn remove_from_library(library_path: &Path) -> Result<(), AppError> {
    if library_path.exists() {
        fs::remove_file(library_path).map_err(|e| {
            AppError::Internal(format!("Failed to remove file from library: {}", e))
        })?;
    }
    Ok(())
}
