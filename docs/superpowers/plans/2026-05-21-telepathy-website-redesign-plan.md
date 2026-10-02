# Telepathy 官网全面重设计 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将当前仅有 Hero 区域的单页官网升级为包含导航、功能特性、工作原理演示和下载区的完整宣传官网。

**Architecture:** Next.js 15 App Router，`page.tsx` 作为服务端组件负责 SSR 拉取 `updater.json`，将数据通过 props 传入 `Hero` 和 `Download` 客户端组件；`Navbar`、`Features`、`HowItWorks`、`Footer` 均为纯展示组件。所有新组件在 `src/components/` 下独立文件，`layout.tsx` 引入 Inter 字体并挂载 `Navbar`。

**Tech Stack:** Next.js 15 (App Router)、React 19、TypeScript、Tailwind CSS v4、Vitest + @testing-library/react

---

## 文件变更总览

```
website/src/
├── app/
│   ├── layout.tsx        [MODIFY] 引入 Inter 字体，添加 Navbar
│   └── page.tsx          [MODIFY] 组装所有新组件
├── components/
│   ├── Hero.tsx          [MODIFY] 大幅升级：徽章、双按钮、平台徽标
│   ├── Navbar.tsx        [NEW] 毛玻璃固定导航栏
│   ├── Features.tsx      [NEW] 3×2 玻璃拟态特性卡片
│   ├── HowItWorks.tsx    [NEW] 三步滚动动效
│   ├── Download.tsx      [NEW] 三平台下载卡片
│   └── Footer.tsx        [NEW] 简洁页脚
```

---

## Task 1: Navbar 组件

**Files:**
- Create: `website/src/components/Navbar.tsx`
- Create: `website/src/components/Navbar.test.tsx`

- [ ] **Step 1: 写失败测试**

```tsx
// website/src/components/Navbar.test.tsx
import { render, screen } from '@testing-library/react';
import { Navbar } from './Navbar';

describe('Navbar', () => {
  it('renders logo text', () => {
    render(<Navbar />);
    expect(screen.getByText('Telepathy')).toBeInTheDocument();
  });

  it('renders download link', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /立即下载/i })).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: 运行测试（应失败）**

```bash
cd website && npx vitest run src/components/Navbar.test.tsx
```
期望：FAIL — "Cannot find module './Navbar'"

- [ ] **Step 3: 实现 Navbar**

```tsx
// website/src/components/Navbar.tsx
"use client";
import { useEffect, useState } from 'react';

export function Navbar() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener('scroll', handleScroll, { passive: true });
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? 'backdrop-blur-md bg-[#070a13]/80 border-b border-white/5 shadow-lg'
          : 'bg-transparent'
      }`}
    >
      <div className="max-w-7xl mx-auto px-4 h-16 flex items-center justify-between">
        {/* Logo */}
        <a href="#" className="flex items-center gap-2 group">
          <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-rose-500 to-violet-600 flex items-center justify-center text-white text-sm font-bold">
            T
          </div>
          <span className="font-bold text-lg text-white">Telepathy</span>
        </a>

        {/* 中部导航 */}
        <div className="hidden md:flex items-center gap-8">
          {[
            { label: '功能特性', href: '#features' },
            { label: '工作原理', href: '#how-it-works' },
            { label: '立即下载', href: '#download' },
          ].map(({ label, href }) => (
            <a
              key={href}
              href={href}
              className="text-[#94a3b8] hover:text-white text-sm transition-colors"
            >
              {label}
            </a>
          ))}
        </div>

        {/* 右侧下载按钮 */}
        <a
          href="#download"
          className="px-4 py-2 text-sm font-semibold rounded-lg bg-gradient-to-r from-rose-500 to-violet-600 text-white hover:opacity-90 transition"
        >
          立即下载
        </a>
      </div>
    </nav>
  );
}
```

- [ ] **Step 4: 运行测试（应通过）**

```bash
cd website && npx vitest run src/components/Navbar.test.tsx
```
期望：PASS — 2 tests passed

- [ ] **Step 5: 提交**

```bash
cd /Users/mac/project/telepathy/.worktrees/website
git add website/src/components/Navbar.tsx website/src/components/Navbar.test.tsx
git commit -m "feat: add Navbar component with glassmorphism scroll effect"
```

---

## Task 2: Features 组件

