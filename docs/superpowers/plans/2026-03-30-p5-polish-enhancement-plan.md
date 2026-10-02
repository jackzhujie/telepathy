# P5: 打磨与增强 实现计划

> **For agentic workers:** Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现设置页面、错误处理优化和 UI 打磨，让 Telepathy 从"可用"变为"好用"。

**Architecture:** 配置存储在 SQLite settings 表，前端通过 Pinia store 管理，Markdown 渲染使用 markdown-it + highlight.js。

---

### Task 1: 设置数据库与后端命令

**Files:**
- Create: `src-tauri/src/db/settings.rs`
- Create: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 reqwest 依赖确认** (已有)

- [ ] **Step 2: 创建 settings 数据库模块**

在 `src-tauri/src/db/settings.rs`:
```rust
#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub const DEFAULT_SETTINGS: &[(&str, &str)] = &[
    ("ollama_url", "http://localhost:11434"),
    ("chat_model", "qwen2.5"),
    ("embedding_model", "bge-large-zh"),
    ("top_k", "5"),
    ("similarity_threshold", "0.3"),
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
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
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

pub fn update_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| AppError::Internal(format!("Failed to update setting: {}", e)))?;
    Ok(())
}

pub fn get_top_k(conn: &Connection) -> Result<usize, AppError> {
    let val = get_setting(conn, "top_k")?.unwrap_or_else(|| "5".to_string());
    Ok(val.parse().unwrap_or(5))
}

pub fn get_similarity_threshold(conn: &Connection) -> Result<f32, AppError> {
    let val = get_setting(conn, "similarity_threshold")?.unwrap_or_else(|| "0.3".to_string());
    Ok(val.parse().unwrap_or(0.3))
}
```

- [ ] **Step 3: 创建 settings 命令**

在 `src-tauri/src/commands/settings.rs`:
```rust
use crate::db::settings;
use crate::errors::AppError;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use reqwest::Client;
use serde::Deserialize;

#[tauri::command]
pub async fn get_settings(app_handle: AppHandle) -> Result<HashMap<String, String>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    settings::get_all_settings(&conn)
}

#[tauri::command]
pub async fn update_setting(
    key: String,
    value: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    settings::init_settings_table(&conn)?;
    settings::update_setting(&conn, &key, &value)
}

#[derive(Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

#[tauri::command]
pub async fn list_ollama_models(app_handle: AppHandle) -> Result<Vec<String>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;
    let url = settings::get_setting(&conn, "ollama_url")?
        .unwrap_or_else(|| "http://localhost:11434".to_string());

    let client = Client::new();
    let resp = client
        .get(format!("{}/api/tags", url))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to Ollama: {}", e)))?;

    let data: OllamaModelsResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse response: {}", e)))?;

    Ok(data.models.into_iter().map(|m| m.name).collect())
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
```

- [ ] **Step 4: 注册模块和命令**

更新 `db/mod.rs`: 添加 `pub mod settings;`
更新 `commands/mod.rs`: 添加 `pub mod settings;`
更新 `lib.rs` invoke_handler: 添加 settings 命令

- [ ] **Step 5: 编译验证 + 测试**

```bash
cd src-tauri && cargo check && cargo test
```

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/db/settings.rs src-tauri/src/commands/settings.rs src-tauri/src/db/mod.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(backend): add settings storage and configuration commands"
```

---

### Task 2: 前端设置页面

**Files:**
- Create: `src/stores/settings.ts`
- Modify: `src/views/Settings.vue`
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 添加 API 封装**

在 `src/api/tauri.ts` 添加:
```typescript
export async function getSettings(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('get_settings');
}

export async function updateSetting(key: string, value: string): Promise<void> {
  return invoke<void>('update_setting', { key, value });
}

export async function listOllamaModels(): Promise<string[]> {
  return invoke<string[]>('list_ollama_models');
}
```

- [ ] **Step 2: 创建 settings store**

在 `src/stores/settings.ts`:
```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { getSettings, updateSetting, listOllamaModels } from '@/api/tauri';

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Record<string, string>>({});
  const models = ref<string[]>([]);
  const isLoading = ref(false);

  async function fetchSettings() {
    isLoading.value = true;
    try {
      settings.value = await getSettings();
    } finally {
      isLoading.value = false;
    }
  }

  async function saveSetting(key: string, value: string) {
    await updateSetting(key, value);
    settings.value[key] = value;
  }

  async function fetchModels() {
    try {
      models.value = await listOllamaModels();
    } catch {
      models.value = [];
    }
  }

  return { settings, models, isLoading, fetchSettings, saveSetting, fetchModels };
});
```

- [ ] **Step 3: 实现 Settings.vue**

重写 Settings.vue 为完整的设置表单，包含:
- Ollama 地址输入框
- 聊天模型下拉选择（从 Ollama API 获取列表）
- 嵌入模型下拉选择
- Top-K 滑块（1-20）
- 相似度阈值滑块（0.0-1.0）

- [ ] **Step 4: 编译验证**

```bash
cd /Users/mac/project/telepathy && npx vue-tsc --noEmit
```

- [ ] **Step 5: 提交**

```bash
git add src/stores/settings.ts src/views/Settings.vue src/api/tauri.ts
git commit -m "feat(frontend): implement settings page with model selection and parameter tuning"
```

---

### Task 3: 集成配置到 RAG 管道

**Files:**
- Modify: `src-tauri/src/commands/rag.rs`
- Modify: `src-tauri/src/commands/document.rs`
- Modify: `src-tauri/src/commands/indexing.rs`

- [ ] **Step 1: 使用配置替代硬编码值**

在 rag_query 中使用 `db::settings::get_setting` 获取:
- `chat_model` 替代硬编码的 `"qwen2.5"`
- `embedding_model` 替代硬编码的 `"bge-large-zh"`
- `top_k` 替代硬编码的 `5`
- `similarity_threshold` 替代硬编码的 `0.3`
- `ollama_url` 替代硬编码的 `"http://localhost:11434"`

- [ ] **Step 2: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/rag.rs src-tauri/src/commands/indexing.rs
git commit -m "refactor: use settings config instead of hardcoded values in RAG pipeline"
```

---

### Task 4: UI 打磨 - Markdown 渲染 + 错误处理

**Files:**
- Modify: `package.json` (add markdown-it, highlight.js)
- Modify: `src/views/Chat.vue`
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 安装依赖**

```bash
npx pnpm add markdown-it highlight.js @types/markdown-it
```

- [ ] **Step 2: Chat.vue 中渲染 Markdown**

在 Chat.vue 中:
- 导入 markdown-it 和 highlight.js
- 使用 `v-html` 渲染 AI 回答
- 配置 highlight.js 代码高亮

- [ ] **Step 3: Documents.vue 空状态优化**

当文档列表为空时显示引导提示，引导用户导入文档。

- [ ] **Step 4: 编译验证**

```bash
cd /Users/mac/project/telepathy && npx vue-tsc --noEmit
```

- [ ] **Step 5: 提交**

```bash
git add -A
git commit -m "feat(frontend): add markdown rendering, code highlighting, and empty state guides"
```

---

### Task 5: 最终验证

- [ ] **Step 1: 全量编译**

```bash
cd src-tauri && cargo check
cd /Users/mac/project/telepathy && npx vue-tsc --noEmit
```

- [ ] **Step 2: 运行全部测试**

```bash
cd src-tauri && cargo test
```
