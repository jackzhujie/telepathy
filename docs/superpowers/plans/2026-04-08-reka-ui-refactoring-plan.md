# Telepathy 项目 Reka-UI 组件整合重构规划

## 一、项目概述

本规划旨在对 Telepathy 前端项目进行系统性组件重构，全面整合 reka-ui 组件库，以提升代码质量、可维护性和用户体验。

### 1.1 项目技术栈
- **前端框架**: Vue 3 + TypeScript + Vite
- **UI 组件库**: reka-ui (无样式、符合 WAI-ARIA 规范)
- **动画库**: @vueuse/motion
- **样式框架**: Tailwind CSS
- **桌面框架**: Tauri v2

### 1.2 现有组件清单

| 文件路径 | 组件类型 | 优先级 |
|---------|---------|--------|
| `src/views/Chat.vue` | 页面 | P0 |
| `src/views/Documents.vue` | 页面 | P0 |
| `src/views/KnowledgeBase.vue` | 页面 | P0 |
| `src/views/Settings.vue` | 页面 | P0 |
| `src/components/layout/AppLayout.vue` | 布局 | P0 |
| `src/components/layout/Sidebar.vue` | 导航 | P1 |
| `src/components/layout/NotificationPanel.vue` | 通知 | P1 |

---

## 二、reka-ui 组件库能力分析

### 2.1 Form 表单组件

| 组件 | 功能 | 当前项目中对应元素 | 替换优先级 |
|-----|------|-------------------|-----------|
| `Select` | 下拉选择 | `<select>` | ✅ 已替换 |
| `Slider` | 范围滑动 | `<input type="range">` | ✅ 已替换 |
| `Switch` | 开关切换 | 自定义样式 div | P1 |
| `Checkbox` | 复选框 | `<input type="checkbox">` | P1 |
| `RadioGroup` | 单选组 | 原生单选 | P2 |
| `Toggle` | 状态切换 | 自定义按钮 | P2 |
| `NumberField` | 数字输入 | `<input type="number">` | P2 |
| `Label` | 标签 | `<label>` | P1 |

### 2.2 General 通用组件

| 组件 | 功能 | 当前项目中对应元素 | 替换优先级 |
|-----|------|-------------------|-----------|
| `Dialog` | 模态框 | 自定义模态 div | ✅ 已替换 |
| `Toast` | 通知提示 | 自定义 Toast div | ✅ 已替换 |
| `Tooltip` | 工具提示 | 自定义 Tooltip | ✅ 已替换 |
| `Progress` | 进度条 | 自定义进度条 | ✅ 已替换 |
| `Tabs` | 标签页 | 自定义 Tab 组件 | P1 |
| `Accordion` | 手风琴 | 自定义折叠面板 | P2 |
| `Separator` | 分隔线 | `<div class="border">` | P1 |
| `HoverCard` | 悬浮卡片 | 自定义卡片 | P2 |
| `Popover` | 弹出框 | 自定义 Popover | P2 |

### 2.3 Navigation 导航组件

| 组件 | 功能 | 当前项目中对应元素 | 替换优先级 |
|-----|------|-------------------|-----------|
| `NavigationMenu` | 导航菜单 | Sidebar 侧边栏 | P1 |
| `Menubar` | 菜单栏 | 顶部导航 | P2 |
| `DropdownMenu` | 下拉菜单 | 右键菜单 | P2 |

### 2.4 其他高级组件

| 组件 | 功能 | 适用场景 | 替换优先级 |
|-----|------|---------|-----------|
| `Collapsible` | 可折叠区域 | 设置页面分组 | P2 |
| `ContextMenu` | 右键菜单 | 文档/知识库 | P2 |
| `Avatar` | 用户头像 | 通知面板 | P2 |
| `ScrollArea` | 滚动区域 | 长列表 | P2 |

---

## 三、重构优先级与详细规划

### 3.1 P0 - 核心组件（必须替换）

#### 3.1.1 Select 组件 ✅ 已完成
- **文件**: `Chat.vue`, `Documents.vue`, `KnowledgeBase.vue`, `Settings.vue`
- **替换内容**: 所有 `<select>` 元素
- **状态**: 已全面替换为 reka-ui Select 组件

