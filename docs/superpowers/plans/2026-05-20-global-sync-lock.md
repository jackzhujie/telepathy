# Global Sync Lock Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 `llama_backend.rs` 中引入全局同步互斥锁 `ACQUIRE_LOCK`，以便后续大模型、视觉模型和向量索引并发操作时进行同步。

**Architecture:** 在 `llama_backend.rs` 中引入标准库的 `Mutex as StdMutex`，并在全局定义 `ACQUIRE_LOCK: StdMutex<()>`。

**Tech Stack:** Rust, standard library sync primitives.

---

### Task 1: 增加全局同步锁 ACQUIRE_LOCK 并验证

**Files:**
- Modify: `src-tauri/src/services/llama_backend.rs`
- Test: 在 `src-tauri/src/services/llama_backend.rs` 中添加内联测试或依靠现有的编译检查

- [ ] **Step 1: 修改 `src-tauri/src/services/llama_backend.rs`**

  In `src-tauri/src/services/llama_backend.rs` 中添加 `ACQUIRE_LOCK` 定义以及相关的单元测试。

  ```rust
  use llama_cpp_4::llama_backend::LlamaBackend;
  use std::sync::{Arc, LazyLock, Mutex as StdMutex};

  pub static GLOBAL_BACKEND: LazyLock<Arc<LlamaBackend>> = LazyLock::new(|| {
      let backend = LlamaBackend::init().expect("Failed to initialize llama backend");
      Arc::new(backend)
  });

  // 全局同步互斥锁，用于同步所有 llama.cpp 底层操作
  pub static ACQUIRE_LOCK: StdMutex<()> = StdMutex::new(());

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_acquire_lock() {
          let lock = ACQUIRE_LOCK.lock();
          assert!(lock.is_ok());
      }
  }
  ```

- [ ] **Step 2: 运行编译和测试检查**

  在 `src-tauri` 目录下运行：
  `cargo test --lib services::llama_backend`
  Expected: PASS

- [ ] **Step 3: 提交代码**

  Run:
  ```bash
  git add src-tauri/src/services/llama_backend.rs
  git commit -m "refactor: add global sync lock ACQUIRE_LOCK to llama_backend"
  ```
  Expected: SUCCESS
