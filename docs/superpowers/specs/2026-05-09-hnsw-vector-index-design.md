# HNSW 向量索引优化设计

**日期**: 2026-05-09
**模块**: P0 - 后端性能优化
**状态**: 设计中

---

## 1. 背景与目标

### 1.1 当前问题

`src-tauri/src/db/vectors.rs` 中的 `search_similar` 函数存在严重的性能瓶颈：

```rust
// 问题：每次查询都全量扫描数据库
let rows = stmt.query_map([], |row| {
    let embedding_bytes: Vec<u8> = row.get(6)?;
    let embedding = bytes_to_f32_vec(&embedding_bytes);
    let score = cosine_similarity(query_embedding, &embedding);  // O(N) 计算
    ...
});
```

**性能问题**：
| 数据规模 | 当前查询时间 | 原因 |
|---------|-------------|------|
| 1,000 chunks | ~50ms | 全量加载 + O(N) 计算 |
| 10,000 chunks | ~500ms | 线性增长 |
| 100,000 chunks | ~5s | 不可接受 |

### 1.2 优化目标

- **查询延迟**: 从 O(N) 降低到 O(log N)
- **预期收益**: 查询速度提升 **10-100x**
- **内存占用**: 控制在合理范围内
- **兼容性**: 保持与现有 API 完全兼容

---

## 2. 技术方案

### 2.1 方案选型：使用 `rann` 或 `usearch` crate

| 方案 | 优点 | 缺点 | 推荐度 |
|------|------|------|--------|
| `rann` (Pure Rust) | 无外部依赖、易集成 | 维护不活跃 | ⭐⭐ |
| `usearch` | 性能极佳、支持 SIMD | 需编译 | ⭐⭐⭐ |
| `vecsimgrocerydemo/hnsw` | 活跃维护 | 功能较少 | ⭐⭐ |
| 手动实现 HNSW | 完全可控 | 工作量大 | ❌ |

**推荐方案**: `usearch` - 最快的 ANN 库之一，支持多种距离度量。

### 2.2 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                      索引服务层                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              HnswIndexManager                        │   │
│  │  - 全局单例                                          │   │
│  │  - 内存索引 + 持久化                                 │   │
│  │  - 项目隔离                                          │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      数据存储层                               │
│  ┌──────────────────────┐  ┌──────────────────────────┐    │
│  │   SQLite (元数据)    │  │   usearch (向量索引)      │    │
│  │   - chunks 表        │  │   - 内存索引文件          │    │
│  │   - chunk_embeddings │  │   - 自动持久化           │    │
│  └──────────────────────┘  └──────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 核心模块设计

#### 2.3.1 新增文件: `src-tauri/src/services/hnsw_index.rs`

```rust
use usearch::{Index, MetricKind, VectorStore};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use crate::errors::AppError;
use crate::db::vectors::ChunkWithScore;

pub struct HnswIndex {
    index: Index,
    chunk_id_to_db_id: HashMap<usize, String>,  // idx -> chunk_id
    db_id_to_idx: HashMap<String, usize>,       // chunk_id -> idx
    dimension: usize,
    is_built: bool,
}

pub struct HnswIndexManager {
    indexes: RwLock<HashMap<Option<String>, Arc<HnswIndex>>>,  // project_id -> index
    data_dir: std::path::PathBuf,
}

impl HnswIndexManager {
    pub fn new(data_dir: std::path::PathBuf) -> Self;

    pub fn get_or_create_index(&self, project_id: Option<String>) -> Result<Arc<HnswIndex>, AppError>;

    pub fn search(
        &self,
        embedding: &[f32],
        top_k: usize,
        project_id: Option<String>,
    ) -> Result<Vec<(String, f32)>, AppError>;  // (chunk_id, score)

    pub fn add_chunk(&self, chunk_id: &str, embedding: &[f32], project_id: Option<String>) -> Result<(), AppError>;

    pub fn remove_chunk(&self, chunk_id: &str, project_id: Option<String>) -> Result<(), AppError>;

    pub fn rebuild_index(&self, project_id: Option<String>) -> Result<(), AppError>;
}
```

#### 2.3.2 修改文件: `src-tauri/src/db/vectors.rs`

保留现有 API，新增 HNSW 加速版本：

```rust
// 新增：使用 HNSW 的搜索方法
pub fn search_similar_hnsw(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
    project_id: Option<String>,
    threshold: f32,
) -> Result<Vec<ChunkWithScore>, AppError> {
    // 1. 从 HNSW 索引获取候选 chunk_ids
    let candidates = hnsw_index_manager::search(query_embedding, limit, project_id)?;

    // 2. 精确计算分数（可选，用于精排）
    // 如果追求极致速度，可以直接使用 HNSW 返回的近似分数

    // 3. 批量从数据库获取元数据
    let chunks = get_chunks_by_ids(conn, &candidates)?;

    // 4. 组装结果
    ...
}
```

#### 2.3.3 修改文件: `src-tauri/src/services/rag.rs`

