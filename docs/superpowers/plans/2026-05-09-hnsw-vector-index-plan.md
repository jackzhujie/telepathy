# HNSW 向量索引实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `subagent-driven-development` (recommended) or `executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将向量搜索从 O(N) 全表扫描优化为 O(log N) HNSW 近似最近邻搜索，实现 10-100x 性能提升

**Architecture:** 使用 usearch 库实现内存 HNSW 索引，索引数据存储在 usearch 专属文件中，chunk 元数据继续保留在 SQLite 中。启动时自动加载索引，新增/删除时更新索引。

**Tech Stack:** Rust (usearch crate), SQLite, rusqlite

---

## 文件结构

```
src-tauri/src/
├── services/
│   └── hnsw_index.rs          # 新增: HNSW 索引管理服务
├── db/
│   └── vectors.rs             # 修改: 集成 HNSW 搜索
└── Cargo.toml                 # 修改: 添加 usearch 依赖
```

---

## 实施任务

### Task 1: 添加 usearch 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 usearch 依赖到 Cargo.toml**

打开文件，在 `[dependencies]` 区块末尾添加：

```toml
usearch = "0.12"
```

**验证**: 运行 `cd src-tauri && cargo check` 确保依赖能正常解析

---

### Task 2: 实现 HNSW 索引管理服务

**Files:**
- Create: `src-tauri/src/services/hnsw_index.rs`
- Modify: `src-tauri/src/services/mod.rs` (添加模块导出)

- [ ] **Step 1: 创建 hnsw_index.rs 基础结构**

```rust
// src-tauri/src/services/hnsw_index.rs
use crate::errors::AppError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use usearch::{Index, MetricKind};

pub struct HnswIndex {
    index: Index,
    chunk_ids: RwLock<HashMap<usize, String>>,
}

pub struct HnswIndexManager {
    indexes: RwLock<HashMap<Option<String>, Arc<HnswIndex>>>,
    data_dir: PathBuf,
}

impl HnswIndexManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            indexes: RwLock::new(HashMap::new()),
            data_dir,
        }
    }

    pub fn get_index_path(&self, project_id: Option<&str>) -> PathBuf {
        let mut path = self.data_dir.clone();
        path.push("hnsw");
        if let Some(pid) = project_id {
            path.push(format!("{}.usearch", pid));
        } else {
            path.push("default.usearch");
        }
        path
    }
}
```

- [ ] **Step 2: 实现索引搜索方法**

在 `HnswIndex` 中添加搜索方法：

```rust
impl HnswIndex {
    pub fn search(&self, embedding: &[f32], top_k: usize) -> Result<Vec<(String, f32)>, AppError> {
        let results = self.index.search(embedding, top_k);
        let chunk_ids = self.chunk_ids.read().unwrap();
        
        Ok(results
            .into_iter()
            .filter_map(|(idx, distance)| {
                chunk_ids.get(&idx).map(|id| (id.clone(), distance as f32))
            })
            .collect())
    }

    pub fn add(&self, chunk_id: &str, embedding: &[f32]) -> Result<(), AppError> {
        let idx = self.index.size();
        self.index.add(idx as u32, embedding);
        self.chunk_ids.write().unwrap().insert(idx, chunk_id.to_string());
        Ok(())
    }

    pub fn remove(&self, chunk_id: &str) -> Result<(), AppError> {
        // usearch 不支持直接删除，需要标记或重建
        // 简化方案：在查询时过滤
        let mut chunk_ids = self.chunk_ids.write().unwrap();
        if let Some(idx) = chunk_ids.iter().find(|(_, v)| *v == chunk_id) {
            // 标记删除（通过额外的数据结构追踪）
            // 这里先简化：移除映射但保留向量
            chunk_ids.remove(&idx.0);
        }
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.index.size()
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), AppError> {
        self.index.save(path).map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn load(path: &PathBuf, dimension: usize) -> Result<Self, AppError> {
        let index = if path.exists() {
            Index::load(path).map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            Index::new(&usearch::Config::new(dimension as u32, 16, MetricKind::Cosine))
                .map_err(|e| AppError::Internal(e.to_string()))?
        };
        Ok(Self {
            index,
            chunk_ids: RwLock::new(HashMap::new()),
        })
    }
}
```

- [ ] **Step 3: 在 mod.rs 中导出模块**

打开 `src-tauri/src/services/mod.rs`，添加：

```rust
pub mod hnsw_index;
```

- [ ] **Step 4: 编译验证**

运行 `cd src-tauri && cargo check` 确保没有编译错误

---

### Task 3: 实现全局 HNSW 管理器单例

