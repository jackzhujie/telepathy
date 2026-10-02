# Telepathy 官方网站全面重设计规格文档

**日期：** 2026-05-21  
**状态：** 已批准，待实施  
**分支：** feature/telepathy-website  
**工作目录：** `.worktrees/website/website`

---

## 1. 背景与目标

### 背景

当前官网仅有一个基础 Hero 区域（单一组件，65 行代码），不足以承担宣传推广的职能。本次重设计目标是打造一个对标 Linear、Vercel 等顶级极客产品官网的完整宣传页面。

### 目标

- 完整呈现 Telepathy 的核心价值主张：本地优先、隐私安全、RAG 智能问答
- 引导访客完成"了解产品 → 信任产品 → 下载产品"的转化路径
- 视觉质量达到顶级独立开发者产品官网水准

---

## 2. 整体视觉设计语言

### 配色
- 背景色：`#070a13`（深空暗蓝）
- 前景文字：`#f8fafc`（近白色）
- 主强调色：Rose-500 `#f43f5e` → Violet-600 `#7c3aed` 渐变
- 次级文字：`#94a3b8`（蓝灰色）
- 卡片背景：`rgba(15, 23, 42, 0.45)` + `backdrop-blur-md`

### 动效原则
- 玻璃拟态（Glassmorphism）：所有卡片使用毛玻璃背景 + 微边框
- 微交互：卡片悬停时边框发光 + 位移 `-2px`
- 滚动驱动（Scroll-driven Animation）：工作原理版块使用 CSS `animation-timeline: scroll()` 或 IntersectionObserver 触发
- 全局浮动动画：`.animate-float` 应用于 Hero 截图

### 字体
- 字体族：Inter（从 Google Fonts 引入）
- 标题：`font-bold`，Hero 主标题 `5xl~6xl`
- 正文：`text-xl`，说明文字 `text-sm~text-base`

---

## 3. 页面版块规格

### 3.1 导航栏（Navbar）

**组件文件：** `src/components/Navbar.tsx`

**行为：**
- 固定在页面顶部（`position: fixed`，`z-50`）
- 初始状态：透明背景
- 滚动超过 50px：激活毛玻璃背景（`backdrop-blur-md` + `bg-[#070a13]/70` + 底部边框线）
- 移动端：隐藏中间导航链接，保留 Logo 和下载按钮

**布局（从左到右）：**
1. Telepathy Logo（图标 SVG + "Telepathy" 文字，带渐变色）
2. 中部锚点导航：`功能特性 / 工作原理 / 立即下载`（点击平滑滚动至对应 section）
3. 右侧高亮下载按钮：渐变背景，`立即下载`

---

### 3.2 Hero 首屏（升级版）

**组件文件：** `src/components/Hero.tsx`（在现有基础上大幅升级）

**左侧内容（从上到下）：**
1. **徽章标签**：`🔒 100% 本地运行 · 完全开源` — 小胶囊形状，带微边框
2. **主标题**（2行）：
   ```
   您的专属本地 AI
   智能知识库
   ```
   第二行应用 `text-gradient`（Rose → Violet）
3. **副标题**：一句话描述产品核心价值
4. **双按钮行**：
   - 主按钮：渐变色填充「立即下载 [OS]版」，OS 自动检测
   - 次按钮：透明边框「了解功能 →」，点击滚动到 Features
5. **版本信息行**：`v{version}` + `100% 离线隐私安全`
6. **平台支持徽标**：macOS / Windows / Linux 系统小图标 + 文字

**右侧内容：**
- App 截图（`/public/app-screenshot.png`）
- 外圈：`bg-gradient-to-r from-rose-500 to-violet-600` 模糊光晕（`blur + opacity-30`）
- 整体应用 `animate-float` 浮动动画

---

### 3.3 功能特性（Features）

**组件文件：** `src/components/Features.tsx`

**布局：**
- Section 标题居中：「为什么选择 Telepathy」
- 副标题：一句话描述
- **3列 × 2行 = 6张**玻璃拟态卡片（`grid-cols-3`，移动端 `grid-cols-1`）

**6张特性卡片内容：**

| # | 图标 | 标题 | 描述 |
|---|------|------|------|
| 1 | 🔒 | 100% 本地隐私 | 所有数据留在您的设备，从不上传任何云端服务 |
| 2 | 🧠 | RAG 智能检索 | 向量语义搜索，精准理解您的问题意图 |
| 3 | ⚡ | 极速本地响应 | 基于 llama.cpp 优化，3秒内给出回答 |
| 4 | 📄 | 多格式文档解析 | 支持 PDF、Word、Markdown、网页等主流格式 |
| 5 | 🎯 | 来源精准引用 | 每条回答标注来源文件与段落，可验证 |
| 6 | 🔌 | 完全离线可用 | 无需联网，飞行模式下依然正常工作 |

