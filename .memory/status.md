# 当前状态

## 更新时间

2026-06-21

## 当前阶段

Liora UI 重建切片已完成并经复核：Narya 已从旧的 splash-first GPUI 入口切换为直接打开主窗口，并接入已发布的 Liora 0.1.5 GPUI 组件库。当前主窗口运行路径通过项目本地 `ui_kit` 封装组合 Liora `Flex/Card/Button/Tag/Progress/Text/Space`，保留现有核心、IPC、订阅解析、daemon/kernel 相关可用逻辑。

## 本次完成

1. **启动行为**：删除运行面 splash，`narya-app` 启动时调用 `liora::init_liora(cx)`，随后直接 `AppShell::open(cx)`。
2. **依赖对齐**：将工作区 GPUI 对齐到 Liora 使用的 registry `gpui = 0.2.2`，移除旧 `gpui_platform` 入口，改用 `gpui::Application::new()`。
3. **项目组件边界**：新增 `crates/narya-app/src/ui_kit.rs`，集中封装 Narya 专属 `NaryaPage/NaryaCard/NaryaButton/NaryaMetric` 和状态标签/进度条工具，避免页面直接散落设计系统细节。
4. **Liora 主窗口**：重写 `crates/narya-app/src/views/app_shell.rs`，用 Liora 组件组合侧边栏、顶部栏、底部状态栏和 Dashboard/Nodes/Subscriptions/Config/Connections/Rules/Logs/Tools/Settings 主要内容面。
5. **回归契约**：新增 `crates/narya-contract-tests`，用不依赖 GPUI test harness 的静态契约测试锁定：无 splash、Liora 初始化、GPUI 版本对齐、项目组件边界、关键红线修复。
6. **红线修复**：主 UI 连接按钮已接入 `AppState::toggle_proxy/connect_node`；IPC request 会忽略先到通知直到匹配 response；daemon `GetKernelStatus` 不再先发通知；`InstallKernel` app/daemon 双侧 fail-closed；运行时 socket/config 改到 per-user runtime dir；sing-box config 生成 unsupported protocol fail-closed，Shadowsocks 明确拆分 `method:password`，不再使用假密码或 direct proxy fallback；app 只有在 daemon response 无 error 时才更新 connected state。
7. **旧运行面隔离**：旧 raw 页面文件仍保留为迁移参考，但已从 `views/mod.rs` 移出编译面；当前编译/运行 UI 面只保留 Liora AppShell。

## 验证证据

最新验证命令均已运行：

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings
cargo clippy -p narya-app --lib -- -D warnings
timeout 8s cargo run -p narya-app
```

结果：fmt/check/test/分段 clippy 均退出 0；`cargo test --workspace` 包含 `narya-contract-tests` 4/4 通过，以及 daemon config/subscription Shadowsocks 回归测试通过；`cargo run -p narya-app` 成功编译并启动 GUI 进程，保持运行直到 8 秒 timeout（退出 124 为预期烟测截断）。独立 code-reviewer 复核 review blockers 后 APPROVED。

## 已知限制

- `cargo clippy --workspace --all-targets` 若包含 `narya-app` test target，会在 GPUI/Liora 宏 test 编译路径长时间卡住/栈风险；已通过 `narya-app` `[lib] test = false`、独立 `narya-contract-tests` 和 `cargo clippy -p narya-app --lib` 规避并记录。
- 新主窗口已使用 Liora 组件搭建可运行产品面，但尚未完成逐 PNG 的截图级 1:1 视觉验收。
- 真实内核安装器、真实测速、订阅添加/导入、配置编辑、工具箱动作仍未实现；相关按钮已禁用或保留为后续入口，避免假成功。
