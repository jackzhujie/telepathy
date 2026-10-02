# Expand Document Import Extensions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand backend `scan_folder` extensions and frontend document import dialog filter & icons to support Excel (xlsx, xls) and PPT (pptx, ppt) files.

**Architecture:** Modify backend Rust commands and frontend Vue components directly to include new file extensions.

**Tech Stack:** Rust (Tauri), Vue 3, TypeScript

---

### Task 1: Backend Rust Commands Support

**Files:**
- Modify: `src-tauri/src/commands/document.rs`

- [x] **Step 1: Expand `supported_extensions` in `scan_folder` function**

Modify `src-tauri/src/commands/document.rs` around line 137, changing:
```rust
let supported_extensions = ["pdf", "md", "txt", "docx"];
```
to:
```rust
let supported_extensions = ["pdf", "md", "txt", "docx", "xlsx", "pptx", "xls", "ppt"];
```

- [x] **Step 2: Verify Rust code compiles**

Run `cargo check` in the `src-tauri` directory.
Expected: Build passes without errors.

---

### Task 2: Frontend Import Dialog Support

**Files:**
- Modify: `src/components/batch/BatchImportDialog.vue`

- [x] **Step 1: Expand file dialog filter extensions in `handleSelectFiles`**

Modify `src/components/batch/BatchImportDialog.vue` around line 87, changing:
```typescript
      extensions: ['pdf', 'md', 'txt', 'docx']
```
to:
```typescript
      extensions: ['pdf', 'md', 'txt', 'docx', 'xlsx', 'pptx', 'xls', 'ppt']
```

- [x] **Step 2: Update document format emojis in template**

Modify `src/components/batch/BatchImportDialog.vue` around line 413, changing:
```vue
                  {{ item.name.endsWith('.pdf') ? '📕' : item.name.endsWith('.md') ? '📝' : item.name.endsWith('.docx') ? '📘' : '📄' }}
```
to:
```vue
                  {{ item.name.endsWith('.pdf') ? '📕' : item.name.endsWith('.md') ? '📝' : ['.docx', '.doc'].some(ext => item.name.endsWith(ext)) ? '📘' : ['.xlsx', '.xls'].some(ext => item.name.endsWith(ext)) ? '📊' : ['.pptx', '.ppt'].some(ext => item.name.endsWith(ext)) ? '📈' : '📄' }}
```

- [x] **Step 3: Verify Frontend code compiles**

Run frontend build check using `tsc --noEmit` or equivalent command.
Expected: Build passes without TypeScript errors.

---

### Task 3: Commit and Verify

**Files:**
- Modify: None (git commit and verify task)

- [x] **Step 1: Add and commit the changes**

Run:
```bash
git add src-tauri/src/commands/document.rs src/components/batch/BatchImportDialog.vue
git commit -m "feat: expand import format filter to support xlsx, xls, pptx, and ppt in frontend and scan_folder cmd"
```
Expected: Files successfully committed to git.
