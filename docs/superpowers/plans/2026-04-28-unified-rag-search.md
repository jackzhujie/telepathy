# 统一 RAG 检索系统实现计划 (Unified RAG Search)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将“随记”（SnapNote）集成到 RAG 检索流程中，使 AI 能基于随记内容回答问题。

**Architecture:** 采用混合检索方案，在 `search_with_embedding` 函数中并行/顺序检索文档向量和随记向量，合并结果并按相似度重排。

**Tech Stack:** Rust, rusqlite, RAG logic.

---

### Task 1: 升级检索核心逻辑

**Files:**
- Modify: `src-tauri/src/services/rag.rs`

- [ ] **Step 1: 修改 `search_with_embedding` 支持混合检索**

```rust
pub fn search_with_embedding(
    embedding: &[f32],
    conn: &rusqlite::Connection,
    top_k: i32,
    project_id: Option<String>,
) -> Result<Vec<SearchSource>, AppError> {
    // 1. 检索文档片段
    let doc_results = vectors::search_similar(conn, embedding, top_k as usize, project_id)
        .map_err(|e| AppError::Internal(format!("Failed to search similar chunks: {}", e)))?;

    let mut combined_results: Vec<SearchSource> = doc_results
        .into_iter()
        .map(|r| SearchSource {
            id: r.chunk.id,
            document_name: r.document_name,
            content: r.chunk.content,
            chunk_index: r.chunk.chunk_index,
            score: r.score,
        })
        .collect();

    // 2. 检索个人随记 (Snaps)
    // 注意：随记是全局的，不属于特定项目
    if let Ok(snap_results) = crate::db::snaps::search_similar_snaps(conn, embedding, top_k as usize) {
        for (snap, score) in snap_results {
            combined_results.push(SearchSource {
                id: snap.id,
                document_name: "【个人随记】".to_string(),
                content: snap.content,
                chunk_index: 0,
                score,
            });
        }
    }

    // 3. 合并排序
    combined_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    
    // 4. 截断到 top_k
    combined_results.truncate(top_k as usize);

    Ok(combined_results)
}
```

- [ ] **Step 2: 验证编译**

运行: `cargo check`
预期: 编译通过，无未定义模块错误。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/services/rag.rs
git commit -m "feat: integrate snaps into RAG search core"
```

### Task 2: 验证与清理

**Files:**
- Modify: `src-tauri/src/commands/rag.rs` (检查调用点)

- [ ] **Step 1: 检查 `rag_query` 调用**
确保 `rag_query` 正确传递了 `conn` 给 `search_with_embedding`。

- [ ] **Step 2: 移除不再需要的冗余逻辑**
由于随记现在作为核心来源 (Sources) 传入，检查 `src-tauri/src/services/rag.rs` 中的 `build_ollama_messages_with_user`，确保它不会在背景信息中重复包含随记（可选，视具体效果而定）。

- [ ] **Step 3: 最终测试**
1. 确保数据库中有包含“张三”信息的随记。
2. 运行应用，在对话框输入“张三的律师费是多少”。
3. 检查控制台输出，确认 `SearchSource` 中包含了 `【个人随记】`。

- [ ] **Step 4: Commit**

```bash
git commit -m "chore: refine RAG prompt logic and finalize snap integration"
```
