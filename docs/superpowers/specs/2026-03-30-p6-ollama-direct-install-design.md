# P6 扩展: Ollama 直接下载安装 - 设计文档

## 1. 项目背景 (Project Context)

P6 基础版已实现 Ollama 安装引导（复制命令/打开网页）。为提升用户体验，新增直接下载安装功能，用户点击"一键安装"即可完成 Ollama 下载，无需手动操作终端。

## 2. 核心目标 (Core Goals)

- **一键安装**: 点击按钮自动下载 Ollama 二进制
- **下载进度**: 实时显示下载进度百分比
- **跨平台支持**: 支持 macOS + Linux
- **用户目录安装**: 安装到 `~/.telepathy/ollama`，无需 sudo 权限

## 3. 架构设计 (Architecture)

### 3.1 平台检测

```rust
fn get_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    return "macos";
    #[cfg(target_os = "linux")]
    return "linux";
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return "unsupported";
}
```

### 3.2 下载逻辑

- 从 `https://ollama.com/download/Ollama-darwin.zip` (macOS) 或 `https://ollama.com/download/Ollama-linux.zip` (Linux) 下载
- 解压到 `~/.telepathy/ollama/`
- 验证可执行文件存在

### 3.3 状态管理

```typescript
interface OllamaInstallState {
  status: 'uninstalled' | 'downloading' | 'ready';
  progress: number;        // 0-100
  installedPath: string;   // 安装路径
  error?: string;
}
```

### 3.4 UI 设计

#### 下载中状态
```
┌─────────────────────────────────────────────────┐
│  📥 正在下载 Ollama...                           │
│                                                 │
│  ████████████░░░░░░░░░░░░  45%                │
│                                                 │
│  正在下载 Ollama-darwin.zip (约 170MB)         │
│                                                 │
│  [取消下载]                                     │
└─────────────────────────────────────────────────┘
```

#### 安装完成状态
```
┌─────────────────────────────────────────────────┐
│  ✅ Ollama 安装完成                              │
│                                                 │
│  安装路径: /Users/user/.telepathy/ollama/ollama │
│                                                 │
│  提示: Ollama 已下载但需要启动才能使用。       │
│  你可以在设置页面配置 Ollama 地址。             │
│                                                 │
│  [检查状态]                                     │
└─────────────────────────────────────────────────┘
```

## 4. 文件结构

### 后端新增
- `src-tauri/src/commands/ollama.rs` — 下载安装命令

### 前端修改
- `src/stores/settings.ts` — 新增安装状态
- `src/views/Settings.vue` — 下载 UI 和进度显示
- `src/api/tauri.ts` — 新增安装相关 API 调用

## 5. 验收标准 (Success Criteria)

- [ ] 支持 macOS 和 Linux 平台自动检测
- [ ] 一键安装按钮触发下载
- [ ] 显示实时下载进度百分比
- [ ] 下载完成后显示安装路径
- [ ] 可取消下载
- [ ] 安装完成后可检测状态
