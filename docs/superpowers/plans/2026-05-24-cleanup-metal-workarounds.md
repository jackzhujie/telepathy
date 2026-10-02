# 撤销 Metal 过度限制与清理临时诊断代码实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 彻底清除先前因误判崩溃原因而注入的临时控制台诊断日志，并撤销对 Intel Mac 设备大语言模型及多模态 GPU 运行的所有强行屏蔽和参数限制，恢复干净并享有显卡加速的大模型底座。

**Architecture:** 按文件逐一移除临时插入的诊断行，回滚各适配器的 layers 计算函数，移除强制注入的 GGML 环境变量。

**Tech Stack:** Rust, Tauri v2, llama-cpp-4

---

### Task 1: 恢复启动文件 (lib.rs)

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 删除 setenv 外部 FFI 声明及 run 启动拦截**
  编辑 [lib.rs](file:///Users/mac/project/telepathy/src-tauri/src/lib.rs)，删除第 7-15 行的 `extern "C" { fn setenv(...) }`，并删除 `run` 函数最开始（第 18-30 行）对 Intel Mac 强制执行 `setenv("GGML_METAL_DISABLE")` 的逻辑。

  ```rust
  // 恢复后的 run() 入口干净状态：
  #[cfg_attr(mobile, tauri::mobile_entry_point)]
  pub fn run() {
      // Set llama.cpp log level early, before any threads spawn (safe single-threaded context)
      if std::env::var("LLAMA_LOG_LEVEL").is_err() {
          unsafe { std::env::set_var("LLAMA_LOG_LEVEL", "0") };
      }
      ...
  ```

---

### Task 2: 恢复全局后端初始化环境 (llama_backend.rs)

**Files:**
- Modify: `src-tauri/src/services/llama_backend.rs`

- [ ] **Step 1: 移除 GLOBAL_BACKEND 处的非 M 芯片强制禁用逻辑**
  编辑 [llama_backend.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/llama_backend.rs)，将 `GLOBAL_BACKEND` 初始化的第 5-12 行全部删除，彻底去除对 `GGML_METAL_DISABLE` 和 `GGML_METAL_DEVICES` 的环境阉割，使其回归最精简的跨平台通用初始化逻辑：

  ```rust
  pub static GLOBAL_BACKEND: LazyLock<Arc<LlamaBackend>> = LazyLock::new(|| {
      let backend = LlamaBackend::init().expect("Failed to initialize llama backend");
      Arc::new(backend)
  });
  ```

---

### Task 3: 恢复大模型推理引擎的 GPU 调度 (llama_adapter.rs)

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 恢复 get_gpu_layers 原始调度函数**
  编辑 [llama_adapter.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/inference/llama_adapter.rs)，修改 `get_gpu_layers` 函数（第 58-81 行），不再拦截并强制把 Intel Mac 的 GPU layers 降为 0，而是直接在 macOS 上检测是 M 系列芯片时提供日志，其他所有情况按需返回用户配置的 layer 数量：

  ```rust
  fn get_gpu_layers(num_gpu_setting: i32) -> u32 {
      if num_gpu_setting <= 0 {
          return 0;
      }
  
      #[cfg(target_os = "macos")]
      {
          if is_m_series_mac() {
              println!("[GPU] Mac M-series detected, enabling Metal acceleration with {} layers", num_gpu_setting);
              return num_gpu_setting as u32;
          }
      }
  
      num_gpu_setting as u32
  }
  ```

- [ ] **Step 2: 移除 LlamaModel 构造中硬编码的 with_main_gpu(-1) 拦截**
  在 `llama_adapter.rs` 的模型加载逻辑（约第 170-174 行）中，移除 `if n_gpu_layers == 0 { params = params.with_main_gpu(-1); }` 强制归零拦截，将其恢复为原版的纯粹参数加载：

  ```rust
  // 恢复后：
  let model_result = tokio::task::spawn_blocking(move || {
      let params = llama_cpp_4::model::params::LlamaModelParams::default()
          .with_n_gpu_layers(n_gpu_layers);
  
      // 保护模型加载
      let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
      LlamaModel::load_from_file(&backend, &path_str, &params)
          .map(|m| Arc::new(m))
          .map_err(|e| AppError::Internal(format!("Failed to load model: {}", e)))
  });
  ```

---

### Task 4: 恢复多模态视觉引擎的 GPU 调度 (vision_adapter.rs)

**Files:**
- Modify: `src-tauri/src/services/inference/vision_adapter.rs`

- [ ] **Step 1: 恢复 vision_adapter 原始 gpu_layers 调度及 main_gpu 逻辑**
  编辑 [vision_adapter.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/inference/vision_adapter.rs)（第 59-86 行），删除在 macOS 且非 M 系列芯片下强制将显卡层数置为 0 以及在 layers = 0 时强制 `.with_main_gpu(-1)` 的逻辑，恢复为原版简洁的参数分配：

  ```rust
  // 恢复后：
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
  });
  ```

---

### Task 5: 还原词向量大模型底座与清理垃圾日志 (embedder.rs)

**Files:**
- Modify: `src-tauri/src/services/embedder.rs`

- [ ] **Step 1: 清理所有 [DIAGNOSTIC] 日志打印及 nullptr 检测逻辑**
  编辑 [embedder.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/embedder.rs)，完全删去被混入的 26 处调试 `println!`，并清除多余的 slice 空指针解包，还原纯净紧凑的词向量提取核心。

- [ ] **Step 2: 恢复 ctx_params 默认自适应调度，并还原 decode 编码接口**
  在模型上下文创建（约第 80-91 行）处，删除强制硬编码的 `with_n_batch(512)` / `with_n_threads(4)` 等，仅保留最开始设定的 `NonZeroU32::new(2048)`；并且，在批次解码迭代处（约第 120-138 行），将 `ctx.encode` 改回底座原生且高度适配的 `ctx.decode`。

  ```rust
  // 恢复后的 ctx_params 状态：
  let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
      .with_n_ctx(NonZeroU32::new(2048))
      .with_embeddings(true)
      .with_flash_attention(false);
  
  // 恢复后的 decode 逻辑：
  let decode_res = {
      let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
      ctx.decode(&mut batch)
  };
  decode_res.map_err(|e| AppError::Internal(format!("Decode error: {}", e)))?;
  ```

---

### Task 6: 清理文档分发与导入临时日志 (indexing.rs)

**Files:**
- Modify: `src-tauri/src/commands/indexing.rs`

- [ ] **Step 1: 删除 indexing 阶段所有的调试诊断行**
  编辑 [indexing.rs](file:///Users/mac/project/telepathy/src-tauri/src/commands/indexing.rs)（第 111-132 行），完全移除 `[DIAGNOSTIC INDEXING]` 临时打印，只保留最原生优雅的向量计算、数据库写入以及 HNSW 索引注入闭包：

  ```rust
  // 恢复后：
  if let Ok(embedding) = emb.embed(chunk_content).await {
      vectors::insert_embedding(&conn, &chunk.id, &embedding)?;
  
      let add_result = with_hnsw_manager(|manager| {
          manager.add_chunk(&chunk.id, &embedding, EMBEDDING_DIMENSION, doc.project_id.as_deref())
      });
      if let Err(e) = add_result.and_then(|r| r) {
          println!("[HNSW] Warning: failed to add chunk to index: {}", e);
      }
  }
  ```
