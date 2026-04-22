use anyhow::Result;
use std::path::PathBuf;
use sysinfo::{System, Pid, ProcessesToUpdate, ProcessRefreshKind, UpdateKind};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: PathBuf,
}

pub trait ProcessTracker: Send + Sync {
    fn get_process_by_pid(&self, pid: u32) -> Result<Option<ProcessInfo>>;
    fn list_running_processes(&self) -> Result<Vec<ProcessInfo>>;
}

pub struct SystemProcessTracker {
    sys: std::sync::Mutex<System>,
}

impl SystemProcessTracker {
    pub fn new() -> Self {
        // 关键优化：只初始化一个空的 System，不进行全量刷新
        let sys = System::new();
        Self {
            sys: std::sync::Mutex::new(sys),
        }
    }

    fn get_refresh_kind() -> ProcessRefreshKind {
        // 关键优化：只刷新进程名称和可执行文件路径
        ProcessRefreshKind::new()
            .with_exe(UpdateKind::Always)
    }
}

impl ProcessTracker for SystemProcessTracker {
    fn get_process_by_pid(&self, pid: u32) -> Result<Option<ProcessInfo>> {
        let mut sys = self.sys.lock().unwrap();
        let target_pid = Pid::from(pid as usize);
        
        // sysinfo 0.32 使用 refresh_processes 并通过 ProcessRefreshKind 进行细粒度控制
        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[target_pid]),
            true,
            Self::get_refresh_kind()
        );
        
        if let Some(process) = sys.process(target_pid) {
            Ok(Some(ProcessInfo {
                pid,
                name: process.name().to_string_lossy().into_owned(),
                path: process.exe().map(|p| p.to_path_buf()).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
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

impl ProcessTracker for MockProcessTracker {
    fn get_process_by_pid(&self, pid: u32) -> Result<Option<ProcessInfo>> {
        Ok(Some(ProcessInfo {
            pid,
            name: "mock_app".to_string(),
            path: PathBuf::from("/usr/bin/mock_app"),
        }))
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
