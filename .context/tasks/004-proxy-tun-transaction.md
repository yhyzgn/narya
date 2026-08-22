# 任务 004：代理与 TUN 事务边界

- 状态：待开始
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

- 待开始。
