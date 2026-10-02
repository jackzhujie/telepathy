# 图片 OCR 实现计划

**Goal:** 实现图片文字提取 (OCR) 和内容描述功能

---

## Task 1: 修改 vision.rs 实现 OCR

**File:** `src-tauri/src/services/parser/vision.rs`

**Steps:**
1. 添加 `image` crate 依赖
2. 实现 `extract_text()` - 使用 tesseract CLI
3. 实现 `describe_image()` - 使用对话模型
4. 实现 `parse_image()` - 组合以上两者

## Task 2: 添加 Rust 依赖

**File:** `src-tauri/Cargo.toml`

**Steps:**
```toml
image = "0.25"
```

## Task 3: 构建验证

Run `cargo check` and `cargo build`

---

## Implementation

Ready to start?
