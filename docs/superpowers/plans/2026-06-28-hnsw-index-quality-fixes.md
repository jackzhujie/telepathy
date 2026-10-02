# HNSW Index 代码质量修复实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 `src-tauri/src/services/hnsw_index.rs` 中的元数据读取异常警告静默问题，并解决重建索引后没有持久化存盘的问题。

**Architecture:** 
1. 在 `rebuild_index` 将新索引插入到 `indexes` 后，通过 `drop(indexes);` 释放写锁，随后调用 `self.save_index(project_id)?;` 保存索引至磁盘。
2. 在 `get_or_create_index` 和 `load_existing_indexes` 中使用闭包 `load_meta` 统一读取和反序列化文件，如果文件存在但报错则通过 `eprintln!` 输出警告信息。
3. 编写专属测试用例，通过 TDD 验证修复效果。

**Tech Stack:** Rust, Tauri v2, serde_json, usearch, rusqlite

---

### Task 1: 编写验证测试用例 (TDD - 编写失败测试)

**Files:**
- Modify: `src-tauri/src/services/hnsw_index.rs:472-568`

- [ ] **Step 1: 在 `tests` 模块中编写 `test_hnsw_rebuild_index_persistence` 和 `test_hnsw_metadata_corrupt_warning` 测试函数**

```rust
    #[test]
    fn test_hnsw_rebuild_index_persistence() {
        let temp_dir_path = std::env::temp_dir().join(format!("telepathy_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir_path).unwrap();
        let manager = HnswIndexManager::new(temp_dir_path.clone());
        let project_id = Some("test_rebuild_proj");
        let dimension = 128;

        // 1. 初始化数据库用于重建索引
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE documents (id TEXT PRIMARY KEY, project_id TEXT)",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE chunks (id TEXT PRIMARY KEY, document_id TEXT)",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE chunk_embeddings (chunk_id TEXT PRIMARY KEY, embedding BLOB)",
            [],
        ).unwrap();

        // 插入测试数据
        conn.execute(
            "INSERT INTO documents (id, project_id) VALUES ('doc1', 'test_rebuild_proj')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO chunks (id, document_id) VALUES ('chunk1', 'doc1')",
            [],
        ).unwrap();
        let embedding_bytes = crate::db::vectors::f32_vec_to_bytes(&vec![0.5; 128]);
        conn.execute(
            "INSERT INTO chunk_embeddings (chunk_id, embedding) VALUES ('chunk1', ?1)",
            [embedding_bytes],
        ).unwrap();

        // 2. 第一次 rebuild_index
        manager.rebuild_index(project_id, &conn, dimension).unwrap();

        // 3. 验证对应的 HNSW 索引和 metadata 是否保存至磁盘
        let index_path = manager.get_index_path(project_id);
        let meta_path = manager.get_meta_path(project_id);
        assert!(index_path.exists(), "Rebuilt index path should exist on disk");
        assert!(meta_path.exists(), "Rebuilt meta path should exist on disk");

        // 清理临时目录
        let _ = fs::remove_dir_all(&temp_dir_path);
    }

    #[test]
    fn test_hnsw_metadata_corrupt_warning() {
        let temp_dir_path = std::env::temp_dir().join(format!("telepathy_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir_path).unwrap();
        let manager = HnswIndexManager::new(temp_dir_path.clone());
        let project_id = Some("corrupt_meta_proj");
        let dimension = 128;

        // 1. 写入无效/损坏的元数据 JSON
        let meta_path = manager.get_meta_path(project_id);
        fs::write(&meta_path, "invalid json data").unwrap();

        // 2. 创建并保存对应 HNSW 索引文件，以触发 get_or_create_index 的读取逻辑
        let index_path = manager.get_index_path(project_id);
        let index = HnswIndex::new(dimension).unwrap();
        index.save(&index_path).unwrap();

        // 3. 触发 get_or_create_index
        // 虽然元数据损坏，但加载过程应当恢复（fallback）而不能崩溃，并打印警告。
        let result = manager.get_or_create_index(dimension, project_id);
        assert!(result.is_ok(), "Should fallback and return Ok even if metadata is corrupt");

        // 清理临时目录
        let _ = fs::remove_dir_all(&temp_dir_path);
    }
```

