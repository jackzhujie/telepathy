# KeepAlive 缓存导致的文档列表刷新问题 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 解决在知识库中删除文档后，回到被 KeepAlive 缓存的“文档管理”页面时文档列表未刷新同步的问题。

**Architecture:** 引入 Vue 的 `onActivated` 生命周期钩子。在缓存页面再次激活（例如路由返回、重新进入）时，重新获取最新的项目列表并刷新当前选中项目的文档列表，保证数据最新。

**Tech Stack:** Vue 3.5 + TypeScript + Vite

---

### Task 1: 引入 onActivated 生命周期钩子并注册激活刷新事件

**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 在 `Documents.vue` 中引入 `onActivated` 钩子**

  找到第 3 行，将其修改为包含 `onActivated` 的引入：
  ```ts
  import { ref, onMounted, onActivated, computed } from 'vue';
  ```

- [ ] **Step 2: 注册 `onActivated` 逻辑，在组件被激活时刷新数据**

  在组件 `onMounted` 钩子下方（第 216 行后）添加以下内容：
  ```ts
  onActivated(async () => {
    // 重新拉取最新的项目列表
    await projectsStore.fetchProjects();
    // 如果已有选中的项目 ID，重新获取该项目的分页文档列表；否则获取首个项目的分页文档列表
    if (selectedProjectId.value) {
      await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
    } else if (projectsStore.projects.length > 0) {
      selectedProjectId.value = projectsStore.projects[0].id;
      await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
    }
  });
  ```

- [ ] **Step 3: 使用 `tsc` 验证 TypeScript 编译没有异常**

  Run: `pnpm tsc --noEmit` 或相关命令确保前端代码无异常。

- [ ] **Step 4: 任务完成提交**
