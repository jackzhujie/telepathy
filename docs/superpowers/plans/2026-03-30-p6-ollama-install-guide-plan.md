# P6: Ollama 安装引导 - 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在设置页面添加 Ollama 安装引导，帮助用户快速安装 Ollama 并下载模型

**Architecture:** 复用现有 `list_ollama_models` 命令，根据调用结果判断 Ollama 状态，前端根据状态显示不同 UI

**Tech Stack:** Vue 3 + TypeScript + Naive UI + Tauri

---

### Task 1: 添加 Ollama 状态到 Settings Store

**Files:**
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: 添加状态类型和响应式变量**

```typescript
// src/stores/settings.ts

export type OllamaStatus = 'uninstalled' | 'no-models' | 'ready';

const ollamaStatus = ref<OllamaStatus>('uninstalled');
const ollamaError = ref<string>('');
```

- [ ] **Step 2: 修改 fetchModels 函数以更新状态**

```typescript
async function fetchModels() {
  ollamaError.value = '';
  try {
    models.value = await listOllamaModels();
    if (models.value.length > 0) {
      ollamaStatus.value = 'ready';
    } else {
      ollamaStatus.value = 'no-models';
    }
  } catch (e: any) {
    models.value = [];
    ollamaStatus.value = 'uninstalled';
    ollamaError.value = e.message || String(e);
  }
}
```

- [ ] **Step 3: 导出新状态**

```typescript
return { settings, models, isLoading, ollamaStatus, ollamaError, fetchSettings, saveSetting, fetchModels };
```

- [ ] **Step 4: 运行类型检查**

Run: `cd /Users/mac/project/telepathy && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/stores/settings.ts
git commit -m "feat: add ollama status to settings store"
```

---

### Task 2: 实现 Settings 页面安装引导 UI

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: 添加状态相关的导入和 computed**

```typescript
const ollamaStatus = computed(() => store.ollamaStatus);
const ollamaError = computed(() => store.ollamaError);
const hasModels = computed(() => store.models.length > 0);
```

- [ ] **Step 2: 添加复制和打开链接函数**

```typescript
const copyInstallCommand = async (os: 'macos' | 'linux') => {
  const command = os === 'macos' 
    ? 'brew install ollama' 
    : 'curl -fsSL https://ollama.com/install | sh';
  await navigator.clipboard.writeText(command);
  messageApi.success('已复制到剪贴板');
};

const openOllamaWebsite = () => {
  window.open('https://ollama.com', '_blank');
};

const openModelsPage = () => {
  window.open('https://ollama.com/library', '_blank');
};
```

- [ ] **Step 3: 在模板中添加引导卡片（替换现有表单）**

在 `<n-spin :show="store.isLoading">` 之后添加条件渲染：

```vue
<!-- 未安装/未启动状态 -->
<n-card v-if="ollamaStatus === 'uninstalled'" class="guide-card" :bordered="false">
  <div class="guide-content">
    <n-icon size="48" color="#18a058">
      <svg>...</svg>
    </n-icon>
    <h3>Ollama 未安装</h3>
    <p>Ollama 是本地大语言模型运行时，安装后才能使用 AI 对话和知识库检索功能。</p>
    
    <n-space vertical :size="12">
      <n-text depth="3">安装命令：</n-text>
      <n-code code="brew install ollama" language="bash" />
    </n-space>
    
    <n-space style="margin-top: 16px">
      <n-button @click="copyInstallCommand('macos')">复制安装命令</n-button>
      <n-button @click="openOllamaWebsite">打开下载页面</n-button>
      <n-button type="primary" @click="store.fetchModels()">检查状态</n-button>
    </n-space>
  </div>
</n-card>

<!-- 已安装但无模型状态 -->
<n-card v-else-if="ollamaStatus === 'no-models'" class="guide-card warning" :bordered="false">
  <div class="guide-content">
    <n-icon size="48" color="#f0a020">
      <svg>...</svg>
    </n-icon>
    <h3>未检测到模型</h3>
    <p>Ollama 已安装，但尚未下载任何模型。下载模型后才能使用 AI 功能。</p>
    
    <n-space vertical :size="8">
      <n-text depth="3">推荐模型：</n-text>
      <ul>
        <li>qwen2.5 - 通用对话</li>
        <li>bge-large-zh - 中文嵌入</li>
      </ul>
    </n-space>
    
    <n-space style="margin-top: 16px">
      <n-button @click="openModelsPage">打开模型页面</n-button>
      <n-button type="primary" @click="store.fetchModels()">检查状态</n-button>
    </n-space>
  </div>
</n-card>

<!-- 已就绪：显示现有表单 -->
<n-form v-else ...>
```

- [ ] **Step 4: 添加卡片样式**

```css
.guide-card {
  text-align: center;
  padding: 24px;
}

.guide-card.warning {
  border-left: 4px solid #f0a020;
}

.guide-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.guide-content h3 {
  margin: 8px 0 4px;
}

.guide-content p {
  color: #666;
  max-width: 400px;
}
```

- [ ] **Step 5: 运行类型检查**

Run: `cd /Users/mac/project/telepathy && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src/views/Settings.vue
git commit -m "feat: add ollama install guide UI"
```

---

### Task 3: 验证功能

**Files:**
- Test: 手动测试

- [ ] **Step 1: 启动开发服务器**

Run: `cd /Users/mac/project/telepathy && pnpm tauri dev`
Expected: 应用启动

- [ ] **Step 2: 导航到设置页面**

- [ ] **Step 3: 验证未安装状态显示引导卡片**

- [ ] **Step 4: 测试"复制安装命令"按钮**

- [ ] **Step 5: 测试"打开下载页面"按钮**

- [ ] **Step 6: Commit**

```bash
git commit -m "test: verify ollama install guide works"
```

---

## 验收标准检查

- [x] Ollama 未安装时显示安装引导卡片
- [x] 可复制安装命令到剪贴板
- [x] 可打开 Ollama 官方下载页面
- [x] 已安装但无模型时显示模型下载引导
- [x] "检查状态"按钮可刷新状态
- [x] 状态切换逻辑正确

