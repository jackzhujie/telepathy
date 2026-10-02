# 同步 Milkdown 编辑器与稳定性修复实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 `.worktrees/snapnote-milkdown` 中的 Milkdown 编辑器集成及所有稳定性修复同步到主分支，解决“不实时”的问题。

**Architecture:** 采用直接文件覆盖同步的方式，确保主分支代码与已验证的工作树代码一致。随后重新安装依赖并验证构建。

**Tech Stack:** Milkdown (WYSIWYG), Vite, Vue 3, pnpm, Tauri v2.

---

### Task 1: 同步核心配置与依赖

**Files:**
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`

- [ ] **Step 1: 备份主分支 package.json (可选但推荐)**
```bash
cp package.json package.json.bak
```

- [ ] **Step 2: 从工作树覆盖 package.json 和 pnpm-lock.yaml**
```bash
cp .worktrees/snapnote-milkdown/package.json ./package.json
cp .worktrees/snapnote-milkdown/pnpm-lock.yaml ./pnpm-lock.yaml
```

- [ ] **Step 3: 提交更改**
```bash
git add package.json pnpm-lock.yaml
git commit -m "chore: update dependencies for Milkdown integration"
```

### Task 2: 同步新增组件与目录

**Files:**
- Create: `src/components/editor/MilkdownEditor.vue`

- [ ] **Step 1: 创建目标目录**
```bash
mkdir -p src/components/editor
```

- [ ] **Step 2: 复制新增组件**
```bash
cp .worktrees/snapnote-milkdown/src/components/editor/MilkdownEditor.vue src/components/editor/MilkdownEditor.vue
```

- [ ] **Step 3: 提交更改**
```bash
git add src/components/editor/
git commit -m "feat: add MilkdownEditor component"
```

### Task 3: 同步视图与 Store 文件

**Files:**
- Modify: `src/views/SnapNote.vue`
- Modify: `src/components/settings/ModelManager.vue`
- Modify: `src/views/Chat.vue`
- Modify: `src/views/Settings.vue`
- Modify: `src/views/KnowledgeBase.vue`
- Modify: `src/views/Profile.vue`
- Modify: `src/components/layout/NotificationPanel.vue`
- Modify: `src/stores/settings.ts`
- Modify: `src/stores/documents.ts`
- Modify: `src/stores/knowledge.ts`

- [ ] **Step 1: 批量覆盖视图和组件文件**
```bash
cp .worktrees/snapnote-milkdown/src/views/SnapNote.vue src/views/SnapNote.vue
cp .worktrees/snapnote-milkdown/src/components/settings/ModelManager.vue src/components/settings/ModelManager.vue
cp .worktrees/snapnote-milkdown/src/views/Chat.vue src/views/Chat.vue
cp .worktrees/snapnote-milkdown/src/views/Settings.vue src/views/Settings.vue
cp .worktrees/snapnote-milkdown/src/views/KnowledgeBase.vue src/views/KnowledgeBase.vue
cp .worktrees/snapnote-milkdown/src/views/Profile.vue src/views/Profile.vue
cp .worktrees/snapnote-milkdown/src/components/layout/NotificationPanel.vue src/components/layout/NotificationPanel.vue
```

- [ ] **Step 2: 批量覆盖 Store 文件**
```bash
cp .worktrees/snapnote-milkdown/src/stores/settings.ts src/stores/settings.ts
cp .worktrees/snapnote-milkdown/src/stores/documents.ts src/stores/documents.ts
cp .worktrees/snapnote-milkdown/src/stores/knowledge.ts src/stores/knowledge.ts
```

- [ ] **Step 3: 提交更改**
```bash
git add src/views/ src/components/ src/stores/
git commit -m "feat: sync views and stores with stability fixes and Milkdown integration"
```

### Task 4: 依赖安装与最终验证

**Files:**
- N/A

- [ ] **Step 1: 安装依赖**
```bash
pnpm install
```

- [ ] **Step 2: 验证项目构建**
```bash
pnpm build
```

- [ ] **Step 3: 检查代码质量 (可选)**
```bash
vue-tsc --noEmit
```

- [ ] **Step 4: 提交依赖更新后的 lock 文件 (如果有变化)**
```bash
git add pnpm-lock.yaml
git commit -m "chore: finalize dependencies sync" || true
```
