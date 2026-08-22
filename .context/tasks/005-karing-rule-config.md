# 任务 005：Karing 对照的规则与 DNS 配置生成

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：大
- 依赖：`.context/tasks/004-proxy-tun-transaction.md`
- 生产行为变更：是

## 任务目标

把统一 `narya-rules` 语义编译为 sing-box 配置，并显式建模 Karing 对照中的 ruleset、分流组、DNS resolver/direct/proxy/outbound 与 TUN hijack 参数。

## 范围

- 规则 AST 到 sing-box route/dns/tun JSON 的确定性编译。
- 规则集来源、版本和校验摘要模型。
- system proxy 与 TUN 共用匹配顺序的 golden tests。
- Karing 对照提交与 sing-box 官方文档引用记录。

## 非目标

- 不复制 Karing 代码或远程规则文件。
- 不在没有校验摘要的情况下自动下载规则集。
- 不实现 mihomo/xray 配置编译，先明确能力缺口。

## 预期文件

- `crates/narya-rules/src/lib.rs`
- `crates/narya-daemon/src/config_gen.rs`
- `crates/narya-core/src/lib.rs`
- 规则与配置测试、上下文证据

## 验收标准

- 同一规则集生成的 system proxy/TUN route 顺序一致。
- DNS 路径不默认为 direct；resolver、direct、proxy、outbound 目标显式可见。
- 未支持的条件或内核能力 fail-closed，并带规则 ID/能力信息。
- golden fixtures 可离线重放并验证配置无污染字段。

## 完成记录

- `narya-rules` 增加可序列化 `RuleSet`、规则集来源元数据和 SHA-256/版本完整性校验。
- `narya-daemon::config_gen` 增加统一 `RoutingConfig`：system proxy/TUN 共用排序后的 route AST，TUN 显式生成 `auto_route`、`strict_route`、排除路由和 DNS 劫持。
- resolver/direct/proxy/outbound DNS 独立生成，DNS 动作进入 `dns.rules`，未匹配流量使用 `route.final=block`，禁止静默 direct fallback。
- 未支持的出站、DNS 条件、规则集字段和模式不一致均带规则 ID/能力信息 fail-closed。
- sing-box、mihomo、xray-core 三个适配器共享路由计划校验；TUN 计划不会被意外带入 system proxy，模式不一致在生成配置前拒绝。
- 离线测试覆盖 system proxy/TUN 规则顺序、DNS 路径、TUN 参数、规则集摘要和错误路径。

## 验证

```bash
cargo fmt --all -- --check
cargo test -p narya-rules -p narya-daemon
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 风险与回滚

- 风险：Karing 上游和内核配置 schema 变化；锁定提交、版本和 fixture 摘要。
- 回滚：保留现有 Shadowsocks fail-closed 生成器，不启用未经 golden 验证的新路径。
