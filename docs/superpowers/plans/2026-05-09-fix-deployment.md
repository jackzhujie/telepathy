# Fix Release Deployment Issues Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the `mv: cannot stat` error during SFTP upload and address the missing signature files issue.

**Architecture:** 
1. Fix race condition in `publish-to-server.cjs` by using unique remote temporary filenames.
2. Remove redundant signature copying in `release.yml`.

**Tech Stack:** Node.js, GitHub Actions

---

### Task 1: Fix Race Condition in `publish-to-server.cjs`

**Files:**
- Modify: `scripts/publish-to-server.cjs`

- [ ] **Step 1: Use unique temp filenames**

Update the `sftp.fastPut` logic to append a unique ID to the `/tmp/` filename, and update the `moveCmds` to move from that unique temp name to the final destination name.

### Task 2: Clean up `release.yml`

**Files:**
- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: Remove custom `.sig` copy steps**

Remove the `Copy signature files (macOS/Linux)` and `Copy signature files (Windows)` steps, as `publish-to-server.cjs` already recursively searches for `.sig` files.
