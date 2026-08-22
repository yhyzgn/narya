# Phase 3：业务页面迁移 (Nodes & Subscriptions)

## 背景

Phase 2 已成功还原了 Splash 和 Dashboard 的核心 UI。现在的任务是开始迁移业务重心的页面：节点列表（Nodes）和订阅管理（Subscriptions）。由于这些页面包含大量列表和表格，我们需要在 GPUI 中探索高效的列表渲染和交互。

## 本阶段目标

1. 在 `AppShell` 的 `Content` 区域实现视图切换（路由）逻辑。
2. 实现 `Nodes` 节点列表页面（对照 `ui/nodes.png`）。
3. 实现 `Subscriptions` 订阅管理页面（对照 `ui/subscriptions.png`）。
4. 封装基础的 `Table` 或 `List` 骨架组件以供复用。

## 具体任务

1. **视图路由切换**：
   - 在 `AppShell` 中维护一个 `active_view` 状态。
   - 点击侧边栏导航项（Dashboard, Nodes, Subscriptions）时，更新该状态并重新渲染内容区。
2. **Nodes 页面实现**：
   - 对照 `ui/specs/nodes.spec.md` 实现。
   - 包含顶部的搜索/过滤工具栏。
   - 列表项包含节点名称、协议标签、延迟值（Badge）和操作按钮。
3. **Subscriptions 页面实现**：
   - 对照 `ui/specs/subscriptions.spec.md` 实现。
   - 展示订阅卡片，包含流量使用进度条、过期时间和更新按钮。
4. **组件封装**：
   - 封装通用的 `Badge`（用于延迟/协议）。
   - 封装 `SearchInput`（对照 UI 中的细蓝边框输入框）。

## 约束

- 列表渲染优先考虑性能，虽然目前数据量小，但应遵循 GPUI 的 `List` 最佳实践。
- 交互反馈（Hover 状态、点击效果）必须对照 PNG 还原。
- 保持 `cargo clippy` 全绿。

## 验证命令

- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run` (手动点击侧边栏切换视图，检查 Nodes 和 Subscriptions 的布局还原度)

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`
- 新增 `.prompt/005-phase-4-core-logic-integration.md`。

## Git 提交建议

`feat(ui): implement Nodes and Subscriptions views with view routing`
