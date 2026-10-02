use crate::db::{self, conversations, memories, profile, settings};
use crate::errors::AppError;
use crate::services::{embedder, rag};
use serde::Serialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

static CANCEL_TOKEN: std::sync::LazyLock<Arc<Mutex<Arc<CancellationToken>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(Arc::new(CancellationToken::new()))));

fn reset_cancel_token() {
    let mut guard = CANCEL_TOKEN.lock().unwrap();
    (**guard).cancel(); // Cancel the existing one
    *guard = Arc::new(CancellationToken::new());
}

fn get_cancel_token() -> Arc<CancellationToken> {
    CANCEL_TOKEN.lock().unwrap().clone()
}

#[derive(Serialize, Clone)]
pub struct RagSourcesPayload {
    pub conversation_id: String,
    pub message_id: String,
    pub sources: Vec<rag::SearchSource>,
}

#[derive(Serialize, Clone)]
pub struct ChatStartedPayload {
    pub conversation_id: String,
    pub assistant_message_id: String,
    pub user_message_id: String,
}

#[tauri::command]
pub async fn rag_query(
    query: String,
    conversation_id: Option<String>,
    project_id: Option<String>,
    skip_user_insert: Option<bool>,
    images: Option<Vec<String>>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let total_start = std::time::Instant::now();
    let db_state = app_handle.state::<db::DbState>();
    let db_path = db_state.db_path.clone();
    let conn = db::open_connection(&db_state).await?;
    let settings_cache =
        app_handle.state::<std::sync::Arc<crate::db::settings_cache::SettingsCache>>();

    let image_bytes_list = if let Some(ref imgs) = images {
        if !imgs.is_empty() {
            crate::services::parser::vision::decode_data_urls(imgs)?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let vision_model = settings::get_setting_cached(&conn, &settings_cache, "vision_model")
        .ok()
        .flatten()
        .filter(|v| !v.is_empty());

    let vision_mmproj = settings::get_setting_cached(&conn, &settings_cache, "vision_mmproj")
        .ok()
        .flatten()
        .filter(|v| !v.is_empty());

    if !image_bytes_list.is_empty() {
        if vision_model.is_none() || vision_mmproj.is_none() {
            return Err(AppError::Internal(
                "当前未配置完整的 Vision 模型和 mmproj 投影器。请先在设置中配置后再上传图片。"
                    .to_string(),
            ));
        }
    }

    let final_query = query.clone();

    // 1. 读取配置（使用缓存）
    let embedding_model = settings::get_setting_cached(&conn, &settings_cache, "embedding_model")?
        .unwrap_or_else(|| "bge-m3:latest".to_string());
    let chat_model = settings::get_setting_cached(&conn, &settings_cache, "chat_model")?
        .unwrap_or_else(|| "qwen3:1.7b".to_string());
    let top_k = settings::get_top_k(&conn, Some(&settings_cache))?;
    let similarity_threshold = settings::get_similarity_threshold(&conn, Some(&settings_cache))?;

    // 2. 获取 App Data 目录并前置检验模型文件存在性
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;

    // 2.1 校验 Embedding 模型
    let embedding_model_path = app_data_dir.join("downloads").join(&embedding_model);
    if !embedding_model_path.exists() {
        return Err(AppError::Internal(format!(
            "未找到 Embedding 模型文件，请先在模型管理中下载。文件名: {}",
            embedding_model
        )));
    }

    // 2.2 校验 Chat 模型
    let chat_model_path = app_data_dir.join("downloads").join(&chat_model);
    if !chat_model_path.exists() {
        return Err(AppError::Internal(format!(
            "未找到聊天模型文件，请先在模型管理中下载。文件名: {}",
            chat_model
        )));
    }

    // 2.3 若上传了图片，校验 Vision 模型与投影器
    if !image_bytes_list.is_empty() {
        if let (Some(v_model), Some(v_mmproj)) = (&vision_model, &vision_mmproj) {
            let v_model_path = app_data_dir.join("downloads").join(v_model);
            let v_mmproj_path = app_data_dir.join("downloads").join(v_mmproj);
            if !v_model_path.exists() {
                return Err(AppError::Internal(format!(
                    "未找到 Vision 多模态模型文件，请先在设置中下载或配置。文件名: {}",
                    v_model
                )));
            }
            if !v_mmproj_path.exists() {
                return Err(AppError::Internal(format!(
                    "未找到 Vision mmproj 投影器文件，请先在设置中下载或配置。文件名: {}",
                    v_mmproj
                )));
            }
        }
    }

    let emb = embedder::Embedder::new(&embedding_model_path.to_string_lossy());
    let query_embedding = emb.embed(&query).await?;

    // 3. 知识库检索
    let sources = rag::search_with_embedding(
        &query_embedding,
        &conn,
        top_k as i32,
        project_id,
        similarity_threshold,
    )?;

    // 3.5 长期记忆检索
    let ltm_results = memories::search_similar_memories(&conn, &query_embedding, 3)?;
    let long_term_memories: Vec<String> = ltm_results.into_iter().map(|(m, _)| m.content).collect();

    // 4. 构建消息与上下文
    let num_thread = settings::get_setting_cached(&conn, &settings_cache, "num_thread")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(6);
    let num_ctx = settings::get_setting_cached(&conn, &settings_cache, "num_ctx")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(4096);
    let temperature = settings::get_setting_cached(&conn, &settings_cache, "temperature")?
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.7);
    let num_gpu = settings::get_setting_cached(&conn, &settings_cache, "num_gpu")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);
    let repeat_penalty = settings::get_setting_cached(&conn, &settings_cache, "repeat_penalty")?
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1.1);
    let num_predict = settings::get_setting_cached(&conn, &settings_cache, "num_predict")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1024);
    let sampling_top_k = settings::get_setting_cached(&conn, &settings_cache, "sampling_top_k")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(40);

    let user_profile = profile::get_user_profile(&conn).unwrap_or_default();
    let user_interests = profile::get_interests(&conn).unwrap_or_default();

    let conv_id = match conversation_id {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            let title = if final_query.chars().count() > 20 {
                Some(final_query.chars().take(20).collect::<String>())
            } else {
                Some(final_query.clone())
            };
            conversations::create_conversation(&conn, &id, title.as_deref())?;
            let _ = app_handle.emit("conversation-created", json!({ "id": id, "title": title }));
            id
        }
    };

    let history = conversations::get_messages(&conn, &conv_id)?;

    let _inference_messages = rag::build_inference_messages_with_user(
        &final_query,
        &sources,
        &history,
        &rag::UserProfile {
            name: user_profile.name,
            gender: user_profile.gender,
            age_group: user_profile.age_group,
            occupation: user_profile.occupation,
            industry: user_profile.industry,
            language: user_profile.language,
            interests: user_interests,
            long_term_memories,
        },
        num_ctx as u32,
        num_predict as i32,
    );
    // Save user message (skip if regenerating)
    let user_msg_id = if skip_user_insert.unwrap_or(false) {
        let last_user_msg = history.iter().rev().find(|m| m.role == "user");
        last_user_msg
            .map(|m| m.id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string())
    } else {
        let images_json = images.as_ref().map(|imgs| serde_json::to_string(imgs).unwrap_or_default());
        let user_msg = conversations::ChatMessage {
            id: Uuid::new_v4().to_string(),
            conversation_id: conv_id.clone(),
            role: "user".to_string(),
            content: final_query.clone(),
            sources: None,
            images: images_json,
            created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        };
        conversations::insert_message(&conn, &user_msg)?;
        user_msg.id
    };

    let sources_json = if sources.is_empty() {
        None
    } else {
        Some(rag::format_sources_for_storage(&sources))
    };

    let assistant_msg_id = Uuid::new_v4().to_string();

    // --- 占位符先行策略 ---
    // 立即向数据库插入一条空的助手机器人消息，确保持久化锚点已建立
    let assistant_msg_placeholder = conversations::ChatMessage {
        id: assistant_msg_id.clone(),
        conversation_id: conv_id.clone(),
        role: "assistant".to_string(),
        content: String::new(),
        sources: sources_json.clone(),
        images: None,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };
    conversations::insert_message(&conn, &assistant_msg_placeholder)?;

    let _ = app_handle.emit(
        "chat-started",
        ChatStartedPayload {
            conversation_id: conv_id.clone(),
            assistant_message_id: assistant_msg_id.clone(),
            user_message_id: user_msg_id.clone(),
        },
    );

    let _ = app_handle.emit(
        "rag-sources",
        RagSourcesPayload {
            conversation_id: conv_id.clone(),
            message_id: assistant_msg_id.clone(),
            sources: sources.clone(),
        },
    );

    reset_cancel_token();
    let cancel_token = get_cancel_token();

    let assistant_msg_id_clone = assistant_msg_id.clone();
    let chat_model_filename = chat_model.clone();

    let sampling = crate::services::inference::SamplingConfig {
        temperature,
        top_k: sampling_top_k,
        top_p: 0.95,
        repeat_penalty,
        max_tokens: num_predict,
    };

    let model_path = app_data_dir.join("downloads").join(&chat_model_filename);

    let engine_state = app_handle.state::<crate::services::inference::EngineManagerState>();
    let engine_arc = engine_state.0.clone();

    let app_handle_clone = app_handle.clone();
    let conv_id_for_task = conv_id.clone();
    let query_for_task = final_query.clone();
    let embedding_model_for_task = embedding_model.clone();

    let vision_model_clone = vision_model.clone();
    let vision_mmproj_clone = vision_mmproj.clone();
    let image_bytes_list_for_task = image_bytes_list.clone();

    tauri::async_runtime::spawn(async move {
        // 3. Prepare channel for streaming
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

        // 4. Spawn receiver task to update UI/DB
        let assistant_msg_id_receiver = assistant_msg_id_clone.clone();
        let app_handle_receiver = app_handle_clone.clone();
        let db_path_receiver = db_path.clone();
        let conv_id_receiver = conv_id_for_task.clone();

        let receiver_handle = tauri::async_runtime::spawn(async move {
            let mut full_content = String::new();
            let mut last_db_update = std::time::Instant::now();

            while let Some(token) = rx.recv().await {
                full_content.push_str(&token);

                // Emit token to frontend
                let _ = app_handle_receiver.emit(
                    "chat-token",
                    json!({
                        "conversation_id": conv_id_receiver.clone(),
                        "message_id": assistant_msg_id_receiver.clone(),
                        "token": token,
                    }),
                );

                // Periodically update DB (every 1s)
                if last_db_update.elapsed().as_secs() >= 1 {
                    if let Ok(conn) = rusqlite::Connection::open(&db_path_receiver) {
                        let _ = conversations::update_message_content(
                            &conn,
                            &assistant_msg_id_receiver,
                            &full_content,
                        );
                        last_db_update = std::time::Instant::now();
                    }
                }
            }

            // Final update
            if let Ok(conn) = rusqlite::Connection::open(&db_path_receiver) {
                let _ = conversations::update_message_content(
                    &conn,
                    &assistant_msg_id_receiver,
                    &full_content,
                );
            }

            let _ = app_handle_receiver.emit(
                "chat-done",
                json!({
                    "conversation_id": conv_id_receiver.clone(),
                    "message_id": assistant_msg_id_receiver.clone(),
                }),
            );

            full_content
        });

        // 5. Run inference and await completion
        let stream_res = if !image_bytes_list_for_task.is_empty() {
            let vision_state =
                app_handle_clone.state::<crate::services::inference::VisionEngineManagerState>();
            let vision_arc = vision_state.0.clone();
            let mut vision_guard = vision_arc.lock().await;

            let v_model_name = vision_model_clone.unwrap();
            let mmproj_name = vision_mmproj_clone.unwrap();
            let app_data = app_handle_clone.path().app_data_dir().unwrap();
            let v_model_path = app_data.join("downloads").join(&v_model_name);
            let mmproj_path = app_data.join("downloads").join(&mmproj_name);

            let load_res = vision_guard
                .load_model(
                    &v_model_path,
                    &mmproj_path,
                    num_ctx as u32,
                    num_gpu,
                    num_thread,
                )
                .await;

            if let Err(e) = load_res {
                Err(e)
            } else {
                vision_guard
                    .stream_vision_chat(
                        &query_for_task,
                        image_bytes_list_for_task,
                        num_predict,
                        tx,
                        cancel_token,
                    )
                    .await
            }
        } else {
            let mut engine_guard = engine_arc.lock().await;

            // 1. Initialize engine if None
            if engine_guard.is_none() {
                *engine_guard = Some(Box::new(
                    crate::services::inference::llama_adapter::LlamaCppAdapter::new(),
                ));
            }

            let engine = engine_guard.as_mut().unwrap();

            // 2. Load model if needed
            let load_res = engine
                .load_model(
                    &model_path,
                    crate::services::inference::ModelLoadOptions {
                        n_gpu_layers: num_gpu,
                        context_size: num_ctx as u32,
                        n_threads: num_thread,
                    },
                )
                .await;

            if let Err(e) = load_res {
                Err(e)
            } else {
                let inference_messages: Vec<crate::services::inference::Message> =
                    _inference_messages
                        .into_iter()
                        .map(|m| crate::services::inference::Message {
                            role: m.role,
                            content: m.content,
                        })
                        .collect();

                engine
                    .stream_chat(inference_messages, sampling, tx, cancel_token)
                    .await
            }
        };

        // 6. Cleanup
        if let Err(e) = stream_res {
            println!("[ERROR] Inference error: {}", e);
            let _ = app_handle_clone.emit(
                "chat-error",
                json!({
                    "conversation_id": conv_id_for_task.clone(),
                    "message_id": assistant_msg_id_clone.clone(),
                    "error": format!("Inference error: {}", e),
                }),
            );
        }

        let full_content = receiver_handle.await.unwrap_or_default();

        // --- 记忆提炼（后台） ---
        let query_clone = query_for_task.clone();
        let app_handle_clone = app_handle_clone.clone();
        let chat_model_filename_clone = chat_model_filename.clone();
        let embedding_model_clone = embedding_model_for_task.clone();

        tauri::async_runtime::spawn(async move {
            let _ = crate::services::memory_service::extract_and_save_memory(
                &app_handle_clone,
                &query_clone,
                &full_content,
                &chat_model_filename_clone,
                &embedding_model_clone,
            )
            .await;
        });
    });

    println!("[PERF] Total Command Time: {:?}", total_start.elapsed());
    Ok(())
}

