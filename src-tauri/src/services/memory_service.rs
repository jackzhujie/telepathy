use crate::db::memories::{self, Memory};
use crate::errors::AppError;
use crate::services::embedder;
use crate::services::inference::{Message, SamplingConfig};
use tauri::Manager;
use uuid::Uuid;
use std::sync::Arc;

const EXTRACTION_PROMPT_TEMPLATE: &str = r#"请结合【用户本轮提问】与【助理本轮回复】，
客观提炼用户长期有效的固定信息，用于个人长期记忆。

严格规则：
1. 只提取：用户技术栈、项目背景、开发习惯、个人偏好、长期需求、固定约束、身份相关信息。
2. 完全忽略：助理的回答、方案、知识点、临时建议、解释内容。
3. 只保留长期有效内容，过滤一次性临时问题、闲聊、客套、无意义短句。
4. 严禁脑补、严禁虚构、严禁扩写、严禁强行凑字数。
5. 有真实有效信息：如实精简输出，字数不限，不用刻意拉长。
6. 本轮无任何长期有效用户信息：直接只输出单词：none

===== 用户提问 =====
{{user_query}}

===== 助理回复 =====
{{ai_response}}

输出记忆摘要："#;

pub async fn extract_and_save_memory(
    app_handle: &tauri::AppHandle,
    user_query: &str,
    ai_response: &str,
    chat_model: &str,
    embedding_model: &str,
) -> Result<(), AppError> {
    let prompt = EXTRACTION_PROMPT_TEMPLATE
        .replace("{{user_query}}", user_query)
        .replace("{{ai_response}}", ai_response);

    let messages = vec![Message {
        role: "user".to_string(),
        content: prompt,
    }];

    let sampling = SamplingConfig {
        temperature: 0.1, // Low temp for extraction
        top_k: 40,
        top_p: 0.9,
        repeat_penalty: 1.1,
        max_tokens: 512,
    };

    let engine_state = app_handle.state::<crate::services::inference::EngineManagerState>();
    let engine_arc = engine_state.0.clone();

    // Ensure engine is valid
    let mut extracted_text = String::new();
    {
        let engine_guard = engine_arc.lock().await;
        if let Some(engine) = engine_guard.as_ref() {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
            let cancel = Arc::new(tokio_util::sync::CancellationToken::new());

            let stream_res = engine.stream_chat(messages, sampling, tx, cancel).await;
            
            if stream_res.is_ok() {
                while let Some(token) = rx.recv().await {
                    extracted_text.push_str(&token);
                }
            } else {
                return Err(AppError::Internal("Failed to run extraction inference".into()));
            }
        } else {
             return Err(AppError::Internal("Inference engine not initialized".into()));
        }
    }

    let extracted_text = extracted_text.trim();
    if extracted_text.to_lowercase().contains("none") || extracted_text.is_empty() {
        // No memory to extract
        println!("[MemoryService] No long term memory extracted.");
        return Ok(());
    }

    println!("[MemoryService] Extracted memory: {}", extracted_text);

    // Embed the memory
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let embedding_model_path = app_data_dir.join("downloads").join(embedding_model);
    
    if !embedding_model_path.exists() {
        return Err(AppError::Internal("Embedding model not found for memory extraction".into()));
    }

    let emb = embedder::Embedder::new(&embedding_model_path.to_string_lossy());
    let memory_embedding = emb.embed(extracted_text).await?;

    let db_state = app_handle.state::<crate::db::DbState>();
    let conn = crate::db::open_connection(&db_state).await?;

    let memory = Memory {
        id: Uuid::new_v4().to_string(),
        content: extracted_text.to_string(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    memories::insert_memory(&conn, &memory)?;
    memories::insert_memory_embedding(&conn, &memory.id, &memory_embedding)?;

    println!("[MemoryService] Saved long term memory.");
    Ok(())
}
