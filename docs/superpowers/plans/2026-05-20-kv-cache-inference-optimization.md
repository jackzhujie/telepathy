# KV Cache 推理复用与智能参数调度实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现单会话内 KV Cache 的智能复用（前缀裁剪），动态计算最大公共前缀并裁剪过期缓存，优化多线程和 Flash Attention 默认值以缩短首字延迟。

**Architecture:** 设计包含 Arc<LlamaModel> 并使用 transmute<'static> 擦除生命周期的 `ActiveContext` 并发容器，避免 Rust 自引用检查错误；改写 `stream_chat` 解码路径，根据前缀做增量解码；并动态根据 GPU 是否启用调整 `n_threads` 为 1 释放总线。

**Tech Stack:** Rust (Tauri Backend), llama-cpp-4 (v0.2.43), tokio (mutex & spawn_blocking).

---

### Task 1: 声明 `ActiveContext` 容器并集成到 `LlamaCppAdapter`

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 编写测试用例验证 struct 可以正常 Drop**
  在 `src-tauri/src/services/inference/llama_adapter.rs` 底部添加测试模块以验证内存所有权：
  ```rust
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
              let model = Arc::new(MockModel { dropped: dropped.clone() });
              // 验证 model 生命周期在 block 结束时释放
          }
          assert!(dropped.load(Ordering::SeqCst));
      }
  }
  ```

- [ ] **Step 2: 运行测试验证测试通过**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib services::inference::llama_adapter::tests`
  Expected: PASS

- [ ] **Step 3: 编写 `ActiveContext` 声明并集成到 `LlamaCppAdapter` 成员中**
  在 `src-tauri/src/services/inference/llama_adapter.rs` 的合适位置写入以下具体数据结构定义并修改 `LlamaCppAdapter`：
  ```rust
  // 在 llama_adapter.rs 前部新增导入
  use llama_cpp_4::token::LlamaToken;
  use llama_cpp_4::context::LlamaContext;

  pub struct ActiveContext {
      pub model: Arc<LlamaModel>,
      pub context: LlamaContext<'static>,
      pub history_tokens: Vec<LlamaToken>,
  }

  unsafe impl Send for ActiveContext {}
  unsafe impl Sync for ActiveContext {}
  ```
  修改 `LlamaCppAdapter` 的声明与 `new` 函数：
  ```rust
  pub struct LlamaCppAdapter {
      pub is_loaded: bool,
      backend: Option<Arc<LlamaBackend>>,
      model: Option<Arc<LlamaModel>>,
      model_path: Option<PathBuf>,
      context_size: u32,
      n_threads: i32,
      pub active_context: Arc<tokio::sync::Mutex<Option<ActiveContext>>>,
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
              active_context: Arc::new(tokio::sync::Mutex::new(None)),
          }
      }
  }
  ```
  在 `impl InferenceEngine for LlamaCppAdapter` 的 `unload` 中清理 Context：
  ```rust
      async fn unload(&mut self) -> Result<(), AppError> {
          let mut active_ctx = self.active_context.lock().await;
          *active_ctx = None;
          self.model = None;
          self.model_path = None;
          self.is_loaded = false;
          Ok(())
      }
  ```

- [ ] **Step 4: 编译检查是否有异常**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 5: 提交**
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs
  git commit -m "feat: add ActiveContext and integrate it to LlamaCppAdapter"
  ```

---

### Task 2: 实现前缀比对与裁剪逻辑

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 编写前缀比对算法的测试**
  在测试模块中，编写测试函数验证前缀匹配和历史 tokens 截断逻辑：
  ```rust
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
  ```

- [ ] **Step 2: 运行测试验证失败**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib services::inference::llama_adapter::tests`
  Expected: FAIL (由于未定义 `calculate_prefix` 辅助函数)

- [ ] **Step 3: 实现 `calculate_prefix` 辅助函数**
  在 `llama_adapter.rs` 中实现 `calculate_prefix` 比较算法：
  ```rust
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
  ```

- [ ] **Step 4: 运行测试验证通过**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib services::inference::llama_adapter::tests`
  Expected: PASS

