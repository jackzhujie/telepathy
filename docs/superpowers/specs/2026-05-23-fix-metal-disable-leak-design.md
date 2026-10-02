# 设计规格文档：防止 GGML_METAL_DISABLE 环境变量在向量化初始化中被清除导致闪退

- **文档状态**：已批准 (Approved)
- **创建日期**：2026-05-23
- **作者**：Antigravity

---

## 1. 背景与问题描述

在非 Apple M 系列芯片的 Mac 设备（如搭载 Intel CPU + AMD Radeon GPU 的 MacBook Pro）上运行 Telepathy 桌面端时，大语言模型与向量化模块若开启 GPU/Metal 推理，会在特定运算中因为设备驱动问题导致进程 abort 闪退。

为此，应用在 [lib.rs](file:///Users/mac/project/telepathy/src-tauri/src/lib.rs) 的 `run` 函数最开始进行了防崩溃机制处理，检测到非 M 系列 Mac 时调用系统的 `setenv` 设置全局环境变量 `GGML_METAL_DISABLE=1`。

然而，在后续测试中，导入 PDF 文件或重建索引时，应用依然闪退。根据崩溃日志，底层的 `ggml_metal_init` 仍然检测到了 AMD 独显设备，并尝试激活 Metal 导致崩溃。这表明我们设置的禁用开关在运行期间被意外清除了。

---

## 2. 根因分析

在 [embedder.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/embedder.rs) 中，为了使 Embedding 模型初始化专用于 CPU 的 backend，设计了如下的临时变量开关：

```rust
std::env::set_var("GGML_METAL_DISABLE", "1");
let res = llama_cpp_4::llama_backend::LlamaBackend::init()...
std::env::remove_var("GGML_METAL_DISABLE"); // 👈 清除变量
```

1. **全局变量泄露与清除**：
   在 `LlamaBackend::init()` 完成后，代码调用了 `std::env::remove_var("GGML_METAL_DISABLE")`。
   这一调用会直接在进程的环境变量集合中将此键**全局移除**。对于非 M 芯片的 Mac 设备而言，这意味着在程序启动时通过 `setenv` 注入的全局防崩溃保护开关被彻底抹除了。
   
2. **连锁崩溃反应**：
   当应用稍后加载大语言模型聊天推理或重新加载上下文时，由于变量已被移除，底层 `llama.cpp` 会重新触发 Metal 设备的初始化。在分配计算图 `sched_reserve: graph splits = 1` 时由于驱动层异常直接发出 Abort，引发第二次闪退。

---

## 3. 设计方案

### 3.1 导出 `is_m_series_mac` 判定函数
将 [lib.rs](file:///Users/mac/project/telepathy/src-tauri/src/lib.rs) 里的私有判断函数 `is_m_series_mac()` 导出为 `pub`，使外部服务可直接调用该安全策略判定：

```rust
#[cfg(target_os = "macos")]
pub fn is_m_series_mac() -> bool {
    // ...
}
```

### 3.2 在 `embedder.rs` 中进行环境变量的状态备份与还原
修改 [embedder.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/embedder.rs) 的 Llama 向量化后端初始化块：
- 在设置 `GGML_METAL_DISABLE` 为 `"1"` 之前，使用 `std::env::var` 备份其原有状态。
- 初始化完 backend 后，对设备进行区分：
  - 如果是 M 芯片 Mac 设备，按照备份的值进行还原（若原本有值则设置原值，否则移除该键）。
  - 如果是非 M 芯片 Mac 设备，强制维持 `GGML_METAL_DISABLE` 为 `"1"`，坚决不进行移除，确保全局强力保护不失效。

```rust
                let embed_backend = {
                    let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    
                    // 1. 备份原状态
                    let old_val = std::env::var("GGML_METAL_DISABLE").ok();
                    std::env::set_var("GGML_METAL_DISABLE", "1");
                    
                    let res = llama_cpp_4::llama_backend::LlamaBackend::init()
                        .map_err(|e| AppError::Internal(format!("Failed to initialize embed backend: {}", e)));
                    
                    // 2. 区分架构还原环境状态
                    #[cfg(target_os = "macos")]
                    let is_m = crate::is_m_series_mac();
                    #[cfg(not(target_os = "macos"))]
                    let is_m = true;

                    if is_m {
                        match old_val {
                            Some(val) => std::env::set_var("GGML_METAL_DISABLE", val),
                            None => std::env::remove_var("GGML_METAL_DISABLE"),
                        }
                    } else {
                        // 非 M 芯片 Mac 强行保持禁用状态，防止被错误清除
                        std::env::set_var("GGML_METAL_DISABLE", "1");
                    }
                    res
                }?;
```

---

## 4. 验证计划

### 4.1 自动测试验证
在 `src-tauri` 中运行现有单元测试，确保代码无编译错误，测试逻辑能完美通过：
```bash
cargo test --package temp-app --lib -- services::embedder::tests::test_real_model_embedding -- --nocapture
```

### 4.2 编译检查
在前端/后端运行全面构建/类型检测，确保没有因修改引入的 API 类型不兼容或未使用的导入警告：
```bash
cargo check
```
