use crate::errors::AppError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

pub struct EngineManagerState(pub Arc<Mutex<Option<Box<dyn InferenceEngine>>>>);
pub struct VisionEngineManagerState(pub Arc<Mutex<vision_adapter::VisionAdapter>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ModelLoadOptions {
    pub n_gpu_layers: i32,
    pub context_size: u32,
    pub n_threads: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineStats {
    pub vram_used_mb: u64,
    pub engine_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub temperature: f32,
    pub top_k: i32,
    pub top_p: f32,
    pub repeat_penalty: f32,
    pub max_tokens: i32,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_k: 40,
            top_p: 0.95,
            repeat_penalty: 1.1,
            max_tokens: 1024,
        }
    }
}

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn load_model(
        &mut self,
        model_path: &Path,
        options: ModelLoadOptions,
    ) -> Result<(), AppError>;
    async fn stream_chat(
        &self,
        messages: Vec<Message>,
        sampling: SamplingConfig,
        tx: mpsc::Sender<String>,
        cancel: Arc<tokio_util::sync::CancellationToken>,
    ) -> Result<(), AppError>;
    fn get_stats(&self) -> EngineStats;
    async fn unload(&mut self) -> Result<(), AppError>;
}

pub mod llama_adapter;
pub mod template;
pub mod vision_adapter;
