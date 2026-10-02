# 模型删除 Loading 状态设计文档

## 概述
当用户从“模型管理”中删除已安装的模型时，由于模型文件可能很大，文件系统的删除操作可能需要几秒钟时间。目前前端在发起删除请求后没有提供视觉反馈，用户可能会重复点击或误以为应用卡死。本项目旨在为删除按钮添加局部 Loading 状态。

## 目标
- 提供即时的删除反馈。
- 防止删除期间的重复点击和误操作。
- 优雅地处理删除成功和失败后的状态清理。

## 架构与逻辑改进

### 1. 状态管理 (`ModelManager.vue`)
- **新增变量**: `const isDeletingInProgress = ref<string | null>(null)`。
  - 存储当前正在执行底层删除 API 的模型完整名称 (`full_name`)。
  - 为 `null` 时表示没有任何模型正在执行实际删除。

### 2. 逻辑流更新
- **handleDelete(fullName)**:
  1. 设置 `isDeletingInProgress.value = fullName`。
  2. 使用 `try...finally` 包裹 `await store.removeModel(fullName)`。
  3. 在 `finally` 块中：
     - 重置 `isDeletingInProgress.value = null`。
     - 重置 `deletingModel.value = null` (退出确认删除 UI 状态)。

### 3. UI 表现层
- **确定删除按钮**:
  - 当 `isDeletingInProgress === model.full_name` 时：
    - 文本改为“正在删除...”。
    - 按钮左侧显示一个旋转的 Spinner 图标（基于 SVG 动画）。
    - 按钮设为 `disabled` 状态。
    - 添加透明度变淡的样式类 `opacity-50 cursor-not-allowed`。
- **取消按钮**:
  - 当 `isDeletingInProgress === model.full_name` 时被禁用，防止删除期间意外中断 UI 逻辑。

## 测试建议
- 模拟大文件删除，观察按钮文本是否变为“正在删除...”且出现 Loading。
- 在删除期间尝试再次点击“确定删除”或“取消”，验证是否已锁定交互。
- 验证删除完成后，该模型行是否从列表中消失，且状态变量是否正确清理。
