# Phase 4：核心逻辑集成 (Core Integration)

## 背景

Phase 3 已完成了所有核心业务页面的 UI 还原与视图路由。现在的任务是打破“纯静态”的现状，将 `src/main.rs` 中的 Mock 数据替换为从 `crates/narya-*` 核心模块获取的真实数据，并实现基础的交互反馈。

## 本阶段目标

1. 对接 `narya-core` 的节点模型，实现 `Nodes` 列表的动态渲染。
2. 实现 `Subscriptions` 的加载与展示（基于 Mock 的 Core 接口）。
3. 实现简单的节点选择（Selected State）与侧边栏 Footer 同步。
4. 在 `main.rs` 中引入统一的数据管理（Model 层或 App State）。

## 具体任务

1. **核心模型对接**：
   - 在 `crates/narya-core` 中定义 `Node` 和 `Subscription` 结构体。
   - 在 `AppShell` 中引入 `nodes: Vec<Node>` 和 `subscriptions: Vec<Subscription>` 状态。
2. **Nodes 动态渲染**：
   - 将 `render_nodes_view` 修改为遍历 `self.nodes`。
   - 实现点击节点后的 `Selected` 状态切换。
3. **数据初始化**：
   - 在 `AppShell::new` 或通过 `cx.spawn` 初始化首屏数据。
4. **侧边栏同步**：
   - 当 `Active Node` 变化时，更新侧边栏 Footer 的连接名称。

## 约束

- 保持 UI 高还原度，数据对接不应破坏现有布局。
- 使用 GPUI 的 `update` 和 `notify` 机制处理状态变更。
- 确保模块化，不要在 `main.rs` 中编写过多的业务逻辑，应调用 `crates` 中的接口。

## 验证命令

- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run` (手动点击节点，观察侧边栏 Footer 状态同步更新)

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`
- 新增 `.prompt/006-phase-5-advanced-features.md`。

## Git 提交建议

`feat(logic): integrate narya-core models and implement dynamic state synchronization`
