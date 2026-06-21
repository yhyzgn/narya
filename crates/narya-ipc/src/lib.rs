use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KernelInfo {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub running: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum IpcNotification {
    TrafficUpdate { down: f32, up: f32 },
    StatusUpdate { running: bool },
    LogLine { level: String, message: String },
    KernelStatusUpdate { kernels: Vec<KernelInfo> },
}

pub fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("TMPDIR").map(|tmp| {
                let mut path = PathBuf::from(tmp);
                path.push(format!("narya-{}", runtime_owner()));
                path
            })
        })
        .unwrap_or_else(|| {
            let mut path = std::env::temp_dir();
            path.push(format!("narya-{}", runtime_owner()));
            path
        })
        .join("narya")
}

fn runtime_owner() -> String {
    std::env::var("UID")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "current".to_string())
}

pub fn socket_path() -> PathBuf {
    runtime_dir().join("narya.sock")
}

pub fn kernel_config_path() -> PathBuf {
    runtime_dir().join("kernel.json")
}
