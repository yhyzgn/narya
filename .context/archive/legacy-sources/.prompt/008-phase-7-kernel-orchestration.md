# Phase 7：内核编排与系统控制 (Kernel Orchestration)

## 背景

Phase 6 已经搭建好了 IPC 通信的“路”与“桥”。现在的任务是在 `narya-daemon` 中实现具体的“载荷”：即真实管理 `sing-box` 内核进程，并能够执行系统级的网络配置修改。

## 本阶段目标

1. 实现 `narya-daemon` 对 `sing-box` 二进制文件的自动下载/查找与启动。
2. 定义并实现系统代理 (System Proxy) 开关的原子操作（支持 macOS/Linux）。
3. 实现配置文件 (Profiles) 的 YAML 解析与持久化。
4. 将 Daemon 获取到的真实内核流量统计通过 IPC 推送至 UI。

## 具体任务

1. **内核管理器**：
   - 实现 `KernelManager` struct，处理进程的 spawn、kill 及状态监控。
   - 使用 `tokio::process::Command` 管理子进程。
2. **系统代理组件**：
   - 编写 `SystemProxy` 抽象层。
   - 实现 `linux_gsettings` 与 `macos_networksetup` 后端。
3. **配置持久化**：
   - 在 `narya-config` 中实现 YAML 配置的 Load/Save。
   - 支持多节点订阅的本地合并生成 `sing-box` 兼容配置。
4. **实时流量推送**：
   - 监听 `sing-box` 的监控接口（或读取日志/统计）。
   - 通过 IPC 广播 `TrafficUpdate` 通知给所有连接的客户端。

## 约束

- 系统代理操作必须能够回滚（关闭时恢复原样）。
- 进程管理需处理僵尸进程与异常退出。
- 保持 `cargo clippy` 与 `cargo fmt` 全绿。

## 验证命令

- `cargo run -p narya-daemon`
- 在外部使用 `curl` 通过代理端口，验证流量是否在 UI 侧边栏 Footer 实时跳动。

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`

## Git 提交建议

`feat(daemon): implement kernel orchestration and system proxy control`
