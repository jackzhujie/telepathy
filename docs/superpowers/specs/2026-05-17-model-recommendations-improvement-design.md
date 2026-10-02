# 设计规格书：模型推荐系统优化与兜底设计

本设计规格书旨在优化 Telepathy 模型管理页面中“对话、向量、视觉”推荐列表的展示质量，通过精细化分类预拉取、本地精品注册表模型注入兜底，以及极简硬件推荐理由的设计，提升用户查找和选用模型的体验。

## 1. 业务痛点与优化目标

### 现有痛点
1. **向量与视觉模型匮乏**：因为目前系统仅拉取线上排名前 50 个热门（Trending）模型，并只对其中前 30 名热门模型拉取可选规格（Variants）。由于 Chat 模型的 Pulls 数（下载量）在各大 Hub 占绝对优势，前 30 名基本被 Chat模型垄断。这导致向量（Embedding）和视觉（Vision）模型由于缺乏规格详情而无法在推荐中展现。
2. **推荐状态文案冗余**：重构为 3 列 Grid 布局后，栏目名称（如“向量推荐”）已包含分类属性。卡片下方的 `"可放入显存，对话性能优先"` / `"可放入显存，向量检索性能优先"` 等表述含有极大的词语冗余，且未最大化突出真正的硬件匹配指标（显存/内存）。

### 优化目标
* 🟢 **保障推荐丰富度**：实现 Chat、Embedding、Vision 三大门类分别保底获取规格，在任何硬件和网络状态下，三列卡片均呈现饱满而高智能的精品模型推荐。
* 🟢 **硬件友好极简文案**：精简推荐卡片状态标签，突出用户设备上模型装载至“统一内存”、“独立显存”或“系统内存”的精准硬件运行级别，降低用户认知负担。

---

## 2. 详细方案设计

### 2.1 后端：按类别保底预拉取机制 (Categorized Pre-fetching)
在 [manager.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/model_hub/manager.rs) 的 `get_models` 接口中，重构选取前 30 名模型的逻辑：
1. 遍历通过线上 Hub 拉取到的 `trending` 候选列表，将其按 [ModelCategory](file:///Users/mac/project/telepathy/src-tauri/src/services/model_hub/mod.rs) 划分为四个子向量（Chat, Embedding, Vision, Other）。
2. 将这四个分类向量分别按 `pulls`（下载热度）降序排序。
3. 从各类中选取特定数量的尖子生：
   * **Chat** 选取最热门的前 **20 个**
   * **Embedding** 选取最热门的前 **10 个**
   * **Vision** 选取最热门的前 **10 个**
4. 将合并去重后的候选 IDs 作为并行 `get_variants` 规格拉取的名单，保证所有维度的模型都有完整的规格可用。

### 2.2 后端：内置高保真模型兜底注入 (Local Registry Backup Injection)
为了应对弱网、离线或外部 Hub（如 HuggingFace、ModelScope）上无 GGUF 格式向量模型的问题，在拉取和排序热门模型后加入**高保真内置注册表注入机制**：
1. 调用本地已定义优秀模型的 `load_registry()` 接口。
2. 循环遍历本地已定义的 [RegistryModel](file:///Users/mac/project/telepathy/src-tauri/src/services/model_registry.rs)。
3. 若内存 HubCache 中尚不包含对应的 `rm.name`，将其**转换并强行注入 `models` 缓存哈希表**。
4. 强行转换时，直接将本地 `sizes` 换算为字节的 `variants` 详情，不再触发额外的网络 variant 请求。
5. **推荐特权**：为保证精品模型显示，注入时将内置模型的 `pulls` 统一模拟设为 `999999` 保证它们拥有高分优先级。

### 2.3 后端：极简硬件适配文案 (Simplified Fit Reason)
修改 [recommender.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/model_hub/recommender.rs) 中 `recommendation_reason` 生成的文案模版：
* 🟢 **统一内存强适配 (Apple Silicon)**：`"🚀 统一内存最佳适配，极速运行"`
* 🟢 **完全装入显存 (NVIDIA/AMD)**：`"🚀 完全放入显存，极速运行"`
* 🟢 **安全装入系统内存**：`"💻 放入系统内存，稳定运行"`
* 🟢 **勉强装载 (内存吃紧)**：`"⚠️ 内存压力较高，可能卡顿"`
* 🟢 **超出限制**：`"❌ 超出推荐内存，不建议运行"`

---

## 3. 验证与交付标准

1. **编译运行验证**：
   * 前端使用 `vue-tsc --noEmit` 验证无类型隐患。
   * 后端通过 `cargo test` 验证 `model_registry` 以及其他 Rust 模块编译运行正确。
2. **UI 测试**：
   * 启动 Tauri 应用，进入 ModelManager 模型管理页面。
   * 确认“对话推荐”、“向量推荐”、“视觉推荐”各列均有丰富的推荐卡片（通常每列 2-3 个精品模型）。
   * 确认推荐理由简短且直接指出硬件适配状态，不存在“对话性能优先”等重复词汇。
