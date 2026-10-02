# Email Notification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add email notification to the GitHub Actions release workflow.

**Architecture:** Modify `.github/workflows/release.yml` to use `dawidd6/action-send-mail@v3` in the `notify` job.

**Tech Stack:** GitHub Actions

---

### Task 1: Update Workflow File

**Files:**
- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: Add email send step**

Replace the existing `notify` job steps with the email sending step.

- [ ] **Step 2: Commit change**

```bash
git add .github/workflows/release.yml
git commit -m "feat: add email notification to release workflow"
```

- [ ] **Step 3: Push change**

```bash
git push origin main
```
