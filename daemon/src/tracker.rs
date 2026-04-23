use anyhow::Result;
use api::tracker::{AppIdentity, BypassRules, ConnectionMeta};
use async_trait::async_trait;
use narya_ebpf_common::ProcessConfig;
use std::net::IpAddr;
use std::path::PathBuf;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: PathBuf,
    pub user_id: Option<u32>,
}

#[async_trait]
pub trait ProcessTracker: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn lookup_connection(&self, src_ip: IpAddr, src_port: u16) -> Result<Option<ConnectionMeta>>;
    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>>;
    async fn update_bypass_rules(&self, rules: &BypassRules) -> Result<()>;
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
            .with_user(UpdateKind::Always)
    }

    fn is_user_application(name: &str, path: &PathBuf, user_id: Option<u32>) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let name_lower = name.to_lowercase();

        // 1. UID 基础过滤
        if let Some(uid) = user_id {
            if uid < 1000 && name_lower != "narya" { return false; }
        } else { return false; }

        // 2. 路径基础过滤
        if path_str.is_empty() { return false; }
        
        // 3. 语义化过滤：封杀辅助进程关键词
        let noise_keywords = [
            "thread", "worker", "pool", "helper", "handler", "broker", 
            "crashpad", "sandbox", "utility", "extension", "plugin", 
            "proxy", "agent", "daemon", "service", "executor", "task",
            "at-spi", "ibus", "gvfs", "xdg", "portal", "dbus", "pipewire"
        ];
        if noise_keywords.iter().any(|&k| name_lower.contains(k)) {
            // 特殊白名单：如果是知名应用名包含这些词，需放行 (暂时没有想到)
            return false;
        }

        // 4. 特征过滤：封杀包含冒号、中括号或过多数字的名称
        if name.contains(':') || name.starts_with('[') || name.ends_with(']') {
            return false;
        }
        
        // 5. 长度与结构过滤
        if name.len() < 2 || name.len() > 32 { return false; }

        // 6. 目录封锁
        let system_dirs = ["/usr/sbin/", "/sbin/", "/usr/libexec/", "/usr/lib/"];
        if system_dirs.iter().any(|dir| path_str.contains(dir)) {
            let app_whitelist = ["chrome", "firefox", "telegram", "code", "narya", "rustrover"];
            if !app_whitelist.iter().any(|&n| name_lower.contains(n)) {
                return false;
            }
        }

        true
    }
}

#[async_trait]
impl ProcessTracker for SystemProcessTracker {
    async fn start(&self) -> Result<()> { Ok(()) }
    async fn stop(&self) -> Result<()> { Ok(()) }
    async fn lookup_connection(&self, _src_ip: IpAddr, _src_port: u16) -> Result<Option<ConnectionMeta>> { Ok(None) }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        let all_processes = self.list_running_processes()?;
        let mut apps = Vec::new();
        let mut seen_names = HashSet::new();

        for proc in all_processes {
            if Self::is_user_application(&proc.name, &proc.path, proc.user_id) {
                // 提取干净的显示名称
                let mut display_name = proc.name.split(|c: char| !c.is_alphanumeric() && c != ' ')
                    .next()
                    .unwrap_or(&proc.name)
                    .to_string();
                
                // 进一步首字母大写美化
                if !display_name.is_empty() {
                    let mut c = display_name.chars();
                    display_name = c.next().unwrap().to_uppercase().collect::<String>() + c.as_str();
                }

                if !seen_names.contains(&display_name) {
                    apps.push(AppIdentity {
                        name: display_name.clone(),
                        identifier: display_name.clone().to_lowercase(),
                        icon_path: None,
                    });
                    seen_names.insert(display_name);
                }
            }
        }
        
        apps.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(apps)
    }

    async fn update_bypass_rules(&self, _rules: &BypassRules) -> Result<()> { Ok(()) }

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
                user_id: process.user_id().map(|u| u.to_string().parse::<u32>().unwrap_or(0)),
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
    async fn start(&self) -> Result<()> { self.system.start().await }
    async fn stop(&self) -> Result<()> { self.system.stop().await }
    async fn lookup_connection(&self, _i: IpAddr, _p: u16) -> Result<Option<ConnectionMeta>> { Ok(None) }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        self.system.list_network_apps().await
    }

    async fn update_bypass_rules(&self, rules: &BypassRules) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let mut bpf_lock = self.bpf.lock().await;
            if let Some(ref mut bpf) = *bpf_lock {
                let mut rules_map: aya::maps::HashMap<_, u32, ProcessConfig> =
                    aya::maps::HashMap::try_from(bpf.map_mut("PROCESS_RULES").unwrap())?;

                let processes = self.system.list_running_processes()?;
                for app_id in &rules.blacklist {
                    for proc in processes.iter().filter(|p| p.name.to_lowercase().contains(app_id)) {
                        let config = ProcessConfig { action: 2 };
                        let _ = rules_map.insert(proc.pid, config, 0);
                    }
                }
            }
        }
        Ok(())
    }

    fn list_running_processes(&self) -> Result<Vec<ProcessInfo>> {
        self.system.list_running_processes()
    }
}

pub struct MockProcessTracker;

#[async_trait]
impl ProcessTracker for MockProcessTracker {
    async fn start(&self) -> Result<()> { Ok(()) }
    async fn stop(&self) -> Result<()> { Ok(()) }
    async fn lookup_connection(&self, _i: IpAddr, _p: u16) -> Result<Option<ConnectionMeta>> { Ok(None) }
    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> { Ok(vec![]) }
    async fn update_bypass_rules(&self, _r: &BypassRules) -> Result<()> { Ok(()) }
    fn list_running_processes(&self) -> Result<Vec<ProcessInfo>> { Ok(vec![]) }
}
