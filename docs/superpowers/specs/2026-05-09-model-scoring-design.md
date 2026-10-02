# 模型推荐评分权重优化设计

**日期**: 2026-05-09
**模块**: 模型推荐优化
**状态**: 设计中

---

## 1. 当前评分结构分析

### 1.1 当前权重分配

| 评分项 | 最大权重 | 说明 |
|--------|----------|------|
| 基础分 | 0.28 | 固定基础分 |
| fit_score | 0.12-0.5 | 内存适配程度 |
| quality_score | 0.09-0.26 | 参数规模匹配 |
| quant_score | -0.04 - 0.08 | 量化类型 |
| family_score | 0 - 0.08 | 模型家族偏好 |
| disk_penalty | -0.25 | 磁盘空间惩罚 |
| pulls_bonus | +0.08 | 流行度加分 |

**最大总分**: 约 1.0

### 1.2 发现的问题

1. **target_params 计算过于激进**: 32GB RAM → 17.6GB 模型目标
2. **磁盘惩罚过重**: -0.25 可能导致推荐分数过低
3. **量化分数差异小**: Q4_K_M (0.08) vs Q2 (0.035) 仅差 0.045
4. **缺少速度考量**: Q4 比 Q8 快约 30%，未考虑
5. **VRAM Apple Silicon 权重偏低**: 0.45 vs 0.5

---

## 2. 优化方案

### 2.1 调整 fit_score

```rust
fn variant_score(...) -> f32 {
    // 内存适配分数优化
    let fit_score = if safe_vram > 0 && size <= safe_vram {
        // VRAM 优先，给更高分数
        if profile.is_apple_silicon {
            0.55  // 之前 0.45
        } else {
            0.52  // 之前 0.50
        }
    } else if size <= safe_ram {
        if profile.vram_total == 0 {
            0.38  // 之前 0.34
        } else {
            0.28  // 之前 0.26
        }
    } else if size <= profile.ram_total {
        0.15  // 之前 0.12
    } else {
        -0.40  // 之前 -0.35
    };
}
```

### 2.2 优化 target_params 计算

```rust
fn target_chat_params(profile: &HardwareProfile) -> f32 {
    let ram_gb = profile.ram_total as f32 / 1_073_741_824.0;
    let vram_gb = profile.vram_total as f32 / 1_073_741_824.0;
    
    // 修正计算逻辑：更保守的估算
    let budget = if profile.is_apple_silicon {
        // Apple Silicon: 统一内存，给 45% 给模型
        ram_gb * 0.45
    } else if vram_gb > 1.0 {
        // 有独立显存: 给 85% 给模型
        vram_gb * 0.85
    } else {
        // 纯 RAM: 给 35% 给模型
        ram_gb * 0.35
    };

    // 量化后的目标参数（Q4 量化约 60% 大小）
    let quantized_target = budget / 0.6;
    
    // 映射到常见模型大小
    if quantized_target >= 30.0 {
        32.0
    } else if quantized_target >= 14.0 {
        14.0
    } else if quantized_target >= 7.0 {
        7.0
    } else if quantized_target >= 3.5 {
        3.0
    } else {
        1.5
    }
}
```

### 2.3 调整量化分数差异

```rust
fn quant_score(tag: &str) -> f32 {
    let t = tag.to_ascii_lowercase();
    
    // 扩大分数差异，更明确推荐 Q4_K_M
    if t.contains("q4_k_m") {
        0.12  // 之前 0.08 (+50%)
    } else if t.contains("q5_k_m") || t.contains("q5_") {
        0.10  // 之前 0.07
    } else if t.contains("q4_k_s") || t.contains("q4_0") {
        0.06  // 之前 0.045
    } else if t.contains("q6_k") {
        0.04  // 之前 0.035
    } else if t.contains("q8_0") {
        0.02  // 之前 0.035
    } else if t.contains("iq4") || t.contains("iq3") {
        0.03  // 之前 0.025
    } else if t.contains("f16") || t.contains("bf16") {
        -0.06  // 之前 -0.04
    } else if t.contains("q2") || t.contains("iq2") {
        -0.05  // 之前 -0.035
    } else {
        0.0
    }
}
```

### 2.4 降低磁盘惩罚

```rust
let disk_penalty = if profile.disk_free < size * 1.5 {
    0.15  // 之前 0.25，且从 2x 改为 1.5x
} else if profile.disk_free < size * 3 {
    0.08  // 新增：中等空间压力
} else {
    0.0
};
```

### 2.5 调整质量分数

```rust
let quality_score = match model.category {
    ModelCategory::Embedding => 0.20 * (1.0 - param_distance),  // 之前 0.18
    ModelCategory::Vision => 0.28 * (1.0 - param_distance),      // 之前 0.26
    _ => 0.30 * (1.0 - param_distance),                         // 之前 0.26
};
```

---

## 3. 优化后权重分布

| 评分项 | 之前 | 之后 | 变化 |
|--------|------|------|------|
| 基础分 | 0.28 | 0.25 | -0.03 |
| fit_score (VRAM) | 0.45-0.50 | 0.52-0.55 | +0.07 |
| fit_score (RAM) | 0.26-0.34 | 0.28-0.38 | +0.04 |
| quality_score | 0.09-0.26 | 0.10-0.30 | +0.04 |
| quant_score | -0.04-0.08 | -0.06-0.12 | 扩大差异 |
| family_score | 0-0.08 | 0-0.08 | 不变 |
| disk_penalty | -0.25 | -0.15 | +0.10 |

---

## 4. 预期效果

- **更准确的模型大小推荐**: 基于更保守的内存估算
- **更明确的 Q4_K_M 偏好**: 质量/速度最佳平衡
- **减少误推荐**: 降低磁盘空间紧张导致的低分
- **VRAM 优先**: Apple Silicon 更好地利用统一内存

---

## 5. 实施计划

**Task 1**: 应用权重调整
**Task 2**: 测试验证
