# Model Hub 重构实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构模型下载模块，实现地理位置感知数据源路由（国内用 ollama.ac.cn）、修复全量搜索重复触发 Bug、修复搜索结果下载失败、并提供按规格大小下载的入口。

**Architecture:**
- **后端**：在 `geo.rs` 增加地区判定辅助函数，在 `manager.rs` 新增基于 `ollama.ac.cn` 的搜索和全量列表方法，通过 `is_china` flag 路由到正确源。
- **前端**：修复 `ModelManager.vue` 的双重搜索触发，清理"推荐 tab 点击跳全量"的行为，给每个模型展示可选规格下载入口。
- **下载修复**：搜索结果下载统一走官方 Ollama 协议名（不加镜像前缀），让 fallback 机制仅在推荐列表中生效。

**Tech Stack:** Rust (Tauri v2), Vue 3 + TypeScript, reqwest, regex, reka-ui

---

## 文件影响矩阵

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/services/model_hub/geo.rs` | Modify | 新增 `best_hub_base_url()` 和 `best_tags_base_url()` |
| `src-tauri/src/services/model_hub/manager.rs` | Modify | `search_on_web` 支持两源解析；`sync_from_ollama` 根据地区选择全量源 |
| `src-tauri/src/commands/models.rs` | Modify | `search_hub_models` 接收 `is_china` 参数（从 GeoSensor 获取）|
| `src/stores/settings.ts` | Modify | 修复搜索去重逻辑；添加 `installModelDirect` |
| `src/components/settings/ModelManager.vue` | Modify | 删除排行榜点击跳搜索的行为；Tab 内独立搜索框；统一规格下载 UI |

---

## Task 1：geo.rs — 新增地区感知 URL 路由辅助函数

**Files:**
- Modify: `src-tauri/src/services/model_hub/geo.rs`

### 问题背景
当前 `geo.rs` 只有 `probe_mirrors()` 和 `is_likely_in_china()`，没有暴露用于搜索/列表的最优 URL。

- [ ] **Step 1: 在 `GeoSensor impl` 末尾添加两个辅助函数**

打开 `src-tauri/src/services/model_hub/geo.rs`，在 `is_likely_in_china` 函数之后、最后一个 `}` 之前，添加：

```rust
    /// 根据 mirror 探测结果，返回最适合进行模型搜索/列表的 base URL
    /// 中国用户 -> ollama.ac.cn；否则 -> ollama.com
    pub fn best_hub_base_url(mirrors: &[MirrorInfo]) -> &'static str {
        if Self::is_likely_in_china(mirrors) {
            "https://ollama.ac.cn"
        } else {
            "https://ollama.com"
        }
    }

    /// 返回 library tags 页面的 base URL（用于抓取完整规格列表）
    pub fn best_tags_base_url(mirrors: &[MirrorInfo]) -> &'static str {
        // tags 页抓取规则与 hub 搜索相同
        Self::best_hub_base_url(mirrors)
    }
```

- [ ] **Step 2: 验证编译**

```bash
cd /Users/mac/project/telepathy
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20
```

预期：无 error，最多有 warnings。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/services/model_hub/geo.rs
git commit -m "feat(geo): add best_hub_base_url() routing helpers for region-aware data sources"
```

---

## Task 2：manager.rs — 地区感知数据源路由

**Files:**
- Modify: `src-tauri/src/services/model_hub/manager.rs`

### 问题背景
- `search_on_web` 硬编码走 `ollama.com/search`，中国用户应走 `ollama.ac.cn/search`
- `fetch_tags_from_web` 硬编码走 `ollama.com/library/{name}/tags`
- `fetch_meta_stats` 硬编码走 `ollama.com/library?sort=popular/newest`
- `sync_from_ollama` 使用第三方 crate `ollama_models_info_fetcher` 抓 ollama.com 全量列表

