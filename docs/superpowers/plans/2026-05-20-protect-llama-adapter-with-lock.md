# Protect Llama Adapter with ACQUIRE_LOCK Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Protect model loading, context creation/destruction, and decoding operations in `llama_adapter.rs` using the global `ACQUIRE_LOCK` to prevent concurrency issues.

**Architecture:** Use `crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap()` around native `llama.cpp` calls to serialize operations.

**Tech Stack:** Rust, Tauri, llama.cpp

---

### Task 1: Protect Model Operations in llama_adapter.rs

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: Lock model loading in load_model**

  Modify `load_model` method to acquire the lock before calling `LlamaModel::load_from_file`.
  
  Code to replace (around line 159-166):
  ```rust
          let model_result = tokio::task::spawn_blocking(move || {
              let params = llama_cpp_4::model::params::LlamaModelParams::default()
                  .with_n_gpu_layers(n_gpu_layers);

              LlamaModel::load_from_file(&backend, &path_str, &params)
                  .map(|m| Arc::new(m))
                  .map_err(|e| AppError::Internal(format!("Failed to load model: {}", e)))
          })
  ```
  Replacement code:
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

- [ ] **Step 2: Lock context clearing in load_model**

  Modify context reset section in `load_model` to acquire the lock when dropping context.
  
  Code to replace (around line 172-175):
  ```rust
          {
              let mut active_ctx = self.active_context.lock().await;
              *active_ctx = None;
          }
  ```
  Replacement code:
  ```rust
          {
              let mut active_ctx = self.active_context.lock().await;
              let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
              *active_ctx = None;
          }
  ```

- [ ] **Step 3: Lock context creation in stream_chat**

  Modify `stream_chat` method to acquire the lock around `model.new_context`.
  
  Code to replace (around line 242-244):
  ```rust
                  let ctx = model
                      .new_context(&backend, ctx_params)
                      .map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;
  ```
  Replacement code:
  ```rust
                  let ctx = {
                      let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
                      model.new_context(&backend, ctx_params)
                  }.map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;
  ```

- [ ] **Step 4: Lock first-stage decode in stream_chat**

  Modify first decode step in `stream_chat` to acquire the lock.
  
  Code to replace (around line 315-320):
  ```rust
                  if let Err(e) = ctx.decode(&mut batch) {
                      println!("[ERROR] Decode failure: {:?}. Resetting context for self-healing.", e);
                      ctx.clear_kv_cache();
                      history_tokens.clear();
                      return Err(AppError::Internal(format!("Decode failure: {}", e)));
                  }
  ```
  Replacement code:
  ```rust
                  let decode_res = {
                      let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
                      ctx.decode(&mut batch)
                  };
                  if let Err(e) = decode_res {
                      println!("[ERROR] Decode failure: {:?}. Resetting context for self-healing.", e);
                      ctx.clear_kv_cache();
                      history_tokens.clear();
                      return Err(AppError::Internal(format!("Decode failure: {}", e)));
                  }
  ```

- [ ] **Step 5: Lock incremental generation decode in stream_chat loop**

  Modify the loop decode step in `stream_chat` to acquire the lock.
  
  Code to replace (around line 399-402):
  ```rust
                  if let Err(e) = ctx.decode(&mut batch) {
                      println!("[ERROR] Loop decode failure: {:?}", e);
                      break;
                  }
  ```
  Replacement code:
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

- [ ] **Step 6: Lock context clearing in unload**

  Modify context reset section in `unload` to acquire the lock.
  
  Code to replace (around line 439-446):
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
  Replacement code:
  ```rust
      async fn unload(&mut self) -> Result<(), AppError> {
          {
              let mut active_ctx = self.active_context.lock().await;
              let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
              *active_ctx = None;
          }
          self.model = None;
          self.model_path = None;
          self.is_loaded = false;
          Ok(())
      }
  ```

- [ ] **Step 7: Run compilation check**

  Run: `cargo check` in `src-tauri`
  Expected: SUCCESS

- [ ] **Step 8: Commit changes**

  Run:
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs
  git commit -m "feat: protect llama_adapter core operations with ACQUIRE_LOCK"
  ```
  Expected: SUCCESS
