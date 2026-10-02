# 多模型管理系统 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`-[x]`) syntax for tracking.

**Goal:** 让用户能浏览、搜索、安装、删除 Ollama 模型，按类型（chat/embedding）严格分类，并基于硬件智能推荐。

**Architecture:** 新增本地模型注册表 JSON 作为元数据源，新增 Rust service 层解析注册表并与 Ollama API 交叉匹配，新增 Tauri commands 暴露给前端，在设置页中嵌入模型管理组件。

**Tech Stack:** Rust (serde, reqwest, serde_json), Vue 3 + TypeScript, reka-ui, Tailwind CSS, Pinia

---

### Task 1: 创建模型注册表 JSON

**Files:**
- Create: `src-tauri/resources/model_registry.json`

-[x] **Step 1: 创建注册表文件**

```json
{
  "version": "1.0.0",
  "updated_at": "2026-04-11",
  "models": [
    {
      "id": "qwen2.5",
      "name": "Qwen 2.5",
      "provider": "Alibaba",
      "type": "chat",
      "description": "强大的中英双语模型，中文理解能力突出",
      "sizes": [
        { "tag": "0.5b", "params": "0.5B", "min_memory_gb": 2, "file_size_gb": 0.4 },
        { "tag": "1.5b", "params": "1.5B", "min_memory_gb": 4, "file_size_gb": 1.0 },
        { "tag": "3b", "params": "3B", "min_memory_gb": 4, "file_size_gb": 2.0 },
        { "tag": "7b", "params": "7B", "min_memory_gb": 8, "file_size_gb": 4.7 },
        { "tag": "14b", "params": "14B", "min_memory_gb": 16, "file_size_gb": 9.0 },
        { "tag": "32b", "params": "32B", "min_memory_gb": 32, "file_size_gb": 20.0 }
      ]
    },
    {
      "id": "qwen3",
      "name": "Qwen 3",
      "provider": "Alibaba",
      "type": "chat",
      "description": "通义千问最新一代，推理与对话能力大幅提升",
      "sizes": [
        { "tag": "0.6b", "params": "0.6B", "min_memory_gb": 2, "file_size_gb": 0.4 },
        { "tag": "1.7b", "params": "1.7B", "min_memory_gb": 4, "file_size_gb": 1.1 },
        { "tag": "4b", "params": "4B", "min_memory_gb": 4, "file_size_gb": 2.6 },
        { "tag": "8b", "params": "8B", "min_memory_gb": 8, "file_size_gb": 5.2 },
        { "tag": "14b", "params": "14B", "min_memory_gb": 16, "file_size_gb": 9.0 },
        { "tag": "30b", "params": "30B", "min_memory_gb": 32, "file_size_gb": 19.0 }
      ]
    },
    {
      "id": "llama3.2",
      "name": "Llama 3.2",
      "provider": "Meta",
      "type": "chat",
      "description": "Meta 最新开源模型，英文能力强",
      "sizes": [
        { "tag": "1b", "params": "1B", "min_memory_gb": 4, "file_size_gb": 0.7 },
        { "tag": "3b", "params": "3B", "min_memory_gb": 4, "file_size_gb": 2.0 }
      ]
    },
    {
      "id": "deepseek-r1",
      "name": "DeepSeek R1",
      "provider": "DeepSeek",
      "type": "chat",
      "description": "推理能力强，适合数学和编程",
      "sizes": [
        { "tag": "1.5b", "params": "1.5B", "min_memory_gb": 4, "file_size_gb": 1.1 },
        { "tag": "7b", "params": "7B", "min_memory_gb": 8, "file_size_gb": 4.7 },
        { "tag": "14b", "params": "14B", "min_memory_gb": 16, "file_size_gb": 9.0 },
        { "tag": "32b", "params": "32B", "min_memory_gb": 32, "file_size_gb": 20.0 }
      ]
    },
    {
      "id": "gemma3",
      "name": "Gemma 3",
      "provider": "Google",
      "type": "chat",
      "description": "Google 最新轻量模型，性能优秀",
      "sizes": [
        { "tag": "1b", "params": "1B", "min_memory_gb": 2, "file_size_gb": 0.8 },
        { "tag": "4b", "params": "4B", "min_memory_gb": 4, "file_size_gb": 3.0 },
        { "tag": "12b", "params": "12B", "min_memory_gb": 12, "file_size_gb": 8.1 },
        { "tag": "27b", "params": "27B", "min_memory_gb": 24, "file_size_gb": 17.0 }
      ]
    },
    {
      "id": "phi4",
      "name": "Phi-4",
      "provider": "Microsoft",
      "type": "chat",
      "description": "微软小参数高智能模型",
      "sizes": [
        { "tag": "14b", "params": "14B", "min_memory_gb": 16, "file_size_gb": 9.1 }
      ]
    },
    {
      "id": "mistral",
      "name": "Mistral",
      "provider": "Mistral AI",
      "type": "chat",
      "description": "法国 AI 公司旗舰模型，综合能力强",
      "sizes": [
        { "tag": "7b", "params": "7B", "min_memory_gb": 8, "file_size_gb": 4.1 }
      ]
    },
    {
      "id": "llava",
      "name": "LLaVA",
      "provider": "LLaVA Team",
      "type": "vision",
      "description": "多模态视觉语言模型，可理解图片",
      "sizes": [
        { "tag": "7b", "params": "7B", "min_memory_gb": 8, "file_size_gb": 4.7 },
        { "tag": "13b", "params": "13B", "min_memory_gb": 16, "file_size_gb": 8.0 }
      ]
    },
    {
      "id": "bge-m3",
      "name": "BGE-M3",
      "provider": "BAAI",
      "type": "embedding",
      "description": "多语言嵌入模型，中文效果好",
      "sizes": [
        { "tag": "latest", "params": "567M", "min_memory_gb": 2, "file_size_gb": 1.2 }
      ]
    },
    {
      "id": "bge-large-zh",
      "name": "BGE Large (中文)",
      "provider": "BAAI",
      "type": "embedding",
      "description": "中文专用嵌入，精度高",
      "sizes": [
        { "tag": "latest", "params": "335M", "min_memory_gb": 2, "file_size_gb": 0.67 }
      ]
    },
    {
      "id": "nomic-embed-text",
      "name": "Nomic Embed Text",
      "provider": "Nomic AI",
      "type": "embedding",
      "description": "高质量英文嵌入模型",
      "sizes": [
        { "tag": "latest", "params": "137M", "min_memory_gb": 2, "file_size_gb": 0.27 }
      ]
    },
    {
      "id": "mxbai-embed-large",
      "name": "MxBAI Embed Large",
      "provider": "Mixedbread AI",
      "type": "embedding",
      "description": "多语言嵌入，维度高",
      "sizes": [
        { "tag": "latest", "params": "335M", "min_memory_gb": 2, "file_size_gb": 0.67 }
      ]
    }
  ]
}
```

