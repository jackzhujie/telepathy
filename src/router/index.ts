import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'KnowledgeBase', component: () => import('@/views/KnowledgeBase.vue') },
  { path: '/chat', name: 'Chat', component: () => import('@/views/Chat.vue'), meta: { keepAlive: true } },
  { path: '/documents', name: 'Documents', component: () => import('@/views/Documents.vue'), meta: { keepAlive: true } },
  { path: '/profile', name: 'Profile', component: () => import('@/views/Profile.vue'), meta: { keepAlive: true } },
  { path: '/settings', name: 'Settings', component: () => import('@/views/Settings.vue'), meta: { keepAlive: true } },
  { path: '/models', name: 'Models', component: () => import('@/views/Models.vue'), meta: { keepAlive: true } },
  { path: '/snapnote', name: 'SnapNote', component: () => import('@/views/SnapNote.vue'), meta: { keepAlive: true } }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
