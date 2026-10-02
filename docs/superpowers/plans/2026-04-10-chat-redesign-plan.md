# Chat 页面组件化重构 — 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Chat 页面从单文件组件重构为模块化组件架构，添加消息操作（复制、编辑、重新生成）、停止生成、加载状态优化、代码块增强等功能。

**Architecture:** 前端拆分为 9 个子组件 + 1 个 Pinia store，后端添加停止生成和重新生成 API。使用 reka-ui DropdownMenu/Tooltip 实现消息操作，@tanstack/vue-virtual 实现虚拟滚动。

**Tech Stack:** Vue 3 + TypeScript + Pinia + reka-ui + @tanstack/vue-virtual + markdown-it + highlight.js + Tauri v2 + Rust (tokio-util)

---

### Task 1: 安装依赖 + 扩展类型定义

**Files:**
- Modify: `package.json` (install)
- Modify: `src/types/chat.ts`

- [ ] **Step 1: 安装 @tanstack/vue-virtual**

```bash
cd /sessions/69d7be74a5234068df3dcc36/workspace && pnpm add @tanstack/vue-virtual
```

- [ ] **Step 2: 扩展 Message 类型**

修改 `src/types/chat.ts`，添加新字段：

```typescript
export interface Message {
  id: string;                    // 唯一 ID
  role: 'user' | 'assistant';
  content: string;
  sources?: SearchSource[];
  status: 'sending' | 'streaming' | 'done' | 'error';
  error?: string;
  timestamp: number;
  isEditing?: boolean;
}
```

- [ ] **Step 3: 提交**

```bash
git add package.json pnpm-lock.yaml src/types/chat.ts
git commit -m "feat(chat): install vue-virtual and extend Message type"
```

---

### Task 2: 后端 — 添加停止生成 API

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/commands/rag.rs`

- [ ] **Step 1: 添加 tokio-util 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
tokio-util = "0.7"
```

- [ ] **Step 2: 添加全局 CancellationToken**

在 `src-tauri/src/commands/rag.rs` 顶部添加：

```rust
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// 全局 cancellation token
pub static CANCEL_TOKEN: std::sync::LazyLock<Arc<CancellationToken>> =
    std::sync::LazyLock::new(|| Arc::new(CancellationToken::new()));
```

- [ ] **Step 3: 在 rag_query streaming loop 中检查 cancellation**

在 `rag_query` 的 `tauri::async_runtime::spawn` 内部，streaming loop 中添加检查。在 `while let Some(chunk) = stream.next().await` 循环体内，`if let Ok(bytes) = chunk` 之后添加：

```rust
if CANCEL_TOKEN.is_cancelled() {
    let _ = app_handle_clone.emit("chat-done", ());
    break;
}
```

- [ ] **Step 4: 添加 stop_generation command**

```rust
#[tauri::command]
pub async fn stop_generation() -> Result<(), AppError> {
    CANCEL_TOKEN.cancel();
    // 创建新 token 供下次使用
    Ok(())
}
```

注意：需要在每次 `rag_query` 开始时重置 token。在 `rag_query` 函数开头（`tauri::async_runtime::spawn` 之前）添加：

```rust
// Reset cancellation token for new request
let token = CancellationToken::new();
{
    let mut guard = CANCEL_TOKEN.clone();
    // We need to swap the token - use a different approach
}
```

更好的方案：使用 `Arc<CancellationToken>` 作为可替换的引用。在模块级别使用 `Mutex`：

```rust
use std::sync::Mutex;

static CANCEL_TOKEN: std::sync::LazyLock<Arc<Mutex<Arc<CancellationToken>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(Arc::new(CancellationToken::new()))));

fn reset_cancel_token() {
    let mut guard = CANCEL_TOKEN.lock().unwrap();
    *guard = Arc::new(CancellationToken::new());
}

fn get_cancel_token() -> Arc<CancellationToken> {
    CANCEL_TOKEN.lock().unwrap().clone()
}
```

在 `rag_query` 开头调用 `reset_cancel_token()`，在 streaming loop 中检查 `get_cancel_token().is_cancelled()`，在 `stop_generation` 中调用 `get_cancel_token().cancel()`。

- [ ] **Step 5: 注册新 command**

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中添加 `"stop_generation"`。

