# 视觉 PDF 解析与并发闪退修复实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 引入全局同步锁以保障 `llama.cpp` 底层操作的线程安全，并重构多页 PDF 的视觉解析流程以复用 Context，彻底解决后台索引时的闪退问题并提高解析速度。

**Architecture:** 
1. 在 `llama_backend.rs` 中定义一个全局静态的 `std::sync::Mutex<()>` (`ACQUIRE_LOCK`)。
2. 任何涉及加载模型、创建/销毁 Context、`decode`/`eval` 计算的同步阻塞操作，都必须在这个全局锁的保护下运行。
3. 重构 `VisionAdapter`，新增 `stream_vision_multi_page` 方法以在一次 `spawn_blocking` 中只分配一次并复用 `MtmdContext` 和 `LlamaContext`。

**Tech Stack:** Rust (Tauri v2), llama_cpp_4, tokio

---

### Task 1: 增加全局同步锁

**Files:**
- Modify: `src-tauri/src/services/llama_backend.rs`

- [ ] **Step 1: 修改 `llama_backend.rs` 增加 `ACQUIRE_LOCK` 定义**

  修改 `src-tauri/src/services/llama_backend.rs` 的内容为：
  ```rust
  use llama_cpp_4::llama_backend::LlamaBackend;
  use std::sync::{Arc, LazyLock, Mutex as StdMutex};

  pub static GLOBAL_BACKEND: LazyLock<Arc<LlamaBackend>> = LazyLock::new(|| {
      let backend = LlamaBackend::init().expect("Failed to initialize llama backend");
      Arc::new(backend)
  });

  // 全局同步互斥锁，用于同步所有 llama.cpp 底层操作
  pub static ACQUIRE_LOCK: StdMutex<()> = StdMutex::new(());
  ```

- [ ] **Step 2: 运行编译检查**
  Run: `cargo check`
  Expected: SUCCESS

- [ ] **Step 3: 提交代码**
  ```bash
  git add src-tauri/src/services/llama_backend.rs
  git commit -m "refactor: add global sync lock ACQUIRE_LOCK to llama_backend"
  ```

---

### Task 2: 保护大模型推理适配器 (`llama_adapter.rs`)

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 用全局锁保护模型加载、创建/销毁 Context 和 `decode` 阶段**

  修改 `src-tauri/src/services/inference/llama_adapter.rs`：
  
  1. 在 `load_model` (约第 159-166 行) 使用 `ACQUIRE_LOCK`：
     ```rust
     let model_result = tokio::task::spawn_blocking(move || {
         let params = llama_cpp_4::model::params::LlamaModelParams::default()
             .with_n_gpu_layers(n_gpu_layers);

         // 保护模型加载
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         LlamaModel::load_from_file(&backend, &path_str, &params)
             .map(|m| Arc::new(m))
             .map_err(|e| AppError::Internal(format!("Failed to load model: {}", e)))
     })
     ```

  2. 在 `stream_chat` (约第 242-244 行) 创建 Context 保护：
     ```rust
     let ctx = {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         model.new_context(&backend, ctx_params)
     }.map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;
     ```

  3. 在 `stream_chat` (约第 315 行) 第一阶段解码保护：
     ```rust
     let decode_res = {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         ctx.decode(&mut batch)
     };
     if let Err(e) = decode_res {
         // ... 错误处理逻辑保持不变
     }
     ```

  4. 在 `stream_chat` 循环迭代 (约第 399 行) 第二阶段解码保护：
     ```rust
     let decode_res = {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         ctx.decode(&mut batch)
     };
     if let Err(e) = decode_res {
         println!("[ERROR] Loop decode failure: {:?}", e);
         break;
     }
     ```

  5. 在 `load_model` 重置 Context (约第 173-175 行) 和 `unload` (约第 440-442 行) 时加锁保护销毁：
     ```rust
     {
         let mut active_ctx = self.active_context.lock().await;
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         *active_ctx = None;
     }
     ```

- [ ] **Step 2: 运行编译检查**
  Run: `cargo check`
  Expected: SUCCESS

