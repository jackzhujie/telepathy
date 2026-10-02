use crate::errors::AppError;

#[tauri::command]
pub fn greet(name: &str) -> Result<String, AppError> {
    Ok(format!("你好，{}！Telepathy 已准备就绪。", name))
}
