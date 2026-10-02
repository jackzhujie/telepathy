# P7: Ollama 模型下载与管理 - 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Ollama 模型下载与管理功能，包括模型推荐、下载进度显示、自动启动服务

**Architecture:** 后端 Rust 处理系统检测、模型下载、进程管理；前端 Vue 显示配置信息、模型列表、下载进度

**Tech Stack:** Rust + Tauri + Vue 3 + TypeScript

---

### Task 1: 后端 - 添加系统检测和模型拉取命令

**Files:**
- Modify: `src-tauri/src/commands/installer.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加系统信息结构体**

```rust
#[derive(Debug, Serialize, Clone)]
pub struct SystemInfo {
    pub cpu_cores: usize,
    pub memory_gb: usize,
    pub has_gpu: bool,
    pub gpu_name: Option<String>,
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    let cpu_cores = num_cpus::get();
    
    // 获取内存信息 (macOS)
    #[cfg(target_os = "macos")]
    let memory_gb = {
        use std::process::Command;
        let output = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok();
        output.map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u64>()
                .unwrap_or(0) / (1024 * 1024 * 1024)
        }).unwrap_or(0) as usize
    };
    
    #[cfg(not(target_os = "macos"))]
    let memory_gb = 8; // 默认值
    
    SystemInfo {
        cpu_cores,
        memory_gb,
        has_gpu: false,
        gpu_name: None,
    }
}
```

- [ ] **Step 2: 添加拉取模型命令**

```rust
#[derive(Debug, Serialize, Clone)]
pub struct PullProgress {
    pub model: String,
    pub percentage: f64,
    pub speed: String,
    pub downloaded: String,
    pub total: String,
    pub eta: String,
    pub status: String,
}

#[tauri::command]
pub async fn pull_model(app: AppHandle, model: String) -> Result<String, AppError> {
    let ollama_path = get_ollama_install_path()?
        .ok_or_else(|| AppError::Internal("Ollama not installed".to_string()))?;
    
    let model_clone = model.clone();
    let app_clone = app.clone();
    
    // 后台运行 ollama pull
    let handle = tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        use tokio::process::Command;
        
        let mut child = Command::new(&ollama_path)
            .args(["pull", &model_clone])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start: {}", e))?;
        
        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("[pull] {}", line);
            // 解析进度并发送事件
            let _ = app_clone.emit("ollama-pull-progress", PullProgress {
                model: model_clone.clone(),
                percentage: -1.0,
                speed: "".to_string(),
                downloaded: "".to_string(),
                total: "".to_string(),
                eta: "".to_string(),
                status: "downloading".to_string(),
            });
        }
        
        let _ = app_clone.emit("ollama-pull-progress", PullProgress {
            model: model_clone,
            percentage: 100.0,
            speed: "".to_string(),
            downloaded: "".to_string(),
            total: "".to_string(),
            eta: "".to_string(),
            status: "done".to_string(),
        });
        
        Ok::<(), String>(())
    });
    
    Ok("Started".to_string())
}
```

- [ ] **Step 3: 添加启动 Ollama 服务命令**

```rust
#[tauri::command]
pub async fn start_ollama() -> Result<(), AppError> {
    let ollama_path = get_ollama_install_path()?
        .ok_or_else(|| AppError::Internal("Ollama not installed".to_string()))?;
    
    // 后台启动 ollama serve
    tokio::spawn(async move {
        let _ = tokio::process::Command::new(&ollama_path)
            .args(["serve"])
            .spawn();
    });
    
    // 等待服务启动
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    Ok(())
}
```

- [ ] **Step 4: 在 lib.rs 中注册新命令**

```rust
commands::installer::get_system_info,
commands::installer::pull_model,
commands::installer::start_ollama,
```

- [ ] **Step 5: 运行 cargo check**

Run: `cd /Users/mac/project/telepathy && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/installer.rs src-tauri/src/lib.rs
git commit -m "feat: add system info, pull model and start ollama commands"
```

---

### Task 2: 前端 - 添加模型管理 API 和 Store 状态

**Files:**
- Modify: `src/api/tauri.ts`
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: 添加 API 调用**

```typescript
export async function getSystemInfo(): Promise<{
  cpu_cores: number;
  memory_gb: number;
  has_gpu: boolean;
  gpu_name: string | null;
}> {
  return invoke('get_system_info');
}

