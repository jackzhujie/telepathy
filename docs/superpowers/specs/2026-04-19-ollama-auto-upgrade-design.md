# Ollama 自动升级与智能源切换设计文档

## 概述
为了确保 Telepathy 能够支持最新的模型架构（如 Gemma 4），应用需要建立一套自动监测、静默下载及安全升级 Ollama 服务端的机制。该功能将根据用户的地理位置自动切换下载源，并在不干扰用户正常使用的情况下完成更新。

## 目标
1. **自动版本比对**：应用启动时自动检测本地 Ollama 版本并与远程最新版本进行比对。
2. **智能源选择**：根据网络环境（是否在中国境内）自动选择最佳下载镜像。
3. **静默下载**：在后台线程完成二进制文件的下载，不阻塞用户操作。
4. **安全升级**：通过用户确认后的快速重启机制，完成服务端二进制文件的替换。

## 详细设计

### 1. 后端逻辑 (Rust)

#### 1.1 版本检测 (`src-tauri/src/commands/installer.rs`)
- **`get_ollama_version`**: 执行 `ollama --version` 捕获 stdout。
  - 使用正则 `ollama version is (\d+\.\d+\.\d+)` 提取版本号。
- **`check_latest_version`**: 
  - 根据 `GeoSensor::is_likely_in_china()` 结果决定 API 终端。
  - 境内优先访问 GitHub 镜像 API 或特定的版本信息文件。
  - 解析版本号并返回给前端。

#### 1.2 智能源配置
- **国际地址**: `https://github.com/ollama/ollama/releases/latest`
- **加速地址**: `https://mirror.ghproxy.com/https://github.com/ollama/ollama/releases/download/{tag}/ollama-darwin-{arch}` (示例)

#### 1.3 服务切换逻辑
- 下载完成后校验二进制文件的基本完整性（如权限设置）。
- 重启函数流程：
  1. 向前端发送 `update-ready` 事件。
  2. 收到用户确认后，执行 `pkill` 停止 `ollama serve`。
  3. 执行文件覆盖（将新二进制从临时目录移动到安装目录）。
  4. 重新 `spawn` Ollama 服务并保持环境变量（如 `OLLAMA_MODELS`）的一致性。

### 2. 前端逻辑 (Vue/Pinia - `src/stores/settings.ts`)

#### 2.1 状态管理
- 新增 `ollamaVersion`: 当前已安装版本。
- 新增 `latestAvailableVersion`: 远程最新版本。
- 新增 `isUpdateDownloading`: 后台下载状态。

#### 2.2 交互流程
1. **App 启动**：后台检查版本逻辑。
2. **发现更新**：发现新版本后，右下角弹出 Toast 提示“发现 Ollama 新版本，正在后台静默同步...”。
3. **进度显示**：在“设置 -> 系统信息 -> Ollama 状态”显示当前下载进度。
4. **完成提示**：下载完成后弹窗：“Ollama 的新版本已就绪。重启服务以支持更多新模型（如 Gemma 4）。[立即重启] [稍后手动]”。

## 异常处理
- **下载失败**: 自动重试 3 次，若仍失败则放弃本次静默升级，记录错误。
- **权限不足**: 若因文件权限无法覆盖，引导用户手动授予权限或通过系统提示完成操作。
- **服务无法启动**: 若新版本执行后崩溃，自动恢复旧版本二进制（备份逻辑）。

## 验证计划
- 模拟低版本环境（将本地版本标记为 0.1.19），验证是否触发升级流程。
- 模拟中国境内网络环境，验证是否切换到加速镜像源。
- 验证在 RAG 对话过程中，服务重启对前端状态的影响是否可控。
