use crate::errors::AppError;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use std::io::Cursor;
use image::imageops::FilterType;
use std::path::Path;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;
use crate::db::settings_cache::SettingsCache;
use crate::services::inference::VisionEngineManagerState;

/// Decode a data URL (e.g. "data:image/png;base64,xxxxx") into raw bytes, and compress large images.
pub fn decode_data_url(data_url: &str) -> Result<Vec<u8>, AppError> {
    let base64_part = if let Some(pos) = data_url.find(',') {
        &data_url[pos + 1..]
    } else {
        data_url
    };

    let raw_bytes = BASE64
        .decode(base64_part)
        .map_err(|e| AppError::Internal(format!("Invalid base64 image data: {}", e)))?;
        
    // Always compress to a maximum dimension of 448 (typical safe bounds for mmproj models on weak CPUs)
    compress_image(&raw_bytes, 448)
}

fn compress_image(bytes: &[u8], max_dim: u32) -> Result<Vec<u8>, AppError> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| AppError::Internal(format!("Failed to load image for compression: {}", e)))?;
    
    let (width, height) = (img.width(), img.height());
    if width <= max_dim && height <= max_dim {
        return Ok(bytes.to_vec()); // Already small enough
    }
    
    // Resize maintaining aspect ratio
    let resized = img.resize(max_dim, max_dim, FilterType::CatmullRom);
    
    let mut out_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut out_bytes);
    
    // Encode as JPEG to pass smaller raw payload to llama.cpp
    resized.write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| AppError::Internal(format!("Failed to encode compressed image: {}", e)))?;
        
    Ok(out_bytes)
}

/// Decode multiple data URLs into raw byte vectors.
pub fn decode_data_urls(data_urls: &[String]) -> Result<Vec<Vec<u8>>, AppError> {
    data_urls.iter().map(|url| decode_data_url(url)).collect()
}



/// 使用本地 Vision 模型解析单张图片并提取文本
pub async fn parse_image_with_vision(path: &Path, app_handle: &AppHandle) -> Result<String, AppError> {
    let image_data = std::fs::read(path)
        .map_err(|e| AppError::Internal(format!("读取图片文件失败: {}", e)))?;
    
    // 压缩图片以保证低配显存能够安全执行
    let compressed_bytes = compress_image(&image_data, 448)?;

    // 获取 VisionEngine 管理状态
    let vision_state = app_handle.state::<VisionEngineManagerState>();
    let vision_arc = vision_state.0.clone();
    let mut vision_guard = vision_arc.lock().await;

    // 从设置中获取 Vision 加载参数
    let db_path = app_handle
        .path()
        .app_data_dir()
        .map(|p| p.join("telepathy.db"))
        .map_err(|e| AppError::Internal(format!("Failed to get app data path: {}", e)))?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open DB: {}", e)))?;
    
    let settings_cache = app_handle.state::<std::sync::Arc<SettingsCache>>();
    
    let vision_model = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_model")?
        .ok_or_else(|| AppError::Internal("未配置视觉模型".to_string()))?;
    let vision_mmproj = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_mmproj")?
        .ok_or_else(|| AppError::Internal("未配置 mmproj 视觉投影器".to_string()))?;

    let num_thread = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_thread")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(6);
    let num_ctx = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_ctx")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(4096);
    let num_gpu = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_gpu")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);

    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let model_path = app_data_dir.join("downloads").join(&vision_model);
    let mmproj_path = app_data_dir.join("downloads").join(&vision_mmproj);

    if !model_path.exists() || !mmproj_path.exists() {
        return Err(AppError::Internal("本地视觉模型文件或投影器文件不存在".to_string()));
    }

    // 加载模型
    vision_guard.load_model(
        &model_path,
        &mmproj_path,
        num_ctx as u32,
        num_gpu,
        num_thread
    ).await?;

    // 进行 OCR 推理
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    let cancel_token = std::sync::Arc::new(CancellationToken::new());

    let query = "请将此图片中的所有文字内容提取出来。如果是表格，请完整保留 Markdown 表格格式；如果含有图表，请在适当位置用文字进行描述。请直接输出提取出的内容，不要有任何前导客套话或多余的解释。";

    let cancel_clone = cancel_token.clone();
    
    // Spawn receiver task to collect text tokens concurrently
    let receiver_task = tokio::spawn(async move {
        let mut extracted_text = String::new();
        while let Some(token) = rx.recv().await {
            extracted_text.push_str(&token);
        }
        extracted_text
    });

    // Run inference in current task
    let stream_res = vision_guard.stream_vision_chat(
        query,
        vec![compressed_bytes],
        1024,
        tx,
        cancel_clone,
    ).await;

    stream_res?;

    let extracted_text = receiver_task.await
        .map_err(|e| AppError::Internal(format!("Receiver task panicked: {}", e)))?;

    Ok(extracted_text)
}

