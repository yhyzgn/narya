# Phase 5：高级业务功能与交互 (Advanced Features)

## 背景

Phase 4 已完成了核心数据模型的注入与同步。现在的任务是开始实现具体的交互式业务逻辑，如实时的节点延迟测试（Latency Test）、流量网速动态更新，以及更复杂的 UI 组件（如弹窗详情）。

## 本阶段目标

1. 实现 Nodes 页面的“一键测速”逻辑，并异步更新 `AppState` 中的延迟数据。
2. 实现侧边栏 Footer 的实时流量网速显示（通过定时任务模拟核心推送）。
3. 开发通用的 `Modal` 弹出层基础组件。
4. 实现点击节点卡片弹出“节点详情”对话框。

## 具体任务

1. **实时测速逻辑**：
   - 在 `AppState` 中增加 `test_latency` 方法，使用 `cx.spawn` 模拟异步网络请求。
   - 更新 `render_nodes_view` 中的“Connect”按钮或增加测速图标触发逻辑。
2. **动态流量监测**：
   - 在 `AppShell` 启动后开启一个后台 timer，每秒更新一次 `AppState` 中的 download/upload 速度。
   - 确保侧边栏 Footer 能够平滑回显这些数值。
3. **Modal 组件开发**：
   - 在 `components.rs` 中设计一个覆盖全屏的半透明遮罩层及居中的内容容器。
   - 实现点击外部区域关闭 Modal 的交互。
4. **节点详情展示**：
   - 对照 `ui/nodes.png` 中的“节点详情”面板，在 Modal 中还原其布局。

## 约束

- 异步逻辑必须使用 `cx.spawn` 或 `cx.background_executor()`。
- 状态更新必须调用 `cx.notify()` 或 `model.update()`。
- 保持 `cargo clippy` 与 `cargo fmt` 全绿。

## 验证命令

- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run` (手动点击测速按钮，验证延迟数据是否在几秒后自动刷新)

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`
- 新增 `.prompt/007-phase-6-kernel-integration.md`。

## Git 提交建议

`feat(logic): implement real-time latency testing and dynamic traffic monitoring`