-[x] **Step 2: Commit**

```bash
git add src-tauri/resources/model_registry.json
git commit -m "feat: add local model registry JSON with 12 models"
```

---

### Task 2: 创建 Rust 模型注册表 Service

**Files:**
- Create: `src-tauri/src/services/model_registry.rs`
- Modify: `src-tauri/src/services/mod.rs`

-[x] **Step 1: 创建 model_registry.rs**

```rust
#![allow(dead_code)]
use crate::errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Chat,
    Embedding,
    Vision,
    Unknown,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Chat => write!(f, "chat"),
            ModelType::Embedding => write!(f, "embedding"),
            ModelType::Vision => write!(f, "vision"),
            ModelType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSize {
    pub tag: String,
    pub params: String,
    pub min_memory_gb: u32,
    pub file_size_gb: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub description: String,
    pub sizes: Vec<ModelSize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub version: String,
    pub updated_at: String,
    pub models: Vec<RegistryModel>,
}

#[derive(Debug, Serialize, Clone)]
pub struct InstalledModel {
    pub full_name: String,
    pub base_name: String,
    pub tag: String,
    pub model_type: ModelType,
    pub size_bytes: u64,
    pub display_name: String,
    pub provider: String,
    pub description: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Recommendation {
    Recommended,
    Marginal,
    NotRecommended,
}

#[derive(Debug, Serialize, Clone)]
pub struct ModelRecommendation {
    pub model: RegistryModel,
    pub size: ModelSize,
    pub recommendation: Recommendation,
}

/// Load the built-in model registry from the embedded JSON
pub fn load_registry() -> Result<ModelRegistry, AppError> {
    let json_str = include_str!("../../resources/model_registry.json");
    serde_json::from_str(json_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse model registry: {}", e)))
}

/// Filter registry models by type
pub fn filter_by_type(registry: &ModelRegistry, model_type: &ModelType) -> Vec<RegistryModel> {
    registry
        .models
        .iter()
        .filter(|m| &m.model_type == model_type)
        .cloned()
        .collect()
}

/// Search registry models by keyword (searches id, name, provider, description)
pub fn search_models(registry: &ModelRegistry, keyword: &str) -> Vec<RegistryModel> {
    let kw = keyword.to_lowercase();
    registry
        .models
        .iter()
        .filter(|m| {
            m.id.to_lowercase().contains(&kw)
                || m.name.to_lowercase().contains(&kw)
                || m.provider.to_lowercase().contains(&kw)
                || m.description.to_lowercase().contains(&kw)
        })
        .cloned()
        .collect()
}

/// Parse an Ollama model name like "qwen2.5:7b" into (base_name, tag)
pub fn parse_model_name(full_name: &str) -> (String, String) {
    if let Some(pos) = full_name.rfind(':') {
        let base = full_name[..pos].to_string();
        let tag = full_name[pos + 1..].to_string();
        (base, tag)
    } else {
        (full_name.to_string(), "latest".to_string())
    }
}

/// Match an installed model name against the registry to determine its type
pub fn match_model_type(registry: &ModelRegistry, base_name: &str) -> (ModelType, String, String, String) {
    for model in &registry.models {
        if model.id == base_name {
            return (
                model.model_type.clone(),
                model.name.clone(),
                model.provider.clone(),
                model.description.clone(),
            );
        }
    }
    (ModelType::Unknown, base_name.to_string(), "Unknown".to_string(), String::new())
}

/// Get recommendations based on available system memory
pub fn get_recommendations(registry: &ModelRegistry, available_memory_gb: f64) -> Vec<ModelRecommendation> {
    let mut recommendations = Vec::new();
    for model in &registry.models {
        for size in &model.sizes {
            let recommendation = if available_memory_gb >= size.min_memory_gb as f64 * 1.5 {
                Recommendation::Recommended
            } else if available_memory_gb >= size.min_memory_gb as f64 {
                Recommendation::Marginal
            } else {
                Recommendation::NotRecommended
            };
            recommendations.push(ModelRecommendation {
                model: model.clone(),
                size: size.clone(),
                recommendation,
            });
        }
    }
    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_registry() {
        let registry = load_registry().unwrap();
        assert!(!registry.models.is_empty());
        assert!(registry.models.iter().any(|m| m.id == "qwen2.5"));
    }

    #[test]
    fn test_parse_model_name() {
        let (base, tag) = parse_model_name("qwen2.5:7b");
        assert_eq!(base, "qwen2.5");
        assert_eq!(tag, "7b");

        let (base2, tag2) = parse_model_name("bge-m3");
        assert_eq!(base2, "bge-m3");
        assert_eq!(tag2, "latest");
    }

    #[test]
    fn test_filter_by_type() {
        let registry = load_registry().unwrap();
        let chat_models = filter_by_type(&registry, &ModelType::Chat);
        assert!(chat_models.iter().all(|m| m.model_type == ModelType::Chat));
        let embedding_models = filter_by_type(&registry, &ModelType::Embedding);
        assert!(embedding_models.iter().all(|m| m.model_type == ModelType::Embedding));
    }

    #[test]
    fn test_search_models() {
        let registry = load_registry().unwrap();
        let results = search_models(&registry, "qwen");
        assert!(results.iter().any(|m| m.id == "qwen2.5"));
    }

    #[test]
    fn test_match_model_type() {
        let registry = load_registry().unwrap();
        let (model_type, _, _, _) = match_model_type(&registry, "qwen2.5");
        assert_eq!(model_type, ModelType::Chat);
        let (model_type2, _, _, _) = match_model_type(&registry, "bge-m3");
        assert_eq!(model_type2, ModelType::Embedding);
        let (model_type3, _, _, _) = match_model_type(&registry, "unknown-model");
        assert_eq!(model_type3, ModelType::Unknown);
    }

    #[test]
    fn test_get_recommendations() {
        let registry = load_registry().unwrap();
        let recs = get_recommendations(&registry, 16.0);
        assert!(!recs.is_empty());
    }
}
```

