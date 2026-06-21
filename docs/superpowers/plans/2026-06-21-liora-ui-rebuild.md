# Liora UI Rebuild Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild Narya's GPUI desktop UI to launch directly into the main window and compose the app surface through Liora-backed project components while preserving existing core/daemon code.

**Architecture:** Keep `narya-core`, `narya-daemon`, IPC, subscription parsing, and kernel-related additions as the recoverable core. Replace legacy splash-first/raw-page UI with a Liora initialized main app shell plus a low-coupled `narya-app` component layer that wraps Liora controls and owns Narya-specific styling/data composition.

**Tech Stack:** Rust workspace, GPUI 0.2.2, Liora 0.1.5, existing Narya crates, compile-time/static regression tests plus cargo validation.

---

## File Structure

- Modify `Cargo.toml`: pin GPUI to registry `0.2.2` so Liora and Narya use one GPUI crate identity.
- Modify `crates/narya-app/Cargo.toml`: add `liora = { version = "0.1.5", default-features = false }`.
- Modify `crates/narya-app/src/lib.rs`: remove splash launch path, call `liora::init_liora(cx)`, open `AppShell` directly.
- Create/modify `crates/narya-app/src/ui_kit.rs`: Narya-specific, low-coupled wrappers around Liora layout/control components and fallback wrappers for missing design-specific primitives.
- Modify `crates/narya-app/src/views/app_shell.rs`: make main shell the canonical Liora-backed layout and route pages through Narya components.
- Keep existing page modules as data/content sources where useful, but prefer new project wrappers for all newly touched layout.
- Add `crates/narya-app/src/ui_contract_tests.rs`: static regression tests for no splash launch, Liora dependency/init, and component boundary rules.
- Update `.memory/*` and `.prompt/*` after validation.

## Task 1: Lock Liora integration contract

**Files:**
- Modify: `crates/narya-app/src/lib.rs`
- Modify: `crates/narya-app/Cargo.toml`
- Modify: `Cargo.toml`
- Add: `crates/narya-app/src/ui_contract_tests.rs`

- [ ] **Step 1: Write failing tests**
  - Test that `crates/narya-app/src/lib.rs` contains `liora::init_liora(cx)` and does not open `Splash`.
  - Test that app Cargo declares `liora`.
  - Test that a project component layer module exists.
- [ ] **Step 2: Run test and verify failure**
  - Run `cargo test -p narya-app ui_contract -- --nocapture`; expected failure before Liora integration.
- [ ] **Step 3: Implement minimal integration**
  - Add Liora dependency and app initialization.
  - Directly open `AppShell` as the startup window.
- [ ] **Step 4: Verify green**
  - Run `cargo test -p narya-app ui_contract -- --nocapture`.

## Task 2: Build Narya Liora component boundary

**Files:**
- Create: `crates/narya-app/src/ui_kit.rs`
- Modify: `crates/narya-app/src/lib.rs`

- [ ] **Step 1: Write/extend static contract tests**
  - Require `ui_kit.rs` to use `liora::components` and expose `NaryaCard`, `NaryaButton`, `NaryaMetric`, `NaryaPage` or equivalent helpers.
- [ ] **Step 2: Implement wrappers**
  - Wrap Liora `Card`, `Button`, `Tag`, `Progress`, `Space`, `Text`, and `Flex` where suitable.
  - Keep any raw GPUI `div()` use inside `ui_kit.rs` only for Narya-specific missing primitives.
- [ ] **Step 3: Verify**
  - Run `cargo test -p narya-app ui_contract -- --nocapture` and `cargo check -p narya-app`.

## Task 3: Replace startup app shell with Liora-backed layout

**Files:**
- Modify: `crates/narya-app/src/views/app_shell.rs`
- Keep: `crates/narya-app/src/state.rs`

- [ ] **Step 1: Implement shell**
  - 1366/1536 scale main window, sidebar, title/header, footer.
  - Navigation routes: Dashboard, Nodes, Subscriptions, Config, Connections, Rules, Logs, Tools, Settings.
  - Direct main window; no splash.
- [ ] **Step 2: Implement first-pass page surfaces**
  - Use cards/metrics/tags/progress/lists based on UI assets and main spec.
  - Use existing state data for nodes/subscriptions/kernels/logs.
- [ ] **Step 3: Verify**
  - Run `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`.

## Task 4: Documentation/memory handoff and validation

**Files:**
- Modify: `.memory/status.md`
- Modify: `.memory/tasks.md`
- Modify: `.memory/changelog.md`
- Modify: `.memory/handoff.md`
- Add: `.prompt/012-liora-ui-rebuild-followup.md`

- [ ] **Step 1: Record what changed and what remains**
- [ ] **Step 2: Run final validation commands**
- [ ] **Step 3: Commit and attempt push if validation passes**

## Self-Review

- Coverage: User constraints are represented: prompt/memory load, Liora integration, no splash, component boundary, preserve core, validation and memory update.
- Placeholder scan: No TODO/TBD placeholders in implementation tasks; remaining work is stated as explicit follow-up only after validation.
- Risk: Full pixel-perfect parity for every PNG may require later screenshot tooling; this plan delivers a compile-verified Liora-based product surface first, then records exact residual visual QA.
