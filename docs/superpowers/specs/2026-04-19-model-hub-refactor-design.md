# Design Spec: Model Hub Multi-Source Refactor & Custom GGUF Downloader

## 1. 概述 (Overview)
本项目是对 Telepathy 桌面应用中的“模型管理 (Model Hub)”模块进行底层彻底重构。原本的实现过度耦合了 Ollama 官方网站的 HTML 数据结构与注册表，通过爬虫读取 `ollama.com`，且下载动作强依赖于 `ollama pull` 命令，导致功能极度受限（如无法下载其他社区微调版模型，易受地理与反爬限制等）。
本次重构将**彻底废弃 Ollama 原生的爬虫入口和直连下载链路**，整体迁移至具备海量开源社区背景的 HuggingFace (HF) 以及 魔塔社区 (ModelScope)。重点实现只获取 GGUF 规格模型、后端自定义断点续传下载，以及透明导入 Ollama。

## 2. 系统核心架构 (Architecture)

### 2.1 数据源与接口规范 (Unified Data Sources)
- 提取统一接口（Trait） `ModelRegistryClient`，作为上层应用和底层数据源之间的隔离桥梁。
- 至少实现两个引擎适配器：
  - `HuggingFaceRegistry`：解析并抓取来自 `huggingface.co/api` 的数据。
  - `ModelScopeRegistry`：解析并抓取来自 `modelscope.cn/api` 的数据。
- **GGUF 强过滤约束**：在请求搜索、拉取可用变体的时候，明确注入 `filter=gguf` 标签条件，从根源掐断无法使用的 PyTorch/Safetensors 格式模型的展示。

### 2.2 智能路由探针 (Auto-Routing via GeoSensor)
复用及强化已有的 `GeoSensor`。
- 应用启动加载模型库时，探测到达 HuggingFace 和 ModelScope 的连通性。
- **国内网络** 自动实例化并且主推使用 `ModelScopeRegistry` 以获得最佳列表加载和下载体验。
- **海外或加速网络** 获取低延迟时，使用原初的 `HuggingFaceRegistry`。

## 3. 核心机制设计 (Core Mechanisms)

### 3.1 独立断点续传下载器 (Custom Resumable Downloader)
由于脱离了 `ollama pull` 机制，我们开发一套独立的下载管理器：
1. 使用 Rust 中的 `reqwest` 支持对底层大文件的 `Chunked/Range` 下载机制。
2. 下载过程直接下盘写入系统的应用专有缓存或专门的 `downloads` 目录，例如：`<app_data>/downloads/<model_name>_<quant>.gguf.part`。
3. 高频异步将下载进度（Percentage、下载速度等）通过 Event Emit 反溃给现有的 Vue UI 进行展示。

### 3.2 Ollama 透明注入引擎 (Ollama Import Injection)
执行“随借随还（A方案）”工作流。
1. **就绪**：当 `.gguf` 文件整件下载并合并完毕后。
2. **Modelfile 创成**：代码自动在同目录写入基于此文件的轻量声明文件：
   ```text
   FROM ./<model_name>_<quant>.gguf
   # 可酌情预注入 TEMPLATE 或 PARAMETER（可选功能，留作扩展）
   ```
3. **静默加载**：通过调用本地 Daemon 或 ollama-rs 接口向 Ollama 执行 `ollama create <自定义标识名> -f Modelfile` 命令进行模型提取装填。
4. **清理现场**：成功读取/创建完成并映射进入 Ollama 自己的 Blob 对象存储后，代码将自动删除暂存所用的原装 `.gguf` 格式文件以释放硬盘空间。

## 4. UI 呈现与用户体验端 (UI & UX Adaptations)
- **前端适配平滑过渡**：保留现有的 `ModelManager.vue` 界面核心要素元素不变。
- **硬件强校验与二次确认弹窗**：结合目前的 `sysinfo::Memory` 获取本地实际内存容量，不再从搜索和榜单列表中强制屏蔽或禁用过大模型，而是维持展示其完整全量列表及标签。但当用户点击下载被判定为“本地极难运行”或“超过可用内存”的模型时，系统抛出显性的二次确认弹窗（提醒用户该规格可能导致严重卡顿或崩溃），由用户最终决定是否继续下载。
- **进度提示刷新**：对接我们自定义 Rust 下载器的 Event，支持暂定/取消操作（因为我们实现了断点续传）。

## 5. 挑战与已知影响
- **并发装填中的磁盘激增**：在第 3.2 节提到的导入步骤内，磁盘需要准备2倍物理空间给 Ollama。目前对现代用户的普遍磁盘足够宽裕，但在硬盘红色警告时下载，会引发中途爆盘的错误，需要提前做本地 `Free Space` 的侦测校验（未来可作为增强点）。
