
# 基于项目的知识库重构实现计划

&gt; **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将文档管理和知识库重构为基于"项目"的结构，支持按项目组织文档和知识库

**Architecture:** 
- 新增 projects 表，修改 documents 表添加 project_id 字段
- 新增项目管理 API 和前端状态管理
- 修改三个页面（Documents、KnowledgeBase、Chat）以支持项目选择

**Tech Stack:** Rust + Tauri + Vue 3 + Naive UI + SQLite

---

## Task 1: 后端数据库层 - projects 表

**Files:**
- Create: `src-tauri/src/db/projects.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: 创建 projects.rs 数据库模块**

```rust
#![allow(dead_code)]
use crate::errors::AppError;
use rusqlite::{params, Connection};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option&lt;String&gt;,
    pub created_at: String,
}

pub fn init_projects_table(conn: &amp;Connection) -&gt; Result&lt;(), AppError&gt; {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create projects table: {}", e)))?;
    Ok(())
}

pub fn insert_project(conn: &amp;Connection, project: &amp;Project) -&gt; Result&lt;(), AppError&gt; {
    conn.execute(
        "INSERT INTO projects (id, name, description, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            project.id,
            project.name,
            project.description,
            project.created_at,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert project: {}", e)))?;
    Ok(())
}

pub fn get_all_projects(conn: &amp;Connection) -&gt; Result&lt;Vec&lt;Project&gt;, AppError&gt; {
    let mut stmt = conn
        .prepare("SELECT id, name, description, created_at FROM projects ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query projects: {}", e)))?;

    let mut projects = Vec::new();
    for row in rows {
        projects.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }
    Ok(projects)
}

