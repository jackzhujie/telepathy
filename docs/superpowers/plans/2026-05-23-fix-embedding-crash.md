# 修复 Embedding 模型向量化处理中闪退的实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 替换 embedder.rs 中的 decode API 为 encode API，解决因纯 Embedding 上下文缺少 logits 计算而在解码时越界访问空指针导致的 App 崩溃闪退问题。

**Architecture:** 将 embedder.rs 中的 `ctx.decode` 调用修改为专为向量编码设计的 `ctx.encode`。处理相应的错误映射，并运行内置的 `test_real_model_embedding` 单元测试验证其在 CPU 纯向量化模式下不再崩坏且正常输出数据。

**Tech Stack:** Rust, Tauri v2, llama-cpp-4 (v0.2.43)

---

### Task 1: 替换为正确的向量编码接口并运行 TDD 验证

**Files:**
- Modify: `src-tauri/src/services/embedder.rs:139-143`
- Test: `src-tauri/src/services/embedder.rs`

- [ ] **Step 1: 运行当前测试，验证存在 crash 段错误（TDD 失败验证）**

  在 `src-tauri` 目录下运行针对真实向量模型的测试：
  Run: `cargo test --package temp-app --lib -- services::embedder::tests::test_real_model_embedding -- --nocapture`
  Expected: 进程发生闪退（输出 `decode: cannot decode batches with this context (calling encode() instead)` 警告，随后测试非正常终止或报错退出）。

- [ ] **Step 2: 修改 embedder.rs 代码，将 decode 替换为 encode**

  修改 [embedder.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/embedder.rs)，定位到第 139-143 行。
  
  原代码：
  ```rust
  let decode_res = {
      let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
      ctx.decode(&mut batch)
  };
  decode_res.map_err(|e| AppError::Internal(format!("Decode error: {}", e)))?;
  ```
  
  替换为：
  ```rust
  let encode_res = {
      let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
      ctx.encode(&mut batch)
  };
  encode_res.map_err(|e| AppError::Internal(format!("Encode error: {}", e)))?;
  ```

- [ ] **Step 3: 运行测试以核实成功通过**

  在 `src-tauri` 目录下运行同一测试：
  Run: `cargo test --package temp-app --lib -- services::embedder::tests::test_real_model_embedding -- --nocapture`
  Expected: 测试通过，控制台成功打印向量信息（且没有任何 `cannot decode batches` 警告，且测试结果为 `ok`），输出类似如下：
  ```
  [TEST EMBEDDING] Embedding len = 1024
  [TEST EMBEDDING] Is all zeros = false
  test services::embedder::tests::test_real_model_embedding ... ok
  ```

- [ ] **Step 4: 运行 cargo check 确保项目无编译 Error**

  Run: `cargo check` in `src-tauri`
  Expected: 编译通过，无任何新增 Error。

- [ ] **Step 5: 提交代码修改**

  Run: 
  ```bash
  git add src-tauri/src/services/embedder.rs
  git commit -m "fix(embedder): use ctx.encode instead of ctx.decode to prevent segmentation faults during embedding generation"
  ```
