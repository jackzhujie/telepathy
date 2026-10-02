use crate::db::notifications;
use crate::errors::AppError;
use crate::services::model_hub::{
    geo::GeoSensor, manager::ModelHubManager, recommender::Recommender, HardwareProfile, HubModel,
    HubRecommendation, PrimaryRecommendations,
};
use crate::services::model_registry::{
    self, InstalledModel, ModelRecommendation, ModelType, RegistryModel,
};

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Serialize)]
pub struct ModelHubResponse {
    pub hardware: HardwareProfile,
    pub recommendations: Vec<HubRecommendation>,
    pub primary_recommendations: PrimaryRecommendations,
    pub all_models: Vec<HubModel>,
    pub leaderboard: Vec<HubModel>,
    pub trending: Vec<HubModel>,
    pub is_china: bool,
    pub cache_timestamp: u64,
}

#[tauri::command]
pub async fn search_hub_models(query: String) -> Result<Vec<HubModel>, AppError> {
    use crate::services::model_hub::hf::HuggingFaceRegistry;
    use crate::services::model_hub::ms::ModelScopeRegistry;
    use crate::services::model_hub::registry::ModelRegistryClient;

    let is_china = GeoSensor::is_likely_in_china().await;

    let registry: Box<dyn ModelRegistryClient> = if is_china {
        Box::new(ModelScopeRegistry::new())
    } else {
        Box::new(HuggingFaceRegistry::new())
    };

    registry.search(&query).await
}

/// Lazily fetch the GGUF variants for a single model on user demand (e.g., when they expand a row).
#[tauri::command]
pub async fn get_model_variants(
    model_id: String,
) -> Result<Vec<crate::services::model_hub::ModelVariant>, AppError> {
    use crate::services::model_hub::hf::HuggingFaceRegistry;
    use crate::services::model_hub::ms::ModelScopeRegistry;
    use crate::services::model_hub::registry::ModelRegistryClient;

    let is_china = GeoSensor::is_likely_in_china().await;

    let registry: Box<dyn ModelRegistryClient> = if is_china {
        Box::new(ModelScopeRegistry::new())
    } else {
        Box::new(HuggingFaceRegistry::new())
    };

    registry.get_variants(&model_id).await
}

#[tauri::command]
pub async fn get_model_hub(
    app_handle: AppHandle,
    force_refresh: bool,
) -> Result<ModelHubResponse, AppError> {
    // 1. 获取硬件概览
    let hardware = Recommender::get_hardware_profile();

    let is_china = GeoSensor::is_likely_in_china().await;

    // 3. 获取/刷新模型库
    let hub_cache = ModelHubManager::get_models(&app_handle, force_refresh, is_china)
        .await
        .map_err(|e| AppError::Internal(e))?;

    // 4. 计算推荐
    let recommendations = Recommender::get_recommendations(&hardware, &hub_cache);
    let primary_recommendations = Recommender::compute_primary_recommendations(&hardware, &hub_cache);

    // 4.5 获取排行榜（与全量列表使用同一数据源）
    let (leaderboard, trending) = ModelHubManager::get_rankings(&hub_cache, is_china).await;

    // 4.6 准备全量模型列表 (按名称排序)
    let mut all_models: Vec<HubModel> = hub_cache.values().cloned().collect();
    all_models.sort_by(|a, b| a.name.cmp(&b.name));

    // 5. 获取缓存时间戳
    let cache_timestamp = ModelHubManager::get_cache_timestamp(&app_handle);

    Ok(ModelHubResponse {
        hardware,
        recommendations,
        primary_recommendations,
        all_models,
        leaderboard,
        trending,
        is_china,
        cache_timestamp,
    })
}

#[tauri::command]
pub async fn list_registry_models(
    model_type: Option<String>,
) -> Result<Vec<RegistryModel>, AppError> {
    let registry = model_registry::load_registry()?;
    match model_type {
        Some(t) => {
            let mt = match t.to_lowercase().as_str() {
                "chat" => ModelType::Chat,
                "embedding" => ModelType::Embedding,
                "vision" => ModelType::Vision,
                _ => return Ok(registry.models),
            };
            Ok(model_registry::filter_by_type(&registry, &mt))
        }
        None => Ok(registry.models),
    }
}

