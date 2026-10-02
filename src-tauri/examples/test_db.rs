use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("com.telepathy.app");
    path.push("telepathy.db");
    
    println!("Path: {:?}", path);
    println!("Exists: {}", path.exists());
    
    // Same as Tauri app does
    let conn = Connection::open(&path).expect("Failed to open");
    
    // Check if it's readonly
    let r = conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('_test_seed', 'ok')",
        [],
    );
    println!("Insert result: {:?}", r);
    
    if r.is_err() {
        // Try multiple things
        println!("\n=== Trying PRAGMA quick_check ===");
        let mut stmt = conn.prepare("PRAGMA quick_check").unwrap();
        for row in stmt.query_map([], |r| r.get::<_, String>(0)).unwrap() {
            println!("  - {}", row.unwrap());
        }
        
        println!("\n=== Trying PRAGMA database_list ===");
        let mut stmt = conn.prepare("PRAGMA database_list").unwrap();
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        }).unwrap();
        for row in rows {
            let (seq, name, file) = row.unwrap();
            println!("  seq={} name={} file={}", seq, name, file);
        }
        
        // Try opening with different flags
        use rusqlite::OpenFlags;
        let conn2 = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE).expect("Failed to open 2");
        let r2 = conn2.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('_test_seed2', 'ok')",
            [],
        );
        println!("Open with flags result: {:?}", r2);
    }
}