pub fn update_project(
    conn: &amp;Connection,
    id: &amp;str,
    name: &amp;str,
    description: Option&lt;&amp;str&gt;,
) -&gt; Result&lt;(), AppError&gt; {
    conn.execute(
        "UPDATE projects SET name = ?1, description = ?2 WHERE id = ?3",
        params![name, description, id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to update project: {}", e)))?;
    Ok(())
}

pub fn delete_project(conn: &amp;Connection, id: &amp;str) -&gt; Result&lt;(), AppError&gt; {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(format!("Failed to delete project: {}", e)))?;
    Ok(())
}
```

- [ ] **Step 2: 在 db/mod.rs 中导出 projects 模块**

```rust
pub mod documents;
pub mod projects;
pub mod settings;
pub mod vectors;
```

- [ ] **Step 3: 编译验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/projects.rs src-tauri/src/db/mod.rs
git commit -m "feat(backend): add projects database module"
```

---

## Task 2: 后端数据库层 - 修改 documents 表

**Files:**
- Modify: `src-tauri/src/db/documents.rs`
- Modify: `src-tauri/src/commands/document.rs`
- Modify: `src-tauri/src/commands/indexing.rs`
- Modify: `src-tauri/src/commands/knowledge.rs`

- [ ] **Step 1: 修改 Document 结构体添加 project_id 字段**

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub original_path: String,
    pub library_path: String,
    pub file_type: String,
    pub size: i64,
    pub status: String,
    pub error_msg: Option&lt;String&gt;,
    pub project_id: Option&lt;String&gt;,
    pub created_at: String,
}
```

- [ ] **Step 2: 修改 init_documents_table 添加 project_id 列**

```rust
pub fn init_documents_table(conn: &amp;Connection) -&gt; Result&lt;(), AppError&gt; {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            original_path TEXT NOT NULL,
            library_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            error_msg TEXT,
            project_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| AppError::Internal(format!("Failed to create documents table: {}", e)))?;
    Ok(())
}
```

- [ ] **Step 3: 修改 insert_document 添加 project_id**

```rust
pub fn insert_document(conn: &amp;Connection, doc: &amp;Document) -&gt; Result&lt;(), AppError&gt; {
    conn.execute(
        "INSERT INTO documents (id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            doc.id,
            doc.name,
            doc.original_path,
            doc.library_path,
            doc.file_type,
            doc.size,
            doc.status,
            doc.error_msg,
            doc.project_id,
            doc.created_at,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert document: {}", e)))?;
    Ok(())
}
```

- [ ] **Step 4: 添加 get_documents_by_project 函数**

```rust
pub fn get_documents_by_project(conn: &amp;Connection, project_id: &amp;str) -&gt; Result&lt;Vec&lt;Document&gt;, AppError&gt; {
    let mut stmt = conn
        .prepare("SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at 
                  FROM documents 
                  WHERE project_id = ?1 
                  ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map(params![project_id], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                original_path: row.get(2)?,
                library_path: row.get(3)?,
                file_type: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                error_msg: row.get(7)?,
                project_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query documents: {}", e)))?;

    let mut documents = Vec::new();
    for row in rows {
        documents.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }
    Ok(documents)
}
```

- [ ] **Step 5: 修改 get_all_documents 和 get_document 以包含 project_id**

```rust
pub fn get_all_documents(conn: &amp;Connection) -&gt; Result&lt;Vec&lt;Document&gt;, AppError&gt; {
    let mut stmt = conn
        .prepare("SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at FROM documents ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                original_path: row.get(2)?,
                library_path: row.get(3)?,
                file_type: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                error_msg: row.get(7)?,
                project_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query documents: {}", e)))?;

    let mut documents = Vec::new();
    for row in rows {
        documents.push(row.map_err(|e| AppError::Internal(format!("Failed to read row: {}", e)))?);
    }
    Ok(documents)
}

pub fn get_document(conn: &amp;Connection, id: &amp;str) -&gt; Result&lt;Option&lt;Document&gt;, AppError&gt; {
    let mut stmt = conn
        .prepare("SELECT id, name, original_path, library_path, file_type, size, status, error_msg, project_id, created_at FROM documents WHERE id = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare query: {}", e)))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                original_path: row.get(2)?,
                library_path: row.get(3)?,
                file_type: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                error_msg: row.get(7)?,
                project_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query document: {}", e)))?;

    match rows.next() {
        Some(row) =&gt; Ok(Some(row.map_err(|e| {
            AppError::Internal(format!("Failed to read row: {}", e))
        })?)),
        None =&gt; Ok(None),
    }
}
```

- [ ] **Step 6: 修改所有其他相关文件的 Document 字段映射**
  - 在 `document.rs` 命令中更新 `add_document` 以接受 `project_id`
  - 在 `indexing.rs` 中确保查询包含 `project_id`
  - 在 `knowledge.rs` 中确保查询可以按 `project_id` 过滤

- [ ] **Step 7: 编译验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/db/documents.rs src-tauri/src/commands/document.rs src-tauri/src/commands/indexing.rs src-tauri/src/commands/knowledge.rs
git commit -m "feat(backend): add project_id to documents table"
```

---

## Task 3: 后端命令层 - projects 命令

**Files:**
- Create: `src-tauri/src/commands/projects.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 projects.rs 命令模块**

```rust
use crate::db::projects;
use crate::errors::AppError;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option&lt;String&gt;,
}

#[derive(serde::Deserialize)]
pub struct UpdateProjectInput {
    pub id: String,
    pub name: String,
    pub description: Option&lt;String&gt;,
}

#[tauri::command]
pub async fn create_project(
    input: CreateProjectInput,
    app_handle: AppHandle,
) -&gt; Result&lt;projects::Project, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    projects::init_projects_table(&amp;conn)?;

    let project = projects::Project {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        description: input.description,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    projects::insert_project(&amp;conn, &amp;project)?;
    Ok(project)
}

#[tauri::command]
pub async fn list_projects(app_handle: AppHandle) -&gt; Result&lt;Vec&lt;projects::Project&gt;, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    projects::init_projects_table(&amp;conn)?;
    projects::get_all_projects(&amp;conn)
}

#[tauri::command]
pub async fn update_project(
    input: UpdateProjectInput,
    app_handle: AppHandle,
) -&gt; Result&lt;(), AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    projects::update_project(&amp;conn, &amp;input.id, &amp;input.name, input.description.as_deref())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_project(id: String, app_handle: AppHandle) -&gt; Result&lt;(), AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    projects::delete_project(&amp;conn, &amp;id)?;
    Ok(())
}

fn get_db_path(app_handle: &amp;AppHandle) -&gt; Result&lt;std::path::PathBuf, AppError&gt; {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&amp;app_data_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
    }

    Ok(app_data_dir.join("telepathy.db"))
}
```

- [ ] **Step 2: 在 commands/mod.rs 中导出 projects 模块**

```rust
pub mod document;
pub mod installer;
pub mod knowledge;
pub mod ollama;
pub mod projects;
pub mod rag;
pub mod settings;
```

- [ ] **Step 3: 在 lib.rs 中注册 projects 命令**

找到 `invoke_handler` 并添加：

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    projects::create_project,
    projects::list_projects,
    projects::update_project,
    projects::delete_project,
])
```

