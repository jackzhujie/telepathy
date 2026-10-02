# 2026-05-24 撤销 Metal 过度限制与清理临时诊断代码设计规范

本规范说明如何将先前因误判 `llama.cpp` 兼容性而引入的过度 Metal 禁用代码、参数限制和控制台临时诊断打印彻底恢复和清理。

## 方案设计

### 1. 清理临时诊断代码
我们将从以下文件中清除所有以 `[DIAGNOSTIC]` 或 `[DIAGNOSTIC INDEXING]` 为前缀的日志行及配套的安全检测：
- **`src-tauri/src/services/embedder.rs`**：完全清理其中的诊断打印和对 slice address 的 nullptr 判定包裹，使批量词向量生成逻辑返回原本精简高效的代码流。
- **`src-tauri/src/commands/indexing.rs`**：完全清理在文档 chunk 向量生成、数据库写入以及 HNSW 索引插入时的冗长诊断流。

### 2. 撤销对 Intel Mac GPU Metal 后端的强制屏蔽
之前误以为 `llama.cpp` 底层在探测 Intel Mac 的显卡硬件时必定引发段错误，因此做了强行降级。既然崩溃根源已确认为 HNSW AVX 指令及越界问题且已得到安全修复，我们需要将 GPU 加速的能力物归原主，允许 Intel Mac 设备按需使用 GPU：
- **`src-tauri/src/lib.rs`**：
  - 彻底删除在应用 `run` 入口通过 POSIX FFI 注入 `GGML_METAL_DISABLE=1` 的宏拦截逻辑。
  - 删除在 macOS 下声明的外部 C 函数 `setenv`。
- **`src-tauri/src/services/llama_backend.rs`**：
  - 在静态单例 `GLOBAL_BACKEND` 中，删除 `if !crate::is_m_series_mac()` 处的拦截，不再对非 M 芯片 Mac 强制注入 `GGML_METAL_DEVICES = "0"` 或 `GGML_METAL_DISABLE = "1"`。
- **`src-tauri/src/services/inference/llama_adapter.rs`** 与 **`src-tauri/src/services/inference/vision_adapter.rs`**：
  - 恢复 `get_gpu_layers` 逻辑，允许非 M 芯片 Mac 返回用户配置的实际 layers 层数。
  - 清理在 `n_gpu_layers == 0` 时硬编码的 `.with_main_gpu(-1)` 参数，使模型参数设置恢复到原版。

### 3. 恢复词向量上下文的原生参数与接口
为了完全保证向量生成的纯净性，并采用系统内部推荐的调度策略：
- **`src-tauri/src/services/embedder.rs`**：
  - 移除对 `n_batch`、`n_threads` 以及 `n_threads_batch` 的硬编码设置，允许系统自适应匹配最优性能。
  - 将 `ctx.encode` 还原为原版的 `ctx.decode` 接口。

## 验证与测试结果要求
- 编译状态：所有改动在 `src-tauri` 中运行 `cargo check` 应该无任何编译 Warn（除预存的 dead code 外）及编译 Error。
- 单元测试：在 `src-tauri` 运行 `cargo test --package temp-app --lib -- services::embedder::tests::test_real_model_embedding -- --nocapture` 必须绿灯通过。
