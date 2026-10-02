export type ModelType = 'chat' | 'embedding' | 'vision' | 'unknown';

export interface ModelSize {
  tag: string;
  params: string;
  min_memory_gb: number;
  file_size_gb: number;
}

export interface RegistryModel {
  id: string;
  name: string;
  provider: string;
  type: ModelType;
  description: string;
  sizes: ModelSize[];
}

export interface InstalledModel {
  full_name: string;
  base_name: string;
  tag: string;
  model_type: ModelType;
  size_bytes: number;
  display_name: string;
  provider: string;
  description: string;
}

export type RecommendationLevel = 'recommended' | 'marginal' | 'notrecommended';

export interface ModelRecommendation {
  model: RegistryModel;
  size: ModelSize;
  recommendation: RecommendationLevel;
}

export interface ModelPullProgress {
  model: string;
  status: string;
  percentage: number;
  completed: number;
  total: number;
  stage?: string;
}

export interface ModelVariant {
  tag: string;
  size: number;
  params: string;
}

export interface HubModel {
  name: string;
  description: string;
  pulls: string;
  updated: string;
  variants: ModelVariant[];
  category: 'chat' | 'embedding' | 'vision' | 'other';
}

export interface HardwareProfile {
  vram_total: number;
  vram_used: number;
  ram_total: number;
  ram_used: number;
  disk_free: number;
  is_apple_silicon: boolean;
  os: string;
}

export interface HubRecommendation {
  model: HubModel;
  variant: ModelVariant;
  score: number;
  reason: string;
}

export interface PrimaryRecommendations {
  chat: HubRecommendation | null;
  embedding: HubRecommendation | null;
  vision: HubRecommendation | null;
}

export interface ModelHubResponse {
  hardware: HardwareProfile;
  recommendations: HubRecommendation[];
  primary_recommendations: PrimaryRecommendations;
  all_models: HubModel[];
  leaderboard: HubModel[];
  trending: HubModel[];
  is_china: boolean;
  cache_timestamp: number;
}
