use crate::db::vectors::Chunk;
use crate::db::{documents, settings, vectors};
use crate::errors::AppError;
use crate::services::{chunker, embedder, hnsw_index::with_hnsw_manager, parser};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReindexProgress {
    pub current: usize,
    pub total: usize,
    pub document_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReindexResult {
    pub success: bool,
    pub total_documents: usize,
    pub indexed_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DocRow {
    id: String,
    name: String,
    library_path: String,
    file_type: String,
    project_id: Option<String>,
}

fn get_documents_to_reindex(db_path: &std::path::Path) -> Result<Vec<DocRow>, AppError> {
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&conn)?;
    vectors::init_vector_tables(&conn)?;
    settings::init_settings_table(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, name, library_path, file_type, project_id FROM documents WHERE status = 'indexed' OR status = 'done'")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let doc_rows: Vec<DocRow> = stmt
        .query_map([], |row| {
            Ok(DocRow {
                id: row.get(0)?,
                name: row.get(1)?,
                library_path: row.get(2)?,
                file_type: row.get(3)?,
                project_id: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(doc_rows)
}

#[tauri::command]
pub async fn reindex_all_documents(app_handle: AppHandle) -> Result<ReindexResult, AppError> {
    // 卸载 Chat 大模型以释放 VRAM 空间，为 Embedding 重建索引腾出空间
    let engine_state = app_handle.state::<crate::services::inference::EngineManagerState>();
    {
        let mut guard = engine_state.0.lock().await;
        if let Some(engine) = guard.as_mut() {
            if let Err(e) = engine.unload().await {
                println!("[Reindexing] Warning: failed to unload chat engine: {}", e);
            } else {
                println!("[Reindexing] Successfully unloaded chat engine to free VRAM");
            }
        }
    }

    let db_path = {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;

        if !app_data_dir.exists() {
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
        }
        app_data_dir.join("telepathy.db")
    };

    let settings_conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    settings::init_settings_table(&settings_conn)?;
    settings::seed_defaults(&settings_conn)?;

    let embedding_model = settings::get_setting(&settings_conn, "embedding_model")?
        .unwrap_or_else(|| "bge-large-zh".to_string());

    drop(settings_conn);

    let doc_rows = get_documents_to_reindex(&db_path)?;

    let total = doc_rows.len();
    let mut indexed_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    let embedding_model_path = app_data_dir.join("downloads").join(&embedding_model);

    if !embedding_model_path.exists() {
        return Err(AppError::Internal(format!(
            "Embedding model not found: {:?}",
            embedding_model_path
        )));
    }
    let emb = embedder::Embedder::new(&embedding_model_path.to_string_lossy());

    for (index, doc) in doc_rows.into_iter().enumerate() {
        let progress = ReindexProgress {
            current: index + 1,
            total,
            document_name: doc.name.clone(),
        };
        let _ = app_handle.emit("reindex-progress", &progress);

        match reindex_single_document(&db_path, &doc.id, &doc.library_path, &doc.project_id, &emb, &app_handle).await {
            Ok(_) => {
                indexed_count += 1;
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("{}: {}", doc.name, e));
            }
        }
    }

    let result = ReindexResult {
        success: failed_count == 0,
        total_documents: total,
        indexed_count,
        failed_count,
        errors,
    };

    let _ = app_handle.emit("reindex-complete", &result);

    Ok(result)
}

async fn reindex_single_document(
    db_path: &std::path::Path,
    doc_id: &str,
    file_path: &str,
    project_id: &Option<String>,
    emb: &embedder::Embedder,
    app_handle: &AppHandle,
) -> Result<(), AppError> {
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    vectors::delete_chunks_by_document(&conn, doc_id)
        .map_err(|e| AppError::Internal(format!("Failed to delete old chunks: {}", e)))?;

    let file_path_owned = file_path.to_string();
    let text = parser::parse_document_async(std::path::Path::new(&file_path_owned), app_handle).await?;

    let config = chunker::ChunkConfig::default();
    let text_chunks = chunker::chunk_text(&text, &config);

    if text_chunks.is_empty() {
        documents::update_document_status(&conn, doc_id, "indexed", None)?;
        return Ok(());
    }

    // Insert chunks and generate embeddings in sized batches (size 16) to avoid lock contention
    const BATCH_SIZE: usize = 16;
    for (batch_idx, chunk_slice) in text_chunks.chunks(BATCH_SIZE).enumerate() {
        // Batch embed current slice
        let embeddings = match emb.embed_batch(chunk_slice).await {
            Ok(embs) => Some(embs),
            Err(e) => {
                println!("[Reindexing] Warning: batch embedding failed for batch {}: {}", batch_idx, e);
                None
            }
        };

        // Write chunks and corresponding embeddings in this batch
        for (i, chunk_content) in chunk_slice.iter().enumerate() {
            let global_idx = batch_idx * BATCH_SIZE + i;
            let chunk_id = Uuid::new_v4().to_string();
            let chunk = Chunk {
                id: chunk_id.clone(),
                document_id: doc_id.to_string(),
                chunk_index: global_idx as i32,
                content: chunk_content.clone(),
                metadata: None,
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };

            vectors::insert_chunk(&conn, &chunk)
                .map_err(|e| AppError::Internal(format!("Failed to insert chunk: {}", e)))?;

            if let Some(ref embs) = embeddings {
                if let Some(embedding) = embs.get(i) {
                    vectors::insert_embedding(&conn, &chunk_id, embedding)
                        .map_err(|e| AppError::Internal(format!("Failed to insert embedding: {}", e)))?;
                }
            }
        }

        // Yield lock and CPU control to allow concurrent tasks (e.g., chat) to run
        if (batch_idx + 1) * BATCH_SIZE < text_chunks.len() {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }

    let dimension = emb.get_dimension().await?;
    let rebuild_result = with_hnsw_manager(|manager| {
        manager.rebuild_index(project_id.as_deref(), &conn, dimension)
    });
    if let Err(e) = rebuild_result.and_then(|r| r) {
        println!("[HNSW] Warning: failed to rebuild index: {}", e);
    }

    documents::update_document_status(&conn, doc_id, "indexed", None)
        .map_err(|e| AppError::Internal(format!("Failed to update status: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn clear_all_data(app_handle: AppHandle) -> Result<(), AppError> {
    // 1. Clear physical files in library
    let library_dir = crate::services::storage::get_library_dir(&app_handle)?;
    if library_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&library_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }
    }

    // 2. Clear database tables
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    let db_path = app_data_dir.join("telepathy.db");

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    // Truncate tables Related to Knowledge Base
    conn.execute("DELETE FROM chunk_embeddings", [])
        .map_err(|e| AppError::Internal(format!("Failed to clear embeddings: {}", e)))?;
    conn.execute("DELETE FROM chunks", [])
        .map_err(|e| AppError::Internal(format!("Failed to clear chunks: {}", e)))?;
    conn.execute("DELETE FROM documents", [])
        .map_err(|e| AppError::Internal(format!("Failed to clear documents: {}", e)))?;

    // Also clear conversation history if desired?
    // User only said documents and vectors, so we stop here.

    Ok(())
}
