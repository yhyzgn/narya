# Phase 12：Liora UI 深度还原与旧页面迁移

## 背景

2026-06-21 已完成 Liora UI 重建基础切片：`narya-app` 直接进入主窗口，启动时初始化 Liora，主 AppShell 由 Liora 组件和 `ui_kit` 项目 wrapper 组合。旧 splash 已删除。核心/daemon/IPC/订阅解析逻辑保留，并已修复本轮 review blockers：连接动作接线、IPC response/error 处理、kernel install fail-closed、runtime path、config generator fail-closed。

## 必读

- `prompt.md`
- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/handoff.md`
- `docs/superpowers/plans/2026-06-21-liora-ui-rebuild.md`
- `ui/specs/main_window_spec_detailed.md`
- 具体页面对应 PNG：`ui/dashboard.png`、`ui/nodes.png`、`ui/subscriptions.png`、`ui/settings/*.png`

## 目标

1. 把当前 `ui_kit` 扩展为稳定的 Narya Design System：Sidebar、TopBar、StatusCard、MetricCard、NodeCard、SubscriptionCard、KernelPanel、Toolbar、Section 等。
2. 按 UI 图逐页校准：Dashboard、Nodes、Subscriptions、Settings 优先。
3. 将旧 raw GPUI 页面中的业务结构迁移到 Liora wrapper；迁完后删除旧页面模块。
4. 保持主窗口无 splash，所有新布局必须通过 Liora 或项目 wrapper。
5. 为真实交互逐步替代禁用按钮：kernel installer、订阅添加/剪贴板导入、YAML 编辑器、工具箱动作。
6. 设计并落地 length-prefixed/framed IPC codec，替代当前临时 JSON read 模型。

## 约束

- 不新增非必要依赖。
- 所有新 UI 组件必须低耦合：数据结构输入 + IntoElement 输出，不直接持有全局业务状态，交互 callback 显式传入。
- 不要回退到旧 raw GPUI 页面作为运行路径。
- GPUI/Liora API 必须以 Liora 0.1.5 源码/文档为准。
- 不允许 fake success：未实现的真实系统动作必须禁用或 fail-closed。

## 验证

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings
cargo clippy -p narya-app --lib -- -D warnings
timeout 8s cargo run -p narya-app
```

如需 UI 截图对比，在有显示服务的环境中运行 app 后按同尺寸截图，与 `ui/*.png` 手动或工具对照。
