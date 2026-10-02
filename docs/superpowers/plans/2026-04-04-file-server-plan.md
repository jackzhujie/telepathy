# 资源分发服务器 + 应用端更新实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 搭建 file.aizhifou.cn 资源分发服务器，实现 Ollama 二进制自动同步，应用端从该服务器下载。

**Architecture:** 服务器端使用 Nginx 提供静态文件下载 + JSON API，cron 定时脚本从 GitHub 同步 Ollama 二进制。应用端请求 `/api/latest.json` 获取下载链接，按架构选择对应文件下载。

**Tech Stack:** Nginx, Bash, Rust (Tauri), Vue 3, TypeScript

---

### Task 1: 服务器端 - 目录结构和 Nginx 配置

**Files (on server):**
- Create: `/var/www/file.aizhifou.cn/downloads/ollama/`
- Create: `/var/www/file.aizhifou.cn/api/`
- Create: `/var/www/file.aizhifou.cn/sync.sh`
- Create: `/etc/nginx/sites-available/file.aizhifou.cn`

- [ ] **Step 1: 创建目录结构**

在服务器上执行：

```bash
mkdir -p /var/www/file.aizhifou.cn/downloads/ollama
mkdir -p /var/www/file.aizhifou.cn/downloads/telepathy
mkdir -p /var/www/file.aizhifou.cn/api
```

- [ ] **Step 2: 创建 Nginx 配置**

创建 `/etc/nginx/sites-available/file.aizhifou.cn`：

```nginx
server {
    listen 443 ssl http2;
    server_name file.aizhifou.cn;

    ssl_certificate /etc/letsencrypt/live/file.aizhifou.cn/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/file.aizhifou.cn/privkey.pem;

    root /var/www/file.aizhifou.cn;

    # 静态文件下载
    location /downloads/ {
        alias /var/www/file.aizhifou.cn/downloads/;
        autoindex off;
        client_max_body_size 0;
        sendfile on;
        tcp_nopush on;
        add_header Access-Control-Allow-Origin *;
        add_header Cache-Control "public, max-age=3600";
    }

    # 版本 API
    location = /api/latest.json {
        alias /var/www/file.aizhifou.cn/api/latest.json;
        default_type application/json;
        add_header Access-Control-Allow-Origin *;
        add_header Cache-Control "no-cache";
    }

    # 默认返回 404
    location / {
        return 404;
    }
}
```

启用站点：

```bash
ln -s /etc/nginx/sites-available/file.aizhifou.cn /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx
```

**注意**: SSL 证书使用 Let's Encrypt，如果还没有证书，先运行 `certbot --nginx -d file.aizhifou.cn`。

- [ ] **Step 3: 创建初始 latest.json**

创建 `/var/www/file.aizhifou.cn/api/latest.json`：

```json
{
  "ollama": {
    "version": "",
    "files": {
      "darwin-arm64": {
        "url": "https://file.aizhifou.cn/downloads/ollama/darwin-arm64",
        "sha256": ""
      },
      "darwin-amd64": {
        "url": "https://file.aizhifou.cn/downloads/ollama/darwin-amd64",
        "sha256": ""
      }
    }
  },
  "app": {
    "version": "0.1.0",
    "release_notes": "",
    "files": {}
  }
}
```

- [ ] **Step 4: 验证 Nginx 配置**

```bash
curl -I https://file.aizhifou.cn/api/latest.json
```

预期返回 `HTTP/2 200` 且 `Content-Type: application/json`。

---

### Task 2: 服务器端 - 自动同步脚本

**Files (on server):**
- Create: `/var/www/file.aizhifou.cn/sync.sh`

- [ ] **Step 1: 创建同步脚本**

创建 `/var/www/file.aizhifou.cn/sync.sh`：

```bash
#!/bin/bash
set -e

DOWNLOAD_DIR="/var/www/file.aizhifou.cn/downloads/ollama"
API_FILE="/var/www/file.aizhifou.cn/api/latest.json"
BASE_URL="https://file.aizhifou.cn"

mkdir -p "$DOWNLOAD_DIR"

echo "[$(date)] Starting Ollama sync..."

# 获取最新 Release 信息
RELEASE=$(curl -s --max-time 30 https://api.github.com/repos/ollama/ollama/releases/latest)
if [ -z "$RELEASE" ] || echo "$RELEASE" | grep -q "API rate limit"; then
    echo "[$(date)] Failed to fetch release info"
    exit 1
fi

TAG=$(echo "$RELEASE" | grep -o '"tag_name": *"[^"]*"' | cut -d'"' -f4)
echo "[$(date)] Latest version: $TAG"

# 下载 darwin-arm64
ARM64_URL=$(echo "$RELEASE" | grep -o '"browser_download_url": *"[^"]*darwin-arm64[^"]*"' | cut -d'"' -f4)
if [ -n "$ARM64_URL" ]; then
    echo "[$(date)] Downloading darwin-arm64..."
    curl -fSL --max-time 600 "$ARM64_URL" -o "$DOWNLOAD_DIR/darwin-arm64.tmp"
    mv "$DOWNLOAD_DIR/darwin-arm64.tmp" "$DOWNLOAD_DIR/darwin-arm64"
    chmod 755 "$DOWNLOAD_DIR/darwin-arm64"
    ARM64_SHA=$(sha256sum "$DOWNLOAD_DIR/darwin-arm64" | awk '{print $1}')
    echo "[$(date)] darwin-arm64 SHA256: $ARM64_SHA"
else
    echo "[$(date)] WARNING: darwin-arm64 not found"
    ARM64_SHA=""
fi

# 下载 darwin-amd64
AMD64_URL=$(echo "$RELEASE" | grep -o '"browser_download_url": *"[^"]*darwin-amd64[^"]*"' | cut -d'"' -f4)
if [ -n "$AMD64_URL" ]; then
    echo "[$(date)] Downloading darwin-amd64..."
    curl -fSL --max-time 600 "$AMD64_URL" -o "$DOWNLOAD_DIR/darwin-amd64.tmp"
    mv "$DOWNLOAD_DIR/darwin-amd64.tmp" "$DOWNLOAD_DIR/darwin-amd64"
    chmod 755 "$DOWNLOAD_DIR/darwin-amd64"
    AMD64_SHA=$(sha256sum "$DOWNLOAD_DIR/darwin-amd64" | awk '{print $1}')
    echo "[$(date)] darwin-amd64 SHA256: $AMD64_SHA"
else
    echo "[$(date)] WARNING: darwin-amd64 not found"
    AMD64_SHA=""
fi

# 生成 API 响应
cat > "$API_FILE" << EOF
{
  "ollama": {
    "version": "$TAG",
    "files": {
      "darwin-arm64": {
        "url": "$BASE_URL/downloads/ollama/darwin-arm64",
        "sha256": "$ARM64_SHA"
      },
      "darwin-amd64": {
        "url": "$BASE_URL/downloads/ollama/darwin-amd64",
        "sha256": "$AMD64_SHA"
      }
    }
  },
  "app": {
    "version": "0.1.0",
    "release_notes": "",
    "files": {}
  }
}
EOF

echo "[$(date)] Sync complete!"
```

- [ ] **Step 2: 设置可执行权限并测试**

```bash
chmod +x /var/www/file.aizhifou.cn/sync.sh
/var/www/file.aizhifou.cn/sync.sh
```

- [ ] **Step 3: 设置定时任务**

```bash
(crontab -l 2>/dev/null; echo "0 4 * * * /var/www/file.aizhifou.cn/sync.sh >> /var/log/ollama-sync.log 2>&1") | crontab -
```

---

### Task 3: 应用端 - 添加更新检查 API

**Files:**
- Modify: `src/api/tauri.ts`
- Create: `src/types/update.ts`

- [ ] **Step 1: 创建更新类型定义**

创建 `src/types/update.ts`：

```typescript
export interface FileDownloadInfo {
  url: string;
  sha256: string;
}

export interface UpdateInfo {
  ollama: {
    version: string;
    files: Record<string, FileDownloadInfo>;
  };
  app: {
    version: string;
    release_notes: string;
    files: Record<string, FileDownloadInfo>;
  };
}
```

- [ ] **Step 2: 添加更新检查 API 函数**

在 `src/api/tauri.ts` 中添加：

```typescript
import type { UpdateInfo } from '@/types/update';

const UPDATE_SERVER = 'https://file.aizhifou.cn';

export async function checkUpdates(): Promise<UpdateInfo> {
  const resp = await fetch(`${UPDATE_SERVER}/api/latest.json`);
  if (!resp.ok) throw new Error('Failed to fetch update info');
  return resp.json();
}
```

- [ ] **Step 3: 提交**

```bash
git add src/types/update.ts src/api/tauri.ts
git commit -m "feat: add update check API for file server"
```

---

### Task 4: 应用端 - 重写下载逻辑使用文件服务器

**Files:**
- Modify: `src-tauri/src/commands/installer.rs`

- [ ] **Step 1: 添加更新服务器常量和类型**

在 `src-tauri/src/commands/installer.rs` 顶部添加：

