use llama_cpp_4::llama_backend::LlamaBackend;
use std::sync::{Arc, LazyLock, Mutex as StdMutex};

pub static GLOBAL_BACKEND: LazyLock<Arc<LlamaBackend>> = LazyLock::new(|| {
    let backend = LlamaBackend::init().expect("Failed to initialize llama backend");
    Arc::new(backend)
});

// 全局同步互斥锁，用于同步所有 llama.cpp 底层操作
pub static ACQUIRE_LOCK: StdMutex<()> = StdMutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquire_lock() {
        let lock = ACQUIRE_LOCK.lock();
        assert!(lock.is_ok());
    }
}
