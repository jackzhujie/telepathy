# Model Manager UI Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Optimize the "For You Recommended" section in the model manager by packing chat, vector, and vision models side-by-side in a 3-column row, and making their individual size variants collapsible.

**Architecture:** Use Vue 3.5 Composition API in `src/components/settings/ModelManager.vue`. Maintain a reactive dictionary mapping expanded models to booleans. Align the recommendation categories into a responsive single-row grid (`grid-cols-1 lg:grid-cols-3`), and enclose variant tables in an animated collapsible element.

**Tech Stack:** Vue 3.5, TypeScript, Tailwind CSS, reka-ui, Vite 6.

---

### Task 1: Initialize Expand/Collapse State and Toggle Logic in ModelManager.vue

**Files:**
- Modify: `src/components/settings/ModelManager.vue:30-41` (In script setup)

- [ ] **Step 1: Write expansion state and handler logic**
  Add the `expandedRecommendModels` reactive state and `toggleRecommendExpand` helper function.
  
  ```typescript
  // 新增：记录推荐模型的展开折叠状态
  const expandedRecommendModels = ref<Record<string, boolean>>({});

  const toggleRecommendExpand = (modelName: string) => {
    expandedRecommendModels.value[modelName] = !expandedRecommendModels.value[modelName];
  };
  ```

- [ ] **Step 2: Verify type-safety with TypeScript compilation**
  Run: `pnpm exec vue-tsc --noEmit`
  Expected: Command succeeds with no compiler errors.

- [ ] **Step 3: Commit**
  ```bash
  git add src/components/settings/ModelManager.vue
  git commit -m "feat(models): add react state and toggle handler for recommended models expansion"
  ```

---

### Task 2: Rewrite Recommendation Layout to a 3-Column Grid and Make Cards Collapsible

**Files:**
- Modify: `src/components/settings/ModelManager.vue:615-800` (In template)

