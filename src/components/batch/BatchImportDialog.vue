<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { useDocumentsStore } from '@/stores/documents';
import { useBackgroundTaskStore } from '@/stores/backgroundTask';
import { scanFolder } from '@/api/tauri';
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from 'reka-ui';

type Phase = 'idle' | 'importing' | 'parsing' | 'indexing' | 'complete';
type ItemPhase = 'pending' | 'importing' | 'imported' | 'parsing' | 'parsed' | 'indexing' | 'success' | 'failed';

interface ImportItem {
  id: string;
  path: string;
  name: string;
  docId?: string;
  phase: ItemPhase;
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
const backgroundTaskStore = useBackgroundTaskStore();
const items = ref<ImportItem[]>([]);
const phase = ref<Phase>('idle');
const isBackgroundRunning = ref(false);

const successCount = computed(() => items.value.filter(i => i.phase === 'success').length);
const failedCount = computed(() => items.value.filter(i => i.phase === 'failed').length);
const totalCount = computed(() => items.value.length);
const importedCount = computed(() => items.value.filter(i => ['imported', 'parsing', 'parsed', 'indexing', 'success'].includes(i.phase)).length);
const parsedCount = computed(() => items.value.filter(i => ['parsed', 'indexing', 'success'].includes(i.phase)).length);
const indexedCount = computed(() => items.value.filter(i => i.phase === 'success').length);

const progress = computed(() => {
  if (phase.value === 'importing') {
    const done = importedCount.value;
    return totalCount.value > 0 ? (done / totalCount.value) * 100 : 0;
  }
  if (phase.value === 'parsing') {
    const done = parsedCount.value;
    return totalCount.value > 0 ? (done / totalCount.value) * 100 : 0;
  }
  if (phase.value === 'indexing') {
    const done = indexedCount.value;
    return totalCount.value > 0 ? (done / totalCount.value) * 100 : 0;
  }
  if (phase.value === 'complete' || phase.value === 'idle') {
    return 100;
  }
  return 0;
});

const canClose = computed(() => phase.value === 'idle' || phase.value === 'complete');

const currentPhaseLabel = computed(() => {
  switch (phase.value) {
    case 'importing': return '正在导入文档';
    case 'parsing': return '正在解析文档';
    case 'indexing': return '正在索引文档';
    case 'complete': return '处理完成';
    default: return '批量导入文档';
  }
});

async function handleSelectFiles() {
  const selected = await openDialog({
    multiple: true,
    filters: [{
      name: 'Documents',
      extensions: ['pdf', 'md', 'txt', 'docx', 'xlsx', 'pptx', 'xls', 'ppt']
    }]
  });

  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    addItems(paths);
  }
}

async function handleSelectFolder() {
  const selected = await openDialog({
    directory: true
  });

  if (selected && typeof selected === 'string') {
    try {
      const files = await scanFolder(selected);
      if (files.length > 0) {
        addItems(files);
      }
    } catch (e) {
      console.error('扫描文件夹失败:', e);
    }
  }
}

function addItems(paths: string[]) {
  for (const path of paths) {
    const name = path.split('/').pop() || path;
    if (!items.value.find(i => i.path === path)) {
      items.value.push({
        id: crypto.randomUUID(),
        path,
        name,
        phase: 'pending'
      });
    }
  }
}

async function startProcess() {
  phase.value = 'importing';

  for (const item of items.value) {
    if (item.phase !== 'pending') continue;

    item.phase = 'importing';

    try {
      const doc = await documentsStore.addDocument(item.path, props.projectId);
      item.docId = doc.id;
      item.phase = 'imported';
    } catch (e) {
      item.phase = 'failed';
      item.error = String(e);
    }
  }

  phase.value = 'parsing';
  for (const item of items.value) {
    if (item.phase !== 'imported') continue;

    item.phase = 'parsing';

    try {
      await documentsStore.runParse(item.docId!);
      item.phase = 'parsed';
    } catch (e) {
      item.phase = 'failed';
      item.error = String(e);
    }
  }

  phase.value = 'indexing';
  for (const item of items.value) {
    if (item.phase !== 'parsed') continue;

    item.phase = 'indexing';

    try {
      await documentsStore.runIndex(item.docId!);
      item.phase = 'success';
      if (isBackgroundRunning.value) {
        backgroundTaskStore.incrementSuccess();
      }
    } catch (e) {
      item.phase = 'failed';
      item.error = String(e);
      if (isBackgroundRunning.value) {
        backgroundTaskStore.incrementFailed();
      }
    }
  }

  phase.value = 'complete';
  if (isBackgroundRunning.value) {
    backgroundTaskStore.completeTask();
    isBackgroundRunning.value = false;
  }
  emit('complete', successCount.value, failedCount.value);
}

async function retryItem(item: ImportItem) {
  if (item.phase === 'failed' && !item.docId) {
    item.phase = 'importing';
    try {
      const doc = await documentsStore.addDocument(item.path, props.projectId);
      item.docId = doc.id;
      item.phase = 'imported';
    } catch (e) {
      item.phase = 'failed';
      item.error = String(e);
    }
  }
}