- [ ] **Step 4: 编译验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/projects.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(backend): add projects commands"
```

---

## Task 4: 后端命令层 - 修改 document 命令

**Files:**
- Modify: `src-tauri/src/commands/document.rs`

- [ ] **Step 1: 修改 add_document 接受 project_id**

```rust
#[tauri::command]
pub async fn add_document(
    file_path: String,
    project_id: String,
    app_handle: AppHandle,
) -&gt; Result&lt;Document, AppError&gt; {
    // ... existing code ...
    
    let doc = Document {
        id: Uuid::new_v4().to_string(),
        name: filename,
        original_path: file_path.clone(),
        library_path: library_path.to_string_lossy().to_string(),
        file_type: ext.to_string(),
        size: metadata.len() as i64,
        status: "pending".to_string(),
        error_msg: None,
        project_id: Some(project_id),
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    
    // ... rest of function ...
}
```

- [ ] **Step 2: 添加 get_documents_by_project 命令**

```rust
#[tauri::command]
pub async fn get_documents_by_project(
    project_id: String,
    app_handle: AppHandle,
) -&gt; Result&lt;Vec&lt;Document&gt;, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = rusqlite::Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&amp;conn)?;
    documents::get_documents_by_project(&amp;conn, &amp;project_id)
}
```

- [ ] **Step 3: 编译验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/document.rs
git commit -m "feat(backend): modify document commands for project support"
```

---

## Task 5: 后端命令层 - 修改 knowledge 命令

**Files:**
- Modify: `src-tauri/src/commands/knowledge.rs`

- [ ] **Step 1: 修改 get_indexed_documents 接受 project_ids 过滤**

```rust
#[derive(serde::Deserialize)]
pub struct GetIndexedDocumentsInput {
    pub project_ids: Option&lt;Vec&lt;String&gt;&gt;,
}

#[tauri::command]
pub async fn get_indexed_documents(
    input: GetIndexedDocumentsInput,
    app_handle: AppHandle,
) -&gt; Result&lt;Vec&lt;IndexedDocument&gt;, AppError&gt; {
    let db_path = get_db_path(&amp;app_handle)?;
    let conn = rusqlite::Connection::open(&amp;db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    documents::init_documents_table(&amp;conn)?;
    vectors::init_vector_tables(&amp;conn)?;

    let (where_clause, params) = match input.project_ids {
        Some(ids) if !ids.is_empty() =&gt; {
            let placeholders: Vec&lt;_&gt; = ids.iter().map(|_| "?").collect();
            (
                format!("AND d.project_id IN ({})", placeholders.join(",")),
                ids,
            )
        }
        _ =&gt; ("".to_string(), vec![]),
    };

    let sql = format!(
        "SELECT d.id, d.name, d.file_type, d.created_at,
                (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) as chunk_count
         FROM documents d
         WHERE d.status = 'indexed' {}
         ORDER BY d.created_at DESC",
        where_clause
    );

    // ... rest of function with params binding ...
}
```

- [ ] **Step 2: 类似修改 get_document_chunks_detail 和 search_chunks**

- [ ] **Step 3: 编译验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/knowledge.rs
git commit -m "feat(backend): modify knowledge commands for project filtering"
```

---

## Task 6: 后端命令层 - 修改 rag 命令

**Files:**
- Modify: `src-tauri/src/commands/rag.rs`

- [ ] **Step 1: 修改 rag_query 接受 project_ids 过滤**

```rust
#[derive(serde::Deserialize)]
pub struct RagQueryInput {
    pub conversation_id: String,
    pub query: String,
    pub project_ids: Option&lt;Vec&lt;String&gt;&gt;,
}

#[tauri::command]
pub async fn rag_query(
    input: RagQueryInput,
    app_handle: AppHandle,
) -&gt; Result&lt;(), AppError&gt; {
    // ... existing code ...
    
    // 在向量检索时添加 project_id 过滤
    // 需要修改 vectors::search_similar 接受 project_ids 参数
}
```

- [ ] **Step 2: 修改 vectors.rs 添加项目过滤**

- [ ] **Step 3: 编译验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/rag.rs src-tauri/src/db/vectors.rs
git commit -m "feat(backend): modify rag for project filtering"
```

---

## Task 7: 前端类型和 API

**Files:**
- Create: `src/types/project.ts`
- Modify: `src/types/document.ts`
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 创建 project.ts 类型**

