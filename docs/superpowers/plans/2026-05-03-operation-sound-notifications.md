# 计划文档：核心操作完成增加声音反馈

## 1. 概述
在应用的三个核心操作完成时增加悦耳的声音反馈：
1. **回答完毕**：AI 回答生成流式输出完成时。
2. **索引完毕**：文档向量化索引建立成功时。
3. **模型下载完毕**：本地模型下载并安装成功时。

## 2. 修改计划

### A. 回答完毕（Chat 回答完毕）声音触发
在 `src/views/Chat.vue` 的流式问答事件接收和处理逻辑末尾，当监听到 `is_generating` 状态变为 false，或调用 `stop_generation`，或者完成整个生成过程时，触发 `playNotificationSound()`。

### B. 索引完毕（Document 索引完毕）声音触发
在 `src/views/Documents.vue` 的 `handleIndex` 和 `BatchImportDialog.vue` 的索引完成逻辑中，当 `runIndex` 成功并完成时，触发 `playNotificationSound()`。

### C. 模型下载完毕（Model 下载完毕）声音触发
在 `src/views/Models.vue` 或是模型下载和安装状态变更的回调分支（如监听到模型下载事件完成、或者是 `Ollama` 安装成功）时，触发 `playNotificationSound()`。

## 3. 验证方案
1. 运行 `pnpm tsc --noEmit` 保证 TypeScript 正常编译通过。
