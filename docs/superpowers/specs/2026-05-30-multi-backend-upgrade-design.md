# 跨平台 llama.cpp 多后端支持升级设计

## 1. 背景与目标

当前 Telepathy 应用使用 `llama-cpp-4` 版本 `0.2`，仅支持 Metal 后端。为了支持跨平台（macOS、Windows、Linux），需要升级到 `0.3` 并启用多后端支持。

## 2. 升级方案

### 2.1 升级 llama-cpp-4

**文件**：`src-tauri/Cargo.toml`

```toml
# 从：
llama-cpp-4 = { version = "0.2", features = ["mtmd", "metal"] }

# 改为：
llama-cpp-4 = { version = "0.3", features = ["mtmd", "metal", "vulkan", "openmp"] }
```

### 2.2 启用的后端

| 后端 | 目标平台 | 说明 |
|------|---------|------|
| `metal` | macOS Apple Silicon | Apple GPU 加速 |
| `vulkan` | Windows/Linux GPU | 跨平台 GPU 加速（NVIDIA/AMD/Intel） |
| `openmp` | 全平台 CPU | CPU 多线程加速 |

### 2.3 运行时自动选择

启用多个后端后，llama.cpp 会在运行时自动检测硬件并选择最佳后端：

```
用户电脑 → 自动检测 → Metal / Vulkan / OpenMP
```

## 3. 代码改动

### 3.1 删除 macOS 兼容性保护逻辑

移除之前为防止 Intel Mac Metal 崩溃而添加的临时保护代码：

**涉及文件**：
- `src-tauri/src/services/embedder.rs`：删除 `GGML_METAL_DISABLE` 相关代码
- `src-tauri/src/lib.rs`：删除 `is_m_series_mac()` 函数
- `src-tauri/src/services/inference/llama_adapter.rs`：删除 macOS GPU 层判断逻辑
- `src-tauri/src/services/inference/vision_adapter.rs`：删除 macOS GPU 层判断逻辑

### 3.2 简化后端选择

让 llama.cpp 完全自主选择后端，不再干预 GPU 层设置。

## 4. 测试计划

- [ ] macOS Apple Silicon：验证 Metal 后端正常工作
- [ ] macOS Intel：验证回退到 CPU/OpenMP
- [ ] Windows + NVIDIA：验证 Vulkan 后端
- [ ] Linux + NVIDIA：验证 Vulkan 后端
- [ ] Linux + AMD：验证 Vulkan 后端
- [ ] 无 GPU 环境：验证 OpenMP CPU 加速

## 5. TODO

- [ ] **CUDA 动态下载功能**：实现用户可选的 CUDA 加速版本下载（后续计划）

## 6. 风险与回滚

- **风险**：Intel Mac 可能仍有 Metal 崩溃问题
- **回滚方案**：如遇崩溃，恢复 `is_m_series_mac()` 逻辑并添加环境变量控制
