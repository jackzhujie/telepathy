# 设计规格：知识库页面视觉与排版美化优化

## 1. 优化背景与目标 (Background & Objective)
当前的知识库页面（`KnowledgeBase.vue`）在左右双栏布局上是合理且高效的，但其视觉呈现和细节排版较为简陋（如卡片没有醒目的高亮指引、文件类型仅为纯文字展示、文本切片排版缺少阅读舒适感）。
本优化的目标是：通过**彩色文件图标系统**、**选中态/悬浮态亚克力（Glassmorphism）高亮微动效**、以及**“沉浸式阅读器”排版和一键复制文本块功能**，显著提高页面的视觉高级感与使用便利性。

## 2. 详细设计规范 (Design Specification)

### 2.1 左侧：文档列表卡片与图标系统
- **文件分类彩色 SVG 图标**：
  为常见的文件扩展名分发高颜值的 SVG 徽标图标：
  - `.pdf` -> 红色文件图标 (代表 PDF 文档)
  - `.md` / `.txt` -> 靛青色/紫色代码文档图标 (代表纯文本或轻量标记语言)
  - `.docx` / `.doc` -> 蓝色文字文档图标 (代表 Word 文档)
  - `.csv` / `.xlsx` -> 绿色表格文档图标 (代表电子表格)
  - 其他类型 -> 灰色通用文档图标
- **选中态 (Selected State)**：
  - 卡片左侧添加一个 `3px` 宽的垂直渐变色条 (`bg-gradient-to-b from-brand to-brand-600`)。
  - 背景升级为微透亮的高亮背景 (`bg-brand/5 backdrop-blur-sm border-brand/50`)。
  - 卡片字体加粗且阴影加深。
- **悬浮态 (Hover State)**：
  - 微量上移 `hover:-translate-y-[1px]`。
  - 阴影过渡 `hover:shadow-md` 且边框变深。

### 2.2 右侧：文本切片沉浸式阅读器与空状态
- **空状态占位 (Empty State)**：
  未选择文档时，显示带有柔和极光发光背景（`bg-brand/5` 与圆角外发光）的卡片，包含一个检索主题的 SVG 动画和友好引导文字。
- **切片卡片 (Chunk Card) 优化**：
  - **精细化行距与字体**：行高设为舒服的 `leading-relaxed` (1.625)，字色调整为 `text-text-secondary`。
  - **复制文本功能**：卡片头部右侧增加一个“一键复制”图标按钮，点击后调用剪贴板 API，并提供 `2秒` 的“已复制 ✅”状态反馈。
  - **文档名面包屑**：在切片顶部不仅展示当前文档名，同时采用淡色调。
- **搜索高亮匹配**：
  将原本粗暴的 `<mark>` 高亮改造成圆角半透明的高级质感：
  ```css
  mark {
    background-color: rgba(var(--color-brand-rgb), 0.15);
    color: var(--color-brand);
    font-weight: 600;
    border-radius: 4px;
    padding: 1px 3px;
  }
  ```

## 3. 代码结构与变更计划 (Implementation Plan)

### 3.1 辅助函数：根据文件扩展名返回 SVG 图标和颜色
在 `KnowledgeBase.vue` 的 `<script>` 区域中增加一个返回相应类型图标的组件或函数：
```typescript
const getFileIcon = (fileType: string) => {
  const type = fileType.toLowerCase();
  if (type === 'pdf') {
    return {
      color: 'text-red-500 bg-red-500/10 border-red-500/20',
      iconPath: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6' // 基础文档框架
    };
  }
  // 其他扩展类型如 md, txt, docx 等同理配置
};
```

### 3.2 切片复制状态控制
新增一个记录复制成功的 ID 状态 `const copiedChunkId = ref<string | null>(null)`，实现 `copyChunkContent(id, text)`：
```typescript
async function copyChunkContent(chunkId: string, text: string) {
  try {
    await navigator.clipboard.writeText(text);
    copiedChunkId.value = chunkId;
    setTimeout(() => {
      if (copiedChunkId.value === chunkId) copiedChunkId.value = null;
    }, 2000);
  } catch (err) {
    console.error('Failed to copy:', err);
  }
}
```

### 3.3 样式微调
- 为高亮 `<mark>` 样式定义对应的 Scoped CSS，使其展现精美。