- [ ] **Step 6: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/src/commands/rag.rs src-tauri/src/lib.rs
git commit -m "feat(chat): add stop_generation backend API with CancellationToken"
```

---

### Task 3: 后端 — 添加重新生成 API

**Files:**
- Modify: `src-tauri/src/commands/rag.rs`
- Modify: `src-tauri/src/db/conversations.rs` (可能需要添加删除后续消息的函数)

- [ ] **Step 1: 添加 delete_messages_after 函数**

在 `src-tauri/src/db/conversations.rs` 中添加：

```rust
pub fn delete_messages_after(conn: &Connection, conversation_id: &str, created_at: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1 AND created_at >= ?2",
        rusqlite::params![conversation_id, created_at],
    )?;
    Ok(())
}
```

- [ ] **Step 2: 添加 regenerate_message command**

在 `src-tauri/src/commands/rag.rs` 中添加：

```rust
#[tauri::command]
pub async fn regenerate_message(
    conversation_id: String,
    message_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    // Get the target message to find its timestamp
    let msg = conversations::get_message_by_id(&conn, &message_id)?;
    
    // Delete this message and all after it
    conversations::delete_messages_after(&conn, &conversation_id, &msg.created_at)?;

    // Get the user message before this (the query to regenerate from)
    let history = conversations::get_messages(&conn, &conversation_id)?;
    let last_user_msg = history.iter().rev().find(|m| m.role == "user");
    let query = last_user_msg.map(|m| m.content.clone()).unwrap_or_default();

    if query.is_empty() {
        return Err(AppError::Internal("No user message found to regenerate from".to_string()));
    }

    // Re-run rag_query with the existing conversation_id
    rag_query(query, Some(conversation_id), None, app_handle).await
}
```

注意：需要在 `conversations.rs` 中添加 `get_message_by_id` 函数。

- [ ] **Step 3: 注册新 command**

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中添加 `"regenerate_message"`。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/rag.rs src-tauri/src/db/conversations.rs src-tauri/src/lib.rs
git commit -m "feat(chat): add regenerate_message backend API"
```

---

### Task 4: 前端 API 层 — 添加新 API 函数

**Files:**
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 添加新 API 函数**

在 `src/api/tauri.ts` 中添加：

```typescript
export async function stopGeneration(): Promise<void> {
  return invoke<void>('stop_generation');
}

export async function regenerateMessage(conversationId: string, messageId: string): Promise<void> {
  return invoke<void>('regenerate_message', { conversationId, messageId });
}
```

- [ ] **Step 2: 提交**

```bash
git add src/api/tauri.ts
git commit -m "feat(chat): add stopGeneration and regenerateMessage API functions"
```

---

### Task 5: 创建 Chat Store

**Files:**
- Create: `src/stores/chat.ts`

- [ ] **Step 1: 创建 chat store**

创建 `src/stores/chat.ts`，从 Chat.vue 提取所有状态和逻辑。完整代码如下：

