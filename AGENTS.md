# Telepathy 项目规则

## 项目概述
这是一个本地 AI 知识库桌面应用（Telepathy），基于 Tauri v2 + Vue 3 + Rust 构建，核心功能是 RAG 检索增强生成文档问答。

## 技术栈
- **前端**: Vue 3.5 + TypeScript + Vite 6 + reka-ui + Tailwind CSS + Pinia
- **后端**: Rust (Tauri v2)，三层架构 (commands → services → db)
- **数据库**: SQLite (rusqlite)
- **AI**: llama.cpp (本地 LLM + Embeddings)

## 代码规范
- Rust 代码遵循标准 Rust 约定 (cargo fmt, clippy)
- 前端使用 Composition API + `<script setup>` 语法
- TypeScript 严格模式已启用
- 所有 Tauri 命令必须有正确的错误处理 `Result<T, E>`
- 计划文档保存到 `docs/superpowers/plans/`
- 设计文档保存到 `docs/superpowers/specs/`

## 语言要求
- 所有响应和讨论必须使用中文（简体中文）
- 代码注释可以使用英文

## 技能使用规则

本项目配置了以下技能（位于 `.trae/skills/` 目录），请根据场景自动调用：

### 开发流程技能（按顺序使用）
1. **brainstorming** — 任何创造性工作前的需求探索和设计（创建功能、构建组件、修改行为前必须使用）
2. **writing-plans** — 有规格说明或需求后，编写实现计划（在写代码之前使用）
3. **executing-plans** — 按照实现计划执行开发任务
4. **test-driven-development** — 实现功能时使用 TDD 方式（先写测试再写实现）
5. **verification-before-completion** — 完成任务前进行验证
6. **requesting-code-review** — 完成功能后请求代码审查
7. **receiving-code-review** — 接收代码审查反馈时使用
8. **finishing-a-development-branch** — 开发分支完成后进行整合

### 调试与质量
9. **systematic-debugging** — 遇到 bug、测试失败或异常行为时使用（先找根因再修复）
当修改了前端代码，请使用tsc测试代码是否有异常
如果修改了后端代码，请使用Cargo验证代码是否能编译

### 效率提升
10. **dispatching-parallel-agents** — 面对 2+ 个独立任务时并行分发
11. **subagent-driven-development** — 在当前会话中执行有独立任务的实现计划

### UI/UX 设计
12. **ui-ux-pro-max** — 构建 Web 组件、页面、应用界面时使用

### 工具使用
13. **using-superpowers** — 每次对话开始时使用，建立如何查找和使用技能的意识
14. **using-git-worktrees** — 需要隔离工作环境时创建 git worktree
15. **writing-skills** — 创建或编辑新技能时使用

### 重要原则
- **using-superpowers** 是入口技能，确保在每次对话中考虑是否有适用的技能
- **brainstorming** 是硬性门槛 — 任何实现工作前必须先完成设计并获得用户批准
- **systematic-debugging** 的铁律：未完成根因分析前不得提出修复方案
- 用户指令优先级最高，技能规则次之，默认系统提示最低
- 项目还没发布，所有功能修改和设计不用考虑兼容性和历史问题
