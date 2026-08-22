# Phase 1：GPUI 基础 Design System

## 背景

Phase 0 已成功完成了 `narya-app` 的初始化和 GPUI 环境验证。现在我们需要进入 UI 构建的实质阶段。由于 GPUI 不提供现成的组件库，我们需要根据 Narya 的视觉真源（`ui/` 下的 PNG 和 `ui/specs/`）自研一套基础组件系统。

## 本阶段目标

1. 定义全局 Design Tokens (Colors, Spacing, Shadows, Typography)。
2. 实现基础的 `AppShell` 布局结构。
3. 封装第一批核心 UI 组件：`GlassCard` (毛玻璃卡片) 和 `Button`。

## 具体任务

1. **Tokens 定义**：在 `narya-app` 中建立 `theme` 模块，对照 `ui/specs/_global-design-system.spec.md` 定义颜色（冷白渐变、蓝紫主色、各种状态色）和圆角/阴影参数。
2. **字体配置**：确保 GPUI 正确加载 Narya 指定的字体（参考 specs）。
3. **GlassCard 实现**：封装一个支持半透明、模糊、细边框和柔和阴影的容器组件。
4. **基础 Button**：封装支持不同尺寸和状态（Primary, Secondary, Ghost）的按钮，并对照 PNG 还原视觉效果。
5. **布局骨架**：在 `main.rs` 中使用这些组件搭建一个包含左侧导航栏占位和中心内容区域的极简 AppShell。

## 约束

- 必须严格遵循 `ui/specs` 中的数值。
- 组件必须是可复用的。
- 保持 `cargo clippy` 无警告。

## 验证命令

- `cargo clippy --workspace`
- `cargo run -p narya-app` (观察布局是否符合初步设计稿比例)

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`
- 新增 `.prompt/003-phase-2-splash-dashboard.md`。

## Git 提交建议

`feat(ui): implement initial design tokens and glass card component`
