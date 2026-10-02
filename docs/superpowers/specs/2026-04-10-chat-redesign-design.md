# Chat 页面组件化重构设计

## 概述

将 Chat 页面从单文件组件重构为模块化组件架构，同时丰富交互功能：消息操作（复制、编辑、重新生成）、加载状态优化、停止生成、代码块增强等。前后端同时改动。

## 设计目标

1. **组件化** — 将 Chat.vue 拆分为职责清晰的子组件
2. **状态集中管理** — 创建 chat store 管理所有聊天状态
3. **丰富交互** — 复制消息、编辑重发、重新生成、停止生成
4. **加载体验** — 思考指示器、流式光标、错误内嵌
5. **性能优化** — 虚拟滚动支持长对话
6. **代码块增强** — 一键复制、语言标签

## 组件架构

```
Chat.vue (页面容器 - 精简为布局 + 组合子组件)
├── ConversationSidebar.vue    (对话列表侧边栏)
├── MessageList.vue            (消息列表 + 虚拟滚动)
│   └── MessageItem.vue        (单条消息)
│       ├── MessageActions.vue (消息操作按钮组 - DropdownMenu)
│       └── CodeBlock.vue      (代码块 + 复制按钮)
├── ChatInput.vue              (输入框 + 发送/停止按钮)
├── SourcesPanel.vue           (RAG 来源面板)
└── ThinkingIndicator.vue      (AI 思考动画)
```

## Chat Store (`src/stores/chat.ts`)

### State

```typescript
interface ChatState {
  messages: Message[]
  conversations: Conversation[]
  currentConversationId: string | undefined
  isGenerating: boolean
  isThinking: boolean
  currentSources: SearchSource[]
  selectedProjectId: string | null
  error: string | null
}
```

### Actions

| Action | 说明 |
|--------|------|
| `sendMessage(prompt)` | 发送消息，触发 RAG 查询 |
| `stopGeneration()` | 停止当前生成 |
| `regenerateMessage(index)` | 重新生成指定 AI 消息 |
| `editAndResend(index, newContent)` | 编辑用户消息并重新发送 |
| `copyMessage(index)` | 复制消息渲染文本 |
| `copyRawMessage(index)` | 复制原始 Markdown |
| `loadConversation(id)` | 加载历史对话 |
| `newConversation()` | 新建对话 |
| `deleteConversation(id)` | 删除对话 |
| `fetchConversations()` | 刷新对话列表 |

## 消息数据结构扩展

```typescript
interface Message {
  id: string                    // 唯一 ID（nanoid 生成）
  role: 'user' | 'assistant'
  content: string
  sources?: SearchSource[]
  status: 'sending' | 'streaming' | 'done' | 'error'
  error?: string
  timestamp: number
  isEditing?: boolean           // 用户消息编辑态
}
```

## 消息操作

使用 reka-ui `DropdownMenu` 实现 hover 显示的操作菜单。

### 用户消息操作

| 操作 | 说明 |
|------|------|
| 编辑并重新发送 | 消息变为 textarea，修改后发送 |
| 复制 | 复制消息纯文本 |

### AI 消息操作

| 操作 | 说明 |
|------|------|
| 重新生成 | 清空该消息，重新调用 RAG |
| 复制 | 复制渲染后的文本 |
| 复制原始文本 | 复制 Markdown 源码 |

## 加载状态

| 状态 | 视觉表现 |
|------|----------|
| 思考中 (`isThinking`) | ThinkingIndicator：三点跳动动画 |
| 流式生成 (`streaming`) | 消息末尾闪烁光标 `▊` |
| 生成完成 (`done`) | 正常显示 |
| 生成错误 (`error`) | 内嵌错误提示 + 重试按钮 |
| 已停止 | 保留已生成内容 + "已停止" 标记 |

## 停止生成

### 后端

- Rust 端使用 `tokio_util::sync::CancellationToken`
- `stop_generation` command 触发 cancellation
- Ollama streaming loop 中检查 `is_cancelled()`
- 添加 `tokio-util` 依赖到 `Cargo.toml`

### 前端

- 生成中，ChatInput 的发送按钮变为停止按钮（⏹ 图标）
- 点击停止调用 `stopGeneration()`
- 停止后当前 AI 消息 status 变为 `'done'`

## 重新生成

### 后端

- `regenerate_message` command：接收 `conversation_id` + `message_index`
- 删除该 AI 消息及之后的所有消息（数据库）
- 用相同的对话上下文重新调用 Ollama streaming

### 前端

