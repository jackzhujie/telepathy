# HNSW Load Null Pointer Crash Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the Tauri application crash during document indexing by resolving a C++ null pointer dereference bug in `HnswIndex::load`.

**Architecture:** Modify `HnswIndex::load` in `src/services/hnsw_index.rs` to initialize a valid `cxx::UniquePtr<Index>` using `new_cos` before calling `.load()` on it. This prevents passing a `nullptr` to the C++ load function and avoids segmentation faults. Also, add unit tests in `hnsw_index.rs` to verify save/load logic.

**Tech Stack:** Rust, usearch (HNSW index)

---

### Task 1: Fix HnswIndex::load and add unit tests in hnsw_index.rs

**Files:**
- Modify: `src/services/hnsw_index.rs`
- Test: `src/services/hnsw_index.rs` (via `cargo test`)

- [ ] **Step 1: Write the implementation changes in src/services/hnsw_index.rs**

Update `HnswIndex::load` and add the test module at the end of `src/services/hnsw_index.rs`.

```rust
    pub fn load(path: &Path) -> Result<Self, AppError> {
        let quant = "f32";
        // Pre-allocate a valid Index instance with temporary dimensions to avoid nullptr dereference
        let index = new_cos(1024, &quant, 16, 0, 0)
            .map_err(|e: cxx::Exception| AppError::Internal(format!("Failed to create temporary index for loading: {}", e.what())))?;
        
        index
            .load(path.to_str().unwrap())
            .map_err(|e: cxx::Exception| AppError::Internal(format!("Failed to load index: {}", e.what())))?;

        let dimension = index.dimensions();
        Ok(Self {
            index,
            chunk_ids: RwLock::new(HashMap::new()),
            dimension,
        })
    }
```

Add the test module at the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_hnsw_save_and_load() {
        let temp_dir = std::env::temp_dir().join("hnsw_test_dir");
        fs::create_dir_all(&temp_dir).unwrap();
        let index_path = temp_dir.join("test_save_load.usearch");

        // 1. Create, add vector, and save index
        let index = HnswIndex::new(128).unwrap();
        let mock_vector = vec![1.0; 128];
        index.add("chunk_test_1", &mock_vector).unwrap();
        index.save(&index_path).unwrap();

        // 2. Load index using our fixed load function
        let loaded_index = HnswIndex::load(&index_path).unwrap();
        assert_eq!(loaded_index.dimension(), 128);
        assert_eq!(loaded_index.size(), 1);

        // Cleanup
        fs::remove_dir_all(temp_dir).ok();
    }
}
```

- [ ] **Step 2: Run cargo check to verify code builds successfully**

Run: `cargo check` (in `src-tauri` directory)
Expected: Build passes with no compilation errors.

- [ ] **Step 3: Run the new unit test to verify it passes**

Run: `cargo test --package temp-app --lib services::hnsw_index::tests` (in `src-tauri` directory)
Expected: Tests pass successfully with `test result: ok. 1 passed; 0 failed`.

- [ ] **Step 4: Commit the changes**

Run:
```bash
git add src/services/hnsw_index.rs
git commit -m "fix(hnsw): fix null pointer dereference crash in HnswIndex::load"
```
