# 任务 012：修复官方内核安装与逐行按钮状态

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：中
- 依赖：`.context/tasks/011-official-kernel-catalog.md`
- 生产行为变更：是

## 任务目标

修复设置页内核无法安装，以及同文案按钮共享按下状态的问题；官方内核应能从对应项目的 GitHub Releases 下载、解包并安装到 Narya 私有目录。

## 范围

- daemon 从各内核固定官方仓库解析最新稳定版本和当前 Linux 架构资产。
- 对官方压缩包执行大小限制、可用的上游摘要校验、受限解包和二进制摘要记录。
- 复用现有原子安装、升级和回滚事务，不写系统目录。
- 每个内核行的安装、升级、卸载和切换按钮使用独立稳定 ID。
- 增加离线解析、解包、按钮 ID 和安装事务测试。

## 非目标

- 不修改系统 PATH、系统代理、字体或桌面配置。
- 不信任用户输入下载地址，不从第三方镜像安装。
- 本任务不承诺尚未取证的平台资产命名。

## 预期文件

- `.context/tasks/012-kernel-install-interaction-fix.md`
- `crates/narya-daemon/Cargo.toml`
- `crates/narya-daemon/src/official_release.rs`
- `crates/narya-daemon/src/installer.rs`
- `crates/narya-daemon/src/kernel.rs`
- `crates/narya-daemon/src/main.rs`
- `crates/narya-app/src/views/app_shell.rs`
- `crates/narya-contract-tests/src/lib.rs`

## 验收标准

- 无需预置隐藏目录或用户输入信任根即可触发官方安装。
- 下载来源固定在对应官方 GitHub 仓库，重定向仅接受 GitHub 发布资产域名。
- 解包只提取预期内核可执行文件，拒绝超限或缺失内容。
- 点击某行安装按钮时，其他行的同文案按钮不共享 hover/pressed 状态。
- 隔离 XDG 目录内实际安装的内核可执行并能输出版本。

## 验证

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- 变更 crate 定向 clippy
- `git diff --check`
- `ctx validate`
- 隔离 XDG 目录执行真实 daemon IPC 安装与内核 `version` 探针。

## 风险与回滚

- 风险：GitHub 页面或发布资产命名变化导致解析失败；失败时不写安装目标，并显示明确错误。
- 风险：部分上游只通过 GitHub HTTPS 提供工件而不发布独立签名；仅固定官方仓库和 GitHub 发布资产域名，并记录下载后二进制摘要。
- 回滚：回退本任务文件即可恢复上一安装实现；私有目录中的已安装文件可通过 UI 卸载。

## 完成记录

- 每个内核行操作按钮改用包含内核 ID 和动作的稳定 Liora `Button::id`，不再因同文案共享 GPUI 按下状态。
- daemon 新增固定官方 GitHub Release 解析器，当前 Linux `x86_64`/`aarch64` 可从 sing-box、mihomo、xray-core、v2ray-core 对应官方仓库下载预期资产。
- 下载仅接受 HTTPS 的官方 GitHub 发布页和 `release-assets.githubusercontent.com` 重定向，限制下载与解包大小；tar/zip 仅提取预期二进制。Xray/v2ray-core 校验上游 `.dgst` SHA2-256；其余内核记录解包后二进制 SHA-256 供启动前复验。
- 使用仓库 `target/narya-official-install-1788405254` 隔离 XDG 目录，通过实际 IPC 从 MetaCubeX/mihomo 官方 Release 安装 `mihomo 1.19.30`；`mihomo -v` 成功，私有 `sha256` 记录与实际二进制一致。
- 通过 `cargo check --workspace`、`cargo test --workspace`、daemon/kernel 与 app lib 定向 clippy、格式检查、`git diff --check`、ctx validate，以及 X11 `cargo run -p narya-app` 启动烟测。
