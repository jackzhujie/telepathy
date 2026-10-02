# P4: RAG 检索与问答 - 设计文档

## 1. 项目背景 (Project Context)
Telepathy 的 P0-P3 阶段已完成文档导入、解析、分块和向量化。P4 将实现核心功能：用户提问 → 向量检索 → LLM 生成带引用的回答。这是 Telepathy 从"知识存储"到"知识问答"的关键跃迁。

## 2. 核心目标 (Core Goals)
- **Top-K + 语义重排**: 向量检索后用相似度重排，选出最优的 K 个上下文块。
- **多轮对话**: 维护对话历史，支持追问和上下文连贯的对话。
- **引用来源**: 回答中标注引用的文档名和段落索引，用户可追溯验证。
- **通用知识兜底**: 知识库无匹配时，调用 LLM 自身知识回答并标注来源。
- **流式响应**: 复用 P1 的流式打字机效果，实时展示生成的回答。

## 3. 架构设计 (Architecture)

### 3.1 RAG 管道 (RAG Pipeline)
```
用户提问
    ↓
[向量检索] → 对问题生成 embedding → cosine similarity 搜索 → Top-K 候选
    ↓
[语义重排] → 按相似度分数排序 → 过滤低分结果（阈值 < 0.3）
    ↓
[上下文组装] → 将检索到的块 + 来源信息组装为 prompt 上下文
    ↓
[LLM 生成] → 调用 Ollama /api/chat（流式）→ 生成回答
    ↓
[引用标注] → 在回答中标注引用来源（文档名、块索引）
    ↓
[流式推送] → 通过 Tauri Events 推送 token → 前端实时显示
```

### 3.2 核心组件

#### 3.2.1 RAG 引擎 (`services/rag.rs`)
- `search_context(query: &str, conn: &Connection, embedder: &Embedder, top_k: usize) -> Result<Vec<SearchResult>>`
- 对 query 生成 embedding，调用 `vectors::search_similar`
- 过滤相似度 < 0.3 的结果
- 返回带来源信息的搜索结果

#### 3.2.2 Prompt 组装器 (`services/prompt.rs`)
- `build_rag_prompt(query: &str, context: &[SearchResult], history: &[ChatMessage]) -> String`
- 将检索到的块组装为带来源标记的上下文
- 包含对话历史（最近 N 轮）
- 指导 LLM 基于上下文回答并引用来源

#### 3.2.3 对话管理 (`db/conversations.rs`)
- `conversations` 表: id, title, created_at
- `messages` 表: id, conversation_id, role, content, sources(JSON), created_at
- 支持创建/查询/删除对话，支持查询对话历史

### 3.3 数据流
```
前端 Chat.vue
  → invoke('rag_query', { query, conversation_id })
  → Rust 后端:
    1. 读取对话历史
    2. 生成 query embedding
    3. 向量相似度搜索
    4. 语义重排 + 过滤
    5. 组装 prompt
    6. 流式调用 Ollama
    7. emit('chat-token') + emit('rag-sources')
    8. 保存消息到数据库
  → 前端实时更新 UI
```

### 3.4 Prompt 模板
```
你是一个智能知识库助手。请基于以下检索到的文档内容回答用户问题。

## 检索到的文档内容

[来源: document_name.txt, 块 #2]
这里是检索到的文本内容...

[来源: document_name.md, 块 #5]
这里是另一段检索到的文本内容...

## 回答要求
1. 优先使用检索到的文档内容回答问题
2. 在相关段落末尾标注引用来源，格式：[来源: 文件名, 块 #N]
3. 如果检索到的内容无法完全回答问题，可以结合你的通用知识补充，但需标注[通用知识]
4. 如果完全没有检索到相关内容，直接用你的知识回答，标注[通用知识]
```

## 4. 接口设计 (API Design)

### 4.1 Tauri Commands
- `rag_query(query: String, conversation_id: Option<String>) -> Result<RagResponse, AppError>`
  - 流式推送: `chat-token`, `chat-done`, `chat-error`, `rag-sources`
- `get_conversations() -> Result<Vec<Conversation>, AppError>`
- `get_messages(conversation_id: String) -> Result<Vec<ChatMessage>, AppError>`
- `create_conversation(title: Option<String>) -> Result<Conversation, AppError>`
- `delete_conversation(conversation_id: String) -> Result<(), AppError>`

### 4.2 前端组件
- 复用 Chat.vue 页面，增加对话列表侧栏
- 新增: 对话历史列表（左侧栏）
- 新增: 引用来源展示（回答下方可展开）
- 流式响应: 复用 P1 的事件监听机制

## 5. 错误处理 (Error Handling)
- **模型未下载**: 检测嵌入模型可用性，提示用户下载
- **无索引文档**: 提示用户先导入并索引文档
- **Ollama 未运行**: 提示用户启动 Ollama 服务
- **向量维度不匹配**: 自动重新索引受影响的文档

## 6. 验收标准 (Success Criteria)
- [ ] 用户提问能从知识库中检索到相关文档块
- [ ] 回答中标注了引用来源（文档名、块索引）
- [ ] 支持多轮对话，追问上下文连贯
- [ ] 知识库无匹配时用通用知识回答并标注
- [ ] 流式响应实时显示
- [ ] 对话历史持久化存储
