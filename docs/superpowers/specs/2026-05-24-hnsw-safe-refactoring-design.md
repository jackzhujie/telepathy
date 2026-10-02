# 2026-05-24 HNSW 向量索引安全重构设计规范

## 1. 概述与背景
在 Telepathy 应用执行本地 RAG 文档向量化索引时，发生了点击索引闪退及应用死锁卡死的情况。经过深度排查和调试，定位出两个核心 Bug：
1. **CPU 内存与线程崩溃**：底层的 llama 向量模型在 Intel Mac CPU 环境下配置过大的 `n_batch`（默认 2048）引发了内存越界崩溃。已在此前修复并限制 `n_batch = 512` 和线程数 `n_threads = 4` 并通过单元测试验证。
2. **HNSW 写入与加载闪退**：`hnsw_index.rs` 内部直接调用了裸的 C++ 动态链接 FFI 指针 `usearch::ffi::Index` 和 `new_cos`。这些接口没有提供 Rust 层的并发和内存生命周期安全屏障，在 Intel Mac 运行 SIMD/AVX 指令时发生未对齐段错误（SIGSEGV 11），触发内核陷阱使 `cargo` 进程陷入不可中断等待状态（僵尸 `U` 状态），并导致加载已有索引时因空指针传入临时维度 Hack 带来闪退风险。

本设计旨在通过将底层裸 C++ 桥接彻底替换为官方高级安全 Rust API `usearch::Index`，从根本上解决内存对齐、AVX 崩溃、死锁和加载崩溃问题。

## 2. 架构设计与变更

### 2.1 依赖关系升级
* **废弃类型**：`cxx::UniquePtr<usearch::ffi::Index>`
* **采用类型**：`usearch::Index`
* **并发安全**：`usearch::Index` 原生在 Rust 内部实现并保证了 `Send` 和 `Sync`。我们可以安全地移除 `unsafe impl Send` / `unsafe impl Sync` 声明，交由 Rust 编译器提供严格的安全检查。

### 2.2 模块设计与数据流
`HnswIndex` 直接作为一个高内聚的组件，在内存中管理 HNSW 近邻图，并在需要时通过 `serde_json` 配合一个独立的物理 `.meta.json` 文件来管理向量主键（`usize`/`u64`）和文档 Chunk 唯一标识（`String`）的映射关系。

## 3. 具体接口实现说明

### 3.1 实例化与选项声明 (IndexOptions)
使用高层的 `IndexOptions` 替换原本传递裸参数的底层 C++ 实例化：
```rust
pub fn new(dimension: usize) -> Result<Self, AppError> {
    let mut options = usearch::IndexOptions::default();
    options.dimensions = dimension;
    options.metric = usearch::MetricKind::Cos;
    options.quantization = usearch::ScalarKind::F32;
    
    let index = usearch::Index::new(&options)
        .map_err(|e| AppError::Internal(format!("Failed to create HNSW index: {:?}", e)))?;
        
    Ok(Self {
        index,
        chunk_ids: RwLock::new(HashMap::new()),
        dimension,
    })
}
```

### 3.2 向量插入 (add)
通过 Rust 标准切片的安全范围检查向底层图添加向量。主键使用 `self.index.size() as u64`：
```rust
pub fn add(&self, chunk_id: &str, embedding: &[f32]) -> Result<(), AppError> {
    if embedding.len() != self.dimension {
        return Err(AppError::Internal(format!(
            "Embedding dimension mismatch: expected {}, got {}",
            self.dimension,
            embedding.len()
        )));
    }

    let key = self.index.size() as u64;
    self.index
        .add(key, embedding)
        .map_err(|e| AppError::Internal(format!("Add vector failed: {:?}", e)))?;

    let mut chunk_ids = self.chunk_ids.write().unwrap();
    chunk_ids.insert(key as usize, chunk_id.to_string());

    Ok(())
}
```

### 3.3 向量检索 (search)
通过高级安全的 search 接口完成近邻检索，并将 labels 字段优雅转换为 keys 字段的迭代过滤：
```rust
pub fn search(
    &self,
    embedding: &[f32],
    top_k: usize,
) -> Result<Vec<(String, f32)>, AppError> {
    if embedding.len() != self.dimension {
        return Err(AppError::Internal(format!(
            "Embedding dimension mismatch: expected {}, got {}",
            self.dimension,
            embedding.len()
        )));
    }

    let results = self
        .index
        .search(embedding, top_k)
        .map_err(|e| AppError::Internal(format!("Search failed: {:?}", e)))?;

    let chunk_ids = self.chunk_ids.read().unwrap();
    let search_results: Vec<(String, f32)> = results
        .keys // 0.12 版本使用 keys 获取匹配的主键迭代器
        .iter()
        .zip(results.distances.iter())
        .filter_map(|(&key, &distance)| {
            chunk_ids.get(&(key as usize)).map(|id| (id.clone(), distance))
        })
        .collect();

    Ok(search_results)
}
```

### 3.4 磁盘存储与自适应恢复 (save & restore)
* **持久化**：
  ```rust
  pub fn save(&self, path: &Path) -> Result<(), AppError> {
      self.index
          .save(path.to_str().unwrap())
          .map_err(|e| AppError::Internal(format!("Save index failed: {:?}", e)))
  }
  ```
* **自适应恢复**：
  使用自适应解析头信息的 `restore` 接口，解决不需要提前预测维度的启动加载难题：
  ```rust
  pub fn load(path: &Path) -> Result<Self, AppError> {
      let index = usearch::Index::restore(path.to_str().unwrap())
          .map_err(|e| AppError::Internal(format!("Failed to load/restore index: {:?}", e)))?;

      let dimension = index.dimensions();
      Ok(Self {
          index,
          chunk_ids: RwLock::new(HashMap::new()),
          dimension,
      })
  }
  ```

## 4. 验证与单元测试规划
为保证重构绝对可靠，必须在 `hnsw_index.rs` 的底端，编写端到端单元测试。

### 4.1 单元测试定义
测试完整创建、插入、获取 size、保存以及使用 `restore` 重新加载的完整功能逻辑（参考方案设计中的 `tests` 模块实现）。

### 4.2 绕过僵尸进程锁的安全编译验证
由于原本的 `target` 目录被残留僵尸 `cargo` 进程锁定，在进行编译测试时，我们必须使用如下指令指定另一个输出目录：
```bash
CARGO_TARGET_DIR=target_temp cargo test --package temp-app --lib services::hnsw_index::tests -- --nocapture
```
确认测试顺利通过且绝无闪退和挂起死锁行为。