### 改动策略
所有需要网络请求的函数改为接收 `base_url: &str` 参数；命令层（commands/models.rs）先探测 mirror，然后将 `base_url` 传入。

- [ ] **Step 1: 改造 `search_on_web`，接收 base_url**

将 `src-tauri/src/services/model_hub/manager.rs` 中 `search_on_web` 整个函数替换为：

```rust
pub async fn search_on_web(query: String, base_url: &str) -> Result<Vec<HubModel>, String> {
    let client = reqwest::Client::new();
    let search_url = format!("{}/search?q={}", base_url, urlencoding::encode(&query));
    
    let mut hub_models = Vec::new();

    if let Ok(res) = client.get(&search_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send().await {
        if let Ok(html) = res.text().await {
            // ollama.com 与 ollama.ac.cn 均使用相同的 x-test-* 属性
            let block_re = Regex::new(r#"(?s)<li x-test-model[^>]*>(.*?)</li>"#).unwrap();
            let name_re = Regex::new(r#"href="/library/([^"/\s>]+)""#).unwrap();
            let desc_re = Regex::new(r#"(?s)<p[^>]*class="[^"]*max-w-lg[^"]*"[^>]*>(.*?)</p>"#).unwrap();
            let pull_re = Regex::new(r#"x-test-pull-count[^>]*>([^<]+)</span>"#).unwrap();
            let update_re = Regex::new(r#"x-test-updated[^>]*>([^<]+)</span>"#).unwrap();
            let cap_re = Regex::new(r#"x-test-capability[^>]*>([^<]+)</span>"#).unwrap();
            let size_re = Regex::new(r#"x-test-size[^>]*>([^<]+)</span>"#).unwrap();

            for block_cap in block_re.captures_iter(&html) {
                let block = &block_cap[1];
                
                if let Some(name_cap) = name_re.captures(block) {
                    let name = name_cap[1].to_string();
                    let description_raw = desc_re.captures(block).map(|c| c[1].trim().to_string()).unwrap_or_default();
                    let description = Self::strip_html(&description_raw);
                    let pulls = pull_re.captures(block).map(|c| c[1].trim().to_string()).unwrap_or_else(|| "N/A".to_string());
                    let updated = update_re.captures(block).map(|c| c[1].trim().to_string()).unwrap_or_else(|| "N/A".to_string());
                    
                    let mut capabilities = Vec::new();
                    let mut category = ModelCategory::Chat;
                    for cap in cap_re.captures_iter(block) {
                        let tag_text = cap[1].trim().to_string();
                        let tag_lower = tag_text.to_lowercase();
                        capabilities.push(tag_text);
                        // 兼容中英文标签（ollama.ac.cn 已汉化）
                        if tag_lower.contains("vision") || tag_lower.contains("视觉") {
                            category = ModelCategory::Vision;
                        } else if tag_lower.contains("embed") || tag_lower.contains("嵌入") {
                            category = ModelCategory::Embedding;
                        }
                    }

                    let mut variants = Vec::new();
                    for cap in size_re.captures_iter(block) {
                        let tag = cap[1].trim().to_string();
                        variants.push(ModelVariant {
                            tag: tag.clone(),
                            size: 0, // 搜索页仅有标签，无实际字节数
                            params: tag,
                        });
                    }

                    // 保底：如果没有解析到任何尺寸，给 latest 占位
                    if variants.is_empty() {
                        variants.push(ModelVariant {
                            tag: "latest".to_string(),
                            size: 0,
                            params: "latest".to_string(),
                        });
                    }

                    hub_models.push(HubModel {
                        name,
                        description,
                        pulls,
                        updated,
                        variants,
                        category,
                        capabilities,
                    });
                }
            }
        }
    }

    Ok(hub_models)
}
```

- [ ] **Step 2: 改造 `fetch_tags_from_web`，接收 `base_url`**

将整个 `fetch_tags_from_web` 函数替换为：

