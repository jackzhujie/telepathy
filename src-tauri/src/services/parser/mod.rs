pub mod core;
pub mod sidecar;
pub mod vision;

use crate::errors::AppError;
use std::path::Path;
use tauri::{AppHandle, Manager};


/// Unified entry point for asynchronous document parsing
pub async fn parse_document_async(path: &Path, app_handle: &AppHandle) -> Result<String, AppError> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let cache = app_handle.state::<std::sync::Arc<crate::db::settings_cache::SettingsCache>>();
    let use_vision_parser = cache.get("use_vision_parser").map(|s| s == "true").unwrap_or(false);
    let enable_office_parser = cache.get("enable_office_parser").map(|s| s == "true").unwrap_or(false);

    match extension.as_str() {
        "txt" | "md" | "json" | "csv" => {
            let path_clone = path.to_path_buf();
            tokio::task::spawn_blocking(move || core::parse_text_file(&path_clone))
                .await
                .map_err(|e| AppError::Internal(format!("Task failed: {}", e)))?
        }
        "docx" | "xlsx" | "xls" | "pptx" => {
            if !enable_office_parser {
                return Err(AppError::Internal(format!(
                    "该文档为高级 Office 格式（.{}）。当前未开启『高级文档解析扩展』，请前往『系统设置』开启该功能以开始解析和检索。",
                    extension
                )));
            }
            let path_clone = path.to_path_buf();
            let ext_clone = extension.clone();
            tokio::task::spawn_blocking(move || {
                match ext_clone.as_str() {
                    "docx" => core::parse_docx(&path_clone),
                    "xlsx" | "xls" => core::parse_excel(&path_clone),
                    "pptx" => core::parse_pptx(&path_clone),
                    _ => unreachable!(),
                }
            })
            .await
            .map_err(|e| AppError::Internal(format!("Task failed: {}", e)))?
        }
        "doc" | "ppt" => {
            let is_sidecar_installed = sidecar::is_sidecar_installed(app_handle);
            if is_sidecar_installed {
                sidecar::parse_with_sidecar(app_handle, path).await
            } else {
                Err(AppError::Internal(format!(
                    "当前文件为老旧二进制格式（.{}），原生解析器暂不支持直接解析。请将其转换为更新的 .docx / .pptx 格式，或者在『系统设置』中下载并安装高级解析插件。",
                    extension
                )))
            }
        }
        "pdf" => {
            let is_sidecar_installed = sidecar::is_sidecar_installed(app_handle);
            if use_vision_parser && is_sidecar_installed {
                vision::parse_pdf_with_vision(path, app_handle).await
            } else {
                let path_clone = path.to_path_buf();
                tokio::task::spawn_blocking(move || core::parse_pdf(&path_clone))
                    .await
                    .map_err(|e| AppError::Internal(format!("Task failed: {}", e)))?
            }
        }
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => {
            if use_vision_parser {
                vision::parse_image_with_vision(path, app_handle).await
            } else {
                Err(AppError::Internal(
                    "该文件为图片格式，需在系统设置中启用『文档多模态视觉解析』才能进行解析和索引。".to_string(),
                ))
            }
        }
        _ => Err(AppError::Internal(format!(
            "Unsupported file type: .{}",
            extension
        ))),
    }
}