#[tauri::command]
pub async fn list_installed_models(app_handle: AppHandle) -> Result<Vec<InstalledModel>, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let downloads_dir = app_data_dir.join("downloads");

    println!("[Models] Scanning for models in: {:?}", downloads_dir);

    let mut installed = Vec::new();
    let registry = model_registry::load_registry()?;

    if downloads_dir.exists() {
        let mut entries = tokio::fs::read_dir(&downloads_dir)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        while let Some(entry_res) = entries.next_entry().await.transpose() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("[Models] Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            let is_gguf = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("gguf"))
                .unwrap_or(false);

            if is_gguf {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);

                println!("[Models] Found model: {}", file_name);

                let (model_type, display_name, provider, description) =
                    model_registry::match_model_type(&registry, &file_name);

                installed.push(InstalledModel {
                    full_name: file_name.clone(),
                    base_name: file_name.clone(),
                    engine_type: "LlamaCpp".to_string(),
                    format: "GGUF".to_string(),
                    file_path: path.to_string_lossy().to_string(),
                    model_type,
                    size_bytes: size,
                    display_name,
                    provider,
                    description,
                });
            }
        }
    } else {
        println!("[Models] Downloads directory does not exist yet.");
    }

    println!("[Models] Total detected models: {}", installed.len());
    Ok(installed)
}

#[tauri::command]
pub async fn get_model_recommendations(
    _app_handle: AppHandle,
) -> Result<Vec<ModelRecommendation>, AppError> {
    let registry = model_registry::load_registry()?;

    // Get system memory via sysinfo
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0;

    Ok(model_registry::get_recommendations(
        &registry,
        total_memory_gb,
    ))
}

#[tauri::command]
pub async fn install_model(
    model_name: String,
    variant: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    use crate::services::model_hub::downloader::ModelDownloader;
    use crate::services::model_hub::geo::GeoSensor;
    use crate::services::model_hub::hf::HuggingFaceRegistry;
    use crate::services::model_hub::ms::ModelScopeRegistry;
    use crate::services::model_hub::registry::ModelRegistryClient;

    let db_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?
        .join("telepathy.db");


    // Identify Source
    let is_china = GeoSensor::is_likely_in_china().await;

    let registry: Box<dyn ModelRegistryClient> = if is_china {
        Box::new(ModelScopeRegistry::new())
    } else {
        Box::new(HuggingFaceRegistry::new())
    };

    // Fetch variants to get expected size
    let variants = registry.get_variants(&model_name).await.map_err(|e| {
        AppError::Internal(format!(
            "Failed to fetch variants for {}: {}",
            model_name, e
        ))
    })?;

    if variants.is_empty() {
        return Err(AppError::Internal(
            "No GGUF variants found for this model".into(),
        ));
    }

    let mut expected_size = 0;
    let mut actual_variant = variant.clone();

    // Automatically resolve "latest" to the best available GGUF variant
    if actual_variant == "latest" {
        // Filter out vision projectors and other auxiliary GGUF files first
        let chat_variants: Vec<_> = variants
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                let t = v.tag.to_lowercase();
                !t.contains("mmproj") && !t.contains("projector")
            })
            .collect();

        if chat_variants.is_empty() {
            return Err(AppError::Internal(
                "No suitable chat GGUF variants found for this model (only projector files found)"
                    .into(),
            ));
        }

        let mut best_priority = -1;
        let mut best_idx = chat_variants[0].0;
        for (orig_idx, v) in &chat_variants {
            let t = v.tag.to_lowercase();
            let priority = if t.contains("q4_k_m") {
                3
            } else if t.contains("q5_k_m") {
                2
            } else if t.contains("q4_0") {
                1
            } else {
                0
            };
            if priority > best_priority {
                best_priority = priority;
                best_idx = *orig_idx;
            }
        }
        actual_variant = variants[best_idx].tag.clone();
        expected_size = variants[best_idx].size;
    } else {
        // Find matching variant size
        for v in &variants {
            if v.tag == actual_variant {
                expected_size = v.size;
                break;
            }
        }
    }

    let download_url = registry.get_download_url(&model_name, &actual_variant);
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;

    // Extract a sanitized safe name for the file system
    let safe_model_name = model_name
        .split('/')
        .last()
        .unwrap_or(&model_name)
        .replace("-", "_")
        .replace(".", "_")
        .to_lowercase();
    let safe_variant_name = actual_variant
        .replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_");

    // Safety check: Block sharded files if they somehow reached here
    if regex::Regex::new(r"(?i)(-\d{5}-of-\d{5}|-split-|part\d+|shard\d+)")
        .map(|re| re.is_match(&safe_variant_name))
        .unwrap_or(false)
    {
        return Err(AppError::Internal(
            "This model is sharded (split into multiple parts). Currently, we only support single-file GGUF installations. Please choose a different quantization version.".into()
        ));
    }

    // Create a clean display tag
    let safe_variant_tag = safe_variant_name
        .to_lowercase()
        .replace(".gguf", "")
        .replace(".bin", "");
    let model_tag = format!("{}:{}", safe_model_name, safe_variant_tag);

    let normalized_variant = safe_variant_name
        .to_lowercase()
        .replace("-", "_")
        .replace(".", "_");
    let safe_variant_name_prefixed = if normalized_variant.starts_with(&safe_model_name) {
        safe_variant_name.clone()
    } else {
        format!("{}_{}", safe_model_name, safe_variant_name)
    };

    let dest_filename = if safe_variant_name_prefixed.to_lowercase().ends_with(".gguf") {
        safe_variant_name_prefixed
    } else {
        format!("{}.gguf", safe_variant_name_prefixed)
    };
    let final_path = app_data_dir.join("downloads").join(&dest_filename);
    let temp_path = app_data_dir.join("downloads").join(format!("{}.part", dest_filename));

    let registration_id = format!("{}:{}", model_name, actual_variant);
    let downloader = ModelDownloader::new();
    let download_res = downloader
        .download(
            &download_url,
            &temp_path, // Pass the temporary path
            &registration_id,
            expected_size,
            &app_handle,
        )
        .await;

    if let Err(e) = download_res {
        let err_msg = e.to_string();
        // Cleanup partial download temp file
        let _ = tokio::fs::remove_file(&temp_path).await;
        let _ = app_handle.emit(
            "model-pull-error",
            json!({ "model": model_tag, "error": err_msg }),
        );
        let notification = notifications::create_notification(
            "error",
            "模型下载失败",
            &format!("模型 {} 下载失败: {}", model_tag, err_msg),
            Some("/settings"),
        );
        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|db_err| AppError::Internal(format!("Failed to open database: {}", db_err)))?;
        let _ = notifications::insert_notification(&conn, &notification);
        let _ = app_handle.emit("notification-created", &notification);
        return Err(e);
    }

    // Rename temporary file to final .gguf file upon successful download completion
    if let Err(e) = tokio::fs::rename(&temp_path, &final_path).await {
        let err_msg = format!("Failed to rename model file: {}", e);
        let _ = tokio::fs::remove_file(&temp_path).await;
        let _ = app_handle.emit(
            "model-pull-error",
            json!({ "model": model_tag, "error": err_msg }),
        );
        let notification = notifications::create_notification(
            "error",
            "模型下载失败",
            &format!("模型 {} 重命名失败: {}", model_tag, err_msg),
            Some("/settings"),
        );
        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|db_err| AppError::Internal(format!("Failed to open database: {}", db_err)))?;
        let _ = notifications::insert_notification(&conn, &notification);
        let _ = app_handle.emit("notification-created", &notification);
        return Err(AppError::Internal(err_msg));
    }

    let _ = app_handle.emit("model-pull-done", json!({ "model": model_tag }));
    let notification = notifications::create_notification(
        "success",
        "模型安装成功",
        &format!("模型 {} 已成功下载", model_tag),
        Some("/settings"),
    );
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    let _ = notifications::insert_notification(&conn, &notification);
    let _ = app_handle.emit("notification-created", &notification);
    Ok(())
}

