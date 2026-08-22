# 当前状态

## 更新时间

2026-06-21

## 当前阶段

Liora UI 重建第二轮已完成：历史 `./ui` spec 相关非图片文件已删除，页面层已按“只组合 Liora/本地组件库、严禁原生 GPUI 布局样式”的红线重构。`crates/narya-app/src/views/app_shell.rs` 现在只负责状态快照、路由和语义组件组合；原生 GPUI 底层能力只隔离在 `crates/narya-app/src/ui_kit.rs` 本地组件库边界。

## 本次完成

1. **清理错误 spec**：删除 `ui/specs/**` 以及 `ui/**/*.spec.json` 等 spec 相关非图片文件；`ui` 下剩余视觉真源均为图片。
2. **红线契约测试**：`narya-contract-tests` 新增 `ui_specs_are_image_only_and_page_layer_has_no_raw_gpui_layout`，自动检查：spec 相关非图片不存在，页面层不出现 `use gpui::`、`div()`、`.flex()`、`.bg()`、`.border_color()`、`.text_color()`、`.padding_` 等原生 GPUI 布局/样式 token。
3. **页面层重做**：删除旧 raw GPUI 页面模块和旧 `components.rs/theme.rs`；`views/app_shell.rs` 改为 Liora 组件 + `narya_ui` 本地组件库的语义组合。
4. **本地组件库边界**：`ui_kit.rs` 承担 ShellFrame、Sidebar、HeaderBar、FooterBar、NaryaPage、NaryaCard、NaryaMetric、NodeCardData、订阅项、设置行、图表卡等可复用组件；必要 GPUI 底层封装集中于此，作为未来反哺 Liora 的候选。
5. **视觉方向**：主骨架按 1536×1024 图片真源重建：约 264px 左侧栏、108px 头部、68px 底栏、冷白背景、蓝紫激活态、圆角卡片、Dashboard/Nodes/Subscriptions/Settings 等页面的高密度卡片布局和图表/状态块。
6. **旧功能红线保留**：无 splash、Liora 初始化、连接按钮接线、IPC error 检查、kernel install fail-closed、runtime path、config generator fail-closed 等契约继续通过。

## 验证证据

已运行：

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings
cargo clippy -p narya-app --lib -- -D warnings
timeout 8s cargo run -p narya-app
```

结果：fmt/check/test/分段 clippy 均退出 0；`narya-contract-tests` 5/5 通过；GUI run 成功进入 `target/debug/narya-app` 并保持运行到 8 秒 timeout，退出 124 为预期烟测截断。

## 已知限制

- 当前完成的是“红线架构重做 + 主视觉骨架重建”，尚未做真实截图与源 PNG 的像素级 diff，因此不能宣称最终 1:1 验收完成。
- 下一轮需要在有显示/截图能力的环境中逐页截图，对照 `ui/*.png` 继续校准具体间距、图标、曲线、卡片高度、字体大小和页面子状态。
- 真实 kernel installer、订阅添加/导入、测速、工具箱动作和 framed IPC codec 仍是后续实现项。

## 2026-06-21 菜单与 Dashboard 纠偏

- 左侧菜单从 Button 拼接改为按图的全宽菜单行，使用 Liora Lucide 图标，不再用文字符号冒充菜单图标。
- active 菜单改为浅蓝背景 + 蓝色图标/文字，更接近效果图。
- 左栏品牌区改用 `ui/icons/narya-logo-v2.png` 图片。
- Dashboard 顶部、中部、底部改为固定列宽布局，减少页面错乱和卡片套卡片。
- 图表卡片内部改为直接渲染 Liora LineChart，去掉嵌套 Card。
- 截图尝试受当前显示/工具限制未得到可靠图片，因此未宣称 1:1 视觉验收。

## 2026-06-21 Light 默认主题

- 应用户要求，启动时从 `liora::init_liora(cx)` 改为 `liora::init_liora_with_mode(cx, liora::ThemeMode::Light)`，默认进入 Light 主题，便于继续按浅色效果图校准。
- 契约测试已同步锁定显式 Light 初始化。
- 验证：`cargo fmt --all -- --check`、`cargo check -p narya-app`、`cargo test -p narya-contract-tests` 通过；`timeout 8s cargo run -p narya-app` 成功启动并按预期 timeout。

## 2026-06-21 视觉还原纠偏切片

用户指出布局仍低于可接受还原度后，本轮改为真实截图驱动纠偏：用 `spectacle` 抓取运行窗口 `/tmp/narya-shot*.png` 对照 `ui/dashboard.png`，不再只凭代码猜测。

完成：
- Linux/Wayland 窗口改为 GPUI client-side decorations，移除原生深色标题栏，窗口比例接近效果图。
- Dashboard 改成按源图三行骨架：164px 顶部开关卡、310px 中部快速连接/网络概览、284px 底部流量/统计/日志。
- 顶部右侧加入源图式窗口控制图标，动作按钮改 Lucide 图标。
- Hero 卡改本地 design_card 封装，使用 Lucide Monitor/Network 图标、已连接 mock 默认状态和蓝/绿开关。
- Nodes 页面按效果图重组为顶部控制卡 + 筛选条 + 策略组/节点列表/测速概览三列 + 底部趋势/详情。
- 快速连接行压缩高度并改国家 emoji，避免第四行裁切和红十字误读。

真实截图证据：`/tmp/narya-shot5.png` 已生成用于对照。仍不能宣称 1:1：图表内框、菜单按钮质感、国旗圆形样式、卡片微阴影/渐变和部分尺寸还需继续校准。

## 2026-06-21 Liora-first 控件纠偏

用户明确补充：所有组件都应尽可能直接使用 Liora 现成控件；需要扩展时只能在 `ui_kit` 包装，只有 Liora 确实没有时才在本地组件库手搓。已按该规则完成一轮纠偏：

- Sidebar 导航正式改为 Liora `Menu`，不再用 Button 拼接菜单。
- 节点页搜索/筛选/排序分别改为 `ui_kit` 中基于 Liora `Input` / `Segmented` / `Select` 的包装。
- Dashboard hero 开关与设置开关改为基于 Liora `Switch` 的包装；没有真实业务回调的展示控件设置为只读，避免产生假本地状态。
- 策略组与设置分类改为整组 Liora `Menu` 包装，不再每行单独手搓/单独 Menu。
- Metric 数值展示改为 Liora `Statistic`，图标参数从文字符号改为 Lucide `IconName`，减少字体大小不协调和符号冒充图标。
- 统一字号 token：display/brand/card/body/small/caption/number，减少 `.sm()` / `.xs()` 与硬编码混用造成的层级不协调。

验证：`cargo fmt --all -- --check`、`cargo check --workspace`、`cargo test --workspace`、`RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings`、`cargo clippy -p narya-app --lib -- -D warnings` 均通过；`timeout 8s cargo run -p narya-app` 成功启动并按预期 124 timeout。截图尝试 `/tmp/narya-menu-typo.png` 仍捕获到 Codex 终端而非应用窗口，未作为视觉验收证据。
