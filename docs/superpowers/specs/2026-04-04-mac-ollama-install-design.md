# Mac 一键安装 Ollama 设计

## 目标
实现 Mac 上一键安装 Ollama，自动检测并安装所有依赖（Homebrew + Ollama），用户只需点击一个按钮。

## 安装检测链

```
一键安装按钮
  ↓
检测 Ollama 是否已安装（系统路径 + 自定义路径）
  ↓ 未安装
检测 Homebrew 是否已安装
  ↓ 未安装
弹出系统授权 → 执行 Homebrew 安装脚本
  ↓
执行 brew install ollama
  ↓
验证安装成功 → 返回路径 → 自动启动 Ollama 服务
```

## 权限处理

使用 `osascript` 调用 macOS `do shell script` 的 `with administrator privileges`，系统弹出标准密码输入框。这是 macOS 标准做法，无需额外依赖。

```bash
osascript -e 'do shell script "命令" with administrator privileges'
```

## 路径策略

检测优先级：
1. `/opt/homebrew/bin/ollama` (Apple Silicon Homebrew)
2. `/usr/local/bin/ollama` (Intel Homebrew)
3. `~/.telepathy/ollama/ollama` (历史兼容)

安装完成后，路径写入 SQLite settings 表，后续直接使用。

## 进度反馈

通过 Tauri Event 发送安装进度到前端：

| 阶段 | 消息 |
|------|------|
| checking_ollama | 正在检测 Ollama... |
| checking_homebrew | 正在检测 Homebrew... |
| installing_homebrew | 正在安装 Homebrew 包管理器... |
| installing_ollama | 正在安装 Ollama... |
| starting_ollama | 正在启动 Ollama 服务... |
| done | 安装完成！ |

## 错误处理

- **Homebrew 安装失败**: 提示用户手动安装，提供"复制安装命令"按钮
- **brew install 失败**: 检查网络、权限、磁盘空间，返回具体错误信息
- **用户取消授权**: 返回"用户取消操作"提示
- **Ollama 启动失败**: 检查端口 11434 是否被占用

## 文件变更清单

### 修改文件
1. `src-tauri/src/commands/installer.rs` - 重写安装逻辑
2. `src-tauri/src/commands/settings.rs` - 新增保存 ollama 路径的函数
3. `src/views/Settings.vue` - 优化安装 UI，显示安装步骤状态

### 新增文件
无

## 关键实现细节

### 1. Homebrew 检测

```rust
fn is_homebrew_installed() -> bool {
    Command::new("brew")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

### 2. 系统授权执行

```rust
fn run_with_privilege(command: &str) -> Result<String, AppError> {
    let script = format!(
        "do shell script \"{}\" with administrator privileges",
        command.replace("\"", "\\\"")
    );
    let output = Command::new("osascript")
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
```

### 3. 安装流程函数签名

```rust
#[tauri::command]
pub async fn download_ollama(app: AppHandle) -> Result<String, AppError>
```

流程：
1. 检测 Ollama 已存在 → 直接返回路径
2. 检测 Homebrew → 未安装则通过 osascript 安装
3. `brew install ollama`
4. 查找安装路径 → 写入 settings
5. 启动 Ollama 服务
6. 发送完成通知
