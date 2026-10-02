# 批量导入功能实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现批量导入功能，支持多文件选择和文件夹导入，显示详细进度列表

**Architecture:** 前端新增 BatchImportDialog 组件，复用现有 import_document API，Documents.vue 添加下拉导入菜单

**Tech Stack:** Vue 3 + TypeScript + reka-ui + Tauri

---

## 文件结构

```
src/
├── components/
│   └── batch/
│       └── BatchImportDialog.vue    # 新建：批量导入弹窗
├── views/
│   └── Documents.vue                # 修改：添加导入下拉菜单
└── stores/
    └── documents.ts                 # 修改：添加批量导入方法
```

---

## Task 1: 创建 BatchImportDialog.vue 组件

**Files:**
- Create: `src/components/batch/BatchImportDialog.vue`

- [ ] **Step 1: 创建组件基础结构**

```vue
<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useDocumentsStore } from '@/stores/documents';
import { open } from '@tauri-apps/plugin-dialog';
import { Document } from '@/api/tauri';

interface ImportItem {
  id: string;
  path: string;
  name: string;
  size: number;
  status: 'pending' | 'importing' | 'success' | 'failed';
  error?: string;
}

const props = defineProps<{
  open: boolean;
  projectId: string;
}>();

const emit = defineEmits<{
  close: [];
  complete: [success: number, failed: number];
}>();

const documentsStore = useDocumentsStore();
const items = ref<ImportItem[]>([]);
const isImporting = ref(false);

const successCount = computed(() => items.value.filter(i => i.status === 'success').length);
const failedCount = computed(() => items.value.filter(i => i.status === 'failed').length);
const totalCount = computed(() => items.value.length);
const progress = computed(() => {
  const done = successCount.value + failedCount.value;
  return totalCount.value > 0 ? (done / totalCount.value) * 100 : 0;
});

const isComplete = computed(() => {
  return items.value.length > 0 && items.value.every(i => i.status !== 'pending' && i.status !== 'importing');
});

async function handleSelectFiles() {
  const selected = await open({
    multiple: true,
    filters: [{
      name: 'Documents',
      extensions: ['pdf', 'md', 'txt', 'docx']
    }]
  });

  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    addItems(paths);
  }
}

async function handleSelectFolder() {
  const selected = await open({
    directory: true
  });

  if (selected) {
    // TODO: 递归扫描文件夹
  }
}

function addItems(paths: string[]) {
  for (const path of paths) {
    const name = path.split('/').pop() || path;
    items.value.push({
      id: crypto.randomUUID(),
      path,
      name,
      size: 0,
      status: 'pending'
    });
  }
}

async function startImport() {
  isImporting.value = true;

  for (const item of items.value) {
    if (item.status !== 'pending') continue;

    item.status = 'importing';

    try {
      await documentsStore.addDocument(item.path, props.projectId);
      item.status = 'success';
    } catch (e) {
      item.status = 'failed';
      item.error = String(e);
    }
  }

  isImporting.value = false;

  if (isComplete.value) {
    emit('complete', successCount.value, failedCount.value);
  }
}

async function retryItem(item: ImportItem) {
  item.status = 'importing';
  item.error = undefined;

  try {
    await documentsStore.addDocument(item.path, props.projectId);
    item.status = 'success';
  } catch (e) {
    item.status = 'failed';
    item.error = String(e);
  }
}

function handleClose() {
  if (!isImporting.value) {
    emit('close');
  }
}

watch(() => props.open, (newVal) => {
  if (!newVal) {
    items.value = [];
    isImporting.value = false;
  }
});
</script>

<template>
  <DialogRoot :open="open" @update:open="handleClose">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 bg-black/50 z-50" />
      <DialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-lg bg-white rounded-xl shadow-2xl z-50 p-6">
        <DialogTitle class="text-xl font-bold mb-4">批量导入文档</DialogTitle>

        <div class="space-y-4">
          <!-- 按钮区 -->
          <div class="flex gap-2">
            <button @click="handleSelectFiles" class="flex-1 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              选择文件
            </button>
            <button @click="handleSelectFolder" class="flex-1 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              选择文件夹
            </button>
          </div>

          <!-- 文件列表 -->
          <div v-if="items.length > 0" class="border rounded-lg max-h-64 overflow-y-auto">
            <div v-for="item in items" :key="item.id" class="flex items-center gap-3 p-3 border-b last:border-b-0">
              <span class="text-lg">📄</span>
              <span class="flex-1 truncate">{{ item.name }}</span>

              <!-- 状态 -->
              <span v-if="item.status === 'pending'" class="text-gray-400">⏳</span>
              <span v-else-if="item.status === 'importing'" class="text-blue-500 animate-spin">⟳</span>
              <span v-else-if="item.status === 'success'" class="text-green-500">✓</span>
              <span v-else-if="item.status === 'failed'" class="text-red-500">✗</span>

              <!-- 重试按钮 -->
              <button v-if="item.status === 'failed'" @click="retryItem(item)" class="text-sm text-blue-500 hover:underline">
                重试
              </button>
            </div>
          </div>

          <!-- 进度条 -->
          <div v-if="items.length > 0">
            <div class="h-2 bg-gray-200 rounded-full overflow-hidden">
              <div class="h-full bg-blue-500 transition-all" :style="{ width: `${progress}%` }"></div>
            </div>
            <p class="text-sm text-gray-500 mt-1">
              {{ successCount + failedCount }} / {{ totalCount }} 个文件
            </p>
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
          <button @click="handleClose" :disabled="isImporting" class="px-4 py-2 border rounded-lg hover:bg-gray-50 disabled:opacity-50">
            取消
          </button>
          <button
            @click="startImport"
            :disabled="items.length === 0 || isImporting"
            class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
          >
            {{ isImporting ? '导入中...' : '开始导入' }}
          </button>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
```

