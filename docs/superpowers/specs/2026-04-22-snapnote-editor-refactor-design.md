# SnapNote 编辑器重构设计方案 (Typora 模式)

## 1. 背景与目标
当前 `SnapNote` 页面使用的 `md-editor-v3` 是一款传统的 Markdown 编辑器，其“分栏预览”模式与用户追求的“即时渲染、沉浸式创作”体验（类似 Typora）存在差距。
本方案旨在通过引入 `Milkdown` 编辑器引擎，为用户提供极致的高性能 WYSIWYG（所见即所得）体验，同时保留双栏模式以兼顾传统用户习惯。

## 2. 核心变更
### 2.1 编辑器引擎切换
- **移除**：`md-editor-v3` 及其相关样式。
- **引入**：`Milkdown` 系列组件包。
    - `@milkdown/core`: 编辑器核心。
    - `@milkdown/preset-gfm`: GitHub 风格 Markdown 预设。
    - `@milkdown/vue`: Vue 3 集成。
    - `@milkdown/plugin-listener`: 实现内容同步与 v-model。
    - `@milkdown/plugin-history`: 撤销重做。
    - `@milkdown/plugin-slash`: 斜杠快捷菜单。
    - `@milkdown/plugin-prism`: 代码块高亮。

### 2.2 模式切换逻辑
我们将实现一个“视角切换器”，支持以下两种视图：
1. **沉浸模式 (Typora Mode)**：
   - 居中单栏布局。
   - 全 WYSIWYG 交互。
   - 点击已渲染块展示源码，离开后恢复预览。
2. **分栏模式 (Split Mode)**：
   - 左右双栏布局。
   - 左侧为 Markdown 源码编辑器。
   - 右侧为 Milkdown 渲染后的预览视图（只读）。

## 3. 技术实现细节
### 3.1 内容同步
使用 `pinia` 存储随笔内容。Milkdown 实例通过 `listener` 插件将内容实时同步到 `snap` store，确保在切换模式或离开页面时数据不丢失。

### 3.2 样式适配
- **深色模式**：对接系统现有的 `isDark` 逻辑，自动切换 Milkdown 的主题。
- **UI 集成**：将编辑器嵌入到现有的 `bg-surface-bg/50` 卡片中，移除 `md-editor-v3` 的自带边框。

## 4. 验证计划
- [ ] 验证 `v-model` 同步是否准确。
- [ ] 验证 `/` 斜杠菜单是否能正确唤起并执行命令。
- [ ] 验证双栏模式下左右滚动是否同步（或至少内容一致）。
- [ ] 验证深色/浅色模式切换时的样式表现。

## 5. 代码清理原则
- 执行完成后，`package.json` 中不再含有 `md-editor-v3`。
- `SnapNote.vue` 中不再含有旧版编辑器的逻辑代码。
