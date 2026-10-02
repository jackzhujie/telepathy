<script setup lang="ts">
defineOptions({ name: 'Documents' });
import { ref, onMounted, onActivated, computed } from 'vue';
import type { Document } from '@/types/document';
import { useDocumentsStore } from '@/stores/documents';
import { useProjectsStore } from '@/stores/projects';
import { useVirtualizer } from '@tanstack/vue-virtual';
import { useInfiniteScroll } from '@/hooks/useInfiniteScroll';
import type { Project, CreateProjectInput, UpdateProjectInput } from '@/types/project';
import { isSidecarInstalled } from '@/api/tauri';
import { useSettingsStore } from '@/stores/settings';
import BatchImportDialog from '@/components/batch/BatchImportDialog.vue';
import {
  SelectContent,
  SelectIcon,
  SelectItem,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  Separator,
  ToastClose,
  ToastDescription,
  ToastProvider,
  ToastRoot,
  ToastViewport,
  TooltipArrow,
  TooltipContent,
  TooltipPortal,
  TooltipRoot,
  TooltipTrigger,
} from 'reka-ui';

const documentsStore = useDocumentsStore();
const projectsStore = useProjectsStore();

import { useNotificationsStore } from '@/stores/notifications';
const notificationsStore = useNotificationsStore();
const settingsStore = useSettingsStore();

const sidecarInstalled = ref(false);
const isBannerDismissed = ref(false);
const showOfficeBanner = computed(() => {
  return !settingsStore.settings.enable_office_parser || settingsStore.settings.enable_office_parser !== 'true';
});
const showBatchImport = ref(false);
const selectedProjectId = ref<string | null>(null);
const showProjectModal = ref(false);
const editingProject = ref<Project | null>(null);
const projectForm = ref({ name: '', description: '' });
const showDeleteConfirm = ref(false);
const projectToDelete = ref<Project | null>(null);
const toastMessages = ref<{ id: number; message: string }[]>([]);
let toastId = 0;

// 虚拟滚动配置
const docListContainer = ref<HTMLElement | null>(null);
const documentsRef = computed(() => documentsStore.documentsPaginated);

const virtualizer = useVirtualizer(computed(() => ({
  count: documentsRef.value.length,
  getScrollElement: () => docListContainer.value,
  estimateSize: () => 80,
  overscan: 5,
})));

// 无限滚动配置
const { target } = useInfiniteScroll({
  loadMore: () => documentsStore.loadMoreDocuments(selectedProjectId.value || undefined),
  isLoading: () => documentsStore.isLoadingMore,
  hasMore: () => documentsStore.documentsHasMore,
  rootMargin: '200px'
});

const showToast = (message: string) => {
  const id = ++toastId;
  toastMessages.value.push({ id, message });
  setTimeout(() => {
    toastMessages.value = toastMessages.value.filter(t => t.id !== id);
  }, 3000);
};

const statusMap: Record<string, { class: string; text: string }> = {
  pending: { class: 'bg-surface-bg text-text-muted', text: '待解析' },
  parsing: { class: 'bg-brand/15 text-brand', text: '解析中' },
  done: { class: 'bg-success-500/15 text-success-500', text: '已就绪' },
  error: { class: 'bg-danger-500/15 text-danger-500', text: '失败' },
  indexed: { class: 'bg-brand/15 text-brand', text: '已索引' },
};

const hasProjects = computed(() => projectsStore.projects.length > 0);

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

async function handleProjectChange() {
  if (selectedProjectId.value) {
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
  }
}

async function handleParse(docId: string) {
  try {
    await documentsStore.runParse(docId);
    showToast('解析完成');
  } catch (e) {
    showToast(`解析失败: ${e}`);
  }
}


const showDeleteDocConfirm = ref(false);
const docToDelete = ref<Document | null>(null);

function confirmDeleteDoc(doc: Document) {
  docToDelete.value = doc;
  showDeleteDocConfirm.value = true;
}

async function handleDeleteDoc() {
  if (!docToDelete.value) return;
  try {
    await documentsStore.removeDocument(docToDelete.value.id);
    showToast('已删除');
    showDeleteDocConfirm.value = false;
    docToDelete.value = null;
  } catch (e) {
    showToast(`删除失败: ${e}`);
  }
}