function handleBackgroundIndex() {
  const pendingItems = items.value.filter(i => i.phase === 'parsed' || i.phase === 'indexing');
  backgroundTaskStore.startTask(pendingItems.length);
  isBackgroundRunning.value = true;
  emit('close');
}



function handleClose() {
  if (canClose.value || isBackgroundRunning.value) {
    emit('close');
  }
}

function removeItem(item: ImportItem) {
  const index = items.value.indexOf(item);
  if (index > -1) {
    items.value.splice(index, 1);
  }
}

function getPhaseLabel(item: ImportItem) {
  switch (item.phase) {
    case 'pending': return '⏳ 等待';
    case 'importing': return '⟳ 导入中';
    case 'imported': return '✓ 已导入';
    case 'parsing': return '⟳ 解析中';
    case 'parsed': return '✓ 已解析';
    case 'indexing': return '⟳ 索引中';
    case 'success': return '✓ 完成';
    case 'failed': return '✗ 失败';
    default: return '';
  }
}

function getPhaseColor(item: ImportItem) {
  switch (item.phase) {
    case 'pending': return 'text-text-muted';
    case 'importing': return 'text-brand';
    case 'imported': return 'text-success-500';
    case 'parsing': return 'text-brand';
    case 'parsed': return 'text-success-500';
    case 'indexing': return 'text-brand';
    case 'success': return 'text-success-500';
    case 'failed': return 'text-danger-500';
    default: return 'text-text-muted';
  }
}

function getFileEmoji(fileName: string): string {
  const ext = '.' + fileName.split('.').pop()?.toLowerCase();
  if (ext === '.pdf') return '📕';
  if (ext === '.md') return '📝';
  if (ext === '.docx' || ext === '.doc') return '📘';
  if (ext === '.xlsx' || ext === '.xls') return '📊';
  if (ext === '.pptx' || ext === '.ppt') return '📈';
  return '📄';
}

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

const unlistenFns = ref<UnlistenFn[]>([]);

async function setupTauriListeners() {
  cleanupTauriListeners();

  const u1 = await listen<any>('tauri://drag-enter', () => {
    if (phase.value === 'idle') {
      isDragging.value = true;
    }
  });

  const u2 = await listen<any>('tauri://drag-leave', () => {
    isDragging.value = false;
  });

  const u3 = await listen<any>('tauri://drag-drop', (event) => {
    isDragging.value = false;
    if (phase.value !== 'idle') return;

    const paths = event.payload?.paths;
    if (Array.isArray(paths) && paths.length > 0) {
      addItems(paths);
    }
  });

  const u4 = await listen<any>('tauri://file-drop', (event) => {
    isDragging.value = false;
    if (phase.value !== 'idle') return;

    const paths = Array.isArray(event.payload) ? event.payload : (event.payload as any)?.paths;
    if (Array.isArray(paths) && paths.length > 0) {
      addItems(paths);
    }
  });

  unlistenFns.value = [u1, u2, u3, u4];

  // 为 window 全局范围挂载 HTML5 拖放事件
  window.addEventListener('dragover', handleDragOver);
  window.addEventListener('dragleave', handleDragLeave);
  window.addEventListener('drop', handleDrop);
}

function cleanupTauriListeners() {
  for (const unlisten of unlistenFns.value) {
    unlisten();
  }
  unlistenFns.value = [];

  window.removeEventListener('dragover', handleDragOver);
  window.removeEventListener('dragleave', handleDragLeave);
  window.removeEventListener('drop', handleDrop);
}

watch(() => props.open, (newVal) => {
  if (newVal) {
    setupTauriListeners();
  } else {
    cleanupTauriListeners();
    if (!isBackgroundRunning.value) {
      items.value = [];
      phase.value = 'idle';
    }
  }
}, { immediate: true });

onUnmounted(() => {
  cleanupTauriListeners();
});

const isDragging = ref(false);

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (phase.value === 'idle') {
    isDragging.value = true;
  }
}

function handleDragLeave(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
  if (phase.value !== 'idle') return;

  if (e.dataTransfer && e.dataTransfer.files) {
    const filesArray = Array.from(e.dataTransfer.files);
    const paths = filesArray.map((f: any) => f.path).filter(Boolean);
    if (paths.length > 0) {
      addItems(paths);
    }
  }
}
</script>

