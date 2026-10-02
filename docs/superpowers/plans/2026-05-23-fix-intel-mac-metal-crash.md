# 2026-05-23 Intel Mac 平台 Metal 后端完全屏蔽与闪退修复计划

本计划包含彻底解决 Intel Mac 用户在使用 Telepathy 应用进行文档解析（词向量化）或模型加载时，因 `llama.cpp` 初始化 Metal 驱动产生的闪退（段错误）问题。

## 根因定位

1. **背景**：之前为了让 Intel Mac 能够稳定运行在 CPU 上，修改了 `embedder.rs` 并强制在 GPU layers 为 0 时设置 `main_gpu = -1`。
2. **瓶颈**：尽管有此设置，在 `llama.cpp` (v0.2.43 对应的 `llama-cpp-sys-4`) 中，只要加载模型或创建 context，即使强制跑在 CPU 上，后台依然会去注册并列举编译进来的 Metal backend。此行为会在 context 初始化时自动探测显卡，并输出 `ggml_metal_init: allocating`。
3. **问题**：在部分 Intel UHD / AMD Radeon 显卡的 Intel Mac 上，显卡探测与驱动分配会由于驱动兼容性问题触发段错误直接导致整个 Rust 程序闪退（直接退出到 shell，无异常捕获）。
4. **关键发现**：
   - 经对底层的 `llama.cpp` 源码审计，发现**当前版本已不包含 `GGML_METAL_DISABLE`** 环境变量。因此之前注入的 `std::env::set_var("GGML_METAL_DISABLE", "1")` 并未生效。
   - `ggml-metal.cpp` 中提供了通过 `GGML_METAL_DEVICES` 控制可用 Metal 设备数量的逻辑：
     ```cpp
     const char * env = getenv("GGML_METAL_DEVICES");
     if (env) {
         g_devices = atoi(env);
     }
     ```
     如果将 `GGML_METAL_DEVICES` 设置为 `"0"`，则 `g_devices` 变为 `0`，底层在注册 Metal 时就会得到设备数为 0，完全不执行任何设备初始化和分配，直接绕过崩溃点。

## 详细改动

1. **修改 `/Users/mac/project/telepathy/src-tauri/src/services/llama_backend.rs`**：
   - 在 `GLOBAL_BACKEND` 的 `LazyLock` 中，当检测到非 M 系列芯片的 Mac 时，注入环境变量：
     ```rust
     std::env::set_var("GGML_METAL_DEVICES", "0");
     ```
   - 保证在底层的 `LlamaBackend::init()`（即触发底层 C 接口 `llama_backend_init`）前设置好。

## 验证方案

1. 编译并运行单元测试 `services::embedder::tests::test_real_model_embedding`。
2. 观察控制台日志，确认没有输出 `ggml_metal_init: allocating` 及其后面的 Metal 硬件查找日志，且测试通过。
