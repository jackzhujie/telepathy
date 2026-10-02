# 同时下载多模型与向量化服务 (Embedder) 闪退修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 Embedding 服务在处理多文档切片时因频繁创建和销毁 Metal 导致的崩溃闪退问题，并改造前端状态管理器与 UI，支持多个模型同时下载并独立显示进度与取消。

**Architecture:** 
1. 后端引入 `CachedEmbedContext` 结构，缓存 `LlamaContext` 的 `'static` 生命周期引用，并在向量化循环中通过 `clear_kv_cache()` 重用它，在 `Embedder` 销毁时加锁安全析构。
2. 前端 Pinia Store (`settings.ts`) 重构为基于 `Record<string, ModelPullProgress>` 的多任务状态管理器，采用全局静态订阅，提供计算属性向前兼容。
3. UI 界面 (`AppLayout.vue`) 改为垂直堆叠列表渲染，各自拥有独立的取消按钮，并汇总折叠状态信息。

**Tech Stack:** Rust (Tauri v2), Vue 3, Pinia, TypeScript, TailWindCSS

---

### Task 1: 向量化服务 (Embedder) 上下文缓存与释放安全化

**Files:**
- Modify: `src-tauri/src/services/embedder.rs`
- Test: `src-tauri/src/services/embedder.rs`

- [ ] **Step 1: 引入 `CachedEmbedContext` 并重构 `Embedder` 结构定义**

  修改 `src-tauri/src/services/embedder.rs`，在文件顶部或结构体上方引入 `LlamaContext`，定义安全缓存结构包装类，并手动实现 `Send` 与 `Sync`，再将 `Embedder` 内部的 `cached_model` 替换为 `cached_context`：

  ```rust
  use llama_cpp_4::context::LlamaContext;

  pub struct CachedEmbedContext {
      pub context: LlamaContext<'static>,
      pub model: Arc<LlamaModel>,
  }

  unsafe impl Send for CachedEmbedContext {}
  unsafe impl Sync for CachedEmbedContext {}

  pub struct Embedder {
      model_path: PathBuf,
      cached_context: Arc<std::sync::Mutex<Option<CachedEmbedContext>>>,
  }
  ```

- [ ] **Step 2: 重构 `Embedder::new` 的初始化**

  修改 `Embedder::new`，初始化 `cached_context` 为 `Arc::new(std::sync::Mutex::new(None))`：

  ```rust
  impl Embedder {
      pub fn new(model_path: &str) -> Self {
          Self {
              model_path: PathBuf::from(model_path),
              cached_context: Arc::new(std::sync::Mutex::new(None)),
          }
      }
  }
  ```

