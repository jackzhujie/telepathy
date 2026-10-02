# Native Multi-Engine Inference (Phase 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Completely remove the external Ollama process dependency and embed `llama.cpp` natively in the Tauri Rust backend via `llama-cpp-2`.

**Architecture:** A unified `InferenceEngine` trait acts as the abstraction layer. A new `LlamaCppAdapter` implements this trait using native FFI bindings. The model manager directly scans local files instead of querying an external daemon.

**Tech Stack:** Rust, Tauri v2, `llama-cpp-2` (Metal/CUDA enabled), SQLite (`rusqlite`).

---

### Task 1: Update Cargo Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: Update Cargo.toml dependencies**
Remove `ollama-rs` and `ollama_models_info_fetcher`. Add `llama-cpp-2`.

```toml
# In src-tauri/Cargo.toml
# Remove:
# ollama-rs = { version = "0.2.1", features = ["stream"] }
# ollama_models_info_fetcher = "0.1.3"

# Add:
llama-cpp-2 = { version = "0.3" }
```

- [ ] **Step 2: Clean up old service module references**
Remove the `ollama` module from `src-tauri/src/services/mod.rs`. Add `pub mod inference;`.

```rust
// In src-tauri/src/services/mod.rs
// Remove: pub mod ollama;
pub mod inference;
```

- [ ] **Step 3: Commit**
```bash
cargo check --manifest-path src-tauri/Cargo.toml
git add src-tauri/Cargo.toml src-tauri/src/services/mod.rs
git commit -m "chore: update dependencies for native inference"
```

---

### Task 2: Define Inference Engine Trait

**Files:**
- Create: `src-tauri/src/services/inference/mod.rs`

- [ ] **Step 1: Define the core traits and structs**

```rust
use async_trait::async_trait;
use crate::errors::AppError;
use std::path::Path;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ModelLoadOptions {
    pub n_gpu_layers: i32,
    pub context_size: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineStats {
    pub vram_used_mb: u64,
    pub engine_type: String,
}

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn load_model(&mut self, model_path: &Path, options: ModelLoadOptions) -> Result<(), AppError>;
    async fn stream_chat(&self, messages: Vec<Message>, tx: mpsc::Sender<String>) -> Result<(), AppError>;
    fn get_stats(&self) -> EngineStats;
    async fn unload(&mut self) -> Result<(), AppError>;
}

pub mod llama_adapter;
```

- [ ] **Step 2: Commit**
```bash
git add src-tauri/src/services/inference/mod.rs
git commit -m "feat: define InferenceEngine trait abstraction"
```

---

### Task 3: Implement LlamaCppAdapter Stub

**Files:**
- Create: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: Create the LlamaCppAdapter implementation**
Write the skeleton for the adapter.

```rust
use super::{InferenceEngine, ModelLoadOptions, EngineStats, Message};
use crate::errors::AppError;
use async_trait::async_trait;
use std::path::Path;
use tokio::sync::mpsc;

pub struct LlamaCppAdapter {
    is_loaded: bool,
    // Future: backend/context struct from llama-cpp-2
}

impl LlamaCppAdapter {
    pub fn new() -> Self {
        Self { is_loaded: false }
    }
}

#[async_trait]
impl InferenceEngine for LlamaCppAdapter {
    async fn load_model(&mut self, _model_path: &Path, _options: ModelLoadOptions) -> Result<(), AppError> {
        // TODO: integrate llama-cpp-2 LlamaBackend and LlamaModel
        self.is_loaded = true;
        Ok(())
    }

    async fn stream_chat(&self, _messages: Vec<Message>, _tx: mpsc::Sender<String>) -> Result<(), AppError> {
        if !self.is_loaded {
            return Err(AppError::Internal("Model not loaded".into()));
        }
        // TODO: context creation and llama-cpp-2 generation loop
        Ok(())
    }

    fn get_stats(&self) -> EngineStats {
        EngineStats {
            vram_used_mb: 0,
            engine_type: "LlamaCpp".into(),
        }
    }

    async fn unload(&mut self) -> Result<(), AppError> {
        self.is_loaded = false;
        Ok(())
    }
}
```

- [ ] **Step 2: Commit**
```bash
cargo check --manifest-path src-tauri/Cargo.toml
git add src-tauri/src/services/inference/llama_adapter.rs
git commit -m "feat: implement LlamaCppAdapter stub"
```

---

### Task 4: Refactor Model Management & DB Schema

**Files:**
- Modify: `src-tauri/src/services/model_registry.rs`
- Modify: `src-tauri/src/commands/models.rs`

- [ ] **Step 1: Update `InstalledModel` struct**
In `src-tauri/src/services/model_registry.rs`, remove `tag` and add `engine_type`, `format`, `file_path`.

```rust
// Replace inside InstalledModel struct:
// pub tag: String,
// with:
pub engine_type: String,
pub format: String,
pub file_path: String,
```

- [ ] **Step 2: Rewrite `list_installed_models`**
In `src-tauri/src/commands/models.rs`, remove the ollama client initialization and scanning. Instead, scan the app data `downloads` directory for `.gguf` files.

```rust
// In src-tauri/src/commands/models.rs
// Rewrite list_installed_models:
#[tauri::command]
pub async fn list_installed_models(app_handle: AppHandle) -> Result<Vec<InstalledModel>, AppError> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let downloads_dir = app_data_dir.join("downloads");
    
    let mut installed = Vec::new();
    let registry = model_registry::load_registry()?;

    if downloads_dir.exists() {
        let mut entries = tokio::fs::read_dir(downloads_dir).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
            
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
                
                let (model_type, display_name, provider, description) =
                    model_registry::match_model_type(&registry, &file_name);
                    
                installed.push(InstalledModel {
                    full_name: file_name.clone(),
                    base_name: file_name.clone(),
                    engine_type: "LlamaCpp".to_string(),
                    format: "GGUF".to_string(),
                    file_path: path.to_string_lossy().to_string(),
                    model_type,
                    size_bytes: size,
                    display_name,
                    provider,
                    description,
                });
            }
        }
    }
    Ok(installed)
}
```

- [ ] **Step 3: Refactor `install_model` and `delete_model`**
In `src-tauri/src/commands/models.rs`, inside `install_model`, remove `ollama::import_local_gguf` completely. Keep the downloaded file in `downloads_dir` and just emit success.
In `delete_model`, directly use `std::fs::remove_file` to delete the `.gguf` file instead of `ollama::delete_model`.

- [ ] **Step 4: Commit**
```bash
cargo check --manifest-path src-tauri/Cargo.toml
git add src-tauri/src/services/model_registry.rs src-tauri/src/commands/models.rs
git commit -m "refactor: rewrite model manager to scan local directory instead of ollama"
```

---

### Task 5: Purge Old Ollama Code

**Files:**
- Delete: `src-tauri/src/services/ollama.rs`
- Modify: `src-tauri/src/commands/document.rs` (Remove any ollama embedding client usage and prepare for native embeddings later, just stub it for now or return error).

- [ ] **Step 1: Delete `ollama.rs`**
```bash
rm src-tauri/src/services/ollama.rs
```

- [ ] **Step 2: Clean up remaining compile errors**
Fix any other files that were relying on `crate::services::ollama` (like chat handlers or document embeddings). Stub them out if necessary (e.g., return `Ok(())` or empty results temporarily).

- [ ] **Step 3: Commit**
```bash
git add -u src-tauri/
git commit -m "refactor: purge all ollama references from project"
```
