use anyhow::Result;
use api::tracker::{AppIdentity, BypassRules, ConnectionMeta};
use async_trait::async_trait;
use narya_ebpf_common::ProcessConfig;
use std::net::IpAddr;
use std::path::PathBuf;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

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
    async fn lookup_connection(
        &self,
        src_ip: IpAddr,
        src_port: u16,
    ) -> Result<Option<ConnectionMeta>>;

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
        ProcessRefreshKind::new().with_exe(UpdateKind::Always)
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

    async fn lookup_connection(
        &self,
        _src_ip: IpAddr,
        _src_port: u16,
    ) -> Result<Option<ConnectionMeta>> {
        // 基础实现暂时返回空
        Ok(None)
    }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        let processes = self.list_running_processes()?;
        Ok(processes
            .into_iter()
            .map(|p| AppIdentity {
                name: p.name,
                identifier: p.pid.to_string(),
                icon_path: None,
            })
            .collect())
    }

    async fn update_bypass_rules(&self, _rules: &BypassRules) -> Result<()> {
        Ok(())
    }

    fn list_running_processes(&self) -> Result<Vec<ProcessInfo>> {
        let mut sys = self.sys.lock().unwrap();

        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, Self::get_refresh_kind());

        let processes = sys
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().into_owned(),
                path: process.exe().map(|p| p.to_path_buf()).unwrap_or_default(),
            })
            .collect();

        Ok(processes)
    }
}

pub struct EbpfProcessTracker {
    bpf: std::sync::Arc<tokio::sync::Mutex<Option<aya::Bpf>>>,
    system: SystemProcessTracker,
}

impl EbpfProcessTracker {
    pub fn new() -> Self {
        Self {
            bpf: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            system: SystemProcessTracker::new(),
        }
    }
}

#[async_trait]
impl ProcessTracker for EbpfProcessTracker {
    async fn start(&self) -> Result<()> {
        self.system.start().await?;
        #[cfg(target_os = "linux")]
        {
            tracing::info!("Loading eBPF programs on Linux (Mocked for now)...");
            // let mut bpf = aya::Bpf::load(include_bytes!("../../target/bpfel-unknown-none/release/narya-ebpf"))?;
            tracing::info!("eBPF programs would be loaded and attached to root cgroup here");
        }
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.system.stop().await?;
        let mut bpf_lock = self.bpf.lock().await;
        *bpf_lock = None; // 销毁 Bpf 对象会自动卸载程序
        Ok(())
    }

    async fn lookup_connection(
        &self,
        _src_ip: IpAddr,
        _src_port: u16,
    ) -> Result<Option<ConnectionMeta>> {
        // 从 BPF Map 中读取溯源信息
        Ok(None)
    }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        self.system.list_network_apps().await
    }

    async fn update_bypass_rules(&self, rules: &BypassRules) -> Result<()> {
        tracing::info!("Backend received bypass rules update: {:?}", rules);

        #[cfg(target_os = "linux")]
        {
            let mut bpf_lock = self.bpf.lock().await;
            if let Some(ref mut bpf) = *bpf_lock {
                let mut rules_map: aya::maps::HashMap<_, u32, ProcessConfig> =
                    aya::maps::HashMap::try_from(bpf.map_mut("PROCESS_RULES").unwrap())?;

                // 1. 获取当前系统所有进程以匹配名称
                let processes = self.system.list_running_processes()?;

                // 2. 这里的逻辑示例：将黑名单（Proxy/Reject 区）的应用写入 Map 实施阻断测试
                for app_name in &rules.blacklist {
                    for proc in processes.iter().filter(|p| p.name.contains(app_name)) {
                        let config = ProcessConfig { action: 2 }; // 2: Drop
                        rules_map.insert(proc.pid, config, 0)?;
                        tracing::debug!(
                            "Kernel rule set: PID {} ({}) -> REJECT",
                            proc.pid,
                            proc.name
                        );
                    }
                }

                // 3. 白名单放行逻辑同理（Map 中默认为空即放行，除非我们要做反向代理劫持）
                tracing::info!("eBPF Rules Map updated successfully");
            }
        }
        Ok(())
    }

    fn list_running_processes(&self) -> Result<Vec<ProcessInfo>> {
        self.system.list_running_processes()
    }
}
