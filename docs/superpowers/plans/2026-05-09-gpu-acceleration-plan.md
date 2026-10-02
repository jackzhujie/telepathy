# GPU 加速智能检测实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `subagent-driven-development` or `executing-plans` to implement this plan.

**Goal:** 实现 GPU 加速的智能检测，Mac M 系列自动启用 Metal

**Architecture:** 在 llama_adapter.rs 中添加 get_gpu_layers 函数，根据操作系统和硬件智能判断是否启用 GPU

---

## 任务分解

### Task 1: 添加 GPU 检测函数

**Files:**
- Modify: `src-tauri/src/services/inference/llama_adapter.rs`

**Steps:**
1. 添加 `is_m_series_mac()` 函数检测 Apple Silicon
2. 添加 `get_gpu_layers()` 函数实现智能判断
3. 替换硬编码的 `n_gpu_layers = 0`

### Task 2: 构建验证

**Files:**
- (无修改)

**Steps:**
1. cargo check
2. cargo build --release

---

## 执行方式选择

**Plan complete and saved.**

1. **Subagent-Driven** - 每个任务派发子 agent
2. **Inline Execution** - 在当前会话执行

Which approach?