**卡片交互：**
- 默认：`border border-white/5`
- Hover：`border-rose-500/30` + `shadow-lg shadow-rose-500/10` + `translateY(-2px)`

---

### 3.4 工作原理（HowItWorks）

**组件文件：** `src/components/HowItWorks.tsx`

**布局：**
- Section 标题居中：「三步开启您的专属知识库」
- **垂直步骤列表**（3个步骤），每步触发一次滚动动画（使用 IntersectionObserver）
- 每步布局：左侧大号步骤数字（`01 / 02 / 03`，渐变色） + 右侧内容块

**三步内容：**

| 步骤 | 标题 | 描述 | 动效元素 |
|------|------|------|----------|
| 01 | 导入您的文档 | 拖放或选择文件，支持多种格式批量导入 | 文件图标 + 模拟上传进度条动效 |
| 02 | 自动智能向量化 | Telepathy 自动将内容转化为语义向量，构建私人知识图谱 | 粒子流或神经网络抽象动效 |
| 03 | 自然语言问答 | 用日常语言提问，获得基于您文档的精准回答 | 模拟聊天气泡逐字打字动效 |

**动效实现：**
- 使用 `IntersectionObserver` 监听每个步骤 div 进入视口
- 进入时触发 CSS class 变换：`opacity-0 translate-y-8` → `opacity-100 translate-y-0`
- 每步过渡时间 `duration-700`，左侧数字和右侧内容略有时差（`delay-150`）

---

### 3.5 下载区（Download）

**组件文件：** `src/components/Download.tsx`

**背景：** 全宽渐变 `from-[#0b0f1e] via-[#0f0a1e] to-[#070a13]`

**内容（从上到下）：**
1. **Section 标题**：「选择您的平台，即刻开始」
2. **当前版本信息**：`最新版本 v{version}，发布于 {pub_date}`
3. **三列下载卡片**（桌面端并列，移动端堆叠）：

   - **macOS 卡片**：
     - Apple Silicon（darwin-aarch64）下载按钮
     - Intel（darwin-x86_64）下载按钮
   - **Windows 卡片**：
     - x86_64 `.msi` 安装包下载按钮
   - **Linux 卡片**：
     - x86_64 `.AppImage` 下载按钮

4. **底部辅助链接**：GitHub Releases 页面链接（查看所有历史版本）

**数据来源：** `updater: UpdaterInfo` 从父组件 `page.tsx` 服务端 SSR 获取并传入

---

### 3.6 页脚（Footer）

**组件文件：** `src/components/Footer.tsx`

**布局：** 单行水平居中
- 左：`© 2026 Telepathy. All rights reserved.`
- 右：GitHub 图标链接

---

## 4. 组件架构与数据流

```
page.tsx (Server Component)
└── 服务端 fetch updater.json
    └── <Navbar />                    (纯展示，无数据依赖)
    └── <Hero updater={updater} />    (需要 updater.version 和 download URLs)
    └── <Features />                  (纯静态内容)
    └── <HowItWorks />               (纯静态内容 + 动效)
    └── <Download updater={updater} /> (需要 updater 的 platforms 和 version)
    └── <Footer />                    (纯静态)
```

---

## 5. 文件结构变更

```
website/src/
├── components/
│   ├── Hero.tsx          (大幅修改)
│   ├── Navbar.tsx        (新增)
│   ├── Features.tsx      (新增)
│   ├── HowItWorks.tsx    (新增)
│   ├── Download.tsx      (新增)
│   └── Footer.tsx        (新增)
├── app/
│   ├── layout.tsx        (修改：引入 Inter 字体、添加 Navbar)
│   └── page.tsx          (修改：组装所有新组件)
├── hooks/
│   └── useOSDetection.ts (保持不变)
└── types/
    └── updater.ts        (保持不变)
```

---

## 6. 验证标准

- [ ] 本地 `npm run dev` 启动无报错
- [ ] 所有现有 Vitest 测试继续通过
- [ ] 页面在 1440px 宽屏下各版块布局正确
- [ ] 页面在 375px 移动端下响应式正常
- [ ] Navbar 滚动时毛玻璃效果正确触发
- [ ] HowItWorks 滚动动效在各步骤正确触发
- [ ] 下载按钮 OS 自动检测正确
