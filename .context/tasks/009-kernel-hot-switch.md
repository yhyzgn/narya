# 任务 009：统一内核生命周期与热切换

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：小
- 依赖：`.context/tasks/008-liora-routing-ui.md`
- 生产行为变更：是

## 任务目标

统一 sing-box、mihomo、xray-core 等可执行内核的安装、升级、状态管理和运行中切换；切换成功后立即沿用当前路由模式，失败时保留旧内核运行。

## 范围

- daemon `KernelManager` 安装/升级状态约束。
- daemon `SwitchKernel` IPC 与现有启动健康检查、回滚事务的衔接。
- GPUI 内核管理页的显式热切换动作和状态提交时机。

## 非目标

- 不新增独立 Shadowsocks 可执行内核；SS 继续作为节点协议由各适配器运行。
- 不执行宿主机 system proxy、TUN、DNS 或权限变更。
- 不引入新的下载源或绕过签名发布清单。

## 预期文件

- `crates/narya-kernel/src/lib.rs`
- `crates/narya-daemon/src/kernel.rs`
- `crates/narya-daemon/src/main.rs`
- `crates/narya-app/src/state.rs`
- `AGENTS.md`、`.context/plans/002-runtime-foundation.md`

## 验收标准

- 运行中的内核可以通过 `SwitchKernel` 切换到已安装的另一内核。
- 新内核未通过监听健康检查时，旧内核和 UI 活动内核保持不变。
- 非活动内核可在后台安装或升级；活动内核禁止原地覆盖。
- 切换完成后现有 system proxy/TUN 路由模式保持有效并重新确认健康状态。

## 验证

- `cargo test --workspace`
- `cargo check --workspace`
- `cargo build --release --workspace`
- `RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings`
- `RUST_MIN_STACK=134217728 cargo clippy -p narya-app --lib -- -D warnings`
- 隔离 `XDG_RUNTIME_DIR` daemon IPC：`Ping`、`GetKernelStatus`
- GUI 启动烟测：`timeout 20s cargo run -p narya-app`
- 根包启动烟测：`timeout 20s cargo run`；已成功编译并进入 `target/debug/narya`，无 `gpui_linux` 后端 panic；当前无图形会话时由超时退出（124）。
- 交互启动回归：`timeout 12s cargo run`；应用进入事件循环，无启动错误。

## 风险与回滚

- 风险：切换期间新旧进程存在极短监听交接窗口；健康检查失败会终止新进程并恢复旧进程。
- 风险：活动内核升级需先切换到另一已安装内核，否则 daemon 明确拒绝。
- 回滚：回退本任务代码即可恢复到 `StartKernel` 单入口；不删除已安装内核文件。

## 完成记录

- `KernelManager::install` 允许非活动内核安装/升级，拒绝覆盖活动内核。
- 新增 `SwitchKernel` IPC 别名，复用 `KernelManager::start` 的健康检查和回滚。
- UI 运行中切换不提前修改活动内核，成功后才提交状态并保持路由模式。
- 修复 `narya-app` 的 `gpui_platform` 依赖未启用 `x11`/`wayland` 导致 `cargo run` 启动时进入 `unreachable_code` 的问题。
- 修复 UI 交互阻塞：移除输入/下拉/分段/分类控件的全尺寸透明拦截层，启用 Liora 控件交互；侧边栏改用稳定 ID 的直接 GPUI 点击行；窗口改回原生 Server Decorations，避免静态窗口图标吞掉窗口操作。
- 修复 Linux 合成器透明底回归：侧栏非活动导航行显式使用侧栏实色背景，避免未绘制透明层显示为黑色整行。
- 修复运行中 System Proxy/TUN 互切：UI 通过 `SwitchKernel` 重新生成目标模式配置并在健康检查后提交 `SetRoutingMode`，daemon 允许从 TUN 恢复系统代理；失败时保留旧内核与旧 UI 状态。
- 追加可运行性修复：应用自动启动同目录 daemon；IPC、内核启动/停止与模式切换失败会显示在活动日志区域；无活动节点时拒绝启动并给出原因。
- 最终验证：`cargo fmt --all -- --check`、`cargo check --workspace`、`cargo test --workspace`（全部通过）；两条 clippy 门禁和 `cargo build --release --workspace`（通过）；隔离 XDG 目录 daemon `Ping`/`GetKernelStatus`/`GetRoutingStatus`（通过）；X11 实际窗口截图（1536×1024，侧栏无黑块，主界面正常绘制）。
- 设置页回归修复：移除设置页左侧嵌套 `page_row` 导致的二次 flex 压缩，改为固定分类列、弹性内容列和可滚动内核列；Liora 分类菜单、开关和外观分段控件均绑定 `AppState`，点击后状态立即重绘并保留在当前会话。
- 设置页验证：`cargo test --workspace`、`cargo check -p narya-app`、`RUST_MIN_STACK=134217728 cargo clippy -p narya-app --lib -- -D warnings`、`cargo fmt --all -- --check`、`git diff --check` 均通过；Wayland 图形会话启动探针已进入事件循环并连接 daemon。
- 设置页交互重构：参考 Liora gallery/docs 的宿主 Entity 模式，将分类导航、开关、外观分段、更新选择、内核工件/清单输入与清单选择提升为 `AppShell` 持久化 Entity；设置主体改用 `SettingsPage`、`SettingsGroup`、`SettingsItem`，避免 render 中重建控件导致焦点、下拉和输入状态丢失。补充源码契约测试锁定该生命周期边界。
- 本轮验证：`cargo fmt --all -- --check`、`cargo check --workspace`、`cargo test --workspace`、`cargo test -p narya-contract-tests`、`RUST_MIN_STACK=134217728 cargo clippy -p narya-app --lib -- -D warnings`、`git diff --check` 通过；`timeout 12s cargo run -p narya-app` 与 `timeout 10s cargo run` 均进入事件循环并连接 daemon。
