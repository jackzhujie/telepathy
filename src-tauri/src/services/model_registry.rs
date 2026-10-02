#![allow(dead_code)]
use crate::errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Chat,
    Embedding,
    Vision,
    Unknown,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Chat => write!(f, "chat"),
            ModelType::Embedding => write!(f, "embedding"),
            ModelType::Vision => write!(f, "vision"),
            ModelType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSize {
    pub tag: String,
    pub params: String,
    pub min_memory_gb: u32,
    pub file_size_gb: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub description: String,
    pub sizes: Vec<ModelSize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub version: String,
    pub updated_at: String,
    pub models: Vec<RegistryModel>,
}

#[derive(Debug, Serialize, Clone)]
pub struct InstalledModel {
    pub full_name: String,
    pub base_name: String,
    pub engine_type: String,
    pub format: String,
    pub file_path: String,
    pub model_type: ModelType,
    pub size_bytes: u64,
    pub display_name: String,
    pub provider: String,
    pub description: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Recommendation {
    Recommended,
    Marginal,
    #[serde(rename = "notrecommended")]
    NotRecommended,
}

#[derive(Debug, Serialize, Clone)]
pub struct ModelRecommendation {
    pub model: RegistryModel,
    pub size: ModelSize,
    pub recommendation: Recommendation,
}

/// Load the built-in model registry from the embedded JSON
pub fn load_registry() -> Result<ModelRegistry, AppError> {
    let json_str = include_str!("../../resources/model_registry.json");
    serde_json::from_str(json_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse model registry: {}", e)))
}

/// Filter registry models by type
pub fn filter_by_type(registry: &ModelRegistry, model_type: &ModelType) -> Vec<RegistryModel> {
    registry
        .models
        .iter()
        .filter(|m| &m.model_type == model_type)
        .cloned()
        .collect()
}

/// Search registry models by keyword (searches id, name, provider, description)
pub fn search_models(registry: &ModelRegistry, keyword: &str) -> Vec<RegistryModel> {
    let kw = keyword.to_lowercase();
    registry
        .models
        .iter()
        .filter(|m| {
            m.id.to_lowercase().contains(&kw)
                || m.name.to_lowercase().contains(&kw)
                || m.provider.to_lowercase().contains(&kw)
                || m.description.to_lowercase().contains(&kw)
        })
        .cloned()
        .collect()
}

/// Parse a model name like "qwen2.5:7b" into (base_name, tag)
pub fn parse_model_name(full_name: &str) -> (String, String) {
    if let Some(pos) = full_name.rfind(':') {
        let base = full_name[..pos].to_string();
        let tag = full_name[pos + 1..].to_string();
        (base, tag)
    } else {
        (full_name.to_string(), "latest".to_string())
    }
}

/// Match an installed model name against the registry to determine its type
pub fn match_model_type(
    registry: &ModelRegistry,
    base_name: &str,
) -> (ModelType, String, String, String) {
    // 1. Precise match from registry
    let lower_base = base_name.to_lowercase();
    for model in &registry.models {
        let lower_id = model.id.to_lowercase();
        let matches = if lower_base == lower_id {
            true
        } else if lower_base.starts_with(&lower_id) {
            let next_char = lower_base.chars().nth(lower_id.chars().count());
            match next_char {
                Some('_') | Some('-') | Some('.') | Some(':') => true,
                _ => false,
            }
        } else {
            false
        };

        if matches {
            return (
                model.model_type.clone(),
                model.name.clone(),
                model.provider.clone(),
                model.description.clone(),
            );
        }
    }

    // 2. Heuristic matching for community/custom models
    let lower_name = base_name.to_lowercase();

    // Check for Embedding keywords
    if lower_name.contains("embed")
        || lower_name.contains("bge")
        || lower_name.contains("m3")
        || lower_name.contains("nomic")
    {
        return (
            ModelType::Embedding,
            base_name.to_string(),
            "Community".to_string(),
            "Auto-detected as Embedding model".to_string(),
        );
    }

    // Check for Vision keywords
    if lower_name.contains("vision")
        || lower_name.contains("llava")
        || lower_name.contains("moondream")
        || lower_name.contains("-vl")
        || lower_name.contains("qwen2-vl")
        || lower_name.contains("qwen2.5-vl")
        || lower_name.contains("mmproj")
        || lower_name.contains("projector")
    {
        return (
            ModelType::Vision,
            base_name.to_string(),
            "Community".to_string(),
            "Auto-detected as Vision model".to_string(),
        );
    }

    // Default to Chat for most models
    (
        ModelType::Chat,
        base_name.to_string(),
        "Community".to_string(),
        "Auto-detected as Chat model".to_string(),
    )
}

/// Get recommendations based on available system memory
pub fn get_recommendations(
    registry: &ModelRegistry,
    available_memory_gb: f64,
) -> Vec<ModelRecommendation> {
    let mut recommendations = Vec::new();
    for model in &registry.models {
        for size in &model.sizes {
            let recommendation = if available_memory_gb >= size.min_memory_gb as f64 * 1.5 {
                Recommendation::Recommended
            } else if available_memory_gb >= size.min_memory_gb as f64 {
                Recommendation::Marginal
            } else {
                Recommendation::NotRecommended
            };
            recommendations.push(ModelRecommendation {
                model: model.clone(),
                size: size.clone(),
                recommendation,
            });
        }
    }
    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_registry() {
        let registry = load_registry().unwrap();
        assert!(!registry.models.is_empty());
        assert!(registry.models.iter().any(|m| m.id == "qwen2.5"));
    }

    #[test]
    fn test_parse_model_name() {
        let (base, tag) = parse_model_name("qwen2.5:7b");
        assert_eq!(base, "qwen2.5");
        assert_eq!(tag, "7b");

        let (base2, tag2) = parse_model_name("bge-m3");
        assert_eq!(base2, "bge-m3");
        assert_eq!(tag2, "latest");
    }

    #[test]
    fn test_filter_by_type() {
        let registry = load_registry().unwrap();
        let chat_models = filter_by_type(&registry, &ModelType::Chat);
        assert!(chat_models.iter().all(|m| m.model_type == ModelType::Chat));
        let embedding_models = filter_by_type(&registry, &ModelType::Embedding);
        assert!(embedding_models
            .iter()
            .all(|m| m.model_type == ModelType::Embedding));
    }

    #[test]
    fn test_search_models() {
        let registry = load_registry().unwrap();
        let results = search_models(&registry, "qwen");
        assert!(results.iter().any(|m| m.id == "qwen2.5"));
    }

    #[test]
    fn test_match_model_type() {
        let registry = load_registry().unwrap();

        // 1. Precise match from registry
        let (model_type, _, _, _) = match_model_type(&registry, "qwen2.5");
        assert_eq!(model_type, ModelType::Chat);

        let (model_type_gguf, _, _, _) = match_model_type(&registry, "qwen2.5_1.5b.gguf");
        assert_eq!(model_type_gguf, ModelType::Chat);

        let (model_type2, _, _, _) = match_model_type(&registry, "bge-m3");
        assert_eq!(model_type2, ModelType::Embedding);

        // 2. Heuristic: Embedding
        let (mt_emb, _, _, _) = match_model_type(&registry, "my-custom-embed-model");
        assert_eq!(mt_emb, ModelType::Embedding);

        // 3. Heuristic: Vision
        let (mt_vis, _, _, _) = match_model_type(&registry, "llava-custom-v1");
        assert_eq!(mt_vis, ModelType::Vision);

        // 4. Heuristic: Default to Chat
        let (mt_chat, _, _, _) = match_model_type(&registry, "any-unknown-model");
        assert_eq!(mt_chat, ModelType::Chat);

        // 5. User's specific case
        let (mt_user, _, _, _) =
            match_model_type(&registry, "gemma_4_e4b_uncensored_hauhaucs_aggressive");
        assert_eq!(mt_user, ModelType::Chat);
    }

    #[test]
    fn test_get_recommendations() {
        let registry = load_registry().unwrap();
        let recs = get_recommendations(&registry, 16.0);
        assert!(!recs.is_empty());
    }
}
