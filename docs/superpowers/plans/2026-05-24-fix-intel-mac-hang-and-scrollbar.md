# 修复 Intel Mac 推理死锁与聊天界面不合理横向滚动条实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 解决非 M 系列芯片 Mac (Intel Mac) 在搭载独显时由于 Metal shader 编译死锁而无法生成 AI 回复的问题，并彻底消除 AI 会话消息列表区域出现的难看且不合理的横向滚动条。

**Architecture:** 
1. 在后端 `llama_adapter.rs` 和 `vision_adapter.rs` 中，复用公共的 `crate::is_m_series_mac` 方法。在 macOS 环境下如果是非 M 芯片（即 Intel Mac），则对 GPU 卸载层数（`n_gpu_layers`）强制重置为 `0`，使其自动降级至安全稳定的纯 CPU 推理模式，并清理 `llama_adapter.rs` 中重复定义的 `is_m_series_mac`。
2. 在前端 `Chat.vue` 中，将消息列表容器 `messageListRef` 的 `overflow` 样式限制为 `overflow-y-auto overflow-x-hidden`，确保即使虚拟滚动计算中出现微弱像素横向溢出，也绝不显示横向滚动条。

**Tech Stack:** Rust (Tauri Backend), Vue 3 + Tailwind CSS (Frontend)

---

### Task 1: 修复后端 Llama 推理与多模态视觉模型在 Intel Mac 上的 GPU 降级屏蔽

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`
- Modify: `src-tauri/src/services/inference/vision_adapter.rs`
- Test: `src-tauri/src/services/inference/llama_adapter.rs`

- [ ] **Step 1: 修改 `llama_adapter.rs` 重新实现 `get_gpu_layers` 并复用公共 `is_m_series_mac`**
  修改 `src-tauri/src/services/inference/llama_adapter.rs` 中的 `get_gpu_layers` 函数，在 macOS 平台且非 M 系列芯片下强制返回 `0`。同时删除底部局部的 `is_m_series_mac` 声明：
  ```rust
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
  ```
  删除 `llama_adapter.rs` 中的以下冗余局部代码（第 74-93 行）：
  ```rust
  #[cfg(target_os = "macos")]
  fn is_m_series_mac() -> bool {
      // ...
  }

  #[cfg(not(target_os = "macos"))]
  fn is_m_series_mac() -> bool {
      false
  }
  ```

- [ ] **Step 2: 修改 `vision_adapter.rs` 强制对 Intel Mac 屏蔽 GPU**
  修改 `src-tauri/src/services/inference/vision_adapter.rs` 中的 `load_model` 函数（约第 62 行左右），对 macOS 平台非 M 芯片强制重置为 `0`：
  ```rust
  let mut gpu_layers = if n_gpu_layers > 0 {
      n_gpu_layers as u32
  } else {
      0
  };

  #[cfg(target_os = "macos")]
  {
      if !crate::is_m_series_mac() {
          println!("[Vision] Intel Mac detected. Metal backend is disabled for stability. Falling back to CPU.");
          gpu_layers = 0;
      }
  }
  ```

- [ ] **Step 3: 编写单元测试验证降级逻辑**
  在 `src-tauri/src/services/inference/llama_adapter.rs` 的测试模块底部增加一个单元测试：
  ```rust
  #[test]
  fn test_intel_mac_gpu_layers_override() {
      // 模拟设置 GPU layers 为 10
      let original_layers = 10;
      let target_layers = get_gpu_layers(original_layers);
      
      #[cfg(target_os = "macos")]
      {
          if crate::is_m_series_mac() {
              assert_eq!(target_layers, original_layers as u32);
          } else {
              // Intel Mac 必须强行降级为 0 层
              assert_eq!(target_layers, 0);
          }
      }
      
      #[cfg(not(target_os = "macos"))]
      {
          assert_eq!(target_layers, original_layers as u32);
      }
  }
  ```

- [ ] **Step 4: 运行 Cargo 编译和测试验证**
  运行命令编译项目并执行推理适配器的单元测试：
  `cargo test --manifest-path src-tauri/Cargo.toml --lib services::inference::llama_adapter::tests`
  验证所有测试绿灯通过 (PASS)。

- [ ] **Step 5: 提交后端变更**
  ```bash
  git add src-tauri/src/services/inference/llama_adapter.rs src-tauri/src/services/inference/vision_adapter.rs
  git commit -m "fix: disable Metal acceleration on Intel Mac to prevent inference hang"
  ```

---

### Task 2: 解决前端聊天会话界面不合理的横向滚动条

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: 在 `messageListRef` 元素上配置 `overflow-x-hidden`**
  修改 `src/views/Chat.vue` 的 template 中 `messageListRef` 的属性：
  原代码（第 264 行）：
  ```vue
  <!-- Messages -->
  <div ref="messageListRef" class="flex-1 overflow-y-auto py-3 relative scroll-smooth" @scroll="handleScroll">
  ```
  修改为：
  ```vue
  <!-- Messages -->
  <div ref="messageListRef" class="flex-1 overflow-y-auto overflow-x-hidden py-3 relative scroll-smooth" @scroll="handleScroll">
  ```

- [ ] **Step 2: 运行前端类型检查**
  运行：`npx tsc --noEmit` 或 `pnpm run build` 以确保前端代码没有异常或 TypeScript 错误。

- [ ] **Step 3: 提交前端变更**
  ```bash
  git add src/views/Chat.vue
  git commit -m "style: add overflow-x-hidden to Chat messages container to eliminate horizontal scrollbar"
  ```
