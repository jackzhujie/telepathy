#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub created_at: String,
}

pub fn init_memory_tables(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create memories table: {}", e)))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_embeddings (
            memory_id TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            dimension INTEGER NOT NULL,
            FOREIGN KEY(memory_id) REFERENCES memories(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create memory embeddings table: {}", e)))?;

    Ok(())
}

pub fn insert_memory(conn: &Connection, memory: &Memory) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO memories (id, content, created_at)
         VALUES (?1, ?2, ?3)",
        params![memory.id, memory.content, memory.created_at,],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert memory: {}", e)))?;
    Ok(())
}

pub fn insert_memory_embedding(
    conn: &Connection,
    memory_id: &str,
    embedding: &[f32],
) -> Result<(), AppError> {
    let bytes = crate::db::vectors::f32_vec_to_bytes(embedding);
    conn.execute(
        "INSERT OR REPLACE INTO memory_embeddings (memory_id, embedding, dimension) VALUES (?1, ?2, ?3)",
        params![memory_id, bytes, embedding.len()],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert memory embedding: {}", e)))?;
    Ok(())
}

pub fn search_similar_memories(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
) -> Result<Vec<(Memory, f32)>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT m.id, m.content, m.created_at, e.embedding
             FROM memories m
             JOIN memory_embeddings e ON m.id = e.memory_id",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare memory search query: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            let embedding_bytes: Vec<u8> = row.get(3)?;
            let embedding = crate::db::vectors::bytes_to_f32_vec(&embedding_bytes);
            let score = crate::db::vectors::cosine_similarity(query_embedding, &embedding);

            Ok((
                Memory {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    created_at: row.get(2)?,
                },
                score,
            ))
        })
        .map_err(|e| AppError::Internal(format!("Failed to query memories: {}", e)))?;

    let mut results: Vec<_> = rows.filter_map(|r| r.ok()).collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    results.truncate(limit);
    Ok(results)
}

pub fn get_all_memories(conn: &Connection) -> Result<Vec<Memory>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, content, created_at FROM memories ORDER BY created_at DESC")
        .map_err(|e| {
            AppError::Internal(format!("Failed to prepare get all memories query: {}", e))
        })?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Memory {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query all memories: {}", e)))?;

    let mut memories = Vec::new();
    for row in rows {
        memories.push(
            row.map_err(|e| AppError::Internal(format!("Failed to read memory row: {}", e)))?,
        );
    }
    Ok(memories)
}

pub fn delete_memory(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM memories WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(format!("Failed to delete memory: {}", e)))?;
    Ok(())
}
