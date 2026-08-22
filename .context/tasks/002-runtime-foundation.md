# 任务 002：建立统一分流规则契约

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：中
- 依赖：`.context/tasks/001_context_bootstrap.md`
- 生产行为变更：是（仅新增纯规则领域契约，暂不切换 daemon 默认路径）

## 任务目标

将 `narya-rules` 从占位函数升级为可序列化、可排序、可解释的统一分流规则模型，为 system proxy 与 TUN 共用编译输入。

## 范围

- 规则条件、动作、优先级、命中解释和确定性排序。
- 纯内存匹配 API 与 JSON/YAML 友好序列化。
- 规则编译能力错误的 fail-closed 表达。
- 针对域名、IP、端口、进程与兜底规则的离线测试。

## 非目标

- 本任务不修改 UI 页面和系统代理命令。
- 本任务不下载或启动任何真实内核。
- 本任务不实现平台 TUN 驱动。

## 预期文件

- 修改 `crates/narya-rules/Cargo.toml`、`crates/narya-rules/src/lib.rs`。
- 视引用需要更新 `crates/narya-core` 与契约测试。

## 验收标准

- 规则可从 JSON/YAML 反序列化并保持稳定优先级。
- 同一请求在规则列表中得到唯一、可解释的首个匹配动作。
- 无匹配且无显式兜底时返回错误，不静默 direct。
- 单元测试覆盖匹配、排序、无效规则和 fail-closed 行为。

## 验证

```bash
cargo fmt --all -- --check
cargo test -p narya-rules
cargo test --workspace
cargo clippy -p narya-rules --all-targets -- -D warnings
```

## 风险与回滚

- 风险：未来内核能力差异可能要求扩展条件；通过不破坏性枚举扩展和能力错误隔离。
- 回滚：删除本任务新增规则模块，保留上下文文件；不触碰 daemon 默认行为。

## 完成记录

- 状态：已完成。
- 实现：`narya-rules` 新增 `Rule`、`Condition`、`Action`、`RequestContext`、`Decision`、`RuleSet` 与 `RuleError`。
- 行为：规则按 priority/id 确定性排序；同一规则的多个条件全部满足才命中；域名后缀按 label 边界匹配；CIDR 校验与 IPv4/IPv6 匹配；无命中返回 `NoMatch`，不回退 direct。
- 验证：`cargo fmt --all`、`cargo test -p narya-rules`（6 passed）、`cargo check --workspace`、分段 `cargo clippy`（均通过）、`cargo test --workspace`（全部通过）。
- 未覆盖：daemon 尚未消费该规则 AST；内核安装、framed IPC、system proxy/TUN 事务将在后续任务实现。
