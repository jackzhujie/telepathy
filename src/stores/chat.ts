import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  Message,
  Conversation,
  ChatMessage,
  SearchSource,
  TokenPayload,
  ErrorPayload,
  RagSourcesPayload,
} from '@/types/chat';
import {
  ragQuery,
  getConversations,
  getConversationsPaginated,
  getMessagesPaginated,
  deleteConversation as deleteConversationApi,
  stopGeneration as stopGenerationApi,
  regenerateMessage as regenerateMessageApi,
  deleteMessagesAfter as deleteMessagesAfterApi,
} from '@/api/tauri';
import { renderMarkdown } from '@/utils/markdown';

let msgCounter = 0;

function generateMessageId(): string {
  return `msg-${Date.now()}-${++msgCounter}`;
}

export const useChatStore = defineStore('chat', () => {
  // --- State ---
  const messages = ref<Message[]>([]);
  const messagesPage = ref(1);
  const messagesHasMore = ref(true);
  const conversations = ref<Conversation[]>([]);
  const conversationsPage = ref(1);
  const conversationsHasMore = ref(true);
  const currentConversationId = ref<string | undefined>();
  const isGenerating = ref(false);
  const isThinking = ref(false);
  const currentSources = ref<SearchSource[]>([]);
  const showSources = ref(false);
  const isConversationsLoading = ref(false);
  const isMessagesLoading = ref(false);
  const error = ref<string | null>(null);

  const selectedProjectId = ref<string | null>(null);
  const isSending = ref(false); // 并发发送锁
  const generatingConversationId = ref<string | undefined>();

  // Track unlisten functions for cleanup
  const unlisteners: UnlistenFn[] = [];

  // --- Actions ---

  async function initListeners() {
    const findAssistantMessage = (messageId?: string) => {
      if (messageId) {
        const matched = messages.value.find((m) => m.id === messageId);
        if (matched && matched.role === 'assistant') return matched;
      }
      const lastMsg = messages.value[messages.value.length - 1];
      return lastMsg && lastMsg.role === 'assistant' ? lastMsg : undefined;
    };

    const unlistenToken = await listen<TokenPayload>('chat-token', (event) => {
      if (
        event.payload.conversation_id &&
        currentConversationId.value &&
        event.payload.conversation_id !== currentConversationId.value
      ) {
        return;
      }

      isThinking.value = false;
      const targetMsg = findAssistantMessage(event.payload.message_id);
      if (targetMsg) {
        targetMsg.content += event.payload.token;
        targetMsg.status = 'streaming';
      }
    });

    const unlistenDone = await listen<{ conversation_id?: string; message_id?: string }>('chat-done', (event) => {
      // Always unlock the UI globally
      isGenerating.value = false;
      isThinking.value = false;
      generatingConversationId.value = undefined;

      if (
        event.payload.conversation_id &&
        currentConversationId.value &&
        event.payload.conversation_id !== currentConversationId.value
      ) {
        return;
      }

      const targetMsg = findAssistantMessage(event.payload.message_id);
      if (targetMsg) {
        if (!targetMsg.content) {
          targetMsg.status = 'error';
          targetMsg.error = '模型未能生成回复，请重试。';
        } else {
          targetMsg.status = 'done';
        }
      }
    });

    const unlistenError = await listen<ErrorPayload>('chat-error', (event) => {
      // Always unlock the UI globally
      isGenerating.value = false;
      isThinking.value = false;
      generatingConversationId.value = undefined;

      if (
        event.payload.conversation_id &&
        currentConversationId.value &&
        event.payload.conversation_id !== currentConversationId.value
      ) {
        return;
      }

      const targetMsg = findAssistantMessage(event.payload.message_id);
      if (targetMsg) {
        targetMsg.status = 'error';
        targetMsg.error = event.payload.error;
      }
      error.value = event.payload.error;
    });

    const unlistenSources = await listen<RagSourcesPayload>('rag-sources', (event) => {
      if (
        event.payload.conversation_id &&
        currentConversationId.value &&
        event.payload.conversation_id !== currentConversationId.value
      ) {
        return;
      }

      currentSources.value = event.payload.sources;
      showSources.value = true;
      const targetMsg = findAssistantMessage(event.payload.message_id);
      if (targetMsg) {
        targetMsg.sources = event.payload.sources;
      }
    });

    const unlistenChatStarted = await listen<{ conversation_id: string; assistant_message_id: string; user_message_id: string }>(
      'chat-started',
      (event) => {
        currentConversationId.value = event.payload.conversation_id;
        
        // Update assistant placeholder ID
        const lastMsg = messages.value[messages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant' && lastMsg.content === '') {
          lastMsg.id = event.payload.assistant_message_id;
        }

        // Update user message ID
        const userMsg = messages.value[messages.value.length - 2];
        if (userMsg && userMsg.role === 'user' && userMsg.id.startsWith('msg-')) {
          userMsg.id = event.payload.user_message_id;
        }
      },
    );

    const unlistenConversationCreated = await listen<{ id: string; title: string }>(
      'conversation-created',
      (event) => {
        currentConversationId.value = event.payload.id;
        
        // Ensure it appears in the sidebar list immediately
        const exists = conversations.value.find(c => c.id === event.payload.id);
        if (!exists) {
          conversations.value.unshift({
            id: event.payload.id,
            title: event.payload.title,
            created_at: new Date().toISOString(),
          });
        }
      },
    );

    unlisteners.push(
      unlistenToken,
      unlistenDone,
      unlistenError,
      unlistenSources,
      unlistenChatStarted,
      unlistenConversationCreated,
    );
  }

  function cleanup() {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    unlisteners.length = 0;
  }

  async function fetchConversations() {
    conversations.value = await getConversations();
  }

  async function fetchConversationsPaginated(reset: boolean = true) {
    if (isConversationsLoading.value) return;
    isConversationsLoading.value = true;
    
    try {
      if (reset) {
        conversationsPage.value = 1;
        conversationsHasMore.value = true;
      }
      
      const result = await getConversationsPaginated(conversationsPage.value, 20);
      if (reset) {
        conversations.value = result.items;
      } else {
        conversations.value = [...conversations.value, ...result.items];
      }
      conversationsHasMore.value = result.hasMore;
      conversationsPage.value++;
    } finally {
      isConversationsLoading.value = false;
    }
  }

  async function loadMoreConversations() {
    if (isConversationsLoading.value || !conversationsHasMore.value) return;
    await fetchConversationsPaginated(false);
  }

  async function loadConversation(id: string) {
    currentConversationId.value = id;
    messages.value = [];
    messagesPage.value = 1;
    messagesHasMore.value = true;
    error.value = null;
    currentSources.value = [];
    showSources.value = false;

    await loadConversationPaginated(id);
  }

  async function loadConversationPaginated(id: string, reset: boolean = true) {
    if (isMessagesLoading.value) return;
    isMessagesLoading.value = true;
    
    try {
      if (reset) {
        messagesPage.value = 1;
        messagesHasMore.value = true;
      }
      
      const result = await getMessagesPaginated(id, messagesPage.value, 20);
      const newMessages = result.items.map((msg: ChatMessage, index: number) => {
        const isLastAssistant = msg.role === 'assistant' && index === result.items.length - 1;
        const isCurrentGenerating = (id === generatingConversationId.value) || (isGenerating.value && id === currentConversationId.value);
        const status: 'done' | 'streaming' | 'sending' | 'error' = (isLastAssistant && isCurrentGenerating) 
          ? (msg.content ? 'streaming' : 'sending') 
          : 'done';

        return {
          id: msg.id,
          role: msg.role as 'user' | 'assistant',
          content: msg.content,
          sources: msg.sources ? JSON.parse(msg.sources) : undefined,
          images: msg.images ? JSON.parse(msg.images) : undefined,
          status,
          timestamp: new Date(msg.created_at).getTime(),
        };
      });
      
      if (reset) {
        messages.value = newMessages;
      } else {
        // Deduplicate messages before prepending
        const existingIds = new Set(messages.value.map(m => m.id));
        const filteredNewMessages = newMessages.filter(m => !existingIds.has(m.id));
        messages.value = [...filteredNewMessages, ...messages.value];
      }
      
      messagesHasMore.value = result.hasMore;
      messagesPage.value++;
    } finally {
      isMessagesLoading.value = false;
    }
  }

  async function loadMoreMessages() {
    if (isMessagesLoading.value || !messagesHasMore.value || !currentConversationId.value) return;
    await loadConversationPaginated(currentConversationId.value, false);
  }

  async function newConversation() {
    messages.value = [];
    messagesPage.value = 1;
    messagesHasMore.value = true;
    currentConversationId.value = undefined;
    error.value = null;
    currentSources.value = [];
    showSources.value = false;
    isGenerating.value = false;
    isThinking.value = false;
  }

  async function deleteConversation(id: string) {
    await deleteConversationApi(id);
    conversations.value = conversations.value.filter((c) => c.id !== id);
    if (currentConversationId.value === id) {
      await newConversation();
    }
  }

  async function sendMessage(prompt: string, images?: string[]) {
    if (isSending.value || isGenerating.value) return;

    // Auto-fill default prompt when sending images without text
    let finalPrompt = prompt.trim();
    if (!finalPrompt && images && images.length > 0) {
      finalPrompt = '请描述这张图片的内容';
    }
    if (!finalPrompt) return;

    const lastMsg = messages.value[messages.value.length - 1];
    if (lastMsg && lastMsg.role === 'user' && lastMsg.content === finalPrompt && Date.now() - (lastMsg.timestamp || 0) < 3000) {
      return;
    }

    isSending.value = true;
    error.value = null;
    isGenerating.value = true;
    isThinking.value = true;
    generatingConversationId.value = currentConversationId.value;
    showSources.value = false;
    currentSources.value = [];

    const userMsg: Message = {
      id: generateMessageId(),
      role: 'user',
      content: finalPrompt,
      images: images,
      status: 'done',
      timestamp: Date.now(),
    };
    messages.value.push(userMsg);

    const assistantMsg: Message = {
      id: generateMessageId(),
      role: 'assistant',
      content: '',
      status: 'sending',
      timestamp: Date.now(),
    };
    messages.value.push(assistantMsg);

    try {
      await ragQuery(finalPrompt, currentConversationId.value, selectedProjectId.value ?? undefined, images);
      // 注意：ragQuery 是异步返回的，后端 spawn 后就会立即完成。
      // 我们通过 isGenerating 持续锁定 UI。
    } catch (e) {
      isGenerating.value = false;
      isThinking.value = false;
      // If the messages were never persisted to the database (still have
      // frontend-generated IDs), remove them so the user doesn't see orphans.
      if (assistantMsg.id.startsWith('msg-')) {
        messages.value = messages.value.filter(
          (m) => m.id !== userMsg.id && m.id !== assistantMsg.id,
        );
      } else {
        const lastMsg = messages.value[messages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          lastMsg.status = 'error';
          lastMsg.error = String(e);
        }
      }
      error.value = String(e);
    } finally {
      // 延迟释放锁定，确保 UI 状态已同步
      setTimeout(() => {
        isSending.value = false;
      }, 500);
    }
  }

  async function stopGeneration() {
    await stopGenerationApi();
    isGenerating.value = false;
    isThinking.value = false;
    const lastMsg = messages.value[messages.value.length - 1];
    if (lastMsg && lastMsg.role === 'assistant' && lastMsg.status === 'sending') {
      lastMsg.status = 'done';
    } else if (lastMsg && lastMsg.role === 'assistant' && lastMsg.status === 'streaming') {
      lastMsg.status = 'done';
    }
  }

  async function regenerateMessage(index: number) {
    const msg = messages.value[index];
    if (!msg || msg.role !== 'assistant') return;

    // Slice messages up to (not including) the assistant message at index
    messages.value = messages.value.slice(0, index);
    error.value = null;
    isGenerating.value = true;
    isThinking.value = true;
    showSources.value = false;
    currentSources.value = [];

    // Push empty assistant message placeholder
    const assistantMsg = {
      id: generateMessageId(),
      role: 'assistant' as const,
      content: '',
      status: 'sending' as const,
      timestamp: Date.now(),
    };
    messages.value.push(assistantMsg);

    try {
      await regenerateMessageApi(
        currentConversationId.value!,
        msg.id,
      );
    } catch (e) {
      isGenerating.value = false;
      isThinking.value = false;
      const lastMsg = messages.value[messages.value.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        lastMsg.status = 'error';
        lastMsg.error = String(e);
      }
      error.value = String(e);
    }
  }

  async function editAndResend(index: number, newContent: string) {
    const msg = messages.value[index];
    if (!msg || msg.role !== 'user' || !currentConversationId.value) return;

    try {
      // 1. 同步删除后端数据库中该消息及之后的所有内容
      await deleteMessagesAfterApi(currentConversationId.value, msg.id);
      
      // 2. 切分前端数组
      messages.value = messages.value.slice(0, index);

      // 3. 发送新消息（保留原图片）
      await sendMessage(newContent, msg.images);
    } catch (e) {
      error.value = `编辑失败: ${e}`;
    }
  }

  function copyMessage(index: number): string {
    const msg = messages.value[index];
    if (!msg) return '';
    const html = renderMarkdown(msg.content);
    const div = document.createElement('div');
    div.innerHTML = html;
    return div.textContent ?? '';
  }

  function copyRawMessage(index: number): string {
    const msg = messages.value[index];
    if (!msg) return '';
    return msg.content;
  }

  return {
    // State
    messages,
    messagesPage,
    messagesHasMore,
    conversations,
    conversationsPage,
    conversationsHasMore,
    currentConversationId,
    isGenerating,
    isThinking,
    currentSources,
    showSources,
    error,
    isConversationsLoading,
    isMessagesLoading,
    selectedProjectId,
    // Actions
    initListeners,
    cleanup,
    fetchConversations,
    fetchConversationsPaginated,
    loadMoreConversations,
    loadConversation,
    loadConversationPaginated,
    loadMoreMessages,
    newConversation,
    deleteConversation,
    sendMessage,
    stopGeneration,
    regenerateMessage,
    editAndResend,
    copyMessage,
    copyRawMessage,
  };
});
