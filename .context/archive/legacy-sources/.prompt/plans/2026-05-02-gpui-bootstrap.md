# GPUI Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Initialize the `narya-app` crate and set up a basic GPUI "Hello World" window.

**Architecture:** Transition from Tauri/Vue to a pure Rust GPUI native application. The `narya-app` will serve as the main entry point for the desktop client.

**Tech Stack:** Rust, GPUI (from zed-industries/zed git).

---

### Task 1: Workspace Cleanup and GPUI Dependency Setup

**Files:**
- Modify: `narya/Cargo.toml`

- [ ] **Step 1: Remove Tauri dependencies and add GPUI to workspace**

```toml
[workspace]
resolver = "2"
members = [
    "crates/*",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Neo"]
license = "MIT"
repository = "https://github.com/yhyzgn/Narya"

[workspace.dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
anyhow = "1"
```

- [ ] **Step 2: Commit workspace configuration**

```bash
git add narya/Cargo.toml
git commit -m "chore: cleanup workspace and add gpui dependency"
```

---

### Task 2: Initialize narya-app Crate

**Files:**
- Create: `narya/crates/narya-app/Cargo.toml`
- Create: `narya/crates/narya-app/src/main.rs`

- [ ] **Step 1: Create narya-app Cargo.toml**

```toml
[package]
name = "narya-app"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
gpui.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
anyhow.workspace = true
narya-core = { path = "../narya-core" }
narya-config = { path = "../narya-config" }
```

- [ ] **Step 2: Create basic GPUI Main entry point**

```rust
use gpui::*;

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x2e7d32))
            .size_full()
            .items_center()
            .justify_center()
            .text_color(rgb(0xffffff))
            .text_xl()
            .child(format!("Hello, {}!", self.text))
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| HelloWorld {
                text: "Narya GPUI Client Started".into(),
            })
        });
    });
}
```

- [ ] **Step 3: Run cargo fmt and clippy**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings`
Expected: Success with no warnings.

- [ ] **Step 4: Verify build and run**

Run: `cargo run -p narya-app`
Expected: A window appears with "Hello, Narya GPUI Client Started!".

- [ ] **Step 5: Commit narya-app initialization**

```bash
git add narya/crates/narya-app
git commit -m "feat(app): initialize narya-app with GPUI hello world"
```

---

### Task 3: Update Project Memory

**Files:**
- Modify: `narya/.memory/status.md`
- Modify: `narya/.memory/tasks.md`
- Modify: `narya/.memory/changelog.md`
- Create: `narya/.prompt/002-phase-1-gpui-design-system.md`

- [ ] **Step 1: Update status.md**
Update "已完成" and "当前阶段".

- [ ] **Step 2: Update tasks.md**
Mark Phase 0 tasks as completed.

- [ ] **Step 3: Update changelog.md**
Add entry for GPUI bootstrap.

- [ ] **Step 4: Create next phase prompt**
Document the requirements for Phase 1 (Design System).

- [ ] **Step 5: Commit memory updates**

```bash
git add narya/.memory narya/.prompt
git commit -m "docs: update memory and tasks for Phase 0 completion"
```
