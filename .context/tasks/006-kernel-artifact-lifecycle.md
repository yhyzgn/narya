# 任务 006：可信内核安装、升级与注册

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：中
- 依赖：`.context/tasks/003-framed-ipc-kernel-registry.md`
- 生产行为变更：是

## 任务目标

为 sing-box/mihomo/xray 建立可观察、可回滚的内核安装与升级入口，禁止未校验工件进入运行目录。

## 范围

- 本地绝对路径、`file://` 和 HTTPS 工件来源。
- SHA-256 校验、临时文件、原子替换、版本记录和注册表恢复。
- `InstallKernel` / `UpgradeKernel` framed IPC。

## 非目标

- 本阶段不实现签名信任根、官方发布清单或自动选择镜像。
- 不在本阶段实现 mihomo/xray 配置编译和跨平台 TUN。

## 预期文件

- `crates/narya-daemon/src/installer.rs`
- `crates/narya-daemon/src/kernel.rs`
- `crates/narya-daemon/src/main.rs`
- `crates/narya-kernel/src/lib.rs`
- `crates/narya-ipc/src/lib.rs`

## 验收标准

- 校验失败不覆盖旧二进制。
- 安装、升级、运行、健康和失败状态可区分。
- daemon 重启后能发现托管内核和版本。
- 安装/升级期间有活动内核时明确拒绝。

## 完成记录

- `InstallKernel` / `UpgradeKernel` 接受显式内核、版本、来源和 SHA-256。
- 来源仅允许本地绝对路径、`file://` 或 HTTPS；缺少摘要、摘要不合法、下载失败或摘要不匹配都会拒绝。HTTPS 工件还必须提供 Ed25519 签名和公钥，daemon 在替换前验证签名。
- 工件先写入临时文件、校验并设置可执行权限，再在同一目录原子替换；校验失败不会覆盖旧内核。
- 安装目录使用 `XDG_DATA_HOME/narya/kernels`，daemon 启动时重新发现托管内核并恢复版本状态。
- 安装、升级状态独立于运行和健康状态；运行期间禁止安装/升级，升级未安装内核也会拒绝。
- framed IPC 烟测验证了安装成功、状态可见和错误摘要拒绝。

## 后续边界

- 当前仍需引入签名/公钥信任根和官方发布清单。
- mihomo/xray 配置编译、跨平台 TUN backend 和内核切换编排在后续任务实现。

## 验证

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 风险与回滚

- 风险：SHA-256 只能证明传输内容一致，尚不能证明发布者身份。
- 回滚：原子替换前保留旧二进制；校验或写入失败时状态回到已安装或失败。
