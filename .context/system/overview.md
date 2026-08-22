# 系统全景：narya

## 已验证事实

- 语言与构建：Rust 2021 workspace，根清单为 `Cargo.toml`，成员由 `crates/*` 扫描加入。
- UI：`crates/narya-app` 使用 GPUI 0.2.2（锁定 Zed revision）与本地 `../../lib/liora` 0.3.0 源码；启动入口为 `src/main.rs` -> `narya_app::run()`，应用初始化在 `crates/narya-app/src/lib.rs`。页面控件通过 Liora `Button`、`Input`、`Select`、`Segmented`、`Switch`、`NavigationMenu` 组合。
- 领域模型：`crates/narya-core/src/lib.rs` 目前仅有 `Node`、`Subscription` 等基础结构。
- 控制面：`crates/narya-daemon/src/main.rs` 通过 Unix socket 接收 JSON IPC；`crates/narya-ipc/src/lib.rs` 定义请求、响应、通知和运行目录。
- 内核：`crates/narya-daemon/src/kernel.rs` 管理单个活动子进程，并通过 `installer.rs` 支持本地/HTTPS 内核安装和升级；所有工件必须有 SHA-256，HTTPS 工件还需匹配本地固定 Ed25519 信任根验证过的发布清单（内核、版本、平台、架构、来源、摘要、工件签名/公钥）；注册表区分安装、运行和健康状态，启动健康还要求生成配置中的本地 HTTP/SOCKS 监听可连接。
- 代理：`crates/narya-daemon/src/proxy.rs` 支持 Linux GNOME gsettings 事务和 Linux TUN 前置检查；macOS/Windows 完整 backend 仍未实现。
- 配置：`crates/narya-daemon/src/config_gen.rs` 以统一 `RoutingConfig` 生成 sing-box、mihomo、xray-core 配置；system proxy/TUN 共用规则语义，DNS resolver/direct/proxy/outbound、分流组和 TUN 参数显式生成，未匹配流量 block。
- 规则：`crates/narya-rules/src/lib.rs` 提供可序列化 `RuleSet`、确定性优先级排序、规则集来源版本/SHA-256/Ed25519 元数据、持久化启停和 fail-closed 决策；daemon 通过规则集缓存管理器下载、验证并原子缓存 HTTPS 源，`StartKernel` 只消费启用且启动前复验的缓存，再编译到 sing-box、mihomo、xray-core；Liora 规则页支持搜索、新增、删除、多条件 AND、目标模式、分流组编辑、规则集启停、本地/HTTPS 规则集导入和 JSON 配置导入导出。
- 测试：`crates/narya-contract-tests` 是源码契约测试；各 crate 另有少量单元测试。测试不应连接真实共享基础设施。
- 外部依赖：仓库扫描未发现数据库、缓存或消息队列；`narya-subscription` 依赖 `reqwest`，真实网络访问需由明确测试场景隔离。

## 当前未知项

- 本地 `../../lib/liora` 0.3.0 与 crates.io 包的资源内容不完全一致（本地包含查询高亮资源），因此应用使用 path 依赖；GPUI 通过同一 Zed revision patch 保持 API/lifetime 一致。
- Karing 的分流组、规则集订阅和平台 TUN 权限仍需进一步源码级对照；当前已落地统一规则 AST、DNS 分离和 TUN 路由参数，后续需要内核能力矩阵与 golden 测试。
- 系统代理恢复已覆盖 Linux，TUN 生命周期由 sing-box inbound + daemon 模式互斥管理；签名信任根、mihomo/xray 配置编译和跨平台安装策略尚未实现。

## Karing 对照证据（2026-08-22）

- 已对照 Karing 仓库提交 `ae12111876a4456cc58c7410950428345f908abb`（临时只读 clone）。其 `lib/app/modules/server_manager.dart` 将 `rule_set_items`、分流组和 DNS 服务器作为独立配置输入；`setting_manager.dart` 明确区分 TUN `auto_route`、`strict_route`、`hijack_dns`、路由排除地址，以及系统代理 bypass domain。
- Karing 的设计证据支持本项目采用“统一规则语义 + 独立 DNS/路由/TUN 参数 + system proxy bypass”模型；不能只把系统代理开关映射成一个布尔值。
- 该上游证据仅用于设计对照，不复制其代码；接入具体内核前仍需锁定各内核官方配置版本并编写 golden tests。
