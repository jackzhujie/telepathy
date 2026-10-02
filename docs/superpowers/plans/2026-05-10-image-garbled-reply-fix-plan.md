# Fix Garbled LLM Replies After Image Upload — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix garbled LLM replies when images are attached by replacing placeholder text injection with a clean `ImageProcessingResult` enum that branches between future vision processing and current text-only fallback.

**Architecture:** Introduce `ImageProcessingResult` enum in `vision.rs` with `Processed` and `Fallback` variants. `describe_images_base64` returns this enum instead of a string. `rag.rs` branches on the variant: `Fallback` + empty query emits an error (no LLM call); `Fallback` + non-empty query emits an `images-ignored` event and proceeds text-only. Frontend listens for the event and shows a dismissable notice.

**Tech Stack:** Rust (Tauri backend), TypeScript/Vue 3 (frontend)

---

## File Structure

| File | Role |
|------|------|
| `src-tauri/src/services/parser/vision.rs` | Data model + image processing functions |
| `src-tauri/src/commands/rag.rs` | Branch logic, event emission |
| `src/stores/chat.ts` | State + event listeners |
| `src/views/Chat.vue` | UI notice display |

---

### Task 1: Refactor vision.rs — Add ImageProcessingResult enum

**Files:**
- Modify: `src-tauri/src/services/parser/vision.rs`

- [ ] **Step 1: Replace `describe_image_base64` and `describe_images_base64` with new enum and function**

Replace the entire file content after the VISION_SYSTEM_PROMPT constant block:

Delete `describe_image_base64` function (lines 21-27).

Replace `describe_images_base64` (lines 29-37) with:

```rust
pub enum ImageProcessingResult {
    /// Vision model successfully processed the images (future implementation)
    #[allow(dead_code)]
    Processed { descriptions: Vec<String> },
    /// No vision model available; fallback to text-only
    Fallback { image_count: usize },
}

pub async fn describe_images_base64(images: &[String]) -> Result<ImageProcessingResult, AppError> {
    // Vision model inference is not yet implemented.
    // When it is, this function will route to a vision adapter.
    Ok(ImageProcessingResult::Fallback {
        image_count: images.len(),
    })
}
```

Remove the unused `parse_image` import path `PathBuf` if it becomes unused — keep `Path` and `PathBuf` imports since `parse_image` and `describe_images` still use them.

- [ ] **Step 2: Replace tests**

Replace the test block (lines 48-67) with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_describe_images_base64_empty() {
        let result = describe_images_base64(&[]).await;
        assert!(result.is_ok());
        match result.unwrap() {
            ImageProcessingResult::Fallback { image_count } => assert_eq!(image_count, 0),
            _ => panic!("expected Fallback"),
        }
    }

    #[tokio::test]
    async fn test_describe_images_base64_single() {
        let images = vec!["dGVzdCBpbWFnZQ==".to_string()];
        let result = describe_images_base64(&images).await;
        assert!(result.is_ok());
        match result.unwrap() {
            ImageProcessingResult::Fallback { image_count } => assert_eq!(image_count, 1),
            _ => panic!("expected Fallback"),
        }
    }

    #[tokio::test]
    async fn test_describe_images_base64_multiple() {
        let images = vec![
            "aW1hZ2Ux".to_string(),
            "aW1hZ2Uy".to_string(),
            "aW1hZ2Uz".to_string(),
        ];
        let result = describe_images_base64(&images).await;
        assert!(result.is_ok());
        match result.unwrap() {
            ImageProcessingResult::Fallback { image_count } => assert_eq!(image_count, 3),
            _ => panic!("expected Fallback"),
        }
    }
}
```

- [ ] **Step 3: Run vision tests**

Run: `cargo test -p telepathy vision -- --nocapture`
Expected: 3 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/services/parser/vision.rs
git commit -m "refactor(vision): add ImageProcessingResult enum, remove placeholder text injection"
```

---

### Task 2: Update rag.rs — Branch on ImageProcessingResult

**Files:**
- Modify: `src-tauri/src/commands/rag.rs:53-64`

- [ ] **Step 1: Replace the images handling block**

Replace lines 53-64 (the `has_images` / `final_query` / `enhanced_query` block) with:

```rust
    // Process images: branch between vision model (future) and text-only fallback
    let image_result = if let Some(ref imgs) = images {
        Some(crate::services::parser::vision::describe_images_base64(imgs).await?)
    } else {
        None
    };

    let mut final_query = query.clone();

    match image_result {
        Some(crate::services::parser::vision::ImageProcessingResult::Fallback { image_count }) => {
            if image_count > 0 && final_query.trim().is_empty() {
                // Images only, no text — emit error and return early
                let _ = app_handle.emit(
                    "chat-error",
                    json!({
                        "conversation_id": "",
                        "message_id": "",
                        "error": "请为图片添加文字描述后再发送。",
                    }),
                );
                return Ok(());
            }
            if image_count > 0 {
                let _ = app_handle.emit(
                    "chat-images-ignored",
                    json!({
                        "image_count": image_count,
                    }),
                );
            }
        }
        Some(crate::services::parser::vision::ImageProcessingResult::Processed { .. }) => {
            todo!("Vision model processing not yet implemented")
        }
        None => {}
    }
```

- [ ] **Step 2: Build Rust backend**

Run: `cargo build -p telepathy 2>&1 | tail -20`
Expected: Compilation succeeds

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/rag.rs
git commit -m "feat(rag): branch on ImageProcessingResult, emit images-ignored event"
```

---

### Task 3: Add images-ignored event listener in chat.ts

**Files:**
- Modify: `src/stores/chat.ts`

- [ ] **Step 1: Add state and event listener**

Add a new ref after the existing `error` ref on line 46:

```typescript
const imagesIgnoredNotice = ref<string | null>(null);
```

In `initListeners()`, after the `unlistenError` block (after line 123), add:

```typescript
const unlistenImagesIgnored = await listen<{ image_count: number }>('chat-images-ignored', (event) => {
  imagesIgnoredNotice.value = `当前未配置 Vision 模型，${event.payload.image_count} 张图片未被理解，仅基于文本回复。`;
  setTimeout(() => {
    imagesIgnoredNotice.value = null;
  }, 6000);
});
```

Add to the `unlisteners` array (after line 185):

```typescript
unlistenImagesIgnored,
```

Make sure the import of `imagesIgnoredNotice` is included in the return statement. Add it after the `error` export:

```typescript
imagesIgnoredNotice,
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No new errors (pre-existing errors unrelated to our changes are OK)

- [ ] **Step 3: Commit**

```bash
git add src/stores/chat.ts
git commit -m "feat(chat): listen for chat-images-ignored event and show notice"
```

---

### Task 4: Display images-ignored notice in Chat.vue

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: Add dismissable notice in template**

Add after the opening `<div class="flex-1 flex flex-col max-w-4xl...">` on line 215, before the Header section:

```html
<!-- Images ignored notice -->
<Transition name="fade">
  <div
    v-if="chatStore.imagesIgnoredNotice"
    class="mx-2 mb-2 px-3 py-2 bg-warning-500/10 border border-warning-500/30 rounded-lg text-warning-500 text-xs flex items-center justify-between gap-2"
  >
    <div class="flex items-center gap-2">
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"></circle>
        <line x1="12" y1="8" x2="12" y2="12"></line>
        <line x1="12" y1="16" x2="12.01" y2="16"></line>
      </svg>
      <span>{{ chatStore.imagesIgnoredNotice }}</span>
    </div>
    <button
      @click="chatStore.imagesIgnoredNotice = null"
      class="text-warning-500/60 hover:text-warning-500 transition-colors flex-shrink-0"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
  </div>
</Transition>
```

Note: The existing `<style scoped>` block already has `.fade-*` transition classes defined (lines 340-349), so the `<Transition name="fade">` will work without any CSS changes.

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No new errors

- [ ] **Step 3: Full build check**

Run: `cargo build -p telepathy 2>&1 | tail -5`
Expected: Compilation succeeds

- [ ] **Step 4: Commit**

```bash
git add src/views/Chat.vue
git commit -m "feat(ui): show dismissable notice when images are ignored due to missing vision model"
```

---

### Task 5: End-to-end manual verification

- [ ] **Step 1: Run the app**

```bash
cargo tauri dev
```

- [ ] **Step 2: Verify text-only messaging still works**

Send a normal text message without images. Expected: LLM replies normally.

- [ ] **Step 3: Verify text+images (vision model not configured)**

1. Set a vision model in settings if not already set (so the upload button appears)
2. Send a text message with an attached image
3. Expected: Yellow notice banner appears saying images were ignored. LLM replies based on text only (no garbled output).

- [ ] **Step 4: Verify images-only**

1. Attach an image and send without typing text
2. Expected: Error notification appears ("请为图片添加文字描述后再发送。"). No LLM call is made.