- [ ] **Step 2: 运行测试并确保其失败**

在 `src-tauri` 目录下运行：
Run: `cargo test --services::hnsw_index::tests::test_hnsw_rebuild_index_persistence`
Expected: FAIL，因为旧的 `rebuild_index` 函数在写入后并没有调用 `save_index` 将新索引写盘。

---

### Task 2: 修复代码并实现持久化与警告打印

**Files:**
- Modify: `src-tauri/src/services/hnsw_index.rs`

- [ ] **Step 1: 在 `rebuild_index` 中释放写锁并调用 `save_index`**

修改 `rebuild_index` 方法（修改点约在 386-391 行）：
```rust
        let index = Arc::new(new_index);
        indexes.insert(key, Arc::clone(&index));
        drop(indexes); // 显式释放写锁，避免在写磁盘时依然持有写锁

        self.save_index(project_id)?;

        println!("[HNSW] Rebuilt index with {} vectors", count);
        Ok(())
```

- [ ] **Step 2: 修复 `get_or_create_index` 中的元数据反序列化静默错误警告**

修改 `get_or_create_index` 中（修改点约在 191-205 行）：
```rust
            if meta_path.exists() {
                let load_meta = || -> Result<_, Box<dyn std::error::Error>> {
                    let content = std::fs::read_to_string(&meta_path)?;
                    let meta: HnswMeta = serde_json::from_str(&content)?;
                    Ok(meta)
                };
                match load_meta() {
                    Ok(meta) => match meta {
                        HnswMeta::V2 { dimension: dim, chunk_ids } => {
                            resolved_dimension = dim;
                            chunk_ids_opt = Some(chunk_ids);
                        }
                        HnswMeta::V1(ids) => {
                            chunk_ids_opt = Some(ids);
                        }
                    },
                    Err(e) => {
                        eprintln!("[HNSW] Warning: failed to load or parse metadata at {:?}: {}", meta_path, e);
                    }
                }
            }
```

- [ ] **Step 3: 修复 `load_existing_indexes` 中的元数据解析静默警告问题**

修改 `load_existing_indexes` 中（修改点约在 413-427 行）：
```rust
                    if meta_path.exists() {
                        let load_meta = || -> Result<_, Box<dyn std::error::Error>> {
                            let content = std::fs::read_to_string(&meta_path)?;
                            let meta: HnswMeta = serde_json::from_str(&content)?;
                            Ok(meta)
                        };
                        match load_meta() {
                            Ok(meta) => match meta {
                                HnswMeta::V2 { dimension: dim, chunk_ids } => {
                                    dimension = dim;
                                    chunk_ids_opt = Some(chunk_ids);
                                }
                                HnswMeta::V1(ids) => {
                                    chunk_ids_opt = Some(ids);
                                }
                            },
                            Err(e) => {
                                eprintln!("[HNSW] Warning: failed to load or parse metadata at {:?}: {}", meta_path, e);
                            }
                        }
                    }
```

---

### Task 3: 验证与提交

- [ ] **Step 1: 运行 `cargo check` 确保通过**

Run: `cargo check`
Expected: 编译通过且无警告或错误。

- [ ] **Step 2: 运行测试并确保所有测试通过**

Run: `cargo test`
Expected: 所有测试通过，且在 stderr 中能够看到 `[HNSW] Warning: failed to load or parse metadata at ...: ...` 类似的相关报错信息。

- [ ] **Step 3: 提交更改**

Run: 
```bash
git add docs/superpowers/specs/2026-06-28-hnsw-index-quality-fixes-design.md \
        docs/superpowers/plans/2026-06-28-hnsw-index-quality-fixes.md \
        src-tauri/src/services/hnsw_index.rs
git commit -m "refactor(hnsw): fix index persistence on rebuild and warn on metadata parse failures"
```
