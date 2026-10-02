# Vision 模型图片对话支持 — 设计规格

**日期**: 2026-05-16  
**状态**: 已批准  
**范围**: 推理引擎升级 + Vision 推理管线 + 前端设置

---

## 1. 目标

让 Telepathy 支持多模态 Vision 模型（如 LLaVA、Qwen2-VL），用户上传图片后由模型直接理解图片内容并回答问题。采用业界标准做法：一个多模态模型同时处理图片+文字，不走两阶段管道。

## 2. 技术决策

### 2.1 推理引擎升级：`llama-cpp-2` → `llama-cpp-4`

**原因**：当前 `llama-cpp-2` (v0.1) 的 FFI bindings 不暴露任何 llava/clip/image_embed 函数，无法实现 vision 推理。`llama-cpp-4` (v0.2) 提供 `mtmd` feature，原生支持多模态推理。

**迁移风险**：两个 crate 同源（同一维护者 eugenehp），核心类型命名一致（`LlamaBackend`, `LlamaModel`, `LlamaContext`, `LlamaBatch`, `LlamaSampler`）。迁移以机械性 API 适配为主。

**依赖变更**：
```toml
# 移除
llama-cpp-2 = { version = "0.1" }

# 新增
llama-cpp-4 = { version = "0.2", features = ["mtmd", "metal"] }
```

### 2.2 Vision 推理 vs 文本推理：独立适配器

Vision 推理流程和纯文本推理完全不同：

- **纯文本**：`tokenize text → batch_add → decode → sample` 循环
- **Vision**：`MtmdBitmap + MtmdInputText → MtmdContext.tokenize → eval_chunks → sample` 循环

因此不强行统一到同一个 `InferenceEngine` trait，而是新建独立的 `VisionAdapter`。

### 2.3 模型管理：双文件模型

Vision 模型由两个 GGUF 文件组成：
- **主模型**：语言模型权重（如 `llava-v1.6-Q4_K_M.gguf`）
- **mmproj**：视觉投影器（如 `llava-v1.6-mmproj-f16.gguf`）

用户手动配置两个文件路径，不做自动匹配。

## 3. 架构设计

### 3.1 数据流

```
用户上传图片+文字
    → 前端: readAsDataURL → base64 data URL
    → IPC: rag_query(query, images: Vec<String>)
    → 后端: rag_query 命令
        ├── 有图片 + 有 vision 模型配置
        │   → 解码 base64 data URL → raw bytes
        │   → 加载 VisionAdapter (主模型 + mmproj)
        │   → MtmdBitmap::from_bytes(raw_bytes)
        │   → MtmdContext::tokenize(text + bitmaps)
        │   → eval_chunks → sample 循环
        │   → 流式 token 输出 (chat-token 事件)
        │
        ├── 有图片 + 无 vision 模型
        │   → 返回 Err("请配置 Vision 模型")
        │
        └── 无图片
            → 现有 LlamaCppAdapter 纯文本推理 (不变)
```

### 3.2 文件结构变更

```
src-tauri/src/services/inference/
├── mod.rs                    # InferenceEngine trait (不变)
├── llama_adapter.rs          # 纯文本推理 (llama-cpp-2 → llama-cpp-4 API 适配)
├── vision_adapter.rs         # 【新增】Vision 推理适配器
└── template.rs               # Chat 模板 (不变)

src-tauri/src/services/
├── llama_backend.rs          # GLOBAL_BACKEND (llama-cpp-2 → llama-cpp-4 适配)
└── parser/vision.rs          # 【重写】移除 Fallback 占位，实现真正的 vision 调用
```

### 3.3 VisionAdapter 设计

