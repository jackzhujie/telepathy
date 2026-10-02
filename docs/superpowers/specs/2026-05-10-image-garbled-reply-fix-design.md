# Fix Garbled LLM Replies After Image Upload

## Problem

When users add images to a conversation, the LLM produces garbled/incoherent replies.

**Root cause**: `describe_images_base64` generates placeholder text for each image (`"[图片数据: N bytes base64 encoded. 由于当前使用的是纯文本对话模型，无法直接理解图片内容...]"`) and injects it into the LLM prompt via `enhanced_query`. The prompt becomes self-contradictory — it tells the LLM there are images with content, then tells it the model can't understand images, yet still instructs it to respond based on image content. This confuses the text-only LLM and produces incoherent output.

## Design

### Architecture: ImageProcessingResult enum

Introduce a clear branch point for future vision model support:

```
Frontend (base64) → IPC → rag_query
                            ↓
                    describe_images_base64()
                            ↓
                    ImageProcessingResult
                      ↓              ↓
              Processed              Fallback
              { descriptions }       { image_count }
              (future)               (current path)
```

The `Fallback` path never injects placeholder text into the LLM prompt. The `Processed` path is reserved for future vision model integration.

### Data model (`vision.rs`)

```rust
pub enum ImageProcessingResult {
    #[allow(dead_code)]
    Processed { descriptions: Vec<String> },  // future: vision model output
    Fallback { image_count: usize },           // no vision model available
}
```

`describe_images_base64` returns `ImageProcessingResult` instead of `String`. Current implementation always returns `Fallback`.

### Branch logic (`rag.rs`)

| Condition | Action |
|-----------|--------|
| `Fallback`, `image_count > 0`, query empty | Emit `chat-error`, return early (no LLM call) |
| `Fallback`, `image_count > 0`, query non-empty | Emit `images-ignored`, proceed with text-only inference |
| `Fallback`, `image_count == 0` | Normal text-only path (no images attached) |
| `Processed` | `todo!()` — future vision path |

### New frontend event (`chat.ts`)

Listen for `chat-images-ignored` event. When received, the store sets a flag that `Chat.vue` uses to display a dismissable notice: "当前未配置 Vision 模型，图片未被理解，仅基于文本回复。"

### Edge cases

| Scenario | Behavior |
|----------|----------|
| Text + images | Text-only inference, notify user images were ignored |
| Images only, no text | Error returned to frontend, no LLM call |
| Empty text + empty images | Blocked by `ChatInput` frontend validation |
| Vision model configured (future) | Currently still `Fallback`; swap to `Processed` when implementation lands |

## Files changed

| File | Change |
|------|--------|
| `src-tauri/src/services/parser/vision.rs` | Add enum, refactor functions, fix tests |
| `src-tauri/src/commands/rag.rs` | Branch on `ImageProcessingResult`, emit new events |
| `src/stores/chat.ts` | Listen for `chat-images-ignored` event |
| `src/views/Chat.vue` | Display images-ignored notice |

## Out of scope

- Actual vision model inference (mmproj loading, image encoding)
- Vision model detection/configuration changes
- UI changes to image upload flow
