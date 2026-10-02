# Vision 模型图片对话 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 升级推理引擎到 llama-cpp-4 并实现原生 Vision 多模态图片对话能力。

**Architecture:** 将 `llama-cpp-2` 替换为 `llama-cpp-4`（启用 `mtmd` feature），新建 `VisionAdapter` 利用 `MtmdContext` 处理图片+文字输入，在 `rag_query` 中根据是否有图片分流到 Vision 或 Text 推理路径。

**Tech Stack:** Rust, llama-cpp-4 (mtmd + metal), Tauri v2, Vue 3

---

## 文件结构

| 操作 | 文件 | 职责 |
|------|------|------|
| 修改 | `src-tauri/Cargo.toml` | `llama-cpp-2` → `llama-cpp-4` |
| 修改 | `src-tauri/src/services/llama_backend.rs` | 适配 `llama_cpp_4` import |
| 修改 | `src-tauri/src/services/inference/llama_adapter.rs` | 适配 `llama_cpp_4` API |
| 修改 | `src-tauri/src/services/embedder.rs` | 适配 `llama_cpp_4` API |
| 新建 | `src-tauri/src/services/inference/vision_adapter.rs` | Vision 推理适配器 |
| 修改 | `src-tauri/src/services/inference/mod.rs` | 注册 vision_adapter 模块 |
| 重写 | `src-tauri/src/services/parser/vision.rs` | 移除 Fallback 占位，改为 base64 解码工具 |
| 修改 | `src-tauri/src/commands/rag.rs` | 图片分流：有图走 VisionAdapter，无图走 LlamaCppAdapter |
| 修改 | `src/stores/chat.ts` | 移除 imagesIgnoredNotice 和 chat-images-ignored 监听 |
| 修改 | `src/views/Chat.vue` | 移除 imagesIgnoredNotice UI |

---

### Task 1: 升级 Cargo 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 替换依赖**

```toml
# 移除这行:
llama-cpp-2 = { version = "0.1" }

# 替换为:
llama-cpp-4 = { version = "0.2", features = ["mtmd", "metal"] }
```

- [ ] **Step 2: 运行 cargo check 确认依赖能解析**

Run: `cd src-tauri && cargo check 2>&1 | head -20`
Expected: 会有编译错误（因为 import 路径还没改），但依赖解析应该成功。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: replace llama-cpp-2 with llama-cpp-4 (mtmd + metal)"
```

---

### Task 2: 适配 llama_backend.rs

**Files:**
- Modify: `src-tauri/src/services/llama_backend.rs`

- [ ] **Step 1: 更新 import**

将文件内容从：
```rust
use llama_cpp_2::llama_backend::LlamaBackend;
use std::sync::{Arc, LazyLock};

pub static GLOBAL_BACKEND: LazyLock<Arc<LlamaBackend>> = LazyLock::new(|| {
    let backend = LlamaBackend::init().expect("Failed to initialize llama backend");
    Arc::new(backend)
});
```

改为：
```rust
use llama_cpp_4::llama_backend::LlamaBackend;
use std::sync::{Arc, LazyLock};