-[x] **Step 2: 注册模块到 `src-tauri/src/services/mod.rs`**

在文件末尾添加一行：

```rust
pub mod model_registry;
```

-[x] **Step 3: 运行测试确认**

```bash
cd src-tauri && cargo test services::model_registry --lib
```

Expected: 所有 5 个测试通过

-[x] **Step 4: Commit**

```bash
git add src-tauri/src/services/model_registry.rs src-tauri/src/services/mod.rs
git commit -m "feat: add model registry service with type matching and recommendations"
```

---

### Task 3: 创建模型管理 Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/models.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

-[x] **Step 1: 创建 `src-tauri/src/commands/models.rs`**

```rust
use crate::db::settings;
use crate::errors::AppError;
use crate::services::model_registry::{
    self, InstalledModel, ModelRecommendation, ModelType, RegistryModel,
};
use reqwest::Client;
use serde::Deserialize;
use tauri::{AppHandle, Manager};

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Deserialize)]
struct OllamaModelInfo {
    name: String,
    size: u64,
}

fn get_db_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
    }
    Ok(app_data_dir.join("telepathy.db"))
}

fn get_ollama_url(app_handle: &AppHandle) -> Result<String, AppError> {
    let db_path = get_db_path(app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    settings::init_settings_table(&conn)?;
    settings::seed_defaults(&conn)?;
    Ok(settings::get_setting(&conn, "ollama_url")?
        .unwrap_or_else(|| "http://localhost:11434".to_string()))
}

#[tauri::command]
pub async fn list_registry_models(
    model_type: Option<String>,
) -> Result<Vec<RegistryModel>, AppError> {
    let registry = model_registry::load_registry()?;
    match model_type {
        Some(t) => {
            let mt = match t.to_lowercase().as_str() {
                "chat" => ModelType::Chat,
                "embedding" => ModelType::Embedding,
                "vision" => ModelType::Vision,
                _ => return Ok(registry.models),
            };
            Ok(model_registry::filter_by_type(&registry, &mt))
        }
        None => Ok(registry.models),
    }
}

#[tauri::command]
pub async fn list_installed_models(
    app_handle: AppHandle,
) -> Result<Vec<InstalledModel>, AppError> {
    let ollama_url = get_ollama_url(&app_handle)?;
    let registry = model_registry::load_registry()?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    let resp = client
        .get(format!("{}/api/tags", ollama_url))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to Ollama: {}", e)))?;

    let data: OllamaTagsResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse Ollama response: {}", e)))?;

    let installed: Vec<InstalledModel> = data
        .models
        .into_iter()
        .map(|m| {
            let (base_name, tag) = model_registry::parse_model_name(&m.name);
            let (model_type, display_name, provider, description) =
                model_registry::match_model_type(&registry, &base_name);
            InstalledModel {
                full_name: m.name,
                base_name,
                tag,
                model_type,
                size_bytes: m.size,
                display_name,
                provider,
                description,
            }
        })
        .collect();

    Ok(installed)
}

#[tauri::command]
pub async fn get_model_recommendations(
    app_handle: AppHandle,
) -> Result<Vec<ModelRecommendation>, AppError> {
    let registry = model_registry::load_registry()?;

    // Get system memory via sysinfo
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0;

    // Also check what's already installed to mark them
    let ollama_url = get_ollama_url(&app_handle)?;
    let _installed = match Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| Client::new())
        .get(format!("{}/api/tags", ollama_url))
        .send()
        .await
    {
        Ok(resp) => resp
            .json::<OllamaTagsResponse>()
            .await
            .map(|d| d.models.into_iter().map(|m| m.name).collect::<Vec<_>>())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    Ok(model_registry::get_recommendations(&registry, total_memory_gb))
}

#[tauri::command]
pub async fn install_model(
    model_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let ollama_url = get_ollama_url(&app_handle)?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3600)) // 1h for large downloads
        .build()
        .unwrap_or_else(|_| Client::new());

    let resp = client
        .post(format!("{}/api/pull", ollama_url))
        .json(&serde_json::json!({ "name": model_id, "stream": true }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to pull model: {}", e)))?;

    use futures_util::StreamExt;
    use tauri::Emitter;

    let mut stream = resp.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        if let Ok(bytes) = chunk {
            buffer.extend_from_slice(&bytes);
            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes = buffer.drain(..=pos).collect::<Vec<u8>>();
                if let Ok(text) = String::from_utf8(line_bytes) {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        let status = json.get("status").and_then(|s| s.as_str()).unwrap_or("");
                        let total = json.get("total").and_then(|t| t.as_u64()).unwrap_or(0);
                        let completed = json.get("completed").and_then(|c| c.as_u64()).unwrap_or(0);
                        let percentage = if total > 0 {
                            (completed as f64 / total as f64 * 100.0) as u32
                        } else {
                            0
                        };
                        let _ = app_handle.emit(
                            "model-pull-progress",
                            serde_json::json!({
                                "model": model_id,
                                "status": status,
                                "percentage": percentage,
                                "completed": completed,
                                "total": total,
                            }),
                        );
                    }
                }
            }
        }
    }

    let _ = app_handle.emit(
        "model-pull-done",
        serde_json::json!({ "model": model_id }),
    );

    Ok(())
}

#[tauri::command]
pub async fn delete_model(model_name: String, app_handle: AppHandle) -> Result<(), AppError> {
    let ollama_url = get_ollama_url(&app_handle)?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| Client::new());

    let resp = client
        .delete(format!("{}/api/delete", ollama_url))
        .json(&serde_json::json!({ "name": model_name }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete model: {}", e)))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "Ollama returned error status: {}",
            resp.status()
        )));
    }

    Ok(())
}
```

