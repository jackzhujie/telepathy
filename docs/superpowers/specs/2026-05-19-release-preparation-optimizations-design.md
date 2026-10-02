# 设计规格：发布前夕的健壮性与性能优化 (Release Preparation Optimizations)

**日期**: 2026-05-19  
**模块**: 模型市场、向量检索 (HNSW)、多模态解析 (Vision Ingestion)  
**状态**: 提案已批准  

---

## 1. 背景与优化原则 (Background & Principles)

在 Telepathy 应用即将发布前，需要对当前的代码细节、容错策略和潜在的并发风险进行排查。本次优化旨在不改动核心业务结构的前提下，通过针对性的局部微调，显著提升系统的运行性能、多线程安全以及异常情况下的用户体验。

优化围绕四个方面展开：
1. **网络与接口效率**：避免在频繁的云端模型列表变体解析中做重复的正则表达式编译。
2. **多线程安全 (Rust Clippy)**：消除 `HnswIndex` 未声明 `Send` / `Sync` 的静态分析安全警示，规范路径类型参数。
3. **视觉解析 PDF 的健壮度（容错）**：若 PDF 页面渲染（Sidecar）报错，自动退回到纯文本模式解析，而不是直接抛出中断异常。
4. **图片 OCR 精度与细节**：使用质量更高的 CatmullRom 插值算法对大图进行预压缩，提供更精确 of OCR 文字信息。

---

## 2. 优化方案设计 (Detailed Design)

### 2.1 优化 1：HF/MS 变体列表正则表达式编译外部化
在解析 HuggingFace 和 ModelScope 变体列表时，为过滤分片 GGUF 文件（如 `*-00001-of-00005.gguf`），原代码在循环体内部反复构建 `regex::Regex`。

#### 修改文件：
* `src-tauri/src/services/model_hub/hf.rs`
* `src-tauri/src/services/model_hub/ms.rs`

#### 设计细节：
在 `get_variants` 函数的 `for item in json` / `for item in files` 循环外部提前编译好 `split_re`。
```rust
let split_re = regex::Regex::new(r"(?i)(-\d{5}-of-\d{5}|-split-|part\d+|shard\d+)")
    .map_err(|e| AppError::Internal(format!("Regex compilation failed: {}", e)))?;
```
在循环内使用 `split_re.is_match(&lower_path)` 进行匹配判定。

---

### 2.2 优化 2：`HnswIndex` 的并发安全声明与 Clippy 警告清理
`HnswIndex` 内部有来自 C++ `usearch` 绑定的 `cxx::UniquePtr<Index>`。因为指针未默认实现 `Send` 和 `Sync`，导致 `Arc<HnswIndex>` 在跨线程边界时存在静态分析警告。

#### 修改文件：
* `src-tauri/src/services/hnsw_index.rs`

#### 设计细节：
1. 显式添加并发安全 trait 的空实现：
   ```rust
   // SAFETY: HnswIndex bindings are thread-safe for search, and writes are mutually exclusive at manager level.
   unsafe impl Send for HnswIndex {}
   unsafe impl Sync for HnswIndex {}
   ```
2. 将 `save(&self, path: &PathBuf)` 及 `load(path: &PathBuf)` 中的路径参数类型由 `&PathBuf` 变更为更通用的 `&Path`，以匹配标准库最佳实践。
3. 替换 `path.extension().map_or(false, |e| e == "usearch")` 为更为简洁直观的 `path.extension().is_some_and(|e| e == "usearch")`。
4. 在 `src-tauri/src/services/memory_service.rs` 中移除未使用的导入 `rusqlite::Connection`。

---

### 2.3 优化 3：PDF 多模态视觉解析自动容错降级
在启用“视觉模型解析文档”的前提下导入 PDF 时，如果本地高级 Sidecar 解析器因为各种不可抗拒原因（例如没有执行权限、PDF 结构严重损坏等）无法输出页面图片，需要平滑过渡到纯文本解析。

#### 修改文件：
* `src-tauri/src/services/parser/vision.rs`

#### 设计细节：
在 `parse_pdf_with_vision` 函数中，对 Sidecar 渲染方法进行拦截。若渲染失败：
1. 捕获错误并使用 `eprintln!`（或日志系统）打印错误和降级提示。
2. 清理已经生成（或未完全生成）的临时文件夹。
3. 使用 `tokio::task::spawn_blocking` 异步调度 `core::parse_pdf` 提取 PDF 纯文本，并将提取的文本作为结果返回，使流程继续完成。

```rust
let render_result = crate::services::parser::sidecar::render_pdf_pages_via_sidecar(
    app_handle,
    path,
    &temp_dir
).await;

if let Err(e) = render_result {
    eprintln!("[Vision Parser] Sidecar rendering failed: {}. Falling back to plain text PDF extraction...", e);
    let _ = std::fs::remove_dir_all(&temp_dir);
    
    // 降级使用纯文本提取
    let path_clone = path.to_path_buf();
    return tokio::task::spawn_blocking(move || crate::services::parser::core::parse_pdf(&path_clone))
        .await
        .map_err(|e| AppError::Internal(format!("Task fallback failed: {}", e)))?;
}
```

---

### 2.4 优化 4：改用 CatmullRom 滤波器以改善 OCR 图像清晰度
大尺寸图片输入在送入本地多模态模型之前，会被缩放到最大尺寸为 448 像素。目前缩放使用的是 `FilterType::Triangle`，其插值结果在低对比度时文字笔锋模糊，会导致本地模型 OCR 出错。

#### 修改文件：
* `src-tauri/src/services/parser/vision.rs`

#### 设计细节：
在 `compress_image` 函数中将 resize 采用的 `FilterType::Triangle` 调整为 `FilterType::CatmullRom`：
```rust
let resized = img.resize(max_dim, max_dim, FilterType::CatmullRom);
```
在不显著增加 CPU 消耗的前提下，`CatmullRom` 算法对于中英文字符的边缘锐度及识别能力有显著提高。

---

## 3. 验证方案 (Verification Plan)

1. **编译和静态分析校验**：
   - 运行 `cargo clippy --manifest-path src-tauri/Cargo.toml` 验证 `arc_with_non_send_sync` 警告、生命周期和 `PathBuf` 警告已完全消除。
   - 运行 `cargo build --manifest-path src-tauri/Cargo.toml` 验证后端一切编译正常。
2. **容错降级验证**：
   - 将高级 Sidecar 文件临时移除，或模拟报错；开启“文档多模态解析”并导入测试 PDF，验证后台控制台打印降级日志，且 PDF 最终仍能提取到其内嵌的文字。
3. **图像清晰度对比**：
   - 使用多模态提取带小字的复杂图表或含有表格的 JPEG，验证输出结果准确度良好，边缘未模糊导致失真。
