# 设计文档：增加全局同步锁

## 1. 背景与需求
由于大语言模型、视觉模型及向量索引对 llama.cpp 底层库的并发访问可能引发崩溃或状态冲突，需要在 `llama_backend.rs` 中定义一个全局的 `ACQUIRE_LOCK`，用以同步所有并发的底层 llama.cpp 操作。

## 2. 方案比较

### 方案 A：使用 `std::sync::Mutex`（推荐）
在 `src-tauri/src/services/llama_backend.rs` 中使用标准库的 `Mutex` 提供同步互斥锁：
```rust
pub static ACQUIRE_LOCK: StdMutex<()> = StdMutex::new(());
```
- **优点**：
  - 标准库自带，无需引入第三方依赖。
  - 在同步代码块中直接使用 `lock()`，简单直观，没有异步 `.await` 开销。
  - 适合用于协调 llama.cpp 这类同步 C 绑定库的调用。
- **缺点**：
  - 如果在异步上下文中长时间持有锁，可能会阻塞当前 Tokio 线程。但这里只作为短时间的关键操作同步锁，影响可控。

### 方案 B：使用 `tokio::sync::Mutex`
- **优点**：
  - 异步锁，不会阻塞 Tokio 工作线程。
- **缺点**：
  - 必须在异步函数中使用 `.lock().await`。由于底层的很多 llama.cpp 操作是同步调用的（例如在外部线程中），或者需要跨越同步-异步边界，异步锁会让同步代码部分的调用变得非常复杂或无法使用。

### 方案 C：使用 `parking_lot::Mutex`
- **优点**：
  - 拥有更好的并发性能，防死锁检测等。
- **缺点**：
  - 引入了额外的第三方库依赖。

**结论**：选择 **方案 A**，使用标准库 `std::sync::Mutex`，符合 Task 1 规格要求。

## 3. 设计实现
在 `src-tauri/src/services/llama_backend.rs` 中引入并定义：

```rust
use llama_cpp_4::llama_backend::LlamaBackend;
use std::sync::{Arc, LazyLock, Mutex as StdMutex};

pub static GLOBAL_BACKEND: LazyLock<Arc<LlamaBackend>> = LazyLock::new(|| {
    let backend = LlamaBackend::init().expect("Failed to initialize llama backend");
    Arc::new(backend)
});

// 全局同步互斥锁，用于同步所有 llama.cpp 底层操作
pub static ACQUIRE_LOCK: StdMutex<()> = StdMutex::new(());
```

## 4. 验证与测试
- 运行 `cargo check` 确保编译通过。
- 该锁目前仅作为定义引入，不对现有业务逻辑造成直接破坏，将在后续任务中具体使用。
