# 用户基本信息模块实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现用户基本信息模块，支持用户录入背景信息并在 RAG 问答时生成个性化回答

**Architecture:** 复用现有 settings 表结构模式，新建 user_profile 和 user_interests 表，Rust 层提供 commands，前端通过 store 管理状态，RAG 问答时注入用户信息到 system prompt

**Tech Stack:** Tauri v2, Vue 3 + Pinia, reka-ui, SQLite

---

## 文件结构

```
src-tauri/src/
├── commands/
│   ├── mod.rs              # 注册 profile commands
│   └── profile.rs          # 新建：用户信息 CRUD commands
└── db/
    ├── mod.rs              # 注册 profile 模块
    └── profile.rs          # 新建：用户信息数据库操作

src/
├── api/
│   └── tauri.ts            # 添加 profile API 调用
├── stores/
│   └── profile.ts          # 新建：用户信息 Pinia store
├── components/
│   └── profile/
│       └── UserProfilePanel.vue  # 新建：用户信息编辑面板
└── views/
    └── Settings.vue        # 添加用户信息面板和快捷入口
```

---

## Task 1: 数据库层 - 创建 user_profile 表

**Files:**
- Modify: `src-tauri/src/db/mod.rs:1-20` - 注册 profile 模块
- Create: `src-tauri/src/db/profile.rs` - 用户信息数据库操作

- [ ] **Step 1: 在 db/mod.rs 添加 profile 模块注册**

```rust
mod profile;
```

- [ ] **Step 2: 创建 src-tauri/src/db/profile.rs**

```rust
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub gender: String,
    pub age_group: String,
    pub occupation: String,
    pub industry: String,
    pub language: String,
}

pub fn init_profile_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_profile (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_interests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT DEFAULT 'default' NOT NULL,
            interest TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, interest)
        )",
        [],
    )?;

    Ok(())
}

pub fn get_profile_value(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM user_profile WHERE key = ?")?;
    let mut rows = stmt.query([key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_profile_value(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO user_profile (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
        [key, value],
    )?;
    Ok(())
}

pub fn get_interests(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT interest FROM user_interests WHERE user_id = 'default'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut interests = Vec::new();
    for interest in rows {
        interests.push(interest?);
    }
    Ok(interests)
}

pub fn add_interest(conn: &Connection, interest: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO user_interests (user_id, interest) VALUES ('default', ?)",
        [interest],
    )?;
    Ok(())
}

pub fn remove_interest(conn: &Connection, interest: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM user_interests WHERE user_id = 'default' AND interest = ?",
        [interest],
    )?;
    Ok(())
}
```

- [ ] **Step 3: 在 lib.rs 或 main.rs 调用 init_profile_table**

查找现有初始化位置，添加：
```rust
use crate::db::profile::init_profile_table;

// 在数据库初始化处添加
init_profile_table(&conn)?;
```

---

## Task 2: Rust Commands - profile.rs

**Files:**
- Create: `src-tauri/src/commands/profile.rs` - 用户信息 commands
- Modify: `src-tauri/src/commands/mod.rs` - 注册 profile commands

- [ ] **Step 1: 创建 src-tauri/src/commands/profile.rs**

```rust
use crate::db::profile::{
    add_interest as db_add_interest,
    get_interests,
    get_profile_value,
    remove_interest as db_remove_interest,
    set_profile_value,
    init_profile_table,
    UserProfile,
};
use crate::errors::AppError;
use tauri::AppHandle;
use std::sync::Mutex;
use rusqlite::Connection;

fn get_db_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
    }

    Ok(app_data_dir.join("telepathy.db"))
}

fn get_db_connection(db_path: &std::path::Path) -> Result<Connection, AppError> {
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))?;

    let _ = init_profile_table(&conn);

    Ok(conn)
}

#[tauri::command]
pub async fn get_user_profile(app_handle: AppHandle) -> Result<UserProfile, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    let name = get_profile_value(&conn, "name")?.unwrap_or_default();
    let gender = get_profile_value(&conn, "gender")?.unwrap_or_default();
    let age_group = get_profile_value(&conn, "age_group")?.unwrap_or_default();
    let occupation = get_profile_value(&conn, "occupation")?.unwrap_or_default();
    let industry = get_profile_value(&conn, "industry")?.unwrap_or_default();
    let language = get_profile_value(&conn, "language")?.unwrap_or("auto".to_string());

    Ok(UserProfile {
        name,
        gender,
        age_group,
        occupation,
        industry,
        language,
    })
}

#[tauri::command]
pub async fn update_user_profile(app_handle: AppHandle, profile: UserProfile) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    set_profile_value(&conn, "name", &profile.name)?;
    set_profile_value(&conn, "gender", &profile.gender)?;
    set_profile_value(&conn, "age_group", &profile.age_group)?;
    set_profile_value(&conn, "occupation", &profile.occupation)?;
    set_profile_value(&conn, "industry", &profile.industry)?;
    set_profile_value(&conn, "language", &profile.language)?;

    Ok(())
}

#[tauri::command]
pub async fn get_user_interests(app_handle: AppHandle) -> Result<Vec<String>, AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    let interests = get_interests(&conn).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(interests)
}

#[tauri::command]
pub async fn add_user_interest(app_handle: AppHandle, interest: String) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    db_add_interest(&conn, &interest).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn remove_user_interest(app_handle: AppHandle, interest: String) -> Result<(), AppError> {
    let db_path = get_db_path(&app_handle)?;
    let conn = get_db_connection(&db_path)?;

    db_remove_interest(&conn, &interest).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}
```

