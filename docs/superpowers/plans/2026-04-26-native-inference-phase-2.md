# Native Inference Engine Phase 2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the real inference loop inside `LlamaCppAdapter` and wire it up to the frontend commands.

**Architecture:** We will initialize the `llama-cpp-2` backend, load the model onto the GPU (if available) via `ModelLoadOptions`, tokenize the prompt, and sample tokens in a streaming fashion, feeding them back to the frontend via Tauri events.

**Tech Stack:** Rust, llama-cpp-2, Tauri State, tokio mpsc

---

### Task 1: Update LlamaCppAdapter Struct

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: Write the failing test**

```rust
// No unit test here as it interacts with FFI which needs a real file, but we verify compilation.
// Just write a stub test to ensure we can create it.
#[test]
fn test_adapter_creation() {
    let adapter = LlamaCppAdapter::new();
    assert!(!adapter.is_loaded);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test`
Expected: FAIL due to missing fields or test passing if minimal. Note that due to `cmake` missing locally, compilation might fail entirely. We write code conceptually passing.

- [ ] **Step 3: Write minimal implementation**

Modify `LlamaCppAdapter` struct to hold model state:
```rust
use llama_cpp_2::backend::LlamaBackend;
use llama_cpp_2::model::LlamaModel;

pub struct LlamaCppAdapter {
    pub is_loaded: bool,
    backend: Option<LlamaBackend>,
    model: Option<LlamaModel>,
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
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo check` (note: may fail if cmake absent)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/inference/llama_adapter.rs
git commit -m "feat: add backend and model fields to LlamaCppAdapter"
```

### Task 2: Implement `load_model`

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: Write the failing test**
(skip direct FFI test in unit tests)

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Write minimal implementation**

Implement `load_model` inside `LlamaCppAdapter`:
```rust
use llama_cpp_2::model::params::LlamaModelParams;

#[async_trait]
impl InferenceEngine for LlamaCppAdapter {
    async fn load_model(&mut self, model_path: &Path, options: ModelLoadOptions) -> Result<(), AppError> {
        let backend = LlamaBackend::init()
            .map_err(|e| AppError::Internal(format!("Failed to init llama backend: {}", e)))?;
            
        let mut params = LlamaModelParams::default();
        params.set_n_gpu_layers(options.gpu_layers as i32);
        
        let path_str = model_path.to_str().ok_or_else(|| AppError::Internal("Invalid path string".into()))?;
        
        let model = LlamaModel::load_from_file(&backend, path_str, &params)
            .map_err(|e| AppError::Internal(format!("Failed to load model: {}", e)))?;
            
        self.backend = Some(backend);
        self.model = Some(model);
        self.is_loaded = true;
        
        Ok(())
    }
    // ... existing stubs
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/inference/llama_adapter.rs
git commit -m "feat: implement load_model using llama-cpp-2"
```

### Task 3: Implement `stream_chat`

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: Write the failing test**

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Write minimal implementation**

Implement token generation loop (simplified version):
```rust
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;

    async fn stream_chat(&self, messages: Vec<Message>, tx: mpsc::Sender<String>) -> Result<(), AppError> {
        if !self.is_loaded {
            return Err(AppError::Internal("Model not loaded".into()));
        }
        
        let backend = self.backend.as_ref().unwrap();
        let model = self.model.as_ref().unwrap();
        
        let mut ctx_params = LlamaContextParams::default();
        ctx_params.set_n_ctx(2048);
        
        let mut ctx = model.new_context(backend, ctx_params)
            .map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;
            
        // Build prompt
        let mut prompt = String::new();
        for msg in messages {
            prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
        prompt.push_str("assistant: ");
        
        let tokens = model.str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)
            .map_err(|e| AppError::Internal(format!("Failed to tokenize: {}", e)))?;
            
        // Decode
        let mut batch = llama_cpp_2::llama_batch::LlamaBatch::new(2048, 1);
        let last_index = tokens.len() - 1;
        
        for (i, token) in tokens.into_iter().enumerate() {
            let is_last = i == last_index;
            batch.add(token, i as i32, &[0], is_last)
                .map_err(|e| AppError::Internal(format!("Batch add error: {}", e)))?;
        }
        
        ctx.decode(&mut batch)
            .map_err(|e| AppError::Internal(format!("Decode error: {}", e)))?;
            
        // Dummy sampling loop
        let mut n_cur = batch.n_tokens();
        
        while n_cur < 2048 {
            let mut candidates = ctx.candidates_ith(batch.n_tokens() - 1);
            let next_token = ctx.sample_token_greedy(&mut candidates);
            
            if next_token == model.token_eos() {
                break;
            }
            
            let text = model.token_to_str(next_token)
                .map_err(|e| AppError::Internal(format!("Token to str error: {}", e)))?;
                
            if tx.send(text).await.is_err() {
                break; // receiver dropped
            }
            
            batch.clear();
            batch.add(next_token, n_cur, &[0], true)
                .map_err(|e| AppError::Internal(format!("Batch add error: {}", e)))?;
                
            ctx.decode(&mut batch)
                .map_err(|e| AppError::Internal(format!("Decode error: {}", e)))?;
                
            n_cur += 1;
        }
        
        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/inference/llama_adapter.rs
git commit -m "feat: implement stream_chat loop"
```

### Task 4: Global Engine Manager State

**Files:**
- Modify: `src-tauri/src/services/inference/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Write minimal implementation**

In `mod.rs`:
```rust
use std::sync::Arc;
use tokio::sync::Mutex;
pub struct EngineManagerState(pub Arc<Mutex<Option<Box<dyn InferenceEngine>>>>);
```

In `lib.rs`:
```rust
app.manage(crate::services::inference::EngineManagerState(std::sync::Arc::new(tokio::sync::Mutex::new(None))));
```

- [ ] **Step 4: Run test to verify it passes**

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/inference/mod.rs src-tauri/src/lib.rs
git commit -m "feat: setup global engine manager state"
```