/// 视觉解析 PDF（借用 Sidecar 渲染 PDF 页面，再调用批量 Vision OCR）
pub async fn parse_pdf_with_vision(path: &Path, app_handle: &AppHandle) -> Result<String, AppError> {
    let temp_id = uuid::Uuid::new_v4().to_string();
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let temp_dir = app_data_dir.join("temp").join(format!("pdf_render_{}", temp_id));
    
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| AppError::Internal(format!("Failed to create temp PDF render directory: {}", e)))?;

    // 1. 调用 Sidecar 渲染 PDF 页面图片
    let render_result = crate::services::parser::sidecar::render_pdf_pages_via_sidecar(
        app_handle,
        path,
        &temp_dir
    ).await;

    if let Err(e) = render_result {
        eprintln!("[Vision Parser] Warning: Sidecar PDF rendering failed: {}. Falling back to plain text extraction.", e);
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        let path_clone = path.to_path_buf();
        return tokio::task::spawn_blocking(move || crate::services::parser::core::parse_pdf(&path_clone))
            .await
            .map_err(|join_err| AppError::Internal(format!("Fallback task execution failed: {}", join_err)))?;
    }

    // 2. 读取并排序所有导出的页面图片
    let mut page_images = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                let is_img = p.extension()
                    .map(|ext| ext.to_string_lossy().to_lowercase() == "jpg" || ext.to_string_lossy().to_lowercase() == "png")
                    .unwrap_or(false);
                if is_img {
                    page_images.push(p);
                }
            }
        }
    }

    page_images.sort_by(|a, b| {
        let a_num = a.file_stem()
            .and_then(|s| s.to_string_lossy().replace("page_", "").parse::<i32>().ok())
            .unwrap_or(0);
        let b_num = b.file_stem()
            .and_then(|s| s.to_string_lossy().replace("page_", "").parse::<i32>().ok())
            .unwrap_or(0);
        a_num.cmp(&b_num)
    });

    // 3. 获取/缓存加载参数
    let db_path = app_data_dir.join("telepathy.db");
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open DB: {}", e)))?;
    let settings_cache = app_handle.state::<std::sync::Arc<SettingsCache>>();
    
    let vision_model = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_model")?
        .ok_or_else(|| AppError::Internal("未配置视觉模型".to_string()))?;
    let vision_mmproj = crate::db::settings::get_setting_cached(&conn, &settings_cache, "vision_mmproj")?
        .ok_or_else(|| AppError::Internal("未配置 mmproj 视觉投影器".to_string()))?;

    let num_thread = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_thread")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(6);
    let num_ctx = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_ctx")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(4096);
    let num_gpu = crate::db::settings::get_setting_cached(&conn, &settings_cache, "num_gpu")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);

    let model_path = app_data_dir.join("downloads").join(&vision_model);
    let mmproj_path = app_data_dir.join("downloads").join(&vision_mmproj);

    if !model_path.exists() || !mmproj_path.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(AppError::Internal("本地视觉模型文件或投影器文件不存在".to_string()));
    }

    // 4. 压缩读入的所有页面图片
    let mut pages_bytes = Vec::new();
    for img_path in &page_images {
        let image_data = match std::fs::read(img_path) {
            Ok(d) => d,
            Err(e) => {
                println!("[Vision Parser] Warning: failed to read page: {}", e);
                continue;
            }
        };
        if let Ok(compressed) = compress_image(&image_data, 448) {
            pages_bytes.push(compressed);
        }
    }

    // 5. 获取 VisionEngine 状态并批量加载
    let vision_state = app_handle.state::<VisionEngineManagerState>();
    let vision_arc = vision_state.0.clone();
    let mut vision_guard = vision_arc.lock().await;
    vision_guard.load_model(
        &model_path,
        &mmproj_path,
        num_ctx as u32,
        num_gpu,
        num_thread
    ).await?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(usize, String)>(100);
    let cancel_token = std::sync::Arc::new(CancellationToken::new());

    // 6. 开启接收者任务收集结果
    let receiver_task = tokio::spawn(async move {
        let mut pages_map = std::collections::BTreeMap::new();
        while let Some((page_num, page_text)) = rx.recv().await {
            pages_map.insert(page_num, page_text);
        }
        pages_map
    });

    // 7. 开启批量多页视觉推理
    let stream_res = vision_guard.stream_vision_multi_page(
        pages_bytes,
        tx,
        cancel_token,
    ).await;

    stream_res?;

    let pages_map = receiver_task.await
        .map_err(|e| AppError::Internal(format!("Receiver task failed: {}", e)))?;

    // 8. 排序拼接 Markdown
    let mut full_markdown = String::new();
    for (page_num, page_text) in pages_map {
        if !full_markdown.is_empty() {
            full_markdown.push_str("\n\n");
        }
        full_markdown.push_str(&format!("--- [第 {} 页] ---\n\n", page_num));
        full_markdown.push_str(&page_text);
    }

    // 9. 清理临时文件夹
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(full_markdown)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_core_pdf_fallback_compiles() {
        let dummy_path = std::path::Path::new("non_existent_file.pdf");
        let result = crate::services::parser::core::parse_pdf(dummy_path);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result);
        assert!(err_msg.contains("Failed to read file") || err_msg.contains("No such file") || err_msg.contains("entity not found"));
    }

    #[test]
    fn test_compress_image_ratio_and_quality() {
        let mut img_buffer = image::ImageBuffer::new(800, 600);
        for pixel in img_buffer.pixels_mut() {
            *pixel = image::Rgb([255, 0, 0]);
        }
        let dynamic_img = image::DynamicImage::ImageRgb8(img_buffer);
        let mut raw_bytes = Vec::new();
        dynamic_img.write_to(&mut std::io::Cursor::new(&mut raw_bytes), image::ImageFormat::Png).unwrap();

        let compressed = super::compress_image(&raw_bytes, 448).unwrap();
        let decompressed = image::load_from_memory(&compressed).unwrap();
        assert!(decompressed.width() <= 448);
        assert!(decompressed.height() <= 448);
    }
}