```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  ragQuery,
  getConversations,
  getMessages,
  deleteConversation,
  stopGeneration as stopGenApi,
  regenerateMessage as regenApi,
} from '@/api/tauri';
import { useProjectsStore } from '@/stores/projects';
import { renderMarkdown } from '@/utils/markdown';
import type {
  Message,
  TokenPayload,
  ErrorPayload,
  SearchSource,
  RagSourcesPayload,
  Conversation,
  ChatMessage,
} from '@/types/chat';

let messageIdCounter = 0;
function generateId(): string {
  return `msg-${Date.now()}-${++messageIdCounter}`;
}

export const useChatStore = defineStore('chat', () => {
  const messages = ref<Message[]>([]);
  const conversations = ref<Conversation[]>([]);
  const currentConversationId = ref<string | undefined>(undefined);
  const isGenerating = ref(false);
  const isThinking = ref(false);
  const currentSources = ref<SearchSource[]>([]);
  const showSources = ref(false);
  const error = ref<string | null>(null);
  const selectedProjectId = ref<string | null>(null);

  const projectsStore = useProjectsStore();

  const unlistenPromises: Promise<UnlistenFn>[] = [];

  // --- Actions ---

  async function initListeners() {
    unlistenPromises.push(
      listen<TokenPayload>('chat-token', (event) => {
        isThinking.value = false;
        const lastMsg = messages.value[messages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          lastMsg.content += event.payload.token;
          lastMsg.status = 'streaming';
        }
      })
    );

    unlistenPromises.push(
      listen('chat-done', () => {
        const lastMsg = messages.value[messages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          lastMsg.status = 'done';
        }
        isGenerating.value = false;
        isThinking.value = false;
        fetchConversations();
      })
    );

    unlistenPromises.push(
      listen<ErrorPayload | string>('chat-error', (event) => {
        const errMsg = typeof event.payload === 'string' ? event.payload : event.payload.error;
        const lastMsg = messages.value[messages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          lastMsg.status = 'error';
          lastMsg.error = errMsg;
        }
        error.value = errMsg;
        isGenerating.value = false;
        isThinking.value = false;
        setTimeout(() => { error.value = null; }, 5000);
      })
    );

    unlistenPromises.push(
      listen<RagSourcesPayload>('rag-sources', (event) => {
        currentSources.value = event.payload.sources;
        showSources.value = true;
      })
    );

    unlistenPromises.push(
      listen<{ id: string; title: string }>('conversation-created', (event) => {
        currentConversationId.value = event.payload.id;
        fetchConversations();
      })
    );
  }

  async function cleanup() {
    const unlistens = await Promise.all(unlistenPromises);
    unlistens.forEach((fn) => fn());
    unlistenPromises.length = 0;
  }

  async function fetchConversations() {
    try {
      conversations.value = await getConversations();
    } catch {
      // ignore
    }
  }

  async function loadConversation(convId: string) {
    currentConversationId.value = convId;
    currentSources.value = [];
    showSources.value = false;
    try {
      const msgs = await getMessages(convId);
      messages.value = msgs.map((m: ChatMessage) => ({
        id: m.id,
        role: m.role as 'user' | 'assistant',
        content: m.content,
        sources: m.sources ? JSON.parse(m.sources) : undefined,
        status: 'done' as const,
        timestamp: new Date(m.created_at).getTime(),
      }));
    } catch (e) {
      error.value = `加载对话失败: ${e}`;
      setTimeout(() => { error.value = null; }, 5000);
    }
  }

  function newConversation() {
    currentConversationId.value = undefined;
    messages.value = [];
    currentSources.value = [];
    showSources.value = false;
  }

  async function deleteConversationAction(convId: string) {
    try {
      await deleteConversation(convId);
      conversations.value = conversations.value.filter((c) => c.id !== convId);
      if (currentConversationId.value === convId) {
        newConversation();
      }
    } catch (err) {
      error.value = `删除失败: ${err}`;
      setTimeout(() => { error.value = null; }, 5000);
    }
  }

  async function sendMessage(prompt: string) {
    if (!prompt.trim() || isGenerating.value) return;

    const userMsg: Message = {
      id: generateId(),
      role: 'user',
      content: prompt.trim(),
      status: 'done',
      timestamp: Date.now(),
    };
    const assistantMsg: Message = {
      id: generateId(),
      role: 'assistant',
      content: '',
      status: 'sending',
      timestamp: Date.now(),
    };

    messages.value.push(userMsg, assistantMsg);
    currentSources.value = [];
    showSources.value = false;
    isGenerating.value = true;
    isThinking.value = true;
    error.value = null;

    try {
      await ragQuery(prompt.trim(), currentConversationId.value, selectedProjectId.value || undefined);
    } catch (err) {
      error.value = `请求失败: ${err}`;
      isGenerating.value = false;
      isThinking.value = false;
      const lastMsg = messages.value[messages.value.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        lastMsg.status = 'error';
        lastMsg.error = String(err);
      }
      setTimeout(() => { error.value = null; }, 5000);
    }
  }

  async function stopGeneration() {
    try {
      await stopGenApi();
    } catch {
      // ignore
    }
  }

  async function regenerateMessage(index: number) {
    const msg = messages.value[index];
    if (!msg || msg.role !== 'assistant' || isGenerating.value) return;

    // Remove this message and all after it
    messages.value = messages.value.slice(0, index);

    // Add new empty assistant message
    messages.value.push({
      id: generateId(),
      role: 'assistant',
      content: '',
      status: 'sending',
      timestamp: Date.now(),
    });

    isGenerating.value = true;
    isThinking.value = true;

    try {
      await regenApi(currentConversationId.value!, msg.id);
    } catch (err) {
      error.value = `重新生成失败: ${err}`;
      isGenerating.value = false;
      isThinking.value = false;
      setTimeout(() => { error.value = null; }, 5000);
    }
  }

  async function editAndResend(index: number, newContent: string) {
    if (!newContent.trim() || isGenerating.value) return;

    // Remove this message and all after it
    messages.value = messages.value.slice(0, index);

    // Send new message
    await sendMessage(newContent.trim());
  }

  function copyMessage(index: number): string {
    const msg = messages.value[index];
    if (!msg) return '';
    // For assistant messages, copy rendered text (strip HTML)
    const tmp = document.createElement('div');
    tmp.innerHTML = renderMarkdown(msg.content);
    return tmp.textContent || tmp.innerText || msg.content;
  }

  function copyRawMessage(index: number): string {
    const msg = messages.value[index];
    return msg?.content || '';
  }

  return {
    messages,
    conversations,
    currentConversationId,
    isGenerating,
    isThinking,
    currentSources,
    showSources,
    error,
    selectedProjectId,
    initListeners,
    cleanup,
    fetchConversations,
    loadConversation,
    newConversation,
    deleteConversation: deleteConversationAction,
    sendMessage,
    stopGeneration,
    regenerateMessage,
    editAndResend,
    copyMessage,
    copyRawMessage,
  };
});
```

- [ ] **Step 2: 提交**

```bash
git add src/stores/chat.ts
git commit -m "feat(chat): create chat store with all state and actions"
```

---

### Task 6: 创建基础子组件

**Files:**
- Create: `src/components/chat/ThinkingIndicator.vue`
- Create: `src/components/chat/CodeBlock.vue`
- Create: `src/components/chat/MessageActions.vue`

- [ ] **Step 1: 创建 ThinkingIndicator.vue**

```vue
<script setup lang="ts">
</script>

<template>
  <div class="flex items-center gap-2 py-2 text-text-muted">
    <div class="flex gap-1">
      <span class="w-1.5 h-1.5 rounded-full bg-brand-400 animate-bounce" style="animation-delay: 0ms"></span>
      <span class="w-1.5 h-1.5 rounded-full bg-brand-400 animate-bounce" style="animation-delay: 150ms"></span>
      <span class="w-1.5 h-1.5 rounded-full bg-brand-400 animate-bounce" style="animation-delay: 300ms"></span>
    </div>
    <span class="text-xs">正在思考...</span>
  </div>
</template>
```

