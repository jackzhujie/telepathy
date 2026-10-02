export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  images?: string[];  // base64 encoded images
  sources?: SearchSource[];
  status: 'sending' | 'streaming' | 'done' | 'error';
  error?: string;
  timestamp: number;
  isEditing?: boolean;
}

export interface TokenPayload {
  conversation_id?: string;
  message_id?: string;
  token: string;
}

export interface ErrorPayload {
  conversation_id?: string;
  message_id?: string;
  error: string;
}

export interface SearchSource {
  chunk_id: string;
  document_name: string;
  chunk_index: number;
  content: string;
  score: number;
}

export interface RagSourcesPayload {
  conversation_id?: string;
  message_id?: string;
  sources: SearchSource[];
}

export interface Conversation {
  id: string;
  title: string | null;
  created_at: string;
}

export interface ChatMessage {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  sources: string | null;
  images: string | null;
  created_at: string;
}
