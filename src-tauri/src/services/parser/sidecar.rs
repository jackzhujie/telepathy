#![allow(dead_code)]
use crate::errors::AppError;
use std::path::Path;
use tauri::{AppHandle, Manager};

/// Check if the advanced parser sidecar is installed
pub fn is_sidecar_installed(app_handle: &AppHandle) -> bool {
    get_sidecar_path(app_handle)
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Get the expected path to the sidecar executable
pub fn get_sidecar_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;

    Ok(app_data_dir.join("sidecar").join("parser-advanced"))
}

/// Check if a file type requires the sidecar
pub fn requires_sidecar(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    matches!(
        extension.as_str(),
        "docx" | "doc" | "pptx" | "ppt" | "xlsx" | "xls"
    )
}

/// Parse a file using the sidecar (placeholder for future implementation)
pub async fn parse_with_sidecar(app_handle: &AppHandle, path: &Path) -> Result<String, AppError> {
    if !is_sidecar_installed(app_handle) {
        return Err(AppError::Internal(
            "Advanced parser sidecar not installed. Please install it from Settings.".to_string(),
        ));
    }

    let sidecar_path = get_sidecar_path(app_handle)?;
    let output = tokio::process::Command::new(&sidecar_path)
        .arg(path.to_string_lossy().as_ref())
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to run sidecar: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Internal(format!(
            "Sidecar parsing failed: {}",
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 调用 Sidecar 将 PDF 页面渲染为临时图片输出到指定文件夹下
/// 运行命令: parser-advanced --render-pdf-pages <pdf_path> <temp_dir>
pub async fn render_pdf_pages_via_sidecar(
    app_handle: &AppHandle,
    pdf_path: &Path,
    temp_dir: &Path,
) -> Result<usize, AppError> {
    if !is_sidecar_installed(app_handle) {
        return Err(AppError::Internal(
            "高级解析器 Sidecar 未安装，无法执行 PDF 页面视觉渲染。".to_string(),
        ));
    }

    let sidecar_path = get_sidecar_path(app_handle)?;
    let output = tokio::process::Command::new(&sidecar_path)
        .arg("--render-pdf-pages")
        .arg(pdf_path.to_string_lossy().as_ref())
        .arg(temp_dir.to_string_lossy().as_ref())
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to run sidecar for rendering: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Internal(format!(
            "Sidecar PDF rendering failed: {}",
            stderr
        )));
    }

    // 从输出文本中尝试获取渲染的总页数，如果无法获取，直接读取输出文件夹中图片数量
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("[Sidecar] Render PDF output: {}", stdout);

    let files = std::fs::read_dir(temp_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read temp page output dir: {}", e)))?;

    let count = files
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .map(|ext| ext.to_string_lossy().to_lowercase() == "jpg" || ext.to_string_lossy().to_lowercase() == "png")
                .unwrap_or(false)
        })
        .count();

    Ok(count)
}

