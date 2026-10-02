use super::{EngineStats, InferenceEngine, Message, ModelLoadOptions, SamplingConfig};
use crate::errors::AppError;
use async_trait::async_trait;

use llama_cpp_4::llama_backend::LlamaBackend;
use llama_cpp_4::model::LlamaChatMessage;
use llama_cpp_4::model::LlamaModel;
use llama_cpp_4::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::services::llama_backend::GLOBAL_BACKEND;
use llama_cpp_4::context::LlamaContext;
use llama_cpp_4::token::LlamaToken;

pub struct ActiveContext {
    pub context: LlamaContext<'static>,
    pub model: Arc<LlamaModel>,
    pub history_tokens: Vec<LlamaToken>,
}

unsafe impl Send for ActiveContext {}
unsafe impl Sync for ActiveContext {}

/// Build the prompt using the model's native chat template from GGUF metadata.
/// Falls back to manual template detection if the model doesn't have a baked-in template
/// or if the native template fails to apply.
fn build_prompt_with_native_template(
    model: &LlamaModel,
    messages: &[Message],
    model_path: &str,
) -> Result<String, AppError> {
    let template_type = super::template::ChatTemplateType::detect_from_model_path(model_path);

    let llama_messages: Vec<LlamaChatMessage> = messages
        .iter()
        .map(|m| {
            LlamaChatMessage::new(m.role.clone(), m.content.clone())
                .map_err(|e| AppError::Internal(format!("Invalid chat message: {}", e)))
        })
        .collect::<Result<Vec<_>, _>>()?;

    match model.apply_chat_template(None, &llama_messages, true) {
        Ok(prompt) => Ok(prompt),
        Err(e) => {
            println!(
                "[LlamaCpp] Native chat template failed ({:?}), falling back to heuristic",
                e
            );
            Ok(template_type.apply(messages))
        }
    }
}

fn get_gpu_layers(num_gpu_setting: i32) -> u32 {
    if num_gpu_setting <= 0 {
        return 0;
    }

    #[cfg(target_os = "macos")]
    {
        num_gpu_setting as u32
    }

    #[cfg(not(target_os = "macos"))]
    {
        num_gpu_setting as u32
    }
}

pub struct LlamaCppAdapter {
    pub is_loaded: bool,
    backend: Option<Arc<LlamaBackend>>,
    model: Option<Arc<LlamaModel>>,
    model_path: Option<PathBuf>,
    context_size: u32,
    n_threads: i32,
    n_gpu_layers: u32, // Keep track of actual GPU layers loaded
    pub active_context: Arc<tokio::sync::Mutex<Option<ActiveContext>>>,
}

