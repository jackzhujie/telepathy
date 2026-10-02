
# 基于项目的知识库重构设计

## 概述

将文档管理和知识库重构为基于"项目"的结构，支持按项目组织文档和知识库。

## 需求

### 核心概念
- **项目**：包含名称和描述，用于组织文档
- **文档**：必须属于某个项目
- **知识库**：基于项目展示，支持选择/多选项目

### 用户故事
1. 用户可以创建、编辑、删除项目
2. 用户在导入文档前必须选择项目
3. 用户可以在文档管理页面顶部管理项目
4. 知识库页面可以选择/多选项目来浏览
5. AI 对话默认使用所有项目的知识库，也可以选择项目

## 数据模型

### 新增表：projects
```sql
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 修改表：documents
```sql
ALTER TABLE documents ADD COLUMN project_id TEXT;
```

## 架构设计

### 后端

#### 新增模块：`src-tauri/src/db/projects.rs`
- `init_projects_table()` - 初始化表
- `insert_project()` - 创建项目
- `get_all_projects()` - 获取所有项目
- `update_project()` - 更新项目
- `delete_project()` - 删除项目

#### 修改模块：`src-tauri/src/db/documents.rs`
- 所有查询新增 `project_id` 过滤

#### 新增命令：`src-tauri/src/commands/projects.rs`
- `create_project()` - 创建项目
- `list_projects()` - 获取项目列表
- `update_project()` - 更新项目
- `delete_project()` - 删除项目

#### 修改命令：其他相关命令
- 导入文档时关联 `project_id`
- 查询文档时按 `project_id` 过滤

### 前端

#### 新增类型：`src/types/project.ts`
```typescript
export interface Project {
  id: string;
  name: string;
  description?: string;
  created_at: string;
}
```

#### 新增 Store：`src/stores/projects.ts`
- 项目列表管理
- 当前选中项目

#### 修改页面：`src/views/Documents.vue`
- 顶部添加项目选择器和项目管理按钮
- 文档列表仅显示当前项目的文档
- 导入文档时关联当前选中项目

#### 修改页面：`src/views/KnowledgeBase.vue`
- 项目选择器（支持多选）
- 内容仅显示选中项目的知识库

#### 修改页面：`src/views/Chat.vue`
- 添加知识库项目选择（默认全选）
- RAG 查询时过滤选中项目的向量

## 实现步骤

1. 后端数据库层（projects.rs + 修改 documents.rs）
2. 后端命令层（projects.rs + 修改相关命令）
3. 前端类型和 API
4. 前端 Store
5. 修改 Documents.vue
6. 修改 KnowledgeBase.vue
7. 修改 Chat.vue
8. 测试和验证

