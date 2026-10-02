# Model Detection Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the model detection issue where installed GGUF models are not displayed in the UI due to frontend state desync and case-sensitive backend checks.

**Architecture:** Refactor Pinia store to use computed properties for state consistency and improve Rust backend's file scanning robustness.

**Tech Stack:** Vue 3 (Pinia), Rust (Tauri v2)

---

### Task 1: Backend Robustness (Rust)

**Files:**
- Modify: `src-tauri/src/commands/models.rs`

- [ ] **Step 1: Update extension check to be case-insensitive**
Change line 135 to handle `.gguf` and `.GGUF` equally.

```rust
// In src-tauri/src/commands/models.rs
// Change:
if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
// To:
if path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("gguf")).unwrap_or(false) {
```

- [ ] **Step 2: Refactor directory reading loop to be more robust**
Modify the loop to log errors and continue instead of stopping on the first failure.

```rust
// In src-tauri/src/commands/models.rs
while let Some(entry_res) = entries.next_entry().await.transpose() {
    let entry = match entry_res {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[Models] Failed to read directory entry: {}", e);
            continue;
        }
    };
    let path = entry.path();
    // ... rest of the logic
}
```

- [ ] **Step 3: Verify backend compilation**
Run `cargo check` in `src-tauri`.

### Task 2: Frontend State Sync (Pinia)

**Files:**
- Modify: `src/stores/settings.ts`

- [ ] **Step 1: Convert `models` to a computed property**
Remove the `models` ref and replace it with a computed property based on `installedModels`.

```typescript
// In src/stores/settings.ts
// Remove:
// const models = ref<string[]>([]);
// Add:
const models = computed(() => installedModels.value.map(m => m.full_name));
```

- [ ] **Step 2: Remove `fetchModels` function and its calls**
Delete the `fetchModels` function and remove calls to it in `installNewModel` and `removeModel`.

```typescript
// In src/stores/settings.ts
// Remove fetchModels definition and cleanup return object
```

- [ ] **Step 3: Verify frontend type checking**
Run `pnpm tsc` in the root.

### Task 3: Final Verification

- [ ] **Step 1: Manual verification of model detection**
Launch the app and check if the existing models are now displayed in the Settings page without the "No models detected" warning.

- [ ] **Step 2: Verify case-insensitivity**
Rename one `.gguf` file to `.GGUF` manually and verify it still shows up.
