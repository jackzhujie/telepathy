# 设计规格书：内置模型自适应推荐与首次运行引导向导

本规格书阐述了在本地 AI 知识库桌面应用（Telepathy）中，如何引入“黄金首选推荐模型”与“首次运行引导向导（Setup Wizard）”的完整设计，旨在通过零门槛的一键式配置，最大化降低用户的上手难度，同时通过临时后缀隔离等机制提升多平台大文件下载的异常容错。

---

## 1. 业务目标与交互设计

### 1.1 首次启动引导向导 (Setup Wizard)
- **触发条件**：当应用启动且**本地没有任何已下载完成的模型**（即 `installedModels.length == 0`）且**用户之前未点击跳过或配置过向导**时弹出。
- **展示信息**：
  - 硬件探测信息：“检测到您的设备为：Apple M1 Max (32GB RAM)”
  - 核心首选推荐组合展示：
    - **对话推理首选**：Qwen 2.5 14B (约 9.0GB，当前硬件体验最佳)
    - **向量检索首选**：BGE Large (中文) (约 0.67GB，提供精准文档搜索检索)
- **操作选项**：
  - **一键配置并安装**：点击后在后台异步静默触发下载这两个推荐模型，关闭向导，进入主界面。
  - **跳过引导，手动配置**：关闭向导，向数据库持久化标记 `wizard_completed = true`。

### 1.2 模型管理页置顶推荐 (Page Pick Card)
- 在模型管理 (`src/components/settings/ModelManager.vue`) 的 `推荐` 选项卡顶部，增加一个显眼的“官方首选推荐组合”看板，分别以 Chat、Embedding、Vision 三大功能区块展示，提供直接的一键下载/切换按钮。

---

## 2. 架构设计与数据流

推荐逻辑遵循**“后端硬件评估、前端呈现消费”**的原则：

```
+-------------------------------------------------------------+
|                      Rust Backend                           |
|                                                             |
|   +-----------------------------------------------------+   |
|   | sysinfo Hardware Detector                           |   |
|   +-------------------------+---------------------------+   |
|                             |                               |
|                             v Hardware Profile              |
|   +-------------------------+---------------------------+   |
|   | Recommender (compute_primary_recommendations)       |   |
|   +-------------------------+---------------------------+   |
|                             |                               |
|                             v PrimaryRecommendations        |
+-----------------------------+-------------------------------+
                              | Tauri Command: get_model_hub
                              v
+-----------------------------+-------------------------------+
|                      Vue Frontend                           |
|                                                             |
|   +-------------------------+---------------------------+   |
|   | Pinia Store (settings.ts)                           |   |
|   |  - showSetupWizard = !wizardCompleted && len == 0   |   |
|   +-------------------------+---------------------------+   |
|                             |                               |
|              +--------------+--------------+                |
|              |                             |                |
|              v                             v                |
|   +----------+----------+       +----------+----------+     |
|   | SetupWizard.vue     |       | ModelManager.vue    |     |
|   +---------------------+       +---------------------+     |
+-------------------------------------------------------------+
```

---

## 3. 详细设计与代码变更

### 3.1 Rust 后端

#### `src-tauri/src/commands/models.rs` [MODIFY]
- 新增 `PrimaryRecommendations` 结构体：
  ```rust
  #[derive(Debug, Serialize, Clone)]
  pub struct PrimaryRecommendations {
      pub chat: Option<HubRecommendation>,
      pub embedding: Option<HubRecommendation>,
      pub vision: Option<HubRecommendation>,
  }
  ```
- 扩展 `ModelHubResponse` 返回体，新增 `primary_recommendations: PrimaryRecommendations` 字段。
- 在 `get_model_hub` 中调用 `Recommender::compute_primary_recommendations(&hardware, &hub_cache)` 完成匹配。

#### `src-tauri/src/services/model_hub/recommender.rs` [MODIFY]
- 新增推荐匹配函数：
  ```rust
  pub fn compute_primary_recommendations(
      profile: &HardwareProfile,
      hub: &HubCache,
  ) -> PrimaryRecommendations {
      let ram_gb = profile.ram_total as f64 / 1_073_741_824.0;
      
      // 1. Chat 模型自适应匹配
      let chat_id = "qwen2.5";
      let chat_tag = if ram_gb <= 9.0 {
          "1.5b"
      } else if ram_gb <= 18.0 {
          "7b"
      } else {
          "14b"
      };
      let chat = Self::find_recommendation_by_id_tag(hub, profile, chat_id, chat_tag);

      // 2. Embedding 向量检索模型匹配
      let embed_id = "bge-large-zh";
      let embedding = Self::find_recommendation_by_id_tag(hub, profile, embed_id, "latest");

      // 3. Vision 多模态模型匹配 (仅对内存 > 8G 设备推荐)
      let vision = if ram_gb > 8.0 {
          Self::find_recommendation_by_id_tag(hub, profile, "qwen2.5-vl", "7b")
      } else {
          None
      };

      PrimaryRecommendations { chat, embedding, vision }
  }
  ```