```rust
async fn fetch_tags_from_web(name: &str, base_url: &str) -> Vec<ModelVariant> {
    let mut variants = Vec::new();
    let client = reqwest::Client::new();
    let url = format!("{}/library/{}/tags", base_url, name);

    if let Ok(res) = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send().await {
        if let Ok(html) = res.text().await {
            // ollama.com 与 ollama.ac.cn 的 tags 页结构相同
            let re = Regex::new(r#"(?s)<span[^>]*class="[^"]*font-medium[^"]*"[^>]*>([^<]+)</span>.*?<span[^>]*class="[^"]*text-neutral-500[^"]*"[^>]*>([^<]+)</span>"#).unwrap();
            for cap in re.captures_iter(&html) {
                let tag = cap[1].trim().to_string();
                let size_str = cap[2].trim().to_string();
                
                // 过滤 SHA 哈希等超长标签
                if tag.len() > 30 || tag.contains(':') {
                    continue;
                }

                variants.push(ModelVariant {
                    tag: tag.clone(),
                    size: Self::parse_size(&size_str),
                    params: tag,
                });
            }
        }
    }
    variants
}
```

- [ ] **Step 3: 改造 `fetch_meta_stats`，接收 `base_url`**

将整个 `fetch_meta_stats` 函数替换为：

```rust
async fn fetch_meta_stats(base_url: &str) -> (HashMap<String, (String, String, String)>, Vec<String>, Vec<String>) {
    let mut stats = HashMap::new();
    let mut popular_order = Vec::new();
    let mut newest_order = Vec::new();
    let client = reqwest::Client::new();
    
    let sorts = vec![
        (format!("{}/library?sort=popular", base_url), true),
        (format!("{}/library?sort=newest", base_url), false),
    ];

    for (url, is_popular) in sorts {
        if let Ok(res) = client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send().await {
            if let Ok(html) = res.text().await {
                let re = Regex::new(r#"(?s)/library/([^"/\s>]+)".*?x-test-pull-count[^>]*>([^<]+)</span>.*?x-test-updated[^>]*>([^<]+)</span>.*?x-test-size[^>]*>([^<]+)</span>"#).unwrap();
                for cap in re.captures_iter(&html) {
                    let name = cap[1].to_string();
                    let pulls = cap[2].trim().to_string();
                    let updated = cap[3].trim().to_string();
                    let size_tag = cap[4].trim().to_string();
                    
                    stats.insert(name.clone(), (pulls, updated, size_tag));
                    if is_popular {
                        if !popular_order.contains(&name) { popular_order.push(name); }
                    } else {
                        if !newest_order.contains(&name) { newest_order.push(name); }
                    }
                }
            }
        }
    }
    (stats, popular_order, newest_order)
}
```

- [ ] **Step 4: 改造 `sync_from_ollama`，接收 `base_url: String`**

将整个 `sync_from_ollama` 函数替换为：

