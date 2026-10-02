# 图片理解功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development for frontend tasks

**Goal:** 实现图片粘贴、上传、多图片发送和 Vision 模型理解

---

## 阶段 1: 前端 - 图片处理

### Task 1: 图片上传组件

**Files:**
- Create: `src/components/ImageHandler.vue`

**Steps:**
1. 创建图片上传/粘贴组件
2. 支持多图片选择
3. 支持 paste 粘贴
4. 图片预览和删除

### Task 2: 修改聊天输入框

**Files:**
- Modify: `src/components/ChatInput.vue` 或类似文件

**Steps:**
1. 集成 ImageHandler
2. 支持发送带图片的消息

### Task 3: 修改消息组件

**Files:**
- Modify: `src/components/Message.vue` 或类似文件

**Steps:**
1. 显示消息中的图片
2. 支持多图片布局

---

## 阶段 2: 后端 - Vision 支持

### Task 4: 实现 VisionService

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`

**Steps:**
1. 实现 describe_image 函数
2. 实现多图片处理
3. 添加 Vision 模型支持

### Task 5: 对话集成

**Files:**
- Modify: `src-tauri/src/services/rag.rs` 或相关文件

**Steps:**
1. 修改 chat 命令支持图片
2. 调用 Vision 模型理解图片

---

## 执行方式

**Frontend Tasks (1-3):** Subagent-Driven
**Backend Tasks (4-5):** 可以并行执行

Ready to start?
