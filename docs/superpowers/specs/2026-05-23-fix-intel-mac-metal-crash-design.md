# 2026-05-23 Intel Mac 平台 Metal 后端完全屏蔽与闪退修复设计规范

本规范详细说明如何在 Intel Mac 上彻底屏蔽并禁用 `llama.cpp` 的 Metal 后端设备列举逻辑。

## 方案设计

### 1. llama.cpp 底层机制分析

在基于 `ggml-backend` 架构的 `llama.cpp` 运行时：
- 各后端（如 CPU、Metal、CUDA 等）作为共享 registry 注册到 `ggml-backend.cpp`。
- macOS 上，`ggml_backend_metal_reg` 会被自动编译并注册。
- 当 Llama 模型或 Context 初始化时，不管是否 offload 模型层，都会通过 backend 探测可用的物理设备。
- 对 Metal 而言，这会触发 `ggml_metal_init(dev)`。若当前环境存在双显卡或特定 Intel 芯片，其内部调用的 Metal API 分配可能导致宿主程序直接段错误奔溃。

### 2. 环境变量绕过设计

通过对 `llama.cpp` 源代码中的 `ggml-metal.cpp` 进行审查，可以确定底层在 `ggml_backend_metal_reg()` 方法中会尝试通过 `getenv` 获取并解析可用的 Metal 设备数：

```cpp
ggml_backend_reg_t ggml_backend_metal_reg(void) {
    static ggml_backend_reg reg;
    static bool initialized = false;

    {
        static std::mutex mutex;
        std::lock_guard<std::mutex> lock(mutex);

        const char * env = getenv("GGML_METAL_DEVICES");
        if (env) {
            g_devices = atoi(env);
        }
        ...
```

由此，在初始化任何 Llama 后端（即通过 `llama_backend_init`）之前，如果程序执行 `std::env::set_var("GGML_METAL_DEVICES", "0")`，则会强制：
- `g_devices` 设置为 0。
- `devices` 列表变为空。
- 在接下来的整个运行生命周期中，由于可用 Metal 设备数量为 0，底层将完全忽略并屏蔽任何 Metal 逻辑，不会执行 `ggml_metal_init()` 或任何导致崩溃的硬件探测逻辑。

### 3. 应用层代码整合

在 `src-tauri/src/services/llama_backend.rs` 中，对 `GLOBAL_BACKEND` 后端初始化时插入架构与平台判断：

```rust
pub static GLOBAL_BACKEND: LazyLock<Arc<LlamaBackend>> = LazyLock::new(|| {
    #[cfg(target_os = "macos")]
    {
        if !crate::is_m_series_mac() {
            println!("[GLOBAL_BACKEND] Intel Mac detected. Setting GGML_METAL_DISABLE=1 and GGML_METAL_DEVICES=0 to prevent crashes.");
            std::env::set_var("GGML_METAL_DISABLE", "1");
            std::env::set_var("GGML_METAL_DEVICES", "0");
        }
    }
    let backend = LlamaBackend::init().expect("Failed to initialize llama backend");
    Arc::new(backend)
});
```

该设计保证了：
- **安全性**：只有在非 M 系列芯片的 Mac 上才会强行通过环境变量将 Metal 可用设备设定为 0，这保证了 M 系列芯片 Mac 用户的 GPU 加速不受任何影响。
- **全局生效**：所有的词向量化及推理操作（Embedder、LlamaAdapter、VisionAdapter）都是共享 `GLOBAL_BACKEND` 实例，由于它是个静态单例，首次执行时进行的环境变量注入将自动对后续的所有 Llama 模型和 Context 生命周期起作用。