#[tauri::command]
pub async fn get_conversations(
    app_handle: AppHandle,
) -> Result<Vec<conversations::Conversation>, AppError> {
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;
    conversations::get_conversations(&conn)
}

#[tauri::command]
pub async fn get_conversations_paginated(
    page: i32,
    page_size: i32,
    app_handle: AppHandle,
) -> Result<crate::models::PaginatedResult<conversations::Conversation>, AppError> {
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;

    let (items, total) = conversations::get_conversations_paginated(&conn, page, page_size)?;
    Ok(crate::models::PaginatedResult::new(
        items, total, page, page_size,
    ))
}

#[tauri::command]
pub async fn get_messages(
    conversation_id: String,
    app_handle: AppHandle,
) -> Result<Vec<conversations::ChatMessage>, AppError> {
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;
    conversations::get_messages(&conn, &conversation_id)
}

#[tauri::command]
pub async fn get_messages_paginated(
    conversation_id: String,
    page: i32,
    page_size: i32,
    app_handle: AppHandle,
) -> Result<crate::models::PaginatedResult<conversations::ChatMessage>, AppError> {
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;

    let (items, total) =
        conversations::get_messages_paginated(&conn, &conversation_id, page, page_size)?;
    Ok(crate::models::PaginatedResult::new(
        items, total, page, page_size,
    ))
}

