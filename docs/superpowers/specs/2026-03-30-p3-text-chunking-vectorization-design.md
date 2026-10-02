# P3: 文本分块与向量化 - 设计文档

## 1. 项目背景 (Project Context)
Telepathy 的 P2 阶段已完成文档导入和文本提取。P3 的目标是将提取的文本进行智能分块，并使用本地嵌入模型生成向量索引，为后续的 RAG 检索问答（P4）提供高质量的语义检索能力。

## 2. 核心目标 (Core Goals)
- **语义分块**: 基于自然段落边界切分文本，确保每个块在语义上完整。
- **本地嵌入**: 使用专用本地模型（如 `bge-large-zh`）通过 Ollama 生成高质量中文向量。
- **轻量级向量存储**: 利用 SQLite + `sqlite-vss` 扩展存储向量索引，无需额外服务。
- **段落级粒度**: 以自然段落为单位进行向量化，平衡上下文完整性和检索精度。

## 3. 架构设计 (Architecture)

### 3.1 文本分块引擎 (Chunking Engine)
- **语义分块策略**:
  - 识别文档的自然段落边界（双换行、标题标记等）。
  - 对超长段落进行二次切分（按固定 Token 上限）。
  - 对过短段落进行合并（确保最小上下文长度）。
- **分块参数**:
  - 最小块长度: 50 字符
  - 最大块长度: 2000 字符（可配置）
  - 段落合并阈值: 相邻短段落若总长度 < 500 字符则合并

### 3.2 向量化引擎 (Vectorization Engine)
- **嵌入模型**: 通过 Ollama `/api/embeddings` 接口调用专用嵌入模型。
- **模型推荐**: `bge-large-zh`（中文语义理解最强）或 `nomic-embed-text`（轻量替代）。
- **批量处理**: 支持批量向量化以提升效率。

### 3.3 向量存储 (Vector Storage)
- **存储方案**: SQLite + `sqlite-vss` 向量扩展。
- **表结构**:
  ```sql
  CREATE TABLE chunks (
      id TEXT PRIMARY KEY,
      document_id TEXT NOT NULL,
      chunk_index INTEGER NOT NULL,
      content TEXT NOT NULL,
      metadata TEXT, -- JSON: {page, heading, etc.}
      created_at TEXT NOT NULL DEFAULT (datetime('now'))
  );
  
  CREATE VIRTUAL TABLE chunk_vectors USING vss0(
      embedding(768)  -- 向量维度取决于嵌入模型
  );
  ```
- **索引管理**: 向量索引与文本数据在同一 SQLite 数据库中，确保数据一致性。

### 3.4 工作流程
```
解析后的文本
    ↓
[语义分块引擎] → 段落边界识别 → 智能合并/拆分 → 生成块列表
    ↓
[向量化引擎] → 批量调用 Ollama Embeddings API → 生成向量
    ↓
[向量存储] → 写入 SQLite chunks 表 + chunk_vectors 虚拟表
```

## 4. 接口设计 (API Design)

### 4.1 Tauri Commands
- `chunk_document(doc_id: String) -> Result<Vec<Chunk>, AppError>`: 对指定文档进行分块。
- `embed_document(doc_id: String) -> Result<(), AppError>`: 对指定文档的块进行向量化。
- `index_document(doc_id: String) -> Result<IndexResult, AppError>`: 一键完成分块+向量化。
- `get_document_chunks(doc_id: String) -> Result<Vec<Chunk>, AppError>`: 获取文档的所有块。

### 4.2 前端界面
- **文档详情面板**: 在文档列表中点击文档可查看其分块结果。
- **索引进度**: 实时显示向量化进度（已完成/总块数）。
- **块预览**: 显示每个块的内容摘要和元数据。

## 5. 错误处理与性能 (Error Handling & Performance)
- **模型未下载**: 检测到嵌入模型未安装时，提示用户下载。
- **向量维度不匹配**: 自动检测模型输出维度并更新表结构。
- **批量优化**: 使用异步批量请求减少 Ollama API 调用次数。
- **增量索引**: 仅对新增或修改的文档进行向量化。

## 6. 验收标准 (Success Criteria)
- [ ] 文档解析后能自动按语义分块。
- [ ] 每个块具有完整的段落上下文（无截断）。
- [ ] 向量正确写入 SQLite 向量表。
- [ ] 嵌入模型未安装时显示明确提示。
- [ ] 批量向量化效率显著优于逐条处理。