```rust
pub struct VisionAdapter {
    model: Arc<LlamaModel>,
    mtmd_ctx: MtmdContext,
    model_path: PathBuf,
    mmproj_path: PathBuf,
    context_size: u32,
    n_threads: i32,
    is_loaded: bool,
}

impl VisionAdapter {
    /// 加载主模型 + mmproj
    pub async fn load(
        model_path: &Path,
        mmproj_path: &Path,
        options: ModelLoadOptions,
    ) -> Result<Self, AppError>;

    /// 流式 vision 推理
    pub async fn stream_vision_chat(
        &self,
        query: &str,
        image_bytes: Vec<Vec<u8>>,
        history: Vec<Message>,
        sampling: SamplingConfig,
        tx: mpsc::Sender<String>,
        cancel: Arc<CancellationToken>,
    ) -> Result<(), AppError>;

    pub fn unload(&mut self);
}
```

**核心推理流程**（基于 llama-cpp-4 mtmd 示例）：

1. 将 base64 data URL 解码为 raw bytes
2. `MtmdBitmap::from_bytes(&mtmd_ctx, &bytes)` 创建图片位图
3. 构造 prompt（包含 `<__media__>` 标记作为图片占位符）
4. `MtmdInputText::new(&prompt, true, true)` 创建文本输入
5. `mtmd_ctx.tokenize(&input_text, &bitmaps, &mut chunks)` 分词
6. `mtmd_ctx.eval_chunks(ctx, &chunks, ...)` 编码所有 chunks
7. Sample 循环：`get_logits → pick token → decode → send to channel`

### 3.4 设置项

| Key | 说明 | 示例值 |
|-----|------|--------|
| `vision_model` | Vision 主模型文件名 | `llava-v1.6-Q4_K_M.gguf` |
| `vision_mmproj` | mmproj 文件名 | `llava-v1.6-mmproj-f16.gguf` |

两个文件都存放在 `{app_data_dir}/downloads/` 目录下。

### 3.5 RAG 命令层修改

`rag_query` 函数中的图片处理逻辑：

```rust
// 简化后的逻辑
if has_images {
    let vision_model = get_setting("vision_model")?;
    let vision_mmproj = get_setting("vision_mmproj")?;
    
    if vision_model.is_empty() || vision_mmproj.is_empty() {
        return Err("请先在设置中配置 Vision 模型");
    }
    
    // 解码 base64 图片
    let image_bytes = decode_data_urls(&images)?;
    
    // 加载 VisionAdapter (如果未加载或模型变了)
    let adapter = load_or_reuse_vision_adapter(&vision_model, &vision_mmproj)?;
    
    // 流式推理
    adapter.stream_vision_chat(query, image_bytes, history, sampling, tx, cancel).await?;
} else {
    // 现有纯文本流程不变
}
```

### 3.6 前端修改

**设置页面**：在模型配置区域增加 Vision 模型选择：
- Vision 主模型下拉框（从已安装模型中筛选 `ModelType::Vision`）
- mmproj 文件选择（下拉框或文件选择器）

**聊天页面**：基本不变——图片上传、base64 转换、预览等已就绪。移除 `chat-images-ignored` 事件的监听和 toast 提示（不再需要 fallback 逻辑）。

## 4. 清理项

- 删除 `services/parser/vision.rs` 中的 `ImageProcessingResult::Fallback` 占位逻辑
- 删除 `rag_query` 中的 `is_image_only_prompt` 检查和 `chat-images-ignored` 事件
- 删除前端 `chat.ts` 中的 `imagesIgnoredNotice` 和 `chat-images-ignored` 监听器

## 5. 不做的事情

- ❌ 不做两阶段管道（先 OCR/描述再喂 chat 模型）
- ❌ 不做 chat + vision 模型同时加载（内存受限，一次一个引擎）
- ❌ 不做自动 mmproj 匹配（用户手动配置）
- ❌ 不做音频多模态支持（仅图片）

## 6. 测试策略

- **单元测试**：`VisionAdapter` 的 base64 解码、模型路径验证
- **集成测试**：加载一个小型 vision 模型（如 moondream2），发送测试图片，验证输出非乱码
- **手动测试**：在 Tauri dev 环境中上传图片并对话，验证端到端流程
