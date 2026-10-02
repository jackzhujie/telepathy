use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod geo;
pub mod manager;
pub mod recommender;
pub mod registry;

// 暂时禁用旧的爬虫部分，如果不再需要的话
pub mod downloader;
pub mod hf;
pub mod ms;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HubModel {
    pub name: String,
    pub description: String,
    pub pulls: String,
    pub updated: String,
    pub variants: Vec<ModelVariant>,
    pub category: ModelCategory,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelVariant {
    pub tag: String,
    pub size: u64, // bytes
    pub params: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelCategory {
    Chat,
    Embedding,
    Vision,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareProfile {
    pub vram_total: u64,
    pub vram_used: u64,
    pub ram_total: u64,
    pub ram_used: u64,
    pub disk_free: u64,
    pub is_apple_silicon: bool,
    pub os: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HubRecommendation {
    pub model: HubModel,
    pub variant: ModelVariant,
    pub score: f32,
    pub reason: String,
}

pub type HubCache = HashMap<String, HubModel>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrimaryRecommendations {
    pub chat: Option<HubRecommendation>,
    pub embedding: Option<HubRecommendation>,
    pub vision: Option<HubRecommendation>,
}

