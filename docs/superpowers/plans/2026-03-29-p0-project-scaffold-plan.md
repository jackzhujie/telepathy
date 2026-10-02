# Telepathy P0 Project Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the P0 project skeleton for Telepathy using Tauri v2, Vue 3.5+, TypeScript, and Rust, ensuring the frontend-to-backend communication link works.

**Architecture:** A standard Tauri v2 architecture with a Vue 3 composition-API frontend (using Naive UI, Pinia, Vue Router) and a 3-layer Rust backend (commands, services, models) with unified error handling.

**Tech Stack:** Tauri v2, Vue 3.5+, TypeScript 5.4+, Vite 5, Naive UI 2.38+, Pinia 2, Vue Router 4, pnpm, Rust, tokio, serde.

---

### Task 1: Environment Setup and Project Scaffold

**Files:**
- Create: `package.json`, `src-tauri/Cargo.toml`, etc.

- [ ] **Step 1: Install pnpm**

```bash
npm install -g pnpm
```

- [ ] **Step 2: Scaffold Tauri v2 App**
Since we are in an existing git repo, create it in a temp directory and move the contents.

```bash
pnpm create tauri-app temp-app --template vue-ts --manager pnpm -y
shopt -s dotglob
mv temp-app/* .
rm -rf temp-app
```

- [ ] **Step 3: Install Additional Frontend Dependencies**

```bash
pnpm add naive-ui pinia vue-router
pnpm add -D unplugin-vue-components @types/node
```

- [ ] **Step 4: Update Tauri Config for Window Size and App Identifier**
Modify `src-tauri/tauri.conf.json`:
Set `identifier` to `"com.telepathy.app"`.
Set `title` to `"Telepathy"`.
Update `windows` array to include `"width": 1200`, `"height": 800`, `"minWidth": 900`, `"minHeight": 600`.

