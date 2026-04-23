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

        // 1. UID 严格过滤 (UID < 1000 通常是系统账号)
        if let Some(uid) = user_id {
            if uid < 1000 && name_lower != "narya" { return false; }
        } else { return false; }

        // 2. 彻底封杀内存地址和无效名称
        if name_lower.starts_with("0x") || name_lower.contains("0x") { return false; }
        if name_lower.len() < 3 { return false; }

        // 3. 路径深度过滤：封杀所有系统私有组件目录
        // 在 Linux 上，位于 /lib, /libexec, /lib64 下的几乎全是后台服务
        if path_str.contains("/lib/") || path_str.contains("/libexec/") || path_str.contains("/lib64/") {
            // 除非是极个别被错误安装在这些目录下的主流 App
            let app_whitelist = ["chrome", "firefox", "telegram", "discord", "code", "rustrover"];
            if !app_whitelist.iter().any(|&n| name_lower.contains(n)) {
                return false;
            }
        }

        // 4. 严苛的“服务/组件”关键词黑名单
        let service_keywords = [
            "thread", "worker", "pool", "helper", "handler", "broker", 
            "crashpad", "sandbox", "utility", "extension", "plugin", 
            "proxy", "agent", "daemon", "service", "executor", "task",
            "at-spi", "ibus", "gvfs", "xdg", "portal", "dbus", "pipewire",
            "factory", "component", "manager", "monitor", "backend", "frontend",
            "controller", "bridge", "bus", "registry", "collector", "store",
            "gnome-shell", "nautilus", "evolution", "tracker", "mission-control"
        ];
        if service_keywords.iter().any(|&k| name_lower.contains(k)) {
            // 白名单二次检查
            let app_whitelist = ["chrome", "firefox", "telegram", "code", "narya", "rustrover"];
            if !app_whitelist.iter().any(|&n| name_lower.contains(n)) {
                return false;
            }
        }

        // 5. 过滤掉解释器本身 (当它们没有携带具体的 App 名称时)
        let interpreters = ["python", "node", "java", "perl", "ruby", "bash", "sh"];
        if interpreters.iter().any(|&i| name_lower == i) {
            return false;
        }

        // 6. 特征过滤：封杀包含冒号或中括号的底层任务
        if name.contains(':') || name.starts_with('[') {
            return false;
        }

        true
    }
}

#[async_trait]
impl ProcessTracker for SystemProcessTracker {
    async fn start(&self) -> Result<()> { Ok(()) }
    async fn stop(&self) -> Result<()> { Ok(()) }
    async fn lookup_connection(&self, _ip: IpAddr, _p: u16) -> Result<Option<ConnectionMeta>> { Ok(None) }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        let all_processes = self.list_running_processes()?;
        let mut apps = Vec::new();
        let mut seen_names = HashSet::new();

        for proc in all_processes {
            if Self::is_user_application(&proc.name, &proc.path, proc.user_id) {
                // 提取最简洁、可读的软件名称
                let mut display_name = proc.name.split(|c: char| !c.is_alphanumeric() && c != ' ')
                    .next()
                    .unwrap_or(&proc.name)
                    .to_string();
                
                // 去除 common 的开发后缀
                display_name = display_name.replace("linux", "").replace("-bin", "");

                if display_name.len() >= 3 && !seen_names.contains(&display_name) {
                    let mut c = display_name.chars();
                    let capitalized = c.next().unwrap().to_uppercase().collect::<String>() + c.as_str();
                    
                    apps.push(AppIdentity {
                        name: capitalized.clone(),
                        identifier: capitalized.to_lowercase(),
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
