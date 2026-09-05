# Narya

Narya 是基于 Rust、GPUI 与 Liora 的桌面代理控制端。应用通过有长度前缀的本地 IPC 与 `narya-daemon` 通信，由 daemon 统一管理可扩展的代理内核、配置、健康检查及路由模式。

## 当前能力

- 当前内置四种可执行内核：sing-box、mihomo/Clash Meta、xray-core、v2ray-core；注册表和安装清单可继续扩展其他内核。
- 节点协议与可执行内核分离：Shadowsocks（SS）可由三种内核运行；sing-box/mihomo 另支持 Hysteria2、VMess、VLESS、Trojan；xray-core 支持 VMess、VLESS、Trojan。
- 统一规则模型、分流组、外部规则集、DNS 路径及 fail-closed 默认出口。
- SHA-256 与 Ed25519 校验的内核工件、签名发布清单和规则集缓存。
- daemon 健康确认后才显示连接成功；IPC 断开不会生成模拟连接或流量数据。

## 运行要求

- Rust stable 工具链。
- Linux 图形环境及 GPUI 所需的 X11/Wayland 系统库。
- 至少一个受支持的代理内核；也可以在应用设置页通过已验证工件安装。
- Linux 系统代理使用 GNOME `gsettings`。TUN 模式还要求 `/dev/net/tun`、`iproute2` 与相应权限。

Windows 和 macOS 的系统代理/TUN backend 尚未开放，调用会明确失败，不会回退到不受控的 direct 模式。

## 开发运行

直接启动应用即可；应用会优先复用同一用户运行目录中的 daemon，缺失时自动启动与应用同目录的 `narya-daemon`。也可以手动分开启动：

```bash
cargo run -p narya-daemon
```

再启动桌面应用：

```bash
cargo run -p narya-app
```

daemon 启动本身不会修改系统代理、DNS 或 TUN；只有用户发起并通过校验的路由模式切换才会应用平台副作用。

## 验证

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
RUST_MIN_STACK=134217728 cargo clippy --workspace --all-targets --exclude narya-app -- -D warnings
RUST_MIN_STACK=134217728 cargo clippy -p narya-app --lib -- -D warnings
cargo build --release --workspace
```

GUI 烟测：

```bash
timeout 8s cargo run -p narya-app
```

退出码 `124` 仅表示到时主动结束，必须同时确认进程启动日志中没有 panic。

## 安全边界

- HTTPS 内核工件必须匹配本地固定信任根验证过的发布清单。
- 规则集必须在 daemon 中完成摘要和签名验证后才会进入内核配置。
- 未支持的内核、协议、规则条件和平台能力会返回错误，不会静默直连。

## License

MIT
