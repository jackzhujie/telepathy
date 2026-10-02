# 后端性能优化设计：配置缓存 + 连接复用

**日期**: 2026-05-09
**模块**: P1 - 后端性能优化
**状态**: 设计中

---

## 1. 背景与目标

### 1.1 当前问题

**问题 1: 配置读取开销大**

`rag_query` 函数中每次请求读取 10+ 个配置项，每个都执行一次 SQL 查询：

```rust
// 每次请求执行 12+ 次 SQL 查询
let embedding_model = settings::get_setting(&conn, "embedding_model")?;
let chat_model = settings::get_setting(&conn, "chat_model")?;
let top_k = settings::get_top_k(&conn)?;
// ... 还有 9 个配置项
```

**问题 2: 数据库连接重复创建**

每次命令调用都创建新的数据库连接：

```rust
// commands/rag.rs:49
let conn = db::open_connection(&db_state)?;  // 每次都新建连接
```

### 1.2 优化目标

| 优化项 | 当前 | 优化后 |
|--------|------|--------|
| 配置读取 | 12+ SQL/请求 | 0 SQL/请求 (缓存命中) |
| DB连接 | 每次新建 | 复用预创建连接 |
| 预期延迟减少 | - | ~5-10ms/请求 |

---

## 2. 方案设计

### 2.1 配置缓存

**架构**:

```
┌─────────────────────────────────────────────────────────┐
│                    AppState                             │
│  ┌─────────────────┐  ┌────────────────────────────┐ │
│  │  DbState         │  │  SettingsCache (new)        │ │
│  │  - db_path      │  │  - RwLock<HashMap>          │ │
│  │  - connection    │  │  - 启动时加载               │ │
│  └─────────────────┘  │  - 变更时更新                │ │
│                       └────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

**实现**:

1. 创建 `SettingsCache` 结构体
2. 启动时加载所有配置到缓存
3. `get_setting` 先查缓存，缓存未命中再查数据库
4. `update_setting` 同时更新缓存和数据库

### 2.2 数据库连接复用

**方案选型**:

| 方案 | 优点 | 缺点 |
|------|------|------|
| r2d2 连接池 | 功能完整、支持连接复用 | 依赖增加 |
| 预创建连接存储在 State | 简单、无额外依赖 | 连接独占 |
| **Arc<RwLock<Connection>>** | 简单、线程安全 | - |

**推荐方案**: 使用 `Arc<RwLock<Connection>>` 存储预创建的连接

**实现**:

```rust
pub struct DbState {
    pub db_path: PathBuf,
    pub connection: Arc<RwLock<Connection>>,  // 新增：复用连接
}
```

---

## 3. 实施计划

### Task 1: 添加配置缓存

**文件**:
- 新建: `src-tauri/src/db/settings_cache.rs`
- 修改: `src-tauri/src/db/settings.rs`
- 修改: `src-tauri/src/db/mod.rs`

**实现**:
1. 创建 `SettingsCache` 结构体（`RwLock<HashMap<String, String>>`）
2. 添加 `load_all_settings()` 方法
3. 添加 `get_cached_setting()` 方法
4. 修改 `get_setting()` 使用缓存

### Task 2: 添加连接复用

**文件**:
- 修改: `src-tauri/src/db/mod.rs`
- 修改: `src-tauri/src/lib.rs`

**实现**:
1. 修改 `DbState` 添加 `connection: Arc<RwLock<Connection>>`
2. 在 `lib.rs` 初始化时创建连接并存储
3. 修改 `open_connection()` 返回复用连接

### Task 3: 更新设置命令

**文件**:
- 修改: `src-tauri/src/commands/settings.rs`

**实现**:
1. 添加 `update_setting_cached()` 方法
2. 修改设置命令使用新方法

### Task 4: 测试验证

**文件**:
- (无修改，纯测试)

**实现**:
1. `cargo build --release`
2. `cargo test`
3. 验证功能正常

---

## 4. 关键代码

### 4.1 SettingsCache

```rust
// src-tauri/src/db/settings_cache.rs
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct SettingsCache {
    cache: RwLock<HashMap<String, String>>,
}

impl SettingsCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn load_from_db(&self, conn: &Connection) {
        let settings = settings::get_all_settings(conn).unwrap_or_default();
        let mut cache = self.cache.write().unwrap();
        *cache = settings;
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.read().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &str) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(key.to_string(), value.to_string());
    }

    pub fn get_all(&self) -> HashMap<String, String> {
        self.cache.read().unwrap().clone()
    }
}
```

### 4.2 修改后的 get_setting

```rust
// 使用缓存的 get_setting
pub fn get_setting_cached(cache: &SettingsCache, conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    // 先查缓存
    if let Some(value) = cache.get(key) {
        return Ok(Some(value));
    }
    // 缓存未命中，查数据库
    let result = get_setting(conn, key)?;
    if let Some(ref v) = result {
        cache.set(key, v);
    }
    Ok(result)
}
```

### 4.3 修改后的 DbState

```rust
// src-tauri/src/db/mod.rs
use std::sync::{Arc, RwLock};
use rusqlite::Connection;

pub struct DbState {
    pub db_path: PathBuf,
    pub connection: Arc<RwLock<Connection>>,  // 复用连接
}

// 返回复用连接
pub fn open_connection(state: &DbState) -> Result<Arc<RwLock<Connection>>, AppError> {
    Ok(state.connection.clone())
}
```

---

## 5. 测试计划

1. **单元测试**: 验证缓存读写
2. **集成测试**: 验证设置更新后缓存同步
3. **性能测试**: 对比优化前后请求延迟

---

## 6. 风险评估

| 风险 | 影响 | 缓解 |
|------|------|------|
| 缓存与数据库不一致 | 中 | 更新时同时写两者 |
| 连接在某些操作后失效 | 低 | rusqlite Connection 可重新打开 |
| 多线程访问冲突 | 低 | RwLock 保护 |

---

## 7. 附录

### A. rusqlite Connection 线程安全

`rusqlite::Connection` 在启用 `bundled` 特性时支持跨线程使用（通过 `Send + Sync`）。

### B. 参考资料

- [rusqlite 文档](https://docs.rs/rusqlite/)
- [dashmap vs RwLock](https://github.com/xacrimon/dashmap)
