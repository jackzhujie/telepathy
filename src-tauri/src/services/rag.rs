use crate::db::{conversations, vectors};
use crate::errors::AppError;
use crate::services::inference::Message as ChatMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchSource {
    pub id: String,
    pub document_name: String,
    pub content: String,
    pub chunk_index: i32,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct UserProfile {
    pub name: String,
    pub gender: String,
    pub age_group: String,
    pub occupation: String,
    pub industry: String,
    pub language: String,
    pub interests: Vec<String>,
    pub long_term_memories: Vec<String>,
}

pub fn search_with_embedding(
    embedding: &[f32],
    conn: &rusqlite::Connection,
    top_k: i32,
    project_id: Option<String>,
    similarity_threshold: f32,
) -> Result<Vec<SearchSource>, AppError> {
    // 1. 检索文档片段 (优先使用 HNSW, 失败则降级到 SQLite 暴力搜索)
    let doc_results = match vectors::search_similar_hnsw(conn, embedding, top_k as usize, project_id.clone(), similarity_threshold) {
        Ok(results) => {
            println!("[RAG] HNSW vector search succeeded, found {} results", results.len());
            results
        }
        Err(e) => {
            eprintln!("[RAG] Warning: HNSW search failed or index not loaded. Falling back to SQLite brute-force search: {}", e);
            vectors::search_similar(conn, embedding, top_k as usize, project_id.clone())
                .map_err(|err| AppError::Internal(format!("Failed to search similar chunks: {}", err)))?
        }
    };


    let mut combined_results: Vec<SearchSource> = doc_results
        .into_iter()
        .map(|r| SearchSource {
            id: r.chunk.id,
            document_name: r.document_name,
            content: r.chunk.content,
            chunk_index: r.chunk.chunk_index,
            score: r.score,
        })
        .collect();

    // 2. 检索个人随记 (Snaps)
    // 注意：随记是全局的，不属于特定项目
    if let Ok(snap_results) =
        crate::db::snaps::search_similar_snaps(conn, embedding, top_k as usize)
    {
        for (snap, score) in snap_results {
            combined_results.push(SearchSource {
                id: snap.id,
                document_name: "【个人随记】".to_string(),
                content: snap.content,
                chunk_index: 0,
                score,
            });
        }
    }

    // 3. 合并排序
    combined_results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 4. 阈值过滤 (强制最低 50% 的相似度过滤，避免毫无关联的内容)
    let actual_threshold = similarity_threshold.max(0.5);
    combined_results.retain(|r| r.score >= actual_threshold);

    // 5. 截断到 top_k
    combined_results.truncate(top_k as usize);

    Ok(combined_results)
}

pub fn format_sources_for_storage(sources: &[SearchSource]) -> String {
    serde_json::to_string(sources).unwrap_or_default()
}

pub fn build_inference_messages(
    query: &str,
    sources: &[SearchSource],
    history: &[conversations::ChatMessage],
    num_ctx: u32,
    num_predict: i32,
) -> Vec<ChatMessage> {
    build_inference_messages_with_user(
        query,
        sources,
        history,
        &UserProfile::default(),
        num_ctx,
        num_predict,
    )
}

pub fn build_inference_messages_with_user(
    query: &str,
    sources: &[SearchSource],
    history: &[conversations::ChatMessage],
    user_profile: &UserProfile,
    num_ctx: u32,
    num_predict: i32,
) -> Vec<ChatMessage> {
    let mut messages = Vec::new();

    // --- 1. 构建 SYSTEM PROMPT ---
    let mut system_prompt = String::new();

    // 1.1 核心身份与禁令 (针对小模型优化的极致指令)
    if sources.is_empty() {
        system_prompt
            .push_str("你是一个专业且友好的智能助手。请始终使用与用户提问相同的语言进行回复。\n");
        system_prompt.push_str("当前知识库未找到相关内容，请基于你的通用知识回答问题。回答末尾标注：[回复基于通用知识]。\n");
    } else {
        system_prompt.push_str("你是一个专业且友好的本地智能知识助手。请遵循以下回答规则：\n");
        system_prompt.push_str("1. 优先根据后面提供的 [参考资料] 提取事实来回答用户的问题。\n");
        system_prompt.push_str("2. 如果参考资料内容不足、模糊或与用户问题完全无关，请直接基于你的通用知识来完整回答用户的问题，并在回答末尾标注 [回复基于通用知识]。\n");
        system_prompt.push_str("3. 当参考资料和通用知识都能解答时，以参考资料的内容为准。\n");
        system_prompt.push_str("4. 不要解释你的回复逻辑，直接给出内容。如果采用了参考资料，请在段落末尾标注 [来源: 文件名, 块 #N]。\n");
    }

    // 1.2 用户基本信息
    let mut user_context = Vec::new();
    if !user_profile.name.is_empty() {
        user_context.push(format!("姓名: {}", user_profile.name));
    }
    if !user_profile.occupation.is_empty() {
        user_context.push(format!("职业: {}", user_profile.occupation));
    }
    if !user_profile.industry.is_empty() {
        user_context.push(format!("行业: {}", user_profile.industry));
    }

    if !user_profile.language.is_empty() {
        let lang_desc = if user_profile.language == "auto" {
            "跟随用户提问语言"
        } else {
            &user_profile.language
        };
        user_context.push(format!("偏好语言: {}", lang_desc));
    }
    if !user_profile.interests.is_empty() {
        user_context.push(format!("兴趣领域: {}", user_profile.interests.join(", ")));
    }

    if !user_context.is_empty() {
        system_prompt.push_str("\n用户基本信息 (请参考这些偏好提供更相关的回答)：\n");
        for context in user_context {
            system_prompt.push_str(&format!("- {}\n", context));
        }
    }

    // 1.3 长期记忆 (基于历史交互的自动总结)
    if !user_profile.long_term_memories.is_empty() {
        system_prompt.push_str("\n用户长期背景 (基于历史交互的自动总结)：\n");
        for (i, memory) in user_profile.long_term_memories.iter().enumerate() {
            system_prompt.push_str(&format!("{}. {}\n", i + 1, memory));
        }
    }

    messages.push(ChatMessage {
        role: "system".into(),
        content: system_prompt,
    });

    // --- 2. 构建对话历史 (近期记忆) ---
    let mut recent_history: Vec<_> = history.iter().rev().take(6).collect();
    recent_history.reverse();

    for msg in recent_history {
        match msg.role.as_str() {
            "user" => messages.push(ChatMessage {
                role: "user".into(),
                content: msg.content.clone(),
            }),
            "assistant" => messages.push(ChatMessage {
                role: "assistant".into(),
                content: msg.content.clone(),
            }),
            _ => {}
        }
    }

    // 2.2 当前用户问题与资料注入
    if !sources.is_empty() {
        // --- 动态上下文预算管理 ---
        // 1 Token ≈ 1.5 字符 (保守估算)
        // 预算 = (总窗口 - 预测长度 - 缓冲区500) * 1.5
        let safe_buffer = 500;
        let prediction_room = num_predict.max(0) as u32;
        let budget_tokens = num_ctx
            .saturating_sub(prediction_room)
            .saturating_sub(safe_buffer);
        let mut char_budget = (budget_tokens as f32 * 1.5) as usize;

        // 减去已知固定内容的长度
        char_budget = char_budget.saturating_sub(messages[0].content.len()); //减去System Prompt
        for msg in &messages[1..] {
            char_budget = char_budget.saturating_sub(msg.content.len());
        }
        char_budget = char_budget.saturating_sub(query.len() + 100); // 预留给提示模板的100字符

        let mut composite_query = String::new();
        composite_query.push_str("【参考资料开始】\n");

        let mut added_count = 0;
        for source in sources {
            let item = format!(
                "> 来源文件: {}, 块 #{}\n{}\n\n",
                source.document_name,
                source.chunk_index + 1,
                source.content
            );

            if item.len() <= char_budget {
                composite_query.push_str(&item);
                char_budget -= item.len();
                added_count += 1;
            } else {
                println!(
                    "[RAG] 资料超出预算，已截断。剩余预算: {}, 缺失资料条数: {}",
                    char_budget,
                    sources.len() - added_count
                );
                break;
            }
        }
        composite_query.push_str("【参考资料结束】\n\n");
        composite_query.push_str(&format!("用户问题：{}\n请结合参考资料进行回答。如果参考资料不足或完全无关，请基于你的通用知识直接回答。", query));

        messages.push(ChatMessage {
            role: "user".into(),
            content: composite_query,
        });
    } else {
        messages.push(ChatMessage {
            role: "user".into(),
            content: query.to_string(),
        });
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::vectors::{self, Chunk};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        vectors::init_vector_tables(&conn).unwrap();
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
    fn test_search_with_embedding_fallback_to_sqlite() {
        let conn = setup_db();
        let chunk = Chunk {
            id: "c1".to_string(),
            document_id: "d1".to_string(),
            chunk_index: 0,
            content: "Test chunk content for fallback".to_string(),
            metadata: None,
            created_at: "2026-01-01".to_string(),
        };
        vectors::insert_chunk(&conn, &chunk).unwrap();
        vectors::insert_embedding(&conn, "c1", &[1.0, 0.0, 0.0]).unwrap();

        // Under unit test environment without initializing HNSW manager, search_similar_hnsw will fail.
        // The search_with_embedding function should print fallback warning and fallback to SQLite vector search.
        let results = search_with_embedding(
            &[1.0, 0.0, 0.0],
            &conn,
            1,
            Some("p1".to_string()),
            0.5,
        );

        assert!(results.is_ok(), "search_with_embedding should succeed via SQLite fallback");
        let results = results.unwrap();
        assert_eq!(results.len(), 1, "Should find 1 document result");
        assert_eq!(results[0].id, "c1");
        assert_eq!(results[0].content, "Test chunk content for fallback");
        assert_eq!(results[0].document_name, "doc-1");
    }
}