- [ ] **Step 3: 提交代码**
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs
  git commit -m "feat: protect llama_adapter core operations with ACQUIRE_LOCK"
  ```

---

### Task 3: 保护 Embedding 处理器 (`embedder.rs`)

**Files:**
- Modify: `src-tauri/src/services/embedder.rs`

- [ ] **Step 1: 用全局锁保护 Embedder 模型加载、Context 创建/析构和 `decode`**

  修改 `src-tauri/src/services/embedder.rs`：

  1. 在 `get_or_load_model` (约第 29-35 行) 模型加载阶段：
     ```rust
     let model = tokio::task::spawn_blocking(move || {
         let backend = crate::services::llama_backend::GLOBAL_BACKEND.clone();
         let params = llama_cpp_4::model::params::LlamaModelParams::default();
         
         // 保护加载模型
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         LlamaModel::load_from_file(&backend, &model_path, &params)
             .map(Arc::new)
             .map_err(|e| AppError::Internal(format!("Failed to load embedding model: {}", e)))
     })
     ```

  2. 在 `embed_batch` (约第 67-69 行) 创建 Context 阶段：
     ```rust
     let mut ctx = {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         model.new_context(&backend, ctx_params)
     }.map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;
     ```

  3. 在 `embed_batch` (约第 92 行) 解码阶段：
     ```rust
     let decode_res = {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         ctx.decode(&mut batch)
     };
     decode_res.map_err(|e| AppError::Internal(format!("Decode error: {}", e)))?;
     ```

  4. 在 `embed_batch` 结束前，进行显式的加锁释放：
     在闭包的返回 `Ok(all_embeddings)` 前：
     ```rust
     // 在全局锁保护下显式 drop 掉 ctx，防止在线程析构时发生 Metal 释放冲突
     {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         drop(ctx);
     }
     Ok(all_embeddings)
     ```

- [ ] **Step 2: 运行编译和单元测试**
  Run: `cargo test --services::embedder::tests`
  Expected: PASS

- [ ] **Step 3: 提交代码**
  ```bash
  git add src-tauri/src/services/embedder.rs
  git commit -m "feat: protect embedder core operations with ACQUIRE_LOCK"
  ```

---

### Task 4: 重构多模态视觉适配器并添加全局锁保护 (`vision_adapter.rs`)

**Files:**
- Modify: `src-tauri/src/services/inference/vision_adapter.rs`

- [ ] **Step 1: 新增 `stream_vision_multi_page` 方法并在全流程加锁保护**

  修改 `src-tauri/src/services/inference/vision_adapter.rs`：

  1. 在 `load_model` (约第 70-75 行) 使用全局锁保护模型加载：
     ```rust
     let model = tokio::task::spawn_blocking(move || {
         let params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
         
         // 保护模型加载
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         LlamaModel::load_from_file(&backend, &path_buf, &params)
             .map(Arc::new)
             .map_err(|e| AppError::Internal(format!("Failed to load vision model: {}", e)))
     })
     ```

  2. 在 `stream_vision_chat` (约第 130-131 行) 保护 `MtmdContext` 和 `LlamaContext` 的创建：
     ```rust
     let mtmd_ctx = {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         MtmdContext::init_from_file(&mmproj_path, &model, mtmd_params)
     }.map_err(|e| AppError::Internal(format!("Failed to load mmproj: {}", e)))?;
     ```
     并且在第 144-146 行：
     ```rust
     let mut lctx = {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         model.new_context(&backend, ctx_params)
     }.map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;
     ```

  3. 在 `stream_vision_chat` 解码循环 (约第 231-232 行) 保护：
     ```rust
     let decode_res = {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         lctx.decode(&mut batch)
     };
     decode_res.map_err(|e| AppError::Internal(format!("Decode: {}", e)))?;
     ```

  4. 在 `stream_vision_chat` 结尾返回前：
     ```rust
     {
         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
         drop(lctx);
         drop(mtmd_ctx);
     }
     Ok(())
     ```

  5. **在 `VisionAdapter` 结构中新增** `stream_vision_multi_page` 方法：
     ```rust
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
                 let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
                 MtmdContext::init_from_file(&mmproj_path, &model, mtmd_params)
             }.map_err(|e| AppError::Internal(format!("Failed to load mmproj: {}", e)))?;

             let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                 .with_n_ctx(NonZeroU32::new(ctx_size))
                 .with_n_batch(512);

             let mut lctx = {
                 let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
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
                     "<|im_start|>system\nYou are a helpful AI assistant.<|im_end|>\n<|im_start|>user\n{} 这是文档的第 {} 页图片。请完整提取此页中的文字（如果是表格，请保留 Markdown 表格格式；如果含有图表，请进行语义描述）。请直接输出提取出的内容，不要有任何前导客套话或多余的解释。<|im_end|>\n<|im_start|>assistant\n",
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
                         let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
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
                 let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
                 drop(lctx);
                 drop(mtmd_ctx);
             }

             Ok(())
         })
         .await
         .map_err(|e| AppError::Internal(format!("Multi-page vision task panicked: {}", e)))?
     }
     ```

- [ ] **Step 2: 运行编译检查**
  Run: `cargo check`
  Expected: SUCCESS

- [ ] **Step 3: 提交代码**
  ```bash
  git add src-tauri/src/services/inference/vision_adapter.rs
  git commit -m "feat: add stream_vision_multi_page and global lock to vision_adapter"
  ```

---

### Task 5: 接入视觉解析并完成多页复用重构 (`vision.rs`)

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`

