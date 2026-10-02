# Mac 一键安装 Ollama 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重写 Mac 上 Ollama 安装逻辑，实现自动检测并安装 Homebrew + Ollama 的一键安装流程，使用 macOS 系统授权对话框处理权限。

**Architecture:** 后端使用 osascript 调用 macOS 系统授权获取密码，按检测链依次检查 Ollama → Homebrew → 安装缺失组件。前端优化 Settings 页面 UI，显示安装步骤状态和进度。安装路径统一使用系统路径。

**Tech Stack:** Tauri v2, Rust, Vue 3, Naive UI, macOS osascript

---

### Task 1: 重写后端安装核心逻辑

**Files:**
- Modify: `src-tauri/src/commands/installer.rs`

- [ ] **Step 1: 添加新的辅助函数**

在 `src-tauri/src/commands/installer.rs` 中，在现有代码后添加以下辅助函数（保留现有的 `get_platform`、`get_telepathy_dir`、`find_system_ollama`、`get_system_info`、`detect_gpu`、`SystemInfo` 不变，删除 `get_ollama_install_path`、`check_ollama_installed`、`get_ollama_path`、`find_ollama_binary` 这些旧的/未使用的函数，因为新逻辑使用系统路径检测）：

```rust
/// 检测 Homebrew是否已安装
fn is_homebrew_installed() -> bool {
    std::process::Command::new("brew")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 通过系统授权执行命令（弹出 macOS 密码输入框）
fn run_with_privilege(command: &str) -> Result<String, AppError> {
    let script = format!(
        "do shell script \"{}\" with administrator privileges",
        command.replace("\"", "\\\"")
    );
    let output = std::process::Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to run osascript: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("User canceled") {
            return Err(AppError::Internal("用户取消了授权".to_string()));
        }
        return Err(AppError::Internal(format!("授权执行失败: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 检测 Ollama 是否已安装（系统路径 + 历史兼容路径）
fn find_ollama_binary() -> Option<String> {
    // 优先检查 Homebrew 系统路径
    let system_paths = [
        "/opt/homebrew/bin/ollama",  // Apple Silicon
        "/usr/local/bin/ollama",     // Intel
    ];
    for path in &system_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    // 其次检查 which 命令
    if let Ok(output) = std::process::Command::new("which").arg("ollama").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }

    // 历史兼容路径
    if let Ok(home) = dirs::home_dir() {
        let legacy_path = home.join(".telepathy/ollama/ollama");
        if legacy_path.exists() {
            return Some(legacy_path.to_string_lossy().to_string());
        }
    }

    None
}

/// 检测 Ollama 服务是否正在运行
fn is_ollama_running() -> bool {
    let output = std::process::Command::new("curl")
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "http://localhost:11434"])
        .output();
    match output {
        Ok(o) => {
            let code = String::from_utf8_lossy(&o.stdout);
            code.trim() == "200"
        }
        Err(_) => false,
    }
}

/// 获取数据库路径（复用 settings.rs 的逻辑）
fn get_db_path_for_settings(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
    }
    Ok(app_data_dir.join("telepathy.db"))
}
```

**注意**: `reqwest::blocking` 需要确认 Cargo.toml 中是否启用了 `blocking` feature。如果没有，改用 `std::process::Command::new("curl").args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "http://localhost:11434"])` 来检测。

- [ ] **Step 2: 重写 download_ollama 命令**

替换现有的 `download_ollama` 函数:

