# Embedder Context Crash Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the Tauri application crash on startup/indexing by optimizing LlamaContext creation parameters for the embedding model in CPU mode.

**Architecture:** Modify `src/services/embedder.rs` to explicitly set `n_batch` to 512, and set both `n_threads` and `n_threads_batch` to 4 in `LlamaContextParams` to avoid out-of-bounds graph buffer issues and unsafe automatic thread detection crashes on macOS. Also clean up diagnostic debug prints.

**Tech Stack:** Rust, Tauri v2, llama-cpp-4

---

### Task 1: Clean up diagnostic prints and configure LlamaContextParams in embedder.rs

**Files:**
- Modify: `src/services/embedder.rs`
- Test: `src/services/embedder.rs` (via `cargo test`)

- [ ] **Step 1: Write the implementation changes in src/services/embedder.rs**

Replace the `LlamaContextParams` configuration in `src/services/embedder.rs` to limit `n_batch` and specify threads, while cleaning up temporary `[Embedder DEBUG]` prints.

```rust
                let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                    .with_n_ctx(NonZeroU32::new(2048))
                    .with_n_batch(512)
                    .with_n_threads(4)
                    .with_n_threads_batch(4)
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
```

- [ ] **Step 2: Run cargo check to verify code builds successfully**

Run: `cargo check` (in `src-tauri` directory)
Expected: Build passes with no compilation errors.

- [ ] **Step 3: Run existing unit tests to verify no regressions and correct parameters**

Run: `cargo test --package temp-app --lib services::embedder::tests -- --nocapture` (in `src-tauri` directory)
Expected: Tests pass successfully and we should see llama_context prints indicating:
`llama_context: n_batch       = 512`
`llama_context: n_ctx         = 2048`
Instead of `n_batch = 2048`.

- [ ] **Step 4: Commit the changes**

Run:
```bash
git add src/services/embedder.rs
git commit -m "fix(embedder): limit n_batch to 512 and n_threads to 4 to prevent CPU context creation crashes"
```