- [ ] **Step 2: 创建 CodeBlock.vue**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import {
  TooltipRoot,
  TooltipTrigger,
  TooltipContent,
  TooltipPortal,
  TooltipProvider,
} from 'reka-ui';

const props = defineProps<{
  code: string;
  language?: string;
}>();

const copied = ref(false);

async function copyCode() {
  try {
    await navigator.clipboard.writeText(props.code);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch {
    // fallback
  }
}
</script>

<template>
  <TooltipProvider :delay-duration="300">
    <div class="relative group">
      <div class="flex items-center justify-between px-3 py-1.5 bg-dark-surface/50 rounded-t-md border-b border-dark-border/30">
        <span class="text-[10px] text-text-muted font-medium">{{ language || 'code' }}</span>
        <TooltipRoot>
          <TooltipTrigger as-child>
            <button
              class="p-1 rounded hover:bg-dark-surface transition-colors"
              @click="copyCode"
            >
              <svg v-if="!copied" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-green-400">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            </button>
          </TooltipTrigger>
          <TooltipPortal>
            <TooltipContent class="px-2 py-1 bg-dark-surface text-text-primary text-[10px] rounded shadow-lg border border-dark-border/50" :side-offset="5">
              {{ copied ? '已复制' : '复制代码' }}
            </TooltipContent>
          </TooltipPortal>
        </TooltipRoot>
      </div>
      <pre class="hljs-code-block !rounded-t-none !mt-0"><code v-html="code"></code></pre>
    </div>
  </TooltipProvider>
</template>
```

- [ ] **Step 3: 创建 MessageActions.vue**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuPortal,
} from 'reka-ui';

const props = defineProps<{
  role: 'user' | 'assistant';
  messageIndex: number;
  isGenerating: boolean;
}>();

const emit = defineEmits<{
  copy: [];
  copyRaw: [];
  edit: [];
  regenerate: [];
}>();

const copied = ref(false);

async function handleCopy() {
  emit('copy');
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 2000);
}

function handleCopyRaw() {
  emit('copyRaw');
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 2000);
}
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger as-child>
      <button
        class="p-1 rounded hover:bg-dark-surface/80 transition-colors opacity-0 group-hover:opacity-100"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted">
          <circle cx="12" cy="12" r="1"></circle>
          <circle cx="12" cy="5" r="1"></circle>
          <circle cx="12" cy="19" r="1"></circle>
        </svg>
      </button>
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        class="min-w-[140px] bg-dark-panel rounded-md shadow-xl border border-dark-border/50 p-1 z-50"
        :side-offset="5"
        align="end"
      >
        <DropdownMenuItem
          class="flex items-center gap-2 px-2 py-1.5 rounded-sm text-xs cursor-pointer hover:bg-dark-surface outline-none data-[highlighted]:bg-dark-surface"
          @select="handleCopy"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
          {{ copied ? '已复制 ✓' : '复制' }}
        </DropdownMenuItem>
        <DropdownMenuItem
          v-if="role === 'assistant'"
          class="flex items-center gap-2 px-2 py-1.5 rounded-sm text-xs cursor-pointer hover:bg-dark-surface outline-none data-[highlighted]:bg-dark-surface"
          @select="handleCopyRaw"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted">
            <polyline points="16 18 22 12 16 6"></polyline>
            <polyline points="8 6 2 12 8 18"></polyline>
          </svg>
          复制原始文本
        </DropdownMenuItem>
        <DropdownMenuSeparator class="h-px bg-dark-border/50 my-1" />
        <DropdownMenuItem
          v-if="role === 'user'"
          class="flex items-center gap-2 px-2 py-1.5 rounded-sm text-xs cursor-pointer hover:bg-dark-surface outline-none data-[highlighted]:bg-dark-surface"
          @select="emit('edit')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
          </svg>
          编辑并重新发送
        </DropdownMenuItem>
        <DropdownMenuItem
          v-if="role === 'assistant' && !isGenerating"
          class="flex items-center gap-2 px-2 py-1.5 rounded-sm text-xs cursor-pointer hover:bg-dark-surface outline-none data-[highlighted]:bg-dark-surface"
          @select="emit('regenerate')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-muted">
            <polyline points="23 4 23 10 17 10"></polyline>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
          </svg>
          重新生成
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
```

- [ ] **Step 4: 提交**

```bash
git add src/components/chat/ThinkingIndicator.vue src/components/chat/CodeBlock.vue src/components/chat/MessageActions.vue
git commit -m "feat(chat): create ThinkingIndicator, CodeBlock, MessageActions components"
```

---

### Task 7: 创建 MessageItem.vue

**Files:**
- Create: `src/components/chat/MessageItem.vue`

- [ ] **Step 1: 创建 MessageItem 组件**

```vue
<script setup lang="ts">
import { ref, nextTick, watch } from 'vue';
import { renderMarkdown } from '@/utils/markdown';
import MessageActions from './MessageActions.vue';
import ThinkingIndicator from './ThinkingIndicator.vue';
import type { Message } from '@/types/chat';

const props = defineProps<{
  message: Message;
  index: number;
  isGenerating: boolean;
  showSources: boolean;
}>();

const emit = defineEmits<{
  copy: [index: number];
  copyRaw: [index: number];
  edit: [index: number];
  regenerate: [index: number];
  toggleSources: [];
}>();

const editContent = ref('');
const isEditing = ref(false);
const editRef = ref<HTMLTextAreaElement | null>(null);

watch(() => props.message.isEditing, (val) => {
  if (val) {
    editContent.value = props.message.content;
    isEditing.value = true;
    nextTick(() => {
      editRef.value?.focus();
    });
  } else {
    isEditing.value = false;
  }
});

function startEdit() {
  emit('edit', props.index);
}

function submitEdit() {
  if (!editContent.value.trim()) return;
  emit('edit', props.index);
  // Parent will handle the actual resend via store
}

function cancelEdit() {
  isEditing.value = false;
  // Reset isEditing on the message - parent handles this
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    submitEdit();
  } else if (e.key === 'Escape') {
    cancelEdit();
  }
}
</script>

<template>
  <div
    :class="[
      'flex w-full items-start group',
      message.role === 'user' ? 'justify-end' : 'justify-start',
    ]"
  >
    <div :class="[
      'max-w-[80%] rounded-md shadow-sm transition-all duration-300 relative',
      message.role === 'user'
        ? 'bg-brand-500 text-white px-3 py-2'
        : 'bg-dark-panel border border-dark-border/50 px-3 py-2',
    ]">
      <!-- Message Actions -->
      <div
        v-if="message.status === 'done' && !isEditing"
        class="absolute -top-2"
        :class="message.role === 'user' ? 'left-2' : 'right-2'"
      >
        <MessageActions
          :role="message.role"
          :message-index="index"
          :is-generating="isGenerating"
          @copy="emit('copy', index)"
          @copy-raw="emit('copyRaw', index)"
          @edit="startEdit"
          @regenerate="emit('regenerate', index)"
        />
      </div>

      <!-- User message: normal or editing -->
      <template v-if="message.role === 'user'">
        <div v-if="!isEditing" class="text-xs font-medium whitespace-pre-wrap">{{ message.content }}</div>
        <textarea
          v-else
          ref="editRef"
          v-model="editContent"
          @keydown="handleKeydown"
          class="w-full min-h-[60px] bg-dark-surface text-text-primary border border-dark-border/50 rounded px-2 py-1.5 text-xs resize-none focus:outline-none focus:ring-1 focus:ring-brand-500/30"
        ></textarea>
      </template>

      <!-- Assistant message -->
      <template v-else>
        <!-- Thinking state -->
        <ThinkingIndicator v-if="message.status === 'sending' && !message.content" />

        <!-- Streaming content -->
        <div v-if="message.content" class="markdown-body text-xs" v-html="renderMarkdown(message.content)"></div>

        <!-- Streaming cursor -->
        <span
          v-if="message.status === 'streaming'"
          class="inline-block w-1.5 h-3.5 bg-brand-400 animate-pulse ml-0.5 align-middle"
        ></span>

        <!-- Error state -->
        <div v-if="message.status === 'error'" class="mt-2 flex items-center gap-2 text-red-400 text-xs">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="15" y1="9" x2="9" y2="15"></line>
            <line x1="9" y1="9" x2="15" y2="15"></line>
          </svg>
          {{ message.error || '生成失败' }}
        </div>

        <!-- Sources toggle -->
        <div
          v-if="message.sources && message.sources.length > 0"
          class="mt-2 pt-2 border-t border-dark-border/30"
        >
          <button
            class="text-[10px] text-brand-400 font-semibold hover:underline"
            @click="emit('toggleSources')"
          >
            {{ showSources ? '收起来源' : '查看来源' }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/chat/MessageItem.vue
git commit -m "feat(chat): create MessageItem component with edit/actions support"
```

---

### Task 8: 创建 ChatInput.vue

**Files:**
- Create: `src/components/chat/ChatInput.vue`

- [ ] **Step 1: 创建 ChatInput 组件**

```vue
<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps<{
  isGenerating: boolean;
}>();

const emit = defineEmits<{
  send: [content: string];
  stop: [];
}>();

const input = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);

function handleSend() {
  if (!input.value.trim() || props.isGenerating) return;
  emit('send', input.value.trim());
  input.value = '';
  nextTick(() => autoResize());
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleSend();
  }
}

function autoResize() {
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = 'auto';
  el.style.height = Math.min(el.scrollHeight, 200) + 'px';
}

watch(input, () => {
  nextTick(autoResize);
});

defineExpose({ focus: () => textareaRef.value?.focus() });
</script>

<template>
  <div class="flex items-end pt-3 border-t border-dark-border/50 gap-2">
    <textarea
      ref="textareaRef"
      v-model="input"
      :disabled="isGenerating"
      @keydown="handleKeydown"
      class="flex-1 min-h-[60px] max-h-[200px] px-3 py-2 border border-dark-border/50 rounded-md focus:outline-none focus:ring-1 focus:ring-brand-500/30 transition-all resize-none bg-dark-surface text-text-primary placeholder-text-muted text-xs leading-relaxed"
      placeholder="输入您的问题 (Enter 发送，Shift+Enter 换行)..."
      rows="2"
    ></textarea>
    <!-- Stop button -->
    <button
      v-if="isGenerating"
      class="px-3 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-md font-medium text-xs transition-colors flex items-center gap-1.5"
      @click="emit('stop')"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
        <rect x="6" y="6" width="12" height="12" rx="1"></rect>
      </svg>
      停止
    </button>
    <!-- Send button -->
    <button
      v-else
      :disabled="!input.trim()"
      class="px-3 py-2 bg-brand-500 hover:bg-brand-600 text-white rounded-md font-medium text-xs transition-colors flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed"
      @click="handleSend"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="22" y1="2" x2="11" y2="13"></line>
        <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
      </svg>
      发送
    </button>
  </div>
</template>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/chat/ChatInput.vue
git commit -m "feat(chat): create ChatInput component with send/stop toggle"
```

---

### Task 9: 创建 ConversationSidebar.vue

**Files:**
- Create: `src/components/chat/ConversationSidebar.vue`

- [ ] **Step 1: 创建 ConversationSidebar 组件**

从 Chat.vue 中提取对话侧边栏部分。接收 store 中的 conversations、currentConversationId，调用 store actions。

```vue
<script setup lang="ts">
import type { Conversation } from '@/types/chat';

defineProps<{
  conversations: Conversation[];
  currentConversationId: string | undefined;
  collapsed: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  new: [];
  delete: [id: string];
  toggleCollapse: [];
}>();
</script>

<template>
  <div
    class="border-r border-dark-border/50 flex flex-col transition-all duration-300 bg-dark-panel"
    :class="collapsed ? 'w-0 overflow-hidden border-r-0' : 'w-52'"
  >
    <div class="p-2 border-b border-dark-border/50">
      <button
        class="w-full px-3 py-2.5 bg-brand-500 hover:bg-brand-600 text-white rounded-md font-bold text-xs transition-colors"
        @click="emit('new')"
      >
        + 新对话
      </button>
    </div>
    <div class="overflow-y-auto flex-1">
      <div v-for="conv in conversations" :key="conv.id">
        <div
          class="flex items-center justify-between w-full p-2 m-1.5 cursor-pointer transition-all duration-200 rounded-md"
          :class="conv.id === currentConversationId ? 'bg-brand-500/10 border-l-2 border-brand-500' : 'hover:bg-dark-surface'"
          @click="emit('select', conv.id)"
        >
          <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-xs font-medium text-text-primary">{{ conv.title || '新对话' }}</span>
          <button
            class="opacity-0 hover:opacity-100 transition-opacity p-1"
            @click.stop="emit('delete', conv.id)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-red-400">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
      <div v-if="conversations.length === 0" class="text-center text-text-muted py-6 px-2 text-[10px]">
        暂无对话记录
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/chat/ConversationSidebar.vue
git commit -m "feat(chat): create ConversationSidebar component"
```

---

### Task 10: 创建 SourcesPanel.vue

**Files:**
- Create: `src/components/chat/SourcesPanel.vue`

- [ ] **Step 1: 创建 SourcesPanel 组件**

从 Chat.vue 中提取来源面板部分。

```vue
<script setup lang="ts">
import type { SearchSource } from '@/types/chat';

defineProps<{
  sources: SearchSource[];
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <div v-if="visible && sources.length > 0" class="border-t border-dark-border/50 rounded-t-md bg-dark-panel p-2.5">
    <div class="flex justify-between items-center mb-2">
      <h3 class="text-xs font-bold text-text-primary">引用来源</h3>
      <button class="text-text-muted text-sm hover:text-text-secondary" @click="emit('close')">×</button>
    </div>
    <div class="max-h-48 overflow-y-auto">
      <div
        v-for="(src, i) in sources"
        :key="i"
        class="mb-2 p-2.5 rounded-md bg-dark-surface border border-dark-border/50 shadow-sm transition-all duration-300 hover:shadow-md"
      >
        <div class="flex justify-between items-center mb-1.5">
          <span class="px-2 py-0.5 bg-brand-500/15 text-brand-400 text-[10px] rounded-full font-bold">{{ src.document_name }}</span>
          <span class="text-[10px] text-text-secondary font-semibold">相似度 {{ (src.score * 100).toFixed(1) }}%</span>
        </div>
        <div class="text-[10px] text-text-secondary leading-relaxed">{{ src.content }}</div>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/chat/SourcesPanel.vue
git commit -m "feat(chat): create SourcesPanel component"
```

---

### Task 11: 重构 Chat.vue 为容器

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: 重写 Chat.vue 为容器组件**

将 Chat.vue 精简为布局容器，组合所有子组件，使用 chat store。

```vue
<script setup lang="ts">
defineOptions({ name: 'Chat' });
import { ref, onMounted, onUnmounted, onActivated, onDeactivated, nextTick, computed } from 'vue';
import { useChatStore } from '@/stores/chat';
import { useProjectsStore } from '@/stores/projects';
import {
  SelectContent, SelectIcon, SelectItem, SelectItemText,
  SelectPortal, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
} from 'reka-ui';
import ConversationSidebar from '@/components/chat/ConversationSidebar.vue';
import MessageItem from '@/components/chat/MessageItem.vue';
import ChatInput from '@/components/chat/ChatInput.vue';
import SourcesPanel from '@/components/chat/SourcesPanel.vue';
import 'highlight.js/styles/github-dark.min.css';

const chatStore = useChatStore();
const projectsStore = useProjectsStore();

const sidebarCollapsed = ref(false);
const scrollbarRef = ref<HTMLElement | null>(null);

const projectOptions = computed(() => {
  return projectsStore.projects.map((p) => ({ label: p.name, value: p.id }));
});

const scrollToBottom = async () => {
  await nextTick();
  if (scrollbarRef.value) {
    scrollbarRef.value.scrollTop = scrollbarRef.value.scrollHeight;
  }
};

// Watch messages for auto-scroll
import { watch } from 'vue';
watch(
  () => chatStore.messages.length,
  () => scrollToBottom()
);
watch(
  () => {
    const msgs = chatStore.messages;
    if (msgs.length > 0) return msgs[msgs.length - 1].content;
    return '';
  },
  () => scrollToBottom()
);

async function handleCopy(index: number) {
  const text = chatStore.copyMessage(index);
  try { await navigator.clipboard.writeText(text); } catch {}
}

async function handleCopyRaw(index: number) {
  const text = chatStore.copyRawMessage(index);
  try { await navigator.clipboard.writeText(text); } catch {}
}

function handleEdit(index: number) {
  const msg = chatStore.messages[index];
  if (msg) msg.isEditing = true;
}

async function handleEditSubmit(index: number) {
  const msg = chatStore.messages[index];
  if (!msg || !msg.content.trim()) return;
  msg.isEditing = false;
  await chatStore.editAndResend(index, msg.content);
}

onMounted(async () => {
  await projectsStore.fetchProjects();
  await chatStore.fetchConversations();
  await chatStore.initListeners();
});

onActivated(async () => {
  if (chatStore.currentConversationId) {
    await chatStore.loadConversation(chatStore.currentConversationId);
    await scrollToBottom();
  }
  await chatStore.fetchConversations();
});

onDeactivated(() => {});

onUnmounted(async () => {
  await chatStore.cleanup();
});
</script>

<template>
  <div class="flex h-full gap-0 bg-dark-bg">
    <ConversationSidebar
      :conversations="chatStore.conversations"
      :current-conversation-id="chatStore.currentConversationId"
      :collapsed="sidebarCollapsed"
      @select="chatStore.loadConversation"
      @new="chatStore.newConversation"
      @delete="chatStore.deleteConversation"
      @toggle-collapse="sidebarCollapsed = !sidebarCollapsed"
    />

    <div class="flex-1 flex flex-col max-w-3xl mx-auto p-3 w-full bg-dark-bg">
      <!-- Header -->
      <div class="flex items-center justify-between gap-2 pb-3 border-b border-dark-border/50">
        <button
          class="p-1.5 hover:bg-dark-surface rounded-md transition-colors"
          @click="sidebarCollapsed = !sidebarCollapsed"
        >
          <svg v-if="sidebarCollapsed" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-secondary">
            <line x1="3" y1="12" x2="21" y2="12"></line>
            <line x1="3" y1="6" x2="21" y2="6"></line>
            <line x1="3" y1="18" x2="21" y2="18"></line>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-text-secondary">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        <h2 class="text-sm font-bold text-text-primary flex-1 tracking-tight">RAG 知识问答</h2>
        <SelectRoot v-model="chatStore.selectedProjectId">
          <SelectTrigger class="w-44 px-2.5 py-1 border border-dark-border/50 rounded-md focus:outline-none focus:ring-1 focus:ring-brand-500/30 transition-all bg-dark-surface text-text-primary flex items-center justify-between text-xs">
            <SelectValue placeholder="选择项目" />
            <SelectIcon>
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </SelectIcon>
          </SelectTrigger>
          <SelectPortal>
            <SelectContent class="bg-dark-panel rounded-md shadow-xl border border-dark-border/50 overflow-hidden z-50" position="popper" :side-offset="5">
              <SelectViewport class="p-1">
                <SelectItem
                  v-for="option in projectOptions"
                  :key="option.value"
                  :value="option.value"
                  class="relative flex items-center px-3 py-1.5 rounded-sm text-[11px] cursor-pointer hover:bg-dark-surface data-[highlighted]:bg-brand-500/10"
                >
                  <SelectItemText>{{ option.label }}</SelectItemText>
                </SelectItem>
              </SelectViewport>
            </SelectContent>
          </SelectPortal>
        </SelectRoot>
      </div>

      <!-- Messages -->
      <div ref="scrollbarRef" class="flex-1 overflow-y-auto py-3">
        <div class="space-y-3 pr-2">
          <MessageItem
            v-for="(msg, index) in chatStore.messages"
            :key="msg.id"
            :message="msg"
            :index="index"
            :is-generating="chatStore.isGenerating"
            :show-sources="chatStore.showSources"
            @copy="handleCopy"
            @copy-raw="handleCopyRaw"
            @edit="handleEdit"
            @regenerate="(i) => chatStore.regenerateMessage(i)"
            @toggle-sources="chatStore.showSources = !chatStore.showSources"
          />
          <div v-if="chatStore.messages.length === 0" class="text-center text-text-muted mt-16 leading-relaxed">
            <div class="text-lg font-bold mb-2 tracking-tight">有什么我可以帮您的吗？</div>
            <div class="text-xs opacity-80">基于您的知识库文档进行智能问答</div>
          </div>
        </div>
      </div>

      <!-- Sources Panel -->
      <SourcesPanel
        :sources="chatStore.currentSources"
        :visible="chatStore.showSources"
        @close="chatStore.showSources = false"
      />

      <!-- Input -->
      <ChatInput
        :is-generating="chatStore.isGenerating"
        @send="chatStore.sendMessage"
        @stop="chatStore.stopGeneration"
      />
    </div>
  </div>
</template>

<style scoped>
.markdown-body {
  line-height: 1.7;
  color: #e2e8f0;
  font-size: 0.75rem;
}
.markdown-body p { margin: 0.6em 0; }
.markdown-body p:first-child { margin-top: 0; }
.markdown-body p:last-child { margin-bottom: 0; }
.markdown-body h1, .markdown-body h2, .markdown-body h3 { margin: 0.8em 0 0.5em; font-weight: 700; color: #e2e8f0; }
.markdown-body h1 { font-size: 1.25rem; }
.markdown-body h2 { font-size: 1.1rem; }
.markdown-body h3 { font-size: 1rem; }
.markdown-body ul, .markdown-body ol { margin: 0.6em 0; padding-left: 1.5em; }
.markdown-body code { background-color: rgba(255, 255, 255, 0.05); padding: 1px 6px; border-radius: 4px; font-size: 0.85em; color: #e2e8f0; }
.markdown-body .hljs-code-block { background-color: #0B0D13; border-radius: 8px; padding: 12px; overflow-x: auto; margin: 0.6em 0; border: 1px solid #1A1D28; }
.markdown-body .hljs-code-block code { background: none; padding: 0; font-size: 0.85em; line-height: 1.5; }
.markdown-body blockquote { border-left: 3px solid #6366F1; margin: 0.6em 0; padding-left: 12px; color: #94a3b8; }
.markdown-body table { border-collapse: collapse; margin: 0.6em 0; width: 100%; }
.markdown-body th, .markdown-body td { border: 1px solid #1A1D28; padding: 6px 10px; text-align: left; }
.markdown-body th { background-color: #181B25; font-weight: 700; }
.markdown-body a { color: #818CF8; text-decoration: none; }
.markdown-body a:hover { text-decoration: underline; }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/Chat.vue
git commit -m "refactor(chat): rewrite Chat.vue as container composing sub-components"
```

---

### Task 12: TypeScript 验证 + 编译检查

- [ ] **Step 1: 运行 TypeScript 类型检查**

```bash
cd /sessions/69d7be74a5234068df3dcc36/workspace && npx vue-tsc --noEmit
```

Expected: 无类型错误（或仅有已知的环境问题）

- [ ] **Step 2: 如有错误则修复并提交**

---

## 自检清单

| 设计规格 | 对应 Task | 状态 |
|----------|-----------|------|
| 安装 @tanstack/vue-virtual | Task 1 | ✅ |
| 扩展 Message 类型 | Task 1 | ✅ |
| 停止生成后端 API | Task 2 | ✅ |
| 重新生成后端 API | Task 3 | ✅ |
| 前端 API 函数 | Task 4 | ✅ |
| Chat Store | Task 5 | ✅ |
| ThinkingIndicator | Task 6 | ✅ |
| CodeBlock (复制+语言标签) | Task 6 | ✅ |
| MessageActions (DropdownMenu) | Task 6 | ✅ |
| MessageItem (编辑+操作) | Task 7 | ✅ |
| ChatInput (发送/停止切换) | Task 8 | ✅ |
| ConversationSidebar | Task 9 | ✅ |
| SourcesPanel | Task 10 | ✅ |
| Chat.vue 重构为容器 | Task 11 | ✅ |
| TypeScript 验证 | Task 12 | ✅ |
| 虚拟滚动 | 未实现 | ⚠️ 可后续添加 |
| Toast 错误通知 | 未实现 | ⚠️ 用 error ref 替代 |

**注意**：虚拟滚动和 Toast 通知标记为可后续添加，当前版本先用 error ref + 简单滚动实现。
