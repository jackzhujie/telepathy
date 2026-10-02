# Model Hub Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Completely replace the Ollama HTML-based model hub with HuggingFace/ModelScope APIs for fetching and downloading exclusively GGUF models, implementing a custom resumable downloader, and transparently injecting them into Ollama with memory mismatch secondary dialogs.

**Architecture:** We will implement a `ModelRegistryClient` trait to wrap both HF and MS APIs. A dedicated downloader service utilizing `reqwest` will stream `.gguf` chunks to a local cache. Upon completion, a dynamically generated `Modelfile` will be utilized to run `ollama create` for ingestion.

**Tech Stack:** Rust (reqwest, tokio, regex), Tauri V2, Vue 3, Ollama CLI.

---

### Task 1: Unify the Registry Interface and API Clients

**Files:**
- Create: `src-tauri/src/services/model_hub/registry.rs`
- Create: `src-tauri/src/services/model_hub/clients/hf.rs`
- Create: `src-tauri/src/services/model_hub/clients/ms.rs`
- Modify: `src-tauri/src/services/model_hub/mod.rs` (Export new modules)

- [x] **Step 1: Write trait Definition**
Define `ModelRegistryClient` in `registry.rs` with `async fn fetch_trending()`, `async fn search(query: &str)`, and `async fn get_gguf_variants()`.

- [x] **Step 2: Implement HuggingFace**
In `hf.rs`, build the client invoking `huggingface.co/api/models` using `?filter=gguf` logic to map their JSON responses into our `HubModel` structs.

- [x] **Step 3: Implement ModelScope**
In `ms.rs`, build the client invoking the ModelScope API equivalent targeting GGUF tags.

- [x] **Step 4: Commit**
`git add src-tauri/src/services/model_hub && git commit -m "feat(hub): add unified registry clients for HF and MS"`

### Task 2: Custom Resumable Downloader

**Files:**
- Create: `src-tauri/src/services/model_hub/downloader.rs`
- Modify: `src-tauri/src/services/model_hub/mod.rs`

- [x] **Step 1: Write Downloader Struct**
Write `ModelDownloader` wrapping `reqwest::Client`. Track downloaded bytes by reading existing file length and appending HTTP `Range` headers.

- [x] **Step 2: Progress Bridging**
Loop over the stream bytes and frequently `tauri::AppHandle::emit` a payload (`"model-pull-progress"`) containing exact percentage and bytes per second.

- [x] **Step 3: Commit**
`git add src-tauri/src/services/model_hub/downloader.rs && git commit -m "feat(hub): implement GGUF chunked resumable downloader"`

### Task 3: Ollama Injection Mechanism

**Files:**
- Modify: `src-tauri/src/services/ollama.rs`

- [x] **Step 1: Write Modelfile Generator and Caller**
Create function `import_local_gguf(name: &str, gguf_path: &Path, app_handle: &AppHandle) -> Result<(), AppError>`. Write `FROM ./model.gguf` to a temporary `Modelfile`.

- [x] **Step 2: Execute command**
Use `tokio::process::Command` to invoke `ollama create <name> -f Modelfile`.

- [x] **Step 3: Cleanup**
Delete both the original `.gguf` and the temp `Modelfile` on successful exit code to recover disk space.

- [x] **Step 4: Commit**
`git add . && git commit -m "feat(ollama): implement local gguf transparent ingestion and cleanup"`

### Task 4: Integration in Manager & Tauri Commands

**Files:**
- Modify: `src-tauri/src/services/model_hub/manager.rs`
- Modify: `src-tauri/src/commands/models.rs`

- [x] **Step 1: Rewrite Geo Probe logic**
In `manager.rs` and `commands/models.rs`, switch from using `search_on_web` (Ollama scraping) to instantiating either `ModelScopeRegistry` or `HuggingFaceRegistry` based on `geo.rs` networking results.

- [x] **Step 2: Rewrite Install Command**
Change `pub async fn install_model()` to first invoke `downloader.download()`, then pass the result to `import_local_gguf()`.

- [x] **Step 3: Commit**
`git add . && git commit -m "feat(backend): wire new registries and downloader to Tauri commands"`

### Task 5: Frontend Warning Implementation

**Files:**
- Modify: `src/components/settings/ModelManager.vue`

- [x] **Step 1: Remove Filtering**
Remove any `v-if` elements that hide models based on hardware specs in the Vue template lists. Render the efficiency badge only.

- [x] **Step 2: Add Secondary Confirmation Dialog**
Add a `<ConfirmDialog>` logic (or native `window.confirm` / Tauri dialog) triggered inside `handleInstallDirect` if efficiency is flagged as "Too Heavy" / "Unable to Run". 

- [x] **Step 3: Commit**
`git add src/components && git commit -m "feat(ui): add hardware mismatch secondary download confirmation"`
