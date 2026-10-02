# HNSW Index 代码质量修复设计规格说明书 (2026-06-28)

## 1. 背景与目标
在 Task 2 审查中发现了 `src-tauri/src/services/hnsw_index.rs` 中的两个主要代码质量问题：
1. **未持久化重建索引**：在 `rebuild_index` 函数中，向 `indexes` 插入新索引后虽释放了锁，但并没有调用 `self.save_index(project_id)?` 将其持久化至磁盘。
2. **静默忽略元数据解析错误**：在 `get_or_create_index` 和 `load_existing_indexes` 中，当元数据文件存在但读取或反序列化失败时，错误被静默忽略。我们需要使用 `eprintln!` 打印警告。

本设计文档旨在提出具体修改规格，在确保系统正确性的同时，提高其健壮性。

## 2. 详细设计 (方案 1)

### 2.1 修复 `rebuild_index` 中的持久化与锁释放问题
**目标代码段**：`src-tauri/src/services/hnsw_index.rs` 的 `rebuild_index` 方法末尾。
在向 `indexes` 写入新重建的索引后，我们将通过 `drop(indexes)` 显式释放写锁，以防止持有写锁进行磁盘 I/O 带来的性能瓶颈与死锁隐跨。接着，调用 `self.save_index(project_id)?;` 对新索引进行序列化持久化。

**具体逻辑修改**：
```diff
         let index = Arc::new(new_index);
         indexes.insert(key, Arc::clone(&index));
+        drop(indexes);
+
+        self.save_index(project_id)?;
 
         println!("[HNSW] Rebuilt index with {} vectors", count);
         Ok(())
```

### 2.2 修复 `get_or_create_index` 中的静默警告问题
当元数据文件存在，但读取或反序列化失败时，我们将使用闭包统一处理错误，并使用 `eprintln!` 打印警告。

**具体逻辑修改**：
```diff
-            if meta_path.exists() {
-                if let Ok(meta_content) = std::fs::read_to_string(&meta_path) {
-                    if let Ok(meta) = serde_json::from_str::<HnswMeta>(&meta_content) {
-                        match meta {
-                            HnswMeta::V2 { dimension: dim, chunk_ids } => {
-                                resolved_dimension = dim;
-                                chunk_ids_opt = Some(chunk_ids);
-                            }
-                            HnswMeta::V1(ids) => {
-                                chunk_ids_opt = Some(ids);
-                            }
-                        }
-                    }
-                }
-            }
+            if meta_path.exists() {
+                let load_meta = || -> Result<_, Box<dyn std::error::Error>> {
+                    let content = std::fs::read_to_string(&meta_path)?;
+                    let meta: HnswMeta = serde_json::from_str(&content)?;
+                    Ok(meta)
+                };
+                match load_meta() {
+                    Ok(meta) => match meta {
+                        HnswMeta::V2 { dimension: dim, chunk_ids } => {
+                            resolved_dimension = dim;
+                            chunk_ids_opt = Some(chunk_ids);
+                        }
+                        HnswMeta::V1(ids) => {
+                            chunk_ids_opt = Some(ids);
+                        }
+                    },
+                    Err(e) => {
+                        eprintln!("[HNSW] Warning: failed to load or parse metadata at {:?}: {}", meta_path, e);
+                    }
+                }
+            }
```

### 2.3 修复 `load_existing_indexes` 中的静默警告问题
与 `get_or_create_index` 类似，如果在加载现有索引时，元数据文件存在但读取/解析失败，使用 `eprintln!` 打印警告。

**具体逻辑修改**：
```diff
-                    if meta_path.exists() {
-                        if let Ok(meta_content) = std::fs::read_to_string(&meta_path) {
-                            if let Ok(meta) = serde_json::from_str::<HnswMeta>(&meta_content) {
-                                match meta {
-                                    HnswMeta::V2 { dimension: dim, chunk_ids } => {
-                                        dimension = dim;
-                                        chunk_ids_opt = Some(chunk_ids);
-                                    }
-                                    HnswMeta::V1(ids) => {
-                                        chunk_ids_opt = Some(ids);
-                                    }
-                                }
-                            }
-                        }
-                    }
+                    if meta_path.exists() {
+                        let load_meta = || -> Result<_, Box<dyn std::error::Error>> {
+                            let content = std::fs::read_to_string(&meta_path)?;
+                            let meta: HnswMeta = serde_json::from_str(&content)?;
+                            Ok(meta)
+                        };
+                        match load_meta() {
+                            Ok(meta) => match meta {
+                                HnswMeta::V2 { dimension: dim, chunk_ids } => {
+                                    dimension = dim;
+                                    chunk_ids_opt = Some(chunk_ids);
+                                }
+                                HnswMeta::V1(ids) => {
+                                    chunk_ids_opt = Some(ids);
+                                }
+                            },
+                            Err(e) => {
+                                eprintln!("[HNSW] Warning: failed to load or parse metadata at {:?}: {}", meta_path, e);
+                            }
+                        }
+                    }
```

## 3. 验证与测试计划

1. **编译验证**：
   在 `src-tauri` 目录下运行 `cargo check`，确保引入的修改不存在类型或借用检查器错误（特别注意 `drop(indexes)` 后是否还有使用 `indexes` 的引用，以及 `self.save_index(project_id)?` 在 `rebuild_index` 的借用状态下是否正确）。
   
2. **测试验证**：
   在 `src-tauri` 目录下运行 `cargo test`。应运行现有的所有 HNSW 单元测试，保证未发生回归。
   
3. **单元测试增强**：
   编写一个特定的测试用例 `test_hnsw_rebuild_index_persistence` 和 `test_hnsw_metadata_parse_warning`，以验证：
   - 索引重建后，元数据和索引文件在磁盘上正确生成（验证 `save_index` 确实被调用）。
   - 当元数据文件损坏时，加载操作能够正常处理并发出警告（可使用 `temp_dir`，写入非法 JSON，然后验证是否输出警告且未发生崩溃）。

## 4. 提交计划
在所有验证通过后，将更改提交至当前分支。
- 提交信息：`refactor(hnsw): fix index persistence on rebuild and warn on metadata parse failures`
