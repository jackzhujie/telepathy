use crate::db::{documents, settings, vectors};
use crate::errors::AppError;
use crate::services::{chunker, embedder, hnsw_index::with_hnsw_manager, parser};
use chrono::Utc;
use std::path::Path;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

fn create_notification(app: &AppHandle, doc_name: &str, chunk_count: usize) {
    let notification = crate::db::notifications::AppNotification {
        id: Uuid::new_v4().to_string(),
        notification_type: "document_index_complete".to_string(),
        title: "文档索引完成".to_string(),
        content: format!(
            "文档「{}」已成功索引，共生成 {} 个文本块",
            doc_name, chunk_count
        ),
        route_path: Some("/documents".to_string()),
        is_read: false,
        created_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    crate::commands::notifications::emit_notification(app, notification);
}

#[derive(serde::Serialize)]
pub struct IndexResult {
    pub chunk_count: usize,
    pub document_id: String,
}

#[tauri::command]
pub async fn index_document(
    doc_id: String,
    app_handle: AppHandle,
) -> Result<IndexResult, AppError> {
    // 卸载 Chat 大模型以释放 VRAM 空间，为 Embedding 并发及批量向量化腾出空间
    let engine_state = app_handle.state::<crate::services::inference::EngineManagerState>();
    {
        let mut guard = engine_state.0.lock().await;
        if let Some(engine) = guard.as_mut() {
            if let Err(e) = engine.unload().await {
                println!("[Indexing] Warning: failed to unload chat engine: {}", e);
            } else {
                println!("[Indexing] Successfully unloaded chat engine to free VRAM");
            }
        }
    }

    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&conn)?;
    vectors::init_vector_tables(&conn)?;
    settings::init_settings_table(&conn)?;
    settings::seed_defaults(&conn)?;

    let embedding_model = settings::get_setting(&conn, "embedding_model")?
        .unwrap_or_else(|| "bge-large-zh".to_string());

    let doc = documents::get_document(&conn, &doc_id)?
        .ok_or_else(|| AppError::Internal("Document not found".to_string()))?;

    // Clear existing chunks for re-indexing
    vectors::delete_chunks_by_document(&conn, &doc_id)?;

    let text = parser::parse_document_async(Path::new(&doc.library_path), &app_handle).await?;

    // Chunk the text
    let config = chunker::ChunkConfig::default();
    let text_chunks = chunker::chunk_text(&text, &config);

    if text_chunks.is_empty() {
        documents::update_document_status(&conn, &doc_id, "indexed", None)?;
        return Ok(IndexResult {
            chunk_count: 0,
            document_id: doc_id,
        });
    }

    // Initialize embedder
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

    // Insert chunks and generate embeddings in sized batches (size 16) to avoid lock contention
    const BATCH_SIZE: usize = 16;
    for (batch_idx, chunk_slice) in text_chunks.chunks(BATCH_SIZE).enumerate() {
        // Batch embed current slice
        let embeddings = match emb.embed_batch(chunk_slice).await {
            Ok(embs) => Some(embs),
            Err(e) => {
                println!("[Indexing] Warning: batch embedding failed for batch {}: {}", batch_idx, e);
                None
            }
        };

        // Write chunks and corresponding embeddings in this batch
        for (i, chunk_content) in chunk_slice.iter().enumerate() {
            let global_idx = batch_idx * BATCH_SIZE + i;
            let chunk_id = Uuid::new_v4().to_string();
            let chunk = vectors::Chunk {
                id: chunk_id.clone(),
                document_id: doc_id.clone(),
                chunk_index: global_idx as i32,
                content: chunk_content.clone(),
                metadata: None,
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };

            vectors::insert_chunk(&conn, &chunk)?;

            if let Some(ref embs) = embeddings {
                if let Some(embedding) = embs.get(i) {
                    vectors::insert_embedding(&conn, &chunk_id, embedding)?;

                    let add_result = with_hnsw_manager(|manager| {
                        manager.add_chunk(&chunk_id, embedding, embedding.len(), doc.project_id.as_deref())
                    });
                    if let Err(e) = add_result.and_then(|r| r) {
                        println!("[HNSW] Warning: failed to add chunk to index: {}", e);
                    }
                }
            }
        }

        // Yield lock and CPU control to allow concurrent tasks (e.g., chat) to run
        if (batch_idx + 1) * BATCH_SIZE < text_chunks.len() {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }

    documents::update_document_status(&conn, &doc_id, "indexed", None)?;

    create_notification(&app_handle, &doc.name, text_chunks.len());

    Ok(IndexResult {
        chunk_count: text_chunks.len(),
        document_id: doc_id,
    })
}

#[tauri::command]
pub async fn get_document_chunks(
    doc_id: String,
    app_handle: AppHandle,
) -> Result<Vec<vectors::Chunk>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    vectors::init_vector_tables(&conn)?;
    vectors::get_chunks_by_document(&conn, &doc_id)
}

fn get_db_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
    }

    Ok(app_data_dir.join("telepathy.db"))
}