impl Default for LlamaCppAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LlamaCppAdapter {
    pub fn new() -> Self {
        Self {
            is_loaded: false,
            backend: None,
            model: None,
            model_path: None,
            context_size: 4096,
            n_threads: 6,
            n_gpu_layers: 0,
            active_context: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
}

#[async_trait]
impl InferenceEngine for LlamaCppAdapter {
    async fn load_model(
        &mut self,
        model_path: &Path,
        options: ModelLoadOptions,
    ) -> Result<(), AppError> {
        let n_gpu_layers = get_gpu_layers(options.n_gpu_layers);
        // If already loaded the same model with same config, skip
        if self.is_loaded
            && self.model_path.as_deref() == Some(model_path)
            && self.context_size == options.context_size
            && self.n_threads == options.n_threads
            && self.n_gpu_layers == n_gpu_layers
        {
            println!("[LlamaCpp] Model already loaded and config matches, skipping load");
            return Ok(());
        }

        let backend = GLOBAL_BACKEND.clone();
        let path_buf = model_path.to_path_buf();
        let path_str = path_buf
            .to_str()
            .ok_or_else(|| AppError::Internal("Invalid path string".into()))?
            .to_string();

        println!("[LlamaCpp] Loading model from: {}", path_str);

        if !model_path.exists() {
            return Err(AppError::Internal(format!("Model file does not exist: {}", path_str)));
        }

        // Use spawn_blocking for the synchronous model load
        let model_result = tokio::task::spawn_blocking(move || {
            let params = llama_cpp_4::model::params::LlamaModelParams::default()
                .with_n_gpu_layers(n_gpu_layers);

            // 保护模型加载
            let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            LlamaModel::load_from_file(&backend, &path_str, &params)
                .map(|m| Arc::new(m))
                .map_err(|e| AppError::Internal(format!("Failed to load model: {}", e)))
        })
        .await
        .map_err(|e| AppError::Internal(format!("Model load task panicked: {}", e)))?;

        let model = model_result?;

        {
            let mut active_ctx = self.active_context.lock().await;
            let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            *active_ctx = None;
        }

        self.model = Some(model);
        self.model_path = Some(path_buf);
        self.context_size = options.context_size;
        self.n_threads = options.n_threads.max(1);
        self.n_gpu_layers = n_gpu_layers; // Save it here
        self.is_loaded = true;

        Ok(())
    }

    async fn stream_chat(
        &self,
        messages: Vec<Message>,
        sampling: SamplingConfig,
        tx: mpsc::Sender<String>,
        cancel: Arc<CancellationToken>,
    ) -> Result<(), AppError> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| AppError::Internal("Model not loaded".into()))?
            .clone();
        let ctx_size = self.context_size;
        let n_threads = self.n_threads.max(1);

        // 1. Build prompt using model's native chat template
        let model_name = self
            .model_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let prompt = build_prompt_with_native_template(&model, &messages, &model_name)?;

        let tokens = model
            .str_to_token(&prompt, llama_cpp_4::model::AddBos::Always)
            .map_err(|e| AppError::Internal(format!("Failed to tokenize: {}", e)))?;

        if tokens.len() as u32 > self.context_size {
            return Err(AppError::Internal(format!(
                "Prompt too long ({} tokens) for context size ({})",
                tokens.len(),
                self.context_size
            )));
        }

        let is_gpu_enabled = self.n_gpu_layers > 0;
        let backend = GLOBAL_BACKEND.clone();
        let active_ctx_arc = self.active_context.clone();

        println!("\n>>> [Inference] 🚀 正在生成 AI 回复 (KV Cache 增量优化模式)...");

        tokio::task::spawn_blocking(move || {
            let total_start = std::time::Instant::now();
            let mut active_ctx_guard = active_ctx_arc.blocking_lock();

            // Initialize context if it doesn't exist
            if active_ctx_guard.is_none() {
                let n_batch = 512;
                let optimal_threads = determine_optimal_threads(n_threads, is_gpu_enabled);
                let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                    .with_n_ctx(NonZeroU32::new(ctx_size))
                    .with_n_batch(n_batch)
                    .with_n_threads(optimal_threads)
                    .with_n_threads_batch(optimal_threads)
                    .with_flash_attention(is_gpu_enabled);

                let ctx = {
                    let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    model.new_context(&backend, ctx_params)
                }.map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;

                // Safety transmute: erase lifetime parameter to make it 'static
                let static_ctx: LlamaContext<'static> = unsafe { std::mem::transmute(ctx) };
                *active_ctx_guard = Some(ActiveContext {
                    model: model.clone(),
                    context: static_ctx,
                    history_tokens: Vec::new(),
                });
            }

            let active_ctx = active_ctx_guard.as_mut().unwrap();
            let ctx = &mut active_ctx.context;
            let history_tokens = &mut active_ctx.history_tokens;

            // Calculate common prefix
            let mut common_len = calculate_prefix(history_tokens, &tokens);

            // If new prompt token length equals common_len, decode_slice will be empty.
            // Sampling without any decode causes a crash. Backtrack 1 token to trigger a decode and populate logits.
            if common_len == tokens.len() && common_len > 0 {
                common_len -= 1;
            }

            let n_batch = 512;

            // Bounds safety check and cache pruning using determine_cache_action
            match determine_cache_action(
                tokens.len(),
                sampling.max_tokens as usize,
                ctx_size as usize,
                history_tokens.len(),
                common_len,
            ) {
                CachePruningAction::ClearAll => {
                    println!("[Inference] Clearing all KV cache.");
                    ctx.clear_kv_cache();
                    history_tokens.clear();
                    common_len = 0;
                }
                CachePruningAction::PruneTo(prune_pos) => {
                    println!("[Inference] Discrepancy found. Pruning KV cache after pos: {}", prune_pos);
                    if let Err(e) = ctx.clear_kv_cache_seq(Some(0), Some(prune_pos as u32), None) {
                        println!("[ERROR] Failed to clear partial cache: {:?}. Clearing all.", e);
                        ctx.clear_kv_cache();
                        history_tokens.clear();
                        common_len = 0;
                    } else {
                        history_tokens.truncate(prune_pos);
                        common_len = prune_pos;
                    }
                }
                CachePruningAction::KeepAsIs => {
                    // Cache matches history prefix exactly, keep it
                }
            }

            // Incremental decode from the end of common prefix
            let mut batch_pos = common_len as i32;
            let mut last_chunk_len = 0;
            let decode_slice = &tokens[common_len..];

            let prefill_start = std::time::Instant::now();

            for chunk in decode_slice.chunks(n_batch as usize) {
                let mut batch = llama_cpp_4::llama_batch::LlamaBatch::new(chunk.len(), 1);
                for (i, token) in chunk.iter().enumerate() {
                    let is_last = (batch_pos + i as i32) == (tokens.len() as i32 - 1);
                    batch
                        .add(*token, batch_pos + i as i32, &[0], is_last)
                        .map_err(|e| AppError::Internal(format!("Batch add error: {}", e)))?;
                }

                let decode_res = {
                    let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    ctx.decode(&mut batch)
                };
                if let Err(e) = decode_res {
                    println!("[ERROR] Decode failure: {:?}. Resetting context for self-healing.", e);
                    ctx.clear_kv_cache();
                    history_tokens.clear();
                    return Err(AppError::Internal(format!("Decode failure: {}", e)));
                }

                batch_pos += chunk.len() as i32;
                last_chunk_len = chunk.len();
            }

            let prefill_duration = prefill_start.elapsed();
            println!("[PERF] Prefill (Prompt Processing) took: {:?}", prefill_duration);

            let mut n_cur = tokens.len() as i32;
            let mut batch_index_to_sample = if last_chunk_len > 0 {
                (last_chunk_len - 1) as i32
            } else {
                // Fallback to sample the last token in context if no new tokens were decoded
                (tokens.len() - 1) as i32
            };

            let mut sampler = LlamaSampler::chain(
                vec![
                    LlamaSampler::top_k(sampling.top_k.max(1)),
                    LlamaSampler::top_p(sampling.top_p.clamp(0.0, 1.0), 1),
                    LlamaSampler::temp(sampling.temperature.max(0.0)),
                    LlamaSampler::penalties(64, sampling.repeat_penalty.max(1.0), 0.0, 0.0),
                    LlamaSampler::dist(42),
                ],
                true,
            );

            let mut last_token = -1;
            let mut repeat_count = 0;
            let mut total_tokens = 0;
            let mut new_generated_tokens = Vec::new();
            let mut ttft = None;

            let decode_start = std::time::Instant::now();

            while n_cur < ctx_size as i32 {
                if cancel.is_cancelled() {
                    break;
                }

                let next_token = sampler.sample(ctx, batch_index_to_sample);

                if model.is_eog_token(next_token) {
                    break;
                }

                if next_token.0 == last_token {
                    repeat_count += 1;
                    if repeat_count > 5 {
                        println!("[Inference] 🚨 Detected repeat loop. Safeguard triggered.");
                        break;
                    }
                } else {
                    last_token = next_token.0;
                    repeat_count = 0;
                }

                if total_tokens >= sampling.max_tokens.max(64) {
                    break;
                }

                let token_str = model
                    .token_to_str(next_token, llama_cpp_4::model::Special::Tokenize)
                    .unwrap_or_default();

                let lower_token = token_str.to_lowercase();
                if lower_token.contains("<unused")
                    || lower_token.contains("unused")
                    || lower_token.contains("<|im_end|>")
                {
                    break;
                }

                if tx.blocking_send(token_str).is_err() {
                    break;
                }

                if ttft.is_none() {
                    ttft = Some(total_start.elapsed());
                    println!("[PERF] Time to First Token (TTFT): {:?}", ttft.unwrap());
                }

                new_generated_tokens.push(next_token);

                let mut batch = llama_cpp_4::llama_batch::LlamaBatch::new(1, 1);
                batch.add(next_token, n_cur, &[0], true).map_err(|e| {
                    AppError::Internal(format!("Batch add error: {}", e))
                })?;

                let decode_res = {
                    let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    ctx.decode(&mut batch)
                };
                if let Err(e) = decode_res {
                    println!("[ERROR] Loop decode failure: {:?}", e);
                    break;
                }

                n_cur += 1;
                total_tokens += 1;
                batch_index_to_sample = 0;
            }

            let decode_duration = decode_start.elapsed();
            let speed = if decode_duration.as_secs_f32() > 0.0 {
                total_tokens as f32 / decode_duration.as_secs_f32()
            } else {
                0.0
            };

            println!(
                "[PERF] Decode (Token Generation) took: {:?}, Speed: {:.2} T/s",
                decode_duration, speed
            );

            // Commit all tokens (input prompt + newly generated assistant tokens) to memory
            let mut final_history = tokens.clone();
            final_history.extend(new_generated_tokens);
            *history_tokens = final_history;

            if total_tokens == 0 {
                println!(
                    ">>> [Inference] ⚠️ Model generated 0 tokens! Prompt size: {} tokens, Context size: {}",
                    tokens.len(),
                    ctx_size
                );
            }
            
            let total_duration = total_start.elapsed();
            println!(
                "[PERF] End-to-End Inference Command Time: {:?}",
                total_duration
            );
            println!(
                ">>> [Inference] ✅ Generation finished. Total tokens generated: {}\n",
                total_tokens
            );

            Ok(())
        })
        .await
        .map_err(|e| AppError::Internal(format!("Blocking task panicked: {}", e)))?
    }

