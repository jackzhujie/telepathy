# 修复 PDF 视觉解析时多线程并发与频繁析构导致的闪退设计文档

## 1. 问题背景与目的
在 Telepathy 应用中，当用户上传 PDF 文件并触发后台向量索引时，如果启用了多模态视觉解析，应用很容易发生闪退。

### 闪退根因：
1. **多页 PDF 解析的高频析构**：旧设计中，多页 PDF 解析采用单页循环，每页都会调用 `VisionAdapter::stream_vision_chat`。而在该方法内，每一页都会重新加载 `MtmdContext` (mmproj) 并 `new_context`，解析结束后自动 drop 并执行 `ggml_metal_free: deallocating`。频繁的显存分配与释放（尤其是 Metal 驱动上）极易引发 Double Free 或状态冲突，从而导致进程崩溃闪退。
2. **多线程并发**：在消息回复（大模型推理）与文件索引（Embedding/视觉 PDF 解析）同时进行时，或者批量导入多个 PDF 时，多个线程会并发在同一个 `LlamaBackend` 上操作不同的 Context 并向 Metal 驱动提交计算指令，导致 Metal 后端内部状态产生竞态冲突。

### 目的：
通过引入**全局同步锁**保障 `llama.cpp` 底层操作的线程安全，并重构视觉解析流程以**复用上下文 (Context)**，降低显存分配频率，彻底解决闪退问题并显著提升多页 PDF 解析速度。

---

## 2. 详细设计与变更方案

### A. 全局同步锁机制 (`GLOBAL_BACKEND`)
由于所有的推理和向量提取均基于单例 `GLOBAL_BACKEND`，我们将在 `llama_backend` 模块中导出一个全局同步互斥锁：
```rust
use std::sync::Mutex as StdMutex;
pub static ACQUIRE_LOCK: StdMutex<()> = StdMutex::new(());
```
在任何涉及 `llama.cpp` 的敏感操作（如 `LlamaModel::load_from_file`、`LlamaModel::new_context`、`LlamaContext::decode` 以及 Context 析构 `drop`）中均需获取此锁，以达到计算流串行化排队的目的。

### B. 视觉上下文复用 (`VisionAdapter`)
在 `VisionAdapter` 中新增 `stream_vision_multi_page` 方法，使其在解析多页 PDF 时：
1. 在进入页面循环前，仅初始化一次 `MtmdContext` 和 `LlamaContext`。
2. 在循环中依次解析每一页，在每页解析前调用 `lctx.clear_kv_cache()` 清理上下文。
3. 循环结束后，统一进行 Context 的销毁。

---

## 3. 拟修改的文件及内容

### 1. `src-tauri/src/services/llama_backend.rs`
- 新增并导出全局同步锁 `ACQUIRE_LOCK`：
  ```rust
  pub static ACQUIRE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
  ```

### 2. `src-tauri/src/services/inference/llama_adapter.rs`
- 在 `load_model` 中，使用 `ACQUIRE_LOCK` 保护 `LlamaModel::load_from_file`。
- 在 `stream_chat` 中：
  - 使用 `ACQUIRE_LOCK` 保护 `model.new_context`。
  - 在 decode chunk 的循环 and 生成的 `while` 循环中，每次调用 `ctx.decode` 前获取 `ACQUIRE_LOCK`（获取后执行 decode 并立即释放锁，允许其他线程穿插计算）。
- 在 `unload` 或修改 `active_context` 导致 `ActiveContext` 析构时，在锁的保护下将其设为 `None`，防止析构发生冲突。

### 3. `src-tauri/src/services/embedder.rs`
- 在 `get_or_load_model` 中加载模型时加锁。
- 在 `embed_batch` 中：
  - 使用 `ACQUIRE_LOCK` 保护 `model.new_context`。
  - 保护 `ctx.decode` 的调用。
  - 在函数结束前，显式调用 `drop(ctx)` 并用锁进行保护：
    ```rust
    {
        let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
        drop(ctx);
    }
    ```

### 4. `src-tauri/src/services/inference/vision_adapter.rs`
- **新增** `stream_vision_multi_page` 方法，入参为 `pages_images_bytes: Vec<Vec<u8>>`，在一次 `spawn_blocking` 中仅初始化一次 `MtmdContext` 和 `LlamaContext`（需在全局锁保护下初始化与销毁），循环各页进行文本提取，每次 decode 加锁保护，每页开始前清除 KV 缓存。
- 原有的 `stream_vision_chat` 中涉及 Context 创建、销毁与 decode 的部分均加入全局锁保护。

### 5. `src-tauri/src/services/parser/vision.rs`
- 修改 `parse_pdf_with_vision` 函数：
  - 移除对单页 `page_images` 的 `for` 循环。
  - 一次性读取并压缩所有页面的图片，组装成 `Vec<Vec<u8>>`。
  - 调用新增的 `vision_guard.stream_vision_multi_page` 一次性完成所有页面的解析，收集并拼接返回结果。

---

## 4. 验证计划

### 自动化验证
- 运行 `cargo check` 确保代码能够顺利编译，警告信息被合理处理。
- 编写/运行针对 `Embedder` 向量提取以及 `LlamaCppAdapter` / `VisionAdapter` 的单元测试，确保加锁后逻辑依然正确且无死锁风险。

### 手动验证
- 索引包含多页的 PDF 文件，确认在终端日志中没有频繁的 `ggml_metal_free: deallocating` 过程，观察解析时间是否明显缩短。
- 在大模型聊天对话（生成回复）期间，手动上传 PDF 文件并执行向量索引，或批量上传多个 PDF 进行索引，验证应用不再发生任何闪退或死锁，能安全平滑地交替执行。
