# P6 扩展: Ollama 直接下载安装 - 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现一键下载安装 Ollama 功能，用户点击按钮即可完成下载，显示进度

**Architecture:** 后端 Rust 处理平台检测和下载逻辑，前端 Vue 显示进度。安装到用户目录 ~/.telepathy/ollama

**Tech Stack:** Rust + Tauri + Vue 3 + TypeScript

---

### Task 1: 后端 - 添加 Ollama 安装命令

**Files:**
- Create: `src-tauri/src/commands/installer.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: 创建 installer.rs**

```rust
use crate::errors::AppError;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

#[tauri::command]
pub fn get_platform() -> PlatformInfo {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    PlatformInfo { os, arch }
}

fn get_install_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or_else(|| {
        AppError::Internal("Cannot find home directory".to_string())
    })?;
    let install_dir = home.join(".telepathy").join("ollama");
    if !install_dir.exists() {
        fs::create_dir_all(&install_dir).map_err(|e| {
            AppError::Internal(format!("Failed to create install directory: {}", e))
        })?;
    }
    Ok(install_dir)
}

#[tauri::command]
pub fn get_ollama_install_path() -> Result<String, AppError> {
    let install_dir = get_install_dir()?;
    let ollama_path = install_dir.join("ollama");
    if ollama_path.exists() {
        Ok(ollama_path.to_string_lossy().to_string())
    } else {
        Err(AppError::Internal("Ollama not installed".to_string()))
    }
}

#[tauri::command]
pub async fn download_ollama(
    app: tauri::AppHandle,
) -> Result<String, AppError> {
    let install_dir = get_install_dir()?;
    let ollama_path = install_dir.join("ollama");
    
    if ollama_path.exists() {
        return Ok(ollama_path.to_string_lossy().to_string());
    }
    
    let (url, filename) = {
        let os = std::env::consts::OS;
        if os == "macos" {
            ("https://ollama.com/download/Ollama-darwin.zip", "Ollama-darwin.zip")
        } else if os == "linux" {
            ("https://ollama.com/download/Ollama-linux.zip", "Ollama-linux.zip")
        } else {
            return Err(AppError::Internal("Unsupported platform".to_string()));
        }
    };
    
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.map_err(|e| {
        AppError::Internal(format!("Failed to download: {}", e))
    })?;
    
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut file_data = Vec::new();
    
    use futures_util::stream::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Internal(format!("Download error: {}", e)))?;
        downloaded += chunk.len() as u64;
        file_data.extend_from_slice(&chunk);
        
        if total_size > 0 {
            let progress = (downloaded as f64 / total_size as f64 * 100.0) as u32;
            let _ = app.emit("ollama-download-progress", progress);
        }
    }
    
    let temp_zip = install_dir.join(filename);
    fs::write(&temp_zip, &file_data).map_err(|e| {
        AppError::Internal(format!("Failed to save file: {}", e))
    })?;
    
    let ollama_executable = extract_ollama(&temp_zip, &install_dir)?;
    
    fs::remove_file(&temp_zip).ok();
    
    Ok(ollama_executable.to_string_lossy().to_string())
}

fn extract_ollama(zip_path: &PathBuf, dest_dir: &PathBuf) -> Result<PathBuf, AppError> {
    let file = fs::File::open(zip_path).map_err(|e| {
        AppError::Internal(format!("Failed to open zip: {}", e))
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        AppError::Internal(format!("Failed to read zip: {}", e))
    })?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            AppError::Internal(format!("Failed to read zip entry: {}", e))
        })?;
        let outpath = dest_dir.join(file.name());
        
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).ok();
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| {
                AppError::Internal(format!("Failed to extract: {}", e))
            })?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| {
                AppError::Internal(format!("Failed to write: {}", e))
            })?;
        }
    }
    
    let ollama_executable = dest_dir.join("ollama");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&ollama_executable).map_err(|e| {
            AppError::Internal(format!("Failed to get metadata: {}", e))
        })?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&ollama_executable, perms).map_err(|e| {
            AppError::Internal(format!("Failed to set permissions: {}", e))
        })?;
    }
    
    Ok(ollama_executable)
}
```

- [ ] **Step 2: 修改 mod.rs 导出新模块**

```rust
pub mod chat;
pub mod document;
pub mod greet;
pub mod indexing;
pub mod installer;  // 新增
pub mod rag;
pub mod settings;
```

- [ ] **Step 3: 添加依赖到 Cargo.toml**

在 `src-tauri/Cargo.toml` 添加：
```toml
[dependencies]
dirs = "5"
futures-util = "0.3"
zip = "0.6"
```

- [ ] **Step 4: 运行 cargo check**

Run: `cd /Users/mac/project/telepathy && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/installer.rs src-tauri/src/commands/mod.rs src-tauri/Cargo.toml
git commit -m "feat: add Ollama download installer commands"
```

---

### Task 2: 前端 - 添加安装状态到 Store

**Files:**
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: 添加安装相关状态**

```typescript
const isDownloading = ref(false);
const downloadProgress = ref(0);
const ollamaInstalledPath = ref('');
const downloadError = ref('');
```

- [ ] **Step 2: 添加安装函数**

```typescript
async function downloadOllama() {
  downloadError.value = '';
  isDownloading.value = true;
  downloadProgress.value = 0;
  try {
    const path = await downloadOllamaCmd();
    ollamaInstalledPath.value = path;
    messageApi.success('Ollama 下载完成！');
  } catch (e: any) {
    downloadError.value = e.message || String(e);
    messageApi.error(`下载失败: ${downloadError.value}`);
  } finally {
    isDownloading.value = false;
  }
}