-[x] **Step 2: 注册模块到 `src-tauri/src/commands/mod.rs`**

在文件末尾添加一行：

```rust
pub mod models;
```

-[x] **Step 3: 在 `src-tauri/src/lib.rs` 注册命令**

在 `.invoke_handler(tauri::generate_handler![` 块中，在 `commands::profile::remove_user_interest,` 之后添加：

```rust
            commands::models::list_registry_models,
            commands::models::list_installed_models,
            commands::models::get_model_recommendations,
            commands::models::install_model,
            commands::models::delete_model,
```

-[x] **Step 4: 编译验证**

```bash
cd src-tauri && cargo build 2>&1 | tail -5
```

Expected: 编译成功（可能有 unused warnings，可忽略）

-[x] **Step 5: Commit**

```bash
git add src-tauri/src/commands/models.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add model management Tauri commands (list/install/delete/recommend)"
```

---

### Task 4: 前端 API 层和类型定义

**Files:**
- Modify: `src/api/tauri.ts`
- Create: `src/types/models.ts`

-[x] **Step 1: 创建类型定义 `src/types/models.ts`**

```typescript
export type ModelType = 'chat' | 'embedding' | 'vision' | 'unknown';

export interface ModelSize {
  tag: string;
  params: string;
  min_memory_gb: number;
  file_size_gb: number;
}

export interface RegistryModel {
  id: string;
  name: string;
  provider: string;
  model_type: ModelType;
  description: string;
  sizes: ModelSize[];
}

export interface InstalledModel {
  full_name: string;
  base_name: string;
  tag: string;
  model_type: ModelType;
  size_bytes: number;
  display_name: string;
  provider: string;
  description: string;
}

export type RecommendationLevel = 'recommended' | 'marginal' | 'notrecommended';

export interface ModelRecommendation {
  model: RegistryModel;
  size: ModelSize;
  recommendation: RecommendationLevel;
}

export interface ModelPullProgress {
  model: string;
  status: string;
  percentage: number;
  completed: number;
  total: number;
}
```

