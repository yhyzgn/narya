# Phase 12：Liora UI 像素级校准与本地组件库拆分

## 背景

2026-06-21 已完成 Liora UI 第二轮红线重做：旧 spec 相关非图片文件已删除；页面层 `views/app_shell.rs` 不再直接写原生 GPUI 布局/样式；必要 GPUI 底层能力集中在 `ui_kit.rs` 本地组件库边界。当前 UI 已按 1536×1024 图片真源重建主骨架，但尚未做截图级 1:1 diff。

## 必读

- `prompt.md`
- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/handoff.md`
- 图片真源：`ui/dashboard.png`、`ui/nodes.png`、`ui/subscriptions.png`、`ui/settings.png` 以及对应子目录 PNG

## 红线

- 不得恢复或依赖 `ui` 下旧 spec 非图片文件。
- 页面/业务 UI 代码不得出现原生 GPUI 布局/样式 token。
- 页面层只能组合 Liora 控件和本地 `narya_ui` 组件。
- Liora 不足时，只能在本地组件库边界封装低耦合组件。
- 不能伪造 1:1；必须用截图对照图片后再声明视觉验收。

## 目标

1. 运行 app 截图，对照 `ui/dashboard.png` 做首屏像素级校准。
2. 依次校准 `nodes.png`、`subscriptions.png`、`settings.png`。
3. 将 `ui_kit.rs` 拆成小模块，保留公共导出，降低耦合。
4. 记录可反哺 Liora 的组件：ShellFrame、Sidebar、MetricCard、NodeCard、SubscriptionItem、SettingsPanel、chart wrappers。

## 验证

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings
cargo clippy -p narya-app --lib -- -D warnings
timeout 8s cargo run -p narya-app
```

## 2026-06-21 菜单与 Dashboard 纠偏

- 左侧菜单从 Button 拼接改为按图的全宽菜单行，使用 Liora Lucide 图标，不再用文字符号冒充菜单图标。
- active 菜单改为浅蓝背景 + 蓝色图标/文字，更接近效果图。
- 左栏品牌区改用 `ui/icons/narya-logo-v2.png` 图片。
- Dashboard 顶部、中部、底部改为固定列宽布局，减少页面错乱和卡片套卡片。
- 图表卡片内部改为直接渲染 Liora LineChart，去掉嵌套 Card。
- 截图尝试受当前显示/工具限制未得到可靠图片，因此未宣称 1:1 视觉验收。
