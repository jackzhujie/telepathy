use crate::errors::AppError;
use crate::services::llama_backend::GLOBAL_BACKEND;
use llama_cpp_4::llama_batch::LlamaBatch;
use llama_cpp_4::model::params::LlamaModelParams;
use llama_cpp_4::model::LlamaModel;
use llama_cpp_4::mtmd::{
    MtmdBitmap, MtmdContext, MtmdContextParams, MtmdInputChunks, MtmdInputText,
};
use llama_cpp_4::token::LlamaToken;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct VisionAdapter {
    model: Option<Arc<LlamaModel>>,
    model_path: Option<PathBuf>,
    mmproj_path: Option<PathBuf>,
    context_size: u32,
    n_threads: i32,
    is_loaded: bool,
}

impl Default for VisionAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl VisionAdapter {
    pub fn new() -> Self {
        Self {
            model: None,
            model_path: None,
            mmproj_path: None,
            context_size: 4096,
            n_threads: 4,
            is_loaded: false,
        }
    }

    pub async fn load_model(
        &mut self,
        model_path: &Path,
        mmproj_path: &Path,
        context_size: u32,
        n_gpu_layers: i32,
        n_threads: i32,
    ) -> Result<(), AppError> {
        // Skip if same model already loaded
        if self.is_loaded
            && self.model_path.as_deref() == Some(model_path)
            && self.mmproj_path.as_deref() == Some(mmproj_path)
        {
            println!("[Vision] Model already loaded, skipping");
            return Ok(());
        }

        let path_buf = model_path.to_path_buf();
        let backend = GLOBAL_BACKEND.clone();
        let mut gpu_layers = if n_gpu_layers > 0 {
            n_gpu_layers as u32
        } else {
            0
        };

        println!("[Vision] Loading model from: {:?}", path_buf);

        let model = tokio::task::spawn_blocking(move || {
            let params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
            
            // 保护模型加载
            let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            LlamaModel::load_from_file(&backend, &path_buf, &params)
                .map(|m| Arc::new(m))
                .map_err(|e| AppError::Internal(format!("Failed to load vision model: {}", e)))
        })
        .await
        .map_err(|e| AppError::Internal(format!("Vision model load panicked: {}", e)))??;

        self.model = Some(model);
        self.model_path = Some(model_path.to_path_buf());
        self.mmproj_path = Some(mmproj_path.to_path_buf());
        self.context_size = context_size;
        self.n_threads = n_threads;
        self.is_loaded = true;
        println!("[Vision] Model loaded successfully");
        Ok(())
    }

