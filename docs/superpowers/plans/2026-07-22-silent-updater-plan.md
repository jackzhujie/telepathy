# 后台静默检测更新与自动发布实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现后台每 30 分钟静默检测并下载更新，并将版本号升级为 `0.3.9` 配合 Git Tag 实现自动化构建分发。

**Architecture:** 
1. 在前端根组件 `App.vue` 挂载时，启动 `setInterval` 后台轮询静默检测，卸载时清除。
2. 升级 `package.json` 与 `tauri.conf.json` 版本号至 `0.3.9`。
3. 本地验证编译与测试，提交推送。
4. 推送 Git Tag `v0.3.9` 触发 GitHub Actions 的全平台发布流水线。

**Tech Stack:** Vue 3 / TypeScript, Tauri v2, GitHub Actions CI/CD

---

### Task 1: 前端静默轮询更新与清理逻辑

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 修改 App.vue 脚本导入及轮询逻辑**

  在 `src/App.vue` 中导入 `onUnmounted` 并加入 `setInterval` 定时轮询，在销毁时清理。
  修改 `src/App.vue` 中的 `<script setup>` 如下：

  ```html
  <script setup lang="ts">
  import AppLayout from '@/components/layout/AppLayout.vue';
  import SetupWizard from '@/components/settings/SetupWizard.vue';
  import { useNotificationsStore } from '@/stores/notifications';
  import { useSettingsStore } from '@/stores/settings';
  import { useUpdater } from '@/hooks/useUpdater';
  import { onMounted, onUnmounted, watch } from 'vue'; // 导入 onUnmounted
  import { TooltipProvider } from 'reka-ui';

  const notificationsStore = useNotificationsStore();
  const settingsStore = useSettingsStore();
  const { checkForUpdates, updateInfo, isDownloading, isDownloaded, installUpdate } = useUpdater();

  // 自动更新触发逻辑
  watch(updateInfo, (val) => { 
    if (val && settingsStore.autoUpdate && !isDownloading.value && !isDownloaded.value) {
      installUpdate();
    }
  });

  let updateInterval: any = null; // 定时器句柄

  onMounted(async () => {
    await notificationsStore.initListener();
    await notificationsStore.fetchNotifications();
    await settingsStore.fetchSettings();
    await settingsStore.fetchInstalledModels();

    // 启动时检查更新 (静默)
    checkForUpdates(true);

    // 每 30 分钟 (1800000 毫秒) 自动静默检测更新一次
    updateInterval = setInterval(() => {
      checkForUpdates(true);
    }, 1800000);

    // 监听首次用户交互来解锁音频限制
    const unlockAudio = () => {
      try {
        const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
        if (ctx.state === 'suspended') {
          ctx.resume();
        }
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        gain.gain.setValueAtTime(0, ctx.currentTime);
        osc.connect(gain);
        gain.connect(ctx.destination);
        osc.start();
        osc.stop(ctx.currentTime + 0.1);
        window.removeEventListener('click', unlockAudio);
      } catch (e) {
        console.error('解锁音频失败:', e);
      }
    };
    window.addEventListener('click', unlockAudio, { once: true });
  });

  onUnmounted(() => {
    if (updateInterval) {
      clearInterval(updateInterval); // 清除定时器，避免内存泄露
    }
  });
  </script>
  ```

- [ ] **Step 2: 运行 vue-tsc 静态编译类型检查**

  在工作区根目录下运行：
  `pnpm vue-tsc --noEmit`
  Expected: PASS

- [ ] **Step 3: 提交代码**

  ```bash
  git add src/App.vue
  git commit -m "feat: add 30-minute interval background silent updater polling to App.vue"
  ```


### Task 2: 升级配置版本号至 0.3.9

**Files:**
- Modify: `package.json`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 修改 package.json 版本号**

  在 `package.json` 的第 4 行，将 `"version": "0.1.0"` 修改为 `"version": "0.3.9"`。

- [ ] **Step 2: 修改 tauri.conf.json 版本号**

  在 `src-tauri/tauri.conf.json` 的第 4 行，将 `"version": "0.3.7"` 修改为 `"version": "0.3.9"`。

- [ ] **Step 3: 运行并验证前端打包编译**

  在工作区根目录下运行：
  `pnpm build`
  Expected: PASS

- [ ] **Step 4: 运行后端单元测试**

  在 `src-tauri` 目录下运行：
  `cargo test`
  Expected: PASS (所有 52 个测试通过)

- [ ] **Step 5: 提交并推送至 GitHub**

  ```bash
  git add package.json src-tauri/tauri.conf.json
  git commit -m "bump: upgrade app version to v0.3.9 for release"
  git push origin main
  ```


### Task 3: 推送 Tag 触发 GitHub Actions 自动发布

- [ ] **Step 1: 推送版本 Tag**

  在工作区根目录下运行：
  ```bash
  git tag v0.3.9
  git push origin v0.3.9
  ```
  Expected: 本地成功创建并推送到 GitHub。GitHub Actions 将触发多平台编译、自动签名以及分发脚本。
