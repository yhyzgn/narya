# Narya Phase 3 Stage 1: GUI 主框架实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 基于 GPUI 构建 Narya 的桌面端主架构，包含侧边导航栏、顶部状态栏及内容区域切换。

**Architecture:** 采用 MVVM 模式。`Workspace` 作为根视图，管理导航状态；各功能模块（Home, Proxies 等）作为独立的 `View`。

**Tech Stack:** Rust, GPUI.

---

### Task 1: 初始化 UI Crate 环境

**Files:**
- Modify: `narya-ui/Cargo.toml`
- Create: `narya-ui/src/lib.rs`

- [ ] **Step 1: 配置 `narya-ui/Cargo.toml`**

```toml
[package]
name = "narya-ui"
version = "0.1.0"
edition = "2024"

[dependencies]
gpui = { workspace = true }
api = { path = "../api" }
anyhow = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
```

- [ ] **Step 2: 编写基础启动代码 `narya-ui/src/lib.rs`**

```rust
use gpui::*;

pub struct Workspace {
    selected_tab: usize,
}

impl Workspace {
    pub fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self { selected_tab: 0 }
    }
}

impl Render for Workspace {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .full()
            .flex()
            .bg(rgb(0x1e1e1e)) // 背景色
            .child(
                div()
                    .w_64()
                    .bg(rgb(0x252526)) // 侧边栏
                    .child("Sidebar")
            )
            .child(
                div()
                    .flex_1()
                    .child("Main Content")
            )
    }
}

pub fn run_app(app: App) {
    app.run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|cx| Workspace::new(cx))
        });
    });
}
```

---

### Task 2: 构建侧边导航栏 (Sidebar)

**Files:**
- Create: `narya-ui/src/components/sidebar.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 实现侧边栏组件**
包含图标、名称和点击切换状态。

- [ ] **Step 2: 集成到 Workspace**
实现点击侧边栏更新 `selected_tab` 状态并重新渲染。

---

### Task 3: 适配主程序入口

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 修改 `main.rs` 以支持 GUI 启动模式**
当检测到非 headless 模式时，启动 GPUI 事件循环。

---

### Task 4: 编译与冒烟测试

- [ ] **Step 1: 运行编译**
`cargo build -p narya-ui`

- [ ] **Step 2: 启动模拟**
确保窗口能正常弹出并显示基础布局。
