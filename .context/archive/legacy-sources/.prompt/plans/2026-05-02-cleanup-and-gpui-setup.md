# Cleanup and GPUI Setup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transition the `narya` project from Tauri/Vue to GPUI by updating the workspace `Cargo.toml`.

**Architecture:** A clean workspace `Cargo.toml` that removes Tauri-related members and dependencies, replacing them with GPUI and other necessary crates.

**Tech Stack:** Rust, GPUI.

---

### Task 1: Update workspace Cargo.toml

**Files:**
- Modify: `narya/Cargo.toml`

- [x] **Step 1: Replace content of Cargo.toml**

- [x] **Step 2: Verify the change**

### Task 2: Verify Dependencies

- [x] **Step 1: Run cargo fetch**

### Task 3: Commit Changes

- [x] **Step 1: Commit the cleanup**
