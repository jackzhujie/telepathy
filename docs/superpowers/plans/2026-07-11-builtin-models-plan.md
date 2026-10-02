# 内置模型自适应推荐与首次运行引导向导实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现应用启动时基于硬件探测的黄金首选模型推荐逻辑，提供一键引导安装的 Setup Wizard，并引入 `.part` 临时后缀以确保大文件下载的异常容错。

**Architecture:** 硬件探测与自适应推荐算法位于 Rust 后端服务层，推荐结果作为 API 返回至前端；Vue 端的 Pinia Store 处理一键并发/串行下载控制及持久化设置；页面和新装向导读取 Store 完成渲染。

**Tech Stack:** Rust (Tauri v2, rusqlite), Vue 3.5, Pinia, Tailwind CSS, reka-ui.

---

### Task 1: Rust 后端 - 定义 PrimaryRecommendations 并扩展 ModelHubResponse

**Files:**
- Modify: `src-tauri/src/services/model_hub/mod.rs`
- Modify: `src-tauri/src/commands/models.rs`

- [ ] **Step 1: 在 `mod.rs` 中定义 `PrimaryRecommendations` 结构体**
  
  在 `src-tauri/src/services/model_hub/mod.rs` 约 59 行末尾添加：
  ```rust
  #[derive(Debug, Serialize, Deserialize, Clone)]
  pub struct PrimaryRecommendations {
      pub chat: Option<HubRecommendation>,
      pub embedding: Option<HubRecommendation>,
      pub vision: Option<HubRecommendation>,
  }
  ```

- [ ] **Step 2: 并在 `mod.rs` 顶部的导入列表导出该结构体**
  
  确保 `pub use` 包含了它：
  ```rust
  pub use mod_name::{... , PrimaryRecommendations};
  ```

- [ ] **Step 3: 修改 `models.rs` 中的 `ModelHubResponse` 返回体**
  
  在 `src-tauri/src/commands/models.rs:16-24` 中增加 `primary_recommendations` 字段：
  ```rust
  #[derive(Debug, Serialize)]
  pub struct ModelHubResponse {
      pub hardware: HardwareProfile,
      pub recommendations: Vec<HubRecommendation>,
      pub primary_recommendations: PrimaryRecommendations,
      pub all_models: Vec<HubModel>,
      pub leaderboard: Vec<HubModel>,
      pub trending: Vec<HubModel>,
      pub is_china: bool,
      pub cache_timestamp: u64,
  }
  ```

- [ ] **Step 4: 编译检查**
  
  运行：`cargo check` inside `src-tauri`
  Expected: FAIL (因为 `get_model_hub` 中尚未构造新字段)

- [ ] **Step 5: 临时在 `get_model_hub` 中填入 Mock 返回并提交**
  
  在 `src-tauri/src/commands/models.rs:91` 修改为：
  ```rust
      Ok(ModelHubResponse {
          hardware,
          recommendations,
          primary_recommendations: crate::services::model_hub::PrimaryRecommendations {
              chat: None,
              embedding: None,
              vision: None,
          },
          all_models,
          leaderboard,
          trending,
          is_china,
          cache_timestamp,
      })
  ```
  运行：`cargo check` inside `src-tauri`
  Expected: PASS
  
  ```bash
  git add src-tauri/src/services/model_hub/mod.rs src-tauri/src/commands/models.rs
  git commit -m "chore: define PrimaryRecommendations and extend ModelHubResponse in backend"
  ```

---

### Task 2: Rust 后端 - 实现自适应推荐算法

**Files:**
- Modify: `src-tauri/src/services/model_hub/recommender.rs`
- Modify: `src-tauri/src/commands/models.rs`