```rust
async fn sync_from_ollama(base_url: String) -> Result<HubCache, String> {
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to build runtime: {}", e))?;

        rt.block_on(async {
            let model_names: Vec<String> = fetch_all_available_models().await
                .map_err(|e| format!("Get model list failed: {}", e))?;
            
            println!("[ModelHub] Found {} models, using base_url: {}", model_names.len(), base_url);
            
            let (stats_map, popular_ids, newest_ids) = Self::fetch_meta_stats(&base_url).await;
            println!("[ModelHub] Scraped {} popular and {} newest models.", popular_ids.len(), newest_ids.len());

            let mut hub_cache = HubCache::new();
            
            let mut sync_names = model_names;
            for name in newest_ids.iter().chain(popular_ids.iter()) {
                if !sync_names.contains(name) {
                    sync_names.push(name.clone());
                }
            }

            let sync_limit = 120;
            let deep_fetch_limit = 30;

            for (idx, name) in sync_names.into_iter().take(sync_limit).enumerate() {
                if let Ok(info) = fetch_model_info(&name).await {
                    let model_name = info.name().to_string();
                    let category = if model_name.to_lowercase().contains("embed") {
                        ModelCategory::Embedding
                    } else if model_name.to_lowercase().contains("vision") || model_name.to_lowercase().contains("llava") {
                        ModelCategory::Vision
                    } else {
                        ModelCategory::Chat
                    };

                    let mut variants: Vec<ModelVariant> = info.varients().iter().map(|v| ModelVariant {
                        tag: v.token_size().to_string(),
                        size: Self::parse_size(v.size()),
                        params: v.token_size().to_string(),
                    }).collect();

                    if idx < deep_fetch_limit || popular_ids.contains(&name) || newest_ids.contains(&name) {
                        let web_tags = Self::fetch_tags_from_web(&name, &base_url).await;
                        if !web_tags.is_empty() {
                            for wt in web_tags {
                                if !variants.iter().any(|v| v.tag == wt.tag) {
                                    variants.push(wt);
                                }
                            }
                        }
                    }

                    let (pulls, updated, auto_size_tag) = stats_map.get(&name)
                        .cloned()
                        .unwrap_or_else(|| ("N/A".to_string(), "N/A".to_string(), "".to_string()));

                    if !auto_size_tag.is_empty() && !variants.iter().any(|v| v.tag == "latest") {
                        variants.push(ModelVariant {
                            tag: "latest".to_string(),
                            size: 0,
                            params: auto_size_tag.clone(),
                        });
                    }

                    if variants.is_empty() {
                        variants.push(ModelVariant {
                            tag: "latest".to_string(),
                            size: 4 * 1024 * 1024 * 1024,
                            params: "7B (est.)".to_string(),
                        });
                    }

                    let model = HubModel {
                        name: model_name.clone(),
                        description: info.summary_content().to_string(),
                        pulls,
                        updated,
                        variants,
                        category,
                        capabilities: Vec::new(),
                    };
                    hub_cache.insert(model_name, model);
                }
            }

            if hub_cache.is_empty() {
                return Err("No models synced".to_string());
            }
            Ok(hub_cache)
        })
    }).await.map_err(|e| format!("Blocking task failed: {}", e))?
}
```

- [ ] **Step 5: 改造 `get_models` 接收 `base_url`**

将 `get_models` 函数签名和实现替换为：

```rust
pub async fn get_models(app_handle: &tauri::AppHandle, force_refresh: bool, base_url: &str) -> Result<HubCache, String> {
    let cache_path = Self::get_cache_path(app_handle);
    
    if !force_refresh && cache_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cache_path) {
            if let Ok(cache) = serde_json::from_str::<HubCache>(&content) {
                if let Ok(metadata) = std::fs::metadata(&cache_path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified.elapsed().map(|e| e.as_secs() < 24 * 3600).unwrap_or(false) {
                            return Ok(cache);
                        }
                    }
                }
            }
        }
    }

    println!("[ModelHub] Cache expired or force refresh. Syncing from {}...", base_url);
    let start = std::time::Instant::now();
    let models = Self::sync_from_ollama(base_url.to_string()).await?;
    println!("[ModelHub] Sync completed in {:?}. Fetched {} models.", start.elapsed(), models.len());
    
    if let Ok(json) = serde_json::to_string_pretty(&models) {
        let _ = std::fs::write(&cache_path, json);
    }

    Ok(models)
}
```

- [ ] **Step 6: 改造 `get_rankings`，接收 `base_url`**

将 `get_rankings` 函数替换为：