<template>
  <DialogRoot :open="open" @update:open="handleClose">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 bg-black/50 z-50" />
      <DialogContent 
        @dragover="handleDragOver" 
        @dragleave="handleDragLeave" 
        @drop="handleDrop" 
        :class="[isDragging ? 'border-brand/60 bg-brand/5 ring-4 ring-brand/10' : 'border-border-main/35 bg-panel-bg/65']"
        class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-md backdrop-blur-md border rounded-2xl p-6 shadow-2xl z-50 max-h-[75vh] flex flex-col animate-fade-in transition-all duration-300 select-none"
      >
        <DialogTitle class="text-base font-black mb-1.5 bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary select-none tracking-tight">{{ currentPhaseLabel }}</DialogTitle>
        <DialogDescription class="text-xs text-text-secondary/80 mb-4 select-none">
          <span v-if="phase === 'idle'">选择文件或文件夹导入文档，支持 PDF、Markdown、TXT、DOCX 格式</span>
          <span v-else-if="phase === 'importing'">正在导入文档到项目...</span>
          <span v-else-if="phase === 'parsing'">正在解析文档内容...</span>
          <span v-else-if="phase === 'indexing'">正在索引文档以便检索...</span>
          <span v-else-if="phase === 'complete'">
            成功 {{ successCount }} 个
            <span v-if="failedCount > 0" class="text-danger-500">，失败 {{ failedCount }} 个</span>
          </span>
        </DialogDescription>

        <div v-if="phase === 'idle'" class="flex gap-3 mb-4 select-none">
          <button
            @click="handleSelectFiles"
            class="flex-1 px-3.5 py-2 border border-border-main/40 rounded-xl hover:bg-surface-bg/60 hover:border-border-main/60 text-text-secondary text-xs font-bold transition-all duration-300 flex items-center justify-center gap-2 cursor-pointer shadow-sm active:scale-98"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" class="text-text-muted">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
            </svg>
            选择文件
          </button>
          <button
            @click="handleSelectFolder"
            class="flex-1 px-3.5 py-2 border border-border-main/40 rounded-xl hover:bg-surface-bg/60 hover:border-border-main/60 text-text-secondary text-xs font-bold transition-all duration-300 flex items-center justify-center gap-2 cursor-pointer shadow-sm active:scale-98"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" class="text-text-muted">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            选择文件夹
          </button>
        </div>

        <div v-if="items.length > 0" class="flex-1 overflow-hidden flex flex-col min-h-0">
          <div class="border border-border-main/50 rounded-md flex-1 overflow-y-auto max-h-56">
            <div
              v-for="item in items"
              :key="item.id"
              class="flex items-center gap-2 p-2.5 border-b border-border-main/50 last:border-b-0"
            >
              <span class="text-sm flex-shrink-0 transition-all duration-300">
                <svg v-if="['importing', 'parsing', 'indexing'].includes(item.phase)" class="animate-spin h-4 w-4 text-brand" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                <span v-else>
                  {{ getFileEmoji(item.name) }}
                </span>
              </span>
              <span class="flex-1 truncate text-xs">{{ item.name }}</span>

              <span :class="getPhaseColor(item)" class="text-xs">{{ getPhaseLabel(item) }}</span>

              <button
                v-if="item.phase === 'failed'"
                @click="retryItem(item)"
                class="text-xs text-brand hover:underline"
              >
                重试
              </button>

              <button
                v-if="item.phase === 'pending' && phase === 'idle'"
                @click="removeItem(item)"
                class="text-text-muted hover:text-danger-500"
              >
                &times;
              </button>
            </div>
          </div>

          <div class="mt-3">
            <div class="h-2 bg-surface-bg rounded-full overflow-hidden">
              <div
                class="h-full bg-brand transition-all duration-300"
                :style="{ width: `${progress}%` }"
              ></div>
            </div>
            <p class="text-xs text-text-secondary mt-1.5">
              <span v-if="phase === 'importing'">{{ importedCount }} / {{ totalCount }} 已导入</span>
              <span v-else-if="phase === 'parsing'">{{ parsedCount }} / {{ totalCount }} 已解析</span>
              <span v-else-if="phase === 'indexing'">{{ indexedCount }} / {{ totalCount }} 已索引</span>
              <span v-else-if="phase === 'complete'">处理完成</span>
              <span v-else>{{ totalCount }} 个文件</span>
              <span v-if="failedCount > 0 && phase !== 'idle'" class="text-danger-500">，{{ failedCount }} 个失败</span>
            </p>
          </div>
        </div>

        <div v-else-if="phase === 'idle'" class="flex-1 flex items-center justify-center text-text-secondary">
          <p class="text-xs">点击上方按钮选择文件或文件夹</p>
        </div>

        <div class="flex justify-end gap-2.5 mt-4 pt-3 border-t border-border-main/25">
          <button
            v-if="phase === 'indexing'"
            @click="handleBackgroundIndex"
            class="px-3.5 py-2 border border-border-main/40 hover:bg-surface-bg/50 text-text-secondary rounded-xl text-xs font-bold transition-all duration-300 shadow-sm cursor-pointer active:scale-98"
          >
            后台运行
          </button>
          <button
            @click="handleClose"
            :disabled="!canClose && !isBackgroundRunning"
            class="px-3.5 py-2 border border-border-main/40 hover:bg-surface-bg/50 text-text-secondary rounded-xl text-xs font-bold transition-all duration-300 shadow-sm disabled:opacity-50 cursor-pointer active:scale-98"
          >
            {{ phase === 'idle' ? '取消' : phase === 'complete' ? '关闭' : '处理中...' }}
          </button>
          <button
            v-if="phase === 'idle'"
            @click="startProcess"
            :disabled="items.length === 0"
            class="px-3.5 py-2 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 text-white rounded-xl text-xs font-bold transition-all hover:scale-[1.02] active:scale-98 disabled:opacity-50 shadow-md cursor-pointer"
          >
            开始处理
          </button>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>