- [ ] **Step 2: 在 commands/mod.rs 添加 profile 模块注册**

```rust
pub mod profile;
```

- [ ] **Step 3: 在 lib.rs 注册 profile commands**

```rust
profile::get_user_profile,
profile::update_user_profile,
profile::get_user_interests,
profile::add_user_interest,
profile::remove_user_interest,
```

---

## Task 3: 前端 API 封装

**Files:**
- Modify: `src/api/tauri.ts` - 添加 profile API

- [ ] **Step 1: 在 src/api/tauri.ts 末尾添加**

```typescript
export interface UserProfile {
  name: string;
  gender: string;
  age_group: string;
  occupation: string;
  industry: string;
  language: string;
}

export const getUserProfile = (): Promise<UserProfile> =>
  invoke('get_user_profile');

export const updateUserProfile = (profile: UserProfile): Promise<void> =>
  invoke('update_user_profile', { profile });

export const getUserInterests = (): Promise<string[]> =>
  invoke('get_user_interests');

export const addUserInterest = (interest: string): Promise<void> =>
  invoke('add_user_interest', { interest });

export const removeUserInterest = (interest: string): Promise<void> =>
  invoke('remove_user_interest', { interest });
```

---

## Task 4: 前端 Store - profile.ts

**Files:**
- Create: `src/stores/profile.ts` - 用户信息 Pinia store

- [ ] **Step 1: 创建 src/stores/profile.ts**

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  getUserProfile,
  updateUserProfile as updateUserProfileApi,
  getUserInterests,
  addUserInterest as addUserInterestApi,
  removeUserInterest as removeUserInterestApi,
  type UserProfile,
} from '@/api/tauri';

