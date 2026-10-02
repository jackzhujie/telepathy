# 设计规格说明书: 防止 Embedder 初始化时清除 GGML_METAL_DISABLE 环境变量

## 1. 背景与需求
在 Telepathy 应用启动时，如果检测到是非 M 系列芯片的 Mac 设备（Intel 或 Radeon 显卡设备），系统会全局设置环境变量 `GGML_METAL_DISABLE = "1"` 以禁用 Metal 加速，防止 llama.cpp 的 Metal 后端在此类设备上引发 VRAM 崩溃崩溃。

然而，在 `src-tauri/src/services/embedder.rs` 加载 CPU-only embedding 模型时，其内部逻辑在初始化 `LlamaBackend` 时会临时设置并紧接着删除 `GGML_METAL_DISABLE`：
```rust
std::env::set_var("GGML_METAL_DISABLE", "1");
let res = llama_cpp_4::llama_backend::LlamaBackend::init()
    .map_err(|e| AppError::Internal(format!("Failed to initialize embed backend: {}", e)));
std::env::remove_var("GGML_METAL_DISABLE");
```
这会导致非 M 系列 Mac 原本在启动时设置的全局防御性 `GGML_METAL_DISABLE` 环境变量被直接 `remove_var` 清除，进而导致后续推理或其它 llama.cpp 实例错误地启用 Metal 并引起崩溃。

本设计旨在修复这一环境变量泄漏（被清除）问题，实现向量化环境设置的状态恢复与防清除机制。

## 2. 方案对比
### Approach 1 (推荐): 手动备份与恢复（平台及架构识别恢复）
- **实现细节**:
  - 在设置 `GGML_METAL_DISABLE` 之前，使用 `std::env::var("GGML_METAL_DISABLE").ok()` 备份原状态。
  - 初始化完成后，根据平台和架构：
    - 如果是 macOS 并且是 M 系列芯片（或非 macOS 平台），恢复之前的备份状态（即若原先有值就 set，原先无值就 remove）。
    - 如果是 macOS 并且是非 M 系列芯片，强制保持 `GGML_METAL_DISABLE` 为 `"1"`，绝不移除。
- **优点**: 逻辑直接，能彻底保证 Intel/Radeon Mac 不受意外清除影响，且能准确还原 M 系列 Mac 和其他平台的原始配置。
- **缺点**: 需将 `lib.rs` 中的 `is_m_series_mac` 暴露为 `pub` 才能跨模块调用。

### Approach 2: 使用 RAII Drop 守护机制
- **实现细节**:
  - 创建一个结构体 `EnvGuard`，在其 `new` 时备份，在 `drop` 时自动还原。
- **优点**: 在发生 panic 时也能保证还原。
- **缺点**: 在多线程 block_on 或者 sequential 执行中，过于复杂的 RAII 会增加编译开销和代码冗余，且对特殊条件（如 Intel Mac 必须强行维持 `"1"` 而不是还原）的处理不够直观。

## 3. 设计实现
1. 将 `src-tauri/src/lib.rs` 中的 `is_m_series_mac` 变为 `pub`。
2. 编写 TDD 失败用例测试全局防崩溃环境变量泄露，确保可以复现错误。
3. 修改 `embedder.rs` 实现环境备份与还原，并强制在 Intel/Radeon Mac 下保持 `"1"`。
4. 运行单元测试验证绿灯，并进行整体编译检查。