- [ ] **Step 3: 重构 `Embedder::embed_batch` 推理方法**

  修改 `Embedder::embed_batch`，利用 `blocking_lock`（在 `spawn_blocking` 中是同步锁）锁定 `cached_context`，如果缓存为空则在全局 `ACQUIRE_LOCK` 下加载 Model 和 Context 并 transmute 转换为 `'static` 存入缓存。在循环处理文本的开始，调用 `ctx.clear_kv_cache()` 清空缓存并重新开始 decode 流程，最后在函数结束时不销毁 context：

  ```rust
  pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
      if texts.is_empty() {
          return Ok(Vec::new());
      }

      let model_path = self.model_path.clone();
      let cached_context = self.cached_context.clone();
      let texts = texts.to_vec();

      tokio::task::spawn_blocking::<_, Result<Vec<Vec<f32>>, AppError>>(move || {
          let mut guard = cached_context.lock().unwrap_or_else(|e| e.into_inner());
          let backend = crate::services::llama_backend::GLOBAL_BACKEND.clone();

          if guard.is_none() {
              println!("[Embedder] Loading model and creating context...");
              let params = llama_cpp_4::model::params::LlamaModelParams::default();
              
              let model = {
                  let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                  LlamaModel::load_from_file(&backend, &model_path, &params)
              }.map_err(|e| AppError::Internal(format!("Failed to load embedding model: {}", e)))?;
              
              let model = Arc::new(model);

              let ctx_params = llama_cpp_4::context::params::LlamaContextParams::default()
                  .with_n_ctx(NonZeroU32::new(2048))
                  .with_embeddings(true);

              let ctx = {
                  let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                  model.new_context(&backend, ctx_params)
              }.map_err(|e| AppError::Internal(format!("Failed to create embedding context: {}", e)))?;

              let static_ctx: LlamaContext<'static> = unsafe { std::mem::transmute(ctx) };
              
              *guard = Some(CachedEmbedContext {
                  context: static_ctx,
                  model,
              });
          }

          let cached = guard.as_mut().unwrap();
          let ctx = &mut cached.context;
          let model = &cached.model;

          let mut all_embeddings = Vec::new();

          for text in texts {
              // 每次推理前必须清理上一次向量化留下的 KV Cache
              ctx.clear_kv_cache();

              let mut tokens = model
                  .str_to_token(&text, llama_cpp_4::model::AddBos::Always)
                  .map_err(|e| AppError::Internal(format!("Tokenize error: {}", e)))?;

              if tokens.len() > 2048 {
                  tokens.truncate(2048);
              }

              let n_batch = 512;
              let mut global_pos = 0i32;
              for chunk in tokens.chunks(n_batch) {
                  let mut batch = llama_cpp_4::llama_batch::LlamaBatch::new(chunk.len(), 1);
                  for token in chunk.iter() {
                      batch
                          .add(*token, global_pos, &[0], true)
                          .map_err(|e| AppError::Internal(format!("Batch add error: {}", e)))?;
                      global_pos += 1;
                  }

                  let decode_res = {
                      let _g = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                      ctx.decode(&mut batch)
                  };
                  decode_res.map_err(|e| AppError::Internal(format!("Decode error: {}", e)))?;
              }

              let mut embedding_opt = None;

              if let Ok(e) = ctx.embeddings_seq_ith(0) {
                  if !e.iter().all(|&v| v == 0.0) {
                      embedding_opt = Some(e.to_vec());
                  }
              }

              if embedding_opt.is_none() {
                  for idx in 0..tokens.len() {
                      if let Ok(e) = ctx.embeddings_ith(idx as i32) {
                          if !e.iter().all(|&v| v == 0.0) {
                              embedding_opt = Some(e.to_vec());
                              break;
                          }
                      }
                  }
              }

              if embedding_opt.is_none() {
                  if let Ok(e) = ctx.embeddings_ith(0) {
                      embedding_opt = Some(e.to_vec());
                  }
              }

              let embedding = embedding_opt
                  .ok_or_else(|| AppError::Internal("Failed to get embeddings".into()))?;

              all_embeddings.push(embedding);
          }

          Ok(all_embeddings)
      })
      .await
      .map_err(|e| AppError::Internal(format!("Blocking task panicked: {}", e)))?
  }
  ```

- [ ] **Step 4: 实现 `Drop` 特征以进行安全释放**

  在 `src-tauri/src/services/embedder.rs` 末尾实现 `Drop` 特征，确保在生命周期结束、释放 `Embedder` 实例时，于全局 ACQUIRE_LOCK 互斥锁保护下销毁 Metal 上下文：

  ```rust
  impl Drop for Embedder {
      fn drop(&mut self) {
          if let Ok(mut guard) = self.cached_context.lock() {
              if guard.is_some() {
                  let _lock = crate::services::llama_backend::ACQUIRE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                  *guard = None;
              }
          }
      }
  }
  ```

- [ ] **Step 5: 验证单元测试通过**

  在 `src-tauri` 目录下运行 Cargo 测试以确认重构后嵌入逻辑依然正确运行并且无内存释放崩溃：
  Run: `cargo test --package telepathy --lib -- services::embedder::tests::test_real_model_embedding`
  Expected: tests pass, outputs successfully.

- [ ] **Step 6: 提交代码**

  ```bash
  git add src-tauri/src/services/embedder.rs
  git commit -m "fix: refactor Embedder to cache LlamaContext and safely drop it under ACQUIRE_LOCK to prevent Metal free crash"
  ```

---

### Task 2: 前端 Pinia Store (settings.ts) 下载状态多任务化

**Files:**
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: 新增 `activeDownloads` 状态并重构 `installingModel` 与 `installProgress` 兼容字段**

  打开 `src/stores/settings.ts`，定义 `activeDownloads` 响应式对象。保留 `installingModel` 和 `installProgress` 为 Computed，使其兼容旧有单进度引用：

  ```typescript
  // 替换原有定义 (第33-34行)
  const activeDownloads = ref<Record<string, ModelPullProgress>>({});
  
  const installingModel = computed(() => {
    const keys = Object.keys(activeDownloads.value);
    return keys.length > 0 ? keys[0] : null;
  });

  const installProgress = computed(() => {
    const keys = Object.keys(activeDownloads.value);
    return keys.length > 0 ? activeDownloads.value[keys[0]] : null;
  });
  ```