async function handleIndex(docId: string) {
  try {
    const result = await documentsStore.runIndex(docId);
    console.log(`索引完成: ${result.chunk_count} 个文本块`);
    notificationsStore.playDebouncedSound();
  } catch (e) {
    console.error(`索引失败: ${e}`);
  }
}

async function handleImportComplete(success: number, failed: number) {
  showBatchImport.value = false;
  if (failed > 0) {
    showToast(`导入完成：成功 ${success} 个，失败 ${failed} 个`);
  } else {
    showToast(`成功导入 ${success} 个文档`);
  }
  if (success > 0) {
    notificationsStore.playDebouncedSound();
  }
  if (selectedProjectId.value) {
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
  }
}

function openCreateProject() {
  editingProject.value = null;
  projectForm.value = { name: '', description: '' };
  showProjectModal.value = true;
}

function openEditProject(project: Project) {
  editingProject.value = project;
  projectForm.value = { name: project.name, description: project.description || '' };
  showProjectModal.value = true;
}

function confirmDeleteProject(project: Project) {
  projectToDelete.value = project;
  showDeleteConfirm.value = true;
}

async function handleDeleteProject() {
  if (!projectToDelete.value) return;
  
  try {
    await projectsStore.deleteProject(projectToDelete.value.id);
    if (selectedProjectId.value === projectToDelete.value.id) {
      selectedProjectId.value = projectsStore.projects[0]?.id || null;
      if (selectedProjectId.value) {
        await documentsStore.fetchDocuments(selectedProjectId.value);
      }
    }
    showToast('项目已删除');
    showDeleteConfirm.value = false;
    projectToDelete.value = null;
  } catch (e) {
    showToast(`删除失败: ${e}`);
  }
}

async function handleSaveProject() {
  if (!projectForm.value.name.trim()) {
    showToast('请输入项目名称');
    return;
  }

  try {
    if (editingProject.value) {
      const input: UpdateProjectInput = {
        id: editingProject.value.id,
        name: projectForm.value.name,
        description: projectForm.value.description || undefined,
      };
      await projectsStore.updateProject(input);
      showToast('项目已更新');
    } else {
      const input: CreateProjectInput = {
        name: projectForm.value.name,
        description: projectForm.value.description || undefined,
      };
      const newProject = await projectsStore.createProject(input);
      selectedProjectId.value = newProject.id;
      await documentsStore.fetchDocuments(newProject.id);
      showToast('项目已创建');
    }
    showProjectModal.value = false;
  } catch (e) {
    showToast(`保存失败: ${e}`);
  }
}

onMounted(async () => {
  await projectsStore.fetchProjects();
  if (projectsStore.projects.length > 0) {
    selectedProjectId.value = projectsStore.projects[0].id;
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
  }
  isSidecarInstalled().then((v) => (sidecarInstalled.value = v));
  await settingsStore.fetchSettings();
});

onActivated(async () => {
  await projectsStore.fetchProjects();
  if (selectedProjectId.value) {
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
  } else if (projectsStore.projects.length > 0) {
    selectedProjectId.value = projectsStore.projects[0].id;
    await documentsStore.fetchDocumentsPaginated(selectedProjectId.value);
  }
});
</script>

