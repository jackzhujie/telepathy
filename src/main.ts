import { createApp } from "vue";
import { createPinia } from 'pinia';
import { MotionPlugin } from '@vueuse/motion';
import router from './router';
import App from "./App.vue";
import "./style.css";
import { invoke } from "@tauri-apps/api/core";

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(MotionPlugin);
app.mount("#app");

// 挂载到全局以便调试服务器 (debug_server.rs) 调用
(window as any).__TELEPATHY_ROUTER__ = router;

// 路由变化时上报到后端，方便 AI 验证导航
router.afterEach((to) => {
  document.body.dataset.route = to.fullPath;
  // 等下一个 tick 让 document.title 更新（如果有）
  setTimeout(() => {
    invoke("tele_debug_report_route", {
      route: to.fullPath,
      title: document.title || "",
    }).catch(() => {});
  }, 100);
});
// 首次进入也要报一次
invoke("tele_debug_report_route", {
  route: router.currentRoute.value.fullPath,
  title: document.title || "",
}).catch(() => {});
