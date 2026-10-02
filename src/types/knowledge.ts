export interface IndexedDocument {
  id: string;
  name: string;
  file_type: string;
  chunk_count: number;
  created_at: string;
}

export interface ChunkDetail {
  id: string;
  document_id: string;
  chunk_index: number;
  content: string;
  created_at: string;
}

export interface SearchResult {
  chunk: ChunkDetail;
  document_name: string;
}