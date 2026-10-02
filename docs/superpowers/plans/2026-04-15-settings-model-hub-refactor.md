# 设置页面与模型中心深度重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 恢复设置页面的核心模型切换功能，增强模型库同步与搜索体验，并移除特化硬件声明。

**Architecture:** 
- 后端改进 `ModelHubManager` 的同步深度与解析能力。
- 新增基于实时抓取的全量搜索命令。
- 前端重构 `ModelManager` 布局，引入已安装模型快速管理区与固定选择器。

**Tech Stack:** Rust (Tauri v2), Vue 3 (Composition API), Pinia, reka-ui (Radix UI components)

---

### Task 1: 后端同步增强 (Sync & Parse)

**Files:**
- Modify: `src-tauri/src/services/model_hub/manager.rs`

- [ ] **Step 1: 提升同步上限并解析下载量数据**
找到 `sync_from_ollama` 函数：
1. 将 `.take(20)` 改为 `.take(100)`。
2. 在循环解析 `info` 时，获取模型下载量 `pulls`。

- [ ] **Step 2: 编译验证**
Run: `cargo check`
Expected: 编译通过无错误。

- [ ] **Step 3: Commit**
```bash
git add src-tauri/src/services/model_hub/manager.rs
git commit -m "feat(backend): increase hub sync limit to 100 and parse pulls data"
```

### Task 2: 环境探测优化 (Geo Cache)

**Files:**
- Modify: `src-tauri/src/services/model_hub/geo.rs`

- [ ] **Step 1: 为 probe_mirrors 增加缓存逻辑**
使用 Lazy/Static 模式缓存 `probe_mirrors` 结果，显著缩短页面加载时间。

- [ ] **Step 2: Commit**
```bash
git add src-tauri/src/services/model_hub/geo.rs
git commit -m "perf(backend): cache geo probe results to speed up settings load"
```

### Task 3: 实时全局搜索 (Global Search API)

**Files:**
- Modify: `src-tauri/src/commands/models.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 实现 search_hub_models Rust 命令**
新增命令，调用后台解析器实现全量云端搜索。

- [ ] **Step 2: 注册命令并导出 TS 定义**

- [ ] **Step 3: Commit**
```bash
git add src-tauri/src/commands/models.rs src-tauri/src/lib.rs src/api/tauri.ts
git commit -m "feat: add global hub search command"
```

### Task 4: 前端 UI 重构 (Model Manager UI)

**Files:**
- Modify: `src/components/settings/ModelManager.vue`
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: Store 增加搜索方法**

- [ ] **Step 2: 重构 ModelManager.vue**
1. 顶部增加“已下载模型”磁贴布局。
2. 在 Tab 之上增加 `SelectRoot` 呈现核心模型切换入口。
3. 实现回车搜索。

- [ ] **Step 3: Commit**

### Task 5: 优化建议通用化 (Parameter Cleanup)

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: 移除硬编码 Intel 文案**

- [ ] **Step 2: 开发者自测与编译确认**

- [ ] **Step 3: Commit**