**Files:**
- Create: `website/src/components/Features.tsx`
- Create: `website/src/components/Features.test.tsx`

- [ ] **Step 1: 写失败测试**

```tsx
// website/src/components/Features.test.tsx
import { render, screen } from '@testing-library/react';
import { Features } from './Features';

describe('Features', () => {
  it('renders section heading', () => {
    render(<Features />);
    expect(screen.getByText(/为什么选择 Telepathy/i)).toBeInTheDocument();
  });

  it('renders all 6 feature cards', () => {
    render(<Features />);
    expect(screen.getByText('100% 本地隐私')).toBeInTheDocument();
    expect(screen.getByText('RAG 智能检索')).toBeInTheDocument();
    expect(screen.getByText('极速本地响应')).toBeInTheDocument();
    expect(screen.getByText('多格式文档解析')).toBeInTheDocument();
    expect(screen.getByText('来源精准引用')).toBeInTheDocument();
    expect(screen.getByText('完全离线可用')).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: 运行测试（应失败）**

```bash
cd website && npx vitest run src/components/Features.test.tsx
```
期望：FAIL — "Cannot find module './Features'"

- [ ] **Step 3: 实现 Features**

```tsx
// website/src/components/Features.tsx
const FEATURES = [
  {
    icon: '🔒',
    title: '100% 本地隐私',
    description: '所有数据留在您的设备，从不上传任何云端服务，您的知识永远属于您。',
  },
  {
    icon: '🧠',
    title: 'RAG 智能检索',
    description: '向量语义搜索引擎，精准理解您问题的意图，不只是关键词匹配。',
  },
  {
    icon: '⚡',
    title: '极速本地响应',
    description: '基于 llama.cpp 深度优化，无需联网等待，本地推理秒级给出回答。',
  },
  {
    icon: '📄',
    title: '多格式文档解析',
    description: '支持 PDF、Word、Markdown、网页等主流格式，一站式导入您的知识。',
  },
  {
    icon: '🎯',
    title: '来源精准引用',
    description: '每条回答均标注来源文件与具体段落，可验证、可追溯、可信赖。',
  },
  {
    icon: '🔌',
    title: '完全离线可用',
    description: '无需联网，飞机上、地下室、任何场景都能正常工作。',
  },
];