- [ ] **Step 1: 在 `recommender.rs` 中编写测试用例**
  
  在 `src-tauri/src/services/model_hub/recommender.rs` 最底部添加：
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::services::model_hub::HardwareProfile;
      use std::collections::HashMap;

      #[test]
      fn test_compute_primary_recommendations_low_end() {
          let profile = HardwareProfile {
              vram_total: 1024 * 1024 * 1024,
              vram_used: 0,
              ram_total: 8 * 1024 * 1024 * 1024, // 8GB RAM
              ram_used: 0,
              disk_free: 50 * 1024 * 1024 * 1024,
              is_apple_silicon: true,
              os: "macos".to_string(),
          };
          let hub = HashMap::new();
          let recs = Recommender::compute_primary_recommendations(&profile, &hub);
          assert!(recs.vision.is_none()); // 8G 设备不推荐视觉模型
      }
  }
  ```

- [ ] **Step 2: 运行测试以确保失败**
  
  运行：`cargo test --lib services::model_hub::recommender`
  Expected: FAIL with "no method named `compute_primary_recommendations`"

- [ ] **Step 3: 在 `recommender.rs` 中实现 `compute_primary_recommendations` 方法**
  
  在 `src-tauri/src/services/model_hub/recommender.rs` 约 42 行之后添加：
  ```rust
      pub fn compute_primary_recommendations(
          profile: &HardwareProfile,
          hub: &HubCache,
      ) -> super::PrimaryRecommendations {
          let ram_gb = profile.ram_total as f64 / 1_073_741_824.0;
          
          let chat_id = "qwen2.5";
          let chat_tag = if ram_gb <= 9.0 {
              "1.5b"
          } else if ram_gb <= 18.0 {
              "7b"
          } else {
              "14b"
          };
          let chat = Self::find_recommendation_by_id_tag(hub, profile, chat_id, chat_tag);

          let embedding = Self::find_recommendation_by_id_tag(hub, profile, "bge-large-zh", "latest")
              .or_else(|| Self::find_recommendation_by_id_tag(hub, profile, "bge-m3", "latest"));

          let vision = if ram_gb > 8.0 {
              Self::find_recommendation_by_id_tag(hub, profile, "qwen2.5-vl", "7b")
          } else {
              None
          };

          super::PrimaryRecommendations { chat, embedding, vision }
      }

      fn find_recommendation_by_id_tag(
          hub: &HubCache,
          profile: &HardwareProfile,
          model_id: &str,
          tag: &str,
      ) -> Option<super::HubRecommendation> {
          let model = hub.get(model_id)?;
          let variant = model.variants.iter().find(|v| v.tag.eq_ignore_ascii_case(tag))?;
          let (score, reason) = Self::calculate_score(profile, model, variant);
          Some(super::HubRecommendation {
              model: model.clone(),
              variant: variant.clone(),
              score,
              reason,
          })
      }
  ```

- [ ] **Step 4: 重新运行单元测试并使其通过**
  
  运行：`cargo test --lib services::model_hub::recommender`
  Expected: PASS

- [ ] **Step 5: 在 `models.rs` 中替换 mock 代码并提交**
  
  在 `src-tauri/src/commands/models.rs:91` 中：
  ```rust
          primary_recommendations: Recommender::compute_primary_recommendations(&hardware, &hub_cache),
  ```
  运行：`cargo check` inside `src-tauri`
  Expected: PASS
  
  ```bash
  git add src-tauri/src/services/model_hub/recommender.rs src-tauri/src/commands/models.rs
  git commit -m "feat: implement primary recommendations adaptive engine in Rust backend"
  ```

---

### Task 3: Rust 后端 - 实现 `.part` 临时后缀重命名机制

**Files:**
- Modify: `src-tauri/src/commands/models.rs`

- [ ] **Step 1: 修改 `install_model` 命令实现，应用临时后缀**
  
  在 `src-tauri/src/commands/models.rs:331` 之后修改为：
  ```rust
      let dest_filename = format!("{}-{}.gguf", model_name, actual_variant);
      let final_path = app_data_dir.join("downloads").join(&dest_filename);
      let temp_path = app_data_dir.join("downloads").join(format!("{}.part", dest_filename));

      let registration_id = format!("{}:{}", model_name, actual_variant);
      let downloader = ModelDownloader::new();
      let download_res = downloader
          .download(
              &download_url,
              &temp_path, // 使用临时路径
              &registration_id,
              expected_size,
              &app_handle,
          )
          .await;

      if let Err(e) = download_res {
          let err_msg = e.to_string();
          // 如果失败，清理临时文件
          let _ = tokio::fs::remove_file(&temp_path).await;
          let _ = app_handle.emit(
              "model-pull-error",
              json!({ "model": model_tag, "error": err_msg }),
          );
          // ... 遗留的通知逻辑保持一致 ...
          return Err(e);
      }

      // 下载成功，执行重命名
      tokio::fs::rename(&temp_path, &final_path)
          .await
          .map_err(|e| AppError::Internal(format!("Failed to rename model file: {}", e)))?;

      let _ = app_handle.emit("model-pull-done", json!({ "model": model_tag }));
      // ... 遗留的成功通知逻辑保持一致 ...
  ```

- [ ] **Step 2: 验证编译**
  
  运行：`cargo check` inside `src-tauri`
  Expected: PASS

- [ ] **Step 3: 提交修改**
  
  ```bash
  git add src-tauri/src/commands/models.rs
  git commit -m "fix: enforce .part extension during download to protect integrity"
  ```

---

### Task 4: Vue 前端 - Store 状态与 Action 扩充

**Files:**
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: 在 Store 中加入 `wizardCompleted` 状态**
  
  在 `src/stores/settings.ts` 约 307 行后添加：
  ```typescript
    const wizardCompleted = computed({
      get: () => settings.value.wizard_completed === 'true',
      set: (v: boolean) => saveSetting('wizard_completed', String(v))
    });

    const showSetupWizard = computed(() => {
      return !wizardCompleted.value && installedModels.value.length === 0;
    });
  ```

- [ ] **Step 2: 增加一键安装/跳过 Action 并在 `return` 中暴露**
  
  在 `src/stores/settings.ts` 约 235 行 `downloadPrimaryPresets` 后添加：
  ```typescript
    async function downloadPrimaryPresets() {
      const recs = modelHubData.value?.primary_recommendations;
      if (!recs) return;

      if (recs.chat) {
        installNewModel(recs.chat.model.id, recs.chat.variant.tag).catch(console.error);
      }
      if (recs.embedding) {
        installNewModel(recs.embedding.model.id, recs.embedding.variant.tag).catch(console.error);
      }
      wizardCompleted.value = true;
    }

    function skipSetupWizard() {
      wizardCompleted.value = true;
    }
  ```
  并在 `return` 中返回：`wizardCompleted, showSetupWizard, downloadPrimaryPresets, skipSetupWizard`

- [ ] **Step 3: 前端类型与静态检查**
  
  运行：`pnpm vue-tsc --noEmit`
  Expected: PASS

- [ ] **Step 4: 提交变更**
  
  ```bash
  git add src/stores/settings.ts
  git commit -m "feat: expand settings store for wizard status and bulk downloader action"
  ```

---

### Task 5: Vue 前端 - 编写 SetupWizard.vue 引导向导

**Files:**
- Create: `src/components/settings/SetupWizard.vue`

- [ ] **Step 1: 新建引导向导界面组件**
  
  在 `src/components/settings/SetupWizard.vue` 写入组件代码，呈现高级感毛玻璃半透明卡片样式：
  - 显示设备基本信息（RAM、是否是 Apple M1/M2/Intel 等）
  - 显示自适应计算出的 Chat / Embedding 模型明细及大小
  - 提供 `一键配置并安装`（绑定 `downloadPrimaryPresets`）与 `跳过引导`（绑定 `skipSetupWizard`）

- [ ] **Step 2: 运行类型校验**
  
  运行：`pnpm vue-tsc --noEmit`
  Expected: PASS

- [ ] **Step 3: 提交变更**
  
  ```bash
  git add src/components/settings/SetupWizard.vue
  git commit -m "feat: implement SetupWizard.vue wizard interface"
  ```

---

### Task 6: Vue 前端 - 在 App.vue 中挂载 SetupWizard

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 引入并挂载 `SetupWizard` 组件**
  
  修改 `src/App.vue`，在模板中当 `settingsStore.showSetupWizard` 为 `true` 时挂载弹窗：
  ```vue
  <script setup lang="ts">
  // ... 导入不变 ...
  import SetupWizard from '@/components/settings/SetupWizard.vue';
  </script>
  <template>
    <TooltipProvider :delay-duration="300">
      <div class="h-screen relative">
        <AppLayout />
        <SetupWizard v-if="settingsStore.showSetupWizard" />
      </div>
    </TooltipProvider>
  </template>
  ```

- [ ] **Step 2: 验证编译**
  
  运行：`pnpm vue-tsc --noEmit`
  Expected: PASS

- [ ] **Step 3: 提交变更**
  
  ```bash
  git add src/App.vue
  git commit -m "feat: mount SetupWizard overlay in App.vue"
  ```

---

### Task 7: Vue 前端 - 模型管理页首选推荐置顶卡片

**Files:**
- Modify: `src/components/settings/ModelManager.vue`

- [ ] **Step 1: 在 `推荐` Tab 顶部设计黄金卡片展台**
  
  读取 `store.modelHubData.primary_recommendations` 数据：
  - 显示 Chat、Embedding、Vision 首选推荐信息。
  - 检测模型是否已装，已装则显示“已就绪”，未装则显示“一键下载”，正在下载则配合已有的 `activeDownloads` 进度渲染环形进度条。

- [ ] **Step 2: 进行前端全量静态类型检查**
  
  运行：`pnpm vue-tsc --noEmit`
  Expected: PASS

- [ ] **Step 3: 提交所有代码**
  
  ```bash
  git add src/components/settings/ModelManager.vue
  git commit -m "feat: add primary recommended picks card gallery in ModelManager"
  ```
