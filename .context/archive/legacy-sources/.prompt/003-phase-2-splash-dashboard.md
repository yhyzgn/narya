# Phase 2：Splash 与 Dashboard 重建

## 背景

Phase 1 已成功建立了 Design Tokens、GlassCard 容器和 AppShell 布局框架。现在我们需要进入具体的页面还原阶段，将 `ui/splash.png` 和 `ui/dashboard.png` 中的视觉设计在 GPUI 中 1:1 实现。

## 本阶段目标

1. 在 GPUI 中还原 Splash 启动页面，包含动画效果。
2. 在 AppShell 中填充 Dashboard 的核心组件：Proxy Cards, Nodes List, Charts。
3. 实现 Splash 到 Dashboard 的窗口切换逻辑。

## 具体任务

1. **Splash 页面还原**：
   - 对照 `ui/specs/splash.spec.md`，实现居中的品牌 Logo、加载进度条和状态文本。
   - 使用 GPUI 的动画系统实现进度条的平滑增长。
2. **Dashboard 组件填充**：
   - 实现 `System Proxy` 和 `TUN` 的开关卡片（对照 PNG 中的蓝色/绿色开关视觉）。
   - 实现 `Quick Connect` 节点列表。
   - 为 `Network Overview` 预留图表渲染位置。
3. **窗口流转**：
   - 模拟启动过程，加载完成后自动从 Splash 窗口切换到 Dashboard 主窗口。

## 约束

- 必须实时查阅 Zed 官方仓库以获取最新的动画和布局 API 使用方法。
- 开关（Switch）和按钮（Button）必须根据 PNG 像素级还原，不使用默认样式。
- 保持 `cargo clippy` 无警告。

## 验证命令

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run` (手动观察 Splash 动画及跳转后 Dashboard 的布局还原度)

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`
- 新增 `.prompt/004-phase-3-business-pages.md`。

## Git 提交建议

`feat(ui): implement splash animation and dashboard core components`
