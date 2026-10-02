<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useUserProfileStore } from '@/stores/profile';
import {
  SelectContent,
  SelectIcon,
  SelectItem,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from 'reka-ui';

defineOptions({ name: 'Profile' });

const store = useUserProfileStore();

const newInterest = ref('');

const genderOptions = [
  { value: 'none', label: '未设置' },
  { value: 'male', label: '男' },
  { value: 'female', label: '女' },
  { value: 'other', label: '其他' },
];

const ageGroupOptions = [
  { value: 'none', label: '未设置' },
  { value: 'under18', label: '18岁以下' },
  { value: '18-24', label: '18-24岁' },
  { value: '25-34', label: '25-34岁' },
  { value: '35-44', label: '35-44岁' },
  { value: '45-54', label: '45-54岁' },
  { value: '55-64', label: '55-64岁' },
  { value: 'over65', label: '65岁以上' },
];

const languageOptions = [
  { value: 'auto', label: '自动检测' },
  { value: 'zh', label: '中文' },
  { value: 'en', label: 'English' },
];

onMounted(() => {
  store.fetchProfile();
});

async function handleFieldChange(field: string, value: string) {
  const val = value === 'none' ? '' : value;
  await store.updateProfile({ [field]: val });
}

async function handleAddInterest() {
  if (newInterest.value.trim()) {
    await store.addInterest(newInterest.value.trim());
    newInterest.value = '';
  }
}

async function handleRemoveInterest(interest: string) {
  await store.removeInterest(interest);
}
</script>

<template>
  <div class="max-w-[680px] mx-auto min-h-full px-1 pb-4 bg-app-bg/40 backdrop-blur-md select-text animate-fade-in">
    <div class="pb-4 border-b border-border-main/35">
      <h2 class="font-black text-xl bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary select-none tracking-tight">用户信息</h2>
      <p class="text-text-secondary mt-1.5 text-xs select-none">填写您的基本信息，帮助 AI 提供更个性化的回答</p>
    </div>

    <div v-if="store.isLoading" class="flex justify-center items-center py-8">
      <div class="animate-spin rounded-full h-10 w-10 border-t-2 border-b-2 border-brand"></div>
    </div>

    <div v-else class="mt-4 space-y-4">
      <section class="border border-border-main/35 rounded-2xl bg-panel-bg/45 backdrop-blur-md p-4 shadow-sm select-none">
        <h3 class="text-xs font-black text-text-primary mb-3 flex items-center gap-2 select-none">
          <span class="text-base">👤</span> 基本信息
        </h3>
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-2">
            <label class="text-[10px] font-bold text-text-muted select-none uppercase tracking-wider">姓名</label>
            <input
              v-model="store.profile.name"
              @blur="handleFieldChange('name', store.profile.name)"
              placeholder="输入您的姓名"
              class="w-full px-3 py-2 border border-border-main/40 bg-panel-bg/45 backdrop-blur-sm rounded-xl text-xs text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-2 focus:ring-brand/30 hover:border-border-main/60 transition-all duration-300 shadow-sm select-text"
            />
          </div>

          <div class="space-y-2">
            <label class="text-[10px] font-bold text-text-muted select-none uppercase tracking-wider">性别</label>
            <SelectRoot :model-value="store.profile.gender || 'none'" @update:model-value="(v: string) => handleFieldChange('gender', v)">
              <SelectTrigger class="inline-flex w-full items-center justify-between rounded-xl px-3 py-2 bg-panel-bg/45 backdrop-blur-sm border border-border-main/40 text-text-primary hover:bg-surface-bg/60 transition-all hover:border-border-main/60 shadow-sm text-xs select-none">
                <SelectValue placeholder="未设置" />
                <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
              </SelectTrigger>
              <SelectPortal>
                <SelectContent class="z-[100] bg-panel-bg/75 backdrop-blur-md border border-border-main/40 rounded-xl shadow-xl p-1 min-w-[120px] animate-fade-in select-none">
                  <SelectViewport>
                    <SelectItem v-for="opt in genderOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-1.5 text-xs rounded-lg text-text-primary cursor-pointer hover:bg-brand/15 hover:text-brand outline-none transition-all duration-200">
                      <SelectItemText class="truncate font-bold">{{ opt.label }}</SelectItemText>
                    </SelectItem>
                  </SelectViewport>
                </SelectContent>
              </SelectPortal>
            </SelectRoot>
          </div>

          <div class="space-y-2">
            <label class="text-[10px] font-bold text-text-muted select-none uppercase tracking-wider">年龄段</label>
            <SelectRoot :model-value="store.profile.age_group || 'none'" @update:model-value="(v: string) => handleFieldChange('age_group', v)">
              <SelectTrigger class="inline-flex w-full items-center justify-between rounded-xl px-3 py-2 bg-panel-bg/45 backdrop-blur-sm border border-border-main/40 text-text-primary hover:bg-surface-bg/60 transition-all hover:border-border-main/60 shadow-sm text-xs select-none">
                <SelectValue placeholder="未设置" />
                <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
              </SelectTrigger>
              <SelectPortal>
                <SelectContent class="z-[100] bg-panel-bg/75 backdrop-blur-md border border-border-main/40 rounded-xl shadow-xl p-1 min-w-[120px] animate-fade-in select-none">
                  <SelectViewport>
                    <SelectItem v-for="opt in ageGroupOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-1.5 text-xs rounded-lg text-text-primary cursor-pointer hover:bg-brand/15 hover:text-brand outline-none transition-all duration-200">
                      <SelectItemText class="truncate font-bold">{{ opt.label }}</SelectItemText>
                    </SelectItem>
                  </SelectViewport>
                </SelectContent>
              </SelectPortal>
            </SelectRoot>
          </div>

          <div class="space-y-2">
            <label class="text-[10px] font-bold text-text-muted select-none uppercase tracking-wider">回答语言</label>
            <SelectRoot :model-value="store.profile.language || 'auto'" @update:model-value="(v: string) => handleFieldChange('language', v)">
              <SelectTrigger class="inline-flex w-full items-center justify-between rounded-xl px-3 py-2 bg-panel-bg/45 backdrop-blur-sm border border-border-main/40 text-text-primary hover:bg-surface-bg/60 transition-all hover:border-border-main/60 shadow-sm text-xs select-none">
                <SelectValue />
                <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
              </SelectTrigger>
              <SelectPortal>
                <SelectContent class="z-[100] bg-panel-bg/75 backdrop-blur-md border border-border-main/40 rounded-xl shadow-xl p-1 min-w-[120px] animate-fade-in select-none">
                  <SelectViewport>
                    <SelectItem v-for="opt in languageOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-1.5 text-xs rounded-lg text-text-primary cursor-pointer hover:bg-brand/15 hover:text-brand outline-none transition-all duration-200">
                      <SelectItemText class="truncate font-bold">{{ opt.label }}</SelectItemText>
                    </SelectItem>
                  </SelectViewport>
                </SelectContent>
              </SelectPortal>
            </SelectRoot>
          </div>
        </div>
      </section>

      <section class="border border-border-main/35 rounded-2xl bg-panel-bg/45 backdrop-blur-md p-4 shadow-sm select-none">
        <h3 class="text-xs font-black text-text-primary mb-3 flex items-center gap-2 select-none">
          <span class="text-base">💼</span> 职业背景
        </h3>
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-2">
            <label class="text-[10px] font-bold text-text-muted select-none uppercase tracking-wider">职业</label>
            <input
              v-model="store.profile.occupation"
              @blur="handleFieldChange('occupation', store.profile.occupation)"
              placeholder="如：软件工程师、教师、医生"
              class="w-full px-3 py-2 border border-border-main/40 bg-panel-bg/45 backdrop-blur-sm rounded-xl text-xs text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-2 focus:ring-brand/30 hover:border-border-main/60 transition-all duration-300 shadow-sm select-text"
            />
          </div>

          <div class="space-y-2">
            <label class="text-[10px] font-bold text-text-muted select-none uppercase tracking-wider">行业</label>
            <input
              v-model="store.profile.industry"
              @blur="handleFieldChange('industry', store.profile.industry)"
              placeholder="如：互联网、教育、医疗"
              class="w-full px-3 py-2 border border-border-main/40 bg-panel-bg/45 backdrop-blur-sm rounded-xl text-xs text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-2 focus:ring-brand/30 hover:border-border-main/60 transition-all duration-300 shadow-sm select-text"
            />
          </div>
        </div>
      </section>

      <section class="border border-border-main/35 rounded-2xl bg-panel-bg/45 backdrop-blur-md p-4 shadow-sm select-none">
        <h3 class="text-xs font-black text-text-primary mb-3 flex items-center gap-2 select-none">
          <span class="text-base">🎯</span> 兴趣偏好
        </h3>
        <p class="text-xs text-text-secondary mb-2 select-none">添加您感兴趣的标签，AI 会根据这些信息提供更相关的内容推荐</p>
        <div class="flex flex-wrap gap-2 mb-3 select-none">
          <span
            v-for="interest in store.interests"
            :key="interest"
            class="inline-flex items-center gap-1.5 px-3 py-1 bg-brand/15 border border-brand/25 text-brand rounded-xl text-xs font-bold transition-all shadow-sm select-none"
          >
            {{ interest }}
            <button @click="handleRemoveInterest(interest)" class="ml-1 hover:text-danger-500 font-bold transition-colors">&times;</button>
          </span>
        </div>
        <div class="flex gap-2">
          <input
            v-model="newInterest"
            @keyup.enter="handleAddInterest"
            placeholder="输入兴趣标签后按回车添加"
            class="flex-1 px-3 py-2 border border-border-main/40 bg-panel-bg/45 backdrop-blur-sm rounded-xl text-xs text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-2 focus:ring-brand/30 hover:border-border-main/60 transition-all duration-300 shadow-sm select-text"
          />
          <button
            @click="handleAddInterest"
            class="px-4 py-2 bg-gradient-to-br from-brand to-brand-600 hover:shadow-lg hover:shadow-brand/20 text-white rounded-xl font-bold text-xs transition-all hover:scale-[1.02] active:scale-98 shadow-md"
          >
            添加
          </button>
        </div>
      </section>
    </div>
  </div>
</template>