# P1 Ollama Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate a local Ollama model (hardcoded to qwen2.5) with the Telepathy app, enabling streaming chat responses in the UI.

**Architecture:** The Rust backend will use `reqwest` to stream responses from the local Ollama API and use Tauri's event system (`app_handle.emit`) to push tokens to the Vue frontend. The frontend will listen to these events and update the chat UI to create a typewriter effect.

**Tech Stack:** Rust, reqwest, futures-util, tokio, serde_json, Vue 3, Naive UI, Tauri Core Events.

---

### Task 1: Add Rust Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add reqwest, futures-util and serde_json**
```bash
cd src-tauri
cargo add reqwest --features json,stream
cargo add futures-util
cargo add serde_json
cd ..
```

- [ ] **Step 2: Run Cargo Check**
```bash
cd src-tauri
cargo check
cd ..
```

- [ ] **Step 3: Commit**
```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "build: add reqwest and streaming dependencies for Ollama"
```

### Task 2: Create Ollama Service

**Files:**
- Create: `src-tauri/src/services/ollama.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: Define Ollama API Types and stream_chat logic**
Create `src-tauri/src/services/ollama.rs`:
```rust
use crate::errors::AppError;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter};

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    message: OllamaMessage,
    done: bool,
}

#[derive(Deserialize, Debug)]
struct OllamaMessage {
    content: String,
}

#[derive(Serialize, Clone)]
struct TokenPayload {
    token: String,
}

#[derive(Serialize, Clone)]
struct ErrorPayload {
    error: String,
}

