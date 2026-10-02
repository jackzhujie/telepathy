# 用户基本信息模块设计

## 1. 概述

为 Telepathy 知识库应用添加用户基本信息模块，支持用户录入个人背景信息，并在 RAG 问答时基于用户信息生成更个性化、更准确的回答。

## 2. 用户信息字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 姓名（选填） |
| `gender` | enum | 性别：male/female/other（选填） |
| `age_group` | enum | 年龄段：under18/18-24/25-34/35-44/45-54/55-64/over65（选填） |
| `occupation` | string | 职业（选填） |
| `industry` | string | 所属行业（选填） |
| `interests` | string[] | 兴趣标签列表（选填） |
| `language` | enum | 回答语言偏好：auto/zh/en（默认 auto） |

## 3. 数据存储

### 3.1 数据库设计

**新建 `user_profile` 表：**

```sql
CREATE TABLE user_profile (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_interests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT DEFAULT 'default',
    interest TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, interest)
);
```

### 3.2 Rust 端

**新增 `commands/profile.rs`：**

- `get_user_profile() -> Result<UserProfile, AppError>`
- `update_user_profile(profile: UserProfile) -> Result<(), AppError>`
- `get_user_interests() -> Result<Vec<String>, AppError>`
- `add_user_interest(interest: String) -> Result<(), AppError>`
- `remove_user_interest(interest: String) -> Result<(), AppError>`

**新增 `db/profile.rs`：**

- `UserProfile` 结构体
- 数据库读写函数

## 4. 前端模块

### 4.1 Store - `stores/profile.ts`

```typescript
interface UserProfile {
  name: string;
  gender: 'male' | 'female' | 'other' | '';
  age_group: string;
  occupation: string;
  industry: string;
  interests: string[];
  language: 'auto' | 'zh' | 'en';
}

export const useUserProfileStore = defineStore('profile', () => {
  const profile = ref<UserProfile>({
    name: '',
    gender: '',
    age_group: '',
    occupation: '',
    industry: '',
    interests: [],
    language: 'auto',
  });

  const isLoading = ref(false);

  async function fetchProfile() { /* ... */ }
  async function updateProfile(data: Partial<UserProfile>) { /* ... */ }
  async function addInterest(interest: string) { /* ... */ }
  async function removeInterest(interest: string) { /* ... */ }

  return { profile, isLoading, fetchProfile, updateProfile, addInterest, removeInterest };
});
```

### 4.2 组件 - `components/profile/UserProfilePanel.vue`

可折叠的用户信息编辑面板，使用 reka-ui Accordion 组件。

**位置：**
- Settings.vue 页面顶部，新增"用户信息"折叠面板
- 右上角添加用户图标快捷入口（触发滚动到面板或展开）

**UI 字段：**
- 姓名：TextInput
- 性别：Select（male/female/other）
- 年龄段：Select
- 职业：TextInput
- 行业：TextInput
- 兴趣：TagInput（可添加/删除标签）
- 回答语言：Select（auto/zh/en）

## 5. RAG 集成

### 5.1 System Prompt 注入

在 RAG 问答时，从 `useUserProfileStore` 获取用户信息，拼接到 system prompt：

```typescript
function buildSystemPrompt(userProfile: UserProfile, context: string): string {
  const parts = [];

  if (userProfile.name || userProfile.occupation) {
    const info = [];
    if (userProfile.name) info.push(`用户名：${userProfile.name}`);
    if (userProfile.occupation) info.push(`职业：${userProfile.occupation}`);
    if (userProfile.industry) info.push(`行业：${userProfile.industry}`);
    parts.push(`用户背景：${info.join('，')}`);
  }

  if (userProfile.interests.length > 0) {
    parts.push(`用户兴趣：${userProfile.interests.join('、')}`);
  }

  let basePrompt = context || '你是一个有用的 AI 助手。';
  if (parts.length > 0) {
    basePrompt += '\n\n' + parts.join('\n');
  }

  return basePrompt;
}
```

### 5.2 回答语言调整

根据 `userProfile.language` 调整回答：

- `auto`：使用查询语言
- `zh`：强制中文回答
- `en`：强制英文回答

在构建 prompt 时添加语言指令。

## 6. API 接口

### 6.1 Rust Commands

```rust
#[tauri::command]
pub async fn get_user_profile(app_handle: AppHandle) -> Result<UserProfile, AppError>;

#[tauri::command]
pub async fn update_user_profile(
    app_handle: AppHandle,
    profile: UserProfile,
) -> Result<(), AppError>;

#[tauri::command]
pub async fn get_user_interests(app_handle: AppHandle) -> Result<Vec<String>, AppError>;

#[tauri::command]
pub async fn add_user_interest(
    app_handle: AppHandle,
    interest: String,
) -> Result<(), AppError>;

#[tauri::command]
pub async fn remove_user_interest(
    app_handle: AppHandle,
    interest: String,
) -> Result<(), AppError>;
```

### 6.2 前端 API

在 `src/api/tauri.ts` 添加：

```typescript
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

## 7. 实施计划

### Phase 1: 数据层
1. 创建 `user_profile` 数据库表和迁移
2. 实现 Rust `profile` commands
3. 前端 API 封装

### Phase 2: 前端 UI
1. 创建 `stores/profile.ts`
2. 创建 `UserProfilePanel.vue` 组件
3. 在 Settings.vue 添加用户信息面板
4. 右上角添加快捷入口图标

### Phase 3: RAG 集成
1. 创建 `buildSystemPrompt` 工具函数
2. 修改 RAG 问答逻辑，注入用户信息
3. 添加语言偏好处理

## 8. 风险与注意事项

- 用户信息为选填，不影响核心功能
- 兴趣标签需要防重复添加
- 语言偏好默认 auto 保持向后兼容
- 存储使用单表 + 关系表，便于后续扩展