```rust
pub async fn get_rankings(hub: &HubCache, base_url: &str) -> (Vec<HubModel>, Vec<HubModel>) {
    let (_, popular_ids, newest_ids) = Self::fetch_meta_stats(base_url).await;
    
    let mut popular_list = Vec::new();
    for id in popular_ids {
        if let Some(model) = hub.get(&id) {
            popular_list.push(model.clone());
        }
    }
    
    let mut newest_list = Vec::new();
    for id in newest_ids {
        if let Some(model) = hub.get(&id) {
            newest_list.push(model.clone());
        }
    }

    // 保底：抓取失败时按下载量/时间排序
    if popular_list.is_empty() {
        let mut all: Vec<HubModel> = hub.values().cloned().collect();
        all.sort_by(|a, b| Self::parse_pull_count(&b.pulls).cmp(&Self::parse_pull_count(&a.pulls)));
        popular_list = all;
        popular_list.truncate(20);
    }

    if newest_list.is_empty() {
        newest_list = hub.values().cloned().collect();
        newest_list.sort_by(|a, b| {
            let score_a = if a.updated.contains("day") || a.updated.contains("hour")
                || a.updated.contains("天") || a.updated.contains("小时") { 1.0f32 } else { 0.0 };
            let score_b = if b.updated.contains("day") || b.updated.contains("hour")
                || b.updated.contains("天") || b.updated.contains("小时") { 1.0f32 } else { 0.0 };
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        newest_list.truncate(20);
    }

    (popular_list, newest_list)
}
```

- [ ] **Step 7: 编译验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "^error" | head -30
```

预期：无 error。

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/services/model_hub/manager.rs
git commit -m "feat(manager): add base_url param to all fetch methods for geo-aware routing"
```

---

## Task 3：commands/models.rs — 整合地区路由

**Files:**
- Modify: `src-tauri/src/commands/models.rs`

### 问题背景
commands 层需要先获取 mirrors，传 `base_url` 给各方法。`search_hub_models` 当前没有 `base_url` 来源。

- [ ] **Step 1: 更新 `get_model_hub`**

将整个 `get_model_hub` 函数替换为：

```rust
#[tauri::command]
pub async fn get_model_hub(app_handle: AppHandle, force_refresh: bool) -> Result<ModelHubResponse, AppError> {
    let hardware = Recommender::get_hardware_profile();
    
    // 先探测 mirror（有缓存则复用）
    let mirrors = GeoSensor::probe_mirrors().await;
    let is_china = GeoSensor::is_likely_in_china(&mirrors);

    // 地区感知：选择最优数据源
    let base_url = GeoSensor::best_hub_base_url(&mirrors);
    
    let hub_cache = ModelHubManager::get_models(&app_handle, force_refresh, base_url).await
        .map_err(|e| AppError::Internal(e))?;
    
    let recommendations = Recommender::get_recommendations(&hardware, &hub_cache);
    
    // 排行榜也用同一 base_url
    let (leaderboard, trending) = ModelHubManager::get_rankings(&hub_cache, base_url).await;
    
    let mut all_models: Vec<HubModel> = hub_cache.values().cloned().collect();
    all_models.sort_by(|a, b| a.name.cmp(&b.name));
    
    let cache_timestamp = ModelHubManager::get_cache_timestamp(&app_handle);

    Ok(ModelHubResponse {
        hardware,
        mirrors,
        recommendations,
        all_models,
        leaderboard,
        trending,
        is_china,
        cache_timestamp,
    })
}
```

- [ ] **Step 2: 更新 `search_hub_models`**

将整个 `search_hub_models` 函数替换为：

```rust
#[tauri::command]
pub async fn search_hub_models(query: String) -> Result<Vec<HubModel>, AppError> {
    // 搜索时也做地区感知（复用已缓存的 mirror 结果，几乎无延迟）
    let mirrors = GeoSensor::probe_mirrors().await;
    let base_url = GeoSensor::best_hub_base_url(&mirrors);
    
    ModelHubManager::search_on_web(query, base_url).await
        .map_err(|e| AppError::Internal(e))
}
```

- [ ] **Step 3: 编译验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "^error" | head -30
```

预期：无 error。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/models.rs
git commit -m "feat(commands): route model hub requests through geo-aware base_url"
```

---

## Task 4：settings.ts — 修复重复搜索 + 新增 installModelDirect