pub static GLOBAL_BACKEND: LazyLock<Arc<LlamaBackend>> = LazyLock::new(|| {
    let backend = LlamaBackend::init().expect("Failed to initialize llama backend");
    Arc::new(backend)
});
```

注意：`LlamaBackend::init()` 在 llama-cpp-4 中可能返回 `Result` 而非直接值，需根据编译器反馈调整。

- [ ] **Step 2: cargo check 此文件**

Run: `cd src-tauri && cargo check 2>&1 | grep "llama_backend"`
Expected: 此文件不再报错。

---

### Task 3: 适配 embedder.rs

**Files:**
- Modify: `src-tauri/src/services/embedder.rs`

- [ ] **Step 1: 全局替换 `llama_cpp_2` → `llama_cpp_4`**

所有 `llama_cpp_2::` 引用替换为 `llama_cpp_4::`（共 5 处）：
- L2: `use llama_cpp_4::model::LlamaModel;`
- L31: `llama_cpp_4::model::params::LlamaModelParams::default()`
- L63: `llama_cpp_4::context::params::LlamaContextParams::default()`
- L75: `llama_cpp_4::model::AddBos::Always`
- L85: `llama_cpp_4::llama_batch::LlamaBatch::new(...)`

注意：`AddBos` 枚举、`LlamaBatch::new` 签名可能在 llama-cpp-4 中有变化，需根据编译器错误适配。

- [ ] **Step 2: cargo check 确认编译**

Run: `cd src-tauri && cargo check 2>&1 | grep -i "embedder"`

---

### Task 4: 适配 llama_adapter.rs

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 全局替换 `llama_cpp_2` → `llama_cpp_4`**

所有 `llama_cpp_2::` 引用替换为 `llama_cpp_4::`（约 8 处，L5-L8, L147, L188, L211, L225, L305）。

注意关键 API 差异点：
- `LlamaModel::load_from_file` — 参数可能调整为接受 `&Path` 而非 `&str`
- `LlamaBatch::new(size, n_seq_max)` — 签名可能变化
- `LlamaSampler::chain` — API 可能有变化
- `model.token_to_piece` — 返回类型/参数可能不同

每处替换后，根据编译器错误逐个修复 API 差异。

- [ ] **Step 2: cargo check 确认编译通过**

Run: `cd src-tauri && cargo check 2>&1 | grep "error" | head -20`
Expected: 0 errors

- [ ] **Step 3: Commit 全部 llama-cpp-4 适配**

```bash
git add -A
git commit -m "refactor: migrate all llama-cpp-2 imports to llama-cpp-4"
```

---

### Task 5: 新建 VisionAdapter

**Files:**
- Create: `src-tauri/src/services/inference/vision_adapter.rs`
- Modify: `src-tauri/src/services/inference/mod.rs`

- [ ] **Step 1: 创建 vision_adapter.rs**

```rust
use crate::errors::AppError;
use crate::services::llama_backend::GLOBAL_BACKEND;
use llama_cpp_4::llama_backend::LlamaBackend;
use llama_cpp_4::llama_batch::LlamaBatch;
use llama_cpp_4::model::params::LlamaModelParams;
use llama_cpp_4::model::LlamaModel;
use llama_cpp_4::mtmd::{MtmdBitmap, MtmdContext, MtmdContextParams, MtmdInputChunks, MtmdInputText};
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
            return Ok(());
        }

        let path_buf = model_path.to_path_buf();
        let backend = GLOBAL_BACKEND.clone();

        let model = tokio::task::spawn_blocking(move || {
            let params = LlamaModelParams::default()
                .with_n_gpu_layers(n_gpu_layers as u32);
            LlamaModel::load_from_file(&backend, &path_buf, &params)
                .map(Arc::new)
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
        Ok(())
    }

    /// Stream vision inference: process images + text query
    pub async fn stream_vision_chat(
        &self,
        query: &str,
        image_bytes_list: Vec<Vec<u8>>,
        sampling_temp: f32,
        max_tokens: i32,
        tx: mpsc::Sender<String>,
        cancel: Arc<CancellationToken>,
    ) -> Result<(), AppError> {
        let model = self.model.as_ref()
            .ok_or_else(|| AppError::Internal("Vision model not loaded".into()))?
            .clone();
        let mmproj_path = self.mmproj_path.as_ref()
            .ok_or_else(|| AppError::Internal("mmproj path not set".into()))?
            .clone();
        let ctx_size = self.context_size;
        let n_threads = self.n_threads;
        let query = query.to_string();

        let backend = GLOBAL_BACKEND.clone();

        tokio::task::spawn_blocking(move || {
            // 1. Create MtmdContext from mmproj file
            let mtmd_params = MtmdContextParams::default()
                .use_gpu(true)
                .n_threads(n_threads)
                .print_timings(false);

            let mtmd_ctx = MtmdContext::init_from_file(&mmproj_path, &model, mtmd_params)
                .map_err(|e| AppError::Internal(format!("Failed to load mmproj: {}", e)))?;

            // 2. Create LLM context
            let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(ctx_size))
                .with_n_batch(512);

            let mut lctx = model.new_context(&backend, ctx_params)
                .map_err(|e| AppError::Internal(format!("Failed to create context: {}", e)))?;

            // 3. Load image bitmaps
            let marker = MtmdContext::default_marker();
            let mut bitmaps: Vec<MtmdBitmap> = Vec::new();
            for bytes in &image_bytes_list {
                let bm = MtmdBitmap::from_bytes(&mtmd_ctx, bytes)
                    .map_err(|e| AppError::Internal(format!("Failed to load image: {}", e)))?;
                bitmaps.push(bm);
            }

            // 4. Build prompt with media markers
            let markers: String = (0..bitmaps.len())
                .map(|_| marker.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let prompt = if bitmaps.is_empty() {
                query.clone()
            } else {
                format!("{} {}", markers, query)
            };

            // 5. Tokenize
            let input_text = MtmdInputText::new(&prompt, true, true);
            let bitmap_refs: Vec<&MtmdBitmap> = bitmaps.iter().collect();
            let mut chunks = MtmdInputChunks::new();
            mtmd_ctx.tokenize(&input_text, &bitmap_refs, &mut chunks)
                .map_err(|e| AppError::Internal(format!("Tokenize failed: {}", e)))?;

            // 6. Eval all chunks
            let mut n_past: i32 = 0;
            mtmd_ctx.eval_chunks(
                lctx.as_ptr(), &chunks, 0, 0, 512, true, &mut n_past,
            ).map_err(|e| AppError::Internal(format!("Eval failed: {}", e)))?;

            // 7. Sample loop
            let eos = model.token_eos();
            let mut generated = 0i32;

            loop {
                if cancel.is_cancelled() { break; }
                if generated >= max_tokens { break; }

                let logits = lctx.get_logits();
                let next_token = logits.iter().enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(i, _)| LlamaToken::new(i as i32))
                    .unwrap();

                if model.is_eog_token(next_token) || next_token == eos {
                    break;
                }

                let piece = model.token_to_str(next_token, llama_cpp_4::model::Special::Tokenize)
                    .unwrap_or_default();

                if tx.blocking_send(piece).is_err() { break; }

                let mut batch = LlamaBatch::new(1, 0);
                batch.add(next_token, n_past, &[0], true)
                    .map_err(|e| AppError::Internal(format!("Batch add: {}", e)))?;
                lctx.decode(&mut batch)
                    .map_err(|e| AppError::Internal(format!("Decode: {}", e)))?;

                n_past += 1;
                generated += 1;
            }

            println!("[Vision] Done, generated {} tokens", generated);
            Ok(())
        })
        .await
        .map_err(|e| AppError::Internal(format!("Vision task panicked: {}", e)))?
    }

    pub fn unload(&mut self) {
        self.model = None;
        self.model_path = None;
        self.mmproj_path = None;
        self.is_loaded = false;
    }
}
```

- [ ] **Step 2: 在 mod.rs 注册模块**

在 `src-tauri/src/services/inference/mod.rs` 末尾添加：
```rust
pub mod vision_adapter;
```

- [ ] **Step 3: cargo check**

Run: `cd src-tauri && cargo check 2>&1 | grep "error" | head -10`

注意：`MtmdBitmap::from_bytes`、`model.token_to_str`、`LlamaBatch::new` 等 API 签名需要根据编译器反馈调整。llama-cpp-4 的 API 仍在快速迭代中，以编译器为准。

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add VisionAdapter for multimodal image inference"
```

