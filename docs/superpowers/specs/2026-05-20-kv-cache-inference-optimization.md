# 本地大模型推理性能优化设计规格说明书 - KV Cache 复用与智能参数调度

## 1. 目标与背景

Telepathy 是一个本地 AI 知识库桌面应用。对于桌面级 AI 产品，用户侧的物理设备硬件配置差异巨大。当前项目存在两个主要的推理性能瓶颈：
1. **多轮对话下的 O(N^2) 首字延迟**：目前每次调用对话生成都会创建全新的 context 并从头对所有历史 Prompt 进行 Prefill，导致首字延迟随着对话轮数显著增加。
2. **GPU 硬件加速与多线程参数的不协调**：在 Metal/GPU 模式下，分配较多 CPU 线程会引起严重的 CPU-GPU 资源同步开销和总线带宽竞争，反而降低了推理吞吐。

本规格说明书旨在设计一套适用于**跨平台桌面产品**的优化方案，实现单会话内 KV Cache 的智能复用（Prefix Pruning），降低首字延迟（TTFT），并智能优化多平台线程与硬件调度参数。

---

## 2. 核心方案设计 (Approach 1: 最大公共前缀无损裁剪)

我们选择 **Approach 1** 作为核心设计：**基于最大公共前缀匹配的无损裁剪 (Prefix Pruning)**。这不仅在常规多轮对话中能够保留缓存，甚至在用户修改历史消息或点击“重新生成回答”时，也能够仅针对修改点后的差异部分进行 decode，最大程度复用已有的 KV 缓存。

### 2.1 架构与自引用生命周期擦除

`llama-cpp-4` 的 `LlamaContext<'model>` 持有对 `LlamaModel` 的生命周期引用。在 Rust 中将这两者存放在同一个 Adapter 中会产生“自引用”编译限制。

我们设计一个包装结构 `ActiveContext`。通过 `std::mem::transmute` 强制将生命周期标记擦除为 `'static`：

```rust
use std::sync::Arc;
use llama_cpp_4::model::LlamaModel;
use llama_cpp_4::context::LlamaContext;
use llama_cpp_4::token::LlamaToken;

pub struct ActiveContext {
    /// 强引用持有 model，保障 context 内部的底层借用在 model 存活期内绝对有效
    model: Arc<LlamaModel>,
    /// 擦除生命周期后的静态 LlamaContext 实例
    context: LlamaContext<'static>,
    /// 缓存历史中已成功 decode 并保存在 KV 缓存中的 tokens
    history_tokens: Vec<LlamaToken>,
}

// 锁机制保障同一时刻仅单个线程在 spawn_blocking 中操作 Context
unsafe impl Send for ActiveContext {}
unsafe impl Sync for ActiveContext {}
```

在 `LlamaCppAdapter` 中，我们将 `ActiveContext` 放入 `tokio::sync::Mutex` 锁中，以便跨线程安全流转：

```rust
pub struct LlamaCppAdapter {
    pub is_loaded: bool,
    backend: Option<Arc<llama_cpp_4::llama_backend::LlamaBackend>>,
    model: Option<Arc<LlamaModel>>,
    model_path: Option<std::path::PathBuf>,
    context_size: u32,
    n_threads: i32,
    /// 全局唯一的活跃 Context，管理单会话 KV 缓存状态
    active_context: Arc<tokio::sync::Mutex<Option<ActiveContext>>>,
}
```

---

## 3. 核心裁剪算法 (Prefix Matching & KV Pruning)

在每次调用 `stream_chat` 时，系统将执行以下增量算法：

### 3.1 算法步骤与伪代码

1. **Token 序列转换与比对**：
   将包含 System 提示词和对话历史的新消息组装为 prompt，通过 `model.str_to_token` 转换为 `new_tokens: Vec<LlamaToken>`。
2. **前缀比对**：
   若 `active_context` 为 `Some`，找出 `new_tokens` 与 `history_tokens` 的最大公共前缀长度 $L$：
   ```rust
   let mut L = 0;
   for (t_new, t_hist) in new_tokens.iter().zip(history_tokens.iter()) {
       if t_new == t_hist {
           L += 1;
       } else {
           break;
       }
   }
   ```
3. **KV 裁剪与增量 Decode 路由**：
   * **追加模式 ($L == history\_tokens.len()$)**：
     不清理任何缓存，仅 decode 从索引 $L$ 开始到末尾的 `&new_tokens[L..]`，首字瞬间响应。
   * **回滚/修改历史模式 ($0 < L < history\_tokens.len()$)**：
     调用 `ctx.clear_kv_cache_seq(Some(0), Some(L as u32), None)` 清理 sequence 0 在 $L$ 位置以后的所有 KV 缓存。
     将 `history_tokens` 截断为前 $L$ 项：`history_tokens.truncate(L);`
     增量 decode `&new_tokens[L..]`。
   * **无前缀/切换 Chat 模式 ($L == 0$)**：
     调用 `ctx.clear_kv_cache()` 清理全部缓存，或重建 Context 实例。
     清空 `history_tokens`，完整重新 decode 所有 `new_tokens`。

---

## 4. 跨平台多线程与 GPU 参数调度设计

### 4.1 动态线程（n_threads）调度
* **GPU 加速启用下** (`n_gpu_layers > 0`)：
  底层的计算被卸载至显卡（如 Apple Silicon 的 Metal，或者 Windows/Linux 的 CUDA）。为了避免 CPU 线程在同步时产生开销和对统一内存总线带来带宽竞争，我们**强制将 CPU 推理线程数 `n_threads` 设为 `1`**（或最大限制为 `2`）。
* **纯 CPU 推理模式** (`n_gpu_layers == 0`)：
  调用 `sysinfo` 检测物理核心数，线程数设置为可用物理核心数（通常在 4 到 6 之间，最大限制为 8），避免因开启超线程而导致的缓存命中率下降。

### 4.2 Flash Attention 与量化 KV Cache
* **Flash Attention**：
  默认在 `LlamaContextParams` 中开启 `with_flash_attention(true)`。如果用户的陈旧显卡产生兼容性报错，在产品的 settings 界面提供手动关闭选项（UI 重建开关），提高产品的平台普适性。
* **KV Cache 压缩量化**：
  提供量化 KV cache 配置参数（如 Q8_0, Q4_0），支持在小显存/小内存设备上运行，显著降低大上下文下的显存消耗。

---

## 5. 异常安全防护设计

1. **接口有序释放**：
   当调用 `LlamaCppAdapter::unload` 时，重置 `active_context` 为 `None`，这会主动触发 `ActiveContext` 的 `Drop` 并调用底层的 `llama_free` 销毁 Context，再释放 Model 计数。
2. **并发生成防御**：
   如果在 AI 仍在生成时用户触发了“重新生成”，将先通过 `CancellationToken` 取消上一次的线程，然后再获取 `active_context` 的互斥锁，保证绝对不会有两个推理线程并发读写同一个底层 C 指针。
3. **容量极限与 OOM 自愈**：
   * 在 decode 前做物理边界判定。若 `tokens.len() + sampling.max_tokens > context_size`，则对当前 context 强行触发 `clear_kv_cache()`，将缓存回滚清空，只 decode 最新对话消息，防止溢出。
   * 一旦在 decode 时捕获 `DecodeError`，自动解开 context 的持有状态，强行重置为 `None` 自愈，防止死锁与卡顿。
