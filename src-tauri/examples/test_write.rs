
// Test debug route file writing
fn main() {
    let dir = std::env::var("TELEPATHY_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME not found");
            std::path::PathBuf::from(home).join(".data")
        });
    println!("Using dir: {:?}", dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        println!("create_dir_all failed: {}", e);
        return;
    }
    let payload = serde_json::json!({
        "route": "/settings",
        "title": "tele",
        "ts": 12345
    });
    let file = dir.join("test_route.json");
    if let Err(e) = std::fs::write(&file, serde_json::to_string(&payload).unwrap_or_default()) {
        println!("write failed: {}", e);
        return;
    }
    println!("Wrote to {:?}", file);
    if let Ok(s) = std::fs::read_to_string(&file) {
        println!("Read back: {}", s);
    }
}
