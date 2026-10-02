# 长期记忆 (Long-term Memory) 设计文档

## 概述
实现一个“长期记忆”功能，通过提炼用户与助理的对话内容，提取用户的长期有效信息（如技术栈、项背景、偏好等），并利用向量化技术在后续提问中自动匹配并补充到对话上下文中。

## 目标
1. **自动提炼**：通过特定触发机制（明确指令、轮数阈值、手动保存）提取用户特征。
2. **向量检索**：基于用户当前提问，实时从记忆库中检索 Top 3 相关记忆条目。
3. **上下文增强**：将检索结果无感注入系统 Prompt，提升 AI 回复的个性化和专业度。

## 架构设计

### 1. 数据存储 (Database)
在 SQLite 中新增两张表，用于存储记忆文本及其对应的向量。

**表：`memories`**
| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| id | TEXT | 主键 (UUID) |
| content | TEXT | 提炼出的记忆摘要文本 |
| created_at | TEXT | 创建时间 (UTC) |

**表：`memory_embeddings`**
| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| memory_id | TEXT | 外键，关联 memories.id |
| embedding | BLOB | 向量数据 (f32 数组序列化) |
| dimension | INTEGER | 向量维度 |

### 2. 触发机制 (Triggering)

在 `rag_query` 处理流程中，通过以下三种方式触发记忆提炼动作：

1.  **显式指令检测 (Explicit Intent)**：
    *   检查 `query` 是否包含 `记住`、`帮我记`、`记录` 等行为动词。
2.  **轮次自动提炼 (Round Detection)**：
    *   当该会话的历史对话数达到 5 轮（或 5 的倍数）时，自动对整段历史进行回顾提炼。
3.  **生命周期事件 (Life-cycle Events)**：
    *   提供暴露给前端的命令 `manual_memory_extraction`，在用户点击“保存记忆”或关闭特定会话时显式触发。

### 3. 处理流程 (Processing Pipeline)

#### A. 检索阶段 (Retrieval)
1.  用户提交问题，后台生成 `query_embedding`。
2.  **并行检索**：
    *   从 `chunks` 表检索文档片段。
    *   从 `memory_embeddings` 表通过余弦相似度检索 Top 3 记忆条目。
3.  **注入**：将检索到的记忆内容作为“用户长期偏好”注入到 `UserProfile` 并最终进入 System Prompt。

#### B. 提炼阶段 (Extraction) - 异步非阻塞执行
1.  对话流结束（`chat-done`）后，判断是否满足触发条件。
2.  若满足，后台启动异步 Task：
    *   **Prompt 构建**：使用用户提供的话术模版。
    *   **LLM 提炼**：请求 Ollama 生成记忆摘要。
    *   **语义去重**：对比库中相似度极高（> 0.95）的旧记忆。若相似，则不入库或更新时间；若新，则向量化入库。

## 接口设计 (Backend/Frontend)

*   `src-tauri/src/db/memories.rs`: 负责 SQL 操作。
*   `src-tauri/src/services/memory_service.rs`: 负责提炼流程与去重策略。
*   `src-tauri/src/commands/memory.rs`: 导出 `get_memories` 等管理命令。

## 验证计划
1.  **链路闭环**：告诉 AI “我主攻 Rust 开发”，下一轮问“我擅长什么”应能准确答出。
2.  **自动触发**：通过连续对话，观察日志确认在第 5 轮是否产生了提炼任务。
3.  **资源开销**：确认后台提炼任务不会导致前端打字卡顿或 UI 闪烁。