- [ ] **Step 2: 注册全局多任务下载事件监听器**

  删除 `installNewModel` 内部的 `listen` 函数临时绑定（原第219-281行代码），改在 `useSettingsStore` 函数体下方、初始化时直接进行**一次性全局静态订阅**：

  ```typescript
  // 注册全局下载事件监听
  listen<any>('model-pull-progress', (event) => {
    const { message, percentage, completed = 0, total = 0, stage, model } = event.payload;
    if (!model) return;
    
    if (stage === 'cancelled') {
      delete activeDownloads.value[model];
      return;
    }

    activeDownloads.value[model] = {
      model,
      status: message || 'Downloading...',
      percentage,
      completed,
      total,
      stage
    };
  });

  listen<{ model: string; error: string }>('model-pull-error', (event) => {
    const { model, error } = event.payload;
    delete activeDownloads.value[model];
    
    pullGlobalNotification.value = {
      message: `模型 ${model} 下载失败: ${error}`,
      type: 'error',
      timestamp: Date.now()
    };
    setTimeout(() => {
      pullGlobalNotification.value = null;
    }, 5000);
  });

  listen<{ model: string }>('model-pull-done', async (event) => {
    const { model } = event.payload;
    delete activeDownloads.value[model];
    
    pullGlobalNotification.value = {
      message: `模型 ${model} 下载并安装完成！`,
      type: 'success',
      timestamp: Date.now()
    };
    setTimeout(() => {
      pullGlobalNotification.value = null;
    }, 4000);

    fetchInstalledModels();

    import('./notifications').then(({ useNotificationsStore }) => {
      const notificationsStore = useNotificationsStore();
      notificationsStore.playDebouncedSound();
    }).catch(err => console.error('播放下载声音失败:', err));
  });
  ```

- [ ] **Step 3: 优化 `installNewModel` 触发逻辑与兼容**

  修改 `installNewModel` 方法，不再在内部监听下载事件，仅初始化任务占位项并发出 Tauri 命令启动下载：

  ```typescript
  async function installNewModel(modelName: string, variant: string) {
    const finalModelId = `${modelName}:${variant}`;
    
    // 初始化占位状态
    activeDownloads.value[finalModelId] = {
      model: finalModelId,
      status: 'Preparing...',
      percentage: 0,
      completed: 0,
      total: 0,
      stage: 'preparing'
    };

    try {
      await installModelApi(modelName, variant);
    } catch (e: any) {
      console.error('[Install] Error:', e);
      delete activeDownloads.value[finalModelId];
    }
  }
  ```

- [ ] **Step 4: 升级 `cancelInstall` 精准取消**

  修改 `cancelInstall` 方法，接收可选的 `modelId` 参数。若传递则精准取消该模型，否则取消全部正在下载的模型：

  ```typescript
  async function cancelInstall(modelId?: string) {
    try {
      if (modelId) {
        delete activeDownloads.value[modelId];
        await cancelPullModelCmd(modelId);
      } else {
        const keys = Object.keys(activeDownloads.value);
        activeDownloads.value = {};
        for (const k of keys) {
          await cancelPullModelCmd(k);
        }
      }
    } catch (e) {
      console.error('[Cancel] Error:', e);
    }
  }
  ```

- [ ] **Step 5: 导出 `activeDownloads` 状态**

  在 `settings.ts` 返回语句末尾导出 `activeDownloads`：
  ```typescript
  return {
    ...
    installingModel,
    installProgress,
    activeDownloads,
    ...
  }
  ```

- [ ] **Step 6: 提交代码**

  ```bash
  git add src/stores/settings.ts
  git commit -m "feat: refactor settings store to support activeDownloads map and global pull listeners for concurrency"
  ```

---

### Task 3: 前端 UI (AppLayout.vue) 下载面板列表多任务化

**Files:**
- Modify: `src/components/layout/AppLayout.vue`