**Files:**
- Modify: `src-tauri/src/services/hnsw_index.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加全局管理器单例**

在 `hnsw_index.rs` 末尾添加：

```rust
use once_cell::sync::Lazy;
use std::sync::OnceLock;

pub static HNSW_MANAGER: OnceLock<HnswIndexManager> = OnceLock::new();

pub fn init_hnsw_manager(data_dir: PathBuf) {
    let manager = HnswIndexManager::new(data_dir);
    let _ = HNSW_MANAGER.set(manager);
}

pub fn get_hnsw_manager() -> &'static HnswIndexManager {
    HNSW_MANAGER.get().expect("HNSW manager not initialized")
}
```

- [ ] **Step 2: 在 lib.rs 中初始化**

找到 `lib.rs` 中 `db::init_all_tables` 调用的位置，添加初始化代码：

```rust
// 在 db 初始化后添加
let app_data_dir = app_data_dir.expect("Failed to get app data dir");
let hnsw_dir = app_data_dir.join("hnsw");
std::fs::create_dir_all(&hnsw_dir).ok();
services::hnsw_index::init_hnsw_manager(hnsw_dir);
```

- [ ] **Step 3: 验证编译**

运行 `cd src-tauri && cargo check`

---

### Task 4: 修改 vectors.rs 实现 HNSW 搜索

**Files:**
- Modify: `src-tauri/src/db/vectors.rs`

- [ ] **Step 1: 添加 HNSW 搜索方法**

在 `vectors.rs` 中添加新的搜索函数：

```rust
pub fn search_similar_hnsw(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
    project_id: Option<String>,
    threshold: f32,
) -> Result<Vec<ChunkWithScore>, AppError> {
    use crate::services::hnsw_index::get_hnsw_manager;

    let manager = get_hnsw_manager();
    
    // 从 HNSW 获取候选
    let candidates = manager
        .search(query_embedding, limit, project_id.as_deref())
        .map_err(|e| AppError::Internal(format!("HNSW search failed: {}", e)))?;

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // 批量获取 chunk_ids
    let ids: Vec<String> = candidates.iter().map(|(id, _)| id.clone()).collect();
    
    // 从数据库获取元数据
    let placeholders: String = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let mut sql = format!(
        "SELECT c.id, c.document_id, c.chunk_index, c.content, c.metadata, c.created_at, d.name
         FROM chunks c
         JOIN documents d ON c.document_id = d.id
         WHERE c.id IN ({})",
        placeholders
    );
    
    if let Some(ref pid) = project_id {
        sql.push_str(" AND d.project_id = ?");
    }

    let mut stmt = conn.prepare(&sql).map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;
    
    let mut params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    if let Some(ref pid) = project_id {
        params.push(pid as &dyn rusqlite::ToSql);
    }

    let mut results = Vec::new();
    let mut rows = stmt.query(params.as_slice()).map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?;
    
    while let Some(row) = rows.next().map_err(|e| AppError::Internal(format!("Row error: {}", e)))? {
        let id: String = row.get(0).map_err(|e| AppError::Internal(format!("Get id failed: {}", e)))?;
        // 从 candidates 中获取分数
        let score = candidates.iter()
            .find(|(cid, _)| cid == &id)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
            
        if score >= threshold {
            results.push(ChunkWithScore {
                chunk: Chunk {
                    id,
                    document_id: row.get(1).map_err(|e| AppError::Internal(format!("Get doc_id failed: {}", e)))?,
                    chunk_index: row.get(2).map_err(|e| AppError::Internal(format!("Get chunk_index failed: {}", e)))?,
                    content: row.get(3).map_err(|e| AppError::Internal(format!("Get content failed: {}", e)))?,
                    metadata: row.get(4).map_err(|e| AppError::Internal(format!("Get metadata failed: {}", e)))?,
                    created_at: row.get(5).map_err(|e| AppError::Internal(format!("Get created_at failed: {}", e)))?,
                },
                document_name: row.get(6).map_err(|e| AppError::Internal(format!("Get doc_name failed: {}", e)))?,
                score,
            });
        }
    }

    // 按分数排序
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);

    Ok(results)
}
```

- [ ] **Step 2: 验证编译**

运行 `cd src-tauri && cargo check`

---

### Task 5: 集成到 RAG 流程

**Files:**
- Modify: `src-tauri/src/services/rag.rs`

- [ ] **Step 1: 替换 search_with_embedding 调用**

找到 `rag.rs` 中的搜索调用，替换为 HNSW 版本：

```rust
// 原代码 (约第 34-35 行):
let sources = rag::search_with_embedding(&query_embedding, &conn, top_k as i32, project_id, similarity_threshold)?;