```bash
sed -i '' 's/"com.tauri.dev"/"com.telepathy.app"/' src-tauri/tauri.conf.json
sed -i '' 's/"title": ".*"/"title": "Telepathy"/' src-tauri/tauri.conf.json
```
*(Verify changes by reading the file and manually editing if sed fails)*

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "chore: scaffold tauri v2 vue-ts app and install dependencies"
```

### Task 2: Configure Vite and TypeScript

**Files:**
- Modify: `vite.config.ts`, `tsconfig.json`

- [ ] **Step 1: Configure Vite Alias and Naive UI Auto-import**
Update `vite.config.ts` to look exactly like this:

```typescript
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import Components from 'unplugin-vue-components/vite';
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers';
import path from "path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [
    vue(),
    Components({
      resolvers: [NaiveUiResolver()]
    })
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
```

- [ ] **Step 2: Configure TypeScript Paths**
Update `tsconfig.json` compilerOptions to include baseUrl and paths, and add node types:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "types": ["node"],

    /* Bundler mode */
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",

    /* Linting */
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,

    /* Paths */
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 3: Run Typecheck**

```bash
pnpm type-check
```

- [ ] **Step 4: Commit**

```bash
git add vite.config.ts tsconfig.json
git commit -m "build: configure vite aliases and naive-ui auto-import"
```

### Task 3: Setup Rust Backend Architecture

**Files:**
- Create: `src-tauri/src/errors.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/greet.rs`, `src-tauri/src/services/mod.rs`, `src-tauri/src/models/mod.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`

- [ ] **Step 1: Add Rust Dependencies**

```bash
cd src-tauri
cargo add thiserror tokio --features tokio/full
cd ..
```

- [ ] **Step 2: Create AppError**
Create `src-tauri/src/errors.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("内部错误: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
```

- [ ] **Step 3: Create Backend Modules**
Create `src-tauri/src/services/mod.rs`:
```rust
// Services module (Empty for P0)
```

Create `src-tauri/src/models/mod.rs`:
```rust
// Models module (Empty for P0)
```

Create `src-tauri/src/commands/greet.rs`:
```rust
use crate::errors::AppError;

#[tauri::command]
pub fn greet(name: &str) -> Result<String, AppError> {
    Ok(format!("你好，{}！Telepathy 已准备就绪。", name))
}
```

Create `src-tauri/src/commands/mod.rs`:
```rust
pub mod greet;
```

- [ ] **Step 4: Update lib.rs**
Modify `src-tauri/src/lib.rs` to register the modules and command:

```rust
mod commands;
mod errors;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::greet::greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Run Cargo Check**

```bash
cd src-tauri && cargo clippy -- -D warnings && cd ..
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/
git commit -m "feat(backend): setup rust architecture and greet command"
```

### Task 4: Setup Frontend Router and Store

**Files:**
- Create: `src/router/index.ts`, `src/stores/app.ts`, `src/api/tauri.ts`
- Create views: `src/views/KnowledgeBase.vue`, `src/views/Chat.vue`, `src/views/Documents.vue`, `src/views/Settings.vue`

- [ ] **Step 1: Create Tauri API wrapper**
Create `src/api/tauri.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';

export async function greet(name: string): Promise<string> {
  return invoke<string>('greet', { name });
}
```

- [ ] **Step 2: Create Views**
Create `src/views/KnowledgeBase.vue`:
```vue
<script setup lang="ts"></script>
<template>
  <div class="page-container">
    <h2>知识库</h2>
  </div>
</template>
```

Create `src/views/Chat.vue`:
```vue
<script setup lang="ts"></script>
<template>
  <div class="page-container">
    <h2>AI 对话</h2>
  </div>
</template>
```

Create `src/views/Documents.vue`:
```vue
<script setup lang="ts"></script>
<template>
  <div class="page-container">
    <h2>文档管理</h2>
  </div>
</template>
```

Create `src/views/Settings.vue`:
```vue
<script setup lang="ts">
import { ref } from 'vue';
import { greet } from '@/api/tauri';

const name = ref('');
const message = ref('');

async function testGreet() {
  try {
    message.value = await greet(name.value || '开发者');
  } catch (e) {
    message.value = String(e);
  }
}
</script>
<template>
  <div class="page-container">
    <h2>系统设置</h2>
    <div style="margin-top: 20px; display: flex; gap: 10px; align-items: center;">
      <n-input v-model:value="name" placeholder="输入你的名字" style="width: 200px" />
      <n-button type="primary" @click="testGreet">测试通信</n-button>
    </div>
    <div v-if="message" style="margin-top: 20px;">
      <n-alert type="success" :show-icon="false">
        {{ message }}
      </n-alert>
    </div>
  </div>
</template>
```

- [ ] **Step 3: Create Router**
Create `src/router/index.ts`:

```typescript
import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'KnowledgeBase', component: () => import('@/views/KnowledgeBase.vue') },
  { path: '/chat', name: 'Chat', component: () => import('@/views/Chat.vue') },
  { path: '/documents', name: 'Documents', component: () => import('@/views/Documents.vue') },
  { path: '/settings', name: 'Settings', component: () => import('@/views/Settings.vue') }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
```

- [ ] **Step 4: Create Store**
Create `src/stores/app.ts`:

```typescript
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
```

- [ ] **Step 5: Commit**

```bash
git add src/api src/views src/router src/stores
git commit -m "feat(frontend): add router, store, views and tauri api"
```

### Task 5: Implement UI Layout

**Files:**
- Create: `src/components/layout/Sidebar.vue`, `src/components/layout/AppLayout.vue`
- Modify: `src/App.vue`, `src/main.ts`, `src/style.css`

- [ ] **Step 1: Create Sidebar Component**
Create `src/components/layout/Sidebar.vue`:

```vue
<script setup lang="ts">
import { h } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAppStore } from '@/stores/app';
import { NMenu } from 'naive-ui';

const router = useRouter();
const route = useRoute();
const appStore = useAppStore();

const menuOptions = [
  { label: '知识库', key: 'KnowledgeBase' },
  { label: 'AI 对话', key: 'Chat' },
  { label: '文档管理', key: 'Documents' },
  { label: '系统设置', key: 'Settings' }
];

function handleMenuClick(key: string) {
  router.push({ name: key });
}
</script>

<template>
  <n-layout-sider
    bordered
    collapse-mode="width"
    :collapsed-width="64"
    :width="200"
    :collapsed="appStore.isSidebarCollapsed"
    show-trigger
    @collapse="appStore.isSidebarCollapsed = true"
    @expand="appStore.isSidebarCollapsed = false"
  >
    <div style="padding: 16px; text-align: center; font-weight: bold; font-size: 18px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
      {{ appStore.isSidebarCollapsed ? 'T' : 'Telepathy' }}
    </div>
    <n-menu
      :value="route.name as string"
      :options="menuOptions"
      @update:value="handleMenuClick"
    />
  </n-layout-sider>
</template>
```

- [ ] **Step 2: Create AppLayout Component**
Create `src/components/layout/AppLayout.vue`:

```vue
<script setup lang="ts">
import Sidebar from './Sidebar.vue';
</script>

<template>
  <n-layout has-sider style="height: 100vh;">
    <Sidebar />
    <n-layout>
      <div style="padding: 24px; height: 100%; box-sizing: border-box;">
        <router-view />
      </div>
    </n-layout>
  </n-layout>
</template>
```

- [ ] **Step 3: Update App.vue**
Modify `src/App.vue` to wrap with Naive UI providers and use the layout:

```vue
<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider } from 'naive-ui';
import AppLayout from '@/components/layout/AppLayout.vue';
</script>

<template>
  <n-config-provider style="height: 100vh;">
    <n-message-provider>
      <n-dialog-provider>
        <AppLayout />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
```

- [ ] **Step 4: Update main.ts**
Modify `src/main.ts`:

```typescript
import { createApp } from "vue";
import { createPinia } from 'pinia';
import router from './router';
import App from "./App.vue";
import "./style.css";

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.mount("#app");
```

- [ ] **Step 5: Clean up default styles**
Modify `src/style.css` to only contain basic resets:

```css
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color-scheme: light dark;
  color: rgba(255, 255, 255, 0.87);
  background-color: #242424;
}

body {
  margin: 0;
  display: flex;
  min-height: 100vh;
  min-width: 100vw;
}

#app {
  width: 100vw;
  height: 100vh;
}

@media (prefers-color-scheme: light) {
  :root {
    color: #213547;
    background-color: #ffffff;
  }
}
```

- [ ] **Step 6: Remove boilerplate components**

```bash
rm -rf src/components/Greet.vue src/assets/tauri.svg src/assets/vue.svg
```

- [ ] **Step 7: Check TypeScript Types**

```bash
pnpm type-check
```

- [ ] **Step 8: Commit**

```bash
git add src/components src/App.vue src/main.ts src/style.css
git commit -m "feat(frontend): implement layout, sidebar and clean up boilerplate"
```
