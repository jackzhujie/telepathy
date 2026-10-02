# Telepathy 核心推理引擎重构设计 (Phase 1)

## 1. 目标与背景 (Context & Goals)
目前 Telepathy 依赖外部的 Ollama 进程进行本地大模型推理。虽然门槛低，但存在外部进程 IPC/HTTP 开销、冷启动耗时、无法精准控制硬件资源分配等问题。
随着产品的多端（Mac、Windows、Linux）定位逐渐清晰，我们需要一个**“硬件感知的多引擎智能路由架构”**来压榨设备的极限性能。

**当前任务（Phase 1）**：完全移除 Ollama 依赖，在 Rust 侧深度内嵌 `llama.cpp` 作为原生引擎，并建立统一的多引擎抽象层（Trait），为后续引入 MLX / ExLlamaV2 铺平道路。
*前置条件*：产品尚未发布，无需考虑向下兼容和历史数据迁移，允许破坏性重构。

## 2. 核心架构设计 (Architecture)

### 2.1 统一推理接口 (Unified Inference Trait)
在 Rust 后端引入一个通用的 `InferenceEngine` Trait。未来所有不同的底层引擎（Llama.cpp, MLX, 等）都必须实现此接口。

```rust
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    /// 启动并加载模型到显存/内存
    async fn load_model(&mut self, model_path: &Path, options: ModelLoadOptions) -> Result<(), AppError>;
    
    /// 流式生成对话内容
    async fn stream_chat(&self, messages: Vec<Message>, tx: mpsc::Sender<String>) -> Result<(), AppError>;
    
    /// 获取当前引擎的硬件资源占用（如 VRAM）
    fn get_stats(&self) -> EngineStats;
    
    /// 卸载模型并释放资源
    async fn unload(&mut self) -> Result<(), AppError>;
}
```

### 2.2 原生 Llama.cpp 适配器 (LlamaCppAdapter)
- 通过 Rust FFI 绑定（如 `llama-cpp-rs` 或 `llama_cpp_client`）直接在 Tauri 进程中运行推理。
- 彻底消灭 HTTP API 序列化开销，直接在内存中操作上下文（Context）和张量。

### 2.3 硬件感知编译策略 (Hardware-Aware Compilation)
在 Cargo 的 `build.rs` 中或通过 features 配置跨平台加速库：
- **Mac (Apple Silicon)**：编译时自动链接 `Metal` 框架，使用 Unified Memory 策略。
- **Windows / Linux**：编译时优先链接 `CUDA` 驱动（针对 Nvidia 显卡）或回退至 `Vulkan/OpenBLAS`。

## 3. 核心改动范围 (Scope of Changes)

1. **废弃 Ollama 模块**：彻底删除 `src-tauri/src/services/ollama.rs` 及其相关的进程检查和 HTTP 请求代码。
2. **重构本地模型发现机制**：本地已安装模型的扫描将不再调用 `ollama list` API，而是直接扫描 Telepathy 的本地数据目录。
3. **数据库更新 (SQLite)**：重新设计 `telepathy.db` 中的已安装模型记录表，剥离 Ollama 特有的 tag 逻辑，新增 `engine_type` (默认为 `LlamaCpp`) 和 `format` (默认为 `GGUF`) 字段。
4. **对话流重构**：重写 `commands/chat.rs`。从原本的发送 HTTP 请求给 Ollama，改为调起 `InferenceEngine::stream_chat`，并将生成的 token 直接通过 Tauri Event 发送给 Vue 前端。

## 4. 数据流图 (Data Flow)

1. 用户点击发送消息 -> Vue 前端调用 Tauri Command `send_chat_message`。
2. Tauri 收到指令 -> 从 `telepathy.db` 查找所选模型路径 -> EngineManager 检查当前引擎状态。
3. 若引擎未挂载 -> 实例化 `LlamaCppAdapter` 并加载 `.gguf` 文件（直接占用 VRAM）。
4. 引擎开始推理 -> 产生 Token -> 触发 Rust `mpsc` channel -> 触发 Tauri Event `chat-token-chunk`。
5. Vue 前端监听 Event 并实时更新气泡 UI。

## 5. 错误处理与容灾 (Error Handling)
- **OOM (Out of Memory)**：在加载模型前，读取文件元数据（GGUF headers）估算所需显存，若严重超标则拒绝加载并弹窗警告，而不是让进程直接崩溃。
- **编译时降级**：如果由于用户系统缺少 CUDA 等导致高级加速库调用失败，引擎自动平滑降级为 CPU 计算（虽然慢，但不至于闪退）。
