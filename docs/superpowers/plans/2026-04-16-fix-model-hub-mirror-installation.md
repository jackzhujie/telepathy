# 模型库镜像安装修复实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复国内环境下通过镜像搜索安装模型失败的问题，确保 library 命名空间被正确补全。

**Architecture:** 在 Pinia store 中改进 `installNewModel` 的模型 ID 生成逻辑，如果使用镜像且模型属于官方库（无斜杠），则自动补全 `library/` 前缀。

**Tech Stack:** Vue 3 + Pinia + TypeScript

---

### Task 1: 改进 `src/stores/settings.ts` 中的镜像逻辑

**Files:**
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: 修改 `installNewModel` 函数中的模型 ID 拼接逻辑**

```typescript
// 修改前 (约 250 行)：
const finalModelId = bestMirrorPrefix.value + modelId;

// 修改后：
const finalModelId = bestMirrorPrefix.value 
  ? (modelId.includes('/') ? bestMirrorPrefix.value + modelId : `${bestMirrorPrefix.value}library/${modelId}`) 
  : modelId;
```

- [ ] **Step 2: 验证编译**
运行 `pnpm tauri build` 确保前端代码编译无误（仅检查语法）。

- [ ] **Step 3: 提交代码**

```bash
git add src/stores/settings.ts
git commit -m "fix(models): auto-prepend library namespace when using mirror for hub models"
```

### Task 2: 增强 `pullModel` 处理逻辑

为了保持一致性，手动输入的拉取逻辑也应支持镜像。

**Files:**
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: 修改 `pullModel` 函数支持镜像**

```typescript
// 修改前 (约 155 行)：
async function pullModel(model: string) {
  // ...
  await pullModelCmd(model);
  // ...
}

// 修改后：
async function pullModel(model: string) {
  // ...
  const finalModelId = bestMirrorPrefix.value 
    ? (model.includes('/') ? bestMirrorPrefix.value + model : `${bestMirrorPrefix.value}library/${model}`) 
    : model;

  console.log('[Pull] Starting:', finalModelId);
  await pullModelCmd(finalModelId);
  // ...
}
```

- [ ] **Step 2: 提交代码**

```bash
git add src/stores/settings.ts
git commit -m "feat(models): support mirroring in manual pullModel as well"
```
