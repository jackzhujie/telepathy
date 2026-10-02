import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import type { AppNotification } from '@/types/notification';
import type { Document, Chunk, IndexResult } from '@/types/document';
import type { Conversation, ChatMessage } from '@/types/chat';
import type { IndexedDocument, ChunkDetail, SearchResult } from '@/types/knowledge';
import type { Project, CreateProjectInput, UpdateProjectInput } from '@/types/project';
import type { RegistryModel, InstalledModel, ModelRecommendation, ModelHubResponse } from '@/types/models';

/**
 * Generic Tauri command invoker
 */
async function invoke<T>(cmd: string, args?: any): Promise<T> {
  console.log(`[API Call] -> ${cmd}`, args || {});
  try {
    const start = performance.now();
    const result = await tauriInvoke<T>(cmd, args);
    const end = performance.now();
    console.log(`[API Return] <- ${cmd} (${(end - start).toFixed(2)}ms)`, result);
    return result;
  } catch (error) {
    console.error(`[API Error] x- ${cmd}`, error);
    throw error;
  }
}

// --- Basic ---
export async function greet(name: string): Promise<string> {
  return invoke<string>('greet', { name });
}

// --- Models ---

export async function listInstalledModels(): Promise<InstalledModel[]> {
  return invoke<InstalledModel[]>('list_installed_models');
}

export const cancelPullModel = (model?: string) => invoke<void>("cancel_pull_model", { model });

export async function listRegistryModels(modelType?: string): Promise<RegistryModel[]> {
  return invoke<RegistryModel[]>('list_registry_models', { modelType });
}


export async function getModelRegistry(): Promise<RegistryModel[]> {
  return invoke<RegistryModel[]>('get_model_registry');
}

export async function getModelRecommendations(): Promise<ModelRecommendation[]> {
  return invoke<ModelRecommendation[]>('get_model_recommendations');
}

export async function getModelHub(forceRefresh: boolean = false): Promise<ModelHubResponse> {
  return invoke<ModelHubResponse>('get_model_hub', { forceRefresh });
}

export async function installModel(modelName: string, variant: string): Promise<void> {
  return invoke<void>('install_model', { modelName, variant });
}

export async function deleteModel(modelName: string): Promise<void> {
  return invoke<void>('delete_model', { modelName });
}

export async function searchModels(query: string): Promise<any[]> {
  return invoke<any[]>('search_hub_models', { query });
}

export async function getModelVariants(modelId: string): Promise<any[]> {
  return invoke<any[]>('get_model_variants', { modelId });
}

// --- Documents ---
export async function importDocument(filePath: string, projectId?: string): Promise<Document> {
  return invoke<Document>('import_document', { filePath, projectId });
}

export async function getDocuments(projectId?: string): Promise<Document[]> {
  return invoke<Document[]>('get_documents', { projectId });
}

export async function getDocumentsByProject(projectId: string): Promise<Document[]> {
  return invoke<Document[]>('get_documents_by_project', { projectId });
}

export async function deleteDocument(docId: string): Promise<void> {
  return invoke<void>('delete_document_cmd', { docId });
}

export async function parseDocument(docId: string): Promise<string> {
  return invoke<string>('parse_document', { docId });
}

export async function scanFolder(folderPath: string): Promise<string[]> {
  return invoke<string[]>('scan_folder', { folderPath });
}

export async function indexDocument(docId: string): Promise<IndexResult> {
  return invoke<IndexResult>('index_document', { docId });
}

export async function getDocumentChunks(docId: string): Promise<Chunk[]> {
  return invoke<Chunk[]>('get_document_chunks', { docId });
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export async function getDocumentsPaginated(projectId?: string, page: number = 1, pageSize: number = 20): Promise<PaginatedResult<any>> {
  return invoke<PaginatedResult<any>>('get_documents_paginated', { projectId, page, pageSize });
}

export async function getIndexedDocumentsPaginated(projectId?: string, page: number = 1, pageSize: number = 20): Promise<PaginatedResult<any>> {
  return invoke<PaginatedResult<any>>('get_indexed_documents_paginated', { projectId, page, pageSize });
}

export async function getDocumentChunksDetailPaginated(docId: string, page: number = 1, pageSize: number = 20): Promise<PaginatedResult<any>> {
  return invoke<PaginatedResult<any>>('get_document_chunks_detail_paginated', { docId, page, pageSize });
}

export async function getIndexedDocuments(projectId?: string): Promise<IndexedDocument[]> {
  return invoke<IndexedDocument[]>('get_indexed_documents', { projectId });
}

export async function getDocumentChunksDetail(docId: string): Promise<ChunkDetail[]> {
  return invoke<ChunkDetail[]>('get_document_chunks_detail', { docId });
}

export async function searchChunks(keyword: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('search_chunks', { keyword });
}

export async function reindexAllDocuments(): Promise<ReindexResult> {
  return invoke<ReindexResult>('reindex_all_documents');
}

export interface ReindexProgress {
  current: number;
  total: number;
  document_name: string;
}

export interface ReindexResult {
  success: boolean;
  total_documents: number;
  indexed_count: number;
  failed_count: number;
  errors: string[];
}

// --- Projects ---
export async function createProject(input: CreateProjectInput): Promise<Project> {
  return invoke<Project>('create_project', { input });
}

export async function listProjects(): Promise<Project[]> {
  return invoke<Project[]>('list_projects');
}

export async function updateProject(input: UpdateProjectInput): Promise<void> {
  return invoke<void>('update_project', { input });
}

export async function deleteProject(id: string): Promise<void> {
  return invoke<void>('delete_project', { id });
}

// --- Chat & Conversations ---
export async function getConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>('get_conversations');
}