-[x] **Step 2: 在 `src/api/tauri.ts` 末尾添加 API 函数**

在文件的最后（269行 `regenerateMessage` 函数之后）添加：

```typescript
import type { RegistryModel, InstalledModel, ModelRecommendation } from '@/types/models';

export async function listRegistryModels(modelType?: string): Promise<RegistryModel[]> {
  return invoke<RegistryModel[]>('list_registry_models', { modelType });
}

export async function listInstalledModels(): Promise<InstalledModel[]> {
  return invoke<InstalledModel[]>('list_installed_models');
}

export async function getModelRecommendations(): Promise<ModelRecommendation[]> {
  return invoke<ModelRecommendation[]>('get_model_recommendations');
}

export async function installModel(modelId: string): Promise<void> {
  return invoke<void>('install_model', { modelId });
}

export async function deleteModel(modelName: string): Promise<void> {
  return invoke<void>('delete_model', { modelName });
}
```

-[x] **Step 3: Commit**

```bash
git add src/types/models.ts src/api/tauri.ts
git commit -m "feat: add model management types and API functions"
```

---

### Task 5: 扩展 Settings Store — 模型管理状态

**Files:**
- Modify: `src/stores/settings.ts`

-[x] **Step 1: 在 `src/stores/settings.ts` 中添加模型管理状态和 actions**

