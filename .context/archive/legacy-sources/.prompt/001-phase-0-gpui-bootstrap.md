# Phase 0：GPUI 项目骨架与基础视窗

## 背景

项目已做出重大战略调整：废弃原有基于 Tauri + Vue + Element Plus 的前端架构，转向使用 **GPUI**（Zed 原生 UI 框架）配合 Rust 进行纯原生桌面客户端开发。
旧有前端代码和 Tauri 配置已被完全清理，现急需搭建纯 Rust 的 GPUI 基础环境。

## 本阶段目标

1. 在 workspace 中创建并初始化 `narya-app`。
2. 引入 `gpui` 依赖。
3. 建立一个极简的 Hello World GPUI 窗口，验证渲染管道可用。

## 具体任务

1. 创建并更新 `crates/narya-app/Cargo.toml`，添加 `gpui` 等必要依赖。
2. 在 `crates/narya-app/src/main.rs` 中编写 GPUI 初始化代码。
3. 配置基础应用的 Window 属性（例如标题设为 "Narya"）。
4. 在屏幕上输出一句基础文本（例如 "Narya GPUI Client Started"）。
5. 检查整个 Rust workspace 是否能编译通过，并保证 `cargo run -p narya-app` 可以拉起窗口。

## 约束

- 请只实现最基础的 GPUI App 启动逻辑，不要在本阶段立刻尝试实现复杂的布局和样式。
- 保证无任何警告 (`cargo clippy`)。
- 虽然不需要复杂的 UI，但依然要阅读 `prompt.md` 熟悉整体项目的宏大视觉目标。

## 验证命令

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p narya-app`

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`
- 新增 `.prompt/002-phase-1-gpui-design-system.md`。

## Git 提交建议

```text
feat(core): initialize GPUI app skeleton

Summary:
- Setup `narya-app` crate with GPUI dependencies.
- Implement bare minimum GPUI app lifecycle and window creation.
- Ensure Rust workspace compiles cleanly.

Validation:
- ✅ cargo clippy
- ✅ cargo run -p narya-app

Memory:
- Updated status, tasks, changelog.
- Added phase 1 prompt.
```
