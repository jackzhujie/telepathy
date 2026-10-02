# 多模型管理系统设计文档

> 日期：2026-04-11
> 状态：已批准

## 1. 概述

### 1.1 目标
为 Telepathy 应用引入完善的多模型管理能力，让用户能够浏览、搜索、安装、删除 Ollama 支持的模型，并根据模型类型严格匹配到对应用途（对话引擎 / 嵌入引擎），同时基于设备硬件配置提供智能推荐。

### 1.2 设计决策摘要

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 模型类型 | 严格区分（chat / embedding / vision） | 防止用户选错导致功能异常 |
| 数据来源 | 混合方案：本地注册表 + Ollama 本地 API + 搜索安装 | 兼顾稳定性与开放性 |
| UI 位置 | 设置页内嵌 | 减少导航复杂度 |
| 推荐策略 | 基于设备内存和 GPU 自动标注 | 降低新用户选择门槛 |

## 2. 数据架构

### 2.1 本地模型注册表

文件位置：`src-tauri/resources/model_registry.json`

内置于应用资源中，包含主流模型的元数据。后续可从远端 URL 热更新。

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
      "id": "llama3.2",
      "name": "Llama 3.2",
      "provider": "Meta",
      "type": "chat",
      "description": "Meta 最新开源模型，英文能力强",
      "sizes": [
        { "tag": "1b", "params": "1B", "min_memory_gb": 4, "file_size_gb": 0.7 },
        { "tag": "3b", "params": "3B", "min_memory_gb": 4, "file_size_gb": 2.0 },
        { "tag": "8b", "params": "8B", "min_memory_gb": 8, "file_size_gb": 4.7 }
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
        { "tag": "14b", "params": "14B", "min_memory_gb": 16, "file_size_gb": 9.0 }
      ]
    },
    {
      "id": "gemma2",
      "name": "Gemma 2",
      "provider": "Google",
      "type": "chat",
      "description": "Google 轻量模型，性能优秀",
      "sizes": [
        { "tag": "2b", "params": "2B", "min_memory_gb": 4, "file_size_gb": 1.6 },
        { "tag": "9b", "params": "9B", "min_memory_gb": 8, "file_size_gb": 5.4 },
        { "tag": "27b", "params": "27B", "min_memory_gb": 24, "file_size_gb": 16.0 }
      ]
    },
    {
      "id": "phi3",
      "name": "Phi-3",
      "provider": "Microsoft",
      "type": "chat",
      "description": "微软小参数高智能模型",
      "sizes": [
        { "tag": "3.8b", "params": "3.8B", "min_memory_gb": 4, "file_size_gb": 2.3 },
        { "tag": "14b", "params": "14B", "min_memory_gb": 16, "file_size_gb": 7.9 }
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

### 2.2 已安装模型

通过 Ollama 本地 API `GET /api/tags` 实时获取。返回数据包含模型名称、大小、修改时间等。

与注册表交叉匹配的规则：
- 将 Ollama 返回的模型名（如 `qwen2.5:7b`）拆分为 `base_name:tag`
- 在注册表中查找 `id == base_name` 的条目
- 匹配到则继承注册表中的 `type`；未匹配到则标注为 `unknown` 类型

### 2.3 设置项（不变）

沿用现有 settings 表的键值对存储：
- `chat_model`：当前对话引擎的完整模型标识（如 `qwen2.5:7b`）
- `embedding_model`：当前嵌入引擎的完整模型标识（如 `bge-m3`）

## 3. 后端设计

### 3.1 新增 Rust 模块 `src-tauri/src/services/model_registry.rs`

职责：
- 加载并解析 `model_registry.json`
- 提供按类型过滤、按关键词搜索的查询方法
- 与 Ollama `/api/tags` 数据做交叉匹配

关键类型定义：

```rust
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Chat,
    Embedding,
    Vision,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSize {
    pub tag: String,
    pub params: String,
    pub min_memory_gb: u32,
    pub file_size_gb: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct InstalledModel {
    pub full_name: String,       // e.g. "qwen2.5:7b"
    pub base_name: String,       // e.g. "qwen2.5"
    pub tag: String,             // e.g. "7b"
    pub model_type: ModelType,   // 从注册表匹配
    pub size_bytes: u64,
    pub display_name: String,    // 从注册表获取，或回退到 base_name
    pub provider: String,        // 从注册表获取，或 "Unknown"
}

#[derive(Debug, Serialize, Clone)]
pub enum Recommendation {
    Recommended,   // 内存充足
    Marginal,      // 勉强可以
    NotRecommended // 内存不足
}
```

### 3.2 新增 Tauri 命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `list_registry_models` | `model_type?: ModelType` | `Vec<RegistryModel>` | 从注册表读取，可按类型过滤 |
| `list_installed_models` | 无 | `Vec<InstalledModel>` | 调用 Ollama API 并与注册表匹配 |
| `get_model_recommendations` | 无 | `Vec<ModelRecommendation>` | 基于硬件推荐模型+规格 |
| `install_model` | `model_id: String` | `()` | 调用 `ollama pull`，复用现有下载进度事件 |
| `delete_model` | `model_name: String` | `()` | 调用 Ollama `DELETE /api/delete` |

### 3.3 现有命令改动

- `listOllamaModels`：保持兼容，但内部实现切换为调用 `list_installed_models`
- 现有的 `pullModel` 逻辑复用，但进度事件标准化

## 4. 前端设计

### 4.1 设置页 - 模型管理板块

在现有设置页中新增"模型管理"区域，分为三个子区域：

#### 区域 A：引擎选择（顶部）
- 两个下拉选择器，水平排列
- **对话引擎**：只显示已安装的 `type=chat` 模型
- **嵌入引擎**：只显示已安装的 `type=embedding` 模型
- 选中模型旁显示参数量和内存需求标签
- 修改后立即写入 settings

#### 区域 B：已安装模型（中部）
- 卡片网格布局（2~3列）
- 每张卡片内容：
  - 模型名称 + 提供商
  - 类型标签（彩色 badge：chat=蓝色，embedding=绿色，vision=紫色，unknown=灰色）
  - 参数量 + 磁盘占用
  - 是否为当前选中的引擎（高亮标识）
  - 删除按钮（当前选中的引擎不可删除）
- 空状态提示："还没有安装任何模型，请在下方搜索并安装"

#### 区域 C：模型搜索与安装（底部）
- 搜索框（支持关键词搜索 + 直接输入完整模型名如 `deepseek-r1:7b`）
- 类型筛选标签：全部 / 对话 / 嵌入
- 搜索结果列表：
  - 模型名称 + 提供商 + 描述
  - 可选规格列表（每个规格一个安装按钮）
  - 每个规格旁显示推荐等级图标：✅ / ⚠️ / ❌
  - 已安装的规格显示"已安装"标签，不可重复安装
- 下载进度：安装中的模型显示进度条和百分比

### 4.2 新增/修改组件

| 组件 | 位置 | 说明 |
|------|------|------|
| `ModelManager.vue` | `src/components/settings/` | 模型管理板块容器 |
| `EngineSelector.vue` | `src/components/settings/` | 引擎选择下拉框 |
| `InstalledModelCard.vue` | `src/components/settings/` | 已安装模型卡片 |
| `ModelSearchPanel.vue` | `src/components/settings/` | 搜索与安装面板 |
| `ModelSizeOption.vue` | `src/components/settings/` | 单个规格的安装按钮 + 推荐标识 |

### 4.3 状态管理

在现有 `useSettingsStore` 中扩展：

```typescript
// 新增 state
const registryModels = ref<RegistryModel[]>([]);
const installedModels = ref<InstalledModel[]>([]);

// 新增 computed
const chatModels = computed(() =>
  installedModels.value.filter(m => m.model_type === 'chat')
);
const embeddingModels = computed(() =>
  installedModels.value.filter(m => m.model_type === 'embedding')
);

// 新增 actions
async function fetchRegistryModels(type?: string) { ... }
async function fetchInstalledModels() { ... }
async function installModel(modelId: string) { ... }
async function deleteModel(modelName: string) { ... }
async function setEngine(role: 'chat' | 'embedding', modelName: string) { ... }
```

## 5. 智能推荐逻辑

### 5.1 推荐算法

```
输入：系统可用内存 (available_gb), 模型最低内存需求 (min_memory_gb)

if available_gb >= min_memory_gb * 1.5:
    → Recommended (✅ 推荐)
elif available_gb >= min_memory_gb:
    → Marginal (⚠️ 勉强可以)
else:
    → NotRecommended (❌ 内存不足)
```

### 5.2 展示规则

- 模型搜索结果按推荐等级排序：✅ 在前，❌ 在后
- ❌ 标注的规格仍然可以点击安装（不阻止），但显示警告提示

## 6. 错误处理

| 场景 | 处理方式 |
|------|----------|
| Ollama 未启动 | 显示"Ollama 未运行"提示，禁用安装/删除按钮 |
| 模型注册表加载失败 | 回退到空列表，搜索功能降级为仅支持直接输入模型名 |
| 安装中断/失败 | 进度条变红，显示错误信息，提供"重试"按钮 |
| 删除失败 | Toast 提示错误原因 |
| 已安装模型与注册表不匹配 | 标注为 unknown 类型，不出现在引擎选择下拉框中 |

## 7. 不做的事情（YAGNI）

- **不做**远程注册表自动更新（第一期手动维护 JSON）
- **不做**模型版本管理（Ollama 自身管控）
- **不做**多模型并发加载
- **不做**聊天页面内的模型切换（本期只在设置页操作）
- **不做**视觉模型的集成使用（仅支持类型标注，不接入功能）

## 8. 文件变更清单

### 新增文件
- `src-tauri/resources/model_registry.json` — 模型注册表数据
- `src-tauri/src/services/model_registry.rs` — 注册表服务
- `src-tauri/src/commands/models.rs` — 模型管理命令
- `src/components/settings/ModelManager.vue` — 管理板块容器
- `src/components/settings/EngineSelector.vue` — 引擎选择
- `src/components/settings/InstalledModelCard.vue` — 已安装模型卡片
- `src/components/settings/ModelSearchPanel.vue` — 搜索安装面板
- `src/components/settings/ModelSizeOption.vue` — 规格安装按钮

### 修改文件
- `src-tauri/src/lib.rs` — 注册新命令
- `src-tauri/src/services/mod.rs` — 注册新模块
- `src/stores/settings.ts` — 扩展模型管理状态
- `src/api/tauri.ts` — 新增 API 调用函数
- 设置页视图文件 — 嵌入 `ModelManager` 组件
