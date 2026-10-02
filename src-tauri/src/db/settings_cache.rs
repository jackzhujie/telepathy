use std::collections::HashMap;
use std::sync::RwLock;
use crate::db::settings;
use rusqlite::Connection;

pub struct SettingsCache {
    cache: RwLock<HashMap<String, String>>,
}

impl SettingsCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn load_from_db(&self, conn: &Connection) {
        if let Ok(settings_map) = settings::get_all_settings(conn) {
            let mut cache = self.cache.write().unwrap();
            *cache = settings_map;
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().unwrap();
        cache.get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &str) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(key.to_string(), value.to_string());
    }

    pub fn get_all(&self) -> HashMap<String, String> {
        let cache = self.cache.read().unwrap();
        cache.clone()
    }
}
