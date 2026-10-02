# 统一 RAG 检索系统设计文档 (Unified RAG Search)

## 1. 背景与目标
目前 Telepathy 应用的 RAG（检索增强生成）系统仅检索从文档管理器导入的外部文件（PDF, MD, TXT 等）。用户记录在“随记”（SnapNote）中的个人知识虽然也进行了向量化处理，但在提问时并未被纳入检索范围，导致 AI 无法回答基于随记内容的问题。

本方案的目标是将“随记”集成到主 RAG 检索流中，使其与普通文档享有同等的检索优先级。

## 2. 详细设计

### 2.1 检索逻辑优化
修改 `src-tauri/src/services/rag.rs` 中的 `search_with_embedding` 函数，实现混合检索：
1. **调用文档检索**：调用 `vectors::search_similar` 获取文档片段。
2. **调用随记检索**：调用 `snaps::search_similar_snaps` 获取随记内容。
3. **映射与合并**：
   - 将 `Snap` 对象转换为 `SearchSource` 结构。
   - `document_name` 统一标记为 `【个人随记】`。
   - `chunk_index` 设置为 `0`（随记暂不分块）。
4. **排序与截断**：将所有结果合并后按相似度分数降序排列，取前 `top_k` 个。

### 2.2 数据模型变更
`UserProfile` 结构体中将保留 `long_term_memories`（用于存放对话总结），但随记将从“背景信息”移动到“核心参考资料”中。

### 2.3 提示词 (Prompt) 调整
由于随记现在作为 `sources` 传入，如果检索到了随记，AI 将进入“严格模式”：
- 强制根据参考资料回答。
- 在段落末尾标注 `[来源: 【个人随记】]`。

## 3. 影响范围
- `src-tauri/src/services/rag.rs`: 修改检索与消息构建逻辑。
- `src-tauri/src/commands/rag.rs`: 确保 `rag_query` 正确传递连接对象。

## 4. 验证计划
1. 在随记中写入“张三的律师费是500万”。
2. 在 RAG 问答界面提问“张三的律师费是多少”。
3. 预期结果：AI 能够准确回答“500万”，并标注来源为“个人随记”。
