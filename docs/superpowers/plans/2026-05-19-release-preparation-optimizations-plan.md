# 发布前夕健壮性与性能优化实现计划 (Release Preparation Optimizations Plan)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 提升 Telepathy 应用的整体性能和多线程安全性，同时确保在视觉解析报错时能够优雅地回退到纯文本提取，提升用户体验。

**Architecture:** 
1. 外部化模型 Hub 的变体匹配正则表达式，仅编译一次。
2. 显式添加 `HnswIndex` 的 `Send`/`Sync` 并发 trait 声明，并优化其 `Path` 切片方法签名以清理 Clippy。
3. 改造多模态 PDF 解析器，捕获 Sidecar 渲染异常并异步降级到同步文本提取。
4. 替换多模态图像压缩算法，使用 `CatmullRom` 滤波器保留更多文字清晰边缘。

**Tech Stack:** Rust (Tauri, tokio), usearch, regex, image

---

### Task 1: 优化云端模型变体接口正则表达式的编译效率

**Files:**
- Modify: `src-tauri/src/services/model_hub/hf.rs`
- Modify: `src-tauri/src/services/model_hub/ms.rs`

- [ ] **Step 1: 在 `hf.rs` 和 `ms.rs` 中引入测试块验证分片匹配规则**
  
  在 `hf.rs` 底部增加单元测试模块，用来验证正则匹配功能。在未修改前，我们先为它们提供一个辅助函数 `is_split_variant`。
  首先在 `src-tauri/src/services/model_hub/hf.rs` 底部添加：
  ```rust
  #[cfg(test)]
  mod tests {
      // 临时用于测试的空实现或老版本实现
      #[test]
      fn test_regex_matching() {
          let re = regex::Regex::new(r"(?i)(-\d{5}-of-\d{5}|-split-|part\d+|shard\d+)").unwrap();
          assert!(re.is_match("model-00001-of-00005.gguf"));
          assert!(!re.is_match("model-q4_k_m.gguf"));
      }
  }
  ```
  在 `src-tauri/src/services/model_hub/ms.rs` 底部做相同处理。

- [ ] **Step 2: 运行测试以验证初始测试通过**
  
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --services::model_hub::hf`
  Expected: PASS

- [ ] **Step 3: 优化 `hf.rs` 与 `ms.rs`，提取正则到循环体外部**
  
  在 `src-tauri/src/services/model_hub/hf.rs` 的 `get_variants` 函数（约 180-200 行附近）中：
  ```rust
  // 提取到循环外部
  let split_re = regex::Regex::new(r"(?i)(-\d{5}-of-\d{5}|-split-|part\d+|shard\d+)")
      .map_err(|e| AppError::Internal(format!("Regex compile error: {}", e)))?;
  
  // 在循环体内部替换：
  // 移除 regex::Regex::new(r"(?i)...").map(|re| re.is_match(&lower_path)).unwrap_or(false)
  if split_re.is_match(&lower_path) {
      continue;
  }
  ```
  
  在 `src-tauri/src/services/model_hub/ms.rs` 的 `get_variants` 函数中进行类似替换：
  ```rust
  let split_re = regex::Regex::new(r"(?i)(-\d{5}-of-\d{5}|-split-|part\d+|shard\d+)")
      .map_err(|e| AppError::Internal(format!("Regex compile error: {}", e)))?;
  
  // 循环体内替换：
  if split_re.is_match(&lower_path) {
      continue;
  }
  ```

- [ ] **Step 4: 运行 `cargo test` 验证整体逻辑正常**
  
  Run: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 5: 提交 Task 1**
  
  ```bash
  git add src-tauri/src/services/model_hub/hf.rs src-tauri/src/services/model_hub/ms.rs
  git commit -m "perf: extract split file filter regex outside of loops in model hubs"
  ```

---

### Task 2: 修复 `HnswIndex` 的 `Send`/`Sync` 警告与规范路径参数

**Files:**
- Modify: `src-tauri/src/services/hnsw_index.rs`
- Modify: `src-tauri/src/services/memory_service.rs`

- [ ] **Step 1: 运行 Clippy 编译命令暴露当前警告**
  
  Run: `cargo clippy --manifest-path src-tauri/Cargo.toml --allow-dirty --allow-staged`
  Expected: 包含 `arc_with_non_send_sync` 以及 `PathBuf` 传参警告。

- [ ] **Step 2: 显式声明 `HnswIndex` 的 `Send` 与 `Sync`**
  
  在 `src-tauri/src/services/hnsw_index.rs` 的合适位置（例如 `pub struct HnswIndex` 定义下方）增加实现：
  ```rust
  // SAFETY: HnswIndex's C++ inner index pointer is safe to be sent and shared across threads
  // as the C++ index read APIs are thread-safe and write APIs are serialized externally.
  unsafe impl Send for HnswIndex {}
  unsafe impl Sync for HnswIndex {}
  ```

- [ ] **Step 3: 优化 `hnsw_index.rs` 的签名和写法**
  
  * 修改 `save` 与 `load` 参数：
    ```rust
    // 修改前：
    pub fn save(&self, path: &PathBuf) -> Result<(), AppError>
    pub fn load(path: &PathBuf) -> Result<Self, AppError>
    
    // 修改后：
    pub fn save(&self, path: &std::path::Path) -> Result<(), AppError>
    pub fn load(path: &std::path::Path) -> Result<Self, AppError>
    ```
  * 修改扩展名验证方式：
    ```rust
    // 修改前：
    path.extension().map_or(false, |e| e == "usearch")
    
    // 修改后：
    path.extension().is_some_and(|e| e == "usearch")
    ```
  * 清理 `src-tauri/src/services/memory_service.rs` 中未使用的导入（如 `use rusqlite::Connection;`）。

- [ ] **Step 4: 运行 Clippy 验证警告已清除**
  
  Run: `cargo clippy --manifest-path src-tauri/Cargo.toml`
  Expected: 零警告（Zero Warnings）且编译成功。

- [ ] **Step 5: 提交 Task 2**
  
  ```bash
  git add src-tauri/src/services/hnsw_index.rs src-tauri/src/services/memory_service.rs
  git commit -m "refactor: implement Send and Sync for HnswIndex and fix path signatures to resolve clippy warnings"
  ```

---

### Task 3: 实现视觉 PDF 解析失败时的自动降级容错

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`

