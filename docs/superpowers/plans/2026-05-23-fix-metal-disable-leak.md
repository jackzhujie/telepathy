# 防止 GGML_METAL_DISABLE 泄露清除的实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复向量化模块初始化时全局清除 GGML_METAL_DISABLE 变量的 Bug，防止非 M 系列 Intel Mac 设备失去 Metal 禁用锁进而加载其它模型时因为 GPU 分配驱动错误发生闪退。

**Architecture:** 
1. 将 `lib.rs` 中的 `is_m_series_mac` 导出为 `pub`。
2. 改造 `embedder.rs` 的环境变量管理：获取原有的 `GGML_METAL_DISABLE` 值并在推理完毕后对其状态进行优雅还原。非 M 系列 Mac 强行保持 `"1"` 不做清除。
3. 编写 TDD 单元测试，主动断言向量化流程后该环境变量的持久有效性。

**Tech Stack:** Rust, Tauri v2

---

### Task 1: 导出判断逻辑并实施环境变量优雅备份与还原

**Files:**
- Modify: `src-tauri/src/lib.rs:180-181`
- Modify: `src-tauri/src/services/embedder.rs:70-78`
- Modify: `src-tauri/src/services/embedder.rs` (追加单元测试)

- [ ] **Step 1: 在 embedder.rs 单元测试中编写环境保留检测用例（TDD 失败验证）**

  修改 [embedder.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/embedder.rs) 末尾的 `mod tests`。
  
  在 `test_real_model_embedding` 之下追加以下测试用例：
  ```rust
      #[tokio::test]
      async fn test_metal_disable_environment_leak() {
          // 1. 模拟全局防崩溃机制注入的变量
          std::env::set_var("GGML_METAL_DISABLE", "1");
          
          let path =
              "/Users/mac/Library/Application Support/com.telepathy.app/downloads/bge-m3-q4_k_m.gguf";
          if std::path::Path::new(path).exists() {
              let emb = Embedder::new(path);
              let _ = emb.embed("hello").await.unwrap();
              
              // 2. 断言向量化操作后，此全局防崩溃锁仍然存在，没有被 remove_var
              let current_val = std::env::var("GGML_METAL_DISABLE").unwrap_or_default();
              assert_eq!(current_val, "1", "GGML_METAL_DISABLE was leaked/removed!");
          }
      }
  ```

  在 `src-tauri` 目录下运行该特定测试：
  Run: `cargo test --package temp-app --lib -- services::embedder::tests::test_metal_disable_environment_leak -- --nocapture`
  Expected: FAIL（由于当前代码直接调用了 `remove_var`，断言会在向量化结束后因变量丢失而失败）。

- [ ] **Step 2: 修改 lib.rs 将 is_m_series_mac 函数改为 pub**

  修改 [lib.rs](file:///Users/mac/project/telepathy/src-tauri/src/lib.rs)，将第 181 行的 `fn is_m_series_mac` 暴露为 `pub fn is_m_series_mac`。
  
  原代码：
  ```rust
  #[cfg(target_os = "macos")]
  fn is_m_series_mac() -> bool {
  ```
  
  修改为：
  ```rust
  #[cfg(target_os = "macos")]
  pub fn is_m_series_mac() -> bool {
  ```

- [ ] **Step 3: 修改 embedder.rs 向量化环境设定，实现状态恢复防清除**

  修改 [embedder.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/embedder.rs)，将第 70-77 行的代码：
  
  原代码：
  ```rust
                  let embed_backend = {
                      let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                      std::env::set_var("GGML_METAL_DISABLE", "1");
                      let res = llama_cpp_4::llama_backend::LlamaBackend::init()
                          .map_err(|e| AppError::Internal(format!("Failed to initialize embed backend: {}", e)));
                      std::env::remove_var("GGML_METAL_DISABLE");
                      res
                  }?;
  ```
  
  修改为：
  ```rust
                  let embed_backend = {
                      let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                      
                      // 备份原状态
                      let old_val = std::env::var("GGML_METAL_DISABLE").ok();
                      std::env::set_var("GGML_METAL_DISABLE", "1");
                      
                      let res = llama_cpp_4::llama_backend::LlamaBackend::init()
                          .map_err(|e| AppError::Internal(format!("Failed to initialize embed backend: {}", e)));
                      
                      // 区分平台与架构还原环境状态
                      #[cfg(target_os = "macos")]
                      let is_m = crate::is_m_series_mac();
                      #[cfg(not(target_os = "macos"))]
                      let is_m = true;

                      if is_m {
                          match old_val {
                              Some(val) => std::env::set_var("GGML_METAL_DISABLE", val),
                              None => std::env::remove_var("GGML_METAL_DISABLE"),
                          }
                      } else {
                          // 非 M 芯片 Mac 强行保持禁用状态，防止被错误清除
                          std::env::set_var("GGML_METAL_DISABLE", "1");
                      }
                      res
                  }?;
  ```

- [ ] **Step 4: 运行 TDD 单元测试验证状态成功保留（绿灯状态）**

  在 `src-tauri` 目录下运行同一测试：
  Run: `cargo test --package temp-app --lib -- services::embedder::tests::test_metal_disable_environment_leak -- --nocapture`
  Expected: PASS（断言成功通过，测试为 `ok`）。

- [ ] **Step 5: 运行 cargo check 确保编译无 Error**

  Run: `cargo check` in `src-tauri`
  Expected: 编译通过，没有类型错位或编译器报错。

- [ ] **Step 6: 提交代码修改**

  Run: 
  ```bash
  git add src-tauri/src/lib.rs src-tauri/src/services/embedder.rs
  git commit -m "fix(embedder): prevent clearing GGML_METAL_DISABLE on Intel/Radeon Mac devices during initialization"
  ```
