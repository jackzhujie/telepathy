# 恢复多模态视觉引擎的 GPU 调度 (vision_adapter.rs) 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 恢复 `vision_adapter.rs` 中多模态视觉引擎的原版 GPU 调度逻辑，移除在 macOS 非 M 芯片下的强制拦截和硬编码的 `.with_main_gpu(-1)`。

**Architecture:** 直接重构 `load_model` 方法中关于 `gpu_layers` 变量初始化和 `LlamaModelParams` 参数构建的逻辑。

**Tech Stack:** Rust, Tauri, llama.cpp (llama_cpp_4)

---

### Task 1: 修改 vision_adapter.rs 中的 GPU layers 分配逻辑

**Files:**
- Modify: `src-tauri/src/services/inference/vision_adapter.rs`

- [ ] **Step 1: 修改 gpu_layers 和 params 构造逻辑**

修改 `src-tauri/src/services/inference/vision_adapter.rs` 中的第 62 行至第 95 行，移除原有的 macOS 非 M 系列芯片强制禁用 Metal 和硬编码 `.with_main_gpu(-1)` 的逻辑，修改为原版简洁的参数分配方式：
```rust
        let gpu_layers = if n_gpu_layers > 0 {
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
```

- [ ] **Step 2: 验证编译**

在 `src-tauri` 目录下运行 `cargo check` 验证是否存在任何编译和语法错误。
期待：编译通过。

- [ ] **Step 3: 暂存并提交代码**

使用 Git 提交修改：
```bash
git add src-tauri/src/services/inference/vision_adapter.rs
git commit -m "cleanup(vision_adapter): restore original gpu_layers configuration and remove main_gpu workaround"
```
