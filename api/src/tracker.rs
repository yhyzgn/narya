use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMeta {
    pub protocol: Protocol,
    pub src_ip: IpAddr,
    pub src_port: u16,
    pub dst_ip: IpAddr,
    pub dst_port: u16,
    // 桌面端标识
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub process_path: Option<PathBuf>,
    // 移动端标识
    pub package_name: Option<String>, // Android
    pub bundle_id: Option<String>,    // iOS/macOS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppIdentity {
    pub name: String,
    pub identifier: String, // PID, PackageName 或 BundleID
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BypassRules {
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}