在文件顶部 import 部分添加新的 API 引用：

```typescript
import {
  listRegistryModels,
  listInstalledModels,
  getModelRecommendations,
  installModel as installModelApi,
  deleteModel as deleteModelApi,
} from '@/api/tauri';
import type { RegistryModel, InstalledModel, ModelRecommendation, ModelPullProgress } from '@/types/models';
import { listen } from '@tauri-apps/api/event';
```

在 store 内部（`refreshingModels` 之后）添加新的 ref：

```typescript
  const registryModels = ref<RegistryModel[]>([]);
  const installedModels = ref<InstalledModel[]>([]);
  const modelRecommendations = ref<ModelRecommendation[]>([]);
  const installingModel = ref<string | null>(null);
  const installProgress = ref<ModelPullProgress | null>(null);
```

在 store 内部添加 computed：

```typescript
  const chatModels = computed(() =>
    installedModels.value.filter(m => m.model_type === 'chat')
  );
  const embeddingModels = computed(() =>
    installedModels.value.filter(m => m.model_type === 'embedding')
  );
```

在 store 内部添加 actions：

```typescript
  async function fetchRegistryModels(modelType?: string) {
    try {
      registryModels.value = await listRegistryModels(modelType);
    } catch (e: any) {
      console.error('[Registry] Error:', e);
    }
  }

  async function fetchInstalledModels() {
    try {
      installedModels.value = await listInstalledModels();
    } catch (e: any) {
      console.error('[InstalledModels] Error:', e);
    }
  }

  async function fetchModelRecommendations() {
    try {
      modelRecommendations.value = await getModelRecommendations();
    } catch (e: any) {
      console.error('[Recommendations] Error:', e);
    }
  }

  async function installNewModel(modelId: string) {
    installingModel.value = modelId;
    installProgress.value = null;

    const unlisten = await listen<ModelPullProgress>('model-pull-progress', (event) => {
      installProgress.value = event.payload;
    });

    const unlistenDone = await listen<{ model: string }>('model-pull-done', async () => {
      installingModel.value = null;
      installProgress.value = null;
      unlisten();
      unlistenDone();
      await fetchInstalledModels();
      await fetchModels();
    });

    try {
      await installModelApi(modelId);
    } catch (e: any) {
      console.error('[Install] Error:', e);
      installingModel.value = null;
      installProgress.value = null;
      unlisten();
      unlistenDone();
    }
  }

  async function removeModel(modelName: string) {
    try {
      await deleteModelApi(modelName);
      await fetchInstalledModels();
      await fetchModels();
    } catch (e: any) {
      console.error('[Delete] Error:', e);
      throw e;
    }
  }
```

