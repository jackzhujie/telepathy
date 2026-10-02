use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedDocument {
    pub id: String,
    pub name: String,
    pub file_type: String,
    pub size: i64,
    pub chunk_count: i32,
    pub created_at: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkDetail {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub chunk_index: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub document_name: String,
    pub content: String,
    pub score: f64,
    pub chunk_index: i32,
    pub document_id: String,
}
