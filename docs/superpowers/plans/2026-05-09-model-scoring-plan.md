# 模型评分权重优化实现计划

**Goal:** 优化模型推荐评分权重，更准确推荐适合硬件的模型

---

## Task 1: 应用权重调整

**File:** `src-tauri/src/services/model_hub/recommender.rs`

**Changes:**
1. 调整 `fit_score`: VRAM +0.07, RAM +0.04
2. 优化 `target_chat_params`: 更保守的内存估算
3. 调整 `quant_score`: 扩大差异，Q4_K_M = 0.12
4. 调整 `disk_penalty`: 从 0.25 降至 0.15
5. 调整 `quality_score`: 各类型 +0.02

## Task 2: 构建验证

Run `cargo check` and `cargo build --release`

---

## Execute

Ready to implement?