export function Features() {
  return (
    <section id="features" className="py-24 px-4">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-4xl font-bold mb-4">为什么选择 Telepathy</h2>
          <p className="text-[#94a3b8] text-lg max-w-2xl mx-auto">
            专为需要完全掌控自己数据的用户设计，没有订阅费，没有数据泄露风险。
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {FEATURES.map((feature) => (
            <div
              key={feature.title}
              className="glass-card rounded-xl p-6 flex flex-col gap-3"
            >
              <div className="text-3xl">{feature.icon}</div>
              <h3 className="text-lg font-semibold text-white">{feature.title}</h3>
              <p className="text-[#94a3b8] text-sm leading-relaxed">{feature.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 4: 运行测试（应通过）**

```bash
cd website && npx vitest run src/components/Features.test.tsx
```
期望：PASS — 2 tests passed

- [ ] **Step 5: 提交**

```bash
cd /Users/mac/project/telepathy/.worktrees/website
git add website/src/components/Features.tsx website/src/components/Features.test.tsx
git commit -m "feat: add Features section with 6 glassmorphism cards"
```

---

## Task 3: HowItWorks 组件

**Files:**
- Create: `website/src/components/HowItWorks.tsx`
- Create: `website/src/components/HowItWorks.test.tsx`

- [ ] **Step 1: 写失败测试**

```tsx
// website/src/components/HowItWorks.test.tsx
import { render, screen } from '@testing-library/react';
import { HowItWorks } from './HowItWorks';

describe('HowItWorks', () => {
  it('renders section heading', () => {
    render(<HowItWorks />);
    expect(screen.getByText(/三步开启您的专属知识库/i)).toBeInTheDocument();
  });

  it('renders all 3 steps', () => {
    render(<HowItWorks />);
    expect(screen.getByText('导入您的文档')).toBeInTheDocument();
    expect(screen.getByText('自动智能向量化')).toBeInTheDocument();
    expect(screen.getByText('自然语言问答')).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: 运行测试（应失败）**

```bash
cd website && npx vitest run src/components/HowItWorks.test.tsx
```
期望：FAIL — "Cannot find module './HowItWorks'"

- [ ] **Step 3: 实现 HowItWorks**

```tsx
// website/src/components/HowItWorks.tsx
"use client";
import { useEffect, useRef, useState } from 'react';

const STEPS = [
  {
    number: '01',
    title: '导入您的文档',
    description: '拖放或选择文件，支持 PDF、Word、Markdown、网页等多种格式批量导入，几秒钟即可完成。',
    visual: (
      <div className="flex flex-col gap-2 w-full">
        {['研究报告.pdf', '会议记录.docx', '技术文档.md'].map((file, i) => (
          <div
            key={file}
            className="flex items-center gap-3 bg-[#0f172a] rounded-lg px-4 py-2.5 border border-white/5"
          >
            <span className="text-rose-400 text-lg">📄</span>
            <span className="text-sm text-[#94a3b8] flex-1">{file}</span>
            <span className="text-xs text-emerald-400">✓ 已导入</span>
          </div>
        ))}
        <div className="h-1.5 rounded-full bg-white/5 overflow-hidden mt-1">
          <div className="h-full w-full bg-gradient-to-r from-rose-500 to-violet-600 rounded-full" />
        </div>
      </div>
    ),
  },
  {
    number: '02',
    title: '自动智能向量化',
    description: 'Telepathy 自动将您的文档内容转化为高维语义向量，构建专属的本地私人知识图谱。',
    visual: (
      <div className="grid grid-cols-5 gap-2 w-full">
        {Array.from({ length: 15 }).map((_, i) => (
          <div
            key={i}
            className="aspect-square rounded-md bg-gradient-to-br from-rose-500/20 to-violet-600/20 border border-rose-500/10 flex items-center justify-center"
          >
            <div
              className="w-1.5 h-1.5 rounded-full bg-rose-400/60"
              style={{ opacity: 0.3 + (i % 5) * 0.15 }}
            />
          </div>
        ))}
      </div>
    ),
  },
  {
    number: '03',
    title: '自然语言问答',
    description: '用日常语言提问，Telepathy 即时检索您的知识库，给出基于您文档的精准回答，并附上来源引用。',
    visual: (
      <div className="flex flex-col gap-3 w-full">
        <div className="flex justify-end">
          <div className="bg-gradient-to-r from-rose-500/20 to-violet-600/20 border border-rose-500/20 rounded-2xl rounded-tr-sm px-4 py-2.5 max-w-[80%]">
            <p className="text-sm text-white">这份报告的核心结论是什么？</p>
          </div>
        </div>
        <div className="flex justify-start">
          <div className="bg-[#0f172a] border border-white/5 rounded-2xl rounded-tl-sm px-4 py-2.5 max-w-[85%]">
            <p className="text-sm text-[#94a3b8]">根据《研究报告.pdf》第3页，核心结论是…</p>
            <p className="text-xs text-rose-400/70 mt-1">📎 来源: 研究报告.pdf · 第3页</p>
          </div>
        </div>
      </div>
    ),
  },
];

export function HowItWorks() {
  const [visibleSteps, setVisibleSteps] = useState<Set<number>>(new Set());
  const stepRefs = useRef<(HTMLDivElement | null)[]>([]);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          const index = Number(entry.target.getAttribute('data-step'));
          if (entry.isIntersecting) {
            setVisibleSteps((prev) => new Set([...prev, index]));
          }
        });
      },
      { threshold: 0.3 }
    );

    stepRefs.current.forEach((ref) => {
      if (ref) observer.observe(ref);
    });

    return () => observer.disconnect();
  }, []);

  return (
    <section id="how-it-works" className="py-24 px-4">
      <div className="max-w-5xl mx-auto">
        <div className="text-center mb-20">
          <h2 className="text-4xl font-bold mb-4">三步开启您的专属知识库</h2>
          <p className="text-[#94a3b8] text-lg">简单直觉的体验，强大复杂的引擎。</p>
        </div>

        <div className="flex flex-col gap-20">
          {STEPS.map((step, index) => (
            <div
              key={step.number}
              ref={(el) => { stepRefs.current[index] = el; }}
              data-step={index}
              className={`flex flex-col lg:flex-row items-center gap-12 transition-all duration-700 ${
                visibleSteps.has(index)
                  ? 'opacity-100 translate-y-0'
                  : 'opacity-0 translate-y-8'
              } ${index % 2 === 1 ? 'lg:flex-row-reverse' : ''}`}
            >
              {/* 文字内容 */}
              <div className="flex-1 flex flex-col gap-4">
                <span
                  className="text-7xl font-bold text-gradient opacity-30 leading-none"
                  style={{ transitionDelay: `${index * 100}ms` }}
                >
                  {step.number}
                </span>
                <h3 className="text-2xl font-bold text-white">{step.title}</h3>
                <p className="text-[#94a3b8] leading-relaxed">{step.description}</p>
              </div>

              {/* 可视化区域 */}
              <div
                className="flex-1 glass-card rounded-xl p-6 flex items-center justify-center min-h-[180px]"
                style={{ transitionDelay: `${index * 100 + 150}ms` }}
              >
                {step.visual}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 4: 运行测试（应通过）**

```bash
cd website && npx vitest run src/components/HowItWorks.test.tsx
```
期望：PASS — 2 tests passed

- [ ] **Step 5: 提交**

```bash
cd /Users/mac/project/telepathy/.worktrees/website
git add website/src/components/HowItWorks.tsx website/src/components/HowItWorks.test.tsx
git commit -m "feat: add HowItWorks section with scroll-triggered animation"
```

---

## Task 4: Download + Footer 组件

**Files:**
- Create: `website/src/components/Download.tsx`
- Create: `website/src/components/Download.test.tsx`
- Create: `website/src/components/Footer.tsx`
- Create: `website/src/components/Footer.test.tsx`

- [ ] **Step 1: 写 Download 失败测试**

```tsx
// website/src/components/Download.test.tsx
import { render, screen } from '@testing-library/react';
import { Download } from './Download';
import { UpdaterInfo } from '../types/updater';

const mockUpdater: UpdaterInfo = {
  version: '0.3.7',
  notes: 'Test',
  pub_date: '2026-05-20T00:00:00Z',
  platforms: {
    'darwin-aarch64': { url: 'https://example.com/mac-arm.dmg' },
    'darwin-x86_64': { url: 'https://example.com/mac-intel.dmg' },
    'windows-x86_64': { url: 'https://example.com/win.msi' },
    'linux-x86_64': { url: 'https://example.com/linux.AppImage' },
  },
};

describe('Download', () => {
  it('renders section heading', () => {
    render(<Download updater={mockUpdater} />);
    expect(screen.getByText(/选择您的平台/i)).toBeInTheDocument();
  });

  it('renders version number', () => {
    render(<Download updater={mockUpdater} />);
    expect(screen.getByText(/0\.3\.7/)).toBeInTheDocument();
  });

  it('renders macOS download links', () => {
    render(<Download updater={mockUpdater} />);
    expect(screen.getByRole('link', { name: /Apple Silicon/i })).toHaveAttribute(
      'href',
      'https://example.com/mac-arm.dmg'
    );
    expect(screen.getByRole('link', { name: /Intel/i })).toHaveAttribute(
      'href',
      'https://example.com/mac-intel.dmg'
    );
  });
});
```

- [ ] **Step 2: 写 Footer 失败测试**

```tsx
// website/src/components/Footer.test.tsx
import { render, screen } from '@testing-library/react';
import { Footer } from './Footer';

describe('Footer', () => {
  it('renders copyright text', () => {
    render(<Footer />);
    expect(screen.getByText(/2026 Telepathy/i)).toBeInTheDocument();
  });
});
```

- [ ] **Step 3: 运行测试（应失败）**

```bash
cd website && npx vitest run src/components/Download.test.tsx src/components/Footer.test.tsx
```
期望：FAIL — "Cannot find module"

- [ ] **Step 4: 实现 Download**

```tsx
// website/src/components/Download.tsx
import { UpdaterInfo } from '@/types/updater';

const PLATFORMS = [
  {
    name: 'macOS',
    icon: '🍎',
    buttons: [
      { label: 'Apple Silicon', platformKey: 'darwin-aarch64' as const },
      { label: 'Intel', platformKey: 'darwin-x86_64' as const },
    ],
  },
  {
    name: 'Windows',
    icon: '🪟',
    buttons: [
      { label: 'Windows x64 (.msi)', platformKey: 'windows-x86_64' as const },
    ],
  },
  {
    name: 'Linux',
    icon: '🐧',
    buttons: [
      { label: 'Linux x64 (.AppImage)', platformKey: 'linux-x86_64' as const },
    ],
  },
];

export function Download({ updater }: { updater: UpdaterInfo }) {
  const formattedDate = new Date(updater.pub_date).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });

  return (
    <section
      id="download"
      className="py-24 px-4 bg-gradient-to-b from-transparent via-[#0b0f1e]/50 to-[#070a13]"
    >
      <div className="max-w-5xl mx-auto">
        <div className="text-center mb-6">
          <h2 className="text-4xl font-bold mb-4">选择您的平台，即刻开始</h2>
          <p className="text-[#94a3b8]">
            最新版本 <span className="text-white font-semibold">v{updater.version}</span>，发布于 {formattedDate}
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-10">
          {PLATFORMS.map((platform) => (
            <div key={platform.name} className="glass-card rounded-xl p-6 flex flex-col gap-4">
              <div className="flex items-center gap-3">
                <span className="text-2xl">{platform.icon}</span>
                <h3 className="text-lg font-semibold text-white">{platform.name}</h3>
              </div>
              <div className="flex flex-col gap-2">
                {platform.buttons.map(({ label, platformKey }) => {
                  const url = updater.platforms[platformKey]?.url;
                  return url ? (
                    <a
                      key={label}
                      href={url}
                      className="w-full text-center px-4 py-2.5 rounded-lg border border-rose-500/30 text-rose-300 hover:bg-rose-500/10 transition text-sm font-medium"
                    >
                      ↓ {label}
                    </a>
                  ) : (
                    <span
                      key={label}
                      className="w-full text-center px-4 py-2.5 rounded-lg border border-white/5 text-[#475569] text-sm"
                    >
                      {label} (暂未发布)
                    </span>
                  );
                })}
              </div>
            </div>
          ))}
        </div>

        <div className="text-center">
          <a
            href="https://github.com/jackjie1025/telepathy/releases"
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-[#94a3b8] hover:text-white transition"
          >
            查看所有历史版本 →
          </a>
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 5: 实现 Footer**

```tsx
// website/src/components/Footer.tsx
export function Footer() {
  return (
    <footer className="py-8 px-4 border-t border-white/5">
      <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
        <p className="text-sm text-[#475569]">
          © 2026 Telepathy. All rights reserved.
        </p>
        <a
          href="https://github.com/jackjie1025/telepathy"
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm text-[#94a3b8] hover:text-white transition"
        >
          GitHub →
        </a>
      </div>
    </footer>
  );
}
```

- [ ] **Step 6: 运行测试（应通过）**

```bash
cd website && npx vitest run src/components/Download.test.tsx src/components/Footer.test.tsx
```
期望：PASS — 4 tests passed

- [ ] **Step 7: 提交**

```bash
cd /Users/mac/project/telepathy/.worktrees/website
git add website/src/components/Download.tsx website/src/components/Download.test.tsx website/src/components/Footer.tsx website/src/components/Footer.test.tsx
git commit -m "feat: add Download and Footer components"
```

---

## Task 5: 升级 Hero 组件

**Files:**
- Modify: `website/src/components/Hero.tsx`
- Modify: `website/src/components/Hero.test.tsx`

- [ ] **Step 1: 更新 Hero 测试（补充新 UI 元素）**

```tsx
// website/src/components/Hero.test.tsx
import { render, screen } from '@testing-library/react';
import { Hero } from './Hero';
import { UpdaterInfo } from '../types/updater';

const mockUpdater: UpdaterInfo = {
  version: '0.3.7',
  notes: 'Test',
  pub_date: '2026-05-20',
  platforms: {
    'darwin-aarch64': { url: 'mac-url' },
    'windows-x86_64': { url: 'win-url' },
    'linux-x86_64': { url: 'linux-url' },
  },
};

describe('Hero', () => {
  it('renders version info', () => {
    render(<Hero updater={mockUpdater} />);
    expect(screen.getByText(/0\.3\.7/)).toBeInTheDocument();
  });

  it('renders badge', () => {
    render(<Hero updater={mockUpdater} />);
    expect(screen.getByText(/100% 本地运行/i)).toBeInTheDocument();
  });

  it('renders secondary CTA button', () => {
    render(<Hero updater={mockUpdater} />);
    expect(screen.getByRole('link', { name: /了解功能/i })).toBeInTheDocument();
  });

  it('renders platform badges', () => {
    render(<Hero updater={mockUpdater} />);
    expect(screen.getByText('macOS')).toBeInTheDocument();
    expect(screen.getByText('Windows')).toBeInTheDocument();
    expect(screen.getByText('Linux')).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: 运行测试（部分应失败）**

```bash
cd website && npx vitest run src/components/Hero.test.tsx
```
期望：badge/secondary CTA/platform badges 测试 FAIL

- [ ] **Step 3: 重写 Hero 组件**

```tsx
// website/src/components/Hero.tsx
"use client";
import { useOSDetection } from '@/hooks/useOSDetection';
import { UpdaterInfo } from '@/types/updater';
import Image from 'next/image';

const PLATFORMS = [
  { label: 'macOS', icon: '🍎' },
  { label: 'Windows', icon: '🪟' },
  { label: 'Linux', icon: '🐧' },
];

export function Hero({ updater }: { updater: UpdaterInfo }) {
  const { os, isMounted } = useOSDetection();

  let downloadUrl = '#download';
  let buttonText = '立即下载最新版';

  if (isMounted) {
    if (os === 'macOS' && updater.platforms['darwin-aarch64']?.url) {
      downloadUrl = updater.platforms['darwin-aarch64'].url;
      buttonText = '立即下载 macOS 版';
    } else if (os === 'Windows' && updater.platforms['windows-x86_64']?.url) {
      downloadUrl = updater.platforms['windows-x86_64'].url;
      buttonText = '立即下载 Windows 版';
    } else if (os === 'Linux' && updater.platforms['linux-x86_64']?.url) {
      downloadUrl = updater.platforms['linux-x86_64'].url;
      buttonText = '立即下载 Linux 版';
    }
  }

  return (
    <section className="min-h-screen flex items-center justify-center pt-24 pb-16 px-4">
      <div className="max-w-7xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
        {/* 左侧内容 */}
        <div className="flex flex-col gap-6">
          {/* 徽章标签 */}
          <div className="inline-flex items-center gap-2 self-start px-3 py-1 rounded-full border border-rose-500/30 bg-rose-500/5 text-rose-300 text-sm">
            <span>🔒</span>
            <span>100% 本地运行 · 完全开源</span>
          </div>

          {/* 主标题 */}
          <h1 className="text-5xl lg:text-6xl font-bold leading-tight">
            您的专属本地 AI<br />
            <span className="text-gradient">智能知识库</span>
          </h1>

          {/* 副标题 */}
          <p className="text-xl text-[#94a3b8] leading-relaxed">
            基于本地 LLM 驱动，100% 隐私安全，超强向量索引与多模态解析，打造您专属的本地第二大脑。
          </p>

          {/* 双按钮 */}
          <div className="flex flex-wrap items-center gap-4">
            <a
              href={downloadUrl}
              className="px-8 py-4 bg-gradient-to-r from-rose-500 via-pink-500 to-violet-600 rounded-lg text-white font-semibold text-lg hover:opacity-90 transition shadow-lg shadow-rose-500/25"
            >
              {buttonText}
            </a>
            <a
              href="#features"
              className="px-8 py-4 rounded-lg border border-white/10 text-[#94a3b8] font-semibold text-lg hover:border-white/30 hover:text-white transition"
            >
              了解功能 →
            </a>
          </div>

          {/* 版本信息 */}
          <p className="text-sm text-[#475569]">
            当前最新版本：v{updater.version} · 100% 离线隐私安全
          </p>

          {/* 平台徽标 */}
          <div className="flex items-center gap-5 pt-2">
            {PLATFORMS.map(({ label, icon }) => (
              <div key={label} className="flex items-center gap-1.5 text-[#64748b] text-sm">
                <span>{icon}</span>
                <span>{label}</span>
              </div>
            ))}
          </div>
        </div>

        {/* 右侧截图 */}
        <div className="relative animate-float">
          <div className="absolute -inset-2 bg-gradient-to-r from-rose-500 to-violet-600 rounded-2xl blur-xl opacity-20" />
          <div className="relative bg-[#0b0f19] border border-gray-800 rounded-xl aspect-video overflow-hidden shadow-2xl">
            <Image
              src="/app-screenshot.png"
              alt="Telepathy App Preview"
              width={1920}
              height={1080}
              className="w-full h-full object-cover"
              priority
            />
          </div>
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 4: 运行测试（应全部通过）**

```bash
cd website && npx vitest run src/components/Hero.test.tsx
```
期望：PASS — 4 tests passed

- [ ] **Step 5: 提交**

```bash
cd /Users/mac/project/telepathy/.worktrees/website
git add website/src/components/Hero.tsx website/src/components/Hero.test.tsx
git commit -m "feat: upgrade Hero with badge, dual CTA, and platform badges"
```

---

## Task 6: 组装 layout.tsx 和 page.tsx

**Files:**
- Modify: `website/src/app/layout.tsx`
- Modify: `website/src/app/page.tsx`
- Modify: `website/src/app/page.test.tsx`

- [ ] **Step 1: 更新 page.tsx 测试**

```tsx
// website/src/app/page.test.tsx
import { render, screen } from '@testing-library/react';
import { expect, vi } from 'vitest';
import Page from './page';

global.fetch = vi.fn(() =>
  Promise.resolve({
    ok: true,
    json: () =>
      Promise.resolve({
        version: '0.3.7',
        notes: '',
        pub_date: '2026-05-20T00:00:00Z',
        platforms: {
          'darwin-aarch64': { url: 'https://example.com/mac.dmg' },
        },
      }),
  })
) as unknown as typeof fetch;

describe('Home Page', () => {
  it('renders Hero and Download sections', async () => {
    const PageComponent = await Page();
    render(PageComponent);
    expect(await screen.findByText(/0\.3\.7/)).toBeInTheDocument();
    expect(screen.getByText(/选择您的平台/i)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: 运行测试（Download 断言应失败）**

```bash
cd website && npx vitest run src/app/page.test.tsx
```
期望：FAIL — "选择您的平台" not found

- [ ] **Step 3: 更新 layout.tsx（引入 Inter 字体）**

```tsx
// website/src/app/layout.tsx
import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { Navbar } from "@/components/Navbar";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Telepathy — 本地优先的 AI 智能知识库",
  description:
    "基于本地 LLM 驱动，100% 隐私安全，超强向量索引与多模态解析，打造您专属的本地第二大脑。",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="zh-CN">
      <body className={inter.className}>
        <Navbar />
        {children}
      </body>
    </html>
  );
}
```

- [ ] **Step 4: 更新 page.tsx（组装全部组件）**

```tsx
// website/src/app/page.tsx
import { Hero } from "@/components/Hero";
import { Features } from "@/components/Features";
import { HowItWorks } from "@/components/HowItWorks";
import { Download } from "@/components/Download";
import { Footer } from "@/components/Footer";
import { UpdaterInfo } from "@/types/updater";

const defaultUpdater: UpdaterInfo = {
  version: "0.1.0",
  notes: "Initial Release",
  pub_date: new Date().toISOString(),
  platforms: {},
};

async function getUpdaterInfo(): Promise<UpdaterInfo> {
  try {
    const res = await fetch(
      "https://download.telepathy-app.com/releases/updater.json",
      { next: { revalidate: 3600 } }
    );
    if (!res.ok) return defaultUpdater;
    return await res.json();
  } catch {
    return defaultUpdater;
  }
}

export default async function Home() {
  const updater = await getUpdaterInfo();

  return (
    <main className="min-h-screen">
      <Hero updater={updater} />
      <Features />
      <HowItWorks />
      <Download updater={updater} />
      <Footer />
    </main>
  );
}
```

- [ ] **Step 5: 运行所有测试（应全部通过）**

```bash
cd website && npx vitest run
```
期望：PASS — 所有测试通过

- [ ] **Step 6: 提交**

```bash
cd /Users/mac/project/telepathy/.worktrees/website
git add website/src/app/layout.tsx website/src/app/page.tsx website/src/app/page.test.tsx
git commit -m "feat: assemble full landing page with all sections"
```

---

## 验证清单

完成所有 Task 后执行：

```bash
# 1. 确认所有测试通过
cd website && npx vitest run

# 2. 本地启动预览
cd website && npm run dev
# 打开 http://localhost:3000 验证：
# - Navbar 固定在顶部，滚动后毛玻璃效果
# - Hero 显示徽章、双按钮、平台徽标
# - Features 显示 6 张卡片，hover 效果正确
# - HowItWorks 滚动时三步动效依次触发
# - Download 显示三平台下载卡片
# - Footer 底部版权信息
```
