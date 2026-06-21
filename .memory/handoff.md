# Narya Handoff — 2026-06-21

## 当前可接手状态

Liora UI 第二轮红线重做已完成。启动命令：

```bash
cargo run -p narya-app
```

当前应用直接进入主窗口，不显示 splash。页面层在 `crates/narya-app/src/views/app_shell.rs`，但该文件只做状态/路由/语义组件组合；原生 GPUI 底层能力只允许在 `crates/narya-app/src/ui_kit.rs` 本地组件库边界出现。

## 关键约束

- `ui` 下历史 spec 相关非图片文件已删除，后续不得恢复或依赖旧 spec。
- UI 视觉真源是 `ui/**/*.png` 等图片。
- 页面/业务 UI 代码严禁直接写原生 GPUI 布局/样式。
- Liora 不足时，先封装本地低耦合组件，后续作为反哺 Liora 候选。

## 关键文件

- `crates/narya-app/src/views/app_shell.rs`：页面层语义组合，无原生 GPUI 布局/样式 token。
- `crates/narya-app/src/ui_kit.rs`：本地 Narya/Liora 组件库边界。
- `crates/narya-contract-tests/src/lib.rs`：锁定 spec 删除和页面层零 GPUI 红线。
- `prompt.md`：已更新为图片真源和 Liora-first 规则。

## 必跑验证

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings
cargo clippy -p narya-app --lib -- -D warnings
timeout 8s cargo run -p narya-app
```

说明：`timeout 8s cargo run -p narya-app` 退出 124 是预期烟测截断，只要输出显示进入 `Running target/debug/narya-app` 且无崩溃即可。

## 下一步推荐

下一轮不要再写 spec。直接打开源图片和运行截图做像素级对照，从 `dashboard.png`、`nodes.png`、`subscriptions.png`、`settings.png` 开始校准。优先拆分 `ui_kit.rs` 为 `ui_kit/shell.rs`、`cards.rs`、`nodes.rs`、`subscriptions.rs`、`settings.rs` 等模块，降低后续反哺 Liora 的成本。

## 2026-06-21 菜单与 Dashboard 纠偏

- 左侧菜单从 Button 拼接改为按图的全宽菜单行，使用 Liora Lucide 图标，不再用文字符号冒充菜单图标。
- active 菜单改为浅蓝背景 + 蓝色图标/文字，更接近效果图。
- 左栏品牌区改用 `ui/icons/narya-logo-v2.png` 图片。
- Dashboard 顶部、中部、底部改为固定列宽布局，减少页面错乱和卡片套卡片。
- 图表卡片内部改为直接渲染 Liora LineChart，去掉嵌套 Card。
- 截图尝试受当前显示/工具限制未得到可靠图片，因此未宣称 1:1 视觉验收。

## 2026-06-21 Light 主题更新

- 当前 app 默认 Light：`crates/narya-app/src/lib.rs` 使用 `liora::init_liora_with_mode(cx, liora::ThemeMode::Light)`。
- 契约测试已更新；后续视觉校准应以浅色主题运行截图为准。

## 2026-06-21 视觉纠偏接力

本轮已用真实截图纠偏 Dashboard/Nodes 骨架，并修掉 Linux 原生标题栏。后续继续从 `/tmp/narya-shot5.png` 与 `ui/dashboard.png` 对照，不要声明 1:1。优先项：去掉/弱化 Liora LineChart 内层边框，菜单 inactive 更接近白底源图，国旗改圆形图标，细调卡片阴影/渐变与右上窗口控制位置。
