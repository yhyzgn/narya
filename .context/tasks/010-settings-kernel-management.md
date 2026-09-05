# 任务 010：常规设置与私有内核管理面板

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：中
- 依赖：`.context/tasks/009-kernel-hot-switch.md`
- 生产行为变更：是

## 任务目标

完成可交互、可滚动且不被父级裁切的常规设置与内核管理面板；内核安装、升级、卸载和切换仅作用于 Narya 私有托管目录。

## 范围

- 常规设置页使用持久化 Liora 控件并保持点击状态。
- 右侧内核面板展示名称、版本、安装、运行、健康和失败状态。
- 为每个内核提供安装、升级、卸载和切换动作，并显示操作结果。
- daemon 仅发现和启动 Narya 私有目录中的已验证内核；卸载拒绝活动内核及非托管路径。
- 修复右侧面板滚动边界与超宽控件裁切。

## 非目标

- 不把 Shadowsocks 等节点协议误建模为独立可执行内核。
- 不安装系统包、不修改 `PATH`，不删除操作系统或用户自行安装的可执行文件。
- 不执行真实系统代理、TUN、DNS 或权限变更。
- 不新增未经签名发布清单验证的自动下载源。

## 预期文件

- `crates/narya-kernel/src/lib.rs`
- `crates/narya-daemon/src/installer.rs`
- `crates/narya-daemon/src/kernel.rs`
- `crates/narya-daemon/src/main.rs`
- `crates/narya-app/src/state.rs`
- `crates/narya-app/src/views/app_shell.rs`
- `crates/narya-contract-tests/src/lib.rs`
- `.context/plans/002-runtime-foundation.md`
- `.context/system/overview.md`

## 验收标准

- 常规设置中的开关可交互且不会在重绘时丢失状态。
- 内核面板在窗口高度不足时可独立纵向滚动，字段和按钮不被卡片裁切。
- 每个已注册内核显示版本、运行、健康和错误信息，并按状态提供有效动作。
- 安装、升级和卸载只访问 `narya_ipc::kernel_install_dir()` 下的内核目录。
- 活动内核不能卸载，系统 `PATH` 中的同名程序不会被识别为 Narya 已安装内核。

## 验证

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets -- -D warnings`
- 使用仓库 `target/` 内隔离 XDG 目录运行 `timeout 12s cargo run`
- 图形会话可用时检查设置页右侧滚动、按钮和控件命中区域。

## 风险与回滚

- 风险：错误的路径判断可能误删外部内核；通过只接受 `KernelId` 并由 daemon 拼接私有根目录消除任意路径输入。
- 风险：活动内核卸载会中断代理；daemon 在文件操作前明确拒绝。
- 回滚：回退本任务文件即可恢复只安装/升级和旧面板；已安装的私有内核文件不会被自动删除。

## 完成记录

- 内核注册表改为只探测 `narya_ipc::kernel_install_dir()`，不再采用系统 `PATH` 中的同名可执行文件。
- daemon 新增 `UninstallKernel`，卸载仅删除私有托管目录中的 `current`、`version`、`sha256`，拒绝活动内核和非托管路径。
- 设置页右栏改为单一有界滚动列；内核列表按单行展示版本、运行、健康信息和操作按钮，错误按需显示。
- UI 安装、升级、卸载、切换均连接真实 AppState/IPC 操作，状态和错误可观察。
- 通过 `cargo fmt --all -- --check`、`cargo check --workspace`、`cargo test --workspace`、`cargo test -p narya-contract-tests`、`RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets -- -D warnings`、`git diff --check`。
- 使用仓库 `target/narya-smoke` 隔离 XDG 目录，以 X11 后端运行 `cargo run`，日志确认 `Connected to daemon IPC`；Wayland 无 compositor 时的 `NoCompositor` 仅为当前探针环境限制。
