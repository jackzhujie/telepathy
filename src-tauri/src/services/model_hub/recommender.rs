use crate::services::model_hub::{HardwareProfile, HubCache, HubRecommendation};
use sysinfo::{Disks, System};

pub struct Recommender;

impl Recommender {
    pub fn get_recommendations(
        profile: &HardwareProfile,
        hub: &HubCache,
    ) -> Vec<HubRecommendation> {
        let mut recommendations = Vec::new();

        for model in hub.values() {
            if let Some(best_variant) = Self::find_best_variant(profile, model) {
                let (score, reason) = Self::calculate_score(profile, model, &best_variant);

                recommendations.push(HubRecommendation {
                    model: model.clone(),
                    variant: best_variant.clone(),
                    score,
                    reason,
                });
            }
        }

        // Sort by score DESC, then by popularity DESC
        recommendations.sort_by(|a, b| {
            let score_cmp = b
                .score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal);
            if score_cmp == std::cmp::Ordering::Equal {
                let pulls_a = a.model.pulls.parse::<u64>().unwrap_or(0);
                let pulls_b = b.model.pulls.parse::<u64>().unwrap_or(0);
                pulls_b.cmp(&pulls_a)
            } else {
                score_cmp
            }
        });

        recommendations
    }

    pub fn compute_primary_recommendations(
        profile: &HardwareProfile,
        hub: &HubCache,
    ) -> crate::services::model_hub::PrimaryRecommendations {
        let ram_gb = profile.ram_total as f64 / 1_073_741_824.0;
        
        let chat_id = "qwen2.5";
        let chat_tag = if ram_gb <= 9.0 {
            "1.5b"
        } else if ram_gb <= 18.0 {
            "7b"
        } else {
            "14b"
        };
        let chat = Self::find_recommendation_by_id_tag(hub, profile, chat_id, chat_tag);

        let embedding = Self::find_recommendation_by_id_tag(hub, profile, "bge-large-zh", "latest")
            .or_else(|| Self::find_recommendation_by_id_tag(hub, profile, "bge-m3", "latest"));

        let vision = if ram_gb > 8.0 {
            Self::find_recommendation_by_id_tag(hub, profile, "qwen2.5-vl", "7b")
        } else {
            None
        };

        crate::services::model_hub::PrimaryRecommendations { chat, embedding, vision }
    }

    fn find_recommendation_by_id_tag(
        hub: &HubCache,
        profile: &HardwareProfile,
        model_id: &str,
        tag: &str,
    ) -> Option<crate::services::model_hub::HubRecommendation> {
        let model = hub.get(model_id).or_else(|| {
            let mapped_name = match model_id {
                "qwen2.5" => "Qwen 2.5",
                "bge-large-zh" => "BGE Large (中文)",
                "bge-m3" => "BGE-M3",
                "qwen2.5-vl" => "Qwen2.5-VL",
                "deepseek-r1" => "DeepSeek R1",
                _ => model_id,
            };
            hub.values().find(|m| {
                m.name.eq_ignore_ascii_case(model_id) || m.name.eq_ignore_ascii_case(mapped_name)
            })
        })?;

        let tag_lower = tag.to_lowercase();
        let suffix = format!(":{}", tag_lower);
        let variant = model.variants.iter().find(|v| {
            v.tag.eq_ignore_ascii_case(tag) || v.tag.to_lowercase().ends_with(&suffix)
        })?;

        let (score, reason) = Self::calculate_score(profile, model, variant);
        Some(crate::services::model_hub::HubRecommendation {
            model: model.clone(),
            variant: variant.clone(),
            score,
            reason,
        })
    }

    fn find_best_variant<'a>(
        profile: &HardwareProfile,
        model: &'a crate::services::model_hub::HubModel,
    ) -> Option<&'a crate::services::model_hub::ModelVariant> {
        use crate::services::model_hub::ModelCategory;

        let min_size = if model.category == ModelCategory::Chat {
            600 * 1024 * 1024
        } else {
            0
        };

        model
            .variants
            .iter()
            .filter(|v| v.size >= min_size && v.size <= profile.ram_total)
            .max_by(|a, b| {
                let a_score = Self::variant_score(profile, model, a);
                let b_score = Self::variant_score(profile, model, b);
                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn calculate_score(
        profile: &HardwareProfile,
        model: &crate::services::model_hub::HubModel,
        variant: &crate::services::model_hub::ModelVariant,
    ) -> (f32, String) {
        let mut score = Self::variant_score(profile, model, variant);
        let reason = Self::recommendation_reason(profile, model, variant);

        let pulls = model.pulls.parse::<f32>().unwrap_or(0.0);
        score += ((pulls + 1.0).log10() / 80.0).min(0.08);

        if profile.disk_free < variant.size * 2 {
            score -= 0.5;
            return (score.clamp(0.1, 1.0), "磁盘空间紧张".to_string());
        }

        (score.clamp(0.1, 1.0), reason)
    }

    fn variant_score(
        profile: &HardwareProfile,
        model: &crate::services::model_hub::HubModel,
        variant: &crate::services::model_hub::ModelVariant,
    ) -> f32 {
        use crate::services::model_hub::ModelCategory;

        let size = variant.size.max(1);
        let safe_ram = Self::safe_ram(profile);
        let safe_vram = Self::safe_vram(profile);

        let fit_score = if safe_vram > 0 && size <= safe_vram {
            if profile.is_apple_silicon {
                0.55
            } else {
                0.52
            }
        } else if size <= safe_ram {
            if profile.vram_total == 0 {
                0.38
            } else {
                0.30
            }
        } else if size <= profile.ram_total {
            0.15
        } else {
            -0.40
        };

        let params_b = Self::extract_params_billion(&variant.params)
            .or_else(|| Self::extract_params_billion(&variant.tag))
            .unwrap_or_else(|| (size as f32 / 1024.0 / 1024.0 / 1024.0 / 0.65).max(0.5));

        let target_params = match model.category {
            ModelCategory::Embedding => 0.6,
            ModelCategory::Vision => Self::target_chat_params(profile).min(8.0),
            ModelCategory::Chat | ModelCategory::Other => Self::target_chat_params(profile),
        };

        let param_distance = ((params_b - target_params).abs() / target_params.max(0.5)).min(1.0);
        let quality_score = match model.category {
            ModelCategory::Embedding => 0.20 * (1.0 - param_distance),
            ModelCategory::Vision => 0.28 * (1.0 - param_distance),
            _ => 0.30 * (1.0 - param_distance),
        };

        let quant_score = Self::quant_score(&variant.tag);
        let disk_penalty = if profile.disk_free < size * 15 / 10 {
            0.15
        } else if profile.disk_free < size * 3 {
            0.08
        } else {
            0.0
        };

        (0.28 + fit_score + quality_score + quant_score - disk_penalty)
            .clamp(0.0, 1.0)
    }

    fn recommendation_reason(
        profile: &HardwareProfile,
        _model: &crate::services::model_hub::HubModel,
        variant: &crate::services::model_hub::ModelVariant,
    ) -> String {
        let size = variant.size;

        if size <= Self::safe_vram(profile) && profile.vram_total > 0 {
            if profile.is_apple_silicon {
                "🚀 本机统一内存最佳适配".to_string()
            } else {
                "🚀 完全放入显存，极速运行".to_string()
            }
        } else if size <= Self::safe_ram(profile) {
            "💻 放入系统内存，稳定运行".to_string()
        } else if size <= profile.ram_total {
            "⚠️ 内存压力较高，可能卡顿".to_string()
        } else {
            "❌ 超出推荐内存，不建议运行".to_string()
        }
    }

    fn safe_ram(profile: &HardwareProfile) -> u64 {
        let reserved = (profile.ram_total as f64 * 0.25).max(3.0 * 1024.0 * 1024.0 * 1024.0) as u64;
        profile.ram_total.saturating_sub(reserved)
    }

    fn safe_vram(profile: &HardwareProfile) -> u64 {
        if profile.vram_total == 0 {
            return 0;
        }
        let reserved = if profile.is_apple_silicon {
            (profile.vram_total as f64 * 0.25) as u64
        } else {
            (512.0 * 1024.0 * 1024.0) as u64
        };
        profile.vram_total.saturating_sub(reserved)
    }

    fn target_chat_params(profile: &HardwareProfile) -> f32 {
        let ram_gb = profile.ram_total as f32 / 1_073_741_824.0;
        let vram_gb = profile.vram_total as f32 / 1_073_741_824.0;

        let budget = if profile.is_apple_silicon {
            ram_gb * 0.45
        } else if vram_gb > 1.0 {
            vram_gb * 0.85
        } else {
            ram_gb * 0.35
        };

        let quantized_target = budget / 0.6;

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

    fn extract_params_billion(text: &str) -> Option<f32> {
        let re = regex::Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*([bm])").ok()?;
        let cap = re.captures(text)?;
        let value = cap.get(1)?.as_str().parse::<f32>().ok()?;
        let unit = cap.get(2)?.as_str().to_ascii_lowercase();
        Some(if unit == "m" { value / 1000.0 } else { value })
    }

    fn quant_score(tag: &str) -> f32 {
        let t = tag.to_ascii_lowercase();
        if t.contains("q4_k_m") {
            0.12
        } else if t.contains("q5_k_m") || t.contains("q5_") {
            0.10
        } else if t.contains("q4_k_s") || t.contains("q4_0") {
            0.06
        } else if t.contains("q6_k") {
            0.04
        } else if t.contains("q8_0") {
            0.02
        } else if t.contains("iq4") || t.contains("iq3") {
            0.03
        } else if t.contains("f16") || t.contains("bf16") {
            -0.06
        } else if t.contains("q2") || t.contains("iq2") {
            -0.05
        } else {
            0.0
        }
    }

    pub fn get_hardware_profile() -> HardwareProfile {
        let mut sys = System::new_all();
        sys.refresh_all();

        let ram_total = sys.total_memory();
        let ram_used = sys.used_memory();

        let disks = Disks::new_with_refreshed_list();
        let disk_free = disks
            .iter()
            .map(|d| d.available_space())
            .max()
            .unwrap_or(50 * 1024 * 1024 * 1024);

        let is_apple_silicon = cfg!(target_os = "macos") && cfg!(target_arch = "aarch64");

        // 针对 Intel Mac 或其他集成显卡系统的保底逻辑
        let vram_total = if is_apple_silicon {
            ram_total / 2
        } else if cfg!(target_os = "macos") {
            // Intel Mac 共享显存通常为内存的 1/8 到 1/4
            ram_total / 8
        } else {
            0
        };

        HardwareProfile {
            vram_total,
            vram_used: 0,
            ram_total,
            ram_used,
            disk_free,
            is_apple_silicon,
            os: std::env::consts::OS.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::model_hub::HardwareProfile;
    use std::collections::HashMap;

    #[test]
    fn test_compute_primary_recommendations_low_end() {
        use crate::services::model_hub::{HubModel, ModelVariant, ModelCategory};
        let profile = HardwareProfile {
            vram_total: 1024 * 1024 * 1024,
            vram_used: 0,
            ram_total: 8 * 1024 * 1024 * 1024, // 8GB RAM
            ram_used: 0,
            disk_free: 50 * 1024 * 1024 * 1024,
            is_apple_silicon: true,
            os: "macos".to_string(),
        };
        let mut hub = HashMap::new();
        
        // Mock qwen2.5 with display name as map key or in name field
        let qwen = HubModel {
            name: "Qwen 2.5".to_string(),
            category: ModelCategory::Chat,
            description: "Qwen chat model".to_string(),
            pulls: "1000".to_string(),
            variants: vec![
                ModelVariant {
                    tag: "qwen2.5:1.5b".to_string(),
                    size: 1024 * 1024 * 1024, // 1GB
                    params: "1.5B".to_string(),
                },
                ModelVariant {
                    tag: "qwen2.5:7b".to_string(),
                    size: 4 * 1024 * 1024 * 1024, // 4GB
                    params: "7B".to_string(),
                }
            ],
            updated: "".to_string(),
            capabilities: vec![],
        };
        hub.insert("Qwen 2.5".to_string(), qwen);

        // Mock bge-large-zh
        let bge = HubModel {
            name: "BGE Large (中文)".to_string(),
            category: ModelCategory::Embedding,
            description: "BGE embedding model".to_string(),
            pulls: "500".to_string(),
            variants: vec![
                ModelVariant {
                    tag: "bge-large-zh:latest".to_string(),
                    size: 500 * 1024 * 1024,
                    params: "326M".to_string(),
                }
            ],
            updated: "".to_string(),
            capabilities: vec![],
        };
        hub.insert("BGE Large (中文)".to_string(), bge);

        let recs = Recommender::compute_primary_recommendations(&profile, &hub);
        
        // For 8GB RAM, we expect chat model to recommend "1.5b" because ram_gb <= 9.0
        assert!(recs.chat.is_some());
        assert_eq!(recs.chat.as_ref().unwrap().variant.tag, "qwen2.5:1.5b");
        
        // Embedding should be found
        assert!(recs.embedding.is_some());
        assert_eq!(recs.embedding.as_ref().unwrap().variant.tag, "bge-large-zh:latest");

        // Vision should be None for <= 8GB RAM
        assert!(recs.vision.is_none());
    }
}