- [ ] **Step 5: 提交**
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs
  git commit -m "feat: implement calculate_prefix token match algorithm"
  ```

---

### Task 3: 重构 `stream_chat` 实现增量 Decode 与异常回退机制

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 修改 `stream_chat` 主流程，复用或构建 `ActiveContext`**
  修改 `stream_chat` 前半部分，加锁获取 `ActiveContext` 并执行裁剪/前缀匹配路由：
  ```rust
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

          // 1. Build prompt
          let model_name = self.model_path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
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

          let backend = GLOBAL_BACKEND.clone();
          let active_ctx_arc = self.active_context.clone();

          println!("\n>>> [Inference] 🚀 正在生成 AI 回复...");

          tokio::task::spawn_blocking(move || {
              let mut active_ctx_guard = active_ctx_arc.blocking_lock();
              
              // 确保 ActiveContext 实例化
              if active_ctx_guard.is_none() {
                  let n_batch = 512;
                  let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                      .with_n_ctx(NonZeroU32::new(ctx_size))
                      .with_n_batch(n_batch)
                      .with_n_threads(n_threads)
                      .with_n_threads_batch(n_threads)
                      .with_flash_attention(true);

                  let ctx = model
                      .new_context(&backend, ctx_params)
                      .map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;
                  
                  // 使用 transmute 擦除生命周期
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

              // 计算最大公共前缀
              let mut common_len = calculate_prefix(history_tokens, &tokens);
              let n_batch = 512;

              // 物理越界防护：如果 prompt 接近上限，强行全部清空
              if tokens.len() + sampling.max_tokens as usize > ctx_size as usize {
                  println!("[Inference] 接近上下文物理极限，强行重置 KV Cache。");
                  ctx.clear_kv_cache();
                  history_tokens.clear();
                  common_len = 0;
              }

              // 清理过期 KV 缓存
              if common_len < history_tokens.len() {
                  if common_len > 0 {
                      println!("[Inference] 差异生成：清理位置 {} 以后的 KV 缓存", common_len);
                      if let Err(e) = ctx.clear_kv_cache_seq(Some(0), Some(common_len as u32), None) {
                          println!("[ERROR] 清理部分缓存失败: {:?}", e);
                          ctx.clear_kv_cache();
                          history_tokens.clear();
                          common_len = 0;
                      } else {
                          history_tokens.truncate(common_len);
                      }
                  } else {
                      println!("[Inference] 清空全部 KV 缓存");
                      ctx.clear_kv_cache();
                      history_tokens.clear();
                  }
              }

              // 增量解码 decode_batch
              let mut batch_pos = common_len as i32;
              let mut last_chunk_len = 0;
              let decode_slice = &tokens[common_len..];

              for chunk in decode_slice.chunks(n_batch as usize) {
                  let mut batch = llama_cpp_4::llama_batch::LlamaBatch::new(chunk.len(), 1);
                  for (i, token) in chunk.iter().enumerate() {
                      let is_last = (batch_pos + i as i32) == (tokens.len() as i32 - 1);
                      batch
                          .add(*token, batch_pos + i as i32, &[0], is_last)
                          .map_err(|e| AppError::Internal(format!("Batch add error: {}", e)))?;
                  }
                  
                  if let Err(e) = ctx.decode(&mut batch) {
                      println!("[ERROR] 解码错误: {:?}。重试全局清空自愈。", e);
                      ctx.clear_kv_cache();
                      history_tokens.clear();
                      return Err(AppError::Internal(format!("Decode error: {}", e)));
                  }
                  
                  batch_pos += chunk.len() as i32;
                  last_chunk_len = chunk.len();
              }

              // 准备循环生成
              let mut n_cur = tokens.len() as i32;
              let mut batch_index_to_sample = if last_chunk_len > 0 {
                  (last_chunk_len - 1) as i32
              } else {
                  // 如果全部命中前缀，则采样上一次 tokens 序列的最后一个 token
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
                  if lower_token.contains("<unused") || lower_token.contains("unused") || lower_token.contains("<|im_end|>") {
                      break;
                  }

                  if tx.blocking_send(token_str).is_err() {
                      break;
                  }

                  new_generated_tokens.push(next_token);

                  let mut batch = llama_cpp_4::llama_batch::LlamaBatch::new(1, 1);
                  batch.add(next_token, n_cur, &[0], true).map_err(|e| {
                      AppError::Internal(format!("Batch add error: {}", e))
                  })?;

                  if let Err(e) = ctx.decode(&mut batch) {
                      println!("[ERROR] 循环解码错误: {:?}", e);
                      break;
                  }

                  n_cur += 1;
                  total_tokens += 1;
                  batch_index_to_sample = 0;
              }

              // 将本次生成过程中 decode 并输入了 context 的所有 tokens 同步追加到 history_tokens 中
              let mut final_history = tokens.clone();
              final_history.extend(new_generated_tokens);
              *history_tokens = final_history;

              Ok(())
          })
          .await
          .map_err(|e| AppError::Internal(format!("Blocking task panicked: {}", e)))?
      }
  ```

- [ ] **Step 2: 编译检查代码是否有语法或借用错误**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 3: 提交**
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs
  git commit -m "feat: refactor stream_chat to support KV cache prefix pruning and incremental decoding"
  ```

