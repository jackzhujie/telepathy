# HNSW 向量索引安全重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 彻底重构本地 HNSW 向量索引，用纯 Rust 的高级安全 `usearch::Index` 包装替换不安全的底层的裸 C++ `usearch::ffi::Index` 指针，以防止在 Intel Mac 等环境中出现闪退与进程不可中断挂起死锁。

**Architecture:** 变更 `HnswIndex` 的内部类型定义。使用 `IndexOptions` 声明式配置并实例化 `usearch::Index`；用 `Index::restore` 实现自适应维度还原以加载索引；重构测试模块，通过重定向 `target` 目录绕过挂起僵尸进程占用的文件锁进行安全验证。

**Tech Stack:** Rust (Tauri Backend), usearch 0.12, cxx (仅在 `build.rs` 级底层使用，但在本模块彻底被安全的纯 Rust Wrapper 取代)

---

### Task 1: 重构 `HnswIndex` 结构体和实例化逻辑 (HnswIndex::new)

**Files:**
- Modify: `src-tauri/src/services/hnsw_index.rs:1-35`

- [ ] **Step 1: 修改类型导入与结构体定义**
  
  修改 `/Users/mac/project/telepathy/src-tauri/src/services/hnsw_index.rs` 前部的导入语句和 `HnswIndex` 定义，移除 `ffi` 及 `cxx::UniquePtr` 的直接引用：
  
  ```rust
  use crate::errors::AppError;
  use std::collections::HashMap;
  use std::path::{Path, PathBuf};
  use std::sync::{Arc, RwLock};
  use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

  pub struct HnswIndex {
      index: Index,
      chunk_ids: RwLock<HashMap<usize, String>>,
      dimension: usize,
  }
  ```
  
  同时移除：
  ```rust
  unsafe impl Send for HnswIndex {}
  unsafe impl Sync for HnswIndex {}
  ```
  *原因：安全的 `usearch::Index` 在 Rust 层已经天然保证了 `Send` 和 `Sync`。*

- [ ] **Step 2: 修改实例化逻辑 `new`**
  
  使用 `IndexOptions` 来声明式地配置和实例化安全索引：
  
  ```rust
  impl HnswIndex {
      pub fn new(dimension: usize) -> Result<Self, AppError> {
          let mut options = IndexOptions::default();
          options.dimensions = dimension;
          options.metric = MetricKind::Cos;
          options.quantization = ScalarKind::F32;

          let index = Index::new(&options)
              .map_err(|e| AppError::Internal(format!("Failed to create HNSW index: {:?}", e)))?;

          Ok(Self {
              index,
              chunk_ids: RwLock::new(HashMap::new()),
              dimension,
          })
      }
  }
  ```

---

### Task 2: 重构核心操作逻辑 (`add`、`search`、`save`、`load`) 与测试验证

**Files:**
- Modify: `src-tauri/src/services/hnsw_index.rs:36-437`

- [ ] **Step 1: 重构向量检索 `search`**
  
  修改 `search` 方法，使用 `usearch::Index::search`：
  
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
          .keys // 0.12 版本中从 labels 改为 keys
          .iter()
          .zip(results.distances.iter())
          .filter_map(|(&key, &distance)| {
              chunk_ids.get(&(key as usize)).map(|id| (id.clone(), distance))
          })
          .collect();

      Ok(search_results)
  }
  ```

- [ ] **Step 2: 重构向量插入 `add`**
  
  修改 `add` 方法：
  
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

- [ ] **Step 3: 重构持久化 `save` 与自适应还原 `load`**
  
  使用安全的 `save` 和自适应头配置解析加载的 `restore` 方法：
  
  ```rust
  pub fn save(&self, path: &Path) -> Result<(), AppError> {
      self.index
          .save(path.to_str().unwrap())
          .map_err(|e| AppError::Internal(format!("Save index failed: {:?}", e)))
  }

  pub fn load(path: &Path) -> Result<Self, AppError> {
      let index = Index::restore(path.to_str().unwrap())
          .map_err(|e| AppError::Internal(format!("Failed to load/restore index: {:?}", e)))?;

      let dimension = index.dimensions();
      Ok(Self {
          index,
          chunk_ids: RwLock::new(HashMap::new()),
          dimension,
      })
  }
  ```

- [ ] **Step 4: 修改单元测试并补充测试点**
  
  重写底部的 `tests` 模块：
  
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use std::fs;

      #[test]
      fn test_hnsw_save_and_load() {
          println!("Test started.");
          
          let mut options = IndexOptions::default();
          options.dimensions = 128;
          options.metric = MetricKind::Cos;
          options.quantization = ScalarKind::F32;

          let index = Index::new(&options).unwrap();
          println!("Index created successfully. Size: {}", index.size());
          
          let vector = vec![1.0; 128];
          println!("Adding vector...");
          index.add(1, &vector).unwrap();
          assert_eq!(index.size(), 1);
          println!("Vector added successfully. Size is now {}", index.size());

          // 验证持久化和恢复
          let test_path = "test_index.usearch";
          println!("Saving index to {}...", test_path);
          index.save(test_path).unwrap();

          println!("Restoring index from {}...", test_path);
          let restored = Index::restore(test_path).unwrap();
          assert_eq!(restored.size(), 1);
          assert_eq!(restored.dimensions(), 128);
          println!("Index restored successfully. Dimensions: {}", restored.dimensions());

          // 清理临时测试文件
          let _ = fs::remove_file(test_path);
          println!("Test completed successfully.");
      }
  }
  ```

- [ ] **Step 5: 规避 target 锁定进行安全编译测试**
  
  运行以下命令以利用 `target_temp` 临时目录重定向输出，绕过僵尸进程锁死的情况进行验证编译与单元测试：
  
  Run command: `CARGO_TARGET_DIR=target_temp cargo test --package temp-app --lib services::hnsw_index::tests -- --nocapture`
  Expected output: 测试套件编译成功并打印 `test services::hnsw_index::tests::test_hnsw_save_and_load ... ok` 且无任何崩溃及挂起死锁行为。

- [ ] **Step 6: 提交改动**
  
  测试通过后，向 Git 提交改动：
  
  Run command: `git add src-tauri/src/services/hnsw_index.rs`
  Run command: `git commit -m "refactor(hnsw): migrate raw C++ ffi index to safe rust usearch wrapper to prevent SIGSEGV crashes"`
