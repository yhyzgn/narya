# Phase 9：订阅解析引擎与多平台适配 (Business Deep Integration)

## 背景

Phase 8 实现了 UI 与后端的基础联调。现在的任务是完善核心业务逻辑：即如何获取节点（订阅解析）以及如何处理不同操作系统的差异。

## 本阶段目标

1. 实现 `narya-subscription` 模块，支持解析多种格式的订阅链接 (Base64, Clash, YAML)。
2. 实现 Daemon 端的定时订阅更新逻辑。
3. 增强跨平台适配，特别是 macOS 的 `networksetup` 支持。
4. 完善 `Nodes` 视图的节点列表过滤与排序功能。

## 具体任务

1. **订阅解析引擎**：
   - 编写 `SubscriptionParser` 能够将 URL 内容转为 `narya_core::Node` 列表。
   - 在 `narya-daemon` 中增加 `UpdateSubscription` IPC 指令。
2. **macOS 代理后端**：
   - 在 `narya-daemon/src/proxy.rs` 中实现 `MacOSNetworkSetup`。
   - 使用 `networksetup -setwebproxy "Wi-Fi" 127.0.0.1 2080` 等命令。
3. **UI 增强**：
   - 在 `Nodes` 页面实现搜索框的过滤逻辑。
   - 增加按延迟、速度排序的功能。
4. **持久化同步**：
   - 订阅成功后，自动将节点保存至本地 `profiles.yaml`。

## 约束

- 订阅解析需处理各种网络异常，具备超时机制。
- 排序逻辑应在 `AppState` 中处理，以保证响应速度。
- 保持 `cargo clippy` 与 `cargo fmt` 全绿。

## 验证命令

- 启动 Daemon 与 App。
- 在 Subscriptions 页面（占位）点击“Update”，验证节点列表是否出现真实数据。

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`

## Git 提交建议

`feat(logic): implement subscription parser and macOS proxy support`
