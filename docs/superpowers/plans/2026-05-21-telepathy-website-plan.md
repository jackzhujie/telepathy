# Telepathy Website Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Telepathy official website using Next.js 15, React, and Tailwind CSS, featuring SSR for update metadata, automatic OS detection for downloads, and a dark tech aesthetic.

**Architecture:** A standalone Next.js App Router project located in the `website/` directory. It uses server-side fetching to retrieve `updater.json` metadata for the latest release, passing it down to client components. Client components will use `navigator.userAgent` to detect the OS and update the primary download button accordingly. 

**Tech Stack:** Next.js 15 (App Router), React 19, TypeScript, Tailwind CSS, Lucide React, Vitest (for testing).

---

### Task 1: Scaffold Next.js Project & Testing Environment

**Files:**
- Create: `website/package.json` (via create-next-app)
- Create: `website/next.config.ts`
- Create: `website/vitest.config.ts`

- [ ] **Step 1: Initialize Next.js app in non-interactive mode**
```bash
npx -y create-next-app@15 website --typescript --tailwind --eslint --app --src-dir --import-alias "@/*" --use-npm --yes
```

- [ ] **Step 2: Configure `next.config.ts` for standalone output**
```typescript
// website/next.config.ts
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "download.telepathy-app.com",
      },
    ],
  },
};

export default nextConfig;
```

- [ ] **Step 3: Install testing libraries**
```bash
cd website && npm install -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/jest-dom @testing-library/dom
```

- [ ] **Step 4: Configure Vitest**
```typescript
// website/vitest.config.ts
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./vitest.setup.ts'],
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  }
})
```

- [ ] **Step 5: Create Vitest setup file**
```typescript
// website/vitest.setup.ts
import '@testing-library/jest-dom';
```

- [ ] **Step 6: Verify testing environment**
Run: `cd website && npx vitest run`
Expected: "No test files found" (or successful run if it finds empty default test).

- [ ] **Step 7: Commit**
```bash
git add website/
git commit -m "chore: scaffold Next.js website and setup vitest"
```

---

### Task 2: Global Styles and Tailwind Configuration

**Files:**
- Modify: `website/tailwind.config.ts`
- Modify: `website/src/app/globals.css`
- Create: `website/src/app/globals.test.tsx`

- [ ] **Step 1: Write a failing test for global styles (dummy test just to ensure setup works)**
```tsx
// website/src/app/globals.test.tsx
import { render } from '@testing-library/react';

describe('Global styles environment', () => {
  it('renders a div to test env', () => {
    const { container } = render(<div className="bg-[#070a13] text-gradient">Test</div>);
    expect(container.firstChild).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it passes**
Run: `cd website && npx vitest run`

- [ ] **Step 3: Update `tailwind.config.ts`**
```typescript
// website/tailwind.config.ts
import type { Config } from "tailwindcss";

