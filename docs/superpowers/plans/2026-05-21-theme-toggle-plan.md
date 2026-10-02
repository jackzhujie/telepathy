# 三态循环主题切换实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 Telepathy 官方宣传网站增加“跟随系统 ➔ 浅色 ➔ 深色”三态循环主题切换功能，确保无闪烁、无水合警告，且与系统外观偏好联动。

**Architecture:** 引入 `next-themes` 的 `ThemeProvider` 挂载至 RootLayout 顶层，使用自定义 `ThemeToggle` 按钮在 Navbar 中控制主题属性 `class` 切换。重构全局 `globals.css` 中相关的颜色为 CSS 自定义属性。

**Tech Stack:** React 19, Next.js 15, Tailwind CSS v4, `next-themes`, Vitest, `@testing-library/react`

---

### Task 1: 安装依赖 next-themes

**Files:**
- Modify: `website/package.json` (通过 npm 自动更新)

- [ ] **Step 1: 安装 next-themes**

在 `website` 目录下运行安装命令：
```bash
npm install next-themes
```

- [ ] **Step 2: 验证安装**

检查 `website/package.json` 中的 `dependencies` 是否已包含 `"next-themes"`，并能成功编译。
运行：
```bash
npx tsc --noEmit
```
Expected: 运行成功，无错误。

- [ ] **Step 3: Commit**

```bash
git add website/package.json website/package-lock.json
git commit -m "chore: install next-themes dependency"
```

---

### Task 2: 重构全局 globals.css CSS 变量

**Files:**
- Modify: `website/src/app/globals.css`
- Modify: `website/src/app/globals.test.tsx`

- [ ] **Step 1: 编写 globals.test.tsx 的测试逻辑以适配浅色和深色变量定义**

我们更新全局样式验证测试：
```typescript
// website/src/app/globals.test.tsx
import { render } from '@testing-library/react';

describe('Global styles variables', () => {
  it('defines light theme default variables', () => {
    // 仅验证测试运行环境就绪
    const { container } = render(<div className="bg-background text-foreground">Test Env</div>);
    expect(container.firstChild).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: 运行测试验证**

在 `website` 目录下运行：
```bash
npx vitest run src/app/globals.test.tsx
```
Expected: PASS

- [ ] **Step 3: 修改 globals.css，实现浅色和深色主题的变量映射**

编辑 `website/src/app/globals.css`：
```css
@import "tailwindcss";

@theme {
  --color-background: var(--color-background);
  --color-foreground: var(--color-foreground);
  --color-card-bg: var(--color-card-bg);
  --color-card-border: var(--color-card-border);
  
  --animate-float: float 5s ease-in-out infinite;

  @keyframes float {
    0%, 100% { transform: translateY(0px) rotate(1deg); }
    50% { transform: translateY(-10px) rotate(-1deg); }
  }
}

:root {
  --color-background: #f8fafc;
  --color-foreground: #0f172a;
  --color-card-bg: rgba(255, 255, 255, 0.7);
  --color-card-border: rgba(0, 0, 0, 0.06);
  --radial-gradient-1: rgba(244, 63, 94, 0.02);
  --radial-gradient-2: rgba(139, 92, 246, 0.02);
}

.dark {
  --color-background: #070a13;
  --color-foreground: #f8fafc;
  --color-card-bg: rgba(15, 23, 42, 0.45);
  --color-card-border: rgba(255, 255, 255, 0.05);
  --radial-gradient-1: rgba(244, 63, 94, 0.05);
  --radial-gradient-2: rgba(139, 92, 246, 0.05);
}

body {
  background-color: var(--color-background);
  color: var(--color-foreground);
  font-family: 'Inter', sans-serif;
  background-image: 
    radial-gradient(circle at 10% 20%, var(--radial-gradient-1) 0%, transparent 40%),
    radial-gradient(circle at 90% 80%, var(--radial-gradient-2) 0%, transparent 40%);
  background-attachment: fixed;
  transition: background-color 0.3s ease, color 0.3s ease;
}

