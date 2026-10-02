use crate::errors::AppError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use usearch::ffi::{Index, new_cos};
use cxx::UniquePtr;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum HnswMeta {
    V2 {
        dimension: usize,
        chunk_ids: std::collections::HashMap<String, String>,
    },
    V1(std::collections::HashMap<String, String>),
}

pub struct HnswIndex {
    index: UniquePtr<Index>,
    chunk_ids: RwLock<HashMap<usize, String>>,
    dimension: usize,
}

unsafe impl Send for HnswIndex {}
unsafe impl Sync for HnswIndex {}


pub struct HnswIndexManager {
    indexes: RwLock<HashMap<Option<String>, Arc<HnswIndex>>>,
    data_dir: PathBuf,
}

impl HnswIndex {
    pub fn new(dimension: usize) -> Result<Self, AppError> {
        let index = new_cos(dimension, "f32", 16, 128, 64)
            .map_err(|e| AppError::Internal(format!("Failed to create HNSW index: {}", e.what())))?;

        // Reserve initial capacity to prevent out of bounds crashes
        index.reserve(1000)
            .map_err(|e| AppError::Internal(format!("Failed to reserve initial capacity: {}", e.what())))?;

        Ok(Self {
            index,
            chunk_ids: RwLock::new(HashMap::new()),
            dimension,
        })
    }

    pub fn search(
        &self,
        embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<(String, f32)>, AppError> {
        if embedding.len() != self.dimension {
            return Err(AppError::Internal(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            )));
        }

        let results = self
            .index
            .search(embedding, top_k)
            .map_err(|e| AppError::Internal(format!("Search failed: {}", e.what())))?;

        let chunk_ids = self.chunk_ids.read().unwrap();
        let search_results: Vec<(String, f32)> = results
            .labels
            .iter()
            .zip(results.distances.iter())
            .filter_map(|(&key, &distance)| {
                chunk_ids.get(&(key as usize)).map(|id| (id.clone(), distance))
            })
            .collect();

        Ok(search_results)
    }

    pub fn add(&self, chunk_id: &str, embedding: &[f32]) -> Result<(), AppError> {
        if embedding.len() != self.dimension {
            return Err(AppError::Internal(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            )));
        }

        let current_size = self.index.size();
        let current_capacity = self.index.capacity();
        if current_size >= current_capacity {
            let new_capacity = (current_capacity * 2).max(1000);
            self.index.reserve(new_capacity)
                .map_err(|e| AppError::Internal(format!("Failed to dynamically reserve HNSW capacity: {}", e.what())))?;
        }

        let key = current_size as u32;
        self.index
            .add(key, embedding)
            .map_err(|e| AppError::Internal(format!("Add vector failed: {}", e.what())))?;

        let mut chunk_ids = self.chunk_ids.write().unwrap();
        chunk_ids.insert(key as usize, chunk_id.to_string());

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.index.size()
    }

    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        self.index
            .save(path.to_str().unwrap())
            .map_err(|e| AppError::Internal(format!("Save index failed: {}", e.what())))
    }

    pub fn load(path: &Path, dimension: usize) -> Result<Self, AppError> {
        let quant = "f32";
        // Fix: Pre-allocate index using the actual dimension of the model (1024 or custom) and healthy connectivity/ef options
        let index = new_cos(dimension, &quant, 16, 128, 64)
            .map_err(|e: cxx::Exception| AppError::Internal(format!("Failed to create temporary index for loading: {}", e.what())))?;
        
        index
            .load(path.to_str().unwrap())
            .map_err(|e: cxx::Exception| AppError::Internal(format!("Failed to load index: {}", e.what())))?;

        let actual_dimension = index.dimensions();
        Ok(Self {
            index,
            chunk_ids: RwLock::new(HashMap::new()),
            dimension: actual_dimension,
        })
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn set_chunk_ids(&self, chunk_ids: HashMap<usize, String>) {
        let mut ids = self.chunk_ids.write().unwrap();
        *ids = chunk_ids;
    }
}

