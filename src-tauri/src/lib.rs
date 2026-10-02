mod commands;
mod db;
mod debug_server;
mod errors;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set llama.cpp log level early, before any threads spawn (safe single-threaded context)
    if std::env::var("LLAMA_LOG_LEVEL").is_err() {
        unsafe { std::env::set_var("LLAMA_LOG_LEVEL", "0") };
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_hwinfo::init())
        // .plugin(tauri_plugin_updater::Builder::new().build()) // updater disabled for open-source build
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            use std::collections::HashMap;
            use std::sync::Mutex;
            use tauri::Manager;

            // --- Database initialization (once at startup) ---
            // Allow overriding the data directory via env var (used for sandboxed dev runs)
            let app_data_dir = if let Ok(custom_dir) = std::env::var("TELEPATHY_DATA_DIR") {
                std::path::PathBuf::from(custom_dir)
            } else {
                app.path()
                    .app_data_dir()
                    .expect("Failed to get app data dir")
            };
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir)
                    .expect("Failed to create app data dir");
            }
            let db_path = app_data_dir.join("telepathy.db");

            // Create database connection
            let conn = rusqlite::Connection::open(&db_path)
                .expect("Failed to open database at startup");
            
            // Initialize all tables once
            crate::db::init_all_tables(&conn)
                .expect("Failed to initialize database tables");
            
            // Initialize SettingsCache
            let settings_cache = std::sync::Arc::new(crate::db::settings_cache::SettingsCache::new());
            settings_cache.load_from_db(&conn);
            app.manage(settings_cache);

            // Initialize HNSW manager
            let hnsw_dir = app_data_dir.join("hnsw");
            if let Err(e) = std::fs::create_dir_all(&hnsw_dir) {
                eprintln!("[HNSW] Failed to create hnsw directory: {}", e);
            }
            crate::services::hnsw_index::init_hnsw_manager(hnsw_dir.clone());
            crate::services::hnsw_index::load_existing_indexes_on_startup();
            println!("[HNSW] Manager initialized");

            // Register shared DbState with connection
            app.manage(crate::db::DbState {
                db_path,
                connection: std::sync::Arc::new(tokio::sync::Mutex::new(conn)),
            });

            // Register download manager state
            app.manage(
                crate::services::model_hub::downloader::DownloadManagerState(Mutex::new(
                    HashMap::new(),
                )),
            );

            // Register Inference engine state
            app.manage(crate::services::inference::EngineManagerState(
                std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            ));
            app.manage(crate::services::inference::VisionEngineManagerState(
                std::sync::Arc::new(tokio::sync::Mutex::new(crate::services::inference::vision_adapter::VisionAdapter::new())),
            ));

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }

                // Start the debug HTTP server (only in debug mode)
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    crate::debug_server::start_debug_server(app_handle).await;
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet::greet,
            commands::debug::tele_debug_report_route,
            commands::document::import_document,
            commands::document::get_documents,
            commands::document::get_documents_paginated,
            commands::document::delete_document_cmd,
            commands::document::parse_document,
            commands::document::is_sidecar_installed_cmd,
            commands::document::scan_folder,
            commands::indexing::index_document,
            commands::indexing::get_document_chunks,
            commands::rag::rag_query,
            commands::rag::get_conversations,
            commands::rag::get_conversations_paginated,
            commands::rag::get_messages,
            commands::rag::get_messages_paginated,
            commands::rag::delete_conversation_cmd,
            commands::rag::stop_generation,
            commands::rag::regenerate_message,
            commands::rag::delete_messages_after,
            commands::settings::get_settings,
            commands::settings::update_setting_cmd,
            commands::settings::is_gpu_supported,
            commands::settings::debug_gpu_support,
            commands::settings::get_model_max_layers,
            commands::settings::get_current_model_max_layers,
            commands::models::cancel_pull_model,
            commands::notifications::get_notifications,
            commands::notifications::get_notifications_paginated,
            commands::notifications::mark_notification_read,
            commands::notifications::get_unread_count,
            commands::knowledge::get_indexed_documents,
            commands::knowledge::get_indexed_documents_paginated,
            commands::knowledge::get_document_chunks_detail,
            commands::knowledge::get_document_chunks_detail_paginated,
            commands::knowledge::search_chunks,
            commands::projects::create_project,
            commands::projects::list_projects,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::window::window_minimize,
            commands::window::window_maximize,
            commands::window::window_close,
            commands::reindex::reindex_all_documents,
            commands::reindex::clear_all_data,
            commands::profile::get_user_profile,
            commands::profile::update_user_profile,
            commands::profile::get_user_interests,
            commands::profile::add_user_interest,
            commands::profile::remove_user_interest,
            commands::models::list_registry_models,
            commands::models::get_model_hub,
            commands::models::search_hub_models,
            commands::models::get_model_variants,
            commands::models::list_installed_models,
            commands::models::get_model_recommendations,
            commands::models::install_model,
            commands::models::delete_model,
            commands::snaps::get_snaps_paginated,
            commands::snaps::create_snap,
            commands::snaps::update_snap,
            commands::snaps::delete_snap,
            commands::memory::get_memories,
            commands::memory::delete_memory,
            commands::memory::manual_memory_extraction,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