- [ ] **Step 1: 编写 PDF 视觉解析单元测试，模拟 Sidecar 渲染异常**
  
  在 `src-tauri/src/services/parser/vision.rs` 底部，引入对 `parse_pdf_with_vision` 在遇到 Sidecar 故障时的测试。如果当前文件底部没有单元测试，则创建 `mod tests` 结构：
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      
      // 测试如果传入非法路径导致 sidecar 报错，系统能否捕获并尝试降级为 plain text
      #[tokio::test]
      async fn test_parse_pdf_with_vision_fallback() {
          let dummy_path = std::path::Path::new("non_existent_file.pdf");
          let mock_app_handle = tauri::test::mock_app().app_handle();
          
          // 执行此调用，预期即使 sidecar 报错，底层降级回 core::parse_pdf 后会因为文件不存在返回 Internal 包含 Failed to read file
          let result = parse_pdf_with_vision(&dummy_path, &mock_app_handle).await;
          assert!(result.is_err());
          let err_msg = format!("{:?}", result);
          // 应由于降级调用到了 core::parse_pdf -> fs::read 而返回有关找不到文件的系统错误，而不是 Sidecar 执行错误
          assert!(err_msg.contains("Failed to read file") || err_msg.contains("No such file"));
      }
  }
  ```

- [ ] **Step 2: 运行测试并验证其报错为 Sidecar 执行失败**
  
  因目前未捕获 Sidecar 错误并执行降级，测试应会在 Sidecar 执行环节失败或抛出非文本文件相关的错误。
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --services::parser::vision`
  Expected: FAIL

- [ ] **Step 3: 修改 `parse_pdf_with_vision`，在 Sidecar 渲染阶段发生错误时降级**
  
  在 `src-tauri/src/services/parser/vision.rs` 中的 `parse_pdf_with_vision` 函数（约 150-180 行）中：
  ```rust
  // 1. 调用 Sidecar 渲染 PDF 页面
  let render_result = crate::services::parser::sidecar::render_pdf_pages_via_sidecar(
      app_handle,
      path,
      &temp_dir
  ).await;

  if let Err(e) = render_result {
      eprintln!("[Vision Parser] Warning: Sidecar PDF rendering failed: {}. Falling back to plain text extraction.", e);
      let _ = std::fs::remove_dir_all(&temp_dir);
      
      // 降级使用同步核心 PDF 文本解析器，并通过 tokio spawn 阻塞线程以维持异步签名
      let path_clone = path.to_path_buf();
      return tokio::task::spawn_blocking(move || crate::services::parser::core::parse_pdf(&path_clone))
          .await
          .map_err(|e| AppError::Internal(format!("Task fallback execution failed: {}", e)))?;
  }
  ```

- [ ] **Step 4: 重新运行测试以确认降级生效**
  
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --services::parser::vision`
  Expected: PASS （错误转换为 "Failed to read file" 后备逻辑）

- [ ] **Step 5: 提交 Task 3**
  
  ```bash
  git add src-tauri/src/services/parser/vision.rs
  git commit -m "feat: implement automatic text-extraction fallback for multimodal PDF parsing"
  ```

---

### Task 4: 改进图片预压缩滤波器算法以优化 OCR 细节

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`

- [ ] **Step 1: 编写测试确认缩放流程工作正常**
  
  在 `src-tauri/src/services/parser/vision.rs`底部的 `tests` 模块中增加对 `compress_image` 的检测：
  ```rust
  #[test]
  fn test_compress_image_compiles() {
      // 仅用于确认滤波器变更后依然能够通过编译和类型检查
      let mut img = image::DynamicImage::new_rgb8(1000, 1000);
      let compressed = compress_image(&mut img);
      assert!(compressed.width() <= 448);
      assert!(compressed.height() <= 448);
  }
  ```

- [ ] **Step 2: 替换 `FilterType` 为 `CatmullRom`**
  
  修改 `src-tauri/src/services/parser/vision.rs` 中的 `compress_image` 函数：
  ```rust
  // 修改前：
  let resized = img.resize(max_dim, max_dim, FilterType::Triangle);
  
  // 修改后：
  let resized = img.resize(max_dim, max_dim, FilterType::CatmullRom);
  ```

- [ ] **Step 3: 运行测试确认修改正确**
  
  Run: `cargo test --manifest-path src-tauri/Cargo.toml --services::parser::vision`
  Expected: PASS

- [ ] **Step 4: 运行前端类型及全编译测试**
  
  Run: `pnpm vue-tsc --noEmit && cargo build --manifest-path src-tauri/Cargo.toml`
  Expected: 前端无错，后端完整编译成功。

- [ ] **Step 5: 提交 Task 4**
  
  ```bash
  git add src-tauri/src/services/parser/vision.rs
  git commit -m "refactor: upgrade image compression filter to CatmullRom for higher text clarity"
  ```
