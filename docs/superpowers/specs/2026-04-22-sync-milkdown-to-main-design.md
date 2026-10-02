# Design Spec: Sync SnapNote Milkdown Integration to Main

## Goal
Synchronize the completed Milkdown editor integration and associated stability fixes from the `.worktrees/snapnote-milkdown` worktree to the main project branch.

## Scope of Changes
The synchronization will cover the following areas:

### 1. Editor Migration
- **Remove**: `md-editor-v3` dependency.
- **Add**: Milkdown core and its plugins (`gfm`, `history`, `listener`, `prism`, `slash`, `theme-nord`, `utils`, `vue`).
- **New Component**: `src/components/editor/MilkdownEditor.vue`.
- **Modified View**: `src/views/SnapNote.vue` updated to use the new WYSIWYG editor.

### 2. Stability & Build Fixes
- **ModelManager.vue**: Fixed missing closing tags and removed unused variables causing build failures.
- **Chat.vue / virtualizer**: Refactored ref handling for better type safety and performance.
- **Store cleanups**: Removed unused imports and variables in `settings.ts`, `documents.ts`, etc.
- **General UI**: Minor styling improvements and consistency fixes across views.

## Implementation Approach
We will use a direct file synchronization approach to ensure the main branch exactly matches the validated state in the worktree:
1. **Sync Files**: Copy all modified and new files from the worktree to the root project.
2. **Update Dependencies**: Run `pnpm install` to update `pnpm-lock.yaml` and install the new Milkdown packages.
3. **Verification**: Run `pnpm build` to confirm the project still builds correctly after the sync.

## User Review Required
> [!IMPORTANT]
> This will overwrite several files in your main project with the versions from the worktree. Since the worktree has been verified to build correctly, this is the safest way to ensure a stable "real-time" editor experience.