在 return 语句中暴露新增的所有 state/computed/actions。

-[x] **Step 2: Commit**

```bash
git add src/stores/settings.ts
git commit -m "feat: extend settings store with model management state and actions"
```

---

### Task 6: 创建 ModelManager 前端组件

**Files:**
- Create: `src/components/settings/ModelManager.vue`
- Modify: `src/views/Settings.vue`

-[x] **Step 1: 创建 `src/components/settings/ModelManager.vue`**

该组件包含三个区域（引擎选择、已安装列表、搜索安装），使用 reka-ui 的 Select/Progress 组件，尺寸与 Tailwind 类名匹配现有设置页的视觉风格。完整的 Vue SFC 代码（含 `<script setup>`, `<template>`）需要实现：

- 区域 A：两个 reka-ui SelectRoot 分别对应 chat_model 和 embedding_model，选项从 `chatModels` / `embeddingModels` computed 过滤
- 区域 B：卡片网格展示 `installedModels`，每张卡片包含模型名、类型 badge（chat=蓝、embedding=绿、vision=紫、unknown=灰）、大小、删除按钮
- 区域 C：搜索框 + 类型筛选 Tab + 注册表模型列表，每个模型可展开查看 sizes，每个 size 有安装按钮 + 推荐图标
- 下载进度条复用 reka-ui ProgressRoot

-[x] **Step 2: 在 Settings.vue 中替换现有"模型管理" Accordion 内容**

将 Settings.vue 中 `<AccordionItem value="model-management">` 的 `<AccordionContent>` 内容替换为：

```vue
<AccordionContent class="overflow-hidden data-[state=open]:animate-slideDown data-[state=closed]:animate-slideUp">
  <div class="p-4">
    <ModelManager />
  </div>
</AccordionContent>
```

并在 `<script setup>` 中添加导入：

```typescript
import ModelManager from '@/components/settings/ModelManager.vue';
```

-[x] **Step 3: Commit**

```bash
git add src/components/settings/ModelManager.vue src/views/Settings.vue
git commit -m "feat: add ModelManager component and integrate into settings page"
```

---

### Task 7: 集成测试与收尾

**Files:**
- 无新增文件，验证全流程

-[x] **Step 1: 启动应用验证**

```bash
pnpm tauri dev
```

验证清单：
1. 设置页 → 模型管理 → 展开后能看到"引擎选择"、"已安装模型"、"搜索安装"三个区域
2. 引擎选择只显示对应类型的已安装模型
3. 搜索框输入 "qwen" 能过滤出注册表中的 qwen 模型
4. 点击安装按钮能触发下载，进度条正常显示
5. 删除按钮能删除非当前引擎的模型

-[x] **Step 2: 删除旧的 RECOMMENDED_MODELS 硬编码**

从 `src/api/tauri.ts` 中删除 `RECOMMENDED_MODELS` 常量和 `RecommendedModel` 接口（约 158-189 行）。
更新 `src/stores/settings.ts` 中引用 `RECOMMENDED_MODELS` 的相关代码。
更新 `Settings.vue` 中引用旧的 `recommendedModels` 和 `pullModel` 的 UI 部分（no-models 状态）改用新的 ModelManager 组件。

-[x] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete multi-model management system with registry, recommendations, and UI"
```