---

## Task 2: 修改 Documents.vue 添加导入下拉菜单

**Files:**
- Modify: `src/views/Documents.vue`

- [ ] **Step 1: 添加导入下拉菜单逻辑**

在 template 中找到导入按钮，替换为：

```vue
<DropdownMenu>
  <DropdownMenuTrigger asChild>
    <button class="px-4 py-2 bg-cta text-white rounded-lg hover:bg-cta/90 flex items-center gap-2">
      <span>导入文档</span>
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="6 9 12 15 18 9"></polyline>
      </svg>
    </button>
  </DropdownMenuTrigger>
  <DropdownMenuPortal>
    <DropdownMenuContent class="w-48">
      <DropdownMenuItem @select="openImportDialog('files')">
        选择文件
      </DropdownMenuItem>
      <DropdownMenuItem @select="openImportDialog('folder')">
        选择文件夹
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenuPortal>
</DropdownMenu>
```

- [ ] **Step 2: 添加 BatchImportDialog**

```vue
<BatchImportDialog
  :open="showBatchImport"
  :project-id="selectedProjectId || ''"
  @close="showBatchImport = false"
  @complete="handleImportComplete"
/>
```

- [ ] **Step 3: 添加相关状态和方法**

```typescript
const showBatchImport = ref(false);
const importType = ref<'files' | 'folder'>('files');

function openImportDialog(type: 'files' | 'folder') {
  importType.value = type;
  showBatchImport.value = true;
}

function handleImportComplete(success: number, failed: number) {
  showBatchImport.value = false;
  showToast(`导入完成：成功 ${success} 个，失败 ${failed} 个`);
  if (selectedProjectId.value) {
    documentsStore.fetchDocuments(selectedProjectId.value);
  }
}
```

---

## Task 3: 添加文件夹扫描功能（后端）

**Files:**
- Modify: `src-tauri/src/commands/document.rs` - 添加扫描文件夹命令

- [ ] **Step 1: 添加 scan_folder 命令**

```rust
#[tauri::command]
pub async fn scan_folder(
    folder_path: String,
) -> Result<Vec<String>, AppError> {
    let mut files = Vec::new();
    let supported_extensions = ["pdf", "md", "txt", "docx"];

    fn scan_dir(dir: &Path, files: &mut Vec<String>, extensions: &[&str]) -> Result<(), AppError> {
        for entry in fs::read_dir(dir).map_err(|e| AppError::Internal(e.to_string()))? {
            let entry = entry.map_err(|e| AppError::Internal(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                scan_dir(&path, files, extensions)?;
            } else if let Some(ext) = path.extension() {
                if extensions.contains(&ext.to_str().unwrap_or("").to_lowercase().as_str()) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
        Ok(())
    }

    scan_dir(Path::new(&folder_path), &mut files, &supported_extensions)?;
    Ok(files)
}
```

- [ ] **Step 2: 在 lib.rs 注册命令**

```rust
commands::document::scan_folder,
```

- [ ] **Step 3: 前端添加 API 调用**

```typescript
export const scanFolder = (folderPath: string): Promise<string[]> =>
  invoke('scan_folder', { folderPath });
```

---

## Task 4: 构建验证

- [ ] **Step 1: 运行 TypeScript 类型检查**

```bash
cd /Users/mac/project/telepathy
npm run typecheck
```

- [ ] **Step 2: 运行构建**

```bash
npm run build
```

- [ ] **Step 3: 测试 Rust 编译**

```bash
cd src-tauri
cargo build
```

---

## Task 5: Git 提交

- [ ] **提交代码**

```bash
git add -A
git commit -m "feat: 添加批量导入功能

- 新增 BatchImportDialog 组件
- 支持多文件选择和文件夹导入
- 显示导入进度列表和状态
- 失败文件支持手动重试"
```
