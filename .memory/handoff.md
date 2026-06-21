# Narya Handoff — 2026-06-21

## 当前可接手状态

本轮完成 Liora UI 重建的可运行主切片，并修复独立 review 提出的集成红线。启动命令：

```bash
cargo run -p narya-app
```

应用会直接进入主窗口，不再显示 splash。主窗口由 `crates/narya-app/src/views/app_shell.rs` 组合，组件边界在 `crates/narya-app/src/ui_kit.rs`。

## 关键文件

- `Cargo.toml` / `Cargo.lock`：GPUI 0.2.2 registry + Liora 0.1.5 依赖对齐。
- `crates/narya-app/src/lib.rs`：Liora 初始化和直接主窗口入口。
- `crates/narya-app/src/ui_kit.rs`：项目 Liora wrapper。
- `crates/narya-app/src/views/app_shell.rs`：当前主 UI 运行路径；连接按钮已接入 AppState；未实现动作显式禁用。
- `crates/narya-app/src/state.rs`：代理开关只在 daemon response 无 error 后更新状态；kernel install 不假成功。
- `crates/narya-app/src/ipc.rs`：`send_request` 等待匹配 `IpcResponse`，忽略先到 notification。
- `crates/narya-ipc/src/lib.rs`：per-user runtime socket/config path。
- `crates/narya-daemon/src/main.rs`：kernel status 通过 response 返回； InstallKernel fail-closed。
- `crates/narya-daemon/src/config_gen.rs`：sing-box config fail-closed；Shadowsocks `method:password`。
- `crates/narya-subscription/src/lib.rs`：Clash/SS URI 解析保留 Shadowsocks password 到 `method:password`。
- `crates/narya-contract-tests/src/lib.rs`：架构/红线契约测试。

## 必跑验证

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings
cargo clippy -p narya-app --lib -- -D warnings
timeout 8s cargo run -p narya-app
```

说明：不要直接要求 `cargo clippy --workspace --all-targets` 覆盖 `narya-app` test target；GPUI/Liora 宏在该路径会长时间卡住/有栈风险。已用 `narya-contract-tests` 替代 UI 架构测试。

## 最新验证结果

2026-06-21 已运行上述完整验证：fmt/check/test/分段 clippy 均退出 0；GUI run 保持运行直到 8 秒 timeout 截断；独立 code-reviewer 对 review blockers 最终 APPROVED。

## 下一步推荐

从 `ui/dashboard.png`、`ui/nodes.png`、`ui/subscriptions.png` 和 `ui/specs/main_window_spec_detailed.md` 开始做 Liora 组件细化。先抽 Sidebar/TopBar/StatusCard/NodeCard 等项目组件，再迁移旧页面逻辑，最后删除旧 raw GPUI 页面模块。真实 kernel installer、订阅添加/导入、测速、工具箱动作和 framed IPC codec 仍是后续重点。
