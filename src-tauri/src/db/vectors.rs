#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[derive(Debug)]
pub struct ChunkWithScore {
    pub chunk: Chunk,
    pub document_name: String,
    pub score: f32,
}

pub fn init_vector_tables(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunks (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            content TEXT NOT NULL,
            metadata TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create chunks table: {}", e)))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunk_embeddings (
            chunk_id TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            dimension INTEGER NOT NULL,
            FOREIGN KEY(chunk_id) REFERENCES chunks(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create embeddings table: {}", e)))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id)",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create index: {}", e)))?;

    Ok(())
}

pub fn insert_chunk(conn: &Connection, chunk: &Chunk) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO chunks (id, document_id, chunk_index, content, metadata, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            chunk.id,
            chunk.document_id,
            chunk.chunk_index,
            chunk.content,
            chunk.metadata,
            chunk.created_at,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert chunk: {}", e)))?;
    Ok(())
}

pub fn get_chunks_by_document(conn: &Connection, doc_id: &str) -> Result<Vec<Chunk>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, document_id, chunk_index, content, metadata, created_at
             FROM chunks WHERE document_id = ?1 ORDER BY chunk_index",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map(params![doc_id], |row| {
            Ok(Chunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_index: row.get(2)?,
                content: row.get(3)?,
                metadata: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query chunks: {}", e)))?;

    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }
    Ok(chunks)
}

pub fn insert_embedding(
    conn: &Connection,
    chunk_id: &str,
    embedding: &[f32],
) -> Result<(), AppError> {
    let bytes = f32_vec_to_bytes(embedding);
    conn.execute(
        "INSERT OR REPLACE INTO chunk_embeddings (chunk_id, embedding, dimension) VALUES (?1, ?2, ?3)",
        params![chunk_id, bytes, embedding.len()],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert embedding: {}", e)))?;
    Ok(())
}

pub fn delete_chunks_by_document(conn: &Connection, doc_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM chunk_embeddings WHERE chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)",
        params![doc_id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to delete embeddings: {}", e)))?;

    conn.execute("DELETE FROM chunks WHERE document_id = ?1", params![doc_id])
        .map_err(|e| AppError::Internal(format!("Failed to delete chunks: {}", e)))?;

    Ok(())
}

pub fn search_similar(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
    project_id: Option<String>,
) -> Result<Vec<ChunkWithScore>, AppError> {
    let mut results: Vec<ChunkWithScore> = if let Some(ref pid) = project_id {
        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.document_id, c.chunk_index, c.content, c.metadata, c.created_at, e.embedding, d.name
                 FROM chunks c
                 JOIN chunk_embeddings e ON c.id = e.chunk_id
                 JOIN documents d ON c.document_id = d.id
                 WHERE d.project_id = ?1",
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare search query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![pid], |row| {
                let embedding_bytes: Vec<u8> = row.get(6)?;
                let embedding = bytes_to_f32_vec(&embedding_bytes);
                let score = cosine_similarity(query_embedding, &embedding);

                Ok(ChunkWithScore {
                    chunk: Chunk {
                        id: row.get(0)?,
                        document_id: row.get(1)?,
                        chunk_index: row.get(2)?,
                        content: row.get(3)?,
                        metadata: row.get(4)?,
                        created_at: row.get(5)?,
                    },
                    document_name: row.get(7)?,
                    score,
                })
            })
            .map_err(|e| AppError::Internal(format!("Failed to search: {}", e)))?;

        rows.filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.document_id, c.chunk_index, c.content, c.metadata, c.created_at, e.embedding, d.name
                 FROM chunks c
                 JOIN chunk_embeddings e ON c.id = e.chunk_id
                 JOIN documents d ON c.document_id = d.id",
            )
            .map_err(|e| AppError::Internal(format!("Failed to prepare search query: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                let embedding_bytes: Vec<u8> = row.get(6)?;
                let embedding = bytes_to_f32_vec(&embedding_bytes);
                let score = cosine_similarity(query_embedding, &embedding);

                Ok(ChunkWithScore {
                    chunk: Chunk {
                        id: row.get(0)?,
                        document_id: row.get(1)?,
                        chunk_index: row.get(2)?,
                        content: row.get(3)?,
                        metadata: row.get(4)?,
                        created_at: row.get(5)?,
                    },
                    document_name: row.get(7)?,
                    score,
                })
            })
            .map_err(|e| AppError::Internal(format!("Failed to search: {}", e)))?;

        rows.filter_map(|r| r.ok()).collect()
    };

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);

    Ok(results)
}

pub fn get_chunk_count(conn: &Connection, doc_id: &str) -> Result<i64, AppError> {
    conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE document_id = ?1",
        params![doc_id],
        |row| row.get(0),
    )
    .map_err(|e| AppError::Internal(format!("Failed to count chunks: {}", e)))
}

