# Narya Phase 2: 跨平台进程溯源与流量控制 (Smart TUN) 详细设计

## 1. 目标与背景

根据 V7.0 架构设计，Narya 需要实现零鬼畜、极低学习成本的“白名单/黑名单”分流机制。这要求系统必须在数据包进入代理协议栈之前，精确识别出流量所属的进程（桌面端）或应用（移动端），并在内核态/系统底层实现极速拦截或放行。

本设计文档旨在定义跨平台（Linux, Windows, macOS, Android, iOS）的统一进程溯源抽象层及各平台的具体实现方案。

## 2. 核心抽象设计 (Rust Trait)

为了隔离平台差异，我们在 `daemon/src/tracker.rs` (或抽象出一个新 crate `narya-tracker`) 中定义统一的溯源接口。

### 2.1 统一连接元数据
```rust
pub struct ConnectionMeta {
    pub protocol: Protocol,
    pub src_ip: IpAddr,
    pub src_port: u16,
    pub dst_ip: IpAddr,
    pub dst_port: u16,
    // 桌面端标识
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    // 移动端标识
    pub package_name: Option<String>, // Android
    pub bundle_id: Option<String>,    // iOS/macOS
}
```

### 2.2 核心 Trait
```rust
#[async_trait]
pub trait ProcessTracker: Send + Sync {
    // 初始化追踪器
    async fn start(&self) -> Result<()>;
    // 停止追踪
    async fn stop(&self) -> Result<()>;
    
    // 核心流：根据源 IP 和端口获取连接元数据（命中极速缓存或内核 Map）
    async fn lookup_connection(&self, src_ip: IpAddr, src_port: u16) -> Result<Option<ConnectionMeta>>;
    
    // UI 流：获取当前活跃的网络应用列表，供拖拽面板使用
    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>>;
    
    // 控制流：下发白名单/黑名单规则到系统底层
    async fn update_bypass_rules(&self, rules: &BypassRules) -> Result<()>;
}
```

## 3. 平台特异性实现方案

### 3.1 Linux (基于 Aya eBPF + cgroup v2)
*   **技术栈**：`aya`, `aya-bpf` (Nightly Rust 编译 eBPF 字节码)。
*   **实现方案（混合动力）**：
    *   在 `cgroup v2` 的 `connect` 和 `sendmsg` 钩子挂载 `SockAddr` 程序，拦截连接，查 BPF Map (规则表)。若在黑名单直接阻断 (`EPERM`)，若需代理则透明重定向。
    *   在 cgroup 挂载 `CgroupSkb` 程序进行流量包级统计。
*   **交互**：用户态 Daemon 通过 Aya 加载 ELF，并实时更新 BPF Maps 从而下发白名单。

### 3.2 Windows (基于 ETW)
*   **技术栈**：`windows-rs`。
*   **实现方案**：
    *   启动 ETW (Event Tracing for Windows) 会话，监听 `Microsoft-Windows-TCPIP` 和 `Microsoft-Windows-Kernel-Network` Provider。
    *   实时捕获 `TcpIp/Connect` 和 `UdpIp/Send` 事件，将 `(源IP:端口) -> PID` 存入基于 LRU 的内存缓存表。
    *   通过 `QueryFullProcessImageNameW` 获取进程路径。

### 3.3 macOS (NetworkExtension / EndpointSecurity)
*   **技术栈**：FFI 调用 Apple SDK。
*   **实现方案**：
    *   桌面端优先使用苹果官方 `NetworkExtension` 框架（系统扩展）的 `NEFilterDataProvider`，系统会在回调中直接提供 `NEFilterFlow`，其中包含 `sourceAppUniqueIdentifier` (Bundle ID)。

### 3.4 Android (VPN Service + JNI)
*   **技术栈**：JNI, Android SDK API 29+。
*   **实现方案**：
    *   Android UI 启动 `VpnService`，将所有流量路由至 Rust 创建的 TUN fd。
    *   Rust 从 TUN 读取到包后，提取源端口，通过 JNI 回调 Java 层执行 `VpnService.getConnectionUid(protocol, src_ip, src_port)`。
    *   Java 层拿到 UID 后，通过 `PackageManager.getPackagesForUid()` 获取应用的 `package_name` (如 `com.whatsapp`)，返回给 Rust 层。

### 3.5 iOS (NEPacketTunnelProvider + FFI)
*   **技术栈**：C-ABI FFI。
*   **实现方案**：
    *   iOS 的 VPN 扩展进程中运行 Rust 静态库。
    *   利用苹果提供的 `NEAppProxyProvider` 截获应用层连接，或者在 Packet Tunnel 中利用内核下发的元数据，提取 `Bundle ID` 映射。

## 4. 整体数据流向 (TUN 分流)

1.  **物理网卡/应用** 发出流量。
2.  进入 **Sing-box TUN (Dummy Interface)** 前，流量被 Rust `Smart TUN Controller` 拦截。
3.  控制器提取五元组，调用对应平台的 `ProcessTracker::lookup_connection()`。
4.  拿到 `ConnectionMeta` (含包名/PID)。
5.  送入 **Rust 分流引擎**，匹配 `Domain / IP-CIDR / PackageName` 等规则。
6.  根据结果返回 `Direct` (物理网卡直出) / `Proxy` (送入 Sing-box 协议栈) / `Reject`。
7.  **GUI 面板**：通过 IPC 订阅 `ProcessTracker::list_network_apps()`，实时展示有网络活动的 APP 列表供用户拖拽。

## 5. 阶段实施计划

*   **Step 1**：创建通用抽象 Trait 与数据结构 (`daemon/src/tracker.rs`)。
*   **Step 2**：搭建 Linux eBPF 开发环境 (创建 `narya-ebpf` crate，配置 Nightly Toolchain)。
*   **Step 3**：实现 Linux eBPF `CgroupSkb` / `SockAddr` 核心追踪代码。
*   **Step 4**：实现 Windows ETW 追踪代码。
*   **Step 5**：定义移动端 FFI/JNI 桥接接口头文件。
