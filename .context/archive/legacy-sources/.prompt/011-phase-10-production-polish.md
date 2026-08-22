# Phase 10：网络拉取、托盘支持与配置合成 (Production Polish)

## 背景

Phase 9 已经实现了本地订阅解析。现在的任务是完成最后的一公里：真实的订阅拉取（网络层）以及增强应用在后台运行的能力（托盘图标）。

## 本阶段目标

1. 在 `narya-subscription` 中集成 `reqwest`，实现真实 URL 的下载逻辑。
2. 为 `narya-app` 增加系统托盘 (System Tray) 支持，允许最小化到后台。
3. 实现从本地 `Node` 列表合成 `sing-box` 生产级 JSON 配置的逻辑。
4. 完善错误处理 UI（如网络超时、权限不足等提示）。

## 具体任务

1. **网络拉取集成**：
   - 在 `narya-subscription` 中增加 `fetch_remote_subscription(url)`。
   - 实现简单的 User-Agent 伪装（如 Clash 标识）。
2. **系统托盘支持**：
   - 调研并集成 `tray-icon` 或使用 GPUI 自带（如果支持）的托盘方案。
   - 实现右键菜单：显示/隐藏、一键连接、退出。
3. **配置合成器**：
   - 在 `narya-daemon` 中实现 `ConfigGenerator`。
   - 将当前选中的 `Node` 转换为 sing-box 兼容的 Outbound JSON。
4. **异常提示 UI**：
   - 开发通用的 Toast 或 Notification 组件，用于回显后端错误。

## 约束

- 托盘逻辑必须支持 Linux (libappindicator) 和 macOS。
- 配置文件合成需遵循 sing-box 官方最新规范。
- 保持 `cargo clippy` 与 `cargo fmt` 全绿。

## 验证命令

- 启动应用 -> 在 Subscriptions 中输入真实订阅 URL -> 点击更新 -> 验证 Nodes 列表出现真实节点。
- 最小化主窗口 -> 在托盘右键点击“显示” -> 验证窗口恢复。

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`

## Git 提交建议

`feat(app): implement remote subscription fetching and system tray support`
