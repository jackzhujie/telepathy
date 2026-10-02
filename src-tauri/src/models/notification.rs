use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: String,
}
