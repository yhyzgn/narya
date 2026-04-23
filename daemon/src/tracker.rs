use anyhow::Result;
use api::tracker::{AppIdentity, BypassRules, ConnectionMeta};
use async_trait::async_trait;
use narya_ebpf_common::ProcessConfig;
use std::collections::HashSet;
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
    async fn lookup_connection(
        &self,
        src_ip: IpAddr,
        src_port: u16,
    ) -> Result<Option<ConnectionMeta>>;
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
        ProcessRefreshKind::new().with_exe(UpdateKind::Always)
    }

    fn is_user_application(name: &str, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let name_lower = name.to_lowercase();

        // 1. 过滤内核线程与无路径进程
        if path_str.is_empty() {
            return false;
        }

        // 2. 核心修复：过滤掉 0x 开头的地址型进程名
        if name_lower.starts_with("0x") {
            return false;
        }

        // 3. 更全面的系统组件黑名单
        let blacklisted_prefixes = [
            "kworker/",
            "systemd",
            "migration/",
            "idle_inject/",
            "irq/",
            "rcu_",
            "cpuhp/",
            "scsi_",
            "dbus",
            "pipewire",
            "wayland",
            "Xwayland",
            "gvfs",
            "at-spi",
            "ibus",
            "fcitx",
            "gnome",
            "kwin",
            "dconf",
            "akonadi",
            "baloo",
            "xdg",
            "polkit",
            "rtw89",
            "iwlwifi",
        ];
        if blacklisted_prefixes.iter().any(|p| name_lower.contains(p)) {
            return false;
        }

        // 4. 过滤常见的内部线程/辅助进程（通常名字较长或带括号）
        if name.starts_with('[') || name.contains("ThreadPool") || name.contains("Worker-") {
            return false;
        }

        // 5. 过滤掉非标准用户目录 (比如 /usr/lib/ 开头的通常是插件)
        if path_str.contains("/usr/lib/") || path_str.contains("/usr/libexec/") {
            return false;
        }

        // 6. 白名单覆盖
        let whitelisted_names = [
            "chrome",
            "firefox",
            "telegram",
            "discord",
            "spotify",
            "code",
            "narya",
            "rustrover",
        ];
        if whitelisted_names.iter().any(|n| name_lower.contains(n)) {
            return true;
        }

        true
    }
}

#[async_trait]
impl ProcessTracker for SystemProcessTracker {
    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }

    async fn lookup_connection(
        &self,
        _src_ip: IpAddr,
        _src_port: u16,
    ) -> Result<Option<ConnectionMeta>> {
        Ok(None)
    }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        let all_processes = self.list_running_processes()?;
        let mut apps = Vec::new();
        let mut seen_names = HashSet::new();

        for proc in all_processes {
            if Self::is_user_application(&proc.name, &proc.path) {
                // 进一步提取更易读的名称 (比如 /usr/bin/google-chrome -> Google Chrome)
                let mut display_name = proc
                    .name
                    .split(' ')
                    .next()
                    .unwrap_or(&proc.name)
                    .to_string();

                // 去除常见的后缀
                display_name = display_name.replace(".app", "").replace(".exe", "");

                if !seen_names.contains(&display_name) {
                    apps.push(AppIdentity {
                        name: display_name.clone(),
                        identifier: display_name.clone(),
                        icon_path: None,
                    });
                    seen_names.insert(display_name);
                }
            }
        }

        apps.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(apps)
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
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.system.stop().await?;
        Ok(())
    }

    async fn lookup_connection(
        &self,
        _src_ip: IpAddr,
        _src_port: u16,
    ) -> Result<Option<ConnectionMeta>> {
        Ok(None)
    }

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
                    for proc in processes.iter().filter(|p| p.name.contains(app_id)) {
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
    async fn start(&self) -> Result<()> {
        Ok(())
    }
    async fn stop(&self) -> Result<()> {
        Ok(())
    }

    async fn lookup_connection(
        &self,
        _src_ip: IpAddr,
        _src_port: u16,
    ) -> Result<Option<ConnectionMeta>> {
        Ok(None)
    }

    async fn list_network_apps(&self) -> Result<Vec<AppIdentity>> {
        Ok(vec![
            AppIdentity {
                name: "browser".to_string(),
                identifier: "browser".to_string(),
                icon_path: None,
            },
            AppIdentity {
                name: "telegram".to_string(),
                identifier: "telegram".to_string(),
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
