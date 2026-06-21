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