#### 3.1.2 Dialog 组件 ✅ 已完成
- **文件**: `Documents.vue`
- **替换内容**: 项目创建/编辑模态框、删除确认模态框
- **状态**: 已替换为 reka-ui Dialog 组件

#### 3.1.3 Toast 组件 ✅ 已完成
- **文件**: `Documents.vue`
- **替换内容**: 自定义 Toast 通知
- **状态**: 已替换为 reka-ui Toast 组件

#### 3.1.4 Progress 组件 ✅ 已完成
- **文件**: `Settings.vue`
- **替换内容**: 模型下载进度条
- **状态**: 已替换为 reka-ui Progress 组件

#### 3.1.5 Slider 组件 ✅ 已完成
- **文件**: `Settings.vue`
- **替换内容**: Top-K 和相似度阈值的 range input
- **状态**: 已替换为 reka-ui Slider 组件

### 3.2 P1 - 重要组件（建议替换）

#### 3.2.1 Switch 组件
- **文件**: `Settings.vue`
- **当前实现**: 自定义样式的开关 div
- **替换方案**: 使用 `SwitchRoot`, `SwitchThumb`
- **影响范围**: RAG 设置中的开关选项
- **预估工作量**: 1-2 小时

```vue
<!-- 当前实现 -->
<div class="w-12 h-6 rounded-full bg-gray-200 relative cursor-pointer">
  <div class="w-5 h-5 rounded-full bg-white shadow absolute top-0.5 transition-all"></div>
</div>

<!-- 替换后 -->
<SwitchRoot class="w-12 h-6 rounded-full bg-gray-200 relative cursor-pointer data-[state=checked]:bg-blue-500">
  <SwitchThumb class="w-5 h-5 rounded-full bg-white shadow absolute top-0.5 transition-all data-[state=checked]:translate-x-6" />
</SwitchRoot>
```

#### 3.2.2 Tabs 组件
- **文件**: `Settings.vue`
- **当前实现**: 自定义 Tab 切换逻辑
- **替换方案**: 使用 `TabsRoot`, `TabsList`, `TabsTrigger`, `TabsContent`
- **影响范围**: 设置页面的分段展示
- **预估工作量**: 2-3 小时

```vue
<TabsRoot v-model="activeTab">
  <TabsList class="flex border-b">
    <TabsTrigger value="system">系统设置</TabsTrigger>
    <TabsTrigger value="model">模型设置</TabsTrigger>
    <TabsTrigger value="rag">RAG 设置</TabsTrigger>
  </TabsList>
  <TabsContent value="system">系统设置内容</TabsContent>
  <TabsContent value="model">模型设置内容</TabsContent>
  <TabsContent value="rag">RAG 设置内容</TabsContent>
</TabsRoot>
```

#### 3.2.3 Separator 组件
- **文件**: `Settings.vue`, `Documents.vue`, `KnowledgeBase.vue`
- **当前实现**: `<div class="border-t border-gray-200">` 或 `<hr>`
- **替换方案**: 使用 `Separator` 组件
- **预估工作量**: 1 小时

```vue
<!-- 当前实现 -->
<div class="border-t border-gray-200 my-4"></div>

<!-- 替换后 -->
<Separator class="my-4 bg-gray-200" />
```

#### 3.2.4 Label 组件
- **文件**: `Settings.vue`, `Documents.vue`
- **当前实现**: `<label class="block text-sm font-bold">`
- **替换方案**: 使用 `Label` 组件配合 `for` 属性
- **预估工作量**: 1-2 小时

```vue
<!-- 当前实现 -->
<label class="block text-sm font-bold mb-2">聊天模型</label>
<input class="w-full..." />

<!-- 替换后 -->
<Label for="chat-model" class="block text-sm font-bold mb-2">聊天模型</Label>
<SelectRoot id="chat-model">...</SelectRoot>
```

### 3.3 P2 - 增强组件（可选替换）

#### 3.3.1 Tooltip 增强
- **文件**: 全局
- **当前实现**: 已添加 TooltipProvider
- **增强内容**: 为更多操作按钮添加工具提示
- **预估工作量**: 2 小时

