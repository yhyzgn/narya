# 阶段任务计划

## 2026-06-21：Liora UI 重建第二轮
状态：完成 ✅

- [x] 根据用户确认删除 `./ui` 下 spec 相关非图片文件，只保留图片视觉真源。
- [x] 建立页面层零原生 GPUI 红线契约测试。
- [x] 删除旧 raw GPUI 页面模块与旧 `components.rs/theme.rs`。
- [x] 将 `views/app_shell.rs` 重写为状态/路由/语义组合层，不出现原生 GPUI 布局/样式 token。
- [x] 扩展 `ui_kit.rs` 为本地解耦组件库边界，集中封装必要底层能力和可复用 Narya 组件。
- [x] 按主图片骨架重建 Dashboard、Nodes、Subscriptions、Settings，并提供 Config/Connections/Rules/Logs/Tools 的 Liora 化运行面。
- [x] 保留上一轮集成红线：无 splash、连接接线、IPC error 检查、kernel install fail-closed、runtime path、config generator fail-closed。
- [x] 运行 fmt、check、test、clippy 分段严格验证和 GUI 启动烟测。

## 下一阶段：像素级视觉校准
状态：待开始 🏗️

- [ ] 使用运行窗口截图逐页对照 `ui/dashboard.png`、`ui/nodes.png`、`ui/subscriptions.png`、`ui/settings.png`。
- [ ] 校准左栏 logo 图形、Lucide 风格图标、卡片尺寸、间距、字体、曲线图、状态标签和按钮位置。
- [ ] 优先将 `ui_kit.rs` 继续拆分为更小的本地组件模块：shell、cards、charts、nodes、subscriptions、settings。
- [ ] 将本地组件中可通用的能力整理为“可反哺 Liora”候选清单。
- [ ] 设计 framed IPC codec 替代当前临时 JSON read 模型。

## 2026-06-21 菜单与 Dashboard 纠偏

- 左侧菜单从 Button 拼接改为按图的全宽菜单行，使用 Liora Lucide 图标，不再用文字符号冒充菜单图标。
- active 菜单改为浅蓝背景 + 蓝色图标/文字，更接近效果图。
- 左栏品牌区改用 `ui/icons/narya-logo-v2.png` 图片。
- Dashboard 顶部、中部、底部改为固定列宽布局，减少页面错乱和卡片套卡片。
- 图表卡片内部改为直接渲染 Liora LineChart，去掉嵌套 Card。
- 截图尝试受当前显示/工具限制未得到可靠图片，因此未宣称 1:1 视觉验收。

## 2026-06-21 Light 默认主题

- [x] 将 Narya 启动默认主题改为 Liora Light。
- [x] 更新启动契约测试以锁定 Light 初始化。
- [x] 运行 fmt、app check、contract tests、GUI timeout smoke。

## 2026-06-21 视觉还原纠偏切片

- [x] 用真实运行截图对照 `ui/dashboard.png`，确认主要偏差。
- [x] 去除 Linux/Wayland 原生深色标题栏，改 GPUI client-side decorations。
- [x] 重做 Dashboard 三行主骨架和顶部 Hero 卡样式。
- [x] 重组 Nodes 主结构为策略组/节点列表/测速概览三列。
- [x] 保持页面层零原生 GPUI 红线，契约测试通过。
- [ ] 下一轮继续像素校准图表内框、菜单 inactive 质感、圆形国旗、阴影/渐变和其它页面。

## 2026-06-21 Liora-first 控件纠偏

- [x] 将 Sidebar 导航从 Button 拼接改为 Liora `Menu`。
- [x] 将搜索/筛选/排序从 Button 冒充控件改为 Liora `Input` / `Segmented` / `Select` 包装。
- [x] 将开关展示改为 Liora `Switch` 包装，并在未接业务状态时只读化。
- [x] 将策略组/设置分类整组改为 Liora `Menu` 包装。
- [x] 将 Metric 文字符号图标改为 Lucide `IconName`，数值使用 Liora `Statistic`。
- [x] 收敛字体 token，减少大小不协调。
- [x] 通过 fmt/check/test/clippy/GUI smoke 验证。
- [ ] 后续继续把连接/日志等列表型页面迁移到 Liora `Table` / `VirtualizedList` 等现成控件。
- [ ] 后续把可交互筛选/search/select 状态提升到 `AppState` 后再解除只读遮罩。
