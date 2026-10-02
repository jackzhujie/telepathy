# Vision Toggle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a toggle in ChatInput to completely disable image upload and paste capabilities, ensuring slow vision models are bypassed when the user only wants to chat with text.

**Architecture:** We will introduce a local `ref(true)` named `visionMode` in `ChatInput.vue`. We will conditionally render the image upload button, watch the ref to clear existing images when disabled, and intercept paste events.

**Tech Stack:** Vue 3.5, TypeScript

---

### Task 1: Add Vision Toggle State and Modify Paste Logic

**Files:**
- Modify: `src/components/chat/ChatInput.vue`

- [ ] **Step 1: Add state, watcher, and intercept paste events**
  We will add `visionMode` ref, watch it to clear images, and update `handlePaste`.

  Locate `<script setup lang="ts">` in `src/components/chat/ChatInput.vue` and inject the `visionMode` state.
  Modify `handlePaste` to return early if `visionMode` is false.

- [ ] **Step 2: Add Toggle UI and conditional rendering**
  In the `<template>` section of `ChatInput.vue`, add the vision toggle button next to the image upload button, and conditionally render the image upload button using `v-if="visionMode"`.

- [ ] **Step 3: Run TypeScript compiler check**
  Run `npx vue-tsc --noEmit` to ensure type correctness.
  Expected: PASS with no errors.

- [ ] **Step 4: Commit changes**
  ```bash
  git add src/components/chat/ChatInput.vue
  git commit -m "feat: add vision mode toggle to disable image input"
  ```
