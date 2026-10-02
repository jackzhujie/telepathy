# 设计规格说明：RAG 管道中的 HNSW 检索降级机制

## 1. 需求背景
目前 Telepathy 应用的 RAG 服务在 `search_with_embedding` 函数中只使用了 SQLite 暴力搜索，这在大规模向量数据下可能存在性能瓶颈。新引入了 HNSW 向量搜索，但可能由于各种原因（例如索引尚未加载、HNSW manager 错误、没有索引数据等）导致 HNSW 检索失败。为了保证搜索服务的可用性，我们需要在 HNSW 检索失败时，能够自动降级回 SQLite 暴力搜索。

## 2. 方案设计
在 `src-tauri/src/services/rag.rs` 的 `search_with_embedding` 函数中：
1. 优先调用 `vectors::search_similar_hnsw(conn, embedding, top_k as usize, project_id.clone(), similarity_threshold)` 进行 HNSW 向量检索。
2. 使用 `match` 表达式对结果进行匹配：
   - `Ok(results)`: 说明检索成功，打印成功检索到的结果条数日志，并直接返回 `results`。
   - `Err(e)`: 说明 HNSW 检索失败或索引未加载。此时使用 `eprintln!` 打印 Warning 警告日志（包含错误信息），并调用 `vectors::search_similar(conn, embedding, top_k as usize, project_id.clone())` 进行 SQLite 暴力搜索，将错误转化为 `AppError::Internal`。
3. 保证原有的个人随记（Snaps）检索、结果合并排序、阈值过滤以及 top_k 截断逻辑保持不变。

### 代码修改点
文件: [rag.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/rag.rs)
修改前：
```rust
    // 1. 检索文档片段
    let doc_results = vectors::search_similar(conn, embedding, top_k as usize, project_id)
        .map_err(|e| AppError::Internal(format!("Failed to search similar chunks: {}", e)))?;
```

修改后：
```rust
    // 1. 检索文档片段 (优先使用 HNSW, 失败则降级到 SQLite 暴力搜索)
    let doc_results = match vectors::search_similar_hnsw(conn, embedding, top_k as usize, project_id.clone(), similarity_threshold) {
        Ok(results) => {
            println!("[RAG] HNSW vector search succeeded, found {} results", results.len());
            results
        }
        Err(e) => {
            eprintln!("[RAG] Warning: HNSW search failed or index not loaded. Falling back to SQLite brute-force search: {}", e);
            vectors::search_similar(conn, embedding, top_k as usize, project_id.clone())
                .map_err(|err| AppError::Internal(format!("Failed to search similar chunks: {}", err)))?
        }
    };
```

## 3. 测试与验证计划
1. **编译检查**: 在 `src-tauri` 目录下执行 `cargo check`，验证修改后的代码是否能正确编译。
2. **单元测试**: 在 `src-tauri` 目录下执行 `cargo test`，验证所有现有的测试套件依然能够通过。
