use crate::services::model_hub::{HubCache, HubModel, ModelCategory, ModelVariant};
use std::collections::HashMap;
use tauri::Manager;

pub struct ModelHubManager;

impl ModelHubManager {
    pub fn get_cache_path(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
        app_handle
            .path()
            .app_data_dir()
            .unwrap_or_default()
            .join("model_hub_cache.json")
    }

    pub fn get_cache_timestamp(app_handle: &tauri::AppHandle) -> u64 {
        let cache_path = Self::get_cache_path(app_handle);
        if cache_path.exists() {
            std::fs::metadata(cache_path)
                .and_then(|m| m.modified())
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0)
        } else {
            0
        }
    }

    pub async fn get_models(
        app_handle: &tauri::AppHandle,
        force_refresh: bool,
        is_china: bool,
    ) -> Result<HubCache, String> {
        let cache_path = Self::get_cache_path(app_handle);

        if !force_refresh && cache_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cache_path) {
                if let Ok(cache) = serde_json::from_str::<HubCache>(&content) {
                    if let Ok(metadata) = std::fs::metadata(&cache_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified
                                .elapsed()
                                .map(|e| e.as_secs() < 24 * 3600)
                                .unwrap_or(false)
                            {
                                return Ok(cache);
                            }
                        }
                    }
                }
            }
        }

        use crate::services::model_hub::hf::HuggingFaceRegistry;
        use crate::services::model_hub::ms::ModelScopeRegistry;
        use crate::services::model_hub::registry::ModelRegistryClient;

        use std::sync::Arc;
        let (registry, source_name): (Arc<dyn ModelRegistryClient>, &str) = if is_china {
            (Arc::new(ModelScopeRegistry::new()), "ModelScope")
        } else {
            (Arc::new(HuggingFaceRegistry::new()), "HuggingFace")
        };

        println!(
            "[ModelHub] Cache expired or force refresh. Syncing from Registry: {}...",
            source_name
        );
        let start = std::time::Instant::now();

        let trending = registry.get_trending().await.map_err(|e| e.to_string())?;
        let mut models = HashMap::new();
        for m in trending {
            models.insert(m.name.clone(), m);
        }

        println!("[ModelHub] List sync complete ({} models). Pre-fetching variants for top recommendations...", models.len());

        // 1. 注入本地高保真注册表兜底模型，确保基础模型集（如 BGE-M3 等）绝不缺席
        if let Ok(registry) = crate::services::model_registry::load_registry() {
            for rm in registry.models {
                let cat = match rm.model_type {
                    crate::services::model_registry::ModelType::Chat => ModelCategory::Chat,
                    crate::services::model_registry::ModelType::Embedding => ModelCategory::Embedding,
                    crate::services::model_registry::ModelType::Vision => ModelCategory::Vision,
                    crate::services::model_registry::ModelType::Unknown => ModelCategory::Other,
                };
                
                // 将本地 sizes 静态转换为 Hub 变体规格，免除线上网络拉取开销
                let variants = rm.sizes.iter().map(|s| ModelVariant {
                    tag: format!("{}:{}", rm.id, s.tag),
                    size: (s.file_size_gb * 1024.0 * 1024.0 * 1024.0) as u64,
                    params: s.params.clone(),
                }).collect();

                let entry = models.entry(rm.name.clone()).or_insert_with(|| HubModel {
                    name: rm.name.clone(),
                    description: rm.description.clone(),
                    pulls: "999999".to_string(), // 给予高权重，保证前排推荐
                    updated: "N/A".to_string(),
                    variants: Vec::new(),
                    category: cat,
                    capabilities: vec!["gguf".to_string()],
                });
                
                if entry.variants.is_empty() {
                    entry.variants = variants;
                }
            }
        }

        // 2. 按类别（Chat / Embedding / Vision）分流排序，确保冷门门类不被霸榜
        let mut chat_candidates = Vec::new();
        let mut embedding_candidates = Vec::new();
        let mut vision_candidates = Vec::new();

        for m in models.values() {
            // 对于已经具有 variants 静态填充的内置兜底模型，跳过网络变体拉取
            if !m.variants.is_empty() {
                continue;
            }
            let item = (m.name.clone(), m.pulls.parse::<u64>().unwrap_or(0));
            match m.category {
                ModelCategory::Chat => chat_candidates.push(item),
                ModelCategory::Embedding => embedding_candidates.push(item),
                ModelCategory::Vision => vision_candidates.push(item),
                ModelCategory::Other => {}
            }
        }

        chat_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        embedding_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        vision_candidates.sort_by(|a, b| b.1.cmp(&a.1));

        // 3. 各自分配保底数量：20 个 Chat，10 个 Embedding，10 个 Vision
        let mut top_ids = Vec::new();
        for (id, _) in chat_candidates.into_iter().take(20) {
            top_ids.push(id);
        }
        for (id, _) in embedding_candidates.into_iter().take(10) {
            top_ids.push(id);
        }
        for (id, _) in vision_candidates.into_iter().take(10) {
            top_ids.push(id);
        }
        let fetched_count = top_ids.len();

        println!(
            "[ModelHub] Pre-fetching details for top {} candidates...",
            fetched_count
        );

        let mut fetch_tasks = Vec::new();
        for id in &top_ids {
            let reg = Arc::clone(&registry);
            let id_clone = id.clone();
            fetch_tasks.push(async move {
                let variants = reg.get_variants(&id_clone).await;
                (id_clone, variants)
            });
        }

        let results = futures_util::future::join_all(fetch_tasks).await;
        for (id, variants_res) in results {
            if let Ok(variants) = variants_res {
                if let Some(model) = models.get_mut(&id) {
                    model.variants = variants;
                }
            }
        }

        println!("[ModelHub] Sync completed from {} in {:?}. Fetched {} models with {} pre-fetched specifications.", 
            source_name, start.elapsed(), models.len(), fetched_count);

        if let Some(parent) = cache_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&models) {
            let _ = std::fs::write(&cache_path, json);
        }

        Ok(models)
    }

    pub async fn get_rankings(
        hub_cache: &HubCache,
        _is_china: bool,
    ) -> (Vec<HubModel>, Vec<HubModel>) {
        // Mock ranking from cache directly
        let mut items: Vec<HubModel> = hub_cache.values().cloned().collect();
        items.sort_by(|a, b| {
            let p_a = a.pulls.parse::<u64>().unwrap_or(0);
            let p_b = b.pulls.parse::<u64>().unwrap_or(0);
            p_b.cmp(&p_a)
        });

        (items.clone(), items)
    }
}
