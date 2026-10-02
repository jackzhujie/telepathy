# Telepathy P1: Ollama Integration Design

## 1. 概述

Telepathy 的 P1 阶段目标是打通桌面应用与本地运行的 Ollama 服务之间的通信，使得用户可以在“AI 对话”页面与本地大语言模型（如 `qwen2.5`）进行流式对话。

为了快速验证核心链路，P1 将采用硬编码配置和最小化可用界面，并在后续阶段（P5）再添加高级配置和 Markdown 渲染。

## 2. 架构决策

- **Ollama API 终结点**: 默认硬编码为 `http://localhost:11434/api/chat`
- **默认模型**: 默认硬编码为 `qwen2.5`（假设用户本地已通过 `ollama run qwen2.5` 安装）
- **通信模式**: 流式响应（Streaming）。
- **Tauri 通信机制**: 由于 Tauri 的 `invoke` 命令是单次请求/响应模式，流式数据将通过 Tauri 的事件系统（`app_handle.emit`）从后端向前端广播，实现打字机效果。

## 3. Rust 后端设计

### 3.1 依赖项
在 `src-tauri/Cargo.toml` 中增加以下依赖：
- `reqwest`（带 `json`, `stream` 功能）：用于向 Ollama 发送 HTTP 请求。
- `futures-util`：用于处理异步流。
- `serde_json`：处理 JSON 的解析。

### 3.2 目录与模块
- **`services/ollama.rs`**: 负责处理实际的 HTTP 请求，开启 `"stream": true`，读取 Ollama 的流，并向前端 emit 对应的 token。
- **`commands/chat.rs`**: 暴露给前端调用的 Tauri command `ask_ollama`。接收用户的 `prompt`，然后异步调用 `services::ollama` 的方法。

### 3.3 事件定义
- `chat-token`: 当接收到新的文字片段时发送。Payload 包含模型回复的文本。
- `chat-done`: 整个回答完成时发送。
- `chat-error`: 请求失败或中断时发送。Payload 包含错误信息。

## 4. Vue 前端设计

### 4.1 聊天气泡布局
在 `src/views/Chat.vue` 实现：
- **聊天历史**: 一个响应式的数组 `messages: { role: 'user' | 'assistant', content: string }[]`。
- **UI 组件**:
  - 使用 `n-scrollbar` 容纳对话流。
  - 使用简单的 CSS 气泡区分用户（靠右，蓝色）和助手（靠左，灰色）。
  - 底部使用 `n-input` 和 `n-button` 构成发送框。

### 4.2 状态和事件监听
- 发送消息时：将用户消息 push 到数组，同时清空输入框并压入一条内容为空的 assistant 消息占位。
- 通过 `@tauri-apps/api/event` 的 `listen` 方法监听：
  - `chat-token`: 将 payload 里的字符串追加到最后一条 assistant 消息的 `content` 中。
  - `chat-done`: 解除发送按钮的禁用状态（Loading 态）。
  - `chat-error`: 停止 Loading 态，并使用 Naive UI 的 `useMessage()` 提示错误。

## 5. P1 验收标准
1. 在输入框输入文字并发送，界面能立即渲染用户的气泡，同时输入框禁用，显示加载状态。
2. 稍等片刻后，机器人的回复以“打字机”效果（流式）逐字呈现在屏幕上。
3. 对话结束后，输入框恢复可用状态，用户可继续发送下一条消息。
4. （前提条件：本地终端确保已通过 `ollama run qwen2.5` 启动了模型）。

## 6. 不在 P1 范围内的事项
- 动态切换模型、配置 Ollama 地址。
- 多会话管理、历史记录存储到数据库（SQLite）。
- Markdown 代码高亮和公式渲染（目前以纯文本显示，利用 CSS 的 `white-space: pre-wrap` 保持换行）。