#### 3.3.2 Accordion 组件
- **文件**: `Settings.vue`
- **当前实现**: 平面列表展示
- **替换方案**: 使用 `AccordionRoot`, `AccordionItem`, `AccordionTrigger`, `AccordionContent`
- **预估工作量**: 3-4 小时

#### 3.3.3 Checkbox 组件
- **文件**: `Documents.vue`, `Settings.vue`
- **当前实现**: 原生 `<input type="checkbox">`
- **替换方案**: 使用 `CheckboxRoot`, `CheckboxIndicator`
- **预估工作量**: 2 小时

#### 3.3.4 NavigationMenu 组件
- **文件**: `Sidebar.vue`
- **当前实现**: 自定义导航菜单
- **替换方案**: 使用 `NavigationMenuRoot`, `NavigationMenuList`, `NavigationMenuTrigger`, `NavigationMenuContent`
- **预估工作量**: 4-5 小时

---

## 四、重构实施流程

### 4.1 第一阶段：核心组件替换 (P0) ✅ 已完成
1. Select 组件全面替换
2. Dialog 模态框替换
3. Toast 通知替换
4. Progress 进度条替换
5. Slider 滑块替换

### 4.2 第二阶段：重要组件替换 (P1)
1. Switch 开关组件替换
2. Tabs 标签页组件替换
3. Separator 分隔线替换
4. Label 标签组件替换

### 4.3 第三阶段：增强组件替换 (P2)
1. Tooltip 工具提示增强
2. Accordion 手风琴组件
3. Checkbox 复选框替换
4. NavigationMenu 导航菜单

---

## 五、已替换组件清单

| 组件类型 | 文件 | 状态 | 验证结果 |
|---------|------|------|---------|
| Select | Chat.vue | ✅ 已完成 | tsc ✅ build ✅ |
| Select | Documents.vue | ✅ 已完成 | tsc ✅ build ✅ |
| Select | KnowledgeBase.vue | ✅ 已完成 | tsc ✅ build ✅ |
| Select | Settings.vue | ✅ 已完成 | tsc ✅ build ✅ |
| Dialog | Documents.vue | ✅ 已完成 | tsc ✅ build ✅ |
| Toast | Documents.vue | ✅ 已完成 | tsc ✅ build ✅ |
| Tooltip | App.vue (全局) | ✅ 已完成 | tsc ✅ build ✅ |
| Tooltip | Documents.vue | ✅ 已完成 | tsc ✅ build ✅ |
| Progress | Settings.vue | ✅ 已完成 | tsc ✅ build ✅ |
| Slider | Settings.vue | ✅ 已完成 | tsc ✅ build ✅ |

---

## 六、重构验证标准

### 6.1 TypeScript 类型检查
```bash
npx vue-tsc --noEmit
```
**预期结果**: 无错误，无警告

### 6.2 构建验证
```bash
npm run build
```
**预期结果**: 构建成功，生成 dist 目录

### 6.3 运行时验证
- [ ] 组件渲染正常
- [ ] 交互功能完整
- [ ] 响应式布局正常
- [ ] 无控制台错误

---

## 七、注意事项

### 7.1 Provider 层级
- `TooltipProvider` 已放置在 `App.vue` 根部，全局可用
- `ToastProvider` 保留在各页面，按需使用
- 新增 Provider 时需考虑层级关系

### 7.2 样式隔离
- reka-ui 为无样式组件，所有样式通过 Tailwind CSS 类名实现
- 保持样式一致性，避免内联样式
- 使用 data-[state=xxx] 选择器处理组件状态样式

### 7.3 可访问性
- 所有 reka-ui 组件符合 WAI-ARIA 规范
- 确保键盘导航正常
- 确保屏幕阅读器支持

---

## 八、后续优化建议

1. **组件抽象**: 将常用组件模式提取为可复用组件
2. **主题系统**: 基于 CSS 变量实现主题切换
3. **动画整合**: 统一 @vueuse/motion 动画风格
4. **单元测试**: 为核心组件添加测试用例

---

*文档创建时间: 2026-04-08*
*最后更新时间: 2026-04-08*
