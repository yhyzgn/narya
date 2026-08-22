# 任务 007：运行时路由模式与 TUN 前置校验

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：中
- 依赖：`.context/tasks/005-karing-rule-config.md`、`.context/tasks/006-kernel-artifact-lifecycle.md`
- 生产行为变更：是

## 任务目标

把生成的 system proxy/TUN 配置接入 daemon 运行时状态，防止 UI 只切换本地布尔值或同时留下两条流量路径。

## 范围

- `StartKernel` 接受 routing plan 并生成对应模式配置。
- Linux TUN 前置检查由 daemon 执行，TUN 由 sing-box inbound 创建。
- system proxy/TUN 模式互斥、代理恢复和实时路由状态查询。

## 非目标

- 本阶段不宣称已完成 Windows/macOS TUN 或完整系统 DNS backend。
- 不在 daemon 中复制 sing-box 的路由实现。

## 预期文件

- `crates/narya-daemon/src/main.rs`
- `crates/narya-daemon/src/proxy.rs`
- `crates/narya-contract-tests/src/lib.rs`

## 验收标准

- TUN 请求缺少运行配置、设备或健康内核时 fail-closed。
- TUN 与 system proxy 不会同时保持活动状态。
- `GetRoutingStatus` 返回 configured/active/healthy 三个独立事实。
- system proxy/TUN 继续共用同一生成顺序。

## 完成记录

- `StartKernel` 可携带 `RoutingPlan` 并生成对应 TUN/system proxy 配置。
- Linux TUN 检查 `/dev/net/tun` 与 `iproute2`，并要求健康内核；macOS 仍明确拒绝。
- daemon 追踪 configured/active mode，阻止 TUN 与 system proxy 同时活动。
- 新增 `GetRoutingStatus`，返回 configured mode、active mode 和 kernel health。

## 验证

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 风险与回滚

- 风险：TUN 创建仍由内核进程负责，真实权限和路由需要宿主机探针。
- 回滚：TUN 前置检查失败不改变系统代理快照；模式切换失败保持原活动模式。
