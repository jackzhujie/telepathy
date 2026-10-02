# 宣传网站三态循环主题切换设计文档

本项目为 Telepathy 宣传官方网站增加深浅色主题切换功能，默认跟随系统，并支持手动切换为“浅色模式”或“深色模式”。

## 1. 用户需求与目标

1. **主题默认跟随系统**：初次访问时，如果系统处于深色模式则展示深色，处于浅色模式则展示浅色。
2. **三态循环手动切换**：用户可以通过点击顶部导航栏的按钮，在“系统默认 (System) ➔ 浅色模式 (Light) ➔ 深深模式 (Dark)”三个状态之间进行循环切换，并持久化存储用户的选择。
3. **完美无闪烁体验**：避免在 Next.js 服务端渲染 (SSR) 页面初次加载时出现颜色瞬间跳变的“主题闪烁”现象。

## 2. 核心技术选型

* **主题框架**：`next-themes`。其能够通过自动注入防闪烁 Script 的方式，优雅解决 App Router 下的 SSR 主题闪烁问题。
* **样式适配**：Tailwind CSS v4 配合 CSS 变量。我们将在 `globals.css` 中根据 `:root` 和 `.dark` 定义不同的颜色变量，而在 Tailwind 样式中直接使用这些变量。

## 3. 设计细则

### 3.1. 样式与颜色系统设计 (`website/src/app/globals.css`)
需要将当前页面中硬编码的深色主题背景和文字提取为 CSS 主题变量。

| 变量名称 | 浅色模式 (`:root`) | 深色模式 (`.dark`) | 用途描述 |
| :--- | :--- | :--- | :--- |
| `--color-background` | `#f8fafc` | `#070a13` | 页面背景色 |
| `--color-foreground` | `#0f172a` | `#f8fafc` | 主文本颜色 |
| `--color-card-bg` | `rgba(255, 255, 255, 0.7)` | `rgba(15, 23, 42, 0.45)` | 卡片背板填充 |
| `--color-card-border` | `rgba(0, 0, 0, 0.06)` | `rgba(255, 255, 255, 0.05)` | 卡片玻璃边框色 |
| `--radial-gradient-1` | `rgba(244, 63, 94, 0.02)` | `rgba(244, 63, 94, 0.05)` | 左上角玫瑰红径向背景光晕 |
| `--radial-gradient-2` | `rgba(139, 92, 246, 0.02)` | `rgba(139, 92, 246, 0.05)` | 右下角紫罗兰径向背景光晕 |

同时，在 `@theme` 配置中映射这些变量：
```css
@theme {
  --color-background: var(--color-background);
  --color-foreground: var(--color-foreground);
  --color-card-bg: var(--color-card-bg);
  --color-card-border: var(--color-card-border);
}
```

### 3.2. 主题 Provider (`website/src/components/ThemeProvider.tsx`)
为避免水合警告与非客户端状态异常，封装 `ThemeProvider`：
```typescript
"use client";
import { ThemeProvider as NextThemeProvider } from "next-themes";
import { ReactNode } from "react";

export function ThemeProvider({ children }: { children: ReactNode }) {
  return (
    <NextThemeProvider attribute="class" defaultTheme="system" enableSystem>
      {children}
    </NextThemeProvider>
  );
}
```

### 3.3. 主题切换组件 (`website/src/components/ThemeToggle.tsx`)
* **循环切换逻辑**：
  * 读取当前主题（通过 `useTheme()` 钩子的 `theme` 属性）。
  * 循环数组：`['system', 'light', 'dark']`。
  * 当用户点击时，计算 `nextTheme`，调用 `setTheme(nextTheme)`。
* **图标与无障碍 (A11y) 设计**：
  * `system`：电脑屏幕图标，带 `aria-label="系统主题"`。
  * `light`：太阳图标，带 `aria-label="浅色主题"`。
  * `dark`：月亮图标，带 `aria-label="深色主题"`。
* **防止 Hydration Mismatch**：
  * 声明 `mounted` 状态，仅在 `useEffect` 执行后（组件已挂载）才渲染图标按钮；挂载前渲染一个同等大小的占位容器，避免服务端和客户端渲染不匹配。

### 3.4. 导航栏集成 (`website/src/components/Navbar.tsx`)
* 引入 `<ThemeToggle />`。
* 将其插在右侧“立即下载”按钮的左侧，通过 Flex 布局和适当间距保持整体一致性。

## 4. 自动化测试设计

我们将为新逻辑编写以下单元测试：
1. **`ThemeToggle` 渲染测试**：验证挂载后能够正常显示，并在点击时触发 `setTheme` 循环调用。
2. **`ThemeProvider` 渲染测试**：验证能否正确透传子组件。
3. **集成验证**：验证在 Navbar 中能成功渲染切换按钮。

## 5. 验收标准
1. **系统跟随**：首次访问且无 localStorage 时，页面模式与系统的 dark-mode 设置保持一致。
2. **手动切换**：点击按钮，依次显示为：`系统偏好` -> `浅色` -> `深色`，每次切换都能立刻改变页面背景颜色。
3. **防闪烁**：刷新页面时，浅色模式下不能有黑色闪烁，深色模式下不能有白色闪烁。
4. **无错误**：通过 `tsc` 检查与测试，打包无任何错误警告。
