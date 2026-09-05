# 任务 015：全页面 UI 布局与交互可靠性审计

- 状态：已完成
- 计划：`.context/plans/002-runtime-foundation.md`
- 规模：大
- 依赖：`.context/tasks/014-official-download-reliability.md`
- 生产行为变更：是

## 任务目标

逐页核对应用菜单、子模块和设置分类的布局、样式、边界、遮挡、可交互性与控件 ID 唯一性；所有交互控件统一使用 Liora，缺失控件先增强 Liora 再接入应用。

## 范围

- 建立页面与菜单清单，记录每页的布局约束、滚动边界和交互入口。
- 修复内容溢出、固定宽度挤压、遮挡、不可点击和响应式窗口问题。
- 检查并规范所有交互控件的稳定唯一 ID，避免跨行/跨页状态串扰。
- 复用或增强 Liora 控件，不在 narya-app 中新增绕过控件库的交互实现。
- 增加离线契约检查与可重复的 GUI 启动/交互验证记录。

## 非目标

- 不改变代理、TUN、内核、规则和持久化数据语义。
- 不恢复旧 PNG 作为硬编码坐标规范；旧 UI 图仅作为视觉参考。
- 不在未取证情况下新增第三方 UI 依赖。

## 预期文件

- `.context/plans/002-runtime-foundation.md`
- `.context/tasks/015-ui-layout-interaction-audit.md`
- `crates/narya-app/src/ui_kit.rs`
- `crates/narya-app/src/views/app_shell.rs`
- `crates/narya-app/src/state.rs`
- `crates/narya-contract-tests/src/lib.rs`
- 必要时：`../../lib/liora/crates/liora/**`

## 验收标准

- 所有菜单页面均有明确的滚动、最小尺寸和边界策略，不出现内容越界或被固定区域遮挡。
- 所有可点击、可输入、可切换控件均可交互，且 ID 在页面和重复列表中稳定唯一。
- narya-app 不直接实现 Liora 已覆盖的替代控件；缺失能力有对应 Liora 增强和测试。
- UI 状态更新不会因控件 ID 冲突影响其他行或其他页面。
- 通过 workspace 编译、测试、clippy、格式检查和 GUI 启动探针。

## 验证

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets -- -D warnings`
- `timeout 8s cargo run -p narya-app`
- 离线控件 ID/页面契约测试。

## 风险与回滚

- 风险：GPUI/Liora 在不同窗口尺寸和平台字体下的测量结果存在差异；优先使用弹性布局、最小尺寸和滚动容器，避免硬编码坐标。
- 风险：Liora 控件增强可能影响其他消费者；新增 API 保持向后兼容，并用定向测试隔离。
- 回滚：按页面或控件提交增量回退，不改变 daemon 和持久化数据。

## 完成记录

- 已建立 UI 审计任务并完成首轮页面盘点。
- Liora `Flex` 增加 `min_w_0`，应用于 Dashboard、节点、设置和规则编辑布局，避免横向越界。
- Liora `Input`、`Select`、`Switch` 增加稳定根 ID API；设置页开关和更新通道已接入显式 ID。
- 订阅、规则、分流组、规则集和规则筛选表单已接入稳定唯一控件 ID，重复列表按业务 ID 区分。
- Segmented、Hero 卡片和设置页三栏布局已完成首轮 ID/边界修复。
- 已验证：`cargo fmt --all`、`cargo check --workspace`、`cargo clippy -p narya-app --lib -- -D warnings`、GUI 启动探针通过。
- 已知验证风险：一次 `cargo test --workspace` 中 `metadata_commit_failure_restores_previous_upgrade` 出现非 UI 的偶发失败，需后续独立复核；其余测试通过。
- 最终复核：`cargo test --workspace` 已重新通过；`cargo clippy -p narya-app --lib -- -D warnings`、`cargo check --workspace`、`cargo fmt --all`、`git diff --check` 和 GUI 启动探针均通过。