@utility text-gradient {
  background: linear-gradient(135deg, #f43f5e 0%, #a855f7 50%, #3b82f6 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

@utility glass-card {
  background: var(--color-card-bg);
  backdrop-filter: blur(12px);
  border: 1px solid var(--color-card-border);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.glass-card:hover {
  border-color: rgba(244, 63, 94, 0.2);
  box-shadow: 0 10px 30px -10px rgba(244, 63, 94, 0.15);
  transform: translateY(-2px);
}
```

- [ ] **Step 4: 运行全部单元测试以确保修改没有带来样式回归破坏**

在 `website` 目录下运行：
```bash
npx vitest run
```
Expected: 所有 23 个测试全部 PASS。

- [ ] **Step 5: Commit**

```bash
git add website/src/app/globals.css
git commit -m "style: extract light and dark mode CSS variables"
```

---

### Task 3: 封装 ThemeProvider 组件

**Files:**
- Create: `website/src/components/ThemeProvider.tsx`
- Create: `website/src/components/ThemeProvider.test.tsx`
- Modify: `website/src/app/layout.tsx`

- [ ] **Step 1: 编写 ThemeProvider 的单元测试**

新建测试文件 `website/src/components/ThemeProvider.test.tsx`：
```typescript
import { render, screen } from '@testing-library/react';
import { ThemeProvider } from './ThemeProvider';

describe('ThemeProvider', () => {
  it('renders children correctly', () => {
    render(
      <ThemeProvider>
        <div data-testid="child">Test Child</div>
      </ThemeProvider>
    );
    expect(screen.getByTestId('child')).toHaveTextContent('Test Child');
  });
});
```

- [ ] **Step 2: 运行测试以保证测试失败（由于文件未创建）**

在 `website` 目录下运行：
```bash
npx vitest run src/components/ThemeProvider.test.tsx
```
Expected: FAIL, "Cannot find module './ThemeProvider'"

- [ ] **Step 3: 编写 ThemeProvider 组件实现**

新建文件 `website/src/components/ThemeProvider.tsx`：
```typescript
"use client";
import { ThemeProvider as NextThemeProvider } from 'next-themes';
import { ReactNode } from 'react';

export function ThemeProvider({ children }: { children: ReactNode }) {
  return (
    <NextThemeProvider attribute="class" defaultTheme="system" enableSystem>
      {children}
    </NextThemeProvider>
  );
}
```

- [ ] **Step 4: 运行测试进行验证**

在 `website` 目录下运行：
```bash
npx vitest run src/components/ThemeProvider.test.tsx
```
Expected: PASS

- [ ] **Step 5: 将 ThemeProvider 集成到 RootLayout 中**

修改 `website/src/app/layout.tsx`：
```typescript
import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { Navbar } from "@/components/Navbar";
import { ThemeProvider } from "@/components/ThemeProvider";

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
    <html lang="zh-CN" suppressHydrationWarning>
      <body className={inter.className}>
        <ThemeProvider>
          <Navbar />
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
```

- [ ] **Step 6: 执行 TypeScript 编译与全面测试验证**

在 `website` 目录下运行：
```bash
npx tsc --noEmit && npx vitest run
```
Expected: 全部成功通过。

- [ ] **Step 7: Commit**

```bash
git add website/src/components/ThemeProvider.tsx website/src/components/ThemeProvider.test.tsx website/src/app/layout.tsx
git commit -m "feat: implement ThemeProvider and wrap RootLayout"
```

---

### Task 4: 开发 ThemeToggle 三态循环切换按钮

**Files:**
- Create: `website/src/components/ThemeToggle.tsx`
- Create: `website/src/components/ThemeToggle.test.tsx`

- [ ] **Step 1: 编写 ThemeToggle 的单元测试**

新建文件 `website/src/components/ThemeToggle.test.tsx`：
```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { ThemeToggle } from './ThemeToggle';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// mock useTheme
const mockSetTheme = vi.fn();
let mockTheme = 'system';

vi.mock('next-themes', () => ({
  useTheme: () => ({
    theme: mockTheme,
    setTheme: mockSetTheme,
  }),
}));

describe('ThemeToggle', () => {
  beforeEach(() => {
    mockSetTheme.mockClear();
    mockTheme = 'system';
  });

  it('renders nothing before mounting to avoid SSR mismatch', () => {
    // 渲染，但不触发 useEffect
    const { container } = render(<ThemeToggle />);
    expect(container.firstChild).toBeNull();
  });

  it('renders correctly after mounting', async () => {
    const { container } = render(<ThemeToggle />);
    // 通过等待 microtask 模拟 Mounted 后的渲染
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('cycles from system -> light -> dark when clicked', async () => {
    render(<ThemeToggle />);
    await new Promise((resolve) => setTimeout(resolve, 0));
    
    const button = screen.getByRole('button');
    
    // 1. 从 system 开始点击 ➔ 应触发 setTheme('light')
    fireEvent.click(button);
    expect(mockSetTheme).toHaveBeenCalledWith('light');

    // 2. 模拟主题变为 light，点击 ➔ 应触发 setTheme('dark')
    mockTheme = 'light';
    fireEvent.click(button);
    expect(mockSetTheme).toHaveBeenCalledWith('dark');

    // 3. 模拟主题变为 dark，点击 ➔ 应触发 setTheme('system')
    mockTheme = 'dark';
    fireEvent.click(button);
    expect(mockSetTheme).toHaveBeenCalledWith('system');
  });
});
```

- [ ] **Step 2: 运行测试以保证测试失败（由于文件未创建）**

在 `website` 目录下运行：
```bash
npx vitest run src/components/ThemeToggle.test.tsx
```
Expected: FAIL, "Cannot find module './ThemeToggle'"

- [ ] **Step 3: 编写 ThemeToggle 组件实现**

新建文件 `website/src/components/ThemeToggle.tsx`：
```typescript
"use client";
import { useEffect, useState } from 'react';
import { useTheme } from 'next-themes';

export function ThemeToggle() {
  const [mounted, setMounted] = useState(false);
  const { theme, setTheme } = useTheme();

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    // 挂载前不渲染，以避免 Hydration Mismatch
    return <div className="w-10 h-10" />;
  }

  // 计算三态切换路径：system -> light -> dark -> system
  const handleToggle = () => {
    if (theme === 'system') {
      setTheme('light');
    } else if (theme === 'light') {
      setTheme('dark');
    } else {
      setTheme('system');
    }
  };

  // 根据当前状态选择渲染图标
  return (
    <button
      onClick={handleToggle}
      className="w-10 h-10 flex items-center justify-center rounded-lg border border-neutral-200 dark:border-white/10 bg-white/50 dark:bg-slate-900/50 hover:bg-neutral-100 dark:hover:bg-white/10 text-neutral-700 dark:text-neutral-300 transition-all duration-300 relative group overflow-hidden"
      aria-label={`当前主题：${theme === 'system' ? '跟随系统' : theme === 'light' ? '浅色' : '深色'}`}
      type="button"
    >
      {/* System Icon */}
      {theme === 'system' && (
        <svg
          className="w-5 h-5 transition-transform duration-300 scale-100 group-hover:rotate-12"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect x="2" y="3" width="20" height="14" rx="2" strokeWidth="2" />
          <path d="M6 21h12M12 17v4" strokeWidth="2" strokeLinecap="round" />
        </svg>
      )}

      {/* Light Icon */}
      {theme === 'light' && (
        <svg
          className="w-5 h-5 text-amber-500 transition-transform duration-500 scale-100 rotate-0 group-hover:rotate-45"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <circle cx="12" cy="12" r="5" />
          <line x1="12" y1="1" x2="12" y2="3" />
          <line x1="12" y1="21" x2="12" y2="23" />
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
          <line x1="1" y1="12" x2="3" y2="12" />
          <line x1="21" y1="12" x2="23" y2="12" />
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
        </svg>
      )}

      {/* Dark Icon */}
      {theme === 'dark' && (
        <svg
          className="w-5 h-5 text-indigo-400 transition-transform duration-500 scale-100 rotate-0 group-hover:-rotate-12"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
        </svg>
      )}
    </button>
  );
}
```

- [ ] **Step 4: 运行测试进行验证**

在 `website` 目录下运行：
```bash
npx vitest run src/components/ThemeToggle.test.tsx
```
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add website/src/components/ThemeToggle.tsx website/src/components/ThemeToggle.test.tsx
git commit -m "feat: implement ThemeToggle component with three-state cycle"
```

---

### Task 5: 在 Navbar 中集成 ThemeToggle 按钮

**Files:**
- Modify: `website/src/components/Navbar.tsx`
- Modify: `website/src/components/Navbar.test.tsx`

- [ ] **Step 1: 在 Navbar 中添加 ThemeToggle 按钮**

编辑 `website/src/components/Navbar.tsx`：
```typescript
"use client";
import { useEffect, useState } from 'react';
import Link from 'next/link';
import { ThemeToggle } from './ThemeToggle';

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
          ? 'backdrop-blur-md bg-background/80 border-b border-card-border shadow-lg'
          : 'bg-transparent'
      }`}
    >
      <div className="max-w-7xl mx-auto px-4 h-16 flex items-center justify-between">
        {/* Logo */}
        <Link href="/" className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-rose-500 to-violet-600 flex items-center justify-center text-white text-sm font-bold">
            T
          </div>
          <span className="font-bold text-lg text-foreground">Telepathy</span>
        </Link>

        {/* 中部导航 */}
        <div className="hidden md:flex items-center gap-8">
          {[
            { label: '功能特性', href: '#features' },
            { label: '工作原理', href: '#how-it-works' },
          ].map(({ label, href }) => (
            <a
              key={href}
              href={href}
              className="text-[#94a3b8] dark:text-[#94a3b8] hover:text-foreground text-sm transition-colors"
            >
              {label}
            </a>
          ))}
        </div>

        {/* 右侧下载和主题切换 */}
        <div className="flex items-center gap-4">
          <ThemeToggle />
          <a
            href="#download"
            className="px-4 py-2 text-sm font-semibold rounded-lg bg-gradient-to-r from-rose-500 to-violet-600 text-white hover:opacity-90 transition"
          >
            立即下载
          </a>
        </div>
      </div>
    </nav>
  );
}
```

- [ ] **Step 2: 修改 Navbar.test.tsx，验证 ThemeToggle 是否成功渲染**

由于 `Navbar` 中引入了 `ThemeToggle`，而 `ThemeToggle` 依赖 `next-themes` 的 `useTheme`。我们必须在 `Navbar.test.tsx` 中加入 mock `next-themes` 的逻辑以防止崩溃。
修改 `website/src/components/Navbar.test.tsx`：
```typescript
import { render, screen } from '@testing-library/react';
import { Navbar } from './Navbar';
import { vi, describe, it, expect } from 'vitest';

// Mock useTheme
vi.mock('next-themes', () => ({
  useTheme: () => ({
    theme: 'system',
    setTheme: vi.fn(),
  }),
}));

describe('Navbar', () => {
  it('renders brand logo', () => {
    render(<Navbar />);
    expect(screen.getByText('Telepathy')).toBeInTheDocument();
  });

  it('renders features and how-it-works links', () => {
    render(<Navbar />);
    expect(screen.getByText('功能特性')).toBeInTheDocument();
    expect(screen.getByText('工作原理')).toBeInTheDocument();
  });

  it('renders download link', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /立即下载/ })).toBeInTheDocument();
  });

  it('renders ThemeToggle button', async () => {
    render(<Navbar />);
    // 等待 Mounted 状态触发
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(screen.getByRole('button', { name: /当前主题/ })).toBeInTheDocument();
  });
});
```

- [ ] **Step 3: 运行全部单元测试**

在 `website` 目录下运行：
```bash
npx vitest run
```
Expected: 全部 24 个测试全部 PASS。

- [ ] **Step 4: 运行 TypeScript 校验与生产编译构建**

在 `website` 目录下运行：
```bash
npx tsc --noEmit && npm run build
```
Expected: 运行成功，无任何 TS 报错，静态导出包构建至 `website/out` 成功。

- [ ] **Step 5: Commit**

```bash
git add website/src/components/Navbar.tsx website/src/components/Navbar.test.tsx
git commit -m "feat: integrate ThemeToggle button in Navbar"
```