```typescript
export interface Project {
  id: string;
  name: string;
  description?: string;
  created_at: string;
}

export interface CreateProjectInput {
  name: string;
  description?: string;
}

export interface UpdateProjectInput {
  id: string;
  name: string;
  description?: string;
}
```

- [ ] **Step 2: 修改 document.ts 添加 project_id**

```typescript
export interface Document {
  id: string;
  name: string;
  original_path: string;
  library_path: string;
  file_type: string;
  size: number;
  status: string;
  error_msg?: string;
  project_id?: string;
  created_at: string;
}
```

- [ ] **Step 3: 在 tauri.ts 中添加项目 API**

```typescript
import type { Project, CreateProjectInput, UpdateProjectInput } from '@/types/project';

export async function createProject(input: CreateProjectInput): Promise&lt;Project&gt; {
  return invoke&lt;Project&gt;('create_project', { input });
}

export async function listProjects(): Promise&lt;Project[]&gt; {
  return invoke&lt;Project[]&gt;('list_projects');
}

export async function updateProject(input: UpdateProjectInput): Promise&lt;void&gt; {
  return invoke&lt;void&gt;('update_project', { input });
}

export async function deleteProject(id: string): Promise&lt;void&gt; {
  return invoke&lt;void&gt;('delete_project', { id });
}

export async function getDocumentsByProject(projectId: string): Promise&lt;Document[]&gt; {
  return invoke&lt;Document[]&gt;('get_documents_by_project', { projectId });
}

// 修改 addDocument 接受 projectId
export async function addDocument(filePath: string, projectId: string): Promise&lt;Document&gt; {
  return invoke&lt;Document&gt;('add_document', { filePath, projectId });
}
```

- [ ] **Step 4: Commit**

```bash
git add src/types/project.ts src/types/document.ts src/api/tauri.ts
git commit -m "feat(frontend): add project types and API"
```

---

## Task 8: 前端 Store - projects

**Files:**
- Create: `src/stores/projects.ts`

- [ ] **Step 1: 创建 projects store**

```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Project, CreateProjectInput, UpdateProjectInput } from '@/types/project';
import { createProject, listProjects, updateProject, deleteProject } from '@/api/tauri';

export const useProjectsStore = defineStore('projects', () =&gt; {
  const projects = ref&lt;Project[]&gt;([]);
  const currentProjectId = ref&lt;string | null&gt;(null);
  const isLoading = ref(false);

  const currentProject = computed(() =&gt; 
    projects.value.find(p =&gt; p.id === currentProjectId.value)
  );

  async function fetchProjects() {
    isLoading.value = true;
    try {
      projects.value = await listProjects();
    } finally {
      isLoading.value = false;
    }
  }

  async function addProject(input: CreateProjectInput) {
    const project = await createProject(input);
    projects.value.unshift(project);
    if (!currentProjectId.value) {
      currentProjectId.value = project.id;
    }
    return project;
  }

  async function editProject(input: UpdateProjectInput) {
    await updateProject(input);
    const index = projects.value.findIndex(p =&gt; p.id === input.id);
    if (index !== -1) {
      projects.value[index] = { ...projects.value[index], ...input };
    }
  }

  async function removeProject(id: string) {
    await deleteProject(id);
    projects.value = projects.value.filter(p =&gt; p.id !== id);
    if (currentProjectId.value === id) {
      currentProjectId.value = projects.value[0]?.id || null;
    }
  }

  function setCurrentProject(id: string | null) {
    currentProjectId.value = id;
  }

  return {
    projects,
    currentProjectId,
    currentProject,
    isLoading,
    fetchProjects,
    addProject,
    editProject,
    removeProject,
    setCurrentProject,
  };
});
```

- [ ] **Step 2: Commit**

```bash
git add src/stores/projects.ts
git commit -m "feat(frontend): add projects store"
```

---

## Task 9: 前端页面 - 修改 Documents.vue

**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 添加项目选择器和管理按钮**