**Files:**
- Modify: `src/stores/settings.ts`

### 问题根因
1. 搜索重复触发：`isSearching && same query` 判定有漏洞，`lastSearchQuery` 未在新查询时及时更新
2. 搜索结果下载失败：使用 `installNewModel` 会加镜像前缀，镜像站可能未同步新模型

- [ ] **Step 1: 修复 `searchHubModels` 去重逻辑**

将 `searchHubModels` 函数整体替换为：

```typescript
async function searchHubModels(query: string) {
    const trimmed = query?.trim();
    if (!trimmed || trimmed.length <= 1) return;
    
    // 若 query 与上次相同（无论是否在搜索中），直接跳过
    if (trimmed === lastSearchQuery.value) {
      console.log('[Search] Same query, skip:', trimmed);
      return;
    }

    const now = Date.now();
    searchTimestamp.value = now;
    lastSearchQuery.value = trimmed;
    isSearching.value = true;
    searchResultModels.value = []; // 清空旧结果，避免闪烁
    
    try {
      const results = await searchModelsCmd(trimmed);
      // 序列校验：若此请求已过期（被更新的请求取代），忽略结果
      if (searchTimestamp.value === now) {
        searchResultModels.value = results;
      }
    } catch (e: any) {
      console.error('[Search] Error:', e);
      if (searchTimestamp.value === now) {
        searchResultModels.value = [];
      }
    } finally {
      if (searchTimestamp.value === now) {
        isSearching.value = false;
      }
    }
  }
```

- [ ] **Step 2: 在 `installNewModel` 函数之后，添加 `installModelDirect`**

```typescript
  /**
   * 直接以官方模型名安装，不使用镜像前缀。
   * 适用于搜索结果下载：模型可能太新，镜像站尚未同步。
   */
  async function installModelDirect(modelId: string) {
    installingModel.value = modelId;
    installProgress.value = null;

    const unlisten = await listen<ModelPullProgress>('model-pull-progress', (event) => {
      installProgress.value = event.payload;
    });

    const unlistenError = await listen<{ model: string; error: string }>('model-pull-error', (event) => {
      const { model, error } = event.payload;
      installingModel.value = null;
      installProgress.value = null;
      pullGlobalNotification.value = {
        message: `模型 ${model} 下载失败: ${error}`,
        type: 'error',
        timestamp: Date.now()
      };
      setTimeout(() => { pullGlobalNotification.value = null; }, 5000);
      unlisten();
      unlistenError();
      unlistenDone();
    });

    const unlistenDone = await listen<{ model: string }>('model-pull-done', async (event: any) => {
      const modelName = event.payload.model;
      installingModel.value = null;
      installProgress.value = null;
      pullGlobalNotification.value = {
        message: `模型 ${modelName} 下载并安装完成！`,
        type: 'success',
        timestamp: Date.now()
      };
      setTimeout(() => { pullGlobalNotification.value = null; }, 4000);
      unlisten();
      unlistenError();
      unlistenDone();
      await fetchInstalledModels();
      await fetchModels();
    });

    try {
      await installModelApi(modelId); // 直接传原始 modelId，不加镜像前缀
    } catch (e: any) {
      console.error('[InstallDirect] Error:', e);
      installingModel.value = null;
      installProgress.value = null;
      unlisten();
      unlistenError();
      unlistenDone();
    }
  }
```

- [ ] **Step 3: 在 `return { ... }` 中导出新增项**

在 return 语句的 `searchHubModels,` 之后添加：

```typescript
    installModelDirect,
    lastSearchQuery,
```

- [ ] **Step 4: TypeScript 验证**

```bash
cd /Users/mac/project/telepathy
pnpm tsc --noEmit 2>&1 | head -40
```

预期：无 error。

- [ ] **Step 5: Commit**

```bash
git add src/stores/settings.ts
git commit -m "fix(store): add installModelDirect(), fix search dedup to skip same query"
```

---

