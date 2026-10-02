use crate::errors::AppError;
use llama_cpp_4::context::LlamaContext;
use llama_cpp_4::model::LlamaModel;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

pub struct CachedEmbedContext {
    pub context: LlamaContext<'static>,
    pub model: Arc<LlamaModel>,
    pub backend: Arc<llama_cpp_4::llama_backend::LlamaBackend>,
    pub model_path: PathBuf,
}

unsafe impl Send for CachedEmbedContext {}
unsafe impl Sync for CachedEmbedContext {}

static GLOBAL_CONTEXT: OnceLock<Arc<Mutex<Option<CachedEmbedContext>>>> = OnceLock::new();

fn get_global_context() -> Arc<Mutex<Option<CachedEmbedContext>>> {
    GLOBAL_CONTEXT.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

pub struct Embedder {
    model_path: PathBuf,
}

impl Embedder {
    pub fn new(model_path: &str) -> Self {
        Self {
            model_path: PathBuf::from(model_path),
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let results = self.embed_batch(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("No embedding returned".into()))
    }

    pub async fn get_dimension(&self) -> Result<usize, AppError> {
        let cached_context = get_global_context();
        let model_path = self.model_path.clone();

        // 1. Try to read from cache first (takes <1 microsecond if already loaded)
        {
            let guard = cached_context.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(cached) = &*guard {
                if cached.model_path == model_path {
                    return Ok(cached.model.n_embd() as usize);
                }
            }
        }

        // 2. If not cached, trigger load by embedding a dummy input once
        let _ = self.embed("a").await?;

        // 3. Now read the cached value
        let guard = cached_context.lock().unwrap_or_else(|e| e.into_inner());
        let cached = guard.as_ref().ok_or_else(|| AppError::Internal("Model not loaded".into()))?;
        Ok(cached.model.n_embd() as usize)
    }

    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let model_path = self.model_path.clone();
        let cached_context = get_global_context();
        let texts = texts.to_vec();

        tokio::task::spawn_blocking::<_, Result<Vec<Vec<f32>>, AppError>>(move || {
            let mut guard = cached_context.lock().unwrap_or_else(|e| e.into_inner());

            let need_load = match &*guard {
                Some(cached) => cached.model_path != model_path,
                None => true,
            };

            if need_load {
                // Drop the old context before loading the new one to save memory
                if guard.is_some() {
                    let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    *guard = None;
                }

                // 1. 获取全局共享 of Llama backend
                let embed_backend = crate::services::llama_backend::GLOBAL_BACKEND.clone();

                let params = llama_cpp_4::model::params::LlamaModelParams::default()
                    .with_n_gpu_layers(0)
                    .with_main_gpu(-1);
                
                let model = {
                    let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    LlamaModel::load_from_file(&embed_backend, &model_path, &params)
                }.map_err(|e| AppError::Internal(format!("Failed to load embedding model: {}", e)))?;
                
                let model = Arc::new(model);

                let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                    .with_n_ctx(NonZeroU32::new(2048))
                    .with_embeddings(true)
                    .with_flash_attention(false);

                let ctx = {
                    let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    model.new_context(&embed_backend, ctx_params)
                }.map_err(|e| AppError::Internal(format!("Failed to create embedding context: {}", e)))?;

                let static_ctx: LlamaContext<'static> = unsafe { std::mem::transmute(ctx) };
                
                *guard = Some(CachedEmbedContext {
                    context: static_ctx,
                    model,
                    backend: embed_backend,
                    model_path: model_path.clone(),
                });
            }

            let cached = guard.as_mut().unwrap();
            let ctx = &mut cached.context;
            let model = &cached.model;

            let mut all_embeddings = Vec::new();

            for text in texts {
                // 每次向量化前，清空 KV Cache，保证上下文干净
                ctx.clear_kv_cache();

                let mut tokens = model
                    .str_to_token(&text, llama_cpp_4::model::AddBos::Always)
                    .map_err(|e| AppError::Internal(format!("Tokenize error: {}", e)))?;

                if tokens.len() > 2048 {
                    tokens.truncate(2048);
                }

                let n_batch = 512;
                let mut global_pos = 0i32;
                for chunk in tokens.chunks(n_batch) {
                    let mut batch = llama_cpp_4::llama_batch::LlamaBatch::new(chunk.len(), 1);
                    for token in chunk.iter() {
                        batch
                            .add(*token, global_pos, &[0], true)
                            .map_err(|e| AppError::Internal(format!("Batch add error: {}", e)))?;
                        global_pos += 1;
                    }

                    let decode_res = {
                        let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                        ctx.decode(&mut batch)
                    };
                    decode_res.map_err(|e| AppError::Internal(format!("Decode error: {}", e)))?;
                }

                let mut embedding_opt = None;

                // 1. Try embeddings_seq_ith(0)
                if let Ok(e) = ctx.embeddings_seq_ith(0) {
                    if !e.iter().all(|&v| v == 0.0) {
                        embedding_opt = Some(e.to_vec());
                    }
                }

                // 2. Try looping through all tokens with embeddings_ith
                if embedding_opt.is_none() {
                    for idx in 0..tokens.len() {
                        if let Ok(e) = ctx.embeddings_ith(idx as i32) {
                            if !e.iter().all(|&v| v == 0.0) {
                                embedding_opt = Some(e.to_vec());
                                break;
                            }
                        }
                    }
                }

                // 3. Fallback
                if embedding_opt.is_none() {
                    if let Ok(e) = ctx.embeddings_ith(0) {
                        embedding_opt = Some(e.to_vec());
                    }
                }

                let embedding = embedding_opt
                    .ok_or_else(|| AppError::Internal("Failed to get embeddings".into()))?;

                all_embeddings.push(embedding);
            }

            Ok(all_embeddings)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Blocking task panicked: {}", e)))?
    }

    pub async fn check_model_available(&self) -> Result<bool, AppError> {
        Ok(self.model_path.exists())
    }

    pub fn model_name(&self) -> String {
        self.model_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

// Global context is static, so we don't clear it when local Embedder is dropped.
// This allows sharing cache between separate Tauri commands.
impl Drop for Embedder {
    fn drop(&mut self) {
        // No-op to preserve cache across Embedder instantiations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_model_embedding() {
        let path =
            "/Users/mac/Library/Application Support/com.telepathy.app/downloads/bge-m3-q4_k_m.gguf";
        if std::path::Path::new(path).exists() {
            let emb = Embedder::new(path);
            let embedding = emb.embed("hello").await.unwrap();
            println!("[TEST EMBEDDING] Embedding len = {}", embedding.len());
            let all_zeros = embedding.iter().all(|&v| v == 0.0);
            println!("[TEST EMBEDDING] Is all zeros = {}", all_zeros);
            assert!(!all_zeros, "Embedding is all zeros!");
        }
    }

}