export async function getConversationsPaginated(page: number, pageSize: number): Promise<PaginatedResult<Conversation>> {
  return invoke<PaginatedResult<Conversation>>('get_conversations_paginated', { page, pageSize });
}

export async function getMessages(conversationId: string): Promise<ChatMessage[]> {
  return invoke<ChatMessage[]>('get_messages', { conversationId });
}

export async function getMessagesPaginated(conversationId: string, page: number, pageSize: number): Promise<PaginatedResult<ChatMessage>> {
  return invoke<PaginatedResult<ChatMessage>>('get_messages_paginated', { conversationId, page, pageSize });
}

export async function deleteConversation(conversationId: string): Promise<void> {
  return invoke<void>('delete_conversation_cmd', { conversationId });
}

export async function ragQuery(query: string, conversationId?: string, projectId?: string, images?: string[]): Promise<void> {
  return invoke<void>('rag_query', { query, conversationId, projectId, images });
}

export async function stopGeneration(): Promise<void> {
  return invoke<void>('stop_generation');
}

export async function regenerateMessage(conversationId: string, messageId: string): Promise<void> {
  return invoke<void>('regenerate_message', { conversationId, messageId });
}

export async function deleteMessagesAfter(conversationId: string, messageId: string): Promise<void> {
  return invoke<void>('delete_messages_after', { conversationId, messageId });
}

// --- Settings ---
export async function getSettings(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('get_settings');
}

export async function updateSetting(key: string, value: string): Promise<void> {
  return invoke<void>('update_setting_cmd', { key, value });
}

export interface SystemInfo {
  cpu_cores: number;
  memory_gb: number;
  has_gpu: boolean;
  gpu_name: string;
}

export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>('get_system_info');
}

export interface UserProfile {
  name: string;
  gender: string;
  age_group: string;
  occupation: string;
  industry: string;
  language: string;
}

export const getUserProfile = (): Promise<UserProfile> =>
  invoke('get_user_profile');

export const updateUserProfile = (profile: UserProfile): Promise<void> =>
  invoke('update_user_profile', { profile });

export const getUserInterests = (): Promise<string[]> =>
  invoke('get_user_interests');

export const addUserInterest = (interest: string): Promise<void> =>
  invoke('add_user_interest', { interest });

export const removeUserInterest = (interest: string): Promise<void> =>
  invoke('remove_user_interest', { interest });

// --- Notifications ---
export async function getNotifications(): Promise<AppNotification[]> {
  return invoke<AppNotification[]>('get_notifications');
}

export async function getNotificationsPaginated(page: number = 1, pageSize: number = 20): Promise<PaginatedResult<AppNotification>> {
  return invoke<PaginatedResult<AppNotification>>('get_notifications_paginated', { page, pageSize });
}

export async function markNotificationRead(id: string): Promise<void> {
  return invoke<void>('mark_notification_read', { id });
}

export async function getUnreadCount(): Promise<number> {
  return invoke<number>('get_unread_count');
}

// --- Others ---
export async function isSidecarInstalled(): Promise<boolean> {
  return invoke<boolean>('is_sidecar_installed_cmd');
}

// --- Snaps ---
export async function getSnapsPaginated(page: number, pageSize: number): Promise<PaginatedResult<any>> {
  return await invoke('get_snaps_paginated', { page, pageSize });
}

export async function createSnap(content: string, tags?: string): Promise<any> {
  return await invoke('create_snap', { content, tags });
}

export async function updateSnap(id: string, content: string, tags?: string, isPinned?: boolean): Promise<any> {
  return await invoke('update_snap', { id, content, tags, isPinned: isPinned || false });
}

export async function deleteSnap(id: string): Promise<void> {
  return await invoke('delete_snap', { id });
}