## Task 5：ModelManager.vue — 修复双重触发 + 搜索结果改用 installModelDirect

**Files:**
- Modify: `src/components/settings/ModelManager.vue`

### 问题背景
1. `watchDebounced` + `handleEnterSearch` 均能触发搜索
2. 推荐 tab 排行榜点击了 `searchQuery = model.name; activeTab = 'all'` 会触发 watchDebounced
3. 搜索结果下载按钮调用 `handleInstall`（加镜像前缀），应改 `installModelDirect`

- [ ] **Step 1: 移除排行榜点击跳搜索的行为**

找到 `ModelManager.vue` 约第 340 行：

```html
<!-- 旧代码 -->
<span ... @click="searchQuery = model.name; activeTab = 'all'">{{ model.name }}</span>
```

改为：
```html
<!-- 新代码：去掉跳转，只展示文字 -->
<span class="text-xs font-bold text-text-primary transition-colors">{{ model.name }}</span>
```

- [ ] **Step 2: 修复 watchDebounced，只在全量 tab 生效**

在 `<script setup>` 中，将原有的 `watchDebounced` 块替换为：

```typescript
// 全量探索 tab 的搜索防抖：仅在 'all' tab 激活时响应 searchQuery 变化
watchDebounced(
  searchQuery,
  (newQuery) => {
    if (activeTab.value !== 'all') return; // 仅全量探索 tab 触发在线检索
    const trimmed = newQuery?.trim();
    if (trimmed && trimmed.length > 1) {
      store.searchHubModels(trimmed);
    } else if (!trimmed) {
      // 清空时同步清空结果和上次查询记录
      store.searchResultModels = [];
      store.lastSearchQuery = '';
    }
  },
  { debounce: 600 }
);
```

- [ ] **Step 3: 修复 handleEnterSearch，避免与 watchDebounced 重复**

将 `handleEnterSearch` 替换为：

```typescript
const handleEnterSearch = () => {
  const trimmed = searchQuery.value?.trim();
  if (!trimmed || trimmed.length <= 1) return;
  // 回车立即触发，跳过防抖等待，但依赖 store 内部去重逻辑避免重复
  store.searchHubModels(trimmed);
};
```

- [ ] **Step 4: 添加 `handleInstallDirect` 函数**

在 `handleInstall` 函数之后添加：

```typescript
// 安装来自搜索结果的模型：直接走官方协议，避免镜像同步延迟
const handleInstallDirect = async (modelName: string) => {
  await store.installModelDirect(modelName);
};
```

- [ ] **Step 5: 搜索结果展开区的下载按钮改为 handleInstallDirect**

在搜索结果规格列表区（约第 625 行），将：

```html
@click.stop="handleInstall(`${model.name}:${variant.tag}`)"
```

改为：

```html
@click.stop="handleInstallDirect(`${model.name}:${variant.tag}`)"
```

- [ ] **Step 6: TypeScript 验证**

```bash
pnpm tsc --noEmit 2>&1 | head -40
```

预期：无 error。

- [ ] **Step 7: Commit**

```bash
git add src/components/settings/ModelManager.vue
git commit -m "fix(ui): guard watchDebounced to 'all' tab, rm ranking click-to-search, use installModelDirect for search results"
```

---

## Task 6：UI 改进 — 搜索结果规格卡片 + 大小显示

**Files:**
- Modify: `src/components/settings/ModelManager.vue`

### 目标
搜索结果展开后，每条规格显示 tag + 文件大小（若 size=0 显示"大小待定"），下载按钮独立可用。

- [ ] **Step 1: 改进搜索结果规格展开区 UI**

找到搜索结果展开区（约第 614-634 行），将整个规格列表 `<div>` 替换为：