- [ ] **Step 1: 修改下载状态的显示控制逻辑**

  在 `src/components/layout/AppLayout.vue` 中，修改原先的 `v-if="settingsStore.installingModel"`，改为使用 `Object.keys(settingsStore.activeDownloads).length > 0` 来展示下载面板：

  ```html
  <!-- 替换原第 499 行 -->
  <div
    v-if="Object.keys(settingsStore.activeDownloads).length > 0"
    :style="modelPullStyle"
    class="fixed z-50 cursor-move select-none"
    :class="hasDraggedModelPull ? '' : 'bottom-24 right-6'"
    @pointerdown="onModelPullPointerDown"
    @pointermove="onModelPullPointerMove"
    @pointerup="onModelPullPointerUp"
  >
  ```

- [ ] **Step 2: 重写下载面板折叠与展开状态的渲染内容**

  重写折叠面板与展开面板部分。如果是折叠状态，计算并显示折叠文案（如 `"{单个模型名称} {进度}% 下载中"` 或 `"{数量}个模型下载中"`）。如果是展开状态，使用 `v-for` 渲染所有处于 `activeDownloads` 中的任务进度条与取消按钮：

  ```html
  <!-- 替换原第 507-561 行 -->
  <div 
    v-if="!isModelPullCollapsed"
    class="px-3 py-2 bg-panel-bg/95 backdrop-blur-xl border border-brand/40 text-text-primary rounded-xl shadow-2xl flex flex-col gap-2.5 min-w-[240px] max-h-[300px] overflow-y-auto scrollbar-hide"
  >
    <div class="flex justify-between items-center border-b border-border-main/30 pb-1.5 flex-shrink-0">
      <span class="font-bold text-brand text-[11px] flex items-center gap-1.5">
        <div class="animate-spin rounded-full h-3 w-3 border border-brand border-t-transparent"></div>
        正在下载模型 ({{ Object.keys(settingsStore.activeDownloads).length }} 个任务)
      </span>
      <button
        @click="!wasJustDraggedModelPull && (isModelPullCollapsed = true)"
        class="text-text-muted hover:text-text-primary p-0.5"
        title="收起进度条"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 15 12 9 18 15"></polyline>
        </svg>
      </button>
    </div>

    <div class="flex flex-col gap-3 overflow-y-auto">
      <div 
        v-for="(progress, modelId) in settingsStore.activeDownloads" 
        :key="modelId"
        class="flex items-center gap-2"
      >
        <div class="flex-1 min-w-0">
          <div class="flex justify-between items-center mb-0.5">
            <span class="font-semibold text-[10px] text-text-primary truncate max-w-[130px]" :title="modelId">
              {{ modelId.split(':')[0] }}
            </span>
            <span class="text-[9px] font-mono text-brand">{{ progress.percentage || 0 }}%</span>
          </div>
          <div class="text-[8px] text-brand/70 mb-1 truncate">
            {{ progress.status || '准备中...' }}
          </div>
          <div class="h-1 bg-surface-bg rounded-full overflow-hidden">
            <div
              class="h-full bg-brand rounded-full transition-all duration-300"
              :style="{ width: `${progress.percentage || 0}%` }"
            ></div>
          </div>
        </div>
        <button
          @click="!wasJustDraggedModelPull && settingsStore.cancelInstall(modelId)"
          class="text-text-muted hover:text-danger-500 p-1 transition-colors flex-shrink-0"
          title="取消下载"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>
  </div>

  <button
    v-else
    @click="!wasJustDraggedModelPull && (isModelPullCollapsed = false)"
    class="inline-flex items-center gap-2 px-3 py-1.5 bg-brand/10 backdrop-blur border border-brand/40 text-brand rounded-full shadow-2xl hover:bg-brand/20 transition-colors pointer-events-auto"
  >
    <div class="animate-spin rounded-full h-3 w-3 border border-brand border-t-transparent flex-shrink-0"></div>
    <span class="text-[10px] font-bold">
      <template v-if="Object.keys(settingsStore.activeDownloads).length === 1">
        {{ Object.values(settingsStore.activeDownloads)[0].percentage }}% 下载中
      </template>
      <template v-else>
        {{ Object.keys(settingsStore.activeDownloads).length }} 个模型下载中
      </template>
    </span>
  </button>
  ```

- [ ] **Step 3: 运行前端类型校验**

  在项目根目录下，执行 TypeScript 静态校验命令验证无编译类型异常：
  Run: `npm run build`
  Expected: Compilation and packaging succeeds without errors.

- [ ] **Step 4: 提交代码**

  ```bash
  git add src/components/layout/AppLayout.vue
  git commit -m "feat: upgrade download status bar in AppLayout.vue to display list of concurrent downloads with separate cancel controls"
  ```