    /// Stream vision inference: process images + text query
    pub async fn stream_vision_chat(
        &self,
        query: &str,
        image_bytes_list: Vec<Vec<u8>>,
        max_tokens: i32,
        tx: mpsc::Sender<String>,
        cancel: Arc<CancellationToken>,
    ) -> Result<(), AppError> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| AppError::Internal("Vision model not loaded".into()))?
            .clone();
        let mmproj_path = self
            .mmproj_path
            .as_ref()
            .ok_or_else(|| AppError::Internal("mmproj path not set".into()))?
            .clone();
        let ctx_size = self.context_size;
        let n_threads = self.n_threads;
        let query = query.to_string();
        let backend = GLOBAL_BACKEND.clone();

        println!(
            "[Vision] Starting inference with {} image(s), query: {}",
            image_bytes_list.len(),
            if query.len() > 50 {
                format!("{}...", &query[..50])
            } else {
                query.clone()
            }
        );

        tokio::task::spawn_blocking(move || {
            // 1. Create MtmdContext from mmproj file
            let mtmd_params = MtmdContextParams::default()
                .use_gpu(false) // Disabled GPU due to llama.cpp ggml-metal bugs with certain mmproj files
                .n_threads(n_threads)
                .print_timings(false);

            let mtmd_ctx = {
                let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                MtmdContext::init_from_file(&mmproj_path, &model, mtmd_params)
            }.map_err(|e| AppError::Internal(format!("Failed to load mmproj: {}", e)))?;

            println!(
                "[Vision] MtmdContext: vision={}, audio={}",
                mtmd_ctx.supports_vision(),
                mtmd_ctx.supports_audio()
            );

            // 2. Create LLM context
            let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(ctx_size))
                .with_n_batch(512);

            let mut lctx = {
                let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                model.new_context(&backend, ctx_params)
            }.map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;

            // 3. Load image bitmaps from raw bytes
            let marker = MtmdContext::default_marker();
            let mut bitmaps: Vec<MtmdBitmap> = Vec::new();
            for (i, bytes) in image_bytes_list.iter().enumerate() {
                println!("[Vision] Loading image {} ({} bytes)", i + 1, bytes.len());
                let bm = MtmdBitmap::from_buf(&mtmd_ctx, bytes)
                    .map_err(|e| AppError::Internal(format!("Failed to load image {}: {}", i + 1, e)))?;
                bitmaps.push(bm);
            }

            // 4. Build prompt with media markers
            // We use ChatML format to ensure Qwen-VL and similar instruction models don't loop endlessly
            let prompt = if bitmaps.is_empty() {
                format!("<|im_start|>system\nYou are a helpful AI assistant.<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n", query)
            } else {
                let markers: String = (0..bitmaps.len())
                    .map(|_| marker.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("<|im_start|>system\nYou are a helpful AI assistant.<|im_end|>\n<|im_start|>user\n{} {}<|im_end|>\n<|im_start|>assistant\n", markers, query)
            };

            // 5. Tokenize
            let input_text = MtmdInputText::new(&prompt, true, true);
            let bitmap_refs: Vec<&MtmdBitmap> = bitmaps.iter().collect();
            let mut chunks = MtmdInputChunks::new();
            mtmd_ctx
                .tokenize(&input_text, &bitmap_refs, &mut chunks)
                .map_err(|e| AppError::Internal(format!("Tokenize failed: {}", e)))?;

            println!(
                "[Vision] Tokenized into {} chunk(s), {} total tokens",
                chunks.len(),
                chunks.n_tokens()
            );

            // 6. Eval all chunks
            let mut n_past: i32 = 0;
            mtmd_ctx
                .eval_chunks(lctx.as_ptr(), &chunks, 0, 0, 512, true, &mut n_past)
                .map_err(|e| AppError::Internal(format!("Eval failed: {}", e)))?;

            // 7. Sample loop (greedy)
            let eos = model.token_eos();
            let mut generated = 0i32;

            println!("\n>>> [Vision] 🚀 正在生成 AI 回复...");

            loop {
                if cancel.is_cancelled() {
                    println!("[Vision] Cancelled by user");
                    break;
                }
                if generated >= max_tokens {
                    println!("[Vision] Reached max tokens ({})", max_tokens);
                    break;
                }

                let logits = lctx.get_logits();
                let next_token = logits
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(i, _)| LlamaToken::new(i as i32))
                    .unwrap();

                if model.is_eog_token(next_token) || next_token == eos {
                    break;
                }

                let piece = model
                    .token_to_str(next_token, llama_cpp_4::model::Special::Tokenize)
                    .unwrap_or_default();

                if tx.blocking_send(piece).is_err() {
                    break;
                }

                // Feed the token back
                let mut batch = LlamaBatch::new(1, 0);
                batch
                    .add(next_token, n_past, &[0], true)
                    .map_err(|e| AppError::Internal(format!("Batch add: {}", e)))?;
                
                let decode_res = {
                    let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    lctx.decode(&mut batch)
                };
                decode_res.map_err(|e| AppError::Internal(format!("Decode: {}", e)))?;

                n_past += 1;
                generated += 1;
            }

            println!(
                "\n>>> [Vision] ✅ 生成完毕！本次共计 {} 个 Token\n",
                generated
            );
            {
                let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                drop(lctx);
                drop(mtmd_ctx);
            }
            Ok(())
        })
        .await
        .map_err(|e| AppError::Internal(format!("Vision task panicked: {}", e)))?
    }

    /// 一次性加载一次 Context 批量解析多页图片以复用资源
    pub async fn stream_vision_multi_page(
        &self,
        pages_images_bytes: Vec<Vec<u8>>,
        tx: mpsc::Sender<(usize, String)>, // 返回 (页码, 解析所得文本)
        cancel: Arc<CancellationToken>,
    ) -> Result<(), AppError> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| AppError::Internal("Vision model not loaded".into()))?
            .clone();
        let mmproj_path = self
            .mmproj_path
            .as_ref()
            .ok_or_else(|| AppError::Internal("mmproj path not set".into()))?
            .clone();
        let ctx_size = self.context_size;
        let n_threads = self.n_threads;
        let backend = GLOBAL_BACKEND.clone();

        println!("[Vision] Starting multi-page inference for {} page(s)", pages_images_bytes.len());

        tokio::task::spawn_blocking(move || {
            let mtmd_params = MtmdContextParams::default()
                .use_gpu(false)
                .n_threads(n_threads)
                .print_timings(false);

            // 1. 初始化 MtmdContext 和 LLM context（加锁）
            let mtmd_ctx = {
                let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                MtmdContext::init_from_file(&mmproj_path, &model, mtmd_params)
            }.map_err(|e| AppError::Internal(format!("Failed to load mmproj: {}", e)))?;

            let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(ctx_size))
                .with_n_batch(512);

            let mut lctx = {
                let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                model.new_context(&backend, ctx_params)
            }.map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;

            // 2. 依次循环处理每页
            for (idx, compressed_bytes) in pages_images_bytes.into_iter().enumerate() {
                let page_num = idx + 1;
                if cancel.is_cancelled() {
                    break;
                }

                // 每页开始前，清理上一次的 KV Cache 缓存
                lctx.clear_kv_cache();

                let marker = MtmdContext::default_marker();
                let bm = MtmdBitmap::from_buf(&mtmd_ctx, &compressed_bytes)
                    .map_err(|e| AppError::Internal(format!("Failed to load page {} image: {}", page_num, e)))?;

                let prompt = format!(
                    "<|im_start|>system\nYou are a helpful AI assistant.<|im_end|>\n<|im_start|>user\n{} 这是文档的第 {} 页图片。请完整提取此页中的文字（如果是表格，请保留 Markdown 表格格式；如果含有图表，请进行语义描述）。请直接输出提取出的内容，不要有任何前导客套话或多余 of 解释。<|im_end|>\n<|im_start|>assistant\n",
                    marker, page_num
                );

                let input_text = MtmdInputText::new(&prompt, true, true);
                let mut chunks = MtmdInputChunks::new();
                mtmd_ctx
                    .tokenize(&input_text, &[&bm], &mut chunks)
                    .map_err(|e| AppError::Internal(format!("Tokenize failed on page {}: {}", page_num, e)))?;

                let mut n_past: i32 = 0;
                mtmd_ctx
                    .eval_chunks(lctx.as_ptr(), &chunks, 0, 0, 512, true, &mut n_past)
                    .map_err(|e| AppError::Internal(format!("Eval failed on page {}: {}", page_num, e)))?;

                let eos = model.token_eos();
                let mut generated = 0i32;
                let mut page_text = String::new();

                // 生成循环
                loop {
                    if cancel.is_cancelled() || generated >= 1024 {
                        break;
                    }

                    let logits = lctx.get_logits();
                    let next_token = logits
                        .iter()
                        .enumerate()
                        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                        .map(|(i, _)| LlamaToken::new(i as i32))
                        .unwrap();

                    if model.is_eog_token(next_token) || next_token == eos {
                        break;
                    }

                    let piece = model
                        .token_to_str(next_token, llama_cpp_4::model::Special::Tokenize)
                        .unwrap_or_default();
                    page_text.push_str(&piece);

                    let mut batch = LlamaBatch::new(1, 0);
                    batch.add(next_token, n_past, &[0], true)
                        .map_err(|e| AppError::Internal(format!("Batch add page {}: {}", page_num, e)))?;

                    let decode_res = {
                        let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                        lctx.decode(&mut batch)
                    };
                    decode_res.map_err(|e| AppError::Internal(format!("Decode page {}: {}", page_num, e)))?;

                    n_past += 1;
                    generated += 1;
                }

                if tx.blocking_send((page_num, page_text)).is_err() {
                    break;
                }
            }

            // 3. 全局锁保护销毁
            {
                let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                drop(lctx);
                drop(mtmd_ctx);
            }

            Ok(())
        })
        .await
        .map_err(|e| AppError::Internal(format!("Multi-page vision task panicked: {}", e)))?
    }

    pub fn unload(&mut self) {
        self.model = None;
        self.model_path = None;
        self.mmproj_path = None;
        self.is_loaded = false;
    }
}
