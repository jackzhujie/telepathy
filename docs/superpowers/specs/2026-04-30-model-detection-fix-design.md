# Design Doc: Model Detection Fix

## Problem Statement
The application fails to detect and display installed models correctly in the UI, even when `.gguf` files exist in the `downloads` directory. This is due to a synchronization bug in the frontend Pinia store and overly restrictive/brittle detection logic in the Rust backend.

## Proposed Changes

### 1. Frontend: Pinia Store (`src/stores/settings.ts`)
- **Change**: Refactor `models` from `ref<string[]>([])` to `computed(() => installedModels.value.map(m => m.full_name))`.
- **Change**: Remove `fetchModels` function.
- **Change**: Update `installNewModel` and `removeModel` to remove calls to `fetchModels`.
- **Rationale**: Ensures the UI is always in sync with the underlying `installedModels` data, eliminating the current initialization race condition.

### 2. Backend: Rust Command (`src-tauri/src/commands/models.rs`)
- **Change**: Update `list_installed_models` to use case-insensitive extension checking.
- **Change**: Refactor the directory entry loop to continue on individual entry errors instead of stopping.
- **Rationale**: Improves robustness against case variations and corrupted/locked files.

### 3. App Initialization (`src/App.vue`)
- **Change**: Ensure `fetchInstalledModels` is called on startup (already exists, but verification is needed).

## Verification Plan
1. **Manual Check**: Open the app and verify that models `qwen2.5-7b-instruct-q4_k_m.gguf` and `bge-m3-q4_k_m.gguf` (known to exist) are displayed in Settings.
2. **Case Sensitivity Check**: Rename a model to `.GGUF` and verify it is still detected.
3. **Log Check**: Verify that the backend logs the correct search path for models.
