use crate::errors::AppError;
use crate::services::model_hub::registry::ModelRegistryClient;
use crate::services::model_hub::{HubModel, ModelCategory, ModelVariant};
use async_trait::async_trait;
use serde_json::Value;

/// Extract a human-readable parameter size label from a GGUF filename.
fn extract_params_from_filename(filename: &str) -> String {
    let base = filename.rsplit('/').next().unwrap_or(filename);
    let stem = base.trim_end_matches(".gguf");
    let quant_re =
        regex::Regex::new(r"(?i)\b(IQ[0-9][_A-Z]*|Q[0-9][0-9]?[_A-Z]*|F16|F32|BF16)\b").ok();
    let quant = quant_re.and_then(|re| re.find(stem).map(|m| m.as_str().to_uppercase()));
    let size_re = regex::Regex::new(r"(?i)\b(\d+\.?\d*[BbMm])\b").ok();
    let size = size_re.and_then(|re| re.find(stem).map(|m| m.as_str().to_uppercase()));
    let is_mmproj = filename.to_lowercase().contains("mmproj");
    let mut label = match (size, quant) {
        (Some(s), Some(q)) => format!("{} {}", s, q),
        (Some(s), None) => s.to_string(),
        (None, Some(q)) => q.to_string(),
        (None, None) => "GGUF".to_string(),
    };
    
    if is_mmproj {
        label = format!("{} (MMPROJ)", label);
    }
    
    label
}

pub struct ModelScopeRegistry {
    client: reqwest::Client,
}

