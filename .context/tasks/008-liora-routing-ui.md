# 任务 008：Liora 迁移与真实分流配置工作台

- 状态：进行中
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：中
- 依赖：`.context/tasks/007-runtime-routing-modes.md`
- 生产行为变更：是

## 任务目标

接管遗留 GPUI 应用，将 UI 依赖迁移到本地 Liora 0.3.0 源码，并让规则配置、运行模式和 daemon 实际状态保持同一条数据链路。

## 范围

- Narya 应用 GPUI/Liora 依赖、规则状态和规则工作台。
- daemon 启动请求中的规则编译与运行模式确认。

## 非目标

- 本任务不实现 Windows/macOS TUN backend，也不复制 Karing 源码。
- 本任务不引入 Narya 自绘替代 Liora 的基础控件。

## 预期文件

- `Cargo.toml`、`Cargo.lock`、`crates/narya-app/Cargo.toml`
- `crates/narya-app/src/lib.rs`、`state.rs`、`views/app_shell.rs`
- `crates/narya-daemon/src/main.rs`

## 已完成

- 锁定 GPUI 与 Liora 0.3.0 兼容 revision，使用 Liora 本地资源构建。
- `AppState` 持久化规则，提供默认 fail-closed 规则、新增/删除/搜索和系统代理/TUN 目标模式。
- daemon `StartKernel` 接收规则列表，先通过 `RuleSet::compile` 校验，再生成 sing-box 路由/DNS 配置。
- 设置页使用 Liora 表单控件提交内核工件，展示真实安装/升级进度与错误；支持 SHA-256 和 HTTPS Ed25519 签名字段。
- 规则模型增加外部规则集条件和 selector/urltest/fallback/load-balance 分流组；sing-box、mihomo、xray-core 具备独立配置适配，缺失能力显式拒绝。
- 规则页使用 Liora `Input`/`Select`/`Button` 管理本地规则集 ID、版本、绝对路径/file URL 和 SHA-256；导入前校验格式与重复 ID，删除被规则引用的规则集会被拒绝并提示。
- 规则页使用 Liora 控件编辑每条规则的多条件 AND（域名、后缀、CIDR、端口、进程、规则集、Any），编辑分流组成员、策略、URL 测试地址和间隔，并支持经过跨引用校验的 JSON 配置导入/导出。
- daemon 离线时 UI 不模拟速度和连接状态；连接状态需内核健康与路由模式 IPC 成功确认。

## 未完成与验收标准

- 内核设置页仍需接入签名公钥的官方发布清单和版本选择器。
- 分流规则编辑仍需覆盖 Karing 风格的规则集远程下载、规则集启停/更新和更多 geosite/geoip/ACL 语义；当前阶段已完成本地规则集、AND 条件、分流组编辑和配置导入导出。
- 需要为 Liora 增强可复用规则编辑器控件时，修改 `../../lib/liora/crates/liora-components`，不得在 Narya 内自绘替代控件。
- system proxy 与 TUN 需在 Linux 实机通过 DNS 泄漏、污染和断开恢复探针。

## 验收标准

- `narya-app` 使用本地 Liora 0.3.0，并通过 GPUI 平台入口启动。
- 规则新增、删除、搜索、动作、优先级和目标模式操作更新持久化状态。
- daemon 拒绝无效规则，且启动配置不使用隐式 direct fallback。
- daemon 离线或 IPC 错误时 UI 不报告连接成功。

## 验证

## 验证记录

```bash
cargo fmt --all -- --check
cargo test --workspace
RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings
RUST_MIN_STACK=134217728 cargo clippy -p narya-app --lib -- -D warnings
python /home/neo/.codex/skills/ctx/scripts/context_bootstrap.py validate --root .
git diff --check
```

## 风险与回滚

- 风险：Liora 0.3.0 与上游 GPUI revision 必须保持锁定；更换任一版本需重新验证资源和生命周期 API。
- 风险：规则编辑器目前是本地规则工作台，规则集远程订阅和分流组编排仍未完成。
- 回滚：回退本任务提交即可恢复到已验证的 runtime routing 阶段；不回退 daemon 的 fail-closed 校验。

## 完成记录

- 已完成依赖迁移、规则状态链路、真实 IPC 连接状态和基础 Liora 规则工作台。
- 任务保持进行中，待可信工件 UI、规则集/分流组和实机流量探针完成后关闭。