impl HnswIndexManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            indexes: RwLock::new(HashMap::new()),
            data_dir,
        }
    }

    fn get_index_path(&self, project_id: Option<&str>) -> PathBuf {
        let filename = match project_id {
            Some(id) => format!("hnsw_{}.usearch", id),
            None => "hnsw_default.usearch".to_string(),
        };
        self.data_dir.join(filename)
    }

    fn get_meta_path(&self, project_id: Option<&str>) -> PathBuf {
        let filename = match project_id {
            Some(id) => format!("hnsw_{}.meta.json", id),
            None => "hnsw_default.meta.json".to_string(),
        };
        self.data_dir.join(filename)
    }

    pub fn get_or_create_index(
        &self,
        dimension: usize,
        project_id: Option<&str>,
    ) -> Result<Arc<HnswIndex>, AppError> {
        let key = project_id.map(|s| s.to_string());

        {
            let indexes = self.indexes.read().unwrap();
            if let Some(index) = indexes.get(&key) {
                return Ok(Arc::clone(index));
            }
        }

        let index_path = self.get_index_path(project_id);
        let meta_path = self.get_meta_path(project_id);

        let hnsw_index = if index_path.exists() {
            let mut resolved_dimension = dimension;
            let mut chunk_ids_opt = None;

            if meta_path.exists() {
                let load_meta = || -> Result<_, Box<dyn std::error::Error>> {
                    let content = std::fs::read_to_string(&meta_path)?;
                    let meta: HnswMeta = serde_json::from_str(&content)?;
                    Ok(meta)
                };
                match load_meta() {
                    Ok(meta) => match meta {
                        HnswMeta::V2 { dimension: dim, chunk_ids } => {
                            resolved_dimension = dim;
                            let converted: HashMap<usize, String> = chunk_ids
                                .into_iter()
                                .filter_map(|(k, v)| k.parse::<usize>().ok().map(|n| (n, v)))
                                .collect();
                            chunk_ids_opt = Some(converted);
                        }
                        HnswMeta::V1(ids) => {
                            let converted: HashMap<usize, String> = ids
                                .into_iter()
                                .filter_map(|(k, v)| k.parse::<usize>().ok().map(|n| (n, v)))
                                .collect();
                            chunk_ids_opt = Some(converted);
                        }
                    },
                    Err(e) => {
                        eprintln!("[HNSW] Warning: failed to load or parse metadata at {:?}: {}", meta_path, e);
                    }
                }
            }

            let index = HnswIndex::load(&index_path, resolved_dimension)?;
            if let Some(chunk_ids) = chunk_ids_opt {
                index.set_chunk_ids(chunk_ids);
            }
            index
        } else {
            HnswIndex::new(dimension)?
        };

        let index = Arc::new(hnsw_index);

        let mut indexes = self.indexes.write().unwrap();
        indexes.insert(key, Arc::clone(&index));

        Ok(index)
    }

    pub fn search(
        &self,
        embedding: &[f32],
        top_k: usize,
        dimension: usize,
        project_id: Option<&str>,
    ) -> Result<Vec<(String, f32)>, AppError> {
        let index = self.get_or_create_index(dimension, project_id)?;
        index.search(embedding, top_k)
    }

    pub fn add_chunk(
        &self,
        chunk_id: &str,
        embedding: &[f32],
        dimension: usize,
        project_id: Option<&str>,
    ) -> Result<(), AppError> {
        let index = self.get_or_create_index(dimension, project_id)?;
        index.add(chunk_id, embedding)?;

        if let Err(e) = self.save_index(project_id) {
            eprintln!("[HNSW] Warning: failed to save index: {}", e);
        }

        Ok(())
    }

    pub fn save_index(&self, project_id: Option<&str>) -> Result<(), AppError> {
        let key = project_id.map(|s| s.to_string());

        let indexes = self.indexes.read().unwrap();
        if let Some(index) = indexes.get(&key) {
            let index_path = self.get_index_path(project_id);
            let meta_path = self.get_meta_path(project_id);

            index.save(&index_path)?;

            let chunk_ids: HashMap<String, String> = index
                .chunk_ids
                .read()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect();
            let meta = HnswMeta::V2 {
                dimension: index.dimension(),
                chunk_ids,
            };
            let meta_content = serde_json::to_string_pretty(&meta)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            std::fs::write(&meta_path, meta_content)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    pub fn save_all(&self) -> Result<(), AppError> {
        let indexes = self.indexes.read().unwrap();
        for project_id in indexes.keys() {
            let index_path = self.get_index_path(project_id.as_deref());
            let meta_path = self.get_meta_path(project_id.as_deref());

            if let Some(index) = indexes.get(project_id) {
                index.save(&index_path)?;

                let chunk_ids: HashMap<String, String> = index
                    .chunk_ids
                    .read()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect();
                let meta = HnswMeta::V2 {
                    dimension: index.dimension(),
                    chunk_ids,
                };
                let meta_content = serde_json::to_string_pretty(&meta)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                std::fs::write(&meta_path, meta_content)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub fn rebuild_index(
        &self,
        project_id: Option<&str>,
        conn: &rusqlite::Connection,
        dimension: usize,
    ) -> Result<(), AppError> {
        let index_path = self.get_index_path(project_id);
        let meta_path = self.get_meta_path(project_id);

        if index_path.exists() {
            std::fs::remove_file(&index_path).ok();
        }
        if meta_path.exists() {
            std::fs::remove_file(&meta_path).ok();
        }

        let mut indexes = self.indexes.write().unwrap();
        indexes.remove(&project_id.map(|s| s.to_string()));

        let new_index = HnswIndex::new(dimension)?;
        let key = project_id.map(|s| s.to_string());

        let count = if let Some(pid) = project_id {
            let mut stmt = conn
                .prepare(
                    "SELECT c.id, e.embedding FROM chunks c
                     JOIN chunk_embeddings e ON c.id = e.chunk_id
                     JOIN documents d ON c.document_id = d.id
                     WHERE d.project_id = ?1",
                )
                .map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;

            let rows: Vec<(String, Vec<f32>)> = stmt
                .query_map([pid], |row| {
                    let id: String = row.get(0)?;
                    let bytes: Vec<u8> = row.get(1)?;
                    Ok((id, crate::db::vectors::bytes_to_f32_vec(&bytes)))
                })
                .map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?
                .filter_map(|r| r.ok())
                .collect();

            let mut cnt = 0;
            for (id, embedding) in rows {
                if new_index.add(&id, &embedding).is_ok() {
                    cnt += 1;
                }
            }
            cnt
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT c.id, e.embedding FROM chunks c
                     JOIN chunk_embeddings e ON c.id = e.chunk_id",
                )
                .map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;

            let rows: Vec<(String, Vec<f32>)> = stmt
                .query_map([], |row| {
                    let id: String = row.get(0)?;
                    let bytes: Vec<u8> = row.get(1)?;
                    Ok((id, crate::db::vectors::bytes_to_f32_vec(&bytes)))
                })
                .map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?
                .filter_map(|r| r.ok())
                .collect();

            let mut cnt = 0;
            for (id, embedding) in rows {
                if new_index.add(&id, &embedding).is_ok() {
                    cnt += 1;
                }
            }
            cnt
        };

        let index = Arc::new(new_index);
        indexes.insert(key, Arc::clone(&index));
        drop(indexes);

        self.save_index(project_id)?;

        println!("[HNSW] Rebuilt index with {} vectors", count);
        Ok(())
    }

    pub fn load_existing_indexes(&self) {
        let data_dir = &self.data_dir;
        if !data_dir.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "usearch") {
                    let file_stem = path.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string());

                    let project_id = file_stem.filter(|s| s != "default");

                    let meta_path = self.get_meta_path(project_id.as_deref());
                    let mut dimension = 1024;
                    let mut chunk_ids_opt = None;

                    if meta_path.exists() {
                        let load_meta = || -> Result<_, Box<dyn std::error::Error>> {
                            let content = std::fs::read_to_string(&meta_path)?;
                            let meta: HnswMeta = serde_json::from_str(&content)?;
                            Ok(meta)
                        };
                        match load_meta() {
                            Ok(meta) => match meta {
                                HnswMeta::V2 { dimension: dim, chunk_ids } => {
                                    dimension = dim;
                                    let converted: HashMap<usize, String> = chunk_ids
                                        .into_iter()
                                        .filter_map(|(k, v)| k.parse::<usize>().ok().map(|n| (n, v)))
                                        .collect();
                                    chunk_ids_opt = Some(converted);
                                }
                                HnswMeta::V1(ids) => {
                                    let converted: HashMap<usize, String> = ids
                                        .into_iter()
                                        .filter_map(|(k, v)| k.parse::<usize>().ok().map(|n| (n, v)))
                                        .collect();
                                    chunk_ids_opt = Some(converted);
                                }
                            },
                            Err(e) => {
                                eprintln!("[HNSW] Warning: failed to load or parse metadata at {:?}: {}", meta_path, e);
                            }
                        }
                    }

                    if let Ok(index) = HnswIndex::load(&path, dimension) {
                        if let Some(chunk_ids) = chunk_ids_opt {
                            index.set_chunk_ids(chunk_ids);
                        }

                        let mut indexes = self.indexes.write().unwrap();
                        indexes.insert(project_id, Arc::new(index));
                        println!("[HNSW] Loaded index from {:?}", path);
                    }
                }
            }
        }
    }
}

