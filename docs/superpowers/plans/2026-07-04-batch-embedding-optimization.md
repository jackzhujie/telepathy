# 批量向量化性能优化 (Batch Embedding Optimization) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Optimize document indexing and reindexing to use sized batches (size 16) with natural lock/CPU yielding, reducing model mutex contention and thread switches.

**Architecture:** Use `Embedder::embed_batch` to vectorize chunks in groups of 16. In between batches, yield CPU control and release the model lock via a small sleep `tokio::time::sleep(20ms)` to allow other concurrent tasks (like chat conversation) to run smoothly.

**Tech Stack:** Rust, Tokio, Tauri v2

---

### Task 1: Optimize Document Indexing in `indexing.rs`

**Files:**
- Modify: `src-tauri/src/commands/indexing.rs`

- [ ] **Step 1: Replace sequential embedding with sized batching in indexing.rs**
  In `src-tauri/src/commands/indexing.rs`, rewrite the chunk insertion and embedding loop in `index_document` to slice `text_chunks` into batches of 16.
  Replace lines 96-121 in `src-tauri/src/commands/indexing.rs`:
  ```rust
      // Insert chunks and generate embeddings
      for (i, chunk_content) in text_chunks.iter().enumerate() {
          let chunk = vectors::Chunk {
              id: Uuid::new_v4().to_string(),
              document_id: doc_id.clone(),
              chunk_index: i as i32,
              content: chunk_content.clone(),
              metadata: None,
              created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
          };

          vectors::insert_chunk(&conn, &chunk)?;

          // Try to embed, continue even if embedding fails
          if let Ok(embedding) = emb.embed(chunk_content).await {
              vectors::insert_embedding(&conn, &chunk.id, &embedding)?;

              let add_result = with_hnsw_manager(|manager| {
                  manager.add_chunk(&chunk.id, &embedding, embedding.len(), doc.project_id.as_deref())
              });
              if let Err(e) = add_result.and_then(|r| r) {
                  println!("[HNSW] Warning: failed to add chunk to index: {}", e);
              }
          }
      }
  ```
  with:
  ```rust
      // Insert chunks and generate embeddings in sized batches (size 16) to avoid lock contention
      const BATCH_SIZE: usize = 16;
      for (batch_idx, chunk_slice) in text_chunks.chunks(BATCH_SIZE).enumerate() {
          // Batch embed current slice
          let embeddings = match emb.embed_batch(chunk_slice).await {
              Ok(embs) => Some(embs),
              Err(e) => {
                  println!("[Indexing] Warning: batch embedding failed for batch {}: {}", batch_idx, e);
                  None
              }
          };

          // Write chunks and corresponding embeddings in this batch
          for (i, chunk_content) in chunk_slice.iter().enumerate() {
              let global_idx = batch_idx * BATCH_SIZE + i;
              let chunk_id = Uuid::new_v4().to_string();
              let chunk = vectors::Chunk {
                  id: chunk_id.clone(),
                  document_id: doc_id.clone(),
                  chunk_index: global_idx as i32,
                  content: chunk_content.clone(),
                  metadata: None,
                  created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
              };

              vectors::insert_chunk(&conn, &chunk)?;

              if let Some(ref embs) = embeddings {
                  if let Some(embedding) = embs.get(i) {
                      vectors::insert_embedding(&conn, &chunk_id, embedding)?;

                      let add_result = with_hnsw_manager(|manager| {
                          manager.add_chunk(&chunk_id, embedding, embedding.len(), doc.project_id.as_deref())
                      });
                      if let Err(e) = add_result.and_then(|r| r) {
                          println!("[HNSW] Warning: failed to add chunk to index: {}", e);
                      }
                  }
              }
          }

          // Yield lock and CPU control to allow concurrent tasks (e.g., chat) to run
          if (batch_idx + 1) * BATCH_SIZE < text_chunks.len() {
              tokio::time::sleep(std::time::Duration::from_millis(20)).await;
          }
      }
  ```

- [ ] **Step 2: Run cargo check to verify compilation**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 3: Commit changes**
  ```bash
  git add src-tauri/src/commands/indexing.rs
  git commit -m "feat: optimize document indexing to use sized batch embedding and lock yielding"
  ```

---

### Task 2: Optimize Document Reindexing in `reindex.rs`

**Files:**
- Modify: `src-tauri/src/commands/reindex.rs`

- [ ] **Step 1: Replace sequential embedding with sized batching in reindex.rs**
  In `src-tauri/src/commands/reindex.rs`, rewrite the chunk insertion and embedding loop in `reindex_single_document` to slice `text_chunks` into batches of 16.
  Replace lines 179-198 in `src-tauri/src/commands/reindex.rs`:
  ```rust
      for (i, chunk_content) in text_chunks.iter().enumerate() {
          let chunk_id = Uuid::new_v4().to_string();

          let chunk = Chunk {
              id: chunk_id.clone(),
              document_id: doc_id.to_string(),
              chunk_index: i as i32,
              content: chunk_content.clone(),
              metadata: None,
              created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
          };

          vectors::insert_chunk(&conn, &chunk)
              .map_err(|e| AppError::Internal(format!("Failed to insert chunk: {}", e)))?;

          if let Ok(embedding) = emb.embed(chunk_content).await {
              vectors::insert_embedding(&conn, &chunk_id, &embedding)
                  .map_err(|e| AppError::Internal(format!("Failed to insert embedding: {}", e)))?;
          }
      }
  ```
  with:
  ```rust
      // Insert chunks and generate embeddings in sized batches (size 16) to avoid lock contention
      const BATCH_SIZE: usize = 16;
      for (batch_idx, chunk_slice) in text_chunks.chunks(BATCH_SIZE).enumerate() {
          // Batch embed current slice
          let embeddings = match emb.embed_batch(chunk_slice).await {
              Ok(embs) => Some(embs),
              Err(e) => {
                  println!("[Reindexing] Warning: batch embedding failed for batch {}: {}", batch_idx, e);
                  None
              }
          };

          // Write chunks and corresponding embeddings in this batch
          for (i, chunk_content) in chunk_slice.iter().enumerate() {
              let global_idx = batch_idx * BATCH_SIZE + i;
              let chunk_id = Uuid::new_v4().to_string();
              let chunk = Chunk {
                  id: chunk_id.clone(),
                  document_id: doc_id.to_string(),
                  chunk_index: global_idx as i32,
                  content: chunk_content.clone(),
                  metadata: None,
                  created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
              };

              vectors::insert_chunk(&conn, &chunk)
                  .map_err(|e| AppError::Internal(format!("Failed to insert chunk: {}", e)))?;

              if let Some(ref embs) = embeddings {
                  if let Some(embedding) = embs.get(i) {
                      vectors::insert_embedding(&conn, &chunk_id, embedding)
                          .map_err(|e| AppError::Internal(format!("Failed to insert embedding: {}", e)))?;
                  }
              }
          }

          // Yield lock and CPU control to allow concurrent tasks (e.g., chat) to run
          if (batch_idx + 1) * BATCH_SIZE < text_chunks.len() {
              tokio::time::sleep(std::time::Duration::from_millis(20)).await;
          }
      }
  ```

- [ ] **Step 2: Run cargo check to verify compilation**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 3: Run all cargo tests to ensure no regressions**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 4: Commit changes**
  ```bash
  git add src-tauri/src/commands/reindex.rs
  git commit -m "feat: optimize document reindexing to use sized batch embedding and lock yielding"
  ```
