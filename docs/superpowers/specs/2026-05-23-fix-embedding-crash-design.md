# 设计规格文档：修复 Embedding 模型在向量化处理中的闪退问题

- **文档状态**：已批准 (Approved)
- **创建日期**：2026-05-23
- **作者**：Antigravity

---

## 1. 背景与问题描述

在 Telepathy 应用进行文档导入（PDF/Word 等）或重建索引时，后台需要调用本地 Embedding 模型将文本块转换为向量。在此过程中，程序在底层 `llama.cpp` 初始化上下文后会发生意外崩溃闪退。

在终端日志中，观察到了以下输出信息：
```
decode: cannot decode batches with this context (calling encode() instead)
```
紧接着进程异常退出，没有打印出任何 panic 或 Rust 层的错误栈。

---

## 2. 根因分析

1. **API 用途不匹配**：
   在 [embedder.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/embedder.rs) 中，我们通过以下配置将 Embedding 模型的上下文参数初始化为专用向量提取模式：
   ```rust
   let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
       .with_embeddings(true);
   ```
   但在循环处理向量化批次时，代码调用了 `ctx.decode(&mut batch)`。
   
2. **底层崩溃链条**：
   - 在 `llama.cpp` 原生层中，`llama_decode` 默认用于自回归文本生成，会强制检查当前 Context 是否包含了预测 Logits 的输出缓冲。
   - 当配置为 `with_embeddings(true)` 时，为了极限压缩资源消耗，底层不会分配该缓冲区。此时调用 `llama_decode` 会触发 `cannot decode batches with this context` 的警告，并尝试 fallback 到编码逻辑。
   - 然而，在 Rust 绑定库 `llama-cpp-4` (v0.2.43) 的 `decode` 实现中，如果 decode 成功，它会执行这一行：
     ```rust
     self.initialized_logits.clone_from(&batch.initialized_logits);
     ```
     由于在纯 Embedding 模式下不存在合法的 logits，导致拷贝时访问了空指针或越界地址，直接在底层触发了 C/C++ 层的 Segmentation fault 或 Abort，绕过了 Rust 的 panic 守护，直接强杀进程。

3. **正确接口**：
   对只进行编码的 Embedding 模型，`llama.cpp` 提供了专用的 `llama_encode` 接口。对应地，`llama-cpp-4` 暴露了：
   ```rust
   pub fn encode(&mut self, batch: &mut LlamaBatch) -> Result<(), EncodeError>
   ```

---

## 3. 设计方案

### 3.1 改造核心向量化批次提交逻辑
修改 [embedder.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/embedder.rs) 中 `Embedder::embed_batch` 内的批次执行代码。
将第 139-143 行的调用由 `ctx.decode` 替换为 `ctx.encode`。

### 3.2 错误类型转换
`ctx.encode` 失败时会返回 `llama_cpp_4::LLamaCppError::EncodeError(EncodeError)`。
我们将其映射到我们的全局错误类型 `AppError::Internal`，输出详细的编码错误描述。

**修改方案对比**：

```diff
-                     let decode_res = {
-                         let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
-                         ctx.decode(&mut batch)
-                     };
-                     decode_res.map_err(|e| AppError::Internal(format!("Decode error: {}", e)))?;
+                     let encode_res = {
+                         let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
+                         ctx.encode(&mut batch)
+                     };
+                     encode_res.map_err(|e| AppError::Internal(format!("Encode error: {}", e)))?;
```

---

## 4. 验证计划

### 4.1 自动测试验证
在 `src-tauri` 中运行为 Embedding 专门编写的原生单元测试，确保加载实际模型并进行 embedding 不再发出警告，且能正确提取出有效的向量值：
```bash
cargo test --package temp-app --lib -- services::embedder::tests::test_real_model_embedding -- --nocapture
```

### 4.2 编译检查
在前端/后端运行全面构建/类型检测，确保没有因修改引入的 API 类型不兼容或未使用的导入警告：
```bash
cargo check
```
