# 配置缓存 + 连接复用实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `subagent-driven-development` or `executing-plans` to implement this plan.

**Goal:** 实现配置缓存和数据库连接复用，减少请求延迟

**Architecture:** 使用 RwLock<HashMap> 实现配置缓存，Arc<RwLock<Connection>> 实现连接复用

**Tech Stack:** Rust, rusqlite, once_cell

---

## 任务分解

### Task 1: 创建 SettingsCache 模块

**Files:**
- Create: `src-tauri/src/db/settings_cache.rs`
- Modify: `src-tauri/src/db/mod.rs`

**Steps:**
1. 创建 SettingsCache 结构体
2. 实现 load_from_db、get、set 方法
3. 添加到 mod.rs 导出

### Task 2: 修改 DbState 添加连接复用

**Files:**
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. 修改 DbState 添加 connection 字段
2. 修改 lib.rs 初始化时创建连接
3. 修改 open_connection 返回复用连接

### Task 3: 修改 settings.rs 使用缓存

**Files:**
- Modify: `src-tauri/src/db/settings.rs`

**Steps:**
1. 添加 get_setting_cached 函数
2. 修改 get_top_k 等函数使用缓存

### Task 4: 修改命令使用缓存和复用连接

**Files:**
- Modify: `src-tauri/src/commands/rag.rs`
- Modify: `src-tauri/src/commands/settings.rs`

**Steps:**
1. 使用缓存读取配置
2. 使用复用连接

### Task 5: 测试验证

**Files:**
- (无修改)

**Steps:**
1. cargo build --release
2. cargo test

---

## 执行方式选择

**Plan complete and saved.** Two execution options:

1. **Subagent-Driven (推荐)** - 每个任务派发子 agent 执行
2. **Inline Execution** - 在当前会话执行

Which approach?