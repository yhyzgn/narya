# 任务 011：官方目录驱动的内核管理

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：中
- 依赖：`.context/tasks/010-settings-kernel-management.md`
- 生产行为变更：是

## 任务目标

设置页只展示可验证的内核列表；安装和升级由 daemon 根据已验证的官方发布目录选择当前平台工件，UI 不再接收或展示来源、摘要、签名和信任根输入。

## 范围

- 移除设置页“可信工件”和“官方发布清单”表单。
- 新增 `InstallOfficialKernel`、`UpgradeOfficialKernel` IPC 契约，仅接受 `KernelId`。
- daemon 从本地已验证目录按平台、架构选择工件，并保留 SHA-256/Ed25519 校验。
- 官方工件来源必须为 HTTPS；目录条目来源按内核官方发布站点白名单校验。
- 仅增加已有配置生成和健康检查能力覆盖的内核标识，不制造不可运行的假条目。

## 非目标

- 不把 Shadowsocks、VMess、VLESS 等节点协议当作独立可执行内核。
- 不写入宿主机配置，不使用系统 PATH 中的程序。
- 不伪造版本、摘要、公钥、签名或官方 URL。

## 预期文件

- `.context/tasks/011-official-kernel-catalog.md`
- `crates/narya-daemon/src/kernel_catalog.rs`
- `crates/narya-daemon/src/kernel.rs`
- `crates/narya-daemon/src/main.rs`
- `crates/narya-app/src/state.rs`
- `crates/narya-app/src/views/app_shell.rs`
- `crates/narya-contract-tests/src/lib.rs`

## 验收标准

- 设置页不再渲染“可信工件”或“官方发布清单”及其输入控件。
- 安装/升级请求不携带任意来源或签名字段，daemon 拒绝未匹配已验证目录的工件。
- 目录工件只允许 HTTPS 官方发布站点，校验失败时不产生安装文件。
- 现有三种可运行内核的安装、升级、卸载和切换行为不回归。

## 验证

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `python /home/neo/.codex/skills/ctx/scripts/context_bootstrap.py validate --root "$PWD"`
- 使用仓库 `target/` 隔离 XDG 目录进行 X11 `cargo run -p narya-app` 启动烟测。

## 风险与回滚

- 风险：官方目录未配置或没有当前平台条目时，安装会明确失败；不回退到任意 URL。
- 风险：官方发布站点路径随上游变化；由目录签名和来源白名单共同约束，更新目录即可调整。
- 回滚：回退本任务源码和上下文文件；不触碰已安装私有内核文件。

## 完成记录

- 已移除设置页可信工件与官方发布清单表单，安装/升级仅通过 `InstallOfficialKernel`/`UpgradeOfficialKernel` 选择内置内核标识。
- daemon 从本地已验证签名目录按平台架构选择最新官方 HTTPS 工件，并执行来源白名单、摘要和 Ed25519 校验。
- 内核列表增加 `v2ray-core`，复用已存在的 Xray 配置适配器；未将 Shadowsocks 等节点协议误建模为内核。
- 通过 workspace 测试、clippy、格式检查、diff 检查和 ctx validate。