---

### Task 6: 重写 vision.rs（移除 Fallback 占位）

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`

- [ ] **Step 1: 重写为 base64 解码工具函数**

```rust
use crate::errors::AppError;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// Decode a data URL (e.g. "data:image/png;base64,xxxxx") into raw bytes.
pub fn decode_data_url(data_url: &str) -> Result<Vec<u8>, AppError> {
    let base64_part = if let Some(pos) = data_url.find(",") {
        &data_url[pos + 1..]
    } else {
        data_url
    };

    BASE64
        .decode(base64_part)
        .map_err(|e| AppError::Internal(format!("Invalid base64 image data: {}", e)))
}

/// Decode multiple data URLs into raw byte vectors.
pub fn decode_data_urls(data_urls: &[String]) -> Result<Vec<Vec<u8>>, AppError> {
    data_urls.iter().map(|url| decode_data_url(url)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_data_url_with_prefix() {
        let data = "data:image/png;base64,aGVsbG8=";
        let result = decode_data_url(data).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_decode_data_url_raw_base64() {
        let data = "aGVsbG8=";
        let result = decode_data_url(data).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_decode_data_urls_multiple() {
        let urls = vec![
            "data:image/png;base64,aGVsbG8=".to_string(),
            "data:image/jpeg;base64,d29ybGQ=".to_string(),
        ];
        let results = decode_data_urls(&urls).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], b"hello");
        assert_eq!(results[1], b"world");
    }
}
```

- [ ] **Step 2: cargo test 运行单元测试**

Run: `cd src-tauri && cargo test parser::vision --lib -- --nocapture`
Expected: 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "refactor: rewrite vision.rs as base64 decode utility, remove Fallback"
```

---

### Task 7: 修改 rag_query 图片分流逻辑

**Files:**
- Modify: `src-tauri/src/commands/rag.rs`

- [ ] **Step 1: 替换图片处理分支（L53-L110）**

将现有的 `ImageProcessingResult::Fallback` / `Processed` 分支替换为：

```rust
    // Process images: decode base64 if present
    let has_images = images.as_ref().map_or(false, |imgs| !imgs.is_empty());

    if has_images {
        // Check vision model config
        let vision_model_file = settings::get_setting_cached(&conn, &settings_cache, "vision_model")?
            .unwrap_or_default();
        let vision_mmproj_file = settings::get_setting_cached(&conn, &settings_cache, "vision_mmproj")?
            .unwrap_or_default();

        if vision_model_file.is_empty() || vision_mmproj_file.is_empty() {
            return Err(AppError::Internal(
                "请先在设置中配置 Vision 模型和 mmproj 文件，才能处理图片。".to_string()
            ));
        }

        // Decode base64 images
        let image_bytes = crate::services::parser::vision::decode_data_urls(
            images.as_ref().unwrap()
        )?;

        // --- Vision inference path (separate from text-only path below) ---
        // Build vision model path
        let app_data_dir = app_handle.path().app_data_dir()
            .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
        let vision_model_path = app_data_dir.join("downloads").join(&vision_model_file);
        let mmproj_path = app_data_dir.join("downloads").join(&vision_mmproj_file);

        if !vision_model_path.exists() {
            return Err(AppError::Internal(format!("Vision model not found: {:?}", vision_model_path)));
        }
        if !mmproj_path.exists() {
            return Err(AppError::Internal(format!("mmproj file not found: {:?}", mmproj_path)));
        }

        // Read settings for inference
        let num_ctx = settings::get_num_ctx(&conn, Some(&settings_cache))?;
        let num_predict = settings::get_num_predict(&conn, Some(&settings_cache))?;
        let temperature = settings::get_temperature(&conn, Some(&settings_cache))?;
        let num_gpu = settings::get_num_gpu(&conn, Some(&settings_cache))?;
        let num_thread = settings::get_num_thread(&conn, Some(&settings_cache))?;

        // Insert user + assistant messages (same pattern as text path)
        // ... (reuse existing message insertion logic, then spawn vision inference)
        
        // Spawn vision inference task using VisionAdapter
        // (follows same tx/rx pattern as existing LlamaCppAdapter path)

        return Ok(());
    }

    // --- Text-only path: existing logic unchanged below ---
```

完整实现需要复制现有的消息插入逻辑（user msg + assistant msg 插入 DB + emit 事件），然后用 `VisionAdapter` 替换 `LlamaCppAdapter`。由于这段代码较长且需要和现有代码精确对齐，实现时请参考 L113-L440 的现有模式。

- [ ] **Step 2: cargo check**

Run: `cd src-tauri && cargo check 2>&1 | grep "error" | head -10`
Expected: 0 errors

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: route image queries to VisionAdapter in rag_query"
```

---

### Task 8: 前端清理（移除 Fallback UI）

**Files:**
- Modify: `src/stores/chat.ts`
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: 清理 chat.ts**

1. 删除 L47 的 `imagesIgnoredNotice` ref 声明
2. 删除 L128-135 的 `listen('chat-images-ignored', ...)` 监听器
3. 删除 L491 的 `imagesIgnoredNotice` 导出

- [ ] **Step 2: 清理 Chat.vue**

删除 L219-232 的 `imagesIgnoredNotice` 横幅 UI 组件。

- [ ] **Step 3: tsc 检查**

Run: `cd /Users/mac/project/telepathy && npx tsc --noEmit 2>&1 | head -20`
Expected: 0 errors

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: remove image fallback UI (vision is now natively supported)"
```

---

### Task 9: 设置页面增加 mmproj 配置

**Files:**
- Modify: `src/components/settings/ModelManager.vue`
- Modify: `src/views/Models.vue`

- [ ] **Step 1: 在 ModelManager.vue 中添加 mmproj 选择**

在 visionModel 的 `SelectRoot` 下方（L437 之后）新增一个 mmproj 文件选择器。由于 mmproj 文件也是 GGUF 文件，可以从已安装模型中筛选名称包含 `mmproj` 的文件，或者提供一个文本输入框让用户手动输入文件名。

- [ ] **Step 2: 在 Models.vue 中添加 mmproj 保存逻辑**

在 `select-vision-model` handler 附近添加 `vision_mmproj` 的 saveSetting 调用。

- [ ] **Step 3: tsc 检查 + Commit**

```bash
npx tsc --noEmit
git add -A
git commit -m "feat: add vision mmproj configuration in settings UI"
```

---

### Task 10: 端到端验证

- [ ] **Step 1: 下载一个小型 Vision 模型用于测试**

推荐 `moondream2`（约 1.7GB），需要下载主模型 + mmproj 两个 GGUF 文件。

- [ ] **Step 2: 在设置中配置 Vision 模型和 mmproj**

- [ ] **Step 3: 上传图片并发送对话，验证**
- 纯图片（无文字）→ 模型应能描述图片
- 图片+文字 → 模型应能根据图片回答问题
- 纯文字 → 走现有 chat 模型（不受影响）

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: vision model support - complete implementation"
```
