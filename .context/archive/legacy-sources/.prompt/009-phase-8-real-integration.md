# Phase 8：UI/后端逻辑闭环与全功能联调 (Full Integration)

## 背景

Phase 7 已经完成了 `narya-daemon` 的核心功能引擎。现在的任务是“合龙”：将 `narya-app` 的 UI 交互与这些真实的后端能力对接，实现真正的代理控制与状态回显。

## 本阶段目标

1. 在 `AppState` 中集成 `IpcClient` 建立持久连接。
2. 实现 Dashboard 页面的一键代理开关逻辑，并调用后端 `SetSystemProxy` 指令。
3. 替换 Nodes 列表的测速触发为后端真实的 `test_latency` (暂由模拟逻辑移至后端)。
4. 实现 Sidebar Footer 流量网速的真实数据驱动（接收 IPC 通知）。

## 具体任务

1. **持久化 IPC 连接**：
   - 完善 `AppState::start_traffic_monitor` 以处理 IPC 断线重连。
   - 实现一个异步消息循环，监听来自 Daemon 的 `IpcNotification`。
2. **Dashboard 开关集成**：
   - 在 Dashboard 视图中增加或激活代理开关。
   - 点击开关后通过 `IpcClient` 发送指令，并根据返回结果更新 UI 状态。
3. **真实流量回显**：
   - 当收到 `TrafficUpdate` 通知时，实时更新 `AppState` 中活动节点的速率。
   - 验证侧边栏网速数值与后端推送一致。
4. **订阅与配置同步**：
   - 启动时从后端加载本地持久化的 Profiles 并展示在 UI。

## 约束

- 所有的 IO/IPC 操作必须异步进行，不得阻塞 GPUI 主线程。
- 考虑到权限，如果 Daemon 连接失败，UI 应显示醒目的“后端未运行”提示。
- 保持 `cargo clippy` 与 `cargo fmt` 全绿。

## 验证命令

- 启动 Daemon: `cargo run -p narya-daemon`
- 启动 App: `cargo run -p narya-app`
- 在 App 中切换开关，观察系统代理设置是否变化。

## 完成后必须更新

- `.memory/status.md`
- `.memory/tasks.md`
- `.memory/changelog.md`

## Git 提交建议

`feat(app): achieve full UI/Backend integration with real IPC data flow`
