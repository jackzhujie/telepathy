# Telepathy

本地优先的 AI 知识库桌面应用。基于本地 LLM 推理引擎，让你在完全离线的环境下构建个人知识库并进行智能问答。

> 你的数据永远只存在于本地，不上传任何服务器。

## 特性

- **本地 LLM 推理** — 内置 llama.cpp 推理引擎，支持 GGUF 格式模型，完全离线运行
- **RAG 检索增强生成** — 基于 HNSW 向量索引，从你的文档中检索相关内容进行问答
- **多格式文档解析** — 支持 PDF、Word、Excel、PPT、Markdown、TXT、图片等多种格式
- **视觉模型支持** — 支持多模态模型，可对图片进行 OCR 和理解
- **模型管理** — 内置模型市场，一键下载和管理本地模型
- **GPU 加速** — 支持 macOS Metal、Vulkan、OpenMP 等多种后端加速
- **SnapNote** — 快速记录灵感笔记
- **多知识库** — 按项目/主题组织你的知识库
- **跨平台** — 支持 macOS（Apple Silicon / Intel）、Windows、Linux

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3 + TypeScript + Vite + Tailwind CSS + Pinia + reka-ui |
| 后端 | Rust (Tauri v2) |
| 数据库 | SQLite (rusqlite) |
| AI 推理 | llama.cpp (llama-cpp-4) |
| 向量索引 | HNSW (usearch) |
| 文档解析 | pdf-extract, calamine, quick-xml, image |

## 系统要求

- **macOS**: 10.15+ (Apple Silicon 推荐)
- **Windows**: 10+
- **Linux**: Ubuntu 22.04+ / 同等发行版

## 快速开始

### 从 Release 安装

前往 [GitHub Releases](https://github.com/jackzhujie/telepathy/releases) 下载对应平台的安装包。

### 从源码构建

#### 前置依赖

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) 稳定版
- [pnpm](https://pnpm.io/) 9+

#### macOS 额外依赖

```bash
brew install openssl@3
```

#### Linux 额外依赖

```bash
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf rpm
```

#### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/jackzhujie/telepathy.git
cd telepathy

# 安装前端依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 项目结构

```
telepathy/
├── src/                    # 前端源码 (Vue 3)
│   ├── views/              # 页面视图
│   ├── components/         # 组件
│   ├── stores/             # Pinia 状态管理
│   ├── hooks/              # 组合式函数
│   └── types/              # TypeScript 类型定义
├── src-tauri/              # 后端源码 (Rust)
│   ├── src/
│   │   ├── commands/       # Tauri 命令层
│   │   ├── services/       # 业务逻辑层
│   │   │   ├── inference/  # LLM 推理引擎
│   │   │   ├── model_hub/  # 模型下载与管理
│   │   │   ├── parser/     # 文档解析
│   │   │   ├── embedder.rs # 向量化
│   │   │   ├── hnsw_index.rs # 向量索引
│   │   │   └── rag.rs      # RAG 检索
│   │   └── db/             # 数据库层
└── docs/                   # 设计文档
```

## 架构概览

```
用户界面 (Vue 3)
    ↓ Tauri IPC
命令层 (commands/)
    ↓
服务层 (services/)
    ├── inference/    → llama.cpp 本地推理
    ├── embedder.rs   → 文本向量化
    ├── hnsw_index.rs → HNSW 向量检索
    ├── rag.rs        → RAG 问答编排
    ├── parser/       → 多格式文档解析
    └── model_hub/    → 模型下载管理
    ↓
数据库层 (db/) → SQLite
```

## 开发说明

### 数据目录

应用数据默认存储在系统的 app data 目录。可通过环境变量覆盖：

```bash
TELEPATHY_DATA_DIR=/path/to/data pnpm tauri dev
```

### 调试

后端 debug 命令默认可用，可通过 `src-tauri/src/commands/debug.rs` 查看调试接口。

## 许可证

本项目采用 [Apache License 2.0](LICENSE) 开源。

## 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建你的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交你的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启一个 Pull Request

## 致谢

- [Tauri](https://tauri.app/) — 跨平台桌面应用框架
- [llama.cpp](https://github.com/ggerganov/llama.cpp) — 本地 LLM 推理
- [usearch](https://github.com/unum-cloud/usearch) — HNSW 向量索引
- 所有开源贡献者