- [ ] **Step 1: Refactor Template grid layout**
  Rewrite the entire recommend tab (`activeTab === 'recommend'`) template block. Replace the three full-width rows with a single responsive 3-column layout grid. Move the variant tables under each card into a collapsible container with the transition classes, and place an expansion trigger button below the card description.

  Ensure that all cards use `sortVariantsByEfficiency(rec.model.variants)` for listing variants, and compute the card's size range dynamically using `getModelSizeRange(rec.model.variants)`.

  Here is the target HTML structure for the grid:
  ```html
  <!-- Tab 内容 1：为您推荐 -->
  <div v-if="activeTab === 'recommend'" class="grid grid-cols-1 lg:grid-cols-3 gap-6 animate-fadeIn">
    <!-- ① 对话模型推荐 -->
    <div class="space-y-4">
      <div class="flex items-center gap-2 pb-2 border-b border-border-main/20">
        <div class="w-1.5 h-4 bg-brand rounded-full"></div>
        <h5 class="text-xs font-bold text-text-primary uppercase tracking-wider">对话推荐 (Chat Models)</h5>
      </div>
      <div class="flex flex-col gap-4">
        <div 
          v-for="rec in chatRecommendations" 
          :key="rec.model.name"
          class="bg-surface-bg border-2 border-border-main-light rounded-xl p-4 flex flex-col gap-3 relative group hover:border-brand/50 transition-all duration-300 shadow-md"
        >
          <div class="flex justify-between items-start gap-2">
            <div class="flex flex-col gap-1 overflow-hidden">
              <h6 class="text-sm font-bold text-text-primary truncate" :title="rec.model.name">{{ rec.model.name }}</h6>
              <div class="flex gap-1">
                <span :class="getModelTypeColor(rec.model.category)" class="text-[9px] px-1.5 py-0.5 rounded uppercase font-bold">{{ getModelTypeName(rec.model.category) }}</span>
              </div>
            </div>
            <span class="flex-shrink-0 text-[10px] bg-brand/10 text-brand px-1.5 py-0.5 rounded border border-brand/20 font-mono">{{ getModelSizeRange(rec.model.variants) }}</span>
          </div>
          <p class="text-xs text-text-secondary line-clamp-2 min-h-[2.5rem]">{{ rec.model.description }}</p>
          
          <!-- 折叠内容容器 -->
          <div 
            class="overflow-hidden transition-all duration-300 ease-in-out"
            :class="expandedRecommendModels[rec.model.name] ? 'max-h-[500px] opacity-100 mt-2' : 'max-h-0 opacity-0'"
          >
            <div class="flex flex-col gap-1.5 pt-2 border-t border-border-main/30">
              <label class="text-[9px] font-bold text-text-muted uppercase tracking-wider mb-1 block">可选规格 · 运行效率</label>
              <div class="flex flex-col gap-1">
                <div
                  v-for="v in sortVariantsByEfficiency(rec.model.variants)"
                  :key="v.tag"
                  class="flex items-center justify-between px-2 py-1.5 rounded-lg border transition-colors"
                  :class="getVariantEfficiency(v.size).color + ' border-opacity-40'"
                >
                  <div class="flex items-center gap-1.5 min-w-0 flex-1 overflow-hidden">
                    <span class="text-[9px] flex-shrink-0" :title="getVariantEfficiency(v.size).description">{{ getVariantEfficiency(v.size).icon }}</span>
                    <span class="text-[10px] font-bold font-mono truncate flex-1" :title="v.params">{{ v.params }}</span>
                    <span class="text-[9px] opacity-70 flex-shrink-0">{{ v.size > 0 ? formatSize(v.size) : '' }}</span>
                  </div>
                  <div class="flex items-center gap-1">
                    <span
                      class="text-[8px] font-bold px-1 py-0.5 rounded"
                      :class="getVariantEfficiency(v.size).color"
                      :title="getVariantEfficiency(v.size).description"
                    >{{ getVariantEfficiency(v.size).label }}</span>
                    <button
                      v-if="!isInstalled(rec.model.name)"
                      @click.stop="handleInstall(`${rec.model.name}:${v.tag}`, v.size)"
                      class="p-1 rounded hover:bg-text-primary/10 transition-colors"
                      :title="`下载 ${rec.model.name}:${v.tag}`"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <div class="mt-auto pt-2 border-t border-border-main/30 flex justify-between items-center gap-2 overflow-hidden">
            <span class="text-[10px] text-brand font-bold truncate flex-1 min-w-0" :title="rec.reason">{{ rec.reason }}</span>
            <div v-if="isInstalled(rec.model.name)" class="text-success-500 font-bold text-xs uppercase tracking-tighter shrink-0">已就绪</div>
          </div>

          <!-- 折叠/展开控制按钮 -->
          <button 
            @click.stop="toggleRecommendExpand(rec.model.name)"
            class="w-full flex items-center justify-center gap-1.5 py-1 px-3 bg-surface-bg border border-border-main/50 hover:bg-border-main/20 text-[10px] font-bold rounded-lg text-text-secondary hover:text-text-primary transition-all mt-2 cursor-pointer group"
          >
            <span>{{ expandedRecommendModels[rec.model.name] ? '收起规格' : '展开规格' }}</span>
            <svg 
              xmlns="http://www.w3.org/2000/svg" 
              class="w-3 h-3 text-text-muted group-hover:text-text-primary transition-transform duration-300"
              :class="{ 'rotate-180': expandedRecommendModels[rec.model.name] }"
              viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
            >
              <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- ② 向量检索推荐 -->
    <div class="space-y-4">
      <div class="flex items-center gap-2 pb-2 border-b border-border-main/20">
        <div class="w-1.5 h-4 bg-success-500 rounded-full"></div>
        <h5 class="text-xs font-bold text-text-primary uppercase tracking-wider">向量推荐 (RAG Embedding)</h5>
      </div>
      <div class="flex flex-col gap-4">
        <div 
          v-for="rec in embeddingRecommendations" 
          :key="rec.model.name"
          class="bg-surface-bg border-2 border-border-main-light rounded-xl p-4 flex flex-col gap-3 hover:border-success-500/50 transition-all shadow-md"
        >
          <div class="flex justify-between items-start gap-2">
            <div class="flex flex-col gap-1 overflow-hidden">
              <h6 class="text-sm font-bold text-text-primary truncate" :title="rec.model.name">{{ rec.model.name }}</h6>
              <div class="flex gap-1">
                <span :class="getModelTypeColor(rec.model.category)" class="text-[9px] px-1.5 py-0.5 rounded uppercase font-bold">{{ getModelTypeName(rec.model.category) }}</span>
              </div>
            </div>
            <span class="flex-shrink-0 text-[10px] bg-success-500/10 text-success-500 px-1.5 py-0.5 rounded border border-success-500/20 font-mono">{{ getModelSizeRange(rec.model.variants) }}</span>
          </div>
          <p class="text-xs text-text-secondary line-clamp-2 min-h-[2.5rem]">{{ rec.model.description }}</p>

          <!-- 折叠内容容器 -->
          <div 
            class="overflow-hidden transition-all duration-300 ease-in-out"
            :class="expandedRecommendModels[rec.model.name] ? 'max-h-[500px] opacity-100 mt-2' : 'max-h-0 opacity-0'"
          >
            <div class="flex flex-col gap-1.5 pt-2 border-t border-border-main/30">
              <label class="text-[9px] font-bold text-text-muted uppercase tracking-wider mb-1 block">可选规格 · 运行效率</label>
              <div class="flex flex-col gap-1">
                <div
                  v-for="v in sortVariantsByEfficiency(rec.model.variants)"
                  :key="v.tag"
                  class="flex items-center justify-between px-2 py-1.5 rounded-lg border transition-colors"
                  :class="getVariantEfficiency(v.size).color + ' border-opacity-40'"
                >
                  <div class="flex items-center gap-1.5 min-w-0 flex-1 overflow-hidden">
                    <span class="text-[9px] flex-shrink-0" :title="getVariantEfficiency(v.size).description">{{ getVariantEfficiency(v.size).icon }}</span>
                    <span class="text-[10px] font-bold font-mono truncate flex-1" :title="v.params">{{ v.params }}</span>
                    <span class="text-[9px] opacity-70 flex-shrink-0">{{ v.size > 0 ? formatSize(v.size) : '' }}</span>
                  </div>
                  <div class="flex items-center gap-1">
                    <span
                      class="text-[8px] font-bold px-1 py-0.5 rounded"
                      :class="getVariantEfficiency(v.size).color"
                      :title="getVariantEfficiency(v.size).description"
                    >{{ getVariantEfficiency(v.size).label }}</span>
                    <button
                      v-if="!isInstalled(rec.model.name)"
                      @click.stop="handleInstall(`${rec.model.name}:${v.tag}`, v.size)"
                      class="p-1 rounded hover:bg-text-primary/10 transition-colors"
                      :title="`下载 ${rec.model.name}:${v.tag}`"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="mt-auto pt-2 border-t border-border-main/30 flex justify-between items-center gap-2 overflow-hidden">
            <span class="text-[10px] text-success-500 font-bold truncate flex-1 min-w-0" :title="rec.reason">{{ rec.reason }}</span>
            <div v-if="isInstalled(rec.model.name)" class="text-success-500 font-bold text-xs uppercase tracking-tighter shrink-0">已就绪</div>
          </div>

          <!-- 折叠/展开控制按钮 -->
          <button 
            @click.stop="toggleRecommendExpand(rec.model.name)"
            class="w-full flex items-center justify-center gap-1.5 py-1 px-3 bg-surface-bg border border-border-main/50 hover:bg-border-main/20 text-[10px] font-bold rounded-lg text-text-secondary hover:text-text-primary transition-all mt-2 cursor-pointer group"
          >
            <span>{{ expandedRecommendModels[rec.model.name] ? '收起规格' : '展开规格' }}</span>
            <svg 
              xmlns="http://www.w3.org/2000/svg" 
              class="w-3 h-3 text-text-muted group-hover:text-text-primary transition-transform duration-300"
              :class="{ 'rotate-180': expandedRecommendModels[rec.model.name] }"
              viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
            >
              <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- ③ 视觉理解推荐 -->
    <div class="space-y-4">
      <div class="flex items-center gap-2 pb-2 border-b border-border-main/20">
        <div class="w-1.5 h-4 bg-purple-500 rounded-full"></div>
        <h5 class="text-xs font-bold text-text-primary uppercase tracking-wider">视觉推荐 (Vision Models)</h5>
      </div>
      <div class="flex flex-col gap-4">
        <div 
          v-for="rec in visionRecommendations" 
          :key="rec.model.name"
          class="bg-surface-bg border-2 border-border-main-light rounded-xl p-4 flex flex-col gap-3 hover:border-purple-500/50 transition-all shadow-md"
        >
          <div class="flex justify-between items-start gap-2">
            <div class="flex flex-col gap-1 overflow-hidden">
              <h6 class="text-sm font-bold text-text-primary truncate" :title="rec.model.name">{{ rec.model.name }}</h6>
              <div class="flex gap-1">
                <span :class="getModelTypeColor(rec.model.category)" class="text-[9px] px-1.5 py-0.5 rounded uppercase font-bold">{{ getModelTypeName(rec.model.category) }}</span>
              </div>
            </div>
            <span class="flex-shrink-0 text-[10px] bg-purple-500/10 text-purple-500 px-1.5 py-0.5 rounded border border-purple-500/20 font-mono">{{ getModelSizeRange(rec.model.variants) }}</span>
          </div>
          <p class="text-xs text-text-secondary line-clamp-2 min-h-[2.5rem]">{{ rec.model.description }}</p>

          <!-- 折叠内容容器 -->
          <div 
            class="overflow-hidden transition-all duration-300 ease-in-out"
            :class="expandedRecommendModels[rec.model.name] ? 'max-h-[500px] opacity-100 mt-2' : 'max-h-0 opacity-0'"
          >
            <div class="flex flex-col gap-1.5 pt-2 border-t border-border-main/30">
              <label class="text-[9px] font-bold text-text-muted uppercase tracking-wider mb-1 block">可选规格 · 运行效率</label>
              <div class="flex flex-col gap-1">
                <div
                  v-for="v in sortVariantsByEfficiency(rec.model.variants)"
                  :key="v.tag"
                  class="flex items-center justify-between px-2 py-1.5 rounded-lg border transition-colors"
                  :class="getVariantEfficiency(v.size).color + ' border-opacity-40'"
                >
                  <div class="flex items-center gap-1.5 min-w-0 flex-1 overflow-hidden">
                    <span class="text-[9px] flex-shrink-0" :title="getVariantEfficiency(v.size).description">{{ getVariantEfficiency(v.size).icon }}</span>
                    <span class="text-[10px] font-bold font-mono truncate flex-1" :title="v.params">{{ v.params }}</span>
                    <span class="text-[9px] opacity-70 flex-shrink-0">{{ v.size > 0 ? formatSize(v.size) : '' }}</span>
                  </div>
                  <div class="flex items-center gap-1">
                    <span
                      class="text-[8px] font-bold px-1 py-0.5 rounded"
                      :class="getVariantEfficiency(v.size).color"
                      :title="getVariantEfficiency(v.size).description"
                    >{{ getVariantEfficiency(v.size).label }}</span>
                    <button
                      v-if="!isInstalled(rec.model.name)"
                      @click.stop="handleInstall(`${rec.model.name}:${v.tag}`, v.size)"
                      class="p-1 rounded hover:bg-text-primary/10 transition-colors"
                      :title="`下载 ${rec.model.name}:${v.tag}`"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="mt-auto pt-2 border-t border-border-main/30 flex justify-between items-center gap-2 overflow-hidden">
            <span class="text-[10px] text-purple-500 font-bold truncate flex-1 min-w-0" :title="rec.reason">{{ rec.reason }}</span>
            <div v-if="isInstalled(rec.model.name)" class="text-success-500 font-bold text-xs uppercase tracking-tighter shrink-0">已就绪</div>
          </div>

          <!-- 折叠/展开控制按钮 -->
          <button 
            @click.stop="toggleRecommendExpand(rec.model.name)"
            class="w-full flex items-center justify-center gap-1.5 py-1 px-3 bg-surface-bg border border-border-main/50 hover:bg-border-main/20 text-[10px] font-bold rounded-lg text-text-secondary hover:text-text-primary transition-all mt-2 cursor-pointer group"
          >
            <span>{{ expandedRecommendModels[rec.model.name] ? '收起规格' : '展开规格' }}</span>
            <svg 
              xmlns="http://www.w3.org/2000/svg" 
              class="w-3 h-3 text-text-muted group-hover:text-text-primary transition-transform duration-300"
              :class="{ 'rotate-180': expandedRecommendModels[rec.model.name] }"
              viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
            >
              <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
  ```

- [ ] **Step 2: Verify type-safety and syntax in template**
  Run: `pnpm exec vue-tsc --noEmit`
  Expected: Command succeeds with no compilation errors.

- [ ] **Step 3: Commit**
  ```bash
  git add src/components/settings/ModelManager.vue
  git commit -m "feat(models): implement 3-column collapsible recommendation grid layout"
  ```