- 点击"重新生成" → store 中删除该消息及之后的消息
- 添加空的 assistant 消息，进入 streaming 状态
- 调用 `regenerate_message` API

## 编辑并重新发送

### 前端

- 用户消息点击"编辑" → `isEditing = true`，消息区域变为 textarea
- 修改后按 Enter 发送（Shift+Enter 换行），Escape 取消
- 发送：删除该消息及之后的所有消息，用新内容调用 `rag_query`

### 后端

- 复用现有 `rag_query`，前端在调用前先清理后续消息
- 后端也需要删除数据库中该消息之后的消息

## 代码块增强

`CodeBlock.vue` 组件：

- 右上角显示语言标签（如 `TypeScript`、`Python`）
- 一键复制按钮，使用 reka-ui `Tooltip` 显示提示
- 复制成功后按钮变为 ✓（2 秒后恢复）
- 从 markdown-it 渲染结果中提取代码块，替换为 Vue 组件

实现方式：markdown-it 自定义 fence renderer，输出带标记的 HTML，Vue 端用 `v-html` 渲染后通过 `onMounted` 查找并替换代码块 DOM。

## 虚拟滚动

使用 `@tanstack/vue-virtual`：

- 消息数量 > 50 时启用虚拟滚动
- `useVirtualizer` 配置 `estimateSize: () => 120`
- 流式生成期间禁用虚拟滚动（保持所有消息可见和自动滚动）
- 新消息到达时平滑滚动到底部

## 错误处理

- 移除所有 `alert()` 调用
- AI 消息 `status: 'error'` 时内嵌错误提示 + 重试按钮
- 使用 reka-ui `Toast` 显示全局错误通知（如网络错误）

## 各组件职责

### Chat.vue（页面容器）

- 布局：侧边栏 + 主内容区
- 组合子组件
- 处理路由参数（如从知识库跳转带 query）

### ConversationSidebar.vue

- 对话列表渲染
- 新建/选择/删除对话
- 折叠/展开

### MessageList.vue

- 虚拟滚动容器
- 消息列表渲染
- 自动滚动逻辑
- 空状态欢迎语

### MessageItem.vue

- 单条消息渲染
- 用户消息：纯文本 / 编辑态 textarea
- AI 消息：Markdown 渲染
- hover 显示操作栏
- 来源折叠/展开

### MessageActions.vue

- reka-ui DropdownMenu
- 根据消息 role 显示不同操作项
- 复制成功反馈

### CodeBlock.vue

- 代码块渲染
- 语言标签
- 复制按钮 + Tooltip

### ChatInput.vue

- textarea 输入框（自动高度）
- 发送按钮 / 停止按钮切换
- Enter 发送，Shift+Enter 换行
- 生成中禁用输入

### SourcesPanel.vue

- RAG 来源列表
- 文档名、相似度、内容预览
- 折叠/展开

### ThinkingIndicator.vue

- 三点跳动动画
- "正在思考..." 文字

## 涉及文件

| 操作 | 文件 | 说明 |
|------|------|------|
| 创建 | `src/stores/chat.ts` | Chat store |
| 创建 | `src/components/chat/ConversationSidebar.vue` | 对话侧边栏 |
| 创建 | `src/components/chat/MessageList.vue` | 消息列表 |
| 创建 | `src/components/chat/MessageItem.vue` | 单条消息 |
| 创建 | `src/components/chat/MessageActions.vue` | 消息操作菜单 |
| 创建 | `src/components/chat/CodeBlock.vue` | 代码块 |
| 创建 | `src/components/chat/ChatInput.vue` | 输入框 |
| 创建 | `src/components/chat/SourcesPanel.vue` | 来源面板 |
| 创建 | `src/components/chat/ThinkingIndicator.vue` | 思考动画 |
| 修改 | `src/views/Chat.vue` | 重构为容器 |
| 修改 | `src/types/chat.ts` | 扩展 Message 类型 |
| 修改 | `src/api/tauri.ts` | 添加新 API |
| 修改 | `src/utils/markdown.ts` | 代码块自定义渲染 |
| 修改 | `src-tauri/src/commands/rag.rs` | 停止生成、重新生成 |
| 修改 | `src-tauri/Cargo.toml` | 添加 tokio-util |
| 安装 | `@tanstack/vue-virtual` | 虚拟滚动 |

## 不变的部分

- RAG 检索逻辑不变
- Ollama API 调用方式不变（仍用 streaming NDJSON）
- 对话持久化（SQLite）不变
- 用户画像注入不变
- 现有 reka-ui Select 组件（项目选择）不变
