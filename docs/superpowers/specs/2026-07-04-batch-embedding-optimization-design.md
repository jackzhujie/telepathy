# 设计规格说明：批量向量化性能优化

## 1. 需求背景
目前 Telepathy 应用在文档导入 (`indexing.rs`) 和重建索引 (`reindex.rs`) 时，采用 `for` 循环依次 `await` 调用单条向量化接口 `emb.embed(text)`。这种实现对每个文本分片都会单独请求全局模型锁并进行线程切换。在处理大文档（分片数多）时，会造成严重的锁竞争和线程切换开销，并在向量化期间会完全阻塞聊天的流式响应。

为了提升性能并兼顾良好的交互体验，我们需要实现**分批次限制大小的批量向量化**优化。

## 2. 方案设计
通过使用 `Embedder` 提供的 `embed_batch` 接口，我们将文本切片列表划分为固定大小的批次，批量生成向量后入库，并在批次之间短暂释放 CPU/锁。

### 2.1 批量大小与调度机制
- **批次大小**：设定常数 `BATCH_SIZE: usize = 16`。
- **让出机制**：每个批次向量化及入库完成后，调用 `tokio::time::sleep(std::time::Duration::from_millis(20)).await`，以确保 Tokio 运行时切换协程，释放全局 Mutex 模型锁，给前台聊天对话等高优请求提供插队机会。

### 2.2 核心修改点

#### 1. 索引服务优化 (`src-tauri/src/commands/indexing.rs`)
在 `index_document` 函数中，重构 Chunks 入库循环：
1. 使用 `text_chunks.chunks(16)` 进行分批。
2. 对每个批次 `batch_chunks`：
   - 调用 `emb.embed_batch(batch_chunks).await` 生成该批次的向量。
   - 遍历 `batch_chunks` 并在数据库中插入 `Chunk`。
   - 如果批量向量化成功，将生成的向量写入数据库 `insert_embedding` 并添加到 HNSW 索引中。
   - 每次批处理结束，调用 `tokio::time::sleep` 让出 CPU 和锁。

#### 2. 重索引服务优化 (`src-tauri/src/commands/reindex.rs`)
在 `reindex_single_document` 函数中，采用与上述完全相同的分批和入库机制。在所有批次处理完毕后，再触发 HNSW 索引重建。

---

## 3. 测试与验证计划
1. **编译检查**：在 `src-tauri` 目录下运行 `cargo check` 确保代码无编译错误。
2. **单元与集成测试**：运行 `cargo test`，确保原有的测试集全部通过，无逻辑破坏。