<template>
  <div class="flex flex-col h-full gap-4 px-1 pb-6 select-text bg-app-bg/40 backdrop-blur-md">
    <ToastProvider swipe-direction="right">
        <ToastRoot
          v-for="t in toastMessages"
          :key="t.id"
          :duration="3000"
          class="fixed bottom-6 right-6 px-4 py-3 bg-panel-bg/75 backdrop-blur-md border border-border-main/40 text-text-primary text-xs rounded-xl shadow-xl z-50 flex items-center gap-2 animate-fade-in"
        >
          <ToastDescription>{{ t.message }}</ToastDescription>
          <ToastClose class="hover:opacity-80">×</ToastClose>
        </ToastRoot>
        <ToastViewport />
      </ToastProvider>

    <div 
      class="flex justify-between items-center pb-6 border-b border-border-main/35"
    >
      <h2 class="font-black text-xl bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary tracking-tight">文档管理</h2>
      <div class="flex gap-2 items-center flex-wrap">
        <SelectRoot v-model="selectedProjectId" @update:model-value="handleProjectChange">
          <SelectTrigger
            class="w-40 px-3 py-1.5 border border-border-main/40 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand/30 transition-all duration-300 bg-panel-bg/45 backdrop-blur-md text-text-primary flex items-center justify-between hover:border-border-main/60 shadow-sm text-xs"
          >
            <SelectValue placeholder="选择项目" />
            <SelectIcon>
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </SelectIcon>
          </SelectTrigger>
          <SelectPortal>
            <SelectContent class="bg-panel-bg/60 backdrop-blur-md rounded-xl shadow-xl border border-border-main/40 overflow-hidden z-50 animate-fade-in" position="popper" :side-offset="5">
              <SelectViewport class="p-1">
                <SelectItem
                  v-for="project in projectsStore.projects"
                  :key="project.id"
                  :value="project.id"
                  class="relative flex items-center px-4 py-1.5 rounded-lg text-xs cursor-pointer text-text-primary hover:bg-brand/10 data-[highlighted]:bg-brand/10 transition-all duration-200"
                >
                  <SelectItemText>{{ project.name }}</SelectItemText>
                </SelectItem>
              </SelectViewport>
            </SelectContent>
          </SelectPortal>
        </SelectRoot>
        <button 
          class="px-3 py-1.5 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md"
          @click="openCreateProject"
        >
          新建项目
        </button>
        <button
          class="px-3 py-1.5 border border-border-main/40 hover:bg-surface-bg/50 text-text-secondary rounded-xl text-xs font-bold transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed hover:border-border-main/60 shadow-sm"
          :disabled="!selectedProjectId"
          @click="openEditProject(projectsStore.projects.find(p => p.id === selectedProjectId)!)"
        >
          编辑项目
        </button>
        <button
          class="px-3 py-1.5 border border-danger-500/40 text-danger-500 bg-danger-500/5 hover:bg-danger-500/15 rounded-xl text-xs font-bold transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
          :disabled="!selectedProjectId"
          @click="confirmDeleteProject(projectsStore.projects.find(p => p.id === selectedProjectId)!)"
        >
          删除项目
        </button>
        <button
          class="px-3 py-1.5 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 disabled:opacity-50 disabled:cursor-not-allowed shadow-md"
          :disabled="!selectedProjectId"
          @click="showBatchImport = true"
        >
          导入文档
        </button>
      </div>
    </div>

    <!-- 高级 Office 文档解析引导 Banner（未开启时显示） -->
    <div v-if="showOfficeBanner && !isBannerDismissed" class="bg-amber-500/10 backdrop-blur-sm border border-amber-500/25 rounded-xl p-4 animate-fade-in shadow-sm select-none">
      <div class="flex justify-between items-start">
        <div class="flex items-start gap-3">
          <div class="p-1 bg-amber-500/15 text-amber-500 rounded-lg flex-shrink-0">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <div>
            <h3 class="font-bold text-amber-500 text-sm tracking-tight">提示：高级 Office 文档解析未开启</h3>
            <p class="text-text-secondary mt-1.5 text-xs leading-relaxed">
              您当前无法解析 Word (.docx)、Excel (.xlsx/.xls) 和 PPT (.pptx) 文档。
              您可免费开启此功能，以启用本地高性能免插件提取。
            </p>
          </div>
        </div>
        <button @click="isBannerDismissed = true" class="text-text-muted hover:text-text-primary p-1 rounded-lg transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      <div class="mt-3.5 flex gap-3">
        <button
          @click="$router.push('/settings')"
          class="text-xs font-bold text-white bg-brand hover:bg-brand/90 px-3 py-1.5 rounded-lg shadow-sm transition-all"
        >
          前往设置开启
        </button>
        <button
          @click="isBannerDismissed = true"
          class="text-xs font-bold text-text-secondary hover:text-text-primary px-3 py-1.5 transition-colors"
        >
          稍后处理
        </button>
      </div>
    </div>

    <!-- 无项目警告 -->
    <div v-if="!hasProjects" class="bg-warning-500/10 backdrop-blur-sm border border-warning-500/25 rounded-xl p-4 shadow-sm select-none animate-fade-in">
      <div class="flex items-start gap-3">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-warning-500 mt-0.5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div>
          <h3 class="font-bold text-warning-500 text-sm tracking-tight">暂无项目</h3>
          <p class="text-text-secondary mt-1.5 text-xs">请先创建项目，然后再导入文档。</p>
          <button class="mt-3 text-xs font-bold text-brand hover:underline" @click="openCreateProject">
            新建项目
          </button>
        </div>
      </div>
    </div>

    <!-- 文档显示区，只有当有项目时显示 -->
    <div v-if="hasProjects" class="flex-1 flex flex-col gap-4">
      <div class="bg-brand/10 backdrop-blur-sm border border-brand/25 rounded-xl p-4 shadow-sm select-none animate-fade-in">
        <div class="flex items-start gap-3">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-brand mt-0.5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div>
            <h4 class="font-bold text-brand text-sm tracking-tight">后台索引提示</h4>
            <p class="text-text-secondary mt-1.5 text-xs select-text">点击「索引」后，文档会在后台进行索引处理。索引完成后，您将收到通知提醒。</p>
          </div>
        </div>
      </div>
      
      <div v-if="selectedProjectId" class="bg-panel-bg/35 backdrop-blur-sm border border-border-main/25 rounded-xl p-4 shadow-sm select-none">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <p class="text-[10px] text-text-muted mb-1 font-bold select-none uppercase tracking-wider">项目名称</p>
            <p class="font-bold text-text-primary text-sm tracking-tight">{{ projectsStore.projects.find(p => p.id === selectedProjectId)?.name }}</p>
          </div>
          <div>
            <p class="text-[10px] text-text-muted mb-1 font-bold select-none uppercase tracking-wider">描述</p>
            <p class="text-text-secondary text-xs truncate" :title="projectsStore.projects.find(p => p.id === selectedProjectId)?.description || '无描述'">
              {{ projectsStore.projects.find(p => p.id === selectedProjectId)?.description || '无描述' }}
            </p>
          </div>
        </div>
      </div>

      <div class="bg-panel-bg/45 backdrop-blur-md border border-border-main/25 rounded-xl overflow-hidden flex-1 flex flex-col shadow-sm select-none animate-fade-in">
        <div class="border-b border-border-main/25 bg-surface-bg/60 backdrop-blur-sm">
          <div class="grid grid-cols-6 px-4 py-3 text-left text-xs font-bold text-text-muted select-none">
            <div>文件名</div>
            <div>类型</div>
            <div>大小</div>
            <div>状态</div>
            <div>导入时间</div>
            <div>操作</div>
          </div>
        </div>
        <div class="flex-1 flex flex-col min-h-0">
          <div v-if="documentsStore.documentsPaginated.length === 0" class="flex flex-col items-center justify-center h-full select-none">
            <div class="flex flex-col items-center gap-3">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-12 h-12 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p class="text-xs font-medium">暂无文档</p>
            </div>
          </div>
          <div v-else ref="docListContainer" class="h-full overflow-y-auto select-none">
            <div class="relative" :style="{ height: virtualizer.getTotalSize() + 'px' }">
              <div
                  v-for="virtualItem in virtualizer.getVirtualItems()"
                  :key="documentsRef[virtualItem.index].id"
                  :style="{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    transform: `translateY(${virtualItem.start}px)`,
                  }"
                  class="grid grid-cols-6 px-4 py-3 border-b border-border-main/20 hover:bg-brand/5 transition-all duration-300 h-[80px] items-center"
                >
                <div class="truncate font-bold text-text-primary text-xs tracking-tight select-text" :title="documentsRef[virtualItem.index].name">{{ documentsRef[virtualItem.index].name }}</div>
                <div>
                  <span class="px-2.5 py-1 rounded-lg text-xs font-bold bg-surface-bg/40 backdrop-blur-sm text-text-secondary border border-border-main/20">.{{ documentsRef[virtualItem.index].file_type }}</span>
                </div>
                <div class="text-xs font-medium text-text-primary">{{ formatSize(documentsRef[virtualItem.index].size) }}</div>
                <div>
                  <span :class="['px-2.5 py-1 rounded-lg text-xs font-bold', statusMap[documentsRef[virtualItem.index].status]?.class || 'bg-surface-bg text-text-muted']">
                    {{ statusMap[documentsRef[virtualItem.index].status]?.text || '未知' }}
                  </span>
                </div>
                <div class="text-[10px] font-medium text-text-muted">{{ documentsRef[virtualItem.index].created_at }}</div>
                <div>
                  <div class="flex gap-2">
                    <TooltipRoot>
                      <TooltipTrigger as-child>
                        <button
                          class="px-2.5 py-1.5 border border-border-main/40 hover:bg-surface-bg text-text-secondary rounded-xl text-xs font-bold transition-all disabled:opacity-30 disabled:cursor-not-allowed hover:border-border-main/60 shadow-sm select-none"
                          :disabled="documentsRef[virtualItem.index].status !== 'pending' && documentsRef[virtualItem.index].status !== 'error'"
                          @click="handleParse(documentsRef[virtualItem.index].id)"
                        >
                          解析
                        </button>
                      </TooltipTrigger>
                      <TooltipPortal>
                        <TooltipContent class="px-2.5 py-1.5 bg-panel-bg/90 backdrop-blur-md border border-border-main/40 text-text-primary text-xs rounded-xl shadow-xl z-50 select-none animate-fade-in">
                          解析文档内容
                          <TooltipArrow class="fill-border-main/40" />
                        </TooltipContent>
                      </TooltipPortal>
                    </TooltipRoot>
                    <TooltipRoot>
                      <TooltipTrigger as-child>
                        <button
                          class="px-2.5 py-1.5 border border-brand/40 text-brand bg-brand/5 hover:bg-brand/15 rounded-xl text-xs font-bold transition-all disabled:opacity-30 disabled:cursor-not-allowed shadow-sm select-none"
                          :disabled="documentsRef[virtualItem.index].status !== 'done'"
                          @click="handleIndex(documentsRef[virtualItem.index].id)"
                        >
                          索引
                        </button>
                      </TooltipTrigger>
                      <TooltipPortal>
                        <TooltipContent class="px-2.5 py-1.5 bg-panel-bg/90 backdrop-blur-md border border-border-main/40 text-text-primary text-xs rounded-xl shadow-xl z-50 select-none animate-fade-in">
                          建立向量索引
                          <TooltipArrow class="fill-border-main/40" />
                        </TooltipContent>
                      </TooltipPortal>
                    </TooltipRoot>
                    <TooltipRoot>
                      <TooltipTrigger as-child>
                        <button
                          class="px-2.5 py-1.5 border border-danger-500/40 text-danger-500 bg-danger-500/5 hover:bg-danger-500/15 rounded-xl text-xs font-bold transition-all shadow-sm select-none"
                          @click="confirmDeleteDoc(documentsRef[virtualItem.index])"
                        >
                          删除
                        </button>
                      </TooltipTrigger>
                      <TooltipPortal>
                        <TooltipContent class="px-2.5 py-1.5 bg-panel-bg/90 backdrop-blur-md border border-border-main/40 text-text-primary text-xs rounded-xl shadow-xl z-50 select-none animate-fade-in">
                          删除此文档
                          <TooltipArrow class="fill-border-main/40" />
                        </TooltipContent>
                      </TooltipPortal>
                    </TooltipRoot>
                  </div>
                </div>
              </div>
            </div>
            <!-- 加载更多指示器 -->
            <div v-if="documentsStore.isLoadingMore || documentsStore.documentsHasMore" ref="target" class="py-4 flex justify-center items-center">
              <div class="flex items-center gap-2 text-text-muted text-xs">
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-text-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                加载中...
              </div>
            </div>
            <div v-else-if="documentsRef.length > 0" class="py-4 flex justify-center items-center text-text-muted text-xs select-none">
              已加载全部
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 项目模态框 -->
    <DialogRoot v-model:open="showProjectModal">
      <DialogPortal>
        <DialogOverlay class="fixed inset-0 bg-black/40 backdrop-blur-sm z-40 animate-fade-in" />
        <DialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-panel-bg/65 backdrop-blur-md border border-border-main/35 rounded-2xl p-6 sm:max-w-lg w-full mx-4 z-50 shadow-2xl animate-fade-in">
          <DialogTitle class="text-base font-black mb-4 bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary select-none tracking-tight">{{ editingProject ? '编辑项目' : '新建项目' }}</DialogTitle>
          <form @submit.prevent="handleSaveProject" class="space-y-4">
            <div>
              <label class="block text-xs font-bold mb-1.5 text-text-primary select-none">项目名称 *</label>
              <input
                v-model="projectForm.name"
                type="text"
                placeholder="请输入项目名称"
                class="w-full px-3.5 py-2 border border-border-main/40 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand/30 transition-all duration-300 bg-panel-bg/45 text-text-primary placeholder:text-text-muted/60 text-xs shadow-sm hover:border-border-main/60 select-text"
              />
            </div>
            <div>
              <label class="block text-xs font-bold mb-1.5 text-text-primary select-none">描述</label>
              <textarea
                v-model="projectForm.description"
                rows="4"
                placeholder="请输入项目描述（可选）"
                class="w-full px-3.5 py-2 border border-border-main/40 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand/30 transition-all duration-300 resize-none bg-panel-bg/45 text-text-primary placeholder:text-text-muted/60 text-xs shadow-sm hover:border-border-main/60 select-text"
              ></textarea>
            </div>
            <Separator class="mt-3 pt-3 border-t border-border-main/25" />
            <div class="flex justify-end gap-2.5">
              <DialogClose class="px-3.5 py-2 border border-border-main/40 hover:bg-surface-bg/50 text-text-secondary rounded-xl text-xs font-bold transition-all duration-300 shadow-sm">
                取消
              </DialogClose>
              <button type="submit" class="px-3.5 py-2 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md">
                保存
              </button>
            </div>
          </form>
          <DialogClose class="absolute top-4 right-4 p-1.5 rounded-full hover:bg-surface-bg/50 transition-colors" aria-label="关闭">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </DialogClose>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>

    <!-- 文档删除确认模态框 -->
    <DialogRoot v-model:open="showDeleteDocConfirm">
      <DialogPortal>
        <DialogOverlay class="fixed inset-0 bg-black/40 backdrop-blur-sm z-40 animate-fade-in" />
        <DialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-panel-bg/65 backdrop-blur-md border border-border-main/35 rounded-2xl p-6 sm:max-w-lg w-full mx-4 z-50 shadow-2xl animate-fade-in select-none">
          <DialogTitle class="text-base font-black text-danger-500 mb-2 tracking-tight">确认删除</DialogTitle>
          <DialogDescription class="text-text-secondary mb-5 text-xs select-text">确定要删除文档"{{ docToDelete?.name }}"吗？此操作不可逆，该文档的所有向量索引和分片都将被永久删除。</DialogDescription>
          <Separator class="mb-4 pt-3 border-t border-border-main/25" />
          <div class="flex justify-end gap-2.5">
            <DialogClose class="px-3.5 py-2 border border-border-main/40 hover:bg-surface-bg/50 text-text-secondary rounded-xl text-xs font-bold transition-all shadow-sm">
              取消
            </DialogClose>
            <button type="button" class="px-3.5 py-2 bg-danger-500 bg-danger-500/90 hover:bg-danger/90 hover:shadow-lg hover:shadow-danger-500/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md" @click="handleDeleteDoc">
              删除
            </button>
          </div>
          <DialogClose class="absolute top-4 right-4 p-1.5 rounded-full hover:bg-surface-bg/50 transition-colors" aria-label="关闭">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </DialogClose>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>

    <!-- 删除确认模态框 -->
    <DialogRoot v-model:open="showDeleteConfirm">
      <DialogPortal>
        <DialogOverlay class="fixed inset-0 bg-black/40 backdrop-blur-sm z-40 animate-fade-in" />
        <DialogContent class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-panel-bg/65 backdrop-blur-md border border-border-main/35 rounded-2xl p-6 sm:max-w-lg w-full mx-4 z-50 shadow-2xl animate-fade-in select-none">
          <DialogTitle class="text-base font-black text-danger-500 mb-2 tracking-tight">确认删除</DialogTitle>
          <DialogDescription class="text-text-secondary mb-5 text-xs select-text">确定要删除项目"{{ projectToDelete?.name }}"吗？</DialogDescription>
          <Separator class="mb-4 pt-3 border-t border-border-main/25" />
          <div class="flex justify-end gap-2.5">
            <DialogClose class="px-3.5 py-2 border border-border-main/40 hover:bg-surface-bg/50 text-text-secondary rounded-xl text-xs font-bold transition-all shadow-sm">
              取消
            </DialogClose>
            <button type="button" class="px-3.5 py-2 bg-danger-500 bg-danger-500/90 hover:bg-danger/90 hover:shadow-lg hover:shadow-danger-500/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md" @click="handleDeleteProject">
              删除
            </button>
          </div>
          <DialogClose class="absolute top-4 right-4 p-1.5 rounded-full hover:bg-surface-bg/50 transition-colors" aria-label="关闭">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </DialogClose>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>

    <BatchImportDialog
      :open="showBatchImport"
      :project-id="selectedProjectId || ''"
      @close="showBatchImport = false"
      @complete="handleImportComplete"
    />
  </div>
</template>