pub async fn stream_chat(prompt: &str, app_handle: AppHandle) -> Result<(), AppError> {
    let client = Client::new();
    let url = "http://localhost:11434/api/chat";
    
    let payload = json!({
        "model": "qwen2.5",
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "stream": true
    });

    let mut response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to Ollama: {}", e)))?
        .bytes_stream();

    while let Some(chunk) = response.next().await {
        match chunk {
            Ok(bytes) => {
                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                    // Ollama streams JSON objects separated by newlines
                    for line in text.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<OllamaResponse>(line) {
                            Ok(ollama_resp) => {
                                let _ = app_handle.emit("chat-token", TokenPayload {
                                    token: ollama_resp.message.content
                                });
                                
                                if ollama_resp.done {
                                    let _ = app_handle.emit("chat-done", ());
                                }
                            }
                            Err(e) => {
                                let _ = app_handle.emit("chat-error", ErrorPayload {
                                    error: format!("Failed to parse JSON: {}", e)
                                });
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let _ = app_handle.emit("chat-error", ErrorPayload {
                    error: format!("Stream error: {}", e)
                });
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Export module in mod.rs**
Modify `src-tauri/src/services/mod.rs`:
```rust
pub mod ollama;
```

- [ ] **Step 3: Run clippy**
```bash
cd src-tauri
cargo clippy -- -D warnings
cd ..
```

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/services
git commit -m "feat(backend): implement ollama streaming service"
```

### Task 3: Create Chat Command

**Files:**
- Create: `src-tauri/src/commands/chat.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create Tauri Command**
Create `src-tauri/src/commands/chat.rs`:
```rust
use crate::errors::AppError;
use crate::services::ollama;
use tauri::AppHandle;

#[tauri::command]
pub async fn ask_ollama(prompt: String, app_handle: AppHandle) -> Result<(), AppError> {
    // Spawn the async service call so it doesn't block the Tauri command handler
    tauri::async_runtime::spawn(async move {
        if let Err(e) = ollama::stream_chat(&prompt, app_handle.clone()).await {
            let _ = app_handle.emit("chat-error", e.to_string());
        }
    });
    
    Ok(())
}
```

- [ ] **Step 2: Export module in mod.rs**
Modify `src-tauri/src/commands/mod.rs` to add chat module:
```rust
pub mod greet;
pub mod chat;
```

- [ ] **Step 3: Register command in lib.rs**
Modify `src-tauri/src/lib.rs` to include `commands::chat::ask_ollama` in the invoke handler:
```rust
mod commands;
mod errors;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet::greet,
            commands::chat::ask_ollama
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Run clippy**
```bash
cd src-tauri
cargo clippy -- -D warnings
cd ..
```

- [ ] **Step 5: Commit**
```bash
git add src-tauri/src/commands src-tauri/src/lib.rs
git commit -m "feat(backend): add ask_ollama tauri command"
```

### Task 4: Setup Frontend API and Types

**Files:**
- Modify: `src/api/tauri.ts`
- Create: `src/types/chat.ts`

- [ ] **Step 1: Define Chat Types**
Create `src/types/chat.ts`:
```typescript
export interface Message {
  role: 'user' | 'assistant';
  content: string;
}

export interface TokenPayload {
  token: string;
}

export interface ErrorPayload {
  error: string;
}
```

- [ ] **Step 2: Add API wrapper**
Modify `src/api/tauri.ts` to append `askOllama`:
```typescript
import { invoke } from '@tauri-apps/api/core';

export async function greet(name: string): Promise<string> {
  return invoke<string>('greet', { name });
}

export async function askOllama(prompt: string): Promise<void> {
  return invoke<void>('ask_ollama', { prompt });
}
```

- [ ] **Step 3: Run type check**
```bash
npx vue-tsc --noEmit
```

- [ ] **Step 4: Commit**
```bash
git add src/api src/types
git commit -m "feat(frontend): add askOllama API wrapper and chat types"
```

### Task 5: Implement Chat UI

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: Implement Chat Layout and Logic**
Update `src/views/Chat.vue` to the following:
```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useMessage } from 'naive-ui';
import { askOllama } from '@/api/tauri';
import type { Message, TokenPayload, ErrorPayload } from '@/types/chat';

const message = useMessage();
const inputPrompt = ref('');
const messages = ref<Message[]>([]);
const isWaiting = ref(false);
const scrollbarRef = ref<any>(null);

let unlistenToken: UnlistenFn | null = null;
let unlistenDone: UnlistenFn | null = null;
let unlistenError: UnlistenFn | null = null;

const scrollToBottom = async () => {
  await nextTick();
  if (scrollbarRef.value) {
    scrollbarRef.value.scrollTo({ position: 'bottom', behavior: 'smooth' });
  }
};

const sendMessage = async () => {
  if (!inputPrompt.value.trim() || isWaiting.value) return;

  const promptText = inputPrompt.value.trim();
  messages.value.push({ role: 'user', content: promptText });
  messages.value.push({ role: 'assistant', content: '' });
  
  inputPrompt.value = '';
  isWaiting.value = true;
  await scrollToBottom();

  try {
    await askOllama(promptText);
  } catch (err) {
    message.error(`请求失败: ${err}`);
    isWaiting.value = false;
  }
};

onMounted(async () => {
  unlistenToken = await listen<TokenPayload>('chat-token', (event) => {
    const lastMsg = messages.value[messages.value.length - 1];
    if (lastMsg && lastMsg.role === 'assistant') {
      lastMsg.content += event.payload.token;
      scrollToBottom();
    }
  });

  unlistenDone = await listen('chat-done', () => {
    isWaiting.value = false;
    scrollToBottom();
  });

  unlistenError = await listen<ErrorPayload | string>('chat-error', (event) => {
    const errMsg = typeof event.payload === 'string' ? event.payload : event.payload.error;
    message.error(`AI 错误: ${errMsg}`);
    isWaiting.value = false;
  });
});

onUnmounted(() => {
  if (unlistenToken) unlistenToken();
  if (unlistenDone) unlistenDone();
  if (unlistenError) unlistenError();
});
</script>

<template>
  <div class="chat-container">
    <div class="chat-header">
      <h2>AI 对话 (Qwen 2.5)</h2>
    </div>

    <n-scrollbar ref="scrollbarRef" class="chat-messages">
      <div class="message-list">
        <div 
          v-for="(msg, index) in messages" 
          :key="index"
          :class="['message-wrapper', msg.role === 'user' ? 'user' : 'assistant']"
        >
          <div class="message-bubble">
            {{ msg.content }}
          </div>
        </div>
        <div v-if="messages.length === 0" class="empty-state">
          有什么我可以帮您的吗？
        </div>
      </div>
    </n-scrollbar>

    <div class="chat-input-area">
      <n-input
        v-model:value="inputPrompt"
        type="textarea"
        placeholder="输入您的问题 (Enter 发送，Shift+Enter 换行)..."
        :autosize="{ minRows: 2, maxRows: 6 }"
        :disabled="isWaiting"
        @keydown.enter.prevent="sendMessage"
        @keydown.shift.enter.exact="() => {}"
      />
      <n-button 
        type="primary" 
        style="margin-left: 12px; height: 100%" 
        :loading="isWaiting"
        :disabled="!inputPrompt.trim() && !isWaiting"
        @click="sendMessage"
      >
        发送
      </n-button>
    </div>
  </div>
</template>

<style scoped>
.chat-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  max-width: 900px;
  margin: 0 auto;
}

.chat-header {
  padding-bottom: 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.09);
}

.chat-header h2 {
  margin: 0;
  font-weight: 500;
}

.chat-messages {
  flex: 1;
  padding: 20px 0;
}

.message-list {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding-right: 16px;
}

.message-wrapper {
  display: flex;
  width: 100%;
}

.message-wrapper.user {
  justify-content: flex-end;
}

.message-wrapper.assistant {
  justify-content: flex-start;
}

.message-bubble {
  max-width: 80%;
  padding: 12px 16px;
  border-radius: 8px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

.user .message-bubble {
  background-color: #2080f0;
  color: #fff;
  border-bottom-right-radius: 2px;
}

.assistant .message-bubble {
  background-color: #333333;
  color: #e5e5e5;
  border-bottom-left-radius: 2px;
}

.empty-state {
  text-align: center;
  color: #666;
  margin-top: 40px;
}

.chat-input-area {
  display: flex;
  align-items: flex-end;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.09);
}
</style>
```

- [ ] **Step 2: Check TypeScript Types**
```bash
npx vue-tsc --noEmit
```

- [ ] **Step 3: Commit**
```bash
git add src/views/Chat.vue
git commit -m "feat(frontend): implement chat UI with streaming response support"
```