export async function pullModel(model: string): Promise<string> {
  return invoke('pull_model', { model });
}

export async function startOllama(): Promise<void> {
  return invoke('start_ollama');
}
```

- [ ] **Step 2: 添加预设模型列表**

```typescript
export const RECOMMENDED_MODELS = [
  { id: 'qwen2.5:0.5b', name: 'qwen2.5', size: '0.5B', memory: '4GB', desc: '响应最快，适合快速问答', category: 'light' },
  { id: 'llama3.2:1b', name: 'llama3.2', size: '1B', memory: '4GB', desc: '响应快，英文能力强', category: 'light' },
  { id: 'qwen2.5:7b', name: 'qwen2.5', size: '7B', memory: '8GB', desc: '平衡响应与智能，中文优化好', category: 'medium' },
  { id: 'llama3.2:3b', name: 'llama3.2', size: '3B', memory: '8GB', desc: '智能水平中等，英文为主', category: 'medium' },
  { id: 'qwen2.5:14b', name: 'qwen2.5', size: '14B', memory: '16GB', desc: '智能较高，中文能力强', category: ' flagship' },
  { id: 'llama3.2:8b', name: 'llama3.2', size: '8B', memory: '16GB', desc: '英文能力最强', category: 'flagship' },
  { id: 'bge-m3', name: 'bge-m3', size: '', memory: '4GB', desc: '多语言嵌入，中文效果好', category: 'embedding' },
];
```

- [ ] **Step 3: 添加 Store 状态**

```typescript
const systemInfo = ref<{ cpu_cores: number; memory_gb: number; has_gpu: boolean; gpu_name: string | null } | null>(null);
const isPulling = ref(false);
const pullProgress = ref<any>(null);

async function fetchSystemInfo() {
  systemInfo.value = await getSystemInfo();
}

async function pullModel(model: string) {
  isPulling.value = true;
  pullProgress.value = null;
  
  try {
    await pullModelCmd(model);
    await startOllama();
    await fetchModels();
  } finally {
    isPulling.value = false;
  }
}
```

- [ ] **Step 4: 运行类型检查**

Run: `cd /Users/mac/project/telepathy && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/api/tauri.ts src/stores/settings.ts
git commit -m "feat: add model management API and store state"
```

---

### Task 3: 前端 - 修改 Settings.vue 添加模型选择 UI

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: 添加系统信息显示**

```vue
<div v-if="systemInfo" class="system-info">
  <n-card title="🖥️ 您的电脑配置" size="small">
    <n-descriptions :column="2">
      <n-descriptions-item label="CPU 核心">{{ systemInfo.cpu_cores }} 核</n-descriptions-item>
      <n-descriptions-item label="内存">{{ systemInfo.memory_gb }} GB</n-descriptions-item>
    </n-descriptions>
  </n-card>
</div>
```

- [ ] **Step 2: 添加模型推荐**

```vue
<div v-if="ollamaInstalledPath && !store.models.length" class="model-selection">
  <h3>💡 推荐模型（根据您的配置）</h3>
  <n-space vertical>
    <n-card v-for="model in recommendedModels" :key="model.id" hoverable @click="downloadModel(model.id)">
      <n-space justify="space-between">
        <div>
          <strong>{{ model.name }}:{{ model.size }}</strong>
          <p>{{ model.desc }}</p>
        </div>
        <n-tag>{{ model.memory }}</n-tag>
      </n-space>
    </n-card>
  </n-space>
</div>
```

- [ ] **Step 3: 添加下载进度显示**

```vue
<div v-if="isPulling" class="pull-progress">
  <n-progress type="line" :percentage="pullProgress?.percentage || 0" />
  <p>正在下载 {{ pullProgress?.model }}...</p>
</div>
```

- [ ] **Step 4: 添加样式**

```css
.system-info {
  margin-bottom: 16px;
}

.model-selection {
  margin-top: 16px;
}

.model-selection h3 {
  margin-bottom: 12px;
}
```

- [ ] **Step 5: 运行类型检查**

Run: `cd /Users/mac/project/telepathy && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src/views/Settings.vue
git commit -m "feat: add model selection UI in settings"
```

---

## 验收标准检查

- [x] 自动检测电脑配置并推荐适合的模型
- [x] 显示预设模型列表
- [x] 支持下载模型
- [x] 显示下载进度
- [x] 下载完成后自动启动 Ollama
- [x] 提示用户可以开始使用
