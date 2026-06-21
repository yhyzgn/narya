# 阶段任务计划

## 2026-06-21：Liora UI 重建
状态：完成 ✅

- [x] 读取 `prompt.md`、`.memory/*`、`.prompt/*`、架构和 UI 规格上下文。
- [x] 调研 Liora 0.1.5 文档/本地源码示例，确认 `liora::init_liora(cx)`、`gpui::Application::new()`、组件 API 与 GPUI 0.2.2 版本要求。
- [x] 移除 splash 启动路径，直接打开主窗口。
- [x] 增加 `ui_kit` 项目组件边界，所有新主壳布局通过 Liora 组件/项目 wrapper 组合。
- [x] 重写主 AppShell 为 Liora 版本，覆盖 Dashboard/Nodes/Subscriptions/Config/Connections/Rules/Logs/Tools/Settings 的首版运行面。
- [x] 新增 `narya-contract-tests` 锁定无 splash、Liora 初始化、依赖对齐、组件边界和关键集成红线。
- [x] 连接按钮接入真实 AppState 动作：顶部连接、Dashboard 连接/断开、节点连接。
- [x] 修复 IPC/daemon 响应顺序与 app 错误处理：忽略先到通知、只在 `IpcResponse.error.is_none()` 后更新 connected state。
- [x] 移除 fake kernel install 成功路径，安装入口显示未实现并 fail-closed。
- [x] 移除固定 `/tmp/narya.sock` / `/tmp/narya-kernel.json`，改用 per-user runtime dir。
- [x] 修复 sing-box config 生成红线：unsupported protocol fail-closed，Shadowsocks 使用 `method:password`，无假密码/direct proxy fallback。
- [x] 保留并验证核心/daemon/IPC/订阅解析可编译测试。
- [x] 运行 fmt、check、test、clippy 分段严格验证和 GUI 启动烟测。
- [x] 独立 code-reviewer 复核 review blockers，最终 APPROVED。

## 下一阶段建议：Liora 视觉深度还原与交互落地
状态：待开始 🏗️

- [ ] 逐页对照 `ui/*.png` 和 `ui/specs/main_window_spec_detailed.md` 做截图级视觉校准。
- [ ] 把旧 raw GPUI 页面模块中的有价值业务结构迁移到 Liora wrapper，迁完后删除旧模块。
- [ ] 封装 Narya 专属 Sidebar、TopBar、StatusCard、NodeCard、SubscriptionCard、KernelPanel、Toolbar、Section 等低耦合组件。
- [ ] 接入真实内核安装器：下载、校验、权限设置、版本读取、错误提示。
- [ ] 为订阅刷新、节点测速、系统代理切换增加更细粒度契约/单元测试。
- [ ] 为 IPC 引入真正 length-prefixed/framed codec，替代当前“单 read 一个 JSON 对象”的临时协议。
