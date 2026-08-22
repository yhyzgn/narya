# 任务 005：Karing 对照的规则与 DNS 配置生成

- 状态：待开始
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

## 完成记录

- 待开始。