---

### Task 4: 动态线程调度策略设计与 Flash Attention 适配

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 实现当使用 GPU 加速时，限制线程为 1 的动态决策**
  在 `stream_chat` 内置 Context 实例化时，如果配置的 `n_gpu_layers > 0`，修改 `n_threads` 值为 `1`；若为纯 CPU 模式，采用系统的物理核心数。
  在 `stream_chat` 开始部分的同步线程代码中，动态优化参数配置：
  ```rust
              // 动态线程配置逻辑
              let mut active_threads = n_threads;
              // 检查当前模型是否开启了 GPU 卸载层
              // 通过外部 ModelLoadOptions.n_gpu_layers 设置传入来决策 (可从 self 获取)
              let has_gpu = self.model_path.is_some() && (options_or_setting_indicates_gpu_is_active); 
  ```
  由于在 `stream_chat` 中我们只有 `self.n_threads`，为了拿到 `n_gpu_layers` 并决策，我们修改 `LlamaCppAdapter` 的 `load_model`，在其中将 `n_gpu_layers` 缓存进 `LlamaCppAdapter` 成员变量中：
  修改 `LlamaCppAdapter` 结构：
  ```rust
  pub struct LlamaCppAdapter {
      pub is_loaded: bool,
      backend: Option<Arc<LlamaBackend>>,
      model: Option<Arc<LlamaModel>>,
      model_path: Option<PathBuf>,
      context_size: u32,
      n_threads: i32,
      n_gpu_layers: u32, // 新置：保存加载时的 GPU 层数
      pub active_context: Arc<tokio::sync::Mutex<Option<ActiveContext>>>,
  }
  ```
  在 `load_model` 里保存：
  ```rust
         let n_gpu_layers = get_gpu_layers(options.n_gpu_layers);
         // ...
         self.n_gpu_layers = n_gpu_layers;
  ```
  在 `stream_chat` 中实现线程动态配置：
  ```rust
          let is_gpu_active = self.n_gpu_layers > 0;
          let target_threads = if is_gpu_active {
              1 // GPU 活跃时，仅使用 1 个 CPU 线程进行控制流派发，避免抢占总线
          } else {
              n_threads // 纯 CPU 时，使用用户配置的线程数
          };
  ```
  并在 `LlamaContextParams` 创建时传入：
  ```rust
                  let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                      .with_n_ctx(NonZeroU32::new(ctx_size))
                      .with_n_batch(n_batch)
                      .with_n_threads(target_threads)
                      .with_n_threads_batch(target_threads)
                      .with_flash_attention(true);
  ```

- [ ] **Step 2: 编译测试**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 3: 提交**
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs
  git commit -m "feat: dynamically limit cpu threads to 1 when GPU offloading is active"
  ```

---

### Task 5: 整体集成验证与功能回归

**Files:**
- Test: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 编写端到端 Token 回归验证测试**
  在 `llama_adapter.rs` 底部测试部分编写完整的增量 Prefill 验证逻辑，模拟多次连续提问，并确认第二次的前缀长度不为 0：
  ```rust
      // 验证重复 prompt 时匹配的前缀长度应等于上一次的长度
  ```
- [ ] **Step 2: 运行测试验证**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 3: 运行完整编译，保证桌面应用一切正常**
  Run: `npm run tauri build -- --no-bundle`
  Expected: SUCCESS

- [ ] **Step 4: 提交**
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs
  git commit -m "test: add integration test and verify telemetry outputs"
  ```
