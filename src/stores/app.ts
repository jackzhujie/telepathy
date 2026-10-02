import { defineStore } from 'pinia';

export const useAppStore = defineStore('app', {
  state: () => ({
    isSidebarCollapsed: false
  }),
  actions: {
    toggleSidebar() {
      this.isSidebarCollapsed = !this.isSidebarCollapsed;
    }
  }
});
