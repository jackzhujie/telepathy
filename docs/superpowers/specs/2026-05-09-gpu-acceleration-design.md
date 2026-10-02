# GPU 加速智能检测设计

**日期**: 2026-05-09
**模块**: 模型性能优化
**状态**: 设计完成

---

## 1. 背景

当前 `llama_adapter.rs` 中硬编码 `n_gpu_layers = 0`，禁用了所有 GPU 加速。原因是 Metal 在 llama-cpp-2 中存在已知问题。

## 2. 设计方案

### 2.1 智能检测逻辑

```rust
fn get_gpu_layers(num_gpu_setting: i32) -> u32 {
    // 用户明确设置为 0 = 禁用
    if num_gpu_setting == 0 {
        return 0;
    }

    // 检测操作系统
    #[cfg(target_os = "macos")]
    {
        // Mac M 系列：尝试启用 Metal
        if is_m_series_mac() {
            println!("[GPU] Detected Mac M-series, enabling Metal acceleration");
            return num_gpu_setting.max(1) as u32;
        }
    }

    // 非 Mac 或非 M 系列：使用用户设置
    num_gpu_setting.max(0) as u32
}

fn is_m_series_mac() -> bool {
    // 检测是否运行在 Apple Silicon 上
    std::process::Command::new("uname")
        .arg("-m")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("arm64"))
        .unwrap_or(false)
}
```

### 2.2 修改位置

**文件**: `src-tauri/src/services/inference/llama_adapter.rs`

```rust
// 替换第 72 行
let n_gpu_layers = get_gpu_layers(options.n_gpu_layers);
```

## 3. 行为说明

| 场景 | num_gpu 设置 | GPU Layers |
|------|-------------|-----------|
| Mac M 系列 | 0 | 0 (用户禁用) |
| Mac M 系列 | > 0 | 使用用户值 |
| Mac Intel | 任意 | 使用用户值 |
| Linux | 任意 | 使用用户值 |
| Windows | 任意 | 使用用户值 |

## 4. 实施计划

**Task 1**: 添加 `get_gpu_layers` 函数
**Task 2**: 替换硬编码的 `n_gpu_layers = 0`
**Task 3**: 构建测试验证

## 5. 风险评估

| 风险 | 影响 | 缓解 |
|------|------|------|
| Metal 精度问题仍存在 | 中 | M 系列用户可设置 num_gpu=0 回退 |
| 非 M Mac 启用 GPU | 低 | 用户主动设置 |

---

## 6. 实现代码

```rust
fn get_gpu_layers(num_gpu_setting: i32) -> u32 {
    if num_gpu_setting <= 0 {
        return 0;
    }

    #[cfg(target_os = "macos")]
    {
        if is_m_series_mac() {
            println!("[GPU] Mac M-series detected, enabling Metal acceleration");
            return num_gpu_setting as u32;
        }
    }

    num_gpu_setting as u32
}

#[cfg(target_os = "macos")]
fn is_m_series_mac() -> bool {
    std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .map(|output| {
            let brand = String::from_utf8_lossy(&output.stdout).to_lowercase();
            brand.contains("apple") || brand.contains("m1") || brand.contains("m2") || brand.contains("m3") || brand.contains("m4")
        })
        .unwrap_or(false)
}
```
