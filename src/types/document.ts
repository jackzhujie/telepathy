export interface Document {
  id: string;
  name: string;
  original_path: string;
  library_path: string;
  file_type: string;
  size: number;
  status: 'pending' | 'parsing' | 'done' | 'error' | 'indexed';
  error_msg: string | null;
  created_at: string;
}

export interface Chunk {
  id: string;
  document_id: string;
  chunk_index: number;
  content: string;
  metadata: string | null;
  created_at: string;
}

export interface IndexResult {
  chunk_count: number;
  document_id: string;
}
