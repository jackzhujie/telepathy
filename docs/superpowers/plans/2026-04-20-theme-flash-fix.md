# 修复主题闪烁问题 (Theme Flash Fix)

## 问题描述
应用启动时会出现短暂的白色背景闪烁，然后才切换到系统深色主题。这是因为主题检测逻辑在 Vue 应用挂载后（onMounted）才执行，此时浏览器已经渲染了默认的浅色背景。

## 解决方案
在 `index.html` 的 `<head>` 中添加一个阻塞式的同步脚本。该脚本会在浏览器开始渲染任何内容之前立即检测系统主题（或本地缓存的主题）并应用 `dark` 类。同时在 Pinia store 中同步更新缓存。

## 提议的更改

### [MODIFY] [index.html](file:///Users/mac/project/telepathy/index.html)
在 `<head>` 中添加内联脚本：
- 检查 `localStorage` 中的 `theme` 设置。
- 如果没有设置，或者设置为 `system`，则使用 `window.matchMedia` 检测系统偏好。
- 立即在 `document.documentElement` 上添加或删除 `dark` 类。

### [MODIFY] [settings.ts](file:///Users/mac/project/telepathy/src/stores/settings.ts)
- 在 `setTheme` 方法中，将选择的主题同步保存到 `localStorage`。
- 在 `fetchSettings` 方法中，从后台加载设置后，同步更新 `localStorage` 以确保下次启动时使用的是最新的用户偏好。

## 验证计划

### 手动验证
1. 彻底关闭应用。
2. 将系统主题设置为深色模式。
3. 启动应用，检查是否还存在白色闪烁。
4. 在应用内切换主题为浅色，重启应用，检查是否立即显示浅色。
5. 在应用内切换主题为系统默认，重启应用并更改系统主题，验证启动时的表现。

### 自动化测试
由于涉及启动时的渲染，主要依靠手动视觉检查。我们可以通过简单的日志打印来验证脚本执行的时机。
