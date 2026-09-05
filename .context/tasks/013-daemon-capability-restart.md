# 任务 013：旧 daemon 能力隔离

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：小
- 依赖：`.context/tasks/012-kernel-install-interaction-fix.md`
- 生产行为变更：是

## 任务目标

避免应用连接到旧版本常驻 daemon 导致 `Unknown method: InstallOfficialKernel`，并确保同一窗口中所有内核操作使用当前 daemon 的 IPC 能力。

## 范围

- 增加 daemon `GetDaemonInfo` 能力握手。
- 应用启动时验证协议版本和 `InstallOfficialKernel` 能力。
- 检测到旧 daemon 时选择当前二进制指纹 socket，启动并等待新 daemon 就绪；不强制终止旧进程。
- 保持 IPC socket 名称仅含安全字符且位于用户运行目录。

## 非目标

- 不杀掉其他 daemon 或修改系统服务。
- 不改变代理、TUN、节点和持久化数据语义。

## 预期文件

- `.context/tasks/013-daemon-capability-restart.md`
- `crates/narya-ipc/src/lib.rs`
- `crates/narya-app/src/ipc.rs`
- `crates/narya-daemon/src/main.rs`
- `crates/narya-contract-tests/src/lib.rs`

## 验收标准

- 旧 daemon 不认识新方法时，应用不再向旧 socket 发送安装请求。
- 新 daemon 握手返回当前版本和官方内核能力后，安装请求走新 socket。
- 旧进程不被终止，宿主机系统配置不变。

## 验证

- `cargo check --workspace`
- `cargo test --workspace`
- app/daemon 定向 clippy
- 隔离 XDG 目录启动 app 与 daemon，确认 `Connected to daemon IPC`。

## 风险与回滚

- 风险：旧 daemon 残留 socket 占用用户运行目录；指纹 socket 避免冲突，旧 socket 由旧进程自行维护。
- 回滚：回退本任务源码恢复默认 socket 连接；不删除旧运行时文件。

## 完成记录

- 已增加 `GetDaemonInfo`，验证协议版本和 `InstallOfficialKernel` 能力。
- 检测到不兼容 daemon 时使用当前 daemon 二进制指纹 socket，不终止旧进程，并等待新 daemon 握手完成。
- 指纹 socket 名称使用短哈希，避免深层隔离目录下超过 Unix `SUN_LEN` 限制。
- 通过定向构建、测试、clippy、格式检查、diff 检查、ctx validate 和隔离 X11 启动烟测。
- 最终复核通过：`cargo check --workspace`、`cargo test --workspace`、`RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all -- --check`、`git diff --check`、ctx validate；隔离 X11 启动输出 `Connected to daemon IPC`。
- 修复首次 `cargo run` 未生成 daemon 的路径：debug 启动先执行 `cargo build -p narya-daemon`，再进行能力握手；自定义 `CARGO_TARGET_DIR` 的隔离 `cargo run` 也已输出 `Connected to daemon IPC`。