```html
<!-- 展开显示规格详情 -->
<div v-if="expandedSearchModel === model.name" class="p-3 border-t border-dark-border/30 bg-dark-panel/30 animate-in slide-in-from-top-1 duration-200">
  <label class="text-[10px] font-bold text-text-muted uppercase tracking-wider mb-3 block">
    可选下载规格 · 按尺寸安装
  </label>
  <div class="flex flex-wrap gap-2">
    <div 
      v-for="variant in model.variants" 
      :key="variant.tag"
      class="flex items-center justify-between gap-3 px-3 py-2 bg-dark-surface border border-dark-border/50 rounded-xl hover:border-brand-500/30 transition-colors min-w-[130px]"
    >
      <div class="flex flex-col">
        <span class="text-xs font-bold text-text-primary uppercase">{{ variant.params }}</span>
        <span class="text-[10px] text-text-muted">
          {{ variant.size > 0 ? formatSize(variant.size) : '大小待定' }}
        </span>
      </div>
      <button 
        @click.stop="handleInstallDirect(`${model.name}:${variant.tag}`)"
        class="p-1.5 bg-brand-500/10 text-brand-400 border border-brand-400/30 rounded-lg hover:bg-brand-500 hover:text-white transition-all flex-shrink-0"
        :title="`下载 ${model.name}:${variant.tag}`"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="7 10 12 15 17 10"></polyline>
          <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
      </button>
    </div>
  </div>
  <!-- 无规格兜底 -->
  <div v-if="model.variants.length === 0" class="text-xs text-text-muted italic mt-2">
    未解析到规格信息，
    <button @click.stop="handleInstallDirect(model.name)" class="text-brand-400 underline hover:text-brand-300">
      下载 latest 版本
    </button>
  </div>
</div>
```

- [ ] **Step 2: 确认排行榜规格展开区保持 `handleInstall`**

排行榜（推荐 tab）中的规格展开区（约第 390 行）使用：
```html
@click.stop="handleInstall(`${model.name}:${variant.tag}`)"
```
保持不变（这些是热门模型，镜像站已同步）。

- [ ] **Step 3: TypeScript + 编译验证**

```bash
pnpm tsc --noEmit 2>&1 | head -40
```

预期：无 error。

- [ ] **Step 4: 手动验证**

启动 `pnpm tauri dev`，在设置 → 全量探索 中：
1. 搜索 "qwen"，确认只触发 **一次** 网络请求（查看控制台）
2. 展开搜索结果，验证规格列表显示正确 tag 和大小（或"大小待定"）
3. 点击下载某规格，确认下载请求用纯模型名（无镜像前缀）

- [ ] **Step 5: Final commit**

```bash
git add src/components/settings/ModelManager.vue
git commit -m "feat(ui): improve search result variant cards with size/tag display and direct download"
```

---

## 自查（Self-Review）

### Spec 覆盖检查
| 需求 | 涵盖任务 |
|------|---------|
| 探测合适的源（geo-aware） | Task 1, 2, 3 |
| ollama.ac.cn 搜索（IS China） | Task 2, 3 |
| 模型类型、描述、大小展示 | Task 2（解析 cap/size），Task 6（UI） |
| 按规格大小提供下载入口 | Task 6 |
| 修复全量搜索重复触发 | Task 4, 5 |
| 修复搜索结果下载失败 | Task 4, 5（installModelDirect） |
| 不加自定义权重，按官方顺序 | Task 2（fetch_meta_stats，官方顺序） |

### 类型一致性检查
- `ModelHubManager::get_models(app_handle, force_refresh, base_url)` — Task 2 定义，Task 3 调用 ✅
- `ModelHubManager::search_on_web(query, base_url)` — Task 2 定义，Task 3 调用 ✅
- `ModelHubManager::get_rankings(hub, base_url)` — Task 2 定义，Task 3 调用 ✅
- `store.installModelDirect(modelId)` — Task 4 定义，Task 5 调用 ✅
- `store.lastSearchQuery` — Task 4 & 5 同步使用 ✅

### 无占位符确认
所有代码块均为完整实现，无 TBD/TODO。✅
