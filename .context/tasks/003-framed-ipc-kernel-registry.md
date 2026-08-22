# 任务 003：Framed IPC 与多内核注册契约

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：大
- 依赖：`.context/tasks/002-runtime-foundation.md`
- 生产行为变更：是

## 任务目标

将 daemon/UI IPC 改为可处理拆包与粘包的 framing，并建立可观察的多内核注册、安装/升级/切换状态契约。

## 范围

- 长度前缀或等价的消息 codec 及离线拆包测试。
- 内核标识、能力、版本、路径和生命周期状态模型。
- daemon 不再硬编码单一启动内核；未知内核与失败状态显式返回。

## 非目标

- 本任务不实现具体平台下载器、签名服务或 TUN 驱动。
- 本任务不改变 UI 视觉布局。

## 预期文件

- `crates/narya-ipc/src/lib.rs`
- `crates/narya-daemon/src/main.rs`
- `crates/narya-daemon/src/kernel.rs`
- `crates/narya-kernel/src/lib.rs`
- 对应 Cargo 清单与测试

## 验收标准

- 任意合法消息在 1 字节拆分、批量粘连和多消息流中均可正确解码。
- 内核状态包含安装、升级、运行、健康和失败原因，不以 `running=true` 代替健康。
- 启动/停止/切换错误不遗留未声明的子进程或代理副作用。

## 验证

```bash
cargo fmt --all -- --check
cargo test -p narya-ipc -p narya-kernel -p narya-daemon
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 风险与回滚

- 风险：协议变更会影响现有 app/daemon 版本；先提供版本字段和单进程兼容窗口。
- 回滚：保留旧方法名但拒绝无 framing 的连接，确保失败关闭而非误解析。

## 完成记录

- 已完成 `narya-ipc` 4 字节大端长度前缀 framing，带 16 MiB 上限、JSON 解码错误和截断检测。
- `FrameDecoder` 覆盖 1 字节拆分、连续粘包、超大帧和截断帧测试。
- `narya-kernel` 新增 `KernelId`、`KernelState`、`KernelRecord`、`KernelRegistry`；按 PATH 探测 sing-box、mihomo、xray，并将运行状态与健康状态分离。
- daemon/app 已切换到 framed IPC；请求带协议版本，未知版本 fail-closed；启动代理失败时 app 会尝试停止刚启动的内核。
- kernel manager 启动前解析生成配置中的本地 HTTP/SOCKS 监听，并在 bounded window 内验证至少一个监听可连接；监听消失时主动终止子进程并报告失败，避免仅凭进程存活伪造健康状态。
- `GetKernelStatus` 返回安装、版本、运行、健康、状态和失败原因；在无核心环境中实际返回三个 `not_installed`、`healthy=false` 状态。
- 验证：`cargo fmt --all -- --check`、`cargo check --workspace`、目标 crate 测试、`cargo test --workspace`、分段 clippy、`ctx validate` 均通过；真实 daemon Python Unix socket framed Ping/KernelStatus/版本错误烟测通过；`timeout 8s cargo run -p narya-app` 在无 DISPLAY 环境下输出可诊断错误并正常退出，不再 panic。
- 未覆盖：真实内核安装/升级下载器、TUN/system proxy 事务和流量级健康探针留在后续任务。
