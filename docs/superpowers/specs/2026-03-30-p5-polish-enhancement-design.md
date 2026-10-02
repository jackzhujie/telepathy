# P5: 打磨与增强 - 设计文档

## 1. 项目背景 (Project Context)
Telepathy 的 P0-P4 阶段已完成核心功能（文档导入、解析、分块、向量化、RAG 问答）。P5 是最终打磨阶段，目标是让应用从"可用"变为"好用"。

## 2. 核心目标 (Core Goals)
- **设置页面**: 提供用户可配置的设置界面，支持 Ollama 地址、模型选择、检索参数调整。
- **错误处理优化**: 更友好的错误提示、加载状态、空状态引导。
- **UI 细节打磨**: Markdown 渲染、代码高亮、主题一致性。

## 3. 架构设计 (Architecture)

### 3.1 设置页面 (Settings Page)

#### 配置存储
- 使用 SQLite `settings` 表存储配置（key-value 模式）。
- 应用启动时从数据库加载配置到内存。
- 修改配置时同步更新数据库。

#### 配置项
| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `ollama_url` | String | `http://localhost:11434` | Ollama 服务地址 |
| `chat_model` | String | `qwen2.5` | 聊天模型名称 |
| `embedding_model` | String | `bge-large-zh` | 嵌入模型名称 |
| `top_k` | Integer | `5` | 检索返回的文档块数量 |
| `similarity_threshold` | Float | `0.3` | 相似度过滤阈值 |

#### 后端接口
- `get_settings() -> Result<HashMap<String, String>, AppError>`: 获取所有配置
- `update_setting(key: String, value: String) -> Result<(), AppError>`: 更新单个配置
- `list_ollama_models() -> Result<Vec<String>, AppError>`: 获取 Ollama 已安装模型列表

#### 前端界面
- 在 Settings.vue 中展示配置表单。
- 使用 `n-form` 组件，包含输入框、下拉选择、滑块等。
- 聊天模型和嵌入模型通过下拉选择，选项来自 Ollama API。
- Top-K 和相似度阈值使用 `n-slider` 调整。

### 3.2 错误处理优化
- 所有 Tauri 命令统一返回 `Result<T, AppError>`。
- 前端使用 `useMessage()` 展示错误提示。
- 空状态页面：文档管理为空时显示引导提示。
- Ollama 连接失败时显示明确的故障排除指引。

### 3.3 UI 打磨
- 引入 `markdown-it` 渲染 AI 回答中的 Markdown 内容。
- 引入 `highlight.js` 实现代码块语法高亮。
- 优化对话气泡样式，支持 Markdown 渲染后的内容。

## 4. 文件结构

### 后端新增
- `src-tauri/src/db/settings.rs` — settings 表 CRUD
- `src-tauri/src/commands/settings.rs` — get_settings, update_setting, list_ollama_models

### 前端修改
- `src/views/Settings.vue` — 完整设置页面
- `src/stores/settings.ts` — Pinia settings store
- `src/views/Chat.vue` — Markdown 渲染 + 代码高亮
- `src/views/Documents.vue` — 空状态引导

## 5. 验收标准 (Success Criteria)
- [ ] 设置页面可配置所有 5 个参数
- [ ] 修改配置后即时生效（无需重启）
- [ ] Ollama 模型列表动态获取
- [ ] 错误提示清晰友好
- [ ] AI 回答支持 Markdown 渲染和代码高亮
- [ ] 空状态页面显示引导信息
