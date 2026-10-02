use futures_util::StreamExt;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use reqwest::StatusCode;
use serde_json::json;
use std::path::PathBuf;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::errors::AppError;

pub struct DownloadManagerState(
    pub std::sync::Mutex<std::collections::HashMap<String, tokio_util::sync::CancellationToken>>,
);

pub struct ModelDownloader {
    client: reqwest::Client,
}

impl ModelDownloader {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Telepathy/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// Download a file to the specified path with resume capability and emit progress to frontend
    pub async fn download(
        &self,
        url: &str,
        dest_path: &PathBuf,
        model_id: &str,
        expected_size: u64,
        app_handle: &AppHandle,
    ) -> Result<(), AppError> {
        let mut downloaded: u64 = 0;

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to create dirs: {}", e)))?;
        }

        // Check if file exists to get existing bytes
        if dest_path.exists() {
            downloaded = tokio::fs::metadata(&dest_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0);
        }

        // Fetch Total Size first without downloading body
        let head_res = self.client.head(url).send().await;

        let mut total_size = match head_res {
            Ok(res) => res
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse::<u64>().ok())
                .unwrap_or(expected_size),
            Err(_) => expected_size,
        };

        // If we still have 0 but expected_size is > 0, use expected_size
        if total_size == 0 && expected_size > 0 {
            total_size = expected_size;
        }

        // Pre-flight Disk Check
        if total_size > 0 {
            use sysinfo::Disks;
            let disks = Disks::new_with_refreshed_list();
            // Find the disk containing the app_data_dir downloads path
            let dest_dir_str = dest_path.to_string_lossy().to_string();
            let mut matching_disk = None;
            let mut max_match_len = 0;

            for disk in disks.list() {
                let mnt = disk.mount_point().to_string_lossy().to_string();
                if dest_dir_str.starts_with(&mnt) && mnt.len() > max_match_len {
                    max_match_len = mnt.len();
                    matching_disk = Some(disk);
                }
            }

            if let Some(disk) = matching_disk {
                // Buffer space (default 1GB + existing downloaded bytes if appending)
                let required_space = total_size.saturating_sub(downloaded) + 1024 * 1024 * 1024;
                if disk.available_space() < required_space {
                    return Err(AppError::Internal(format!(
                        "磁盘空间不足！需要至少 {} GB 可用空间，当前仅剩余 {} GB",
                        required_space / 1_073_741_824,
                        disk.available_space() / 1_073_741_824
                    )));
                }
            }
        }

        if total_size > 0 && downloaded == total_size {
            // Already downloaded completely
            return Ok(());
        }

        let mut req = self.client.get(url);
        if downloaded > 0 {
            // Add Range Header to start from where we left off
            req = req.header(RANGE, format!("bytes={}-", downloaded));
        }

        let res = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to start download: {}", e)))?;

        if downloaded > 0 && res.status() != StatusCode::PARTIAL_CONTENT {
            downloaded = 0;
            let _ = tokio::fs::remove_file(dest_path).await;
        }

        // Open file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dest_path)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stream = res.bytes_stream();

        let mut last_emit = Instant::now();
        let emit_interval = std::time::Duration::from_millis(200);

        // Setup Cancellation Token
        use tauri::Manager;
        use tokio_util::sync::CancellationToken;
        let token = CancellationToken::new();
        let child_token = token.child_token();

        if let Some(state) = app_handle.try_state::<DownloadManagerState>() {
            if let Ok(mut map) = state.0.lock() {
                map.insert(model_id.to_string(), token);
            }
        }

        loop {
            tokio::select! {
                _ = child_token.cancelled() => {
                    // Download Cancelled
                    if let Some(state) = app_handle.try_state::<DownloadManagerState>() {
                        if let Ok(mut map) = state.0.lock() {
                            map.remove(model_id);
                        }
                    }

                    // Cleanup partial download
                    drop(file);
                    let _ = tokio::fs::remove_file(dest_path).await;

                    // Inform Frontend
                    let _ = app_handle.emit(
                        "model-pull-progress",
                        json!({
                            "stage": "cancelled",
                            "model": model_id,
                            "percentage": 0.0,
                            "completed": 0,
                            "total": 0,
                            "message": "Download manually cancelled."
                        })
                    );
                    return Err(AppError::Internal("User Cancelled Download".to_string()));
                }
                chunk_opt = stream.next() => {
                    match chunk_opt {
                        Some(Ok(chunk)) => {
                            if let Err(e) = file.write_all(&chunk).await {
                                return Err(AppError::Internal(format!("File write error: {}", e)));
                            }
                            downloaded += chunk.len() as u64;

                            if last_emit.elapsed() >= emit_interval {
                                let mut percentage = 0.0;
                                if total_size > 0 {
                                    percentage = (downloaded as f64 / total_size as f64) * 100.0;
                                }

                                let _ = app_handle.emit(
                                    "model-pull-progress",
                                    json!({
                                        "stage": "downloading",
                                        "model": model_id,
                                        "percentage": percentage,
                                        "completed": downloaded,
                                        "total": total_size,
                                        "message": format!("Downloading... {:.2} GB / {:.2} GB", downloaded as f64 / 1_073_741_824.0, total_size as f64 / 1_073_741_824.0),
                                    })
                                );
                                last_emit = Instant::now();
                            }
                        }
                        Some(Err(e)) => {
                            return Err(AppError::Internal(format!("Chunk error: {}", e)));
                        }
                        None => {
                            break; // Stream finished
                        }
                    }
                }
            }
        }

        if let Some(state) = app_handle.try_state::<DownloadManagerState>() {
            if let Ok(mut map) = state.0.lock() {
                map.remove(model_id);
            }
        }

        // Final flush
        file.flush()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if total_size > 0 && downloaded < total_size {
            return Err(AppError::Internal(format!(
                "Downloaded file is incomplete: {} / {} bytes",
                downloaded, total_size
            )));
        }

        // Emit 100% completion
        let _ = app_handle.emit(
            "model-pull-progress",
            json!({
                "stage": "downloading",
                "model": model_id,
                "percentage": 100.0,
                "completed": downloaded,
                "total": total_size,
                "message": "Download complete. Processing...",
            }),
        );

        Ok(())
    }
}
