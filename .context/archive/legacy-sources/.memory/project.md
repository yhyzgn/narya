# Narya 项目简介

Narya 是一款跨平台 PC + App 端全能 VPN / 代理客户端，底层支持切换 `sing-box`、`mihomo`、`xray-core` 等内核，提供系统代理、TUN、可视化规则与实时流量监控等高级功能。

## 技术路线

完全采用 Rust 单栈开发：
- **UI 层**：GPUI (原生 GPU 加速界面框架)
- **后端层**：Rust Daemon (Control Plane) + sidecar 模式调用底层代理内核。

## 核心指导原则

- **视觉真源**：所有的页面布局、颜色、卡片和特效（包括复杂的玻璃拟态），必须严格对照 `ui/*.png` 和 `ui/specs/*.spec.md` 规范。在 GPUI 中 100% 手工构建元素树，无妥协。
- **单向数据流**：以 GPUI 的 Model-View 模式结合 async Rust 管理网络状态，避免多进程 IPC 负担。
