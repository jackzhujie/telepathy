# 2026-04-14 智能模型枢纽 UI 与数据修复设计

## 问题背景
当前 Model Hub UI 显示为空，原因如下：
1. **数据缺失**：抓取器抓取的 `variants`（变体/标签）列表为空。
2. **推荐过滤过严**：推荐引擎检测到变体为空或显存为 0 时，过滤掉了所有模型。
3. **API 结构受限**：`get_model_hub` 命令仅返回推荐列表，缺失全量模型数据字段。

## 设计方案

### 1. 后端数据补全 (Manager)
- **文件**: `src/services/model_hub/manager.rs`
- **逻辑**: 
    - 抓取时检查 `info.varients()` 长度。
    - 若为空，手动添加 `tag: "latest"`, `size: 4GB`, `params: "7B (est.)"` 作为保底项。
    - 确保 `hub_cache` 至少包含基础变体信息，避免下游过滤。

### 2. API 扩容 (Commands)
- **文件**: `src/commands/models.rs`
- **逻辑**:
    - 修改 `ModelHubResponse` 结构体，增加 `all_models: Vec<HubModel>` 字段。
    - 在返回给前端时，将 `hub_cache` 中的原始模型列表一并返回，支撑“全量探索”Tab 页。

### 3. 系统兼容性增强 (Recommender)
- **文件**: `src/services/model_hub/recommender.rs`
- **逻辑**:
    - 针对非 M 系列芯片的 Mac (x86_64)，在显存识别为 0 时，根据系统物理内存（RAM）启用保底推荐逻辑。
    - 调整评分公式，确保显存为 0 时，模型大小不再作为致命的降分项。

## 验证计划
- 检查 `model_hub_cache.json` 是否包含非空的 `variants` 数组。
- 观察 UI “为您推荐”是否出现分值 > 0 的模型项目。
- 点击“全量探索”Tab，验证是否能看到完整的 20 个热门模型。