use std::sync::{Mutex, OnceLock};

// SAFETY: HnswIndexManager is only accessed through the Mutex above, which serializes all access.
unsafe impl Send for HnswIndexManager {}

static HNSW_MANAGER: OnceLock<Mutex<HnswIndexManager>> = OnceLock::new();

pub fn init_hnsw_manager(data_dir: std::path::PathBuf) {
    let _ = HNSW_MANAGER.set(Mutex::new(HnswIndexManager::new(data_dir)));
}

pub fn with_hnsw_manager<F, R>(f: F) -> Result<R, AppError>
where
    F: FnOnce(&HnswIndexManager) -> R,
{
    HNSW_MANAGER.get().map(|m| f(&*m.lock().unwrap())).ok_or_else(|| {
        AppError::Internal("HNSW manager not initialized".to_string())
    })
}

pub fn load_existing_indexes_on_startup() {
    if let Err(e) = with_hnsw_manager(|manager| {
        manager.load_existing_indexes();
    }) {
        eprintln!("[HNSW] Failed to load existing indexes: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_hnsw_save_and_load() {
        println!("Test started.");
        
        let index = HnswIndex::new(128).unwrap();
        println!("Index created successfully. Size: {}", index.size());
        
        let vector = vec![1.0; 128];
        println!("Adding vector...");
        index.add("chunk_1", &vector).unwrap();
        assert_eq!(index.size(), 1);
        println!("Vector added successfully. Size is now {}", index.size());

        // Verify persistence and restoration
        let test_path = "test_index.usearch";
        println!("Saving index to {}...", test_path);
        index.save(&Path::new(test_path)).unwrap();

        println!("Restoring index from {}...", test_path);
        let restored = HnswIndex::load(&Path::new(test_path), 128).unwrap();
        assert_eq!(restored.size(), 1);
        assert_eq!(restored.dimension(), 128);
        println!("Index restored successfully. Dimensions: {}", restored.dimension());

        // Clean up
        let _ = fs::remove_file(test_path);
        println!("Test completed successfully.");
    }

    #[test]
    fn test_hnsw_manager_metadata_compatibility() {
        let temp_dir_path = std::env::temp_dir().join(format!("telepathy_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir_path).unwrap();
        let manager = HnswIndexManager::new(temp_dir_path.clone());
        
        let project_id = Some("test_proj");
        let dimension = 128;
        
        // 1. Create and add a chunk
        manager.add_chunk("chunk_v2", &vec![1.0; 128], dimension, project_id).unwrap();
        
        // save_index is implicitly called inside add_chunk, but let's call it explicitly as well
        manager.save_index(project_id).unwrap();
        
        // 2. Verify that the saved metadata is in V2 format
        let meta_path = manager.get_meta_path(project_id);
        assert!(meta_path.exists());
        let meta_content = fs::read_to_string(&meta_path).unwrap();
        
        // Attempt to parse it as HnswMeta and ensure it matches V2
        let parsed_meta: HnswMeta = serde_json::from_str(&meta_content).unwrap();
        match parsed_meta {
            HnswMeta::V2 { dimension: dim, chunk_ids } => {
                assert_eq!(dim, dimension);
                assert_eq!(chunk_ids.len(), 1);
                assert_eq!(chunk_ids.get("0").unwrap(), "chunk_v2");
            }
            HnswMeta::V1(_) => {
                panic!("Expected V2 metadata format, found V1");
            }
        }
        
        // 3. Test V1 compatibility
        // Create V1 metadata format (just a HashMap<usize, String>)
        let mut v1_map = HashMap::new();
        v1_map.insert(0, "chunk_v1".to_string());
        let v1_meta_content = serde_json::to_string_pretty(&v1_map).unwrap();
        
        let v1_project_id = Some("v1_proj");
        let v1_index_path = manager.get_index_path(v1_project_id);
        let v1_meta_path = manager.get_meta_path(v1_project_id);
        
        // Create and save temporary index
        let index = HnswIndex::new(dimension).unwrap();
        index.add("chunk_v1", &vec![2.0; 128]).unwrap();
        index.save(&v1_index_path).unwrap();
        
        // Write V1 metadata format file
        fs::write(&v1_meta_path, v1_meta_content).unwrap();
        
        // Load index using HnswIndexManager and check V1 compatibility
        let loaded_v1_index = manager.get_or_create_index(dimension, v1_project_id).unwrap();
        assert_eq!(loaded_v1_index.size(), 1);
        
        let ids = loaded_v1_index.chunk_ids.read().unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids.get(&0).unwrap(), "chunk_v1");

        // Clean up temp dir
        let _ = fs::remove_dir_all(&temp_dir_path);
    }

    #[test]
    fn test_hnsw_rebuild_index_persistence() {
        let temp_dir_path = std::env::temp_dir().join(format!("telepathy_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir_path).unwrap();
        let manager = HnswIndexManager::new(temp_dir_path.clone());
        let project_id = Some("test_rebuild_proj");
        let dimension = 128;

        // 1. 初始化数据库用于重建索引
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE documents (id TEXT PRIMARY KEY, project_id TEXT)",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE chunks (id TEXT PRIMARY KEY, document_id TEXT)",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE chunk_embeddings (chunk_id TEXT PRIMARY KEY, embedding BLOB)",
            [],
        ).unwrap();

        // 插入测试数据
        conn.execute(
            "INSERT INTO documents (id, project_id) VALUES ('doc1', 'test_rebuild_proj')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO chunks (id, document_id) VALUES ('chunk1', 'doc1')",
            [],
        ).unwrap();
        let embedding_bytes = crate::db::vectors::f32_vec_to_bytes(&vec![0.5; 128]);
        conn.execute(
            "INSERT INTO chunk_embeddings (chunk_id, embedding) VALUES ('chunk1', ?1)",
            [embedding_bytes],
        ).unwrap();

        // 2. 第一次 rebuild_index
        manager.rebuild_index(project_id, &conn, dimension).unwrap();

        // 3. 验证对应的 HNSW 索引和 metadata 是否保存至磁盘
        let index_path = manager.get_index_path(project_id);
        let meta_path = manager.get_meta_path(project_id);
        assert!(index_path.exists(), "Rebuilt index path should exist on disk");
        assert!(meta_path.exists(), "Rebuilt meta path should exist on disk");

        // 清理临时目录
        let _ = fs::remove_dir_all(&temp_dir_path);
    }

    #[test]
    fn test_hnsw_metadata_corrupt_warning() {
        let temp_dir_path = std::env::temp_dir().join(format!("telepathy_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir_path).unwrap();
        let manager = HnswIndexManager::new(temp_dir_path.clone());
        let project_id = Some("corrupt_meta_proj");
        let dimension = 128;

        // 1. 写入无效/损坏的元数据 JSON
        let meta_path = manager.get_meta_path(project_id);
        fs::write(&meta_path, "invalid json data").unwrap();

        // 2. 创建并保存对应 HNSW 索引文件，以触发 get_or_create_index 的读取逻辑
        let index_path = manager.get_index_path(project_id);
        let index = HnswIndex::new(dimension).unwrap();
        index.save(&index_path).unwrap();

        // 3. 触发 get_or_create_index
        // 虽然元数据损坏，但加载过程应当恢复（fallback）而不能崩溃，并打印警告。
        let result = manager.get_or_create_index(dimension, project_id);
        assert!(result.is_ok(), "Should fallback and return Ok even if metadata is corrupt");

        // 清理临时目录
        let _ = fs::remove_dir_all(&temp_dir_path);
    }
}