impl ModelScopeRegistry {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Telepathy/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// Internal helper to parse ModelScope model JSON with robust case-insensitivity
    fn parse_models_json(&self, json: &Value) -> Vec<HubModel> {
        let mut models = Vec::new();
        let data = json.get("data").or_else(|| json.get("Data"));
        let models_node = data.and_then(|d| d.get("models").or_else(|| d.get("Models")));

        if let Some(items) = models_node.and_then(|m| m.as_array()) {
            for item in items {
                let id = item
                    .get("id")
                    .or_else(|| item.get("ID"))
                    .or_else(|| item.get("Id"))
                    .and_then(|v| v.as_str());

                if let Some(name) = id {
                    let pulls = item
                        .get("downloads")
                        .or_else(|| item.get("Downloads"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    let desc = item
                        .get("description")
                        .or_else(|| item.get("Description"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("ModelScope Model")
                        .to_string();

                    // Heuristic to detect embedding models
                    let lower_name = name.to_lowercase();
                    let lower_desc = desc.to_lowercase();
                    let category = if lower_name.contains("embedding")
                        || lower_name.contains("embed")
                        || lower_name.contains("bge")
                        || lower_name.contains("gte")
                        || lower_name.contains("e5-")
                        || lower_name.contains("jina-embeddings")
                        || lower_name.contains("bert")
                        || lower_desc.contains("embedding")
                        || lower_desc.contains("向量")
                    {
                        ModelCategory::Embedding
                    } else if lower_name.contains("vision")
                        || lower_name.contains("llava")
                        || lower_name.contains("qwen2-vl")
                        || lower_name.contains("qwen2.5-vl")
                        || lower_desc.contains("vision")
                    {
                        ModelCategory::Vision
                    } else {
                        ModelCategory::Chat
                    };

                    models.push(HubModel {
                        name: name.to_string(),
                        description: if desc.is_empty() {
                            "ModelScope Model".to_string()
                        } else {
                            desc
                        },
                        pulls: pulls.to_string(),
                        updated: "N/A".to_string(),
                        variants: Vec::new(),
                        category,
                        capabilities: vec!["gguf".to_string()],
                    });
                }
            }
        }
        models
    }
}

#[async_trait]
impl ModelRegistryClient for ModelScopeRegistry {
    async fn search(&self, query: &str) -> Result<Vec<HubModel>, AppError> {
        // ModelScope search endpoint. Use correct page_size=50 (max allowed)
        let q = if query.trim().is_empty() {
            "gguf".to_string()
        } else {
            format!("{} gguf", query.trim())
        };

        let url = format!(
            "https://modelscope.cn/openapi/v1/models?search={}&page_size=50&page_number=1",
            urlencoding::encode(&q)
        );

        let res = self.client.get(&url).send().await;
        if let Ok(res) = res {
            if let Ok(json) = res.json::<Value>().await {
                return Ok(self.parse_models_json(&json));
            }
        }

        Ok(Vec::new())
    }

    async fn get_trending(&self) -> Result<Vec<HubModel>, AppError> {
        // Fetch 50 trending/top GGUF models (single page)
        let url = "https://modelscope.cn/openapi/v1/models?search=gguf&page_size=50&page_number=1";
        let res = self.client.get(url).send().await;
        if let Ok(res) = res {
            if let Ok(json) = res.json::<Value>().await {
                return Ok(self.parse_models_json(&json));
            }
        }

        Ok(Vec::new())
    }

    async fn get_variants(&self, model_id: &str) -> Result<Vec<ModelVariant>, AppError> {
        // Find .gguf files in the repo files list for MS. Use Recursive=true to find files in subdirectories.
        let url = format!(
            "https://modelscope.cn/api/v1/models/{}/repo/files?Recursive=true",
            model_id
        );
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to MS API: {}", e)))?;

        let json: Value = res
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse MS API response: {}", e)))?;

        let split_re = regex::Regex::new(r"(?i)(-\d{5}-of-\d{5}|-split-|part\d+|shard\d+)")
            .map_err(|e| AppError::Internal(format!("Failed to compile split regex: {}", e)))?;

        let mut variants = Vec::new();

        // Robust extraction: Handle both "Data" (legacy) and "data" (OpenAPI) styles
        let data = json.get("Data").or_else(|| json.get("data"));

        if let Some(files_val) = data.and_then(|d| d.get("Files").or_else(|| d.get("files"))) {
            if let Some(files) = files_val.as_array() {
                for item in files {
                    // Check both "Path" and "Name" as some APIs might return one or the other
                    let path = item
                        .get("Path")
                        .or_else(|| item.get("path"))
                        .or_else(|| item.get("Name"))
                        .or_else(|| item.get("name"))
                        .and_then(|v| v.as_str());

                    if let Some(p) = path {
                        let lower_p = p.to_lowercase();
                        if lower_p.ends_with(".gguf") {
                            // We removed the mmproj exclusion so users can download projector files
                            // if lower_p.contains("mmproj") || lower_p.contains("projector") {
                            //     continue;
                            // }

                            // Skip sharded files (e.g. -00001-of-00004.gguf) as we don't support multi-file merge yet
                            if split_re.is_match(&lower_p) {
                                continue;
                            }

                            let size = item
                                .get("Size")
                                .or_else(|| item.get("size"))
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);

                            // Extract human-readable params from filename
                            let params = extract_params_from_filename(p);
                            variants.push(ModelVariant {
                                tag: p.to_string(),
                                size,
                                params,
                            });
                        }
                    }
                }
            }
        }

        // If still empty, it might be an error or a different structure
        if variants.is_empty() {
            if let Some(msg) = json
                .get("Message")
                .or_else(|| json.get("message"))
                .and_then(|m| m.as_str())
            {
                return Err(AppError::Internal(format!("ModelScope API error: {}", msg)));
            }
        }

        Ok(variants)
    }

    fn get_download_url(&self, model_id: &str, variant: &str) -> String {
        // MS direct download URL format
        format!(
            "https://modelscope.cn/models/{}/resolve/master/{}",
            model_id, variant
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_regex_matching() {
        let re = regex::Regex::new(r"(?i)(-\d{5}-of-\d{5}|-split-|part\d+|shard\d+)").unwrap();
        assert!(re.is_match("model-00001-of-00005.gguf"));
        assert!(!re.is_match("model-q4_k_m.gguf"));
    }
}

