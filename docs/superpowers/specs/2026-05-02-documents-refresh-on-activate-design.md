# 2026-05-02 解决 KeepAlive 导致文档列表不刷新设计文档

## 1. 概述
在知识库中删除文档后，返回到“文档管理”页面（`Documents.vue`）时，由于 `Documents.vue` 被 `KeepAlive` 缓存，`onMounted` 不会重新执行，导致被删除的文档仍然在页面上显示。

## 2. 解决方案：使用 onActivated 钩子

在 `Documents.vue` 中引入 `onActivated` 生命周期钩子。当缓存的页面再次被激活（用户从其他页面返回）时，重新获取最新的项目列表和文档数据，确保状态同步。

### 代码修改计划：
在 `src/views/Documents.vue` 的 `<script setup>` 中：
1. 从 `vue` 引入 `onActivated`。
2. 注册 `onActivated`，在其中拉取项目列表及最新分页文档。

```typescript
import { ref, onMounted, onActivated, computed } from 'vue';

// ... 其他代码 ...

onActivated(async () => {
  await projectsStore.fetchProjects();
  if (selectedProjectId.value) {
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
  } else if (projectsStore.projects.length > 0) {
    selectedProjectId.value = projectsStore.projects[0].id;
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
  }
});
```

## 3. 验收标准
- 用户在“知识库”中删除文档后，通过侧边栏或返回按钮回到“文档管理”页面。
- “文档管理”页面中的文档列表能自动更新，被删除的文档不再展示。
