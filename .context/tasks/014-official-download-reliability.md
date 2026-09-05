# 任务 014：官方内核下载可靠性

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：小
- 依赖：`.context/tasks/013-daemon-capability-restart.md`
- 生产行为变更：是

## 任务目标

修复 sing-box 官方 Release 资产可访问但瞬时网络错误导致安装直接失败的问题，并避免开发模式复用具备旧下载行为的同版本 daemon。

## 范围

- 为官方资产下载增加明确超时和有限重试。
- 保留官方 GitHub Release 域名、大小限制和解包校验边界。
- 在最终错误中保留底层网络原因。
- 开发模式优先连接当前 daemon 二进制指纹 socket。

## 非目标

- 不引入第三方镜像或非官方内核来源。
- 不修改系统代理、字体、PATH 或系统服务。
- 不终止已有 daemon 进程。

## 预期文件

- `AGENTS.md`
- `.context/plans/002-runtime-foundation.md`
- `.context/tasks/014-official-download-reliability.md`
- `crates/narya-app/src/ipc.rs`
- `crates/narya-daemon/src/official_release.rs`

## 验收标准

- sing-box 官方 `v1.14.0` Linux amd64 资产可在隔离目录完成真实安装。
- 连接、超时、传输中断、HTTP 429 和 5xx 使用有限重试，永久错误立即返回。
- UI 错误包含底层请求原因，不再只有泛化 URL。
- debug 应用不复用默认 socket 上的同版本旧 daemon。

## 验证

- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- 隔离 XDG 目录通过 IPC 真实安装 sing-box 并执行 `sing-box version`。

## 风险与回滚

- 风险：重试会延长网络故障时的等待；次数固定为三次且总请求受超时限制。
- 回滚：回退下载重试和 debug socket 选择逻辑；已安装工件仍留在隔离的 Narya 私有目录。

## 完成记录

- 官方 sing-box `v1.14.0` Linux amd64 资产已通过真实隔离 IPC 安装。
- 安装后的 `sing-box version` 输出 `1.14.0`，记录 SHA-256 与实际二进制一致。
- 下载器增加 120 秒请求超时、连接/请求/超时/429/5xx 有限重试，并透传底层错误。
- debug 应用优先使用当前 daemon 指纹 socket，避免同版本旧 daemon 复用。
- 验证通过：`cargo check --workspace`、`cargo test --workspace`、`RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all -- --check`、`git diff --check`。
