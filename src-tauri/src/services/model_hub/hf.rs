use crate::errors::AppError;
use crate::services::model_hub::registry::ModelRegistryClient;
use crate::services::model_hub::{HubModel, ModelCategory, ModelVariant};
use async_trait::async_trait;
use serde_json::Value;

/// Extract a human-readable parameter size label from a GGUF filename.
/// Examples:
///   "Meta-Llama-3-8B-Q4_K_M.gguf" -> "8B Q4_K_M"
///   "gemma-2-2b-it-IQ2_M.gguf" -> "2B IQ2_M"
fn extract_params_from_filename(filename: &str) -> String {
    // Strip path prefix if present
    let base = filename.rsplit('/').next().unwrap_or(filename);
    // Remove .gguf extension
    let stem = base.trim_end_matches(".gguf");

    // Try to find quantization label (e.g. Q4_K_M, IQ2_M, F16, Q8_0)
    let quant_re =
        regex::Regex::new(r"(?i)\b(IQ[0-9][_A-Z]*|Q[0-9][0-9]?[_A-Z]*|F16|F32|BF16)\b").ok();
    let quant = quant_re.and_then(|re| re.find(stem).map(|m| m.as_str().to_uppercase()));

    // Try to find parameter size (e.g. 7B, 13B, 70B, 0.5B)
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

fn infer_category(model_id: &str, description: &str) -> ModelCategory {
    let text = format!("{} {}", model_id, description).to_lowercase();
    if text.contains("embedding")
        || text.contains("embed")
        || text.contains("bge")
        || text.contains("gte")
        || text.contains("e5-")
        || text.contains("jina-embeddings")
    {
        ModelCategory::Embedding
    } else if text.contains("vision")
        || text.contains("vl")
        || text.contains("llava")
        || text.contains("qwen2-vl")
        || text.contains("qwen2.5-vl")
    {
        ModelCategory::Vision
    } else {
        ModelCategory::Chat
    }
}

pub struct HuggingFaceRegistry {
    client: reqwest::Client,
}

impl HuggingFaceRegistry {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Telepathy/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}

#[async_trait]
impl ModelRegistryClient for HuggingFaceRegistry {
    async fn search(&self, query: &str) -> Result<Vec<HubModel>, AppError> {
        let url = format!(
            "https://huggingface.co/api/models?search={}&filter=gguf&limit=100",
            query
        );
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let json: Vec<Value> = res
            .json()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut models = Vec::new();
        for item in json {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                let pulls = item.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0);
                let description = item
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Hugging Face GGUF Model");
                models.push(HubModel {
                    name: id.to_string(),
                    description: description.to_string(),
                    pulls: pulls.to_string(),
                    updated: "N/A".to_string(),
                    variants: Vec::new(),
                    category: infer_category(id, description),
                    capabilities: vec!["gguf".to_string()],
                });
            }
        }

        // Variants are fetched lazily on-demand when user expands a model
        Ok(models)
    }

    async fn get_trending(&self) -> Result<Vec<HubModel>, AppError> {
        // Similar to search but sorted by downloads
        let url = "https://huggingface.co/api/models?filter=gguf&sort=downloads&limit=100";
        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let json: Vec<Value> = res
            .json()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut models = Vec::new();
        for item in json {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                let description = item
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Hugging Face Top GGUF Model");
                models.push(HubModel {
                    name: id.to_string(),
                    description: description.to_string(),
                    pulls: item
                        .get("downloads")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        .to_string(),
                    updated: "N/A".to_string(),
                    variants: Vec::new(),
                    category: infer_category(id, description),
                    capabilities: vec!["gguf".to_string()],
                });
            }
        }

        // Variants are fetched lazily on-demand when user expands a model
        Ok(models)
    }

    async fn get_variants(&self, model_id: &str) -> Result<Vec<ModelVariant>, AppError> {
        // Find .gguf files in the repo tree
        let url = format!("https://huggingface.co/api/models/{}/tree/main", model_id);
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let json: Vec<Value> = res
            .json()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let split_re = regex::Regex::new(r"(?i)(-\d{5}-of-\d{5}|-split-|part\d+|shard\d+)")
            .map_err(|e| AppError::Internal(format!("Failed to compile split regex: {}", e)))?;

        let mut variants = Vec::new();
        for item in json {
            if let Some(path) = item.get("path").and_then(|v| v.as_str()) {
                if path.ends_with(".gguf") {
                    let lower_path = path.to_lowercase();
                    // We removed the mmproj exclusion so users can download projector files
                    // if lower_path.contains("mmproj") || lower_path.contains("projector") {
                    //     continue;
                    // }

                    // Skip sharded files (e.g. -00001-of-00004.gguf) as we don't support multi-file merge yet
                    if split_re.is_match(&lower_path) {
                        continue;
                    }

                    let size = item.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                    // Extract a human-readable params label from the filename
                    let params = extract_params_from_filename(path);
                    variants.push(ModelVariant {
                        tag: path.to_string(),
                        size,
                        params,
                    });
                }
            }
        }
        Ok(variants)
    }

    fn get_download_url(&self, model_id: &str, variant: &str) -> String {
        format!(
            "https://huggingface.co/{}/resolve/main/{}",
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

