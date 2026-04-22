use anyhow::Result;
use std::path::PathBuf;
use sysinfo::{System, Pid, ProcessesToUpdate, ProcessRefreshKind, UpdateKind};
use async_trait::async_trait;
use api::tracker::{ConnectionMeta, AppIdentity, BypassRules};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: PathBuf,
}

#[async_trait]
pub trait ProcessTracker: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;

    // 核心流：根据源 IP 和端口极速获取连接元数据
    async fn lookup_connection(&self, src_ip: IpAddr, src_port: u16) -> Result<Option<ConnectionMeta>>;

    // UI 流：获取当前活跃的网络应用列表
    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>>;

    // 控制流：下发白名单/黑名单规则到系统底层
    async fn update_bypass_rules(&self, rules: &BypassRules) -> Result<()>;
    
    // 兼容旧接口的内部方法
    fn list_running_processes(&self) -> Result<Vec<ProcessInfo>>;
}

pub struct SystemProcessTracker {
    sys: std::sync::Mutex<System>,
}

impl SystemProcessTracker {
    pub fn new() -> Self {
        let sys = System::new();
        Self {
            sys: std::sync::Mutex::new(sys),
        }
    }

    fn get_refresh_kind() -> ProcessRefreshKind {
        ProcessRefreshKind::new()
            .with_exe(UpdateKind::Always)
    }
}

#[async_trait]
impl ProcessTracker for SystemProcessTracker {
    async fn start(&self) -> Result<()> {
        tracing::info!("SystemProcessTracker started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        tracing::info!("SystemProcessTracker stopped");
        Ok(())
    }

    async fn lookup_connection(&self, _src_ip: IpAddr, _src_port: u16) -> Result<Option<ConnectionMeta>> {
        // 基础实现暂时返回空
        Ok(None)
    }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        let processes = self.list_running_processes()?;
        Ok(processes.into_iter().map(|p| AppIdentity {
            name: p.name,
            identifier: p.pid.to_string(),
            icon_path: None,
        }).collect())
    }

    async fn update_bypass_rules(&self, _rules: &BypassRules) -> Result<()> {
        Ok(())
    }

    fn list_running_processes(&self) -> Result<Vec<ProcessInfo>> {
        let mut sys = self.sys.lock().unwrap();
        
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            Self::get_refresh_kind()
        );
        
        let processes = sys.processes().iter().map(|(pid, process)| {
            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().into_owned(),
                path: process.exe().map(|p| p.to_path_buf()).unwrap_or_default(),
            }
        }).collect();
        
        Ok(processes)
    }
}

pub struct MockProcessTracker;

#[async_trait]
impl ProcessTracker for MockProcessTracker {
    async fn start(&self) -> Result<()> { Ok(()) }
    async fn stop(&self) -> Result<()> { Ok(()) }
    
    async fn lookup_connection(&self, _src_ip: IpAddr, _src_port: u16) -> Result<Option<ConnectionMeta>> {
        Ok(None)
    }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        Ok(vec![
            AppIdentity {
                name: "browser".to_string(),
                identifier: "1001".to_string(),
                icon_path: None,
            },
            AppIdentity {
                name: "telegram".to_string(),
                identifier: "1002".to_string(),
                icon_path: None,
            },
        ])
    }

    async fn update_bypass_rules(&self, _rules: &BypassRules) -> Result<()> {
        Ok(())
    }

    fn list_running_processes(&self) -> Result<Vec<ProcessInfo>> {
        Ok(vec![
            ProcessInfo {
                pid: 1001,
                name: "browser".to_string(),
                path: PathBuf::from("/usr/bin/browser"),
            },
            ProcessInfo {
                pid: 1002,
                name: "telegram".to_string(),
                path: PathBuf::from("/usr/bin/telegram"),
            },
        ])
    }
}