```vue
&lt;template&gt;
  &lt;div class="documents-page"&gt;
    &lt;div class="documents-header"&gt;
      &lt;h2&gt;文档管理&lt;/h2&gt;
      &lt;div class="header-actions"&gt;
        &lt;n-select
          v-model:value="projectsStore.currentProjectId"
          :options="projectOptions"
          placeholder="选择项目"
          style="width: 200px"
          @update:value="handleProjectChange"
        /&gt;
        &lt;n-button @click="showCreateProjectModal = true"&gt;新建项目&lt;/n-button&gt;
        &lt;n-button
          v-if="projectsStore.currentProject"
          quaternary
          @click="showEditProjectModal = true"
        &gt;
          编辑
        &lt;/n-button&gt;
        &lt;n-button
          v-if="projectsStore.currentProject"
          type="error"
          quaternary
          @click="handleDeleteProject"
        &gt;
          删除
        &lt;/n-button&gt;
      &lt;/div&gt;
    &lt;/div&gt;

    &lt;n-alert v-if="projectsStore.projects.length === 0" title="暂无项目" type="warning"&gt;
      请先创建一个项目来开始导入文档
      &lt;template #icon&gt;
        &lt;n-icon&gt;&lt;WarningOutlined /&gt;&lt;/n-icon&gt;
      &lt;/template&gt;
    &lt;/n-alert&gt;

    &lt;div v-else-if="!projectsStore.currentProject" class="no-project-selected"&gt;
      &lt;n-empty description="请选择一个项目" /&gt;
    &lt;/div&gt;

    &lt;template v-else&gt;
      &lt;div class="project-info"&gt;
        &lt;h3&gt;{{ projectsStore.currentProject?.name }}&lt;/h3&gt;
        &lt;p v-if="projectsStore.currentProject?.description"&gt;
          {{ projectsStore.currentProject?.description }}
        &lt;/p&gt;
      &lt;/div&gt;

      &lt;n-button type="primary" @click="handleImport" :loading="store.isImporting"&gt;
        导入文档
      &lt;/n-button&gt;

      &lt;!-- rest of existing template --&gt;
    &lt;/template&gt;

    &lt;!-- 项目创建/编辑模态框 --&gt;
    &lt;n-modal v-model:show="showCreateProjectModal" preset="card" title="新建项目"&gt;
      &lt;n-form :model="createForm"&gt;
        &lt;n-form-item label="项目名称" required&gt;
          &lt;n-input v-model:value="createForm.name" placeholder="请输入项目名称" /&gt;
        &lt;/n-form-item&gt;
        &lt;n-form-item label="项目描述"&gt;
          &lt;n-input
            v-model:value="createForm.description"
            type="textarea"
            placeholder="请输入项目描述"
          /&gt;
        &lt;/n-form-item&gt;
      &lt;/n-form&gt;
      &lt;template #footer&gt;
        &lt;n-button @click="showCreateProjectModal = false"&gt;取消&lt;/n-button&gt;
        &lt;n-button type="primary" @click="handleCreateProject"&gt;创建&lt;/n-button&gt;
      &lt;/template&gt;
    &lt;/n-modal&gt;

    &lt;!-- 类似的编辑模态框 --&gt;
  &lt;/div&gt;
&lt;/template&gt;

&lt;script setup lang="ts"&gt;
import { useProjectsStore } from '@/stores/projects';
// ... other imports ...

const projectsStore = useProjectsStore();
const showCreateProjectModal = ref(false);
const showEditProjectModal = ref(false);

const createForm = ref({ name: '', description: '' });
const editForm = ref({ name: '', description: '' });

const projectOptions = computed(() =&gt;
  projectsStore.projects.map(p =&gt; ({ label: p.name, value: p.id }))
);

async function handleProjectChange() {
  if (projectsStore.currentProjectId) {
    await store.fetchDocumentsByProject(projectsStore.currentProjectId);
  }
}

async function handleCreateProject() {
  if (!createForm.value.name) return;
  await projectsStore.addProject(createForm.value);
  showCreateProjectModal.value = false;
  createForm.value = { name: '', description: '' };
}

async function handleDeleteProject() {
  if (!projectsStore.currentProjectId) return;
  await projectsStore.removeProject(projectsStore.currentProjectId);
}

// 修改 handleImport 使用当前项目
async function handleImport() {
  if (!projectsStore.currentProjectId) {
    message.warning('请先选择一个项目');
    return;
  }
  // ... rest of import logic, pass projectId to addDocument
}

onMounted(() =&gt; {
  projectsStore.fetchProjects();
  // ... rest of onMounted
});
&lt;/script&gt;
```

- [ ] **Step 2: 修改 store 以支持项目**

修改 `src/stores/documents.ts` 添加 `fetchDocumentsByProject` 方法。

- [ ] **Step 3: Commit**

```bash
git add src/views/Documents.vue src/stores/documents.ts
git commit -m "feat(frontend): update Documents.vue for project support"
```

---

## Task 10: 前端页面 - 修改 KnowledgeBase.vue

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: 添加项目多选器**

