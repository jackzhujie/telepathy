use crate::db::{self, settings};
use crate::errors::AppError;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn get_settings(app_handle: AppHandle) -> Result<HashMap<String, String>, AppError> {
    let settings_cache = app_handle.state::<std::sync::Arc<crate::db::settings_cache::SettingsCache>>();
    Ok(settings_cache.get_all())
}

#[tauri::command]
pub async fn update_setting_cmd(
    key: String,
    value: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;
    let settings_cache = app_handle.state::<std::sync::Arc<crate::db::settings_cache::SettingsCache>>();
    settings::init_settings_table(&conn)?;
    settings::update_setting(&conn, Some(&settings_cache), &key, &value)
}

#[tauri::command]
pub async fn is_gpu_supported() -> Result<bool, AppError> {
    println!("[GPU] is_gpu_supported() called");
    
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output();

        if let Ok(output) = output {
            let brand = String::from_utf8_lossy(&output.stdout).to_lowercase();
            println!("[GPU] CPU brand string: {}", brand);
            if brand.contains("apple") || brand.contains("m1") || brand.contains("m2") || brand.contains("m3") || brand.contains("m4") {
                println!("[GPU] Apple Silicon detected, enabling GPU acceleration");
                return Ok(true);
            } else {
                println!("[GPU] Intel Mac detected, trying Metal acceleration");
                return Ok(true);
            }
        }
        
        println!("[GPU] GPU detection failed, defaulting to enabled");
        Ok(true)
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("[GPU] Non-macOS platform, GPU support depends on Vulkan/CUDA features");
        #[cfg(any(feature = "vulkan", feature = "cuda"))]
        {
            println!("[GPU] Vulkan or CUDA feature enabled");
            Ok(true)
        }
        #[cfg(not(any(feature = "vulkan", feature = "cuda")))]
        {
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn debug_gpu_support() -> Result<String, AppError> {
    let result = is_gpu_supported().await?;
    Ok(format!("is_gpu_supported returned: {}", result))
}

pub fn get_gguf_block_count(path: &str) -> Option<u32> {
    use std::fs::File;
    use std::io::Read;
    if let Ok(mut file) = File::open(path) {
        let mut buf = vec![0u8; 256 * 1024];
        if file.read_exact(&mut buf).is_ok() {
            if let Some(pos) = buf.windows(11).position(|w| w == b"block_count") {
                for i in pos + 11..pos + 48 {
                    if i + 4 <= buf.len() {
                        let val = u32::from_le_bytes([buf[i], buf[i+1], buf[i+2], buf[i+3]]);
                        if val >= 12 && val <= 160 {
                            return Some(val);
                        }
                    }
                }
            }
        }
    }
    None
}

#[tauri::command]
pub async fn get_model_max_layers(model_path: String) -> Result<u32, AppError> {
    if let Some(layers) = get_gguf_block_count(&model_path) {
        Ok(layers)
    } else {
        Ok(33)
    }
}

#[tauri::command]
pub async fn get_current_model_max_layers(app_handle: tauri::AppHandle) -> Result<u32, AppError> {
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;
    let settings_cache = app_handle.state::<std::sync::Arc<crate::db::settings_cache::SettingsCache>>();
    
    let chat_model = settings::get_setting_cached(&conn, &settings_cache, "chat_model")?
        .unwrap_or_else(|| "qwen2.5".to_string());

    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get data dir: {}", e)))?;
    let model_path = app_data_dir.join("downloads").join(&chat_model);

    if let Some(layers) = get_gguf_block_count(model_path.to_str().unwrap_or("")) {
        Ok(layers)
    } else {
        Ok(60) // 缺省上限，如果没有解析成功，允许最高到60
    }
}

#[allow(dead_code)]
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