#[tauri::command]
pub async fn delete_conversation_cmd(
    conversation_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;
    conversations::delete_conversation(&conn, &conversation_id)
}

#[tauri::command]
pub async fn stop_generation() -> Result<(), AppError> {
    get_cancel_token().cancel();
    Ok(())
}

#[tauri::command]
pub async fn regenerate_message(
    conversation_id: String,
    message_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let app_handle_clone = app_handle.clone();
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;

    let msg = conversations::get_message_by_id(&conn, &message_id)
        .map_err(|e| AppError::Internal(format!("Message not found: {}", e)))?;

    // Delete everything from this assistant message onwards
    conversations::delete_messages_from(&conn, &conversation_id, &msg.created_at)
        .map_err(|e| AppError::Internal(format!("Failed to delete messages: {}", e)))?;

    let history = conversations::get_messages(&conn, &conversation_id)?;
    let last_user_msg = history.iter().rev().find(|m| m.role == "user");
    let query = last_user_msg.map(|m| m.content.clone()).unwrap_or_default();

    if query.is_empty() {
        return Err(AppError::Internal(
            "No user message found to regenerate from".to_string(),
        ));
    }

    rag_query(
        query,
        Some(conversation_id),
        None,
        Some(true),
        None,
        app_handle_clone,
    )
    .await
}

#[tauri::command]
pub async fn delete_messages_after(
    conversation_id: String,
    message_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let db_state = app_handle.state::<db::DbState>();
    let conn = db::open_connection(&db_state).await?;

    let msg = conversations::get_message_by_id(&conn, &message_id)
        .map_err(|e| AppError::Internal(format!("Message not found: {}", e)))?;

    // Delete everything from this message onwards (including itself)
    conversations::delete_messages_from(&conn, &conversation_id, &msg.created_at)
        .map_err(|e| AppError::Internal(format!("Failed to delete messages: {}", e)))?;

    Ok(())
}
