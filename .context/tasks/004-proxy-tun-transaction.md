# 任务 004：代理与 TUN 事务边界

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：大
- 依赖：`.context/tasks/003-framed-ipc-kernel-registry.md`
- 生产行为变更：是

## 任务目标

建立 system proxy 与 TUN 的统一模式模型、应用/回滚事务和健康状态边界，避免内核已启动但系统流量未按规则转发。

## 范围

- 代理模式、TUN 模式、DNS 劫持和 bypass 配置的领域模型。
- system proxy 旧状态捕获、apply、rollback 接口。
- daemon 启动/停止事务：内核、代理、TUN 失败时按逆序回滚。
- 离线 fake backend 测试，不调用真实桌面或路由服务。

## 非目标

- 不在无平台权限和官方实现证据时伪造 TUN 驱动。
- 不实现完整内核下载器或 UI 视觉重构。

## 预期文件

- `crates/narya-core/src/lib.rs`
- `crates/narya-platform/src/lib.rs`
- `crates/narya-daemon/src/proxy.rs`
- `crates/narya-daemon/src/main.rs`
- 对应测试与上下文记录

## 验收标准

- system proxy 与 TUN 共享规则选择输入，但平台副作用由独立 backend 执行。
- apply 任一步失败会回滚已经成功的步骤，并返回结构化失败原因。
- stop 先恢复代理、DNS、路由，再停止内核；重复 stop 幂等。
- fake backend 测试覆盖成功、部分失败、回滚和恢复旧状态。

## 验证

```bash
cargo fmt --all -- --check
cargo test -p narya-core -p narya-platform -p narya-daemon
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 风险与回滚

- 风险：不同 OS 的代理/TUN 能力不等价；按 backend 能力返回明确 unsupported，不共享 shell 字符串拼接。
- 回滚：仅启用 fake backend 和离线事务；平台 backend 未通过测试前默认不改变真实系统设置。

## 完成记录

- `narya-platform` 新增 `ProxyMode`、`RoutingPlan`、`SystemProxyPlan`、`TunPlan`、`DnsPlan`、`PlatformSnapshot` 和 `PlatformAdapter`。
- `apply_routing` 在 system proxy/TUN/DNS 任一步失败时恢复快照；fake backend 覆盖成功、DNS 失败、TUN 缺失和 TUN 失败四种场景。
- daemon 新增 `SetRoutingMode`；Linux GNOME backend 捕获并恢复 mode、HTTP、HTTPS、SOCKS、bypass domains，并将代理 apply/restore 接入启停边界。
- `StopKernel` 先恢复代理，再停止内核；恢复失败时保留内核运行，避免留下未知流量路径。TUN 和 macOS 事务在无安全实现时明确 unsupported。
- 验证：`cargo fmt --all -- --check`、`cargo check --workspace`、`cargo test --workspace`、分段 clippy、`timeout 5s cargo run -p narya-daemon` 启动烟测、`timeout 5s cargo run -p narya-app` 无显示环境诊断烟测及 daemon routing-mode smoke test 均通过。
- 未覆盖：Windows/macOS 完整快照、Linux TUN 驱动、DNS 系统级接管和真实流量泄漏探针留在后续任务。
