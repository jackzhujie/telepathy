// Debug-only command: write the current route + page title to a file
// so the AI / external tools can verify the navigation worked.

use std::path::PathBuf;
use tauri::Manager;

#[tauri::command]
pub fn tele_debug_report_route(
    app: tauri::AppHandle,
    route: String,
    title: String,
) -> Result<(), String> {
    let dir: PathBuf = if let Ok(custom) = std::env::var("TELEPATHY_DATA_DIR") {
        PathBuf::from(custom)
    } else {
        app.path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let payload = format!(
        "{{\"route\":\"{}\",\"title\":\"{}\",\"ts\":{}}}",
        route.replace('"', "\\\""),
        title.replace('"', "\\\""),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let _ = std::fs::write(dir.join("current_route.json"), payload);
    Ok(())
}
