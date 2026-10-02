# 图片理解功能设计（修订版）

**日期**: 2026-05-09
**模块**: 视觉能力扩展
**状态**: 设计中

---

## 1. 设计原则

1. **不依赖外部 OCR 工具** - 纯 Rust 实现
2. **支持粘贴图片** - 前端粘贴上传
3. **支持多图片** - 一次发送多张图片
4. **使用 Vision 模型** - 用本地 LLM 理解图片

---

## 2. 功能概述

### 2.1 图片理解流程

```
用户粘贴/上传图片
        │
        ▼
┌───────────────────────┐
│   前端处理             │
│   - 图片压缩/缩放      │
│   - 转为 base64        │
│   - 多图片支持         │
└───────────────────────┘
        │
        ▼
┌───────────────────────┐
│   后端处理             │
│   - 调用 Vision 模型   │
│   - 或用文字描述图片   │
└───────────────────────┘
        │
        ▼
   返回图片内容
```

### 2.2 Vision 模型支持

使用本地 vision 模型（如 LLaVA、Qwen-VL）直接理解图片，无需 OCR：

| 模型 | 特点 |
|------|------|
| **LLaVA** | 开源 vision model，需要 mmproj |
| **Qwen-VL** | 阿里开源，效果好 |
| **BakLLaVA** | 免 mmproj 版本 |

---

## 3. 前端设计 (Reka UI)

### 3.1 图片粘贴/上传

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { Icon } from 'reka-ui'

const images = ref<File[]>([])
const previewUrls = ref<string[]>([])
const pasteHandler = (e: ClipboardEvent) => {
  const items = e.clipboardData?.items
  if (!items) return

  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile()
      if (file) {
        images.value.push(file)
        previewUrls.value.push(URL.createObjectURL(file))
      }
    }
  }
}

const removeImage = (index: number) => {
  URL.revokeObjectURL(previewUrls.value[index])
  images.value.splice(index, 1)
  previewUrls.value.splice(index, 1)
}
</script>

<template>
  <div class="image-handler">
    <!-- 图片预览区 -->
    <div v-if="previewUrls.length" class="previews">
      <div v-for="(url, i) in previewUrls" :key="i" class="preview-item">
        <img :src="url" />
        <button @click="removeImage(i)" class="remove-btn">×</button>
      </div>
    </div>

    <!-- 上传按钮 -->
    <button class="upload-btn">
      <Icon name="image" />
      <span>添加图片</span>
    </button>
  </div>
</template>
```

### 3.2 消息组件

```vue
<script setup lang="ts">
interface Message {
  id: string
  content: string
  images?: string[]  // base64 图片
  role: 'user' | 'assistant'
}
</script>

<template>
  <div class="message" :class="message.role">
    <!-- 多图片展示 -->
    <div v-if="message.images?.length" class="message-images">
      <img
        v-for="(img, i) in message.images"
        :key="i"
        :src="img"
        class="message-image"
      />
    </div>

    <!-- 文字内容 -->
    <div class="message-content">{{ message.content }}</div>
  </div>
</template>
```

### 3.3 粘贴监听

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

const onPaste = (e: ClipboardEvent) => {
  // 处理粘贴事件
  const items = e.clipboardData?.items
  // ...
}

onMounted(() => {
  document.addEventListener('paste', onPaste)
})

onUnmounted(() => {
  document.removeEventListener('paste', onPaste)
})
</script>
```

---

## 4. 后端设计

### 4.1 VisionService

```rust
// src-tauri/src/services/vision.rs

use crate::errors::AppError;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageResult {
    pub description: String,
    pub extracted_text: Option<String>,
}

pub struct VisionService;

impl VisionService {
    /// 使用 Vision 模型理解图片
    pub async fn describe_image(
        image_path: &Path,
        model_path: &str,
    ) -> Result<ImageResult, AppError> {
        // 1. 读取图片并转为 base64
        let image_data = std::fs::read(image_path)?;
        let base64_image = BASE64.encode(&image_data);

        // 2. 构建 vision prompt
        let prompt = r#"请详细描述这张图片的内容，包括：
1. 图片中的主要元素和场景
2. 文字内容（如果有，请完整转录）
3. 图片的类型和用途

请用简洁的语言描述。"#;

        // 3. 调用 vision 模型（llama.cpp multimodal）
        let description = Self::call_vision_model(
            model_path,
            prompt,
            &base64_image,
        ).await?;

        Ok(ImageResult {
            description,
            extracted_text: None, // Vision 模型直接返回文字内容
        })
    }

    /// 处理多张图片
    pub async fn describe_images(
        image_paths: &[PathBuf],
        model_path: &str,
    ) -> Result<Vec<ImageResult>, AppError> {
        let mut results = Vec::new();
        for path in image_paths {
            let result = Self::describe_image(path, model_path).await?;
            results.push(result);
        }
        Ok(results)
    }
}
```

### 4.2 对话集成

```rust
// 在 rag.rs 或 chat.rs 中

pub async fn chat_with_images(
    content: &str,
    images: &[String],  // base64 图片
    chat_model: &str,
) -> Result<String, AppError> {
    let prompt = if images.is_empty() {
        content.to_string()
    } else {
        // 构建多图片 prompt
        let image_descriptions: Vec<String> = images
            .iter()
            .enumerate()
            .map(|(i, _)| format!("[图片{}]", i + 1))
            .collect();

        format!(
            "{}\n\n用户上传了 {} 张图片：{}\n\n请根据这些图片回答用户的问题。",
            content,
            images.len(),
            image_descriptions.join(", ")
        )
    };

    // 调用对话模型
    chat(&prompt, chat_model).await
}
```

---

## 5. 模型要求

### 5.1 推荐 Vision 模型

| 模型 | 大小 | 推荐场景 |
|------|------|----------|
| **llava-1.6-mistral** | ~4GB | 通用图片理解 |
| **llava-1.6-34b** | ~20GB | 高精度 |
| **qwen-vl** | ~7GB | 中文优化 |
| **bakllava** | ~4GB | 无需 mmproj |

### 5.2 模型配置

```json
{
  "chat_model": "llava-1.6-mistral-q4_k_m.gguf",
  "vision_model": "llava-1.6-mistral-7b-mmproj.gguf"
}
```

---

## 6. 实现计划

| Phase | 内容 | 复杂度 |
|-------|------|--------|
| 1 | 前端：图片上传/粘贴/预览 | 中 |
| 2 | 前端：多图片发送 | 低 |
| 3 | 前端：消息展示图片 | 低 |
| 4 | 后端：VisionService | 高 |
| 5 | 后端：对话集成图片 | 中 |
| 6 | 测试：多图片场景 | 中 |

---

## 7. 注意事项

1. **图片大小限制**: 前端压缩或后端限制（如 10MB）
2. **Vision 模型可选**: 没有时提示用户下载
3. **回退方案**: 无 Vision 模型时，提示图片无法处理
