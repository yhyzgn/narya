# Phase 6：后端内核集成 (Kernel Integration)

## 背景

Phase 5 已经完成了所有主要交互逻辑的“高保真模拟”。现在的任务是建立真正的后端通信骨架，将 `narya-app` (UI) 与即将开发的 `narya-daemon` (后端服务) 对接。

## 本阶段目标

1. 在 `narya-daemon` 中建立基础的服务骨架。
2. 定义 App 与 Daemon 之间的通信协议 (RPC)。
3. 实现最核心的功能闭环：点击 UI 上的代理开关，触发 Daemon 修改系统网络设置。
4. 接入真实的 `sing-box` 内核 API 获取节点状态。

## 具体任务

1. **Daemon 骨架搭建**：
   - 在 `crates/narya-daemon` 中实现基础的事件循环。
   - 使用 `tokio` 处理并发任务。
2. **RPC 通信协议**：
   - 调研并选择适合跨进程通信的方案 (Unix Domain Socket + JSON-RPC 为首选)。
   - 在 `narya-ipc` 中定义共享的 Request/Response 结构。
3. **内核 API 接入**：
   - 实现 Daemon 调用 `sing-box` 的 REST API 或交互式接口。
   - 将真实的节点流量、延迟数据推送至 App 端的 `AppState`。
4. **系统设置管理**：
   - 实现 Linux/macOS 下系统代理 (System Proxy) 的开启与关闭逻辑。

## 约束

- 后端逻辑必须是线程安全的。
- 考虑到权限问题，Daemon 可能需要以 sudo 权限运行（或通过 helper 提权）。
- 保持 `cargo clippy` 与 `cargo fmt` 全绿。

## 验证命令

- `cargo run -p narya-daemon` (启动后台服务)
- 在 App UI 中切换开关，观察 Daemon 控制台日志是否正确接收指令。

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`

## Git 提交建议

`feat(daemon): establish IPC skeleton and implement system proxy control`
