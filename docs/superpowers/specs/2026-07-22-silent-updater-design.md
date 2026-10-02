# 2026-07-22-silent-updater-design

本设计规范描述了本地 AI 知识库桌面应用（Telepathy）后台静默检查更新与自动下载的实现，以及配合 GitHub Actions Git Tag 的云端自动发布流程。

## 需求概述
1. **后台静默检测**：应用运行期间，每隔 30 分钟在后台静默检查是否有新版本，不弹窗打扰用户。
2. **静默下载**：若发现有更新且配置中开启了自动更新（默认开启），自动在后台拉取更新包进行安装。
3. **无打扰提示**：下载完毕后，仅在头部展示绿色的“重启更新”小按钮，由用户在闲暇时自主点击重启应用更新。
4. **CI/CD 发布流水线**：使用 Git Tag 触发 GitHub Actions 编译、自签名，并运行内置部署脚本分发至服务器发布。

## 详细设计

### 1. 更新检测与自动下载（前端）
在 `src/App.vue` 中导入 `onUnmounted`，在生命周期钩子中实现 30 分钟的静默轮询：

- **轮询控制**：
  在 `onMounted` 阶段声明定时器，周期为 1800000 毫秒（30 分钟），执行 `checkForUpdates(true)`。
- **防止内存泄露**：
  在 `onUnmounted` 阶段清除定时器：`clearInterval(updateInterval)`。
- **自动后台下载**：
  保持现有的 `watch(updateInfo)`。如果检测到有更新（`val` 非空）且开启了自动更新（`settingsStore.autoUpdate !== 'false'`），则直接触发 `installUpdate()` 开启后台下载。
- **静默提示**：
  当下载完成（`isAppUpdateReady` 为 `true`），`AppLayout.vue` 头部的绿色“重启更新”按钮将亮起。

### 2. 版本号与 CI/CD 自动发布配置
- **版本号升级**：
  将版本号升级为 `0.3.9`，影响文件：
  - [tauri.conf.json](file:///Users/mac/project/telepathy/src-tauri/tauri.conf.json) 中的 `version` 字段。
  - [package.json](file:///Users/mac/project/telepathy/package.json) 中的 `version` 字段。
- **打包触发**：
  提交所有更改并推送至仓库，然后打上 Git 标签 `v0.3.9`。
- **自动化构建与分发**：
  - GitHub Actions 捕获 `v*` tag，启动 [release.yml](file:///Users/mac/project/telepathy/.github/workflows/release.yml) 工作流。
  - 在云端进行跨平台构建。
  - 自动读取 GitHub Secrets 中的签名密钥 `TAURI_SIGNING_PRIVATE_KEY` 并在构建输出目录中生成对应的 `.sig` 签名文件。
  - 使用 `tauri-apps/tauri-action` 将构建产物上传至 GitHub Releases。

## 验证计划
1. **编译检查**：在本地运行 `pnpm build` 和 `pnpm vue-tsc --noEmit`，验证 TypeScript 和打包编译通过。
2. **后端测试**：运行 `cargo test` 确保无 Rust 回归故障。
3. **人工确认**：确认版本号配置一致，标签发布说明准备就绪。
