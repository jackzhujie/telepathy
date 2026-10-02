use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::os::unix::fs::PermissionsExt;
fn main() {
    let path = std::env::var("HOME").unwrap() + "/Library/Application Support/com.telepathy.app/telepathy.db";

    println!("Path: {}", path);
    println!("Exists: {}", std::path::Path::new(&path).exists());

    // Read metadata
    let metadata = std::fs::metadata(&path).unwrap();
    println!("Size: {} bytes", metadata.len());
    println!("Permissions: {:o}", metadata.permissions().mode());
    println!("Readonly: {}", metadata.permissions().readonly());

    // Try opening with explicit read-write flags
    let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();
    println!("\nOpened with SQLITE_OPEN_READ_WRITE");

    // Set WAL mode first
    let r = conn.pragma_update(None, "journal_mode", &"WAL");
    println!("Set journal_mode=WAL: {:?}", r);

    // Try write
    let r = conn.execute("CREATE TABLE IF NOT EXISTS _rw_test (x INTEGER)", []);
    println!("Create: {:?}", r);
    let r = conn.execute("INSERT INTO _rw_test VALUES (1)", []);
    println!("Insert: {:?}", r);

    // Check journal_mode after write
    let jm: String = conn.pragma_query_value(None, "journal_mode", |r| r.get(0)).unwrap();
    println!("journal_mode after: {}", jm);

    // Check WAL file
    let wal_path = path + "-wal";
    println!("WAL file exists: {}", std::path::Path::new(&wal_path).exists());
    if std::path::Path::new(&wal_path).exists() {
        let m = std::fs::metadata(&wal_path).unwrap();
        println!("WAL size: {} bytes", m.len());
    }
}
