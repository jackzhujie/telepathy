# Design Spec: SnapNote Editor Synchronization & Visibility Fix

## Goal
Fix "Zen Mode" and "Split Mode" issues by implementing full bi-directional synchronization between the raw Markdown textarea and the Milkdown WYSIWYG editor, and ensuring the editor is visible in all themes.

## Proposed Changes

### 1. MilkdownEditor.vue
- **Bidirectional Sync**:
  - Update `watch` on `props.modelValue` to call `editor.action(replaceAll(newValue))` whenever the external value changes.
  - Add a flag or check to prevent recursive updates (loops) when the editor itself emits a change.
- **Styling**:
  - Ensure `prose` classes are correctly applied for both light and dark modes.
  - Remove absolute positioning or overflows that might hide the content in Zen mode.

### 2. SnapNote.vue
- **Layout Balance**:
  - In `split` mode, ensure both panes share 50% width and handle scrolling independently but stay in sync via the shared `newSnapContent` ref.
- **Visibility Fix**:
  - Add `prose-invert` and appropriate background classes to ensure Milkdown text is visible against the dark theme.

## Architecture & Data Flow
- `newSnapContent` (ref in SnapNote.vue) -> `v-model` -> `MilkdownEditor`
- `newSnapContent` (ref in SnapNote.vue) -> `v-model` -> `textarea` (Split Mode)
- `MilkdownEditor` (listener) -> `emit('update:modelValue')` -> `newSnapContent`

## Verification Plan
1. **Zen Mode**: Open SnapNote, type in Milkdown, save. Ensure it's visible.
2. **Split Mode**: 
   - Type in left `textarea`, right `Milkdown` should update immediately.
   - Type in right `Milkdown`, left `textarea` should update immediately.
3. **Save**: Click "发布随记", both sides should clear.
