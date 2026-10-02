#防止 Embedder 初始化时清除 GGML_METAL_DISABLE 环境变量的实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复在非 M 系列芯片的 Mac 上使用 Embedder 时，GGML_METAL_DISABLE 环境变量由于 remove_var 导致被清空以致引发应用崩溃的问题。

**Architecture:** 将 `src-tauri/src/lib.rs` 中的 `is_m_series_mac` 暴露为 `pub fn`。在 `src-tauri/src/services/embedder.rs` 的 CPU-only backend 初始化模块中，备份 `GGML_METAL_DISABLE` 的原始值，并在初始化完成后恢复。若为非 M 芯片 Mac，则强行置为 `"1"`。

**Tech Stack:** Rust, Tauri v2

---

### Task 1: 编写 TDD 失败验证测试用例

**Files:**
- Modify: `src-tauri/src/services/embedder.rs` (末尾的 `mod tests`)

- [ ] **Step 1: 在 embedder.rs 单元测试中追加测试用例**

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

- [ ] **Step 2: 运行测试并验证它因断言失败而 FAIL**

运行以下命令：
```bash
cargo test --package temp-app --lib -- services::embedder::tests::test_metal_disable_environment_leak -- --nocapture
```
预期：测试失败（如果本地有该模型的话，应该跑这一用例并由于 `remove_var` 产生断言失败；如果没有模型，该用例直接通过且控制台没有失败信息。如果是后者，我们将根据实际情况记录下来）。

---

### Task 2: 暴露 is_m_series_mac 函数为 pub

**Files:**
- Modify: `src-tauri/src/lib.rs:181`

- [ ] **Step 1: 将 is_m_series_mac 变更为 pub**

修改前：
```rust
#[cfg(target_os = "macos")]
fn is_m_series_mac() -> bool {
```
修改后：
```rust
#[cfg(target_os = "macos")]
pub fn is_m_series_mac() -> bool {
```

- [ ] **Step 2: 运行 cargo check 确保编译无误**

运行：
```bash
cargo check
```
预期：OK

---

### Task 3: 优化向量化环境设定，实现状态恢复

**Files:**
- Modify: `src-tauri/src/services/embedder.rs:70-78`

- [ ] **Step 1: 修改 embedder.rs 向量化环境设定**

修改前：
```rust
                // 1. 初始化专用于 Embedding 的 CPU-only backend
                let embed_backend = {
                    let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                    std::env::set_var("GGML_METAL_DISABLE", "1");
                    let res = llama_cpp_4::llama_backend::LlamaBackend::init()
                        .map_err(|e| AppError::Internal(format!("Failed to initialize embed backend: {}", e)));
                    std::env::remove_var("GGML_METAL_DISABLE");
                    res
                }?;
```

修改后：
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

---

### Task 4: 验证单元测试与项目构建

**Files:**
- Test/Check: 全局

- [ ] **Step 1: 运行 TDD 单元测试验证状态成功保留（绿灯状态）**

运行：
```bash
cargo test --package temp-app --lib -- services::embedder::tests::test_metal_disable_environment_leak -- --nocapture
```
预期：ok

- [ ] **Step 2: 运行 cargo check 确保项目无编译 Error**

运行：
```bash
cargo check
```
预期：OK

---

### Task 5: 提交代码修改

**Files:**
- Commit: git repository

- [ ] **Step 1: 执行 git add 与 commit**

运行：
```bash
git add src-tauri/src/lib.rs src-tauri/src/services/embedder.rs
git commit -m "fix(embedder): prevent clearing GGML_METAL_DISABLE on Intel/Radeon Mac devices during initialization"
```
预期：提交成功