```vue
&lt;template&gt;
  &lt;div class="knowledge-base-page"&gt;
    &lt;div class="page-header"&gt;
      &lt;h2&gt;知识库&lt;/h2&gt;
      &lt;n-select
        v-model:value="selectedProjectIds"
        :options="projectOptions"
        multiple
        placeholder="选择项目（默认全部）"
        max-tag-count="responsive"
        style="width: 400px"
        @update:value="handleProjectFilterChange"
      /&gt;
    &lt;/div&gt;

    &lt;!-- rest of existing template --&gt;
  &lt;/div&gt;
&lt;/template&gt;

&lt;script setup lang="ts"&gt;
import { useProjectsStore } from '@/stores/projects';
// ... other imports ...

const projectsStore = useProjectsStore();
const selectedProjectIds = ref&lt;string[]&gt;([]);

const projectOptions = computed(() =&gt;
  projectsStore.projects.map(p =&gt; ({ label: p.name, value: p.id }))
);

async function handleProjectFilterChange() {
  await loadIndexedDocuments();
}

// 修改 loadIndexedDocuments 传递 projectIds
async function loadIndexedDocuments() {
  const input = {
    project_ids: selectedProjectIds.value.length &gt; 0 ? selectedProjectIds.value : undefined,
  };
  indexedDocuments.value = await getIndexedDocuments(input);
}

onMounted(() =&gt; {
  projectsStore.fetchProjects();
  loadIndexedDocuments();
});
&lt;/script&gt;
```

- [ ] **Step 2: Commit**

```bash
git add src/views/KnowledgeBase.vue
git commit -m "feat(frontend): update KnowledgeBase.vue for project filtering"
```

---

## Task 11: 前端页面 - 修改 Chat.vue

**Files:**
- Modify: `src/views/Chat.vue`

- [ ] **Step 1: 添加项目选择器**

```vue
&lt;template&gt;
  &lt;div class="chat-page"&gt;
    &lt;div class="chat-header"&gt;
      &lt;h2&gt;AI 对话&lt;/h2&gt;
      &lt;div class="knowledge-source-selector"&gt;
        &lt;span&gt;知识库来源：&lt;/span&gt;
        &lt;n-select
          v-model:value="selectedProjectIds"
          :options="projectOptions"
          multiple
          placeholder="全部项目"
          max-tag-count="responsive"
          style="width: 300px"
        /&gt;
      &lt;/div&gt;
    &lt;/div&gt;

    &lt;!-- rest of existing template --&gt;
  &lt;/div&gt;
&lt;/template&gt;

&lt;script setup lang="ts"&gt;
import { useProjectsStore } from '@/stores/projects';
// ... other imports ...

const projectsStore = useProjectsStore();
const selectedProjectIds = ref&lt;string[]&gt;([]);

const projectOptions = computed(() =&gt;
  projectsStore.projects.map(p =&gt; ({ label: p.name, value: p.id }))
);

// 修改 sendMessage 传递 projectIds
async function sendMessage() {
  // ... existing code ...
  
  const input = {
    conversation_id: currentConversationId.value,
    query: inputMessage.value,
    project_ids: selectedProjectIds.value.length &gt; 0 ? selectedProjectIds.value : undefined,
  };
  
  await ragQuery(input);
  
  // ... rest of function
}

onMounted(() =&gt; {
  projectsStore.fetchProjects();
  // ... rest of onMounted
});
&lt;/script&gt;
```

- [ ] **Step 2: Commit**

```bash
git add src/views/Chat.vue
git commit -m "feat(frontend): update Chat.vue for project selection"
```

---

## Task 12: 测试和验证

**Files:** 全项目

- [ ] **Step 1: 完整编译**

Run: `pnpm tauri build`
Expected: Build completes successfully

- [ ] **Step 2: 手动测试**
  - 创建项目
  - 导入文档到项目
  - 索引文档
  - 在知识库查看项目内容
  - 在 AI 对话使用项目知识库

- [ ] **Step 3: Commit（如果有修复）**

---

## 实现完成

恭喜！基于项目的知识库重构已完成。

### Summary of Changes

- 新增 `projects` 表和相关数据库操作
- 修改 `documents` 表添加 `project_id` 字段
- 新增项目管理 API 和前端 Store
- 修改 Documents.vue 添加项目选择和管理
- 修改 KnowledgeBase.vue 添加项目过滤
- 修改 Chat.vue 添加项目选择
- 保持了向后兼容性（旧数据的 project_id 为 null）

