# 设计规格：修复设置页面中模型大小范围显示错误的 Bug

## 1. 背景与问题描述 (Background & Context)
在本地 AI 知识库桌面应用（Telepathy）中，用户可以在设置页面的模型管理器（ModelManager）中查看不同规格的推荐或可探索模型。
当查看多模态视觉模型（如 `LLaVA`）时，界面的大小范围被错误地显示为了 `0.6 GB ~ 8.0 GB`，而下方可选的规格为：
- `7B` 变体：大小为 `4.7 GB`
- `13B` 变体：大小为 `8.0 GB`

这个错误的起因是多模态视觉模型的变体列表（`variants`）中，除了包含 `7B` 和 `13B` 的主模型（GGUF），还包含了一起分发的辅助性**视觉投影器模型（mmproj/projector）**，其体积非常小（仅有约 0.6 GB）。

目前前端计算大小范围的 `getModelSizeRange` 工具函数在遍历变体时，未对 mmproj/projector 类型的变体进行过滤，导致将辅助性的投影器大小（0.6 GB）也视为主模型的大小下限，算出了不符合用户认知的 `0.6 GB ~ 8.0 GB` 大小区间。

## 2. 解决方案设计 (Proposed Solution)
采用**方案一：在前端计算大小时过滤 mmproj/projector 变体**。

我们直接在 `src/components/settings/ModelManager.vue` 中的 `getModelSizeRange` 逻辑中，增加针对 mmproj 和 projector 的排除过滤。这能与目前列表中“过滤并隐藏 mmproj/projector 变体”的行为完全对齐。

### 过滤规则
在分析变体 `variants` 的大小时，如果变体符合以下任一特征，则视为投影器，应予以排除：
1. `tag` 字段（转为小写）包含 `'mmproj'` 或 `'projector'`。
2. `params` 字段存在且（转为小写）包含 `'mmproj'` 或 `'projector'`。

### 目标结果
- `LLaVA` 的大小范围显示将排除 0.6 GB 的投影器，显示为真正的 `4.7 GB ~ 8.0 GB`。
- 其他无多模态配套文件的传统文本大模型（如 `Qwen2.5`）或向量模型展示保持正常，不受影响。

## 3. 详细代码变更 (Detailed Changes)

### 修改 [ModelManager.vue](file:///Users/mac/project/telepathy/src/components/settings/ModelManager.vue) 的 `getModelSizeRange` 函数：

```typescript
const getModelSizeRange = (variants: any[]) => {
  if (!variants || variants.length === 0) return '大小待定';
  
  // 过滤掉投影器 (projector/mmproj) 变体，仅计算主模型的大小范围
  const mainVariants = variants.filter(v => 
    !v.tag.toLowerCase().includes('mmproj') && 
    !v.tag.toLowerCase().includes('projector') &&
    !(v.params && v.params.toLowerCase().includes('mmproj')) &&
    !(v.params && v.params.toLowerCase().includes('projector'))
  );
  
  // 如果过滤后没有主模型（防呆，如单独的投影器模型），则回退使用原变体列表
  const targetVariants = mainVariants.length > 0 ? mainVariants : variants;
  const sizes = targetVariants.map(v => v.size).filter(s => s > 0);
  if (sizes.length === 0) return '大小待定';
  
  if (sizes.length === 1) return formatSize(sizes[0]);
  
  const minSize = Math.min(...sizes);
  const maxSize = Math.max(...sizes);
  return `${formatSize(minSize)} ~ ${formatSize(maxSize)}`;
};
```

## 4. 验证方案 (Verification Plan)
修改后，我们将运行验证步骤确保修改正确且无副作用：
1. **静态代码检查**：通过前端类型检查 (`pnpm tsc` 或 `npm run ts-check`）确保无 TypeScript 或语法错误。
2. **页面展示验证**：由于已有运行中的 tauri dev，前端修改将热重载（HMR）。确认 LLaVA 的大小范围变为 `4.7 GB ~ 8.0 GB`，而不是原来的 `0.6 GB ~ 8.0 GB`。
3. **其他模型范围**：随机点开一个非视觉模型（如 Qwen 或 BGE），确认其范围依然能够正常且准确地计算，无 NaN 或“大小待定”等错误显示。