export default {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "#070a13",
        foreground: "#f8fafc",
      },
    },
  },
  plugins: [],
} satisfies Config;
```

- [ ] **Step 4: Update `src/app/globals.css`**
```css
/* website/src/app/globals.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  body {
    background-color: #070a13;
    color: #f8fafc;
    font-family: 'Inter', sans-serif;
    background-image: 
      radial-gradient(circle at 10% 20%, rgba(244, 63, 94, 0.05) 0%, transparent 40%),
      radial-gradient(circle at 90% 80%, rgba(139, 92, 246, 0.05) 0%, transparent 40%);
    background-attachment: fixed;
  }
}

.text-gradient {
  background: linear-gradient(135deg, #f43f5e 0%, #a855f7 50%, #3b82f6 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.glass-card {
  background: rgba(15, 23, 42, 0.45);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.05);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.glass-card:hover {
  border-color: rgba(244, 63, 94, 0.2);
  box-shadow: 0 10px 30px -10px rgba(244, 63, 94, 0.15);
  transform: translateY(-2px);
}

@keyframes float {
  0%, 100% { transform: translateY(0px) rotate(1deg); }
  50% { transform: translateY(-10px) rotate(-1deg); }
}

.animate-float {
  animation: float 5s ease-in-out infinite;
}
```

- [ ] **Step 5: Run tests and Commit**
```bash
cd website && npx vitest run
git add .
git commit -m "style: configure tailwind and global css"
```

---

### Task 3: OS Detection Hook and Download Helper

**Files:**
- Create: `website/src/hooks/useOSDetection.ts`
- Create: `website/src/hooks/useOSDetection.test.ts`
- Create: `website/src/types/updater.ts`

- [ ] **Step 1: Define Updater Types**
```typescript
// website/src/types/updater.ts
export interface PlatformInfo {
  url: string;
  signature?: string;
}

export interface UpdaterInfo {
  version: string;
  notes: string;
  pub_date: string;
  platforms: {
    "darwin-aarch64"?: PlatformInfo;
    "darwin-x86_64"?: PlatformInfo;
    "windows-x86_64"?: PlatformInfo;
    "linux-x86_64"?: PlatformInfo;
    [key: string]: PlatformInfo | undefined;
  };
}
```

- [ ] **Step 2: Write failing test for useOSDetection hook**
```typescript
// website/src/hooks/useOSDetection.test.ts
import { renderHook } from '@testing-library/react';
import { useOSDetection } from './useOSDetection';

describe('useOSDetection', () => {
  it('should detect Windows', () => {
    Object.defineProperty(window, 'navigator', {
      value: { userAgent: 'Windows NT 10.0' },
      configurable: true,
    });
    const { result } = renderHook(() => useOSDetection());
    expect(result.current.os).toBe('Windows');
  });

  it('should detect macOS', () => {
    Object.defineProperty(window, 'navigator', {
      value: { userAgent: 'Macintosh; Intel Mac OS X 10_15_7' },
      configurable: true,
    });
    const { result } = renderHook(() => useOSDetection());
    expect(result.current.os).toBe('macOS');
  });
});
```

- [ ] **Step 3: Run test (should fail because hook doesn't exist)**
Run: `cd website && npx vitest run`

- [ ] **Step 4: Implement `useOSDetection`**
```typescript
// website/src/hooks/useOSDetection.ts
import { useState, useEffect } from 'react';

export type OS = 'macOS' | 'Windows' | 'Linux' | 'Unknown';

export function useOSDetection() {
  const [os, setOS] = useState<OS>('Unknown');
  const [isMounted, setIsMounted] = useState(false);

  useEffect(() => {
    setIsMounted(true);
    const userAgent = window.navigator.userAgent.toLowerCase();
    if (userAgent.includes('mac')) {
      setOS('macOS');
    } else if (userAgent.includes('win')) {
      setOS('Windows');
    } else if (userAgent.includes('linux')) {
      setOS('Linux');
    }
  }, []);

  return { os, isMounted };
}
```

- [ ] **Step 5: Run tests to pass**
Run: `cd website && npx vitest run`

- [ ] **Step 6: Commit**
```bash
git add website/src/hooks website/src/types
git commit -m "feat: add useOSDetection hook and updater types"
```

---

### Task 4: Hero Component with Dynamic Download Button

**Files:**
- Create: `website/src/components/Hero.tsx`
- Create: `website/src/components/Hero.test.tsx`

- [ ] **Step 1: Write a failing test for Hero**
```tsx
// website/src/components/Hero.test.tsx
import { render, screen } from '@testing-library/react';
import { Hero } from './Hero';
import { UpdaterInfo } from '../types/updater';

const mockUpdater: UpdaterInfo = {
  version: "0.3.7",
  notes: "Test",
  pub_date: "2026-05-20",
  platforms: {
    "darwin-aarch64": { url: "mac-url" },
    "windows-x86_64": { url: "win-url" },
    "linux-x86_64": { url: "linux-url" }
  }
};

describe('Hero', () => {
  it('renders version info', () => {
    render(<Hero updater={mockUpdater} />);
    expect(screen.getByText(/0\.3\.7/)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test (fails)**
Run: `cd website && npx vitest run`

- [ ] **Step 3: Implement Hero**
```tsx
// website/src/components/Hero.tsx
"use client";
import { useOSDetection } from '@/hooks/useOSDetection';
import { UpdaterInfo } from '@/types/updater';

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
    <section className="min-h-screen flex items-center justify-center pt-20 px-4">
      <div className="max-w-7xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
        <div>
          <h1 className="text-5xl lg:text-6xl font-bold mb-6">
            本地优先的个人 AI <br />
            <span className="text-gradient">智能知识库</span>
          </h1>
          <p className="text-xl text-[#94a3b8] mb-8">
            基于本地 LLM 驱动，100% 隐私安全，超强向量索引与多模态解析，打造您专属的本地第二大脑。
          </p>
          <div className="flex flex-col items-start gap-4">
            <a 
              href={downloadUrl}
              className="px-8 py-4 bg-gradient-to-r from-rose-500 via-pink-500 to-violet-600 rounded-lg text-white font-semibold text-lg hover:opacity-90 transition shadow-lg shadow-rose-500/20"
            >
              {buttonText}
            </a>
            <p className="text-sm text-[#94a3b8]">
              当前最新版本：v{updater.version} | 100% 离线隐私安全
            </p>
          </div>
        </div>
        <div className="relative animate-float">
          <div className="absolute -inset-1 bg-gradient-to-r from-rose-500 to-violet-600 rounded-xl blur opacity-30"></div>
          <div className="relative bg-[#0b0f19] border border-gray-800 rounded-xl aspect-video flex items-center justify-center text-gray-500">
            [App Screenshot Placeholder]
          </div>
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 4: Run tests to pass**
Run: `cd website && npx vitest run`

- [ ] **Step 5: Commit**
```bash
git add website/src/components
git commit -m "feat: implement Hero component with OS detection"
```

---

### Task 5: Assemble Homepage & Server-Side Fetch

**Files:**
- Modify: `website/src/app/page.tsx`
- Create: `website/src/app/page.test.tsx`

- [ ] **Step 1: Write a failing test for Page**
```tsx
// website/src/app/page.test.tsx
import { render, screen } from '@testing-library/react';
import Page from './page';

global.fetch = vi.fn(() =>
  Promise.resolve({
    ok: true,
    json: () => Promise.resolve({ version: "0.3.7", platforms: {} }),
  })
) as jest.Mock;

describe('Page', () => {
  it('renders Hero after fetching', async () => {
    const page = await Page();
    render(page);
    expect(await screen.findByText(/0\.3\.7/)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test (fails)**
Run: `cd website && npx vitest run`

- [ ] **Step 3: Implement Page**
```tsx
// website/src/app/page.tsx
import { Hero } from '@/components/Hero';
import { UpdaterInfo } from '@/types/updater';

const fallbackUpdater: UpdaterInfo = {
  version: "0.0.0",
  notes: "",
  pub_date: "",
  platforms: {}
};

export default async function Home() {
  let updater: UpdaterInfo = fallbackUpdater;
  
  try {
    const res = await fetch('https://download.telepathy-app.com/releases/updater.json', {
      next: { revalidate: 300 }
    });
    if (res.ok) {
      updater = await res.json();
    }
  } catch (e) {
    console.error("Failed to fetch updater.json", e);
  }

  return (
    <main>
      <Hero updater={updater} />
      {/* DownloadCenter and Features to be added in future iterations */}
    </main>
  );
}
```

- [ ] **Step 4: Run tests to pass**
Run: `cd website && npx vitest run`

- [ ] **Step 5: Commit**
```bash
git add website/src/app
git commit -m "feat: server side fetch of updater.json"
```
