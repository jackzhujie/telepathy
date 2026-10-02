# 模型推荐系统优化与兜底设计 实施计划 (Implementation Plan)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 优化后端模型推荐系统，保证“向量推荐”和“视觉推荐”栏目内有丰富且质量过硬的模型，并精简和极净化卡片底部的硬件匹配理由文案。

**Architecture:** 
1. 在后端 `ModelHubManager::get_models` 中引入本地内置经典模型的注入兜底机制，并对热门模型按照类别（Chat、Embedding、Vision）进行分流，保底拉取非 Chat 类模型的规格变体。
2. 简化 `recommender.rs` 中的硬件匹配说明文案，只保留用户最关心的显存/内存状态描述。

**Tech Stack:** Rust, Tauri v2, Vue 3 + TypeScript

---

### Task 1: 优化后端模型预拉取机制与内置注册表兜底

**Files:**
- Modify: `src-tauri/src/services/model_hub/manager.rs`

- [ ] **Step 1: 修改 manager.rs 头部导入**

打开 [manager.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/model_hub/manager.rs)，在第 1 行更新导入以包括 `ModelCategory` 和 `ModelVariant`：
```rust
use crate::services::model_hub::{HubCache, HubModel, ModelCategory, ModelVariant};
```

- [ ] **Step 2: 注入本地高保真注册表兜底模型与分流预拉取逻辑**

修改 [manager.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/model_hub/manager.rs) 中的 `get_models` 方法。定位到如下的原逻辑（第 82 至 94 行左右）：
```rust
        // Parallel pre-fetching of variants for top models (ranking by pulls)
        let mut model_ranking: Vec<(String, u64)> = models
            .values()
            .map(|m| (m.name.clone(), m.pulls.parse::<u64>().unwrap_or(0)))
            .collect();
        model_ranking.sort_by(|a, b| b.1.cmp(&a.1));

        let top_ids: Vec<String> = model_ranking
            .into_iter()
            .take(30)
            .map(|(id, _)| id)
            .collect();
        let fetched_count = top_ids.len();
```
将其修改为以下逻辑（注入本地兜底模型 + 按类别保底分流）：
```rust
        // 1. 注入本地高保真注册表兜底模型，确保基础模型集（如 BGE-M3 等）绝不缺席
        if let Ok(registry) = crate::services::model_registry::load_registry() {
            for rm in registry.models {
                let cat = match rm.model_type {
                    crate::services::model_registry::ModelType::Chat => ModelCategory::Chat,
                    crate::services::model_registry::ModelType::Embedding => ModelCategory::Embedding,
                    crate::services::model_registry::ModelType::Vision => ModelCategory::Vision,
                    crate::services::model_registry::ModelType::Unknown => ModelCategory::Other,
                };
                
                // 将本地 sizes 静态转换为 Hub 变体规格，免除线上网络拉取开销
                let variants = rm.sizes.iter().map(|s| ModelVariant {
                    tag: format!("{}:{}", rm.id, s.tag),
                    size: (s.file_size_gb * 1024.0 * 1024.0 * 1024.0) as u64,
                    params: s.params.clone(),
                }).collect();

                let entry = models.entry(rm.name.clone()).or_insert_with(|| HubModel {
                    name: rm.name.clone(),
                    description: rm.description.clone(),
                    pulls: "999999".to_string(), // 给予高权重，保证前排推荐
                    updated: "N/A".to_string(),
                    variants: Vec::new(),
                    category: cat,
                    capabilities: vec!["gguf".to_string()],
                });
                
                if entry.variants.is_empty() {
                    entry.variants = variants;
                }
            }
        }

        // 2. 按类别（Chat / Embedding / Vision）分流排序，确保冷门门类不被霸榜
        let mut chat_candidates = Vec::new();
        let mut embedding_candidates = Vec::new();
        let mut vision_candidates = Vec::new();

        for m in models.values() {
            // 对于已经具有 variants 静态填充的内置兜底模型，跳过网络变体拉取
            if !m.variants.is_empty() {
                continue;
            }
            let item = (m.name.clone(), m.pulls.parse::<u64>().unwrap_or(0));
            match m.category {
                ModelCategory::Chat => chat_candidates.push(item),
                ModelCategory::Embedding => embedding_candidates.push(item),
                ModelCategory::Vision => vision_candidates.push(item),
                ModelCategory::Other => {}
            }
        }

        chat_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        embedding_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        vision_candidates.sort_by(|a, b| b.1.cmp(&a.1));

        // 3. 各自分配保底数量：20 个 Chat，10 个 Embedding，10 个 Vision
        let mut top_ids = Vec::new();
        for (id, _) in chat_candidates.into_iter().take(20) {
            top_ids.push(id);
        }
        for (id, _) in embedding_candidates.into_iter().take(10) {
            top_ids.push(id);
        }
        for (id, _) in vision_candidates.into_iter().take(10) {
            top_ids.push(id);
        }
        let fetched_count = top_ids.len();
```