```rust
#[tauri::command]
pub async fn download_ollama(app: AppHandle) -> Result<String, AppError> {
    // Step 1: 检测 Ollama 是否已存在
    if let Some(path) = find_ollama_binary() {
        // Ollama 已安装，检查服务是否运行
        if !is_ollama_running() {
            let _ = app.emit(
                "ollama-download-progress",
                DownloadProgress {
                    stage: "starting_ollama".to_string(),
                    percentage: 0.0,
                    message: "正在启动 Ollama 服务...".to_string(),
                },
            );
            // 启动服务（不需要权限）
            tokio::process::Command::new(&path)
                .args(["serve"])
                .spawn()
                .map_err(|e| AppError::Internal(format!("Failed to start Ollama: {}", e)))?;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
        return Ok(path);
    }

    // Step 2: 检测 Homebrew
    if !is_homebrew_installed() {
        let _ = app.emit(
            "ollama-download-progress",
            DownloadProgress {
                stage: "installing_homebrew".to_string(),
                percentage: 0.0,
                message: "正在安装 Homebrew 包管理器...".to_string(),
            },
        );

        // 使用 osascript 弹出系统密码框安装 Homebrew
        let homebrew_install_cmd = r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#;
        run_with_privilege(homebrew_install_cmd)?;

        // 验证 Homebrew 安装成功
        if !is_homebrew_installed() {
            return Err(AppError::Internal(
                "Homebrew 安装失败，请手动安装后重试".to_string(),
            ));
        }
    }

    // Step 3: 安装 Ollama
    let _ = app.emit(
        "ollama-download-progress",
        DownloadProgress {
            stage: "installing_ollama".to_string(),
            percentage: 0.0,
            message: "正在安装 Ollama...".to_string(),
        },
    );

    // brew install 不需要管理员权限（Homebrew 在用户目录有写入权限）
    let output = tokio::process::Command::new("brew")
        .args(["install", "ollama"])
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to run brew install: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already installed") {
            return Err(AppError::Internal(format!("安装失败: {}", stderr)));
        }
    }

    // Step 4: 验证安装并获取路径
    let ollama_path = find_ollama_binary()
        .ok_or_else(|| AppError::Internal("Ollama 安装完成但未找到可执行文件".to_string()))?;

    // Step 5: 启动 Ollama 服务
    let _ = app.emit(
        "ollama-download-progress",
        DownloadProgress {
            stage: "starting_ollama".to_string(),
            percentage: 0.0,
            message: "正在启动 Ollama 服务...".to_string(),
        },
    );

    tokio::process::Command::new(&ollama_path)
        .args(["serve"])
        .spawn()
        .map_err(|e| AppError::Internal(format!("Failed to start Ollama: {}", e)))?;

    // 等待服务启动
    let mut retries = 15;
    while retries > 0 {
        if is_ollama_running() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        retries -= 1;
    }

    if !is_ollama_running() {
        return Err(AppError::Internal("Ollama 服务启动超时".to_string()));
    }

    // Step 6: 保存路径到 settings（供后续使用）
    {
        let db_path = match get_db_path_for_settings(&app) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to get db path: {}", e);
                // 不影响安装成功，继续
                String::new()
            }
        };
        if !db_path.is_empty() {
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let _ = crate::db::settings::update_setting(&conn, "ollama_binary_path", &ollama_path);
            }
        }
    }

    // Step 6: 发送完成通知
    let _ = app.emit(
        "ollama-download-progress",
        DownloadProgress {
            stage: "done".to_string(),
            percentage: 100.0,
            message: "安装完成！".to_string(),
        },
    );

    use crate::commands::notifications::emit_notification;
    use crate::db::notifications;
    let notification = notifications::create_notification(
        "ollama_install_complete",
        "Ollama 安装完成",
        "Ollama 已成功安装并启动",
        Some("/settings"),
    );
    emit_notification(&app, notification);

    Ok(ollama_path)
}
```

- [ ] **Step 3: 更新 check_ollama_installed 和 get_ollama_path**

替换这两个命令使用新的 `find_ollama_binary`:

```rust
#[tauri::command]
pub async fn check_ollama_installed() -> Result<bool, AppError> {
    Ok(find_ollama_binary().is_some())
}

#[tauri::command]
pub async fn get_ollama_path() -> Result<Option<String>, AppError> {
    Ok(find_ollama_binary())
}
```

- [ ] **Step 4: 删除不再使用的函数**

删除 `get_ollama_install_path` 和 `find_ollama_binary`（旧的）函数。

- [ ] **Step 5: 验证编译**

运行 `cd src-tauri && cargo check` 确保编译通过。

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/commands/installer.rs
git commit -m "refactor: rewrite Ollama installer with one-click install and system authorization"
```

---

### Task 2: 优化前端 Settings 安装 UI

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: 更新未安装状态的 UI**

修改 Settings.vue 中 `ollamaStatus === 'uninstalled'` 状态下的安装引导区域，替换旧的警告提示和多个按钮为更清晰的一键安装流程：

将 `v-if="!isDownloading && !ollamaInstalledPath"` 区域内的内容替换为：

```vue
<n-space v-if="!isDownloading && !ollamaInstalledPath" vertical :size="16">
  <p class="install-desc">点击下方按钮，应用将自动检测并安装所有依赖（Homebrew + Ollama）</p>
  <n-button type="primary" size="large" @click="handleDownload">
    一键安装 Ollama
  </n-button>
  <n-divider style="margin: 8px 0" />
  <n-text depth="3" style="font-size: 12px;">安装过程中需要输入系统密码以授权</n-text>
</n-space>
```

在 `<style scoped>` 中添加：

```css
.install-desc {
  color: #999;
  font-size: 14px;
  margin: 0;
}
```

- [ ] **Step 2: 提交**

```bash
git add src/views/Settings.vue
git commit -m "ui: simplify Ollama install flow to one-click button"
```

---

### Task 3: 验证和测试

- [ ] **Step 1: 运行 TypeScript 类型检查**

```bash
npx vue-tsc --noEmit
```

预期: 无错误输出

- [ ] **Step 2: 运行 Rust 编译检查**

```bash
cd src-tauri && cargo check
```

预期: 编译通过，无新增错误

- [ ] **Step 3: 提交**

```bash
git add .
git commit -m "chore: fix type errors"
```
