use crate::errors::AppError;
use crate::services::model_hub::{HubModel, ModelVariant};
use async_trait::async_trait;

/// The uniform interface for querying models from external hubs (HuggingFace, ModelScope, etc.)
#[async_trait]
pub trait ModelRegistryClient: Send + Sync {
    /// Search for models on the registry
    async fn search(&self, query: &str) -> Result<Vec<HubModel>, AppError>;

    /// Get the trending or popular models
    async fn get_trending(&self) -> Result<Vec<HubModel>, AppError>;

    /// Fetch GGUF variants for a specific model ID
    async fn get_variants(&self, model_id: &str) -> Result<Vec<ModelVariant>, AppError>;

    /// Obtain the direct download URL for a specific GGUF variant
    fn get_download_url(&self, model_id: &str, variant: &str) -> String;
}
