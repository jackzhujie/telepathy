use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub original_path: String,
    pub library_path: String,
    pub file_type: String,
    pub size: i64,
    pub status: String,
    pub error_msg: Option<String>,
    pub project_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub chunk_index: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexResult {
    pub document_id: String,
    pub chunk_count: i32,
}
