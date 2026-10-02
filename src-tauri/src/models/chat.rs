use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sources: Option<String>,
    pub images: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPayload {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RagSourcesPayload {
    pub sources: Vec<SearchSource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchSource {
    pub chunk_id: String,
    pub document_name: String,
    pub chunk_index: i32,
    pub content: String,
    pub score: f64,
}