- [ ] **Step 1: 改写 `parse_pdf_with_vision` 使用多页复用方法**

  修改 `src-tauri/src/services/parser/vision.rs` 中的 `parse_pdf_with_vision` 函数（约第 148-306 行）：
  
  ```rust
  /// 视觉解析 PDF（借用 Sidecar 渲染 PDF 页面，再调用批量 Vision OCR）
  pub async fn parse_pdf_with_vision(path: &Path, app_handle: &AppHandle) -> Result<String, AppError> {
      let temp_id = uuid::Uuid::new_v4().to_string();
      let app_data_dir = app_handle.path().app_data_dir().unwrap();
      let temp_dir = app_data_dir.join("temp").join(format!("pdf_render_{}", temp_id));
      
      std::fs::create_dir_all(&temp_dir)
          .map_err(|e| AppError::Internal(format!("Failed to create temp PDF render directory: {}", e)))?;

      // 1. 调用 Sidecar 渲染 PDF 页面图片
      let render_result = crate::services::parser::sidecar::render_pdf_pages_via_sidecar(
          app_handle,
          path,
          &temp_dir
      ).await;

      if let Err(e) = render_result {
          eprintln!("[Vision Parser] Warning: Sidecar PDF rendering failed: {}. Falling back to plain text extraction.", e);
          let _ = std::fs::remove_dir_all(&temp_dir);
          
          let path_clone = path.to_path_buf();
          return tokio::task::spawn_blocking(move || crate::services::parser::core::parse_pdf(&path_clone))
              .await
              .map_err(|join_err| AppError::Internal(format!("Fallback task execution failed: {}", join_err)))?;
      }

      // 2. 读取并排序所有导出的页面图片
      let mut page_images = Vec::new();
      if let Ok(entries) = std::fs::read_dir(&temp_dir) {
          for entry in entries.flatten() {
              let p = entry.path();
              if p.is_file() {
                  let is_img = p.extension()
                      .map(|ext| ext.to_string_lossy().to_lowercase() == "jpg" || ext.to_string_lossy().to_lowercase() == "png")
                      .unwrap_or(false);
                  if is_img {
                      page_images.push(p);
                  }
              }
          }
      }

      page_images.sort_by(|a, b| {
          let a_num = a.file_stem()
              .and_then(|s| s.to_string_lossy().replace("page_", "").parse::<i32>().ok())
              .unwrap_or(0);
          let b_num = b.file_stem()
              .and_then(|s| s.to_string_lossy().replace("page_", "").parse::<i32>().ok())
              .unwrap_or(0);
          a_num.cmp(&b_num)
      });

      // 3. 获取/缓存加载参数
      let db_path = app_data_dir.join("telepathy.db");
      let conn = rusqlite::Connection::open(&db_path)
          .map_err(|e| AppError::Internal(format!("Failed to open DB: {}", e)))?;
      let settings_cache = app_handle.state::<std::sync::Arc<SettingsCache>>();
      
      let vision_model = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_model")?
          .ok_or_else(|| AppError::Internal("未配置视觉模型".to_string()))?;
      let vision_mmproj = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_mmproj")?
          .ok_or_else(|| AppError::Internal("未配置 mmproj 视觉投影器".to_string()))?;

      let num_thread = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_thread")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(6);
      let num_ctx = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_ctx")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(4096);
      let num_gpu = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_gpu")?
          .and_then(|v| v.parse::<i32>().ok())
          .unwrap_or(0);

      let model_path = app_data_dir.join("downloads").join(&vision_model);
      let mmproj_path = app_data_dir.join("downloads").join(&vision_mmproj);

      if !model_path.exists() || !mmproj_path.exists() {
          let _ = std::fs::remove_dir_all(&temp_dir);
          return Err(AppError::Internal("本地视觉模型文件或投影器文件不存在".to_string()));
      }

      // 4. 压缩读入的所有页面图片
      let mut pages_bytes = Vec::new();
      for img_path in &page_images {
          let image_data = match std::fs::read(img_path) {
              Ok(d) => d,
              Err(e) => {
                  println!("[Vision Parser] Warning: failed to read page: {}", e);
                  continue;
              }
          };
          if let Ok(compressed) = compress_image(&image_data, 448) {
              pages_bytes.push(compressed);
          }
      }

      // 5. 获取 VisionEngine 状态并批量加载
      let vision_state = app_handle.state::<VisionEngineManagerState>();
      let vision_arc = vision_state.0.clone();
      let mut vision_guard = vision_arc.lock().await;
      vision_guard.load_model(
          &model_path,
          &mmproj_path,
          num_ctx as u32,
          num_gpu,
          num_thread
      ).await?;

      let (tx, mut rx) = tokio::sync::mpsc::channel::<(usize, String)>(100);
      let cancel_token = std::sync::Arc::new(CancellationToken::new());

      // 6. 开启并发收集任务
      let receiver_task = tokio::spawn(async move {
          let mut pages_map = std::collections::BTreeMap::new();
          while let Some((page_num, page_text)) = rx.recv().await {
              pages_map.insert(page_num, page_text);
          }
          pages_map
      });

      // 7. 开启批量多页视觉推理
      let stream_res = vision_guard.stream_vision_multi_page(
          pages_bytes,
          tx,
          cancel_token,
      ).await;

      stream_res?;

      let pages_map = receiver_task.await
          .map_err(|e| AppError::Internal(format!("Receiver task failed: {}", e)))?;

      // 8. 排序拼接 Markdown
      let mut full_markdown = String::new();
      for (page_num, page_text) in pages_map {
          if !full_markdown.is_empty() {
              full_markdown.push_str("\n\n");
          }
          full_markdown.push_str(&format!("--- [第 {} 页] ---\n\n", page_num));
          full_markdown.push_str(&page_text);
      }

      // 9. 清理临时文件夹
      let _ = std::fs::remove_dir_all(&temp_dir);

      Ok(full_markdown)
  }
  ```

- [ ] **Step 2: 运行编译和现有的 parser 单元测试**
  Run: `cargo test --services::parser::vision::tests`
  Expected: PASS

- [ ] **Step 3: 提交代码**
  ```bash
  git add src-tauri/src/services/parser/vision.rs
  git commit -m "refactor: optimize pdf vision parser to reuse context and stream batch"
  ```

---

### Task 6: 终期验证

**Files:**
- Test: 全库编译与基础测试

- [ ] **Step 1: 执行所有后端测试**
  Run: `cargo test`
  Expected: PASS (检查是否生成了任何编译错误或测试失败)

- [ ] **Step 2: 提交最终成果分支**
  ```bash
  git status
  ```
  Expected: Clean working tree
