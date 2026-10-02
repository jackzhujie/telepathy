<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useUserProfileStore } from '@/stores/profile';
import {
  AccordionContent,
  AccordionHeader,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger,
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

defineOptions({ name: 'UserProfilePanel' });

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
  // 转换 'none' 回空字符串
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
  <AccordionRoot class="w-full" type="single" collapsible defaultValue="user-profile">
    <AccordionItem value="user-profile">
      <AccordionHeader>
        <AccordionTrigger class="group flex w-full items-center justify-between py-3 text-xs font-medium transition-all text-left">
          <div class="flex items-center gap-2">
            <span class="text-sm">👤</span>
            <span>用户信息</span>
          </div>
          <span class="text-xs text-text-secondary mr-2">选填，用于个性化回答</span>
        </AccordionTrigger>
      </AccordionHeader>
      <AccordionContent class="overflow-hidden data-[state=open]:animate-slide-down data-[state=closed]:animate-slide-up">
        <div class="space-y-3 py-3">
          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-2">
              <label class="text-xs text-text-secondary">姓名</label>
              <input
                v-model="store.profile.name"
                @blur="handleFieldChange('name', store.profile.name)"
                placeholder="输入姓名"
                class="w-full px-2.5 py-2 bg-surface-bg border border-border-main-light rounded-md text-xs text-text-primary placeholder-text-muted"
              />
            </div>

            <div class="space-y-2">
              <label class="text-xs text-text-secondary">性别</label>
              <SelectRoot :model-value="store.profile.gender || 'none'" @update:model-value="(v) => handleFieldChange('gender', v)">
                <SelectTrigger class="inline-flex w-full items-center justify-between rounded-lg px-2.5 py-2 text-xs bg-surface-bg border border-border-main-light text-text-primary hover:bg-surface-bg/80 transition-colors overflow-hidden">
                  <SelectValue placeholder="未设置" />
                  <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
                </SelectTrigger>
                <SelectPortal>
                  <SelectContent class="z-[100] bg-panel-bg border border-border-main/50 rounded-lg shadow-2xl p-1 min-w-[120px]">
                    <SelectViewport>
                      <SelectItem v-for="opt in genderOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-1.5 text-xs rounded-md text-text-secondary cursor-pointer hover:bg-brand hover:text-white outline-none overflow-hidden">
                        <SelectItemText class="truncate">{{ opt.label }}</SelectItemText>
                      </SelectItem>
                    </SelectViewport>
                  </SelectContent>
                </SelectPortal>
              </SelectRoot>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-2">
              <label class="text-xs text-text-secondary">年龄段</label>
              <SelectRoot :model-value="store.profile.age_group || 'none'" @update:model-value="(v) => handleFieldChange('age_group', v)">
                <SelectTrigger class="inline-flex w-full items-center justify-between rounded-lg px-2.5 py-2 text-xs bg-surface-bg border border-border-main-light text-text-primary hover:bg-surface-bg/80 transition-colors overflow-hidden">
                  <SelectValue placeholder="未设置" />
                  <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
                </SelectTrigger>
                <SelectPortal>
                  <SelectContent class="z-[100] bg-panel-bg border border-border-main/50 rounded-lg shadow-2xl p-1 min-w-[120px]">
                    <SelectViewport>
                      <SelectItem v-for="opt in ageGroupOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-1.5 text-xs rounded-md text-text-secondary cursor-pointer hover:bg-brand hover:text-white outline-none overflow-hidden">
                        <SelectItemText class="truncate">{{ opt.label }}</SelectItemText>
                      </SelectItem>
                    </SelectViewport>
                  </SelectContent>
                </SelectPortal>
              </SelectRoot>
            </div>

            <div class="space-y-2">
              <label class="text-xs text-text-secondary">回答语言</label>
              <SelectRoot :model-value="store.profile.language || 'auto'" @update:model-value="(v) => handleFieldChange('language', v)">
                <SelectTrigger class="inline-flex w-full items-center justify-between rounded-lg px-2.5 py-2 text-xs bg-surface-bg border border-border-main-light text-text-primary hover:bg-surface-bg/80 transition-colors overflow-hidden">
                  <SelectValue />
                  <SelectIcon class="ml-2 flex-shrink-0"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg></SelectIcon>
                </SelectTrigger>
                <SelectPortal>
                  <SelectContent class="z-[100] bg-panel-bg border border-border-main/50 rounded-lg shadow-2xl p-1 min-w-[120px]">
                    <SelectViewport>
                      <SelectItem v-for="opt in languageOptions" :key="opt.value" :value="opt.value" class="flex items-center px-3 py-1.5 text-xs rounded-md text-text-secondary cursor-pointer hover:bg-brand hover:text-white outline-none overflow-hidden">
                        <SelectItemText class="truncate">{{ opt.label }}</SelectItemText>
                      </SelectItem>
                    </SelectViewport>
                  </SelectContent>
                </SelectPortal>
              </SelectRoot>
            </div>
          </div>

          <div class="space-y-2">
            <label class="text-xs text-text-secondary">职业</label>
            <input
              v-model="store.profile.occupation"
              @blur="handleFieldChange('occupation', store.profile.occupation)"
              placeholder="输入职业，如：软件工程师、教师"
              class="w-full px-2.5 py-2 bg-surface-bg border border-border-main-light rounded-md text-xs text-text-primary placeholder-text-muted"
            />
          </div>

          <div class="space-y-2">
            <label class="text-xs text-text-secondary">行业</label>
            <input
              v-model="store.profile.industry"
              @blur="handleFieldChange('industry', store.profile.industry)"
              placeholder="输入行业，如：互联网、教育、医疗"
              class="w-full px-2.5 py-2 bg-surface-bg border border-border-main-light rounded-md text-xs text-text-primary placeholder-text-muted"
            />
          </div>

          <div class="space-y-2">
            <label class="text-xs text-text-secondary">兴趣标签</label>
            <div class="flex flex-wrap gap-2 mb-2">
              <span
                v-for="interest in store.interests"
                :key="interest"
                class="inline-flex items-center gap-1 px-2 py-1 bg-brand/15 text-brand rounded-md text-xs"
              >
                {{ interest }}
                <button @click="handleRemoveInterest(interest)" class="ml-1 hover:text-destructive">&times;</button>
              </span>
            </div>
            <div class="flex gap-2">
              <input
                v-model="newInterest"
                @keyup.enter="handleAddInterest"
                placeholder="添加兴趣标签"
                class="flex-1 px-2.5 py-2 bg-surface-bg border border-border-main-light rounded-md text-xs text-text-primary placeholder-text-muted"
              />
              <button
                @click="handleAddInterest"
                class="px-4 py-2 bg-brand hover:bg-brand/90 text-white rounded-md text-xs"
              >
                添加
              </button>
            </div>
          </div>
        </div>
      </AccordionContent>
    </AccordionItem>
  </AccordionRoot>
</template>