export const useUserProfileStore = defineStore('profile', () => {
  const profile = ref<UserProfile>({
    name: '',
    gender: '',
    age_group: '',
    occupation: '',
    industry: '',
    language: 'auto',
  });

  const interests = ref<string[]>([]);
  const isLoading = ref(false);
  const isSaving = ref(false);

  async function fetchProfile() {
    isLoading.value = true;
    try {
      const [profileData, interestsData] = await Promise.all([
        getUserProfile(),
        getUserInterests(),
      ]);
      profile.value = profileData;
      interests.value = interestsData;
    } catch (e) {
      console.error('Failed to fetch user profile:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function updateProfile(data: Partial<UserProfile>) {
    isSaving.value = true;
    try {
      const updated = { ...profile.value, ...data };
      await updateUserProfileApi(updated);
      profile.value = updated;
    } catch (e) {
      console.error('Failed to update user profile:', e);
      throw e;
    } finally {
      isSaving.value = false;
    }
  }

  async function addInterest(interest: string) {
    if (!interest.trim() || interests.value.includes(interest)) return;

    await addUserInterestApi(interest);
    interests.value.push(interest);
  }

  async function removeInterest(interest: string) {
    await removeUserInterestApi(interest);
    interests.value = interests.value.filter(i => i !== interest);
  }

  return {
    profile,
    interests,
    isLoading,
    isSaving,
    fetchProfile,
    updateProfile,
    addInterest,
    removeInterest,
  };
});
```

---

## Task 5: UI 组件 - UserProfilePanel.vue

**Files:**
- Create: `src/components/profile/UserProfilePanel.vue` - 用户信息编辑面板

- [ ] **Step 1: 创建 src/components/profile/UserProfilePanel.vue**

```vue
<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useUserProfileStore } from '@/stores/profile';
import {
  AccordionContent,
  AccordionHeader,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger,
  TextInput,
  TextInputRoot,
  TextInputInput,
  SelectContent,
  SelectIcon,
  SelectItem,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
  Badge,
  Button,
} from 'reka-ui';

const store = useUserProfileStore();

const newInterest = ref('');

const genderOptions = [
  { value: '', label: '未设置' },
  { value: 'male', label: '男' },
  { value: 'female', label: '女' },
  { value: 'other', label: '其他' },
];

const ageGroupOptions = [
  { value: '', label: '未设置' },
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
  await store.updateProfile({ [field]: value });
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
        <AccordionTrigger class="group flex w-full items-center justify-between py-4 text-sm font-medium transition-all text-left">
          <div class="flex items-center gap-2">
            <span class="text-lg">👤</span>
            <span>用户信息</span>
          </div>
          <span class="text-xs text-muted-foreground mr-2">选填，用于个性化回答</span>
        </AccordionTrigger>
      </AccordionHeader>
      <AccordionContent class="overflow-hidden data-[state=open]:animate-slide-down data-[state=closed]:animate-slide-up">
        <div class="space-y-4 py-4">
          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <label class="text-sm text-muted-foreground">姓名</label>
              <TextInput v-model="store.profile.name" @blur="handleFieldChange('name', store.profile.name)">
                <TextInputInput placeholder="输入姓名" />
              </TextInput>
            </div>

            <div class="space-y-2">
              <label class="text-sm text-muted-foreground">性别</label>
              <Select v-model="store.profile.gender" @update:modelValue="(v) => handleFieldChange('gender', v)">
                <SelectTrigger>
                  <SelectValue placeholder="选择性别" />
                </SelectTrigger>
                <SelectPortal>
                  <SelectContent>
                    <SelectViewport>
                      <SelectItem v-for="opt in genderOptions" :key="opt.value" :value="opt.value">
                        <SelectItemText>{{ opt.label }}</SelectItemText>
                      </SelectItem>
                    </SelectViewport>
                  </SelectContent>
                </SelectPortal>
              </Select>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <label class="text-sm text-muted-foreground">年龄段</label>
              <Select v-model="store.profile.age_group" @update:modelValue="(v) => handleFieldChange('age_group', v)">
                <SelectTrigger>
                  <SelectValue placeholder="选择年龄段" />
                </SelectTrigger>
                <SelectPortal>
                  <SelectContent>
                    <SelectViewport>
                      <SelectItem v-for="opt in ageGroupOptions" :key="opt.value" :value="opt.value">
                        <SelectItemText>{{ opt.label }}</SelectItemText>
                      </SelectItem>
                    </SelectViewport>
                  </SelectContent>
                </SelectPortal>
              </Select>
            </div>

            <div class="space-y-2">
              <label class="text-sm text-muted-foreground">回答语言</label>
              <Select v-model="store.profile.language" @update:modelValue="(v) => handleFieldChange('language', v)">
                <SelectTrigger>
                  <SelectValue placeholder="选择语言" />
                </SelectTrigger>
                <SelectPortal>
                  <SelectContent>
                    <SelectViewport>
                      <SelectItem v-for="opt in languageOptions" :key="opt.value" :value="opt.value">
                        <SelectItemText>{{ opt.label }}</SelectItemText>
                      </SelectItem>
                    </SelectViewport>
                  </SelectContent>
                </SelectPortal>
              </Select>
            </div>
          </div>

          <div class="space-y-2">
            <label class="text-sm text-muted-foreground">职业</label>
            <TextInput v-model="store.profile.occupation" @blur="handleFieldChange('occupation', store.profile.occupation)">
              <TextInputInput placeholder="输入职业，如：软件工程师、教师" />
            </TextInput>
          </div>

          <div class="space-y-2">
            <label class="text-sm text-muted-foreground">行业</label>
            <TextInput v-model="store.profile.industry" @blur="handleFieldChange('industry', store.profile.industry)">
              <TextInputInput placeholder="输入行业，如：互联网、教育、医疗" />
            </TextInput>
          </div>

          <div class="space-y-2">
            <label class="text-sm text-muted-foreground">兴趣标签</label>
            <div class="flex flex-wrap gap-2 mb-2">
              <Badge v-for="interest in store.interests" :key="interest" variant="secondary" class="pr-1">
                {{ interest }}
                <button @click="handleRemoveInterest(interest)" class="ml-1 hover:text-destructive">&times;</button>
              </Badge>
            </div>
            <div class="flex gap-2">
              <TextInput v-model="newInterest" @keyup.enter="handleAddInterest">
                <TextInputInput placeholder="添加兴趣标签" />
              </TextInput>
              <Button @click="handleAddInterest" size="sm">添加</Button>
            </div>
          </div>
        </div>
      </AccordionContent>
    </AccordionItem>
  </AccordionRoot>
</template>
```

---

## Task 6: Settings.vue 集成

**Files:**
- Modify: `src/views/Settings.vue` - 添加用户信息面板和快捷入口

- [ ] **Step 1: 在 Settings.vue 添加导入**

```typescript
import UserProfilePanel from '@/components/profile/UserProfilePanel.vue';
```

- [ ] **Step 2: 在 template 顶部添加用户信息面板（AccordionRoot 之前）**

```vue
<UserProfilePanel />

<AccordionRoot class="w-full" type="single" collapsible>
  <!-- 现有的其他面板... -->
</AccordionRoot>
```

- [ ] **Step 3: 右上角添加用户图标快捷入口**

在页面头部或右上角添加：
```vue
<button @click="scrollToProfile" class="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground">
  <span>👤</span>
  <span>用户信息</span>
</button>
```

添加方法：
```typescript
const scrollToProfile = () => {
  const panel = document.querySelector('[data-accordion-item="user-profile"]');
  panel?.scrollIntoView({ behavior: 'smooth' });
};
```

---

## Task 7: RAG 集成 - System Prompt 注入

**Files:**
- Modify: `src/services/rag.ts` 或相关 RAG 处理文件 - 注入用户信息到 prompt

- [ ] **Step 1: 创建 buildSystemPrompt 工具函数**

在 `src/utils/profile.ts` 创建：

```typescript
import type { UserProfile } from '@/api/tauri';

export function buildSystemPrompt(
  basePrompt: string,
  userProfile: UserProfile,
  interests: string[]
): string {
  const parts = [];

  if (userProfile.name || userProfile.occupation || userProfile.industry) {
    const info = [];
    if (userProfile.name) info.push(`姓名：${userProfile.name}`);
    if (userProfile.occupation) info.push(`职业：${userProfile.occupation}`);
    if (userProfile.industry) info.push(`行业：${userProfile.industry}`);
    parts.push(`【用户背景】${info.join('，')}`);
  }

  if (interests.length > 0) {
    parts.push(`【用户兴趣】${interests.join('、')}`);
  }

  let prompt = basePrompt;
  if (parts.length > 0) {
    prompt += '\n\n' + parts.join('\n');
  }

  if (userProfile.language !== 'auto') {
    const langInstruction = userProfile.language === 'zh'
      ? '\n\n【语言要求】请用中文回答所有问题。'
      : '\n\n【Language Requirement】Please answer all questions in English.';
    prompt += langInstruction;
  }

  return prompt;
}
```

- [ ] **Step 2: 在 RAG 问答时调用**

找到 RAG 问答的代码位置，注入用户信息：

```typescript
import { buildSystemPrompt } from '@/utils/profile';
import { useUserProfileStore } from '@/stores/profile';

const profileStore = useUserProfileStore();

// 在构建 system prompt 时
const systemPrompt = buildSystemPrompt(
  baseRagPrompt,
  profileStore.profile,
  profileStore.interests
);
```

---

## Task 8: 构建验证

- [ ] **Step 1: 运行 TypeScript 类型检查**

```bash
cd /Users/mac/project/telepathy
npm run typecheck
```

- [ ] **Step 2: 运行构建**

```bash
npm run build
```

- [ ] **Step 3: 测试 Rust 编译**

```bash
cd src-tauri
cargo build
```

---

## Task 9: Git 提交

- [ ] **提交代码**

```bash
git add -A
git commit -m "feat: 添加用户基本信息模块

- 新建 user_profile 和 user_interests 数据库表
- 添加 profile commands (get/update user profile, interests CRUD)
- 创建 profile store 和 UserProfilePanel 组件
- Settings 页面集成用户信息面板
- RAG 问答时注入用户背景到 system prompt
- 支持语言偏好设置 (auto/zh/en)"
```
