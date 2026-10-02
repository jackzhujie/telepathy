# 2026-05-17-model-manager-ui-optimization-design

## 1. 概述与目标 (Overview & Goals)

### 背景与现状
当前 Telepathy 项目的模型管理界面 (`src/views/Models.vue`) 和模型管理组件 (`src/components/settings/ModelManager.vue`) 在展示「为您推荐」模块时，采用了纵向三排大版面布局（分别是“最佳对话推荐”、“最佳向量推荐”和“最佳视觉推荐”）。每一排内部又横向并排展示了 3 个模型卡片。
此外，每个模型卡片内部**直接展开并列出了该模型的所有可用尺寸/量化规格**（如 Qwen2.5 的 1.5B/7B 各种版本）以及它们各自的硬件运行效率评估和下载按钮。这导致：
1. 页面纵向极长，信息密度过载，用户容易视觉疲劳。
2. 推荐卡片高度参差不齐，界面缺乏整齐感。

### 优化目标
1. **界面精简与紧凑化**：将「为您推荐」底部的三个横排（对话、向量、视觉）重构为**一行 3 列的并排布局**（1 Row, 3 Columns Grid）。
2. **渐进式信息呈现（折叠与展开）**：模型卡片默认保持极致紧凑的折叠状态，仅显示模型核心介绍、名称和大小范围。
3. **按需展开**：用户对某款模型感兴趣时，点击底部的展开按钮，才以平滑动画向下展开具体的参数尺寸变体（GGUF 规格）、硬件匹配度及下载入口。

---

## 2. 详细设计 (Detailed Design)

### 2.1 整体网格布局 (Grid Layout)
重构「为您推荐」区域 (`activeTab === 'recommend'`)，采用 Flex / Grid 混合布局。

```html
<div v-if="activeTab === 'recommend'" class="grid grid-cols-1 lg:grid-cols-3 gap-6 animate-fadeIn">
  <!-- 第一列：对话推荐 -->
  <div class="space-y-4">
    <div class="flex items-center gap-2 pb-2 border-b border-border-main/20">
      <div class="w-1.5 h-4 bg-brand rounded-full"></div>
      <h5 class="text-xs font-bold text-text-primary uppercase tracking-wider">对话推荐 (Chat Models)</h5>
    </div>
    ...
  </div>
  ...
</div>
```
* **响应式设计**：在移动端或小窗口下（`lg` 以下）自动回退为 1列纵向排列；在桌面端（`lg` 及以上）自动变为 3 列横向排列。

### 2.2 响应式折叠状态 (Expansion State)
我们在 `ModelManager.vue` 内部引入一个响应式状态 `expandedRecommendModels` 记录各模型的折叠状态：
```typescript
const expandedRecommendModels = ref<Record<string, boolean>>({});

const toggleRecommendExpand = (modelName: string) => {
  expandedRecommendModels.value[modelName] = !expandedRecommendModels.value[modelName];
};
```
该状态使用模型名称（`rec.model.name`）作为唯一键，确保各卡片独立控制，且支持多卡片同时展开。

### 2.3 极简卡片与折叠过渡 (Collapsible Card Style)
卡片结构调整如下：
* **头部 (Header)**：展示模型名称、大小范围。使用辅助函数 `getModelSizeRange(rec.model.variants)` 计算并显示该模型的大小区间（例如 `1.8 GB ~ 9.2 GB`）。
* **介绍 (Body)**：用一行极简文本介绍模型，配合 `line-clamp-2` 限制描述长度，保证卡片基础高度绝对整齐。
* **展开按钮与指示器 (Trigger & Arrow)**：卡片底部带有一个圆角按钮用于展开规格，并结合 Chevron 旋转动画指示当前状态。
* **折叠容器 (Accordion Container)**：使用 Tailwind 实现高平滑过渡：
  ```html
  <div 
    class="overflow-hidden transition-all duration-300 ease-in-out"
    :class="expandedRecommendModels[rec.model.name] ? 'max-h-[500px] opacity-100 mt-3' : 'max-h-0 opacity-0'"
  >
    <!-- 规格与下载列表 -->
  </div>
  ```

---

## 3. 实现计划与步骤 (Implementation Steps)

1. **备份并验证代码环境**：确保当前的 Tauri dev 状态正常。
2. **在 `<script setup>` 中新增响应式变量和逻辑**：
   * 定义 `expandedRecommendModels` 变量。
   * 定义 `toggleRecommendExpand` 触发函数。
3. **重构模板 `<template>` 结构**：
   * 修改 `activeTab === 'recommend'` 的外层容器为 3 列 Grid。
   * 将三大分类（Chat, Embedding, Vision）分别塞入这 3 列。
   * 移除原本各自独立的大区块标题和多列网格，改为一列内垂直排列。
4. **重构推荐卡片内部结构**：
   * 将模型变体循环展示（`rec.model.variants`）移动到折叠过渡容器中。
   * 引入“折叠/展开”交互触发器。
   * 细化折叠部分的动画与渐变展示。
5. **本地验证与调试**：
   * 观察 3 列在不同缩放比例下的响应式表现。
   * 验证折叠和展开的过渡是否平滑。
   * 确保点击“闪电安装”依然能正确触发后端下载与状态变更。

---

## 4. 验证方案 (Verification Plan)

### 界面与视觉表现
- [ ] 3 列在桌面端（宽高比正常）下均分宽度，列与列之间有 24px (gap-6) 的间距。
- [ ] 所有的模型卡片默认高度几乎完全统一（通常在 120px - 140px 左右），没有任何变体列表直接溢出。
- [ ] 点击「展开规格」时，规格列表伴随淡入和向下滑动滑出，没有突兀闪烁。
- [ ] 缩放窗口测试：窗口缩小时，布局在 lg 临界点优雅地回退为单列。

### 交互行为与状态
- [ ] 点击卡片 A 的展开，仅卡片 A 展开，卡片 B 和卡片 C 保持原样。
- [ ] 点击已经展开卡片的「收起规格」，列表平滑收回。
- [ ] 展开变体后，点击其内部的下载按钮（闪电图标），能正常下载并且在本地显示就绪状态。
