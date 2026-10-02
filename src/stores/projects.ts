import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Project, CreateProjectInput, UpdateProjectInput } from '@/types/project';
import { createProject, listProjects, updateProject, deleteProject } from '@/api/tauri';

export const useProjectsStore = defineStore('projects', () => {
  const projects = ref<Project[]>([]);
  const currentProjectId = ref<string | null>(null);
  const isLoading = ref(false);

  const currentProject = computed(() => 
    projects.value.find(p => p.id === currentProjectId.value)
  );

  async function fetchProjects() {
    isLoading.value = true;
    try {
      projects.value = await listProjects();
    } finally {
      isLoading.value = false;
    }
  }

  async function createProjectStore(input: CreateProjectInput) {
    const project = await createProject(input);
    projects.value.unshift(project);
    if (!currentProjectId.value) {
      currentProjectId.value = project.id;
    }
    return project;
  }

  async function updateProjectStore(input: UpdateProjectInput) {
    await updateProject(input);
    const index = projects.value.findIndex(p => p.id === input.id);
    if (index !== -1) {
      projects.value[index] = { ...projects.value[index], ...input };
    }
  }

  async function deleteProjectStore(id: string) {
    await deleteProject(id);
    projects.value = projects.value.filter(p => p.id !== id);
    if (currentProjectId.value === id) {
      currentProjectId.value = projects.value[0]?.id || null;
    }
  }

  function setCurrentProject(id: string | null) {
    currentProjectId.value = id;
  }

  return {
    projects,
    currentProjectId,
    currentProject,
    isLoading,
    fetchProjects,
    createProject: createProjectStore,
    updateProject: updateProjectStore,
    deleteProject: deleteProjectStore,
    setCurrentProject,
  };
});