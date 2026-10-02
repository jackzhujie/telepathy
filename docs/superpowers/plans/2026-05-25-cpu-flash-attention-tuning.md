# CPU 推理性能调优与回复时间日志监测实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 动态按 GPU 启用状态决策 Flash Attention，消除纯 CPU 模式推理下严重的计算卡顿；并嵌入高精度的端到端推理性能日志 Telemetry（耗时监控）。

**Architecture:** 
1. 在 `llama_adapter.rs` 中，将 `with_flash_attention` 的开启状态从原本的硬编码 `true` 修改为根据 `is_gpu_enabled` 动态决策，纯 CPU 模式下强制为 `false`，彻底打通 CPU 高效运算通道。
2. 在 `llama_adapter.rs` 的 `stream_chat` 生成循环中嵌入 `std::time::Instant`。依次输出：Prefill 耗时、TTFT 首字延迟、Decode 端打字总耗时与每秒 Token 生成速度（T/s）、以及总端到端执行时间。

**Tech Stack:** Rust (Tauri Backend), llama-cpp-4

---

### Task 1: 动态 CPU 性能参数调优与高精度推理日志嵌入

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 在 `stream_chat` 初始化 Context 参数时将 Flash Attention 设为动态决定**
  修改 `src-tauri/src/services/inference/llama_adapter.rs` 的 `stream_chat` 方法，将 `.with_flash_attention(true)` 改为 `.with_flash_attention(is_gpu_enabled)`，使 CPU 推理自动规避有缺陷的 Flash Attention：
  ```rust
                  let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                      .with_n_ctx(NonZeroU32::new(ctx_size))
                      .with_n_batch(n_batch)
                      .with_n_threads(optimal_threads)
                      .with_n_threads_batch(optimal_threads)
                      .with_flash_attention(is_gpu_enabled);
  ```

- [ ] **Step 2: 在 `stream_chat` 内部的各推理阶段嵌入 Instant 计时并输出 Telemetry 日志**
  修改 `llama_adapter.rs` 的 `stream_chat` 闭包中的推理与生成循环代码：
  - 记录 Prefill 的耗时；
  - 记录 TTFT (Time to First Token) 耗时；
  - 记录 Token Decode 速度与打字机输出耗时。
  
  修改后的 `stream_chat` 相关部分应精密包含以下性能计算与打印代码：
  ```rust
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
                  CachePruningAction::KeepAsIs => {}
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

              // Commit all tokens to memory
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
  ```

- [ ] **Step 3: 运行后台 Cargo 编译和测试检验**
  运行：`cargo check --manifest-path src-tauri/Cargo.toml`
  验证编译完全通过 (SUCCESS)。

- [ ] **Step 4: 提交调优和性能监控变动**
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs
  git commit -m "perf: dynamically disable flash attention on CPU and embed detailed inference latency logs"
  ```
