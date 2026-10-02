# 设计文档：迁移至 ollama-rs 库进行模型操作

## 1. 背景与目标
当前 Telepathy 项目采用手动构造 HTTP 请求（使用 `reqwest`）的方式与 Ollama 后端通信。这种方式维护成本高、错误处理复杂，且在模型拉取、聊天流处理等方面存在重复实现（`models.rs` 与 `installer.rs`）。

本项目标是使用 `ollama-rs` 库全面替代现有的手动 API 交互，提高代码质量和稳定性。

## 2. 核心架构变更

### 2.1 新增 `services/ollama.rs`
统一管理 Ollama 客户端。
```rust
pub struct OllamaService;
impl OllamaService {
    // 获取异步客户端实例，从设置中读取 URL
    pub async fn get_client(app_handle: &AppHandle) -> Result<Ollama, AppError>;
}
```

### 2.2 重构向量化模块 (`services/embedder.rs`)
- 移除 `reqwest` 和手动 JSON 构造。
- 使用 `ollama.generate_embeddings()` 替换现有的 `embed` 方法。
- 支持批量向量化优化。

### 2.3 重构对话模块 (`commands/rag.rs`)
- 使用 `ChatMessage` 结构体管理对话历史。
- 使用 `send_chat_messages_stream` 简化流式输出。
- 统一参数注入（Temperature, Context Size, Num Threads）。

## 3. 统一模型管理 (Unified Model Management)

### 3.1 废弃冗余代码
- 移除 `commands/installer.rs` 中使用 `Command::new("ollama pull")` 的 `pull_model` 逻辑。
- 移除 `commands/models.rs` 中手动调用 `/api/pull` 的 `install_model` 逻辑。

### 3.2 统一拉取命令 (`commands/models.rs`)
- 单一入口：`pull_model(model_name: String)`。
- 实现：使用 `ollama.pull_model_stream()`。
- 事件：输出统一格式的进度更新事件 `model-pull-progress`。

## 4. 迁移路线图

1.  **准备期**：
    - `Cargo.toml` 添加 `ollama-rs` 依赖。
    - 创建 `services/ollama.rs` 基础设施。
2.  **实施期**：
    - 阶段 A：重构 Embedder（最稳定）。
    - 阶段 B：重构模型列表与删除命令。
    - 阶段 C：重构模型拉取（统一 installer 逻辑）。
    - 阶段 D：重构 RAG Chat（最复杂，需小心处理取消令牌）。
3.  **验证期**：
    - 运行 AI 对话验证流式效果。
    - 验证断点续传/重新拉取逻辑。
    - 检查日志中原有的 `[PERF]` 监控是否依然有效。

## 5. 成功指标
- `src-tauri` 中不再出现手动构造的 Ollama API URL 字符串。
- `reqwest` 依赖仅用于 GitHub Release 下载等非 Ollama 操作。
- 移除的代码行数（手动解析逻辑）应大于新增的代码行数。
