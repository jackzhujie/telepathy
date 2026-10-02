use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Snap {
    pub id: String,
    pub content: String,
    pub tags: Option<String>,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub fn init_snaps_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS snaps (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            tags TEXT,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    // 新增：随记向量表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS snap_embeddings (
            snap_id TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            dimension INTEGER NOT NULL,
            FOREIGN KEY(snap_id) REFERENCES snaps(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

pub fn insert_snap(conn: &Connection, snap: &Snap) -> Result<()> {
    conn.execute(
        "INSERT INTO snaps (id, content, tags, is_pinned, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            snap.id,
            snap.content,
            snap.tags,
            if snap.is_pinned { 1 } else { 0 },
            snap.created_at,
            snap.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update_snap(conn: &Connection, snap: &Snap) -> Result<()> {
    conn.execute(
        "UPDATE snaps SET content = ?1, tags = ?2, is_pinned = ?3, updated_at = ?4 WHERE id = ?5",
        params![
            snap.content,
            snap.tags,
            if snap.is_pinned { 1 } else { 0 },
            snap.updated_at,
            snap.id,
        ],
    )?;
    Ok(())
}

pub fn delete_snap(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM snaps WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_snaps_paginated(conn: &Connection, limit: i32, offset: i32) -> Result<Vec<Snap>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, tags, is_pinned, created_at, updated_at
         FROM snaps
         ORDER BY is_pinned DESC, created_at DESC
         LIMIT ?1 OFFSET ?2",
    )?;
    let snaps = stmt.query_map(params![limit, offset], |row| {
        Ok(Snap {
            id: row.get(0)?,
            content: row.get(1)?,
            tags: row.get(2)?,
            is_pinned: row.get::<_, i32>(3)? != 0,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    snaps.collect()
}

pub fn insert_snap_embedding(conn: &Connection, snap_id: &str, embedding: &[f32]) -> Result<()> {
    let bytes = crate::db::vectors::f32_vec_to_bytes(embedding);
    conn.execute(
        "INSERT OR REPLACE INTO snap_embeddings (snap_id, embedding, dimension) VALUES (?1, ?2, ?3)",
        params![snap_id, bytes, embedding.len()],
    )?;
    Ok(())
}

pub fn search_similar_snaps(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
) -> Result<Vec<(Snap, f32)>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.content, s.tags, s.is_pinned, s.created_at, s.updated_at, e.embedding
         FROM snaps s
         JOIN snap_embeddings e ON s.id = e.snap_id",
    )?;

    let rows = stmt.query_map([], |row| {
        let embedding_bytes: Vec<u8> = row.get(6)?;
        let embedding = crate::db::vectors::bytes_to_f32_vec(&embedding_bytes);
        let score = crate::db::vectors::cosine_similarity(query_embedding, &embedding);

        Ok((
            Snap {
                id: row.get(0)?,
                content: row.get(1)?,
                tags: row.get(2)?,
                is_pinned: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            },
            score,
        ))
    })?;

    let mut results: Vec<_> = rows.filter_map(|r| r.ok()).collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);

    Ok(results)
}

pub fn get_total_count(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM snaps")?;
    stmt.query_row([], |row| row.get(0))
}