```rust
const UPDATE_SERVER: &str = "https://file.aizhifou.cn";

#[derive(Debug, serde::Deserialize)]
struct FileDownloadInfo {
    url: String,
    sha256: String,
}

#[derive(Debug, serde::Deserialize)]
struct OllamaUpdateInfo {
    version: String,
    files: std::collections::HashMap<String, FileDownloadInfo>,
}

#[derive(Debug, serde::Deserialize)]
struct AppUpdateInfo {
    version: String,
    release_notes: String,
    files: std::collections::HashMap<String, FileDownloadInfo>,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateInfo {
    ollama: OllamaUpdateInfo,
    app: AppUpdateInfo,
}
```

- [ ] **Step 2: 重写 download_ollama_binary 函数**

替换现有的 `download_ollama_binary` 函数：

```rust
/// 从文件服务器下载 Ollama 二进制文件
async fn download_ollama_binary(app: &AppHandle) -> Result<String, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("Cannot find home directory".to_string()))?;
    let ollama_dir = home.join(".telepathy/ollama");
    
    if !ollama_dir.exists() {
        std::fs::create_dir_all(&ollama_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create ollama dir: {}", e)))?;
    }

    // 检测架构
    let arch = if cfg!(target_arch = "aarch64") {
        "darwin-arm64"
    } else {
        "darwin-amd64"
    };

    let _ = app.emit(
        "ollama-download-progress",
        DownloadProgress {
            stage: "fetching_info".to_string(),
            percentage: 0.0,
            message: "正在获取最新版本...".to_string(),
        },
    );

    // 从文件服务器获取版本信息
    let client = reqwest::Client::builder()
        .user_agent("Telepathy/0.1.0")
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let update_info: UpdateInfo = client
        .get(&format!("{}/api/latest.json", UPDATE_SERVER))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch update info: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse update info: {}", e)))?;

    let file_info = update_info.ollama.files.get(arch)
        .ok_or_else(|| AppError::Internal(format!("No download available for your architecture ({})", arch)))?;

    let _ = app.emit(
        "ollama-download-progress",
        DownloadProgress {
            stage: "downloading".to_string(),
            percentage: 0.0,
            message: format!("正在下载 Ollama ({})...", arch),
        },
    );

    // 下载二进制
    let response = client.get(&file_info.url).send().await
        .map_err(|e| AppError::Internal(format!("Failed to download Ollama: {}", e)))?;

    let bytes = response.bytes().await
        .map_err(|e| AppError::Internal(format!("Failed to read Ollama binary: {}", e)))?;

    // 验证 SHA256
    if !file_info.sha256.is_empty() {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_sha = format!("{:x}", hasher.finalize());
        
        if actual_sha != file_info.sha256 {
            return Err(AppError::Internal(format!(
                "SHA256 校验失败: 期望 {} 实际 {}",
                file_info.sha256, actual_sha
            )));
        }
    }

    let _ = app.emit(
        "ollama-download-progress",
        DownloadProgress {
            stage: "installing".to_string(),
            percentage: 100.0,
            message: "正在安装...".to_string(),
        },
    );

    // 写入文件
    let ollama_bin = ollama_dir.join("ollama");
    std::fs::write(&ollama_bin, &bytes)
        .map_err(|e| AppError::Internal(format!("Failed to write Ollama binary: {}", e)))?;

    // 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&ollama_bin)
            .map_err(|e| AppError::Internal(format!("Failed to read file metadata: {}", e)))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&ollama_bin, perms)
            .map_err(|e| AppError::Internal(format!("Failed to set executable permission: {}", e)))?;
    }

    Ok(ollama_bin.to_string_lossy().to_string())
}
```

- [ ] **Step 3: 添加 sha2 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
sha2 = "0.10"
```

- [ ] **Step 4: 删除不再使用的函数**

删除 `is_homebrew_installed` 和 `run_with_privilege` 函数（不再需要 Homebrew 相关逻辑）。

- [ ] **Step 5: 验证编译**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 6: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/src/commands/installer.rs
git commit -m "refactor: download Ollama from file server instead of GitHub"
```

---

### Task 5: 验证端到端流程

- [ ] **Step 1: 在服务器上运行同步脚本**

```bash
/var/www/file.aizhifou.cn/sync.sh
```

确认文件下载成功且 `latest.json` 格式正确。

- [ ] **Step 2: 验证 API 可访问**

```bash
curl https://file.aizhifou.cn/api/latest.json
```

确认返回正确的 JSON。

- [ ] **Step 3: 运行应用端类型检查**

```bash
npx vue-tsc --noEmit
```

- [ ] **Step 4: 运行 Rust 编译检查**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 5: 提交**

```bash
git add .
git commit -m "chore: final verification fixes"
```
