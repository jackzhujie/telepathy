# Telepathy P0：项目骨架搭建 — 设计规格

## 1. 概述

Telepathy 是一个本地 AI 知识库桌面应用，使用 Tauri v2 + Vue 3 + Rust + Ollama 构建。本文档描述 P0 阶段的目标：搭建项目骨架，验证工具链和前后端通信链路。

P0 不包含任何业务逻辑，只建立项目结构和开发环境。

## 2. 技术栈

| 层 | 技术 | 版本要求 |
|---|------|---------|
| 框架 | Tauri | v2 |
| 前端 | Vue | 3.5+ |
| 语言 | TypeScript | 5.4+ (strict) |
| 构建 | Vite | 5+ |
| UI 组件库 | Naive UI | 2.38+ |
| 状态管理 | Pinia | 2+ |
| 路由 | Vue Router | 4+ |
| 包管理器 | pnpm | 9+ |
| 后端语言 | Rust | stable 1.75+ |
| Rust 依赖 | tauri v2, serde, serde_json, thiserror, tokio | — |

**初始化方式：** `pnpm create tauri-app telepathy --template vue-ts`，然后手动添加 Naive UI / Pinia / Vue Router。

## 3. 前端目录结构

```
src/
├── api/
│   └── tauri.ts            # Tauri invoke 封装
├── assets/                  # 静态资源
├── components/
│   └── layout/
│       ├── AppLayout.vue    # 主布局（侧边栏 + 内容区）
│       └── Sidebar.vue      # 左侧导航栏
├── router/
│   └── index.ts             # Vue Router 配置
├── stores/
│   └── app.ts               # Pinia store（全局状态）
├── views/
│   ├── KnowledgeBase.vue    # 知识库页面（占位）
│   ├── Chat.vue             # 对话页面（占位）
│   ├── Documents.vue        # 文档管理页面（占位）
│   └── Settings.vue         # 设置页面（含 greet 测试）
├── types/
│   └── index.ts             # 全局类型定义
├── App.vue                  # 根组件
└── main.ts                  # 入口文件
```

## 4. 路由设计

| 路径 | 页面组件 | 说明 |
|------|---------|------|
| `/` | KnowledgeBase.vue | 知识库首页 |
| `/chat` | Chat.vue | AI 对话 |
| `/documents` | Documents.vue | 文档管理 |
| `/settings` | Settings.vue | 设置（含通信测试） |

## 5. 前端布局

使用 Naive UI 的 `n-layout` + `n-layout-sider` 实现：
- **左侧：** 可折叠侧边栏（`Sidebar.vue`），包含 4 个导航项，使用 `n-menu` 组件
- **右侧：** 主内容区，通过 `<router-view>` 渲染当前路由页面
- 4 个页面均为空白占位页，只显示页面标题

## 6. Rust 后端目录结构

```
src-tauri/src/
├── main.rs                  # 入口
├── lib.rs                   # Tauri app builder、注册 commands
├── commands/
│   ├── mod.rs               # commands 模块导出
│   └── greet.rs             # greet command
├── services/
│   └── mod.rs               # services 模块导出（空）
├── models/
│   └── mod.rs               # models 模块导出（空）
└── errors.rs                # 统一错误类型 AppError
```

## 7. 后端架构

三层架构：
- **commands/** — Tauri command 层，接收前端请求，调用 services
- **services/** — 业务逻辑层（P0 阶段为空）
- **models/** — 数据模型层（P0 阶段为空）

统一错误处理：
```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("内部错误: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}
```

## 8. 前后端通信验证

P0 的核心验收点——证明 Vue ↔ Tauri ↔ Rust 通信链路完整。

**Rust 侧 (`commands/greet.rs`)：**
```rust
#[tauri::command]
pub fn greet(name: &str) -> Result<String, AppError> {
    Ok(format!("你好，{}！Telepathy 已准备就绪。", name))
}
```

**前端侧 (`api/tauri.ts`)：**
```typescript
import { invoke } from '@tauri-apps/api/core'

export async function greet(name: string): Promise<string> {
  return invoke<string>('greet', { name })
}
```

**验证位置：** Settings 页面包含一个简单的测试区域——输入名字 → 点击按钮 → 显示 Rust 返回的结果。

## 9. 关键配置文件

| 文件 | 说明 |
|------|------|
| `tauri.conf.json` | App identifier: `com.telepathy.app`，窗口 1200×800，最小 900×600，标题 "Telepathy" |
| `vite.config.ts` | 路径别名 `@` → `src/`，Naive UI 按需导入 |
| `tsconfig.json` | strict: true, paths alias, Tauri 类型引入 |
| `Cargo.toml` | tauri v2, serde, serde_json, thiserror, tokio |
| `.gitignore` | node_modules, target, dist, .env 等 |

## 10. P0 验收标准

| # | 验收项 | 验证方式 |
|---|--------|----------|
| 1 | `pnpm tauri dev` 启动无报错 | 终端无 error |
| 2 | 窗口正常显示，标题为 "Telepathy" | 目视 |
| 3 | 侧边栏 4 个导航项可点击切换 | 点击测试 |
| 4 | 路由切换正确，页面标题/内容对应 | 点击每个导航项 |
| 5 | Greet command 前后端通信成功 | Settings 页面输入名字，显示 Rust 返回结果 |
| 6 | Naive UI 组件正常渲染（按钮、布局等） | 目视 |
| 7 | `cargo clippy` 无 warning | 终端 |
| 8 | `pnpm type-check` 无 TS 错误 | 终端 |
| 9 | `pnpm build` + `pnpm tauri build` 可正常构建 | 终端 |

## 11. 明确排除（P0 不做）

- 数据库（SQLite）
- Ollama 集成
- 文档导入/解析
- 向量存储
- 任何业务逻辑
- 主题切换 / 暗色模式
- 自动化测试

## 12. 后续阶段概览

| 阶段 | 内容 |
|------|------|
| P1 | Ollama 集成 |
| P2 | 文档导入与解析 |
| P3 | 文本分块与向量化 |
| P4 | RAG 检索与问答 |
| P5 | 打磨与增强 |