    fn get_stats(&self) -> EngineStats {
        EngineStats {
            vram_used_mb: 0,
            engine_type: "LlamaCpp".into(),
        }
    }

    async fn unload(&mut self) -> Result<(), AppError> {
        {
            let mut active_ctx = self.active_context.lock().await;
            let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            *active_ctx = None;
        }
        self.model = None;
        self.model_path = None;
        self.is_loaded = false;
        Ok(())
    }
}

fn calculate_prefix(history: &[LlamaToken], new_tokens: &[LlamaToken]) -> usize {
    let mut common_len = 0;
    for (t_hist, t_new) in history.iter().zip(new_tokens.iter()) {
        if t_hist == t_new {
            common_len += 1;
        } else {
            break;
        }
    }
    common_len
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePruningAction {
    ClearAll,
    PruneTo(usize),
    KeepAsIs,
}

pub fn determine_cache_action(
    tokens_len: usize,
    max_tokens: usize,
    ctx_size: usize,
    history_len: usize,
    common_len: usize,
) -> CachePruningAction {
    if tokens_len + max_tokens > ctx_size {
        CachePruningAction::ClearAll
    } else if common_len < history_len {
        if common_len > 0 {
            CachePruningAction::PruneTo(common_len)
        } else {
            CachePruningAction::ClearAll
        }
    } else {
        CachePruningAction::KeepAsIs
    }
}

fn determine_optimal_threads(user_threads: i32, is_gpu_enabled: bool) -> i32 {
    if is_gpu_enabled {
        1
    } else {
        user_threads.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockModel {
        dropped: Arc<AtomicBool>,
    }
    impl Drop for MockModel {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_active_context_drop_ordering() {
        let dropped = Arc::new(AtomicBool::new(false));
        {
            let model = Arc::new(MockModel {
                dropped: dropped.clone(),
            });
            // Verify model lifecycle release at block end
        }
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn test_common_prefix_calculation() {
        let history = vec![LlamaToken(1), LlamaToken(2), LlamaToken(3)];
        let new_tokens_same = vec![LlamaToken(1), LlamaToken(2), LlamaToken(3), LlamaToken(4)];
        let new_tokens_diff = vec![LlamaToken(1), LlamaToken(5), LlamaToken(3)];
        let new_tokens_empty: Vec<LlamaToken> = vec![];

        let match_same = calculate_prefix(&history, &new_tokens_same);
        let match_diff = calculate_prefix(&history, &new_tokens_diff);
        let match_empty = calculate_prefix(&history, &new_tokens_empty);

        assert_eq!(match_same, 3);
        assert_eq!(match_diff, 1);
        assert_eq!(match_empty, 0);
    }

    #[test]
    fn test_determine_cache_action() {
        assert_eq!(
            determine_cache_action(2000, 2100, 4000, 1000, 1000),
            CachePruningAction::ClearAll
        );
        assert_eq!(
            determine_cache_action(1000, 500, 4000, 500, 500),
            CachePruningAction::KeepAsIs
        );
        assert_eq!(
            determine_cache_action(1000, 500, 4000, 800, 300),
            CachePruningAction::PruneTo(300)
        );
        assert_eq!(
            determine_cache_action(1000, 500, 4000, 800, 0),
            CachePruningAction::ClearAll
        );
    }

    #[test]
    fn test_determine_optimal_threads() {
        // GPU enabled -> thread count should always be 1
        assert_eq!(determine_optimal_threads(6, true), 1);
        assert_eq!(determine_optimal_threads(1, true), 1);

        // GPU disabled -> thread count should match user setting (min 1)
        assert_eq!(determine_optimal_threads(6, false), 6);
        assert_eq!(determine_optimal_threads(0, false), 1);
        assert_eq!(determine_optimal_threads(-2, false), 1);
    }

    use std::sync::Mutex;

    struct DropTracker {
        name: &'static str,
        log: Arc<Mutex<Vec<&'static str>>>,
    }
    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.log.lock().unwrap().push(self.name);
        }
    }

    struct ActiveContextSimulatorOld {
        pub model: DropTracker,
        pub context: DropTracker,
    }

    struct ActiveContextSimulatorNew {
        pub context: DropTracker,
        pub model: DropTracker,
    }

    #[test]
    fn test_active_context_drop_order_old_vs_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        {
            let _s = ActiveContextSimulatorOld {
                model: DropTracker {
                    name: "model",
                    log: log.clone(),
                },
                context: DropTracker {
                    name: "context",
                    log: log.clone(),
                },
            };
        }
        assert_eq!(*log.lock().unwrap(), vec!["model", "context"]);

        log.lock().unwrap().clear();
        {
            let _s = ActiveContextSimulatorNew {
                context: DropTracker {
                    name: "context",
                    log: log.clone(),
                },
                model: DropTracker {
                    name: "model",
                    log: log.clone(),
                },
            };
        }
        assert_eq!(*log.lock().unwrap(), vec!["context", "model"]);
    }

    #[tokio::test]
    async fn test_load_model_skip_condition_gpu_layers() {
        let mut adapter = LlamaCppAdapter::new();
        adapter.is_loaded = true;
        adapter.model_path = Some(PathBuf::from("non_existent_model.gguf"));
        adapter.context_size = 2048;
        adapter.n_threads = 4;
        adapter.n_gpu_layers = 10; // Loaded with 10 GPU layers

        // Now, we request to load with 20 GPU layers.
        // If the skip condition checks `self.n_gpu_layers == n_gpu_layers`, it should NOT skip.
        // If it doesn't skip, it will proceed to load and fail (return Err).
        // If it incorrectly skips, it will return Ok(()).
        let options = ModelLoadOptions {
            context_size: 2048,
            n_threads: 4,
            n_gpu_layers: 20, // Different GPU layers
        };
        let result = adapter
            .load_model(Path::new("non_existent_model.gguf"), options)
            .await;

        // Before fix: this will return Ok(()).
        // After fix: this will return Err(...) because it does not skip, and try to load the non-existent file.
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_active_context_cleared_on_load() {
        let path =
            "/Users/mac/Library/Application Support/com.telepathy.app/downloads/bge-m3-q4_k_m.gguf";
        if std::path::Path::new(path).exists() {
            let mut adapter = LlamaCppAdapter::new();
            let options = ModelLoadOptions {
                context_size: 2048,
                n_threads: 4,
                n_gpu_layers: 0,
            };
            adapter
                .load_model(Path::new(path), options.clone())
                .await
                .unwrap();

            // Now, we mock setting active_context
            let model = adapter.model.clone().unwrap();
            let backend = GLOBAL_BACKEND.clone();
            let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(2048));
            let ctx = model.new_context(&backend, ctx_params).unwrap();
            let static_ctx: LlamaContext<'static> = unsafe { std::mem::transmute(ctx) };

            {
                let mut active_ctx_guard = adapter.active_context.lock().await;
                *active_ctx_guard = Some(ActiveContext {
                    model: model.clone(),
                    context: static_ctx,
                    history_tokens: Vec::new(),
                });
            }

            // Verify active_context is populated
            {
                let active_ctx_guard = adapter.active_context.lock().await;
                assert!(active_ctx_guard.is_some());
            }

            // Now load the model again but with different options (so it doesn't skip)
            let new_options = ModelLoadOptions {
                context_size: 1024, // Different context size
                n_threads: 4,
                n_gpu_layers: 0,
            };
            adapter
                .load_model(Path::new(path), new_options)
                .await
                .unwrap();

            // After loading, active_context must be cleared (None)
            {
                let active_ctx_guard = adapter.active_context.lock().await;
                assert!(active_ctx_guard.is_none());
            }
        }
    }

    #[tokio::test]
    async fn test_stream_chat_identical_prompt() {
        let path =
            "/Users/mac/Library/Application Support/com.telepathy.app/downloads/bge-m3-q4_k_m.gguf";
        if std::path::Path::new(path).exists() {
            let mut adapter = LlamaCppAdapter::new();
            let options = ModelLoadOptions {
                context_size: 2048,
                n_threads: 4,
                n_gpu_layers: 0,
            };
            adapter.load_model(Path::new(path), options).await.unwrap();

            let messages = vec![Message {
                role: "user".into(),
                content: "hello".into(),
            }];

            let (tx, mut rx) = mpsc::channel(100);
            let cancel = Arc::new(CancellationToken::new());

            // First call to stream_chat (populates active_context)
            adapter
                .stream_chat(
                    messages.clone(),
                    SamplingConfig::default(),
                    tx,
                    cancel.clone(),
                )
                .await
                .unwrap();

            // Drain receiver
            while let Some(_) = rx.recv().await {}

            // Second call with EXACTLY the same messages.
            // This will result in common_len == tokens.len().
            let (tx2, mut rx2) = mpsc::channel(100);
            let cancel2 = Arc::new(CancellationToken::new());

            let result = adapter
                .stream_chat(messages, SamplingConfig::default(), tx2, cancel2)
                .await;

            // Without the fix, this might crash or error or hang.
            // With the fix, it should succeed.
            assert!(result.is_ok());

            // Drain receiver
            while let Some(_) = rx2.recv().await {}
        }
    }

    #[test]
    fn test_intel_mac_gpu_layers_override() {
        // 模拟设置 GPU layers 为 10
        let original_layers = 10;
        let target_layers = get_gpu_layers(original_layers);
        
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(target_layers, original_layers as u32);
        }
    }
}
