# Implement HNSW Search Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Modify `search_with_embedding` in RAG service to try HNSW vector search first, and fallback to SQLite brute-force search if HNSW search fails, and verify correctness with a new unit test.

**Architecture:** Implement a match expression to catch the result of `vectors::search_similar_hnsw`. If it is `Ok(results)`, return it. If it is `Err(e)`, print a warning message with `eprintln!` and query SQLite via `vectors::search_similar` as a fallback.

**Tech Stack:** Rust, rusqlite, Tauri v2

---

### Task 1: Add unit test to reproduce current behaviour and test fallback mechanism

**Files:**
- Modify: `src-tauri/src/services/rag.rs`

- [ ] **Step 1: Add unit test verifying SQLite fallback when HNSW search fails**
  We will add a new test module `tests` at the bottom of `src-tauri/src/services/rag.rs` (if it does not exist) or add a test case. Since HNSW manager is not initialized by default in unit tests, `search_similar_hnsw` will return `Err`. This test will write a chunk and its embedding to an in-memory SQLite database, call `search_with_embedding` (which will fail HNSW search), and verify that it successfully falls back to SQLite search and returns the expected chunk.

  Add the following code to the end of `src-tauri/src/services/rag.rs`:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use rusqlite::Connection;
      use crate::db::vectors::{self, Chunk};

      fn setup_db() -> Connection {
          let conn = Connection::open_in_memory().unwrap();
          vectors::init_vector_tables(&conn).unwrap();
          conn.execute(
              "CREATE TABLE IF NOT EXISTS documents (
                  id TEXT PRIMARY KEY,
                  project_id TEXT,
                  name TEXT NOT NULL,
                  path TEXT NOT NULL,
                  created_at TEXT NOT NULL DEFAULT (datetime('now'))
              )",
              [],
          )
          .unwrap();
          conn.execute(
              "INSERT INTO documents (id, project_id, name, path) VALUES ('d1', 'p1', 'doc-1', 'path-1')",
              [],
          ).unwrap();
          conn
      }

      #[test]
      fn test_search_with_embedding_fallback_to_sqlite() {
          let conn = setup_db();
          let chunk = Chunk {
              id: "c1".to_string(),
              document_id: "d1".to_string(),
              chunk_index: 0,
              content: "Test chunk content for fallback".to_string(),
              metadata: None,
              created_at: "2026-01-01".to_string(),
          };
          vectors::insert_chunk(&conn, &chunk).unwrap();
          vectors::insert_embedding(&conn, "c1", &[1.0, 0.0, 0.0]).unwrap();

          // Under unit test environment without initializing HNSW manager, search_similar_hnsw will fail.
          // The search_with_embedding function should print fallback warning and fallback to SQLite vector search.
          let results = search_with_embedding(
              &[1.0, 0.0, 0.0],
              &conn,
              1,
              Some("p1".to_string()),
              0.5,
          );

          assert!(results.is_ok(), "search_with_embedding should succeed via SQLite fallback");
          let results = results.unwrap();
          assert_eq!(results.len(), 1, "Should find 1 document result");
          assert_eq!(results[0].id, "c1");
          assert_eq!(results[0].content, "Test chunk content for fallback");
          assert_eq!(results[0].document_name, "doc-1");
      }
  }
  ```

- [ ] **Step 2: Run the test to make sure it fails (since we haven't implemented fallback yet)**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --services::rag` (or similar filter)
  Expected: FAIL or compilation failure because `search_with_embedding` is currently only querying SQLite but we want to assert that it attempts HNSW first. Actually, wait! The current implementation of `search_with_embedding` only queries SQLite, so the test will PASS right now because it doesn't fail when HNSW is missing (it doesn't even call HNSW). However, once we implement Task 2 (which calls HNSW first), HNSW will fail, and if we don't have fallback, the test would fail. With fallback, the test will pass. Let's run it now to see it pass (with SQLite-only search).

### Task 2: Implement HNSW Search Fallback in `search_with_embedding`

**Files:**
- Modify: `src-tauri/src/services/rag.rs:34-36`

- [ ] **Step 1: Modify `search_with_embedding` to call HNSW search and handle fallback**
  Replace lines 34-36 in `src-tauri/src/services/rag.rs`:
  ```rust
      // 1. 检索文档片段
      let doc_results = vectors::search_similar(conn, embedding, top_k as usize, project_id)
          .map_err(|e| AppError::Internal(format!("Failed to search similar chunks: {}", e)))?;
  ```
  with:
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

- [ ] **Step 2: Run cargo check to verify compilation**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 3: Run the unit test created in Task 1**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib services::rag`
  Expected: PASS (It will print the Warning log for HNSW search failing, and then output SQLite search results successfully).

- [ ] **Step 4: Run all cargo tests to ensure no regressions**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

### Task 3: Commit Changes

- [ ] **Step 1: Commit modified files**
  Run:
  ```bash
  git add src-tauri/src/services/rag.rs
  git commit -m "feat: implement HNSW search fallback to SQLite brute-force search in RAG pipeline"
  ```
  Expected: PASS
