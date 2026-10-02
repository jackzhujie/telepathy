# 跨平台 llama.cpp 多后端支持升级实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 升级 llama-cpp-4 到 0.3 版本，启用多后端支持（metal, vulkan, openmp），并移除旧的 macOS 兼容性保护代码。

**Architecture:** 通过升级 llama-cpp-4 crate 并启用 vulkan 和 openmp features，让 llama.cpp 在运行时自动检测硬件并选择最佳后端（Metal/Vulkan/OpenMP）。

**Tech Stack:** Rust, Tauri v2, llama-cpp-4 0.3

---

## 文件改动清单

### 修改文件
- `src-tauri/Cargo.toml` - 升级 llama-cpp-4 版本和 features
- `src-tauri/src/services/embedder.rs` - 删除 GGML_METAL_DISABLE 相关代码
- `src-tauri/src/lib.rs` - 删除 is_m_series_mac() 函数
- `src-tauri/src/services/inference/llama_adapter.rs` - 删除 macOS GPU 层判断逻辑
- `src-tauri/src/services/inference/vision_adapter.rs` - 删除 macOS GPU 层判断逻辑

### 测试文件
- `src-tauri/src/services/llama_backend.rs` - 验证后端初始化正常

---

## 实施步骤

### Task 1: 升级 Cargo.toml 中的 llama-cpp-4 版本和 features

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 修改 Cargo.toml**

将 `llama-cpp-4` 从版本 0.2 升级到 0.3，并添加 vulkan 和 openmp features。

打开 `src-tauri/Cargo.toml`，找到第 30 行：

```toml
# 修改前（第 30 行）：
llama-cpp-4 = { version = "0.2", features = ["mtmd", "metal"] }

# 修改后：
llama-cpp-4 = { version = "0.3", features = ["mtmd", "metal", "vulkan", "openmp"] }
```

- [ ] **Step 2: 提交改动**

```bash
cd /Users/mac/.trae-cn/worktrees/telepathy/feat-llama-cpp-version-support-t8tRuE
git add src-tauri/Cargo.toml
git commit -m "chore(deps): upgrade llama-cpp-4 from 0.2 to 0.3 with vulkan and openmp features"
```

---

### Task 2: 删除 embedder.rs 中的 GGML_METAL_DISABLE 代码

**Files:**
- Modify: `src-tauri/src/services/embedder.rs`

- [ ] **Step 1: 查看需要删除的代码位置**

使用 Grep 搜索 GGML_METAL_DISABLE 相关代码：

```bash
grep -n "GGML_METAL_DISABLE" src-tauri/src/services/embedder.rs
```

预期输出应包含：
- 第 218 行：`std::env::set_var("GGML_METAL_DISABLE", "1");`
- 第 227 行：`let current_val = std::env::var("GGML_METAL_DISABLE")...`
- 第 228 行：`assert_eq!(current_val, "1", ...);`

- [ ] **Step 2: 删除 GGML_METAL_DISABLE 相关代码**

在 `embedder.rs` 文件中，删除以下代码块（大约在第 210-230 行）：

```rust
// 删除这段代码：
#[cfg(target_os = "macos")]
{
    std::env::set_var("GGML_METAL_DISABLE", "1");
    println!("[Embedder] macOS detected, disabling Metal for CPU-only embedding");
    let backend = LlamaBackend::init()
        .map_err(|e| AppError::Internal(format!("Failed to initialize llama backend: {}", e)))?;
    let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
    let current_val = std::env::var("GGML_METAL_DISABLE").unwrap_or_default();
    assert_eq!(current_val, "1", "GGML_METAL_DISABLE was leaked/removed!");
}
```

并替换为简单的后端初始化：

```rust
let backend = LlamaBackend::init()
    .map_err(|e| AppError::Internal(format!("Failed to initialize llama backend: {}", e)))?;
let _guard = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap();
```

- [ ] **Step 3: 验证删除后代码能编译**

```bash
cd src-tauri && cargo check 2>&1 | head -50
```

- [ ] **Step 4: 提交改动**

```bash
git add src-tauri/src/services/embedder.rs
git commit -m "refactor(embedder): remove GGML_METAL_DISABLE workaround, trust llama.cpp backend selection"
```

---

### Task 3: 删除 lib.rs 中的 is_m_series_mac() 函数

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 查看 is_m_series_mac() 函数位置**

使用 Grep 搜索：

```bash
grep -n "is_m_series_mac" src-tauri/src/lib.rs
```

预期输出：
- 第 157 行：`#[cfg(target_os = "macos")]`
- 第 158 行：`pub fn is_m_series_mac() -> bool {`

- [ ] **Step 2: 删除 is_m_series_mac() 函数**

删除第 157-173 行的整个函数：

```rust
#[cfg(target_os = "macos")]
pub fn is_m_series_mac() -> bool {
    static IS_M: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *IS_M.get_or_init(|| {
        std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .map(|output| {
                let brand = String::from_utf8_lossy(&output.stdout).to_lowercase();
                brand.contains("apple")
                    || brand.contains("m1")
                    || brand.contains("m2")
                    || brand.contains("m3")
            })
            .unwrap_or(false)
    })
}
```

