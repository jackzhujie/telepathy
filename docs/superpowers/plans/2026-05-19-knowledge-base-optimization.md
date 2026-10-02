# Knowledge Base UI Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the Knowledge Base view (`KnowledgeBase.vue`) to deliver a modern, premium double-column layout with visual extensions, colored icons, reader layout, and functional enhancements (e.g. copying text).

**Architecture:** Inject helper methods for file icon mappings, add local reactive states for copy success indicators, customize scroll containers and highlights, and rebuild the template styles using utility classes and Scoped CSS.

**Tech Stack:** Vue 3.5, Tailwind CSS, TypeScript

---

### Task 1: Add File Icon Helper and Copy Utility State

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: Write helper function and state for file icons & copying**
  We will add `getFileIcon` helper which returns specific Tailwind classes and SVG paths.
  We will also define `copiedChunkId` to store the ID of the chunk currently copied successfully.

  In `<script setup lang="ts">`, add:

  ```typescript
  // 记录刚刚成功复制的 Chunk ID，用于展示复制成功提示
  const copiedChunkId = ref<string | null>(null);

  async function copyChunkContent(chunkId: string, text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedChunkId.value = chunkId;
      setTimeout(() => {
        if (copiedChunkId.value === chunkId) copiedChunkId.value = null;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy text:', err);
    }
  }

  // 获取各种文件类型的彩色图标和样式
  const getFileIcon = (fileType: string) => {
    const ext = fileType.toLowerCase();
    switch (ext) {
      case 'pdf':
        return {
          bgClass: 'bg-red-500/10 text-red-500 border-red-500/20',
          path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6'
        };
      case 'md':
      case 'txt':
        return {
          bgClass: 'bg-indigo-500/10 text-indigo-500 border-indigo-500/20',
          path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M16 13H8 M16 17H8 M10 9H8'
        };
      case 'doc':
      case 'docx':
        return {
          bgClass: 'bg-blue-500/10 text-blue-500 border-blue-500/20',
          path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M16 12H8 M16 16H8'
        };
      case 'csv':
      case 'xlsx':
      case 'xls':
        return {
          bgClass: 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20',
          path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M8 13h2v2H8zm4 0h2v2h-2zm4 0h2v2h-2zm-8 4h2v2H8zm4 0h2v2h-2zm4 0h2v2h-2z'
        };
      case 'json':
        return {
          bgClass: 'bg-amber-500/10 text-amber-500 border-amber-500/20',
          path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M8 12h8 M8 16h8'
        };
      default:
        return {
          bgClass: 'bg-text-muted/10 text-text-muted border-text-muted/20',
          path: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6'
        };
    }
  };
  ```

---

### Task 2: Refactor Document Card Design and File Icons in Template

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: Replace document card visual styles and inject SVG icons**
  Locate the document list card rendering block in the template and update it to feature the new design:
  - Add visual left-accent bar for selected state.
  - Implement dynamic file icons using `getFileIcon`.
  - Update transition and hover classes.

---

### Task 3: Refactor Chunks Reader Details and Empty State

**Files:**
- Modify: `src/views/KnowledgeBase.vue`

- [ ] **Step 1: Design Chunk Reader Layout and Copy button**
  Locate the chunk lists card rendering block and update it:
  - Add Copy Text button with indicator state (`copiedChunkId === chunk.id`).
  - Upgrade font colors, styles, and card borders.
  - Design a premium "Empty State" placeholder when no document is selected.

- [ ] **Step 2: Add custom CSS highlights and masks**
  Add a `<style scoped>` section or update existing scoped CSS to style `<mark>` tag highlights cleanly.

---

### Task 4: Compilation and Verification

- [ ] **Step 1: Check build**
  Run: `npx vue-tsc --noEmit`
  Expected Output: Build runs clean with no TypeScript errors.

- [ ] **Step 2: Commit changes**
  ```bash
  git add src/views/KnowledgeBase.vue
  git commit -m "style: optimize knowledge base UI with colored file icons and premium reader layout"
  ```
