# 按钮悬浮状态主题化设计

## 1. 现状与痛点

在当前 Telepathy 项目中，许多按钮组件的悬浮（hover）、激活（active）状态颜色是硬编码在 Tailwind 样式或 `style.css` 中的：
- `style.css` 内预定义的 `.btn-brand`、`.btn-danger`、`.btn-success` 使用了 `hover:bg-brand-600`、`hover:bg-danger-600` 等硬编码色阶。
- Vue 文件中许多按钮使用了类似于 `hover:bg-brand-600` 等静态 Tailwind 颜色。

这些硬编码会导致：
1. **主题不自适应**：在亮色与暗色模式下，或未来更换应用主品牌色（`--brand-rgb`）时，硬编码的悬浮颜色无法自动调整，导致视觉上的不一致。
2. **重复代码多**：每个按钮都需要手动编写长串的 hover/active 颜色样式。

---

## 2. 设计目标

通过**“带透明度的品牌色（方案 A）”**与**“全局抽象基础类（方案 C）”**结合的方式，实现悬浮/激活状态的自动化、主题感知化，达到：
- **一致性**：所有按钮在不同主题下均具备统一、和谐的悬浮与点击回馈。
- **高可维护性**：主品牌色、全局背景等变更时，交互状态自动调整。
- **开发高效性**：提供原子类和全局组件类，简化样式编写。

---

## 3. 详细设计方案

我们将从两方面入手：**全局基础按钮样式抽象**，以及**样式文件中现有硬编码规则的重构**。

### 3.1 改造 `src/style.css` 中的预定义按钮类

将 `style.css` 中的所有基础按钮类，全部使用带有 Alpha 通道的颜色来实现 `hover` 和 `active` 状态的亮暗变化（利用 `bg-color/alpha`）：

```css
/* 修改后 */
.btn-brand {
  @apply px-4 py-2 bg-brand text-white font-medium rounded-lg
         hover:bg-brand/90 active:bg-brand/80
         focus:outline-none focus:ring-2 focus:ring-brand/50
         transition-all duration-200
         flex items-center justify-center gap-2;
}

.btn-ghost {
  @apply px-4 py-2 text-text-secondary font-medium rounded-lg
         hover:bg-surface-bg hover:text-text-primary
         active:bg-hover-bg/60
         focus:outline-none focus:ring-2 focus:ring-border-main
         transition-all duration-200
         flex items-center justify-center gap-2;
}

.btn-danger {
  @apply px-4 py-2 bg-danger text-white font-medium rounded-lg
         hover:bg-danger/90 active:bg-danger/80
         focus:outline-none focus:ring-2 focus:ring-danger/50
         transition-all duration-200
         flex items-center justify-center gap-2;
}

.btn-success {
  @apply px-4 py-2 bg-success text-white font-medium rounded-lg
         hover:bg-success/90 active:bg-success/80
         focus:outline-none focus:ring-2 focus:ring-success/50
         transition-all duration-200
         flex items-center justify-center gap-2;
}
```

### 3.2 针对现有 Vue 组件中硬编码颜色的替换

对于零散在 Vue 模板中的按钮样式，我们将批量排查替换：
1. `hover:bg-brand-600` -> `hover:bg-brand/90`
2. `hover:bg-danger-600` -> `hover:bg-danger/90`
3. `hover:bg-success-600` -> `hover:bg-success/90`
4. 带有渐变的按钮：`from-brand to-brand-600` 在 hover 时，可以使用透明度降低的不透明度阴影：`hover:shadow-brand/20`，或者为其附加 `hover:brightness-105` 或 `hover:bg-brand/95`，来实现具有主题适应性的回馈。

---

## 4. 验证与回归测试

1. **样式一致性**：确认修改后所有按钮的悬浮与点击状态在**亮色模式**与**暗色模式**下视觉回馈均表现良好，且不影响其原本的字号、圆角、内边距等。
2. **前端打包与编译测试**：运行 `npm run dev` 和 `pnpm tauri build`（如适用），确保 Tailwind 样式没有因为 Alpha 语法导致编译报错或样式丢失。