#[tauri::command]
pub async fn delete_model(model_name: String, app_handle: AppHandle) -> Result<(), AppError> {
    let path = std::path::Path::new(&model_name);
    if path.components().count() > 1 || model_name.contains("..") {
        return Err(AppError::Internal("非法文件名".into()));
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let dest_path = app_data_dir.join("downloads").join(&model_name);

    if dest_path.exists() {
        tokio::fs::remove_file(dest_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete model file: {}", e)))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn cancel_pull_model(
    model: Option<String>,
    app: tauri::AppHandle,
) -> Result<(), AppError> {
    // Attempt to cancel all tokens or the specific token
    if let Some(state) =
        app.try_state::<crate::services::model_hub::downloader::DownloadManagerState>()
    {
        if let Ok(mut map) = state.0.lock() {
            if let Some(m) = model {
                if let Some(token) = map.remove(&m) {
                    token.cancel();
                } else if let Some((key, token)) = map
                    .iter()
                    .find(|(key, _)| {
                        key.starts_with(&format!("{}:", m.split(':').next().unwrap_or(&m)))
                    })
                    .map(|(key, token)| (key.clone(), token.clone()))
                {
                    map.remove(&key);
                    token.cancel();
                }
            } else {
                for (_, token) in map.drain() {
                    token.cancel();
                }
            }
        }
    }

    Ok(())
}