// 替换为:
let sources = crate::db::vectors::search_similar_hnsw(
    &conn, 
    &query_embedding, 
    top_k as usize, 
    project_id, 
    similarity_threshold
)?;
```

- [ ] **Step 2: 验证编译**

运行 `cd src-tauri && cargo check`

---

### Task 6: 实现索引写入逻辑

**Files:**
- Modify: `src-tauri/src/commands/indexing.rs`
- Modify: `src-tauri/src/commands/reindex.rs`

- [ ] **Step 1: 在 indexing.rs 中添加索引更新**

找到 `index_document` 函数中插入 embedding 的位置，添加 HNSW 索引更新：

```rust
// 在 vectors::insert_embedding(&conn, &chunk.id, &embedding)?; 之后添加:
use crate::services::hnsw_index::get_hnsw_manager;
if let Err(e) = get_hnsw_manager().add_chunk(&chunk.id, &embedding, doc.project_id.as_deref()) {
    println!("[HNSW] Warning: failed to add chunk to index: {}", e);
}
```

- [ ] **Step 2: 在 reindex.rs 中添加索引清理**

找到删除 chunks 的位置，添加 HNSW 索引更新：

```rust
// 在 vectors::delete_chunks_by_document 之后添加:
use crate::services::hnsw_index::get_hnsw_manager;
if let Err(e) = get_hnsw_manager().rebuild_index(doc.project_id.as_deref()) {
    println!("[HNSW] Warning: failed to rebuild index: {}", e);
}
```

- [ ] **Step 3: 验证编译**

运行 `cd src-tauri && cargo check`

---

### Task 7: 添加索引持久化

**Files:**
- Modify: `src-tauri/src/services/hnsw_index.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 实现启动时加载索引**

在 `lib.rs` 中，找到 `init_hnsw_manager` 调用位置，修改为：

```rust
let hnsw_dir = app_data_dir.join("hnsw");
std::fs::create_dir_all(&hnsw_dir).ok();
services::hnsw_index::init_hnsw_manager(hnsw_dir.clone());

// 加载已存在的索引
services::hnsw_index::get_hnsw_manager().load_existing_indexes(&hnsw_dir);
```

在 `hnsw_index.rs` 中添加 `load_existing_indexes` 方法：

```rust
pub fn load_existing_indexes(&self, dir: &PathBuf) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "usearch") {
                if let Ok(index) = HnswIndex::load(&path, 1024) { // TODO: 从配置获取 dimension
                    let project_id = path.file_stem()
                        .and_then(|s| s.to_str())
                        .filter(|s| *s != "default")
                        .map(|s| s.to_string());
                    
                    let mut indexes = self.indexes.write().unwrap();
                    indexes.insert(project_id, Arc::new(index));
                    println!("[HNSW] Loaded index from {:?}", path);
                }
            }
        }
    }
}
```

- [ ] **Step 2: 实现优雅关闭时保存**

在 `lib.rs` 中添加关闭钩子：

```rust
// 需要在 Tauri builder 中添加 on_window_event 或 on_app_exit
use tauri::Manager;

.app_setup(|app| {
    // ... 现有代码 ...
    
    // 注册关闭回调保存 HNSW 索引
    let app_handle = app.handle().clone();
    app.on_app_exit(move || {
        println!("[HNSW] Saving indexes before exit...");
        if let Some(manager) = services::hnsw_index::HNSW_MANAGER.get() {
            manager.save_all();
        }
    });
})
```

- [ ] **Step 3: 验证编译**

运行 `cd src-tauri && cargo check`

---

### Task 8: 完整构建测试

**Files:**
- (无文件修改，纯测试)

- [ ] **Step 1: 运行 cargo build**

```bash
cd src-tauri && cargo build --release 2>&1 | tail -50
```

- [ ] **Step 2: 如有编译错误，逐一修复**

常见问题：
- usearch API 变更 → 查阅 usearch 0.12 文档
- 类型不匹配 → 调整 f32/f64 类型
- 路径问题 → 使用 Path/PathBuf 正确处理

- [ ] **Step 3: 测试运行**

```bash
pnpm tauri dev
```

观察日志输出 `[HNSW]` 相关调试信息

---

## 自检清单

在完成实现后，对照设计文档检查：

- [ ] spec 中的架构设计是否都已实现
- [ ] 索引持久化是否正常工作
- [ ] 新增 chunk 时索引是否同步更新
- [ ] 删除 chunk 时索引是否同步更新
- [ ] 搜索结果与之前是否一致（允许顺序有细微差异）
- [ ] 性能是否有明显提升

---

## 执行方式选择

**Plan complete and saved to `docs/superpowers/plans/2026-05-09-hnsw-vector-index-plan.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
