#![allow(dead_code)]
use crate::errors::AppError;
use crate::db::settings_cache::SettingsCache;
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub const DEFAULT_SETTINGS: &[(&str, &str)] = &[
    ("chat_model", "qwen2.5"),
    ("embedding_model", "bge-large-zh"),
    ("top_k", "5"),
    ("similarity_threshold", "0.3"),
    ("num_thread", "6"),
    ("num_ctx", "4096"),
    ("temperature", "0.7"),
    ("num_gpu", "0"),
    ("repeat_penalty", "1.1"),
    ("num_predict", "1024"),
    ("enable_office_parser", "false"),
];

pub fn init_settings_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create settings table: {}", e)))?;
    Ok(())
}

pub fn seed_defaults(conn: &Connection) -> Result<(), AppError> {
    for (key, value) in DEFAULT_SETTINGS {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| AppError::Internal(format!("Failed to seed setting: {}", e)))?;
    }
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> Result<HashMap<String, String>, AppError> {
    init_settings_table(conn)?;
    seed_defaults(conn)?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row.map_err(|e| AppError::Internal(format!("Failed to read: {}", e)))?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare: {}", e)))?;
    let mut rows = stmt
        .query_map(params![key], |row| row.get::<_, String>(0))
        .map_err(|e| AppError::Internal(format!("Failed to query: {}", e)))?;
    match rows.next() {
        Some(Ok(v)) => Ok(Some(v)),
        _ => Ok(None),
    }
}

pub fn get_setting_cached(conn: &Connection, cache: &SettingsCache, key: &str) -> Result<Option<String>, AppError> {
    if let Some(value) = cache.get(key) {
        return Ok(Some(value));
    }
    if let Some(value) = get_setting(conn, key)? {
        cache.set(key, &value);
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

pub fn update_setting(conn: &Connection, cache: Option<&SettingsCache>, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| AppError::Internal(format!("Failed to update setting: {}", e)))?;
    if let Some(cache) = cache {
        cache.set(key, value);
    }
    Ok(())
}

pub fn get_top_k(conn: &Connection, cache: Option<&SettingsCache>) -> Result<usize, AppError> {
    let val = match cache {
        Some(cache) => get_setting_cached(conn, cache, "top_k")?.unwrap_or_else(|| "5".to_string()),
        None => get_setting(conn, "top_k")?.unwrap_or_else(|| "5".to_string()),
    };
    Ok(val.parse().unwrap_or(5))
}

pub fn get_similarity_threshold(conn: &Connection, cache: Option<&SettingsCache>) -> Result<f32, AppError> {
    let val = match cache {
        Some(cache) => get_setting_cached(conn, cache, "similarity_threshold")?.unwrap_or_else(|| "0.3".to_string()),
        None => get_setting(conn, "similarity_threshold")?.unwrap_or_else(|| "0.3".to_string()),
    };
    Ok(val.parse().unwrap_or(0.3))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_settings_table(&conn).unwrap();
        seed_defaults(&conn).unwrap();
        conn
    }

    #[test]
    fn test_get_all_settings() {
        let conn = setup_db();
        let settings = get_all_settings(&conn).unwrap();
        assert_eq!(settings.get("chat_model").unwrap(), "qwen2.5");
        assert_eq!(settings.len(), DEFAULT_SETTINGS.len());
    }

    #[test]
    fn test_update_setting() {
        let conn = setup_db();
        update_setting(&conn, None, "chat_model", "llama3").unwrap();
        let val = get_setting(&conn, "chat_model").unwrap().unwrap();
        assert_eq!(val, "llama3");
    }

    #[test]
    fn test_get_top_k() {
        let conn = setup_db();
        assert_eq!(get_top_k(&conn, None).unwrap(), 5);
        update_setting(&conn, None, "top_k", "10").unwrap();
        assert_eq!(get_top_k(&conn, None).unwrap(), 10);
    }

    #[test]
    fn test_get_similarity_threshold() {
        let conn = setup_db();
        let threshold = get_similarity_threshold(&conn, None).unwrap();
        assert!((threshold - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_get_setting_cached() {
        let conn = setup_db();
        let cache = SettingsCache::new();
        let val = get_setting_cached(&conn, &cache, "chat_model").unwrap().unwrap();
        assert_eq!(val, "qwen2.5");
        let val_cached = get_setting_cached(&conn, &cache, "chat_model").unwrap().unwrap();
        assert_eq!(val_cached, "qwen2.5");
    }

    #[test]
    fn test_update_setting_with_cache() {
        let conn = setup_db();
        let cache = SettingsCache::new();
        update_setting(&conn, Some(&cache), "chat_model", "llama3").unwrap();
        let val = cache.get("chat_model").unwrap();
        assert_eq!(val, "llama3");
    }
}