- [ ] **Step 3: 运行 Cargo 编译检查，验证修改无语法错误**

Run: `cargo check` (在 `/Users/mac/project/telepathy/src-tauri` 目录)
Expected: 编译通过，无任何 warning/error。

- [ ] **Step 4: 保存并提交代码**

```bash
git add src-tauri/src/services/model_hub/manager.rs
git commit -m "feat(backend): implement categorized model pre-fetching and local registry fallback injection"
```

---

### Task 2: 精简模型硬件匹配推荐理由文案

**Files:**
- Modify: `src-tauri/src/services/model_hub/recommender.rs:147-175`

- [ ] **Step 1: 修改 recommender.rs 的文案生成函数**

打开 [recommender.rs](file:///Users/mac/project/telepathy/src-tauri/src/services/model_hub/recommender.rs)，找到 `recommendation_reason` 方法（第 147 至 175 行左右）：
```rust
    fn recommendation_reason(
        profile: &HardwareProfile,
        model: &crate::services::model_hub::HubModel,
        variant: &crate::services::model_hub::ModelVariant,
    ) -> String {
        use crate::services::model_hub::ModelCategory;

        let size = variant.size;
        let category = match model.category {
            ModelCategory::Chat => "对话",
            ModelCategory::Embedding => "向量检索",
            ModelCategory::Vision => "视觉",
            ModelCategory::Other => "通用",
        };

        if size <= Self::safe_vram(profile) && profile.vram_total > 0 {
            if profile.is_apple_silicon {
                format!("适合本机统一内存的{}模型", category)
            } else {
                format!("可放入显存，{}性能优先", category)
            }
        } else if size <= Self::safe_ram(profile) {
            format!("内存余量安全，{}质量/速度均衡", category)
        } else if size <= profile.ram_total {
            format!("可运行但内存压力较高，建议关闭其他应用")
        } else {
            format!("超出推荐内存，仅建议测试用途")
        }
    }
```
将其精简并改造成不带有重复前缀的极简版：
```rust
    fn recommendation_reason(
        profile: &HardwareProfile,
        _model: &crate::services::model_hub::HubModel,
        variant: &crate::services::model_hub::ModelVariant,
    ) -> String {
        let size = variant.size;

        if size <= Self::safe_vram(profile) && profile.vram_total > 0 {
            if profile.is_apple_silicon {
                "🚀 本机统一内存最佳适配".to_string()
            } else {
                "🚀 完全放入显存，极速运行".to_string()
            }
        } else if size <= Self::safe_ram(profile) {
            "💻 放入系统内存，稳定运行".to_string()
        } else if size <= profile.ram_total {
            "⚠️ 内存压力较高，可能卡顿".to_string()
        } else {
            "❌ 超出推荐内存，不建议运行".to_string()
        }
    }
```

- [ ] **Step 2: 运行单元测试验证推荐正确性**

Run: `cargo test` (在 `/Users/mac/project/telepathy/src-tauri` 目录)
Expected: 所有 31 个单元测试成功通过。

- [ ] **Step 3: 运行前端类型检查检查有无断层**

Run: `vue-tsc --noEmit` (在 `/Users/mac/project/telepathy` 目录)
Expected: 无类型检查报错。

- [ ] **Step 4: 保存并提交代码**

```bash
git add src-tauri/src/services/model_hub/recommender.rs
git commit -m "feat(backend): simplify model hardware recommendation reason descriptions"
```
