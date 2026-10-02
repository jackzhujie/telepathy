# P7: Ollama 模型下载与管理 - 设计文档

## 1. 项目背景 (Project Context)

P6 已实现 Ollama 安装引导，用户可一键安装 Ollama。P7 新增模型下载与管理功能，包括：预设模型列表、电脑配置检测、模型推荐、下载进度显示、自动启动服务。

## 2. 核心目标 (Core Goals)

- **预设模型列表** + **搜索更多模型**
- **自动检测电脑配置** → 推荐适合的模型
- **下载进度显示** → 百分比 + 速度 + 剩余时间
- **自动启动 Ollama** → 下载完成后自动启动服务

## 3. 架构设计 (Architecture)

### 3.1 预设模型列表

| 分类 | 模型 | 参数 | 最低内存 | 效果描述 |
|------|------|------|----------|----------|
| 轻量 | qwen2.5 | 0.5B | 4GB | 响应最快，适合快速问答，简单对话 |
| 轻量 | llama3.2 | 1B | 4GB | 响应快，适合日常对话，英文能力强 |
| 中等 | qwen2.5 | 7B | 8GB | 平衡响应与智能，中文优化好 |
| 中等 | llama3.2 | 3B | 8GB | 响应较快，智能水平中等，英文为主 |
| 旗舰 | qwen2.5 | 14B | 16GB | 智能较高，响应稍慢，中文能力强 |
| 旗舰 | llama3.2 | 8B | 16GB | 智能高，响应较慢，英文能力最强 |
| 大型 | qwen2.5 | 32B | 32GB+ | 智能最高，响应慢，需强力 GPU |
| 大型 | llama3.2 | 80B | 64GB+ | 最强智能，极慢，需高端 GPU |

### 3.2 嵌入模型（用于知识库）

| 模型 | 效果描述 |
|------|----------|
| bge-m3 | 多语言嵌入，中文效果好，主流选择 |
| bge-large-zh | 中文专用嵌入，效果最好但较大 |

### 3.3 电脑配置检测

```rust
struct SystemInfo {
    pub cpu_cores: usize,
    pub memory_gb: usize,
    pub has_gpu: bool,
    pub gpu_name: Option<String>,
}
```

推荐逻辑：
- < 4GB 内存：仅推荐轻量模型
- 4-8GB：推荐轻量 + 中等
- 8-16GB：推荐中等 + 旗舰
- 16GB+：推荐全部

### 3.4 下载进度

```typescript
interface PullProgress {
    model: string;
    percentage: number;      // 0-100
    speed: string;          // "2.5 MB/s"
    downloaded: string;    // "150 MB"
    total: string;         // "500 MB"
    eta: string;           // "2:30"
    status: 'downloading' | 'verifying' | 'done' | 'error';
}
```

### 3.5 自动启动

- 下载完成后自动执行 `ollama serve`
- 使用 `tokio::process::Command` 后台运行
- 监听端口 11434 确认服务启动

### 3.6 UI 设计

#### 模型选择界面
```
┌─────────────────────────────────────────────────┐
│  🖥️ 您的电脑配置: 16GB 内存, Apple Silicon     │
│                                                 │
│  💡 推荐模型（根据您的配置）                     │
│                                                 │
│  [qwen2.5:7b] ★★★★☆  8GB  平衡款               │
│  [llama3.2:3b]  ★★★★☆  8GB  英文对话          │
│                                                 │
│  📦 更多模型                                    │
│                                                 │
│  [搜索模型...]                                  │
│  • qwen2.5 (多尺寸)                            │
│  • llama3.2 (多尺寸)                           │
│  • mistral                                     │
│                                                 │
│  [下载选中模型]                                 │
└─────────────────────────────────────────────────┘
```

#### 下载进度界面
```
┌─────────────────────────────────────────────────┐
│  📥 正在下载 qwen2.5:7b                        │
│                                                 │
│  ████████████████░░░░░░  75%                  │
│                                                 │
│  速度: 5.2 MB/s  |  已下载: 380 MB            │
│  剩余时间: 约 30 秒                             │
│                                                 │
│  [取消下载]                                     │
└─────────────────────────────────────────────────┘
```

## 4. 文件结构

### 后端新增
- `src-tauri/src/commands/installer.rs` — 扩展 download_ollama，新增 pull_model, start_ollama, get_system_info

### 前端修改
- `src/stores/settings.ts` — 新增模型下载状态
- `src/views/Settings.vue` — 新增模型选择 UI
- `src/api/tauri.ts` — 新增 API 调用

## 5. 验收标准 (Success Criteria)

- [ ] 自动检测电脑配置并推荐适合的模型
- [ ] 显示预设模型列表
- [ ] 支持搜索更多模型
- [ ] 显示下载进度（百分比 + 速度 + 剩余时间）
- [ ] 下载完成后自动启动 Ollama
- [ ] 提示用户可以开始使用