pub fn f32_vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for &f in v {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

pub fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

pub fn search_similar_hnsw(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
    project_id: Option<String>,
    threshold: f32,
) -> Result<Vec<ChunkWithScore>, AppError> {
    use crate::services::hnsw_index::with_hnsw_manager;

    let candidates = with_hnsw_manager(|manager| {
        manager.search(query_embedding, limit, query_embedding.len(), project_id.as_deref())
    })
    .map_err(|e| AppError::Internal(format!("HNSW manager: {}", e)))?
    .map_err(|e| AppError::Internal(format!("HNSW search failed: {}", e)))?;

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let ids: Vec<String> = candidates.iter().map(|(id, _)| id.clone()).collect();

    let placeholders: String = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let mut sql = format!(
        "SELECT c.id, c.document_id, c.chunk_index, c.content, c.metadata, c.created_at, d.name
         FROM chunks c
         JOIN documents d ON c.document_id = d.id
         WHERE c.id IN ({})",
        placeholders
    );

    if project_id.is_some() {
        sql.push_str(" AND d.project_id = ?");
    }

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;

    let mut params: Vec<&dyn rusqlite::ToSql> =
        ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    if let Some(ref pid) = project_id {
        params.push(pid as &dyn rusqlite::ToSql);
    }

    let mut results = Vec::new();
    let mut rows = stmt
        .query(params.as_slice())
        .map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?;

    while let Some(row) = rows.next().map_err(|e| AppError::Internal(e.to_string()))? {
        let id: String = row.get(0).map_err(|e| AppError::Internal(e.to_string()))?;
        let score = candidates
            .iter()
            .find(|(cid, _)| cid == &id)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);

        if score >= threshold {
            results.push(ChunkWithScore {
                chunk: Chunk {
                    id,
                    document_id: row.get(1).map_err(|e| AppError::Internal(e.to_string()))?,
                    chunk_index: row.get(2).map_err(|e| AppError::Internal(e.to_string()))?,
                    content: row.get(3).map_err(|e| AppError::Internal(e.to_string()))?,
                    metadata: row.get(4).map_err(|e| AppError::Internal(e.to_string()))?,
                    created_at: row.get(5).map_err(|e| AppError::Internal(e.to_string()))?,
                },
                document_name: row.get(6).map_err(|e| AppError::Internal(e.to_string()))?,
                score,
            });
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_vector_tables(&conn).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                project_id TEXT,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO documents (id, project_id, name, path) VALUES ('d1', 'p1', 'doc-1', 'path-1')",
            [],
        ).unwrap();
        conn
    }

    #[test]
    fn test_init_tables() {
        let conn = setup_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_insert_and_query_chunks() {
        let conn = setup_db();
        let chunk = Chunk {
            id: "test-1".to_string(),
            document_id: "doc-1".to_string(),
            chunk_index: 0,
            content: "Hello world".to_string(),
            metadata: None,
            created_at: "2026-01-01".to_string(),
        };
        insert_chunk(&conn, &chunk).unwrap();
        let chunks = get_chunks_by_document(&conn, "doc-1").unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Hello world");
    }

    #[test]
    fn test_embedding_storage() {
        let conn = setup_db();
        let chunk = Chunk {
            id: "test-2".to_string(),
            document_id: "doc-1".to_string(),
            chunk_index: 0,
            content: "Test".to_string(),
            metadata: None,
            created_at: "2026-01-01".to_string(),
        };
        insert_chunk(&conn, &chunk).unwrap();
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        insert_embedding(&conn, "test-2", &embedding).unwrap();
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let score = cosine_similarity(&a, &b);
        assert!((score - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        let score2 = cosine_similarity(&a, &c);
        assert!((score2 - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_search() {
        let conn = setup_db();
        let chunk1 = Chunk {
            id: "c1".to_string(),
            document_id: "d1".to_string(),
            chunk_index: 0,
            content: "Similar content".to_string(),
            metadata: None,
            created_at: "2026-01-01".to_string(),
        };
        let chunk2 = Chunk {
            id: "c2".to_string(),
            document_id: "d1".to_string(),
            chunk_index: 1,
            content: "Different content".to_string(),
            metadata: None,
            created_at: "2026-01-01".to_string(),
        };
        insert_chunk(&conn, &chunk1).unwrap();
        insert_chunk(&conn, &chunk2).unwrap();
        insert_embedding(&conn, "c1", &[1.0, 0.0, 0.0]).unwrap();
        insert_embedding(&conn, "c2", &[0.0, 1.0, 0.0]).unwrap();

        let results = search_similar(&conn, &[1.0, 0.0, 0.0], 2, None).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].chunk.id, "c1");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_delete_by_document() {
        let conn = setup_db();
        let chunk = Chunk {
            id: "del-1".to_string(),
            document_id: "doc-del".to_string(),
            chunk_index: 0,
            content: "To delete".to_string(),
            metadata: None,
            created_at: "2026-01-01".to_string(),
        };
        insert_chunk(&conn, &chunk).unwrap();
        delete_chunks_by_document(&conn, "doc-del").unwrap();
        let chunks = get_chunks_by_document(&conn, "doc-del").unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_real_db_similarity() {
        let path = "/Users/mac/Library/Application Support/com.telepathy.app/telepathy.db";
        println!("Checking path: {}", path);
        assert!(std::path::Path::new(path).exists(), "DB does not exist");
        let conn = Connection::open(path).unwrap();
        let mut stmt = conn
            .prepare("SELECT embedding FROM chunk_embeddings LIMIT 1")
            .unwrap();
        let mut rows = stmt
            .query_map([], |row| {
                let bytes: Vec<u8> = row.get(0)?;
                Ok(bytes_to_f32_vec(&bytes))
            })
            .unwrap();
        if let Some(Ok(embedding)) = rows.next() {
            println!(
                "[TEST REAL DB SIMILARITY] Real embedding len = {}",
                embedding.len()
            );
            let score = cosine_similarity(&embedding, &embedding);
            println!(
                "[TEST REAL DB SIMILARITY] Real score with itself = {}",
                score
            );
        }
    }
}