function setDownloadProgress(progress: number) {
  downloadProgress.value = progress;
}
```

- [ ] **Step 3: 导出新状态和函数**

```typescript
return {
  settings, models, isLoading,
  ollamaStatus, ollamaError,
  isDownloading, downloadProgress, ollamaInstalledPath, downloadError,
  fetchSettings, saveSetting, fetchModels,
  downloadOllama, setDownloadProgress
};
```

- [ ] **Step 4: 运行类型检查**

Run: `cd /Users/mac/project/telepathy && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/stores/settings.ts
git commit -m "feat: add Ollama download state to settings store"
```

---

### Task 3: 前端 - 修改 Settings.vue 添加下载 UI

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: 导入新的 store 状态**

```typescript
const isDownloading = computed(() => store.isDownloading);
const downloadProgress = computed(() => store.downloadProgress);
const ollamaInstalledPath = computed(() => store.ollamaInstalledPath);
const downloadError = computed(() => store.downloadError);
```

- [ ] **Step 2: 修改卸载安装卡片的模板**

在 uninstalled 状态卡片中添加一键安装按钮和下载进度：

```vue
<n-card v-if="ollamaStatus === 'uninstalled'" class="guide-card" :bordered="false">
  <div class="guide-content">
    <n-icon size="48" color="#18a058">
      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>
    </n-icon>
    <h3>Ollama 未安装</h3>
    <p v-if="!isDownloading">Ollama 是本地大语言模型运行时，安装后才能使用 AI 对话和知识库检索功能。</p>
    
    <!-- 下载进度显示 -->
    <div v-if="isDownloading" class="download-progress">
      <n-progress
        type="line"
        :percentage="downloadProgress"
        :indicator-placement="'inside'"
      />
      <p>正在下载 Ollama... ({{ downloadProgress }}%)</p>
    </div>
    
    <!-- 安装成功提示 -->
    <div v-else-if="ollamaInstalledPath" class="install-success">
      <n-alert type="success" :show-icon="true">
        Ollama 下载完成！路径: {{ ollamaInstalledPath }}
      </n-alert>
    </div>
    
    <n-space v-if="!isDownloading && !ollamaInstalledPath" vertical :size="12">
      <n-text depth="3">安装命令：</n-text>
      <n-code code="brew install ollama" language="bash" />
    </n-space>
    
    <n-space style="margin-top: 16px">
      <n-button v-if="!isDownloading && !ollamaInstalledPath" @click="store.downloadOllama()">
        一键安装
      </n-button>
      <n-button v-if="!isDownloading && !ollamaInstalledPath" @click="copyInstallCommand('macos')">
        复制安装命令
      </n-button>
      <n-button v-if="!isDownloading && !ollamaInstalledPath" @click="openOllamaWebsite">
        打开下载页面
      </n-button>
      <n-button type="primary" @click="store.fetchModels()">
        检查状态
      </n-button>
    </n-space>
  </div>
</n-card>
```

- [ ] **Step 3: 添加下载进度样式**

```css
.download-progress {
  width: 100%;
  max-width: 300px;
  margin: 16px 0;
}

.download-progress p {
  text-align: center;
  color: #666;
  margin-top: 8px;
}

.install-success {
  width: 100%;
  max-width: 400px;
  margin: 12px 0;
}
```

- [ ] **Step 4: 运行类型检查**

Run: `cd /Users/mac/project/telepathy && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/views/Settings.vue
git commit -m "feat: add one-click install UI with progress"
```

---

### Task 4: 验证功能

**Files:**
- Test: 手动测试

- [ ] **Step 1: 启动开发服务器**

Run: `cd /Users/mac/project/telepathy && pnpm tauri dev`
Expected: 应用启动

- [ ] **Step 2: 导航到设置页面**

- [ ] **Step 3: 验证未安装状态显示一键安装按钮**

- [ ] **Step 4: 点击一键安装，验证下载进度显示**

- [ ] **Step 5: 验证安装完成后的提示**

- [ ] **Step 6: Commit**

```bash
git commit -m "test: verify one-click install works"
```

---

## 验收标准检查

- [x] 支持 macOS 和 Linux 平台自动检测
- [x] 一键安装按钮触发下载
- [x] 显示实时下载进度百分比
- [x] 下载完成后显示安装路径
- [x] 安装完成后可检测状态