#### 下载命令异常容错 `src-tauri/src/commands/models.rs` [MODIFY]
- 引入 `.part` 临时后缀进行隔离下载：
  ```rust
  let dest_filename = format!("{}-{}.gguf", model_name, actual_variant);
  let final_path = app_data_dir.join("downloads").join(&dest_filename);
  let temp_path = app_data_dir.join("downloads").join(format!("{}.part", dest_filename));

  // 1. 将临时路径 temp_path 传给 downloader.download()
  let download_res = downloader
      .download(&download_url, &temp_path, &registration_id, expected_size, &app_handle)
      .await;

  // 2. 下载成功后重命名
  if download_res.is_ok() {
      tokio::fs::rename(&temp_path, &final_path).await?;
  }
  ```

---

### 3.2 Vue 前端

#### `src/stores/settings.ts` [MODIFY]
- 新增持久化字段 `wizard_completed` 读写：
  ```typescript
  const wizardCompleted = computed({
    get: () => settings.value.wizard_completed === 'true',
    set: (v: boolean) => saveSetting('wizard_completed', String(v))
  });

  const showSetupWizard = computed(() => {
    return !wizardCompleted.value && installedModels.value.length === 0;
  });
  ```
- 新增批量并行下载 Action：
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
  ```

#### `src/App.vue` [MODIFY]
- 挂载 `<SetupWizard />` 遮罩组件，且只在 `settingsStore.showSetupWizard` 为 `true` 时渲染。

#### `src/components/settings/SetupWizard.vue` [NEW]
- 创建一个全新的 UI 弹窗，采用高级的 HSL 主题色卡片、磨砂玻璃背景。
- 展示硬件探测结果，列出推荐的 Chat/Embedding 模型及其大小。
- 绑定 `downloadPrimaryPresets` 与 `skipSetupWizard` 事件。

#### `src/components/settings/ModelManager.vue` [MODIFY]
- 在“推荐”Tab 的顶部引入一排首选推荐的黄金卡片。当检测到其中任意一款已下载时，按钮变为“设为当前默认”或“已就绪”状态。

---

## 4. 异常与容错方案设计 (Resilience Plan)

### 4.1 网络中断 / 程序关闭
- **已下载字节保留**：因采用 `.part` 临时后缀，断网或关闭程序后，已下载的临时分块得以完整保留在磁盘。
- **自动断点续传**：下次用户再次触发该模型下载时，`ModelDownloader` 会检测到 `.part` 文件的实际字节，并发送 HTTP `Range` 头请求剩下部分，100% 避免重复拉取。
- **防止崩溃**：只要未下载完成，该模型永远不会以 `.gguf` 格式出现在 `downloads` 文件夹中，彻底避免了 Llamacpp 引擎加载未下完模型导致的奔溃。

### 4.2 磁盘空间紧张
- 后端在下载开始前会通过 `sysinfo` 对目标磁盘空间进行预先评估（剩余空间必须大于 `模型大小 + 1GB` 缓存空间），空间不足时抛出明确的警告通知“磁盘空间不足，无法开始下载”。

---

## 5. 验证计划 (Verification Plan)

### 5.1 自动化测试 (Automated Tests)
- 在 `src-tauri` 中为推荐核心编写单元测试：
  ```rust
  #[test]
  fn test_compute_primary_recommendations() {
      // 模拟 8GB、16GB、32GB 的内存配置，验证算出的 primary 结果符合推荐矩阵规格。
  }
  ```
- 运行测试命令：
  ```bash
  cargo test --lib services::model_registry
  ```

### 5.2 手动验证流程 (Manual Verification)
1. **删除本地模型与缓存**：清理 `Library/Application Support/com.telepathy.app/downloads` 目录，并在数据库中清除 `wizard_completed`（或通过 sqlite3 执行 `DELETE FROM settings WHERE key='wizard_completed';`）。
2. **首次启动检测**：使用 `pnpm tauri dev` 启动，验证是否自动弹出 `SetupWizard` 引导。
3. **硬件自适应检查**：验证弹窗中识别的 RAM 大小是否与您本机一致，且算出的模型推荐完全符合您的内存规格。
4. **一键后台下载验证**：
   - 点击“一键配置并安装”，弹窗应自动关闭。
   - 验证左下角状态区是否交替/同时展示两个推荐模型的下载百分比。
5. **异常中断验证**：
   - 在下载到 10% 时，**强行关闭终端的 tauri dev 进程**。
   - 检查 `downloads` 文件夹下是否存在 `.part` 临时文件，且**不存在**同名的 `.gguf` 文件。
   - 重新运行 `pnpm tauri dev`。在模型管理页再次点击该模型下载，验证控制台输出中 `Range: bytes=` 是否正确包含已有文件大小，且下载百分比从 10% 左右直接开始增长。