- [ ] **Step 3: 验证删除后代码能编译**

```bash
cd src-tauri && cargo check 2>&1 | head -50
```

- [ ] **Step 4: 提交改动**

```bash
git add src-tauri/src/lib.rs
git commit -m "refactor(lib): remove is_m_series_mac() function, no longer needed with llama.cpp 0.3"
```

---

### Task 4: 删除 llama_adapter.rs 中的 macOS GPU 层判断逻辑

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 查看需要删除的代码位置**

使用 Grep 搜索：

```bash
grep -n "is_m_series_mac\|target_os = \"macos\"" src-tauri/src/services/inference/llama_adapter.rs
```

预期输出应包含：
- 第 63 行：`#[cfg(target_os = "macos")]`
- 第 65 行：`if crate::is_m_series_mac() {`
- 第 802 行：`#[cfg(target_os = "macos")]`
- 第 804 行：`if crate::is_m_series_mac() {`

- [ ] **Step 2: 修改 get_gpu_layers 函数**

在第 58-78 行的 `get_gpu_layers` 函数中，删除 macOS 特定的判断逻辑：

```rust
// 修改前（第 58-78 行）：
fn get_gpu_layers(num_gpu_setting: i32) -> u32 {
    if num_gpu_setting <= 0 {
        return 0;
    }

    #[cfg(target_os = "macos")]
    {
        if crate::is_m_series_mac() {
            println!("[GPU] Mac M-series detected, enabling Metal acceleration with {} layers", num_gpu_setting);
            return num_gpu_setting as u32;
        } else {
            println!("[GPU] Intel Mac detected. Metal backend is disabled for stability. Falling back to CPU.");
            return 0;
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        num_gpu_setting as u32
    }
}

// 修改后：
fn get_gpu_layers(num_gpu_setting: i32) -> u32 {
    if num_gpu_setting <= 0 {
        return 0;
    }
    num_gpu_setting as u32
}
```

- [ ] **Step 3: 修改测试代码（第 796-816 行）**

删除 `test_intel_mac_gpu_layers_override` 测试，因为不再需要。

- [ ] **Step 4: 验证删除后代码能编译**

```bash
cd src-tauri && cargo check 2>&1 | head -50
```

- [ ] **Step 5: 提交改动**

```bash
git add src-tauri/src/services/inference/llama_adapter.rs
git commit -m "refactor(llama_adapter): remove macOS GPU layer override logic, trust llama.cpp backend selection"
```

---

### Task 5: 删除 vision_adapter.rs 中的 macOS GPU 层判断逻辑

**Files:**
- Modify: `src-tauri/src/services/inference/vision_adapter.rs`

- [ ] **Step 1: 查看需要删除的代码位置**

使用 Grep 搜索：

```bash
grep -n "is_m_series_mac\|target_os = \"macos\"" src-tauri/src/services/inference/vision_adapter.rs
```

预期输出应包含：
- 第 68 行：`#[cfg(target_os = "macos")]`
- 第 70 行：`if !crate::is_m_series_mac() {`

- [ ] **Step 2: 删除 macOS GPU 层判断逻辑**

找到包含 `is_m_series_mac` 的代码块（大约在第 60-80 行），删除 macOS 特定的判断：

```rust
// 删除类似这样的代码块：
#[cfg(target_os = "macos")]
{
    if !crate::is_m_series_mac() {
        println!("[Vision] Intel Mac detected. Metal backend is disabled for stability. Falling back to CPU.");
        gpu_layers = 0;
    }
}
```

- [ ] **Step 3: 验证删除后代码能编译**

```bash
cd src-tauri && cargo check 2>&1 | head -50
```

- [ ] **Step 4: 提交改动**

```bash
git add src-tauri/src/services/inference/vision_adapter.rs
git commit -m "refactor(vision_adapter): remove macOS GPU layer override logic, trust llama.cpp backend selection"
```

---

### Task 6: 最终验证

**Files:**
- Test: `src-tauri/src/services/llama_backend.rs`

- [ ] **Step 1: 运行完整的 cargo check**

```bash
cd src-tauri && cargo check 2>&1
```

确保没有任何编译错误。

- [ ] **Step 2: 检查 Cargo.lock 是否更新**

```bash
grep -A 2 'name = "llama-cpp-4"' src-tauri/Cargo.lock
```

确保版本已更新到 0.3.x。

- [ ] **Step 3: 运行现有测试**

```bash
cd src-tauri && cargo test 2>&1 | tail -30
```

- [ ] **Step 4: 提交所有改动**

```bash
git add -A
git commit -m "feat: upgrade llama-cpp-4 to 0.3 with multi-backend support (metal, vulkan, openmp)"
```

---

## 预期结果

完成后：
1. `llama-cpp-4` 版本从 0.2.x 升级到 0.3.x
2. 启用了 metal, vulkan, openmp 三个后端
3. 删除了所有 macOS 特定的后端覆盖代码
4. llama.cpp 将在运行时自动检测硬件并选择最佳后端
5. 所有代码能正常编译，测试通过

## 回滚方案

如遇问题，回滚命令：

```bash
git reset --hard HEAD~1
git revert <commit_hash>
```