```rust
// 使用新的 HNSW 搜索
let sources = if use_hnsw {
    rag::search_with_embedding_hnsw(&query_embedding, &conn, top_k, project_id, similarity_threshold)?
} else {
    rag::search_with_embedding(&query_embedding, &conn, top_k as i32, project_id, similarity_threshold)?
};
```

---

## 3. 实现细节

### 3.1 依赖添加

```toml
# Cargo.toml
usearch = "0.12"
```

### 3.2 HNSW 参数配置

```rust
pub struct HnswConfig {
    pub m: usize,           // 每层最大连接数 (default: 16)
    pub ef_construction: usize,  // 构建时的搜索范围 (default: 100)
    pub ef_search: usize,   // 查询时的搜索范围 (default: 50)
    pub batch_size: usize,  // 批量索引大小 (default: 1000)
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 100,
            ef_search: 50,
            batch_size: 1000,
        }
    }
}
```

### 3.3 索引持久化

```rust
impl HnswIndex {
    pub fn save(&self, path: &Path) -> Result<(), AppError>;

    pub fn load(path: &Path, dimension: usize) -> Result<Self, AppError>;

    pub fn get_index_path(data_dir: &Path, project_id: Option<&str>) -> PathBuf {
        // 全局索引: data_dir/hnsw/index.usearch
        // 项目索引: data_dir/hnsw/projects/{project_id}.usearch
    }
}
```

### 3.4 增量更新策略

| 操作 | 策略 |
|------|------|
| 新增 chunk | 增量添加到索引 |
| 删除 chunk | 标记删除，定期重建 |
| 批量导入 | 批量构建索引 |
| 启动时 | 懒加载 + 后台预热 |

---

## 4. 数据迁移

### 4.1 迁移策略

1. **双写阶段** (v0.4.x): 同时写入 SQLite 和 HNSW
2. **灰度切换** (v0.5.x): 90% 流量走 HNSW
3. **完全切换** (v0.6.x): 移除旧的 brute-force 实现

### 4.2 迁移脚本

```rust
pub async fn migrate_to_hnsw(conn: &Connection, data_dir: &Path) -> Result<(), AppError> {
    // 1. 读取所有 embeddings
    let all_embeddings = read_all_embeddings(conn)?;

    // 2. 构建 HNSW 索引
    let mut index = HnswIndex::new(dimension)?;
    for (chunk_id, embedding) in all_embeddings {
        index.add(&chunk_id, &embedding)?;
    }

    // 3. 保存索引
    index.save(index_path)?;

    println!("[HNSW] Migration complete: {} chunks indexed", all_embeddings.len());
}
```

---

## 5. 回滚方案

如果 HNSW 出现严重问题，可快速回滚：

```rust
// 在 rag_query 中添加降级开关
let use_hnsw = settings::get_setting(conn, "use_hnsw_index")?
    .map(|v| v == "true")
    .unwrap_or(true);

if !use_hnsw {
    println!("[RAG] Falling back to brute-force search");
    return search_with_embedding(...);
}
```

---

## 6. 测试计划

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_hnsw_search_consistency() {
        // 对比 HNSW 结果与 brute-force 结果
        // 允许 top_k 内的结果顺序有小幅差异
    }

    #[test]
    fn test_hnsw_with_empty_index() {
        // 空索引应返回空结果
    }

    #[test]
    fn test_hnsw_persistence() {
        // 保存后重新加载，数据应一致
    }
}
```

### 6.2 性能基准测试

```rust
#[tokio::test]
async fn benchmark_search_performance() {
    // 测试不同数据规模下的查询时间
    // 100, 1000, 10000, 100000 chunks
    // 目标: 100k chunks 查询 < 10ms
}
```

---

## 7. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| usearch 编译问题 | 低 | 提供 fallback |
| 内存占用增加 | 中 | 限制索引大小、分页加载 |
| 索引损坏 | 中 | 保留 SQLite 作为备份 |
| 查询结果不一致 | 高 | 严格测试、灰度发布 |

---

## 8. 实施计划

### Phase 1: 基础设施 (1-2天)
- [ ] 添加 usearch 依赖
- [ ] 实现 HnswIndex 核心结构
- [ ] 实现持久化加载/保存

### Phase 2: 集成 (1-2天)
- [ ] 修改 vectors.rs 添加 HNSW 搜索
- [ ] 修改 rag.rs 集成 HNSW
- [ ] 实现增量更新逻辑

### Phase 3: 测试与调优 (1-2天)
- [ ] 单元测试
- [ ] 性能基准测试
- [ ] 参数调优 (ef_construction, m)

### Phase 4: 部署 (0.5天)
- [ ] 数据迁移脚本
- [ ] 灰度发布
- [ ] 监控指标添加

---

## 9. 附录

### A. usearch 简介

usearch 是一个高性能的近似最近邻搜索库：
- **速度**: 比 FAISS 快 10-50x
- **内存**: 紧凑的向量存储
- **精度**: 可配置召回率
- **距离度量**: 余弦、点积、L2

### B. 参考资料

- [usearch GitHub](https://github.com/unum-cloud/usearch)
- [HNSW 算法论文](https://arxiv.org/abs/1603.09320)
