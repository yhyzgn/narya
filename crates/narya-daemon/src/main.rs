mod config_gen;
mod kernel;
mod proxy;

use crate::kernel::KernelManager;
use crate::proxy::{LinuxGSettings, MacOSNetworkSetup, ProxyBackend, SystemProxy};
use anyhow::{Context, Result};
use narya_ipc::{IpcRequest, IpcResponse};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::{broadcast, Mutex};

struct DaemonState {
    kernel: KernelManager,
    proxy: ProxyBackend,
    log_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let runtime_dir = narya_ipc::runtime_dir();
    fs::create_dir_all(&runtime_dir).with_context(|| {
        format!(
            "failed to create Narya runtime dir {}",
            runtime_dir.display()
        )
    })?;

    let socket_path = narya_ipc::socket_path();
    if fs::metadata(&socket_path).is_ok() {
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!("Daemon listening on {}", socket_path.display());

    let proxy = if cfg!(target_os = "macos") {
        ProxyBackend::MacOS(MacOSNetworkSetup)
    } else {
        ProxyBackend::Linux(LinuxGSettings)
    };

    let (log_tx, _) = broadcast::channel(100);
    let state = Arc::new(Mutex::new(DaemonState {
        kernel: KernelManager::new(),
        proxy,
        log_tx: log_tx.clone(),
    }));

    loop {
        let (mut socket, _) = listener.accept().await?;
        let state = Arc::clone(&state);
        let mut log_rx = state.lock().await.log_tx.subscribe();

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                tokio::select! {
                    res = socket.read(&mut buf) => {
                        match res {
                            Ok(0) => break,
                            Ok(n) => {
                                let Ok(request) = serde_json::from_slice::<IpcRequest>(&buf[..n]) else {
                                    continue;
                                };
                                let response = handle_request(request, &state).await;
                                if let Ok(res_json) = serde_json::to_vec(&response) {
                                    let _ = socket.write_all(&res_json).await;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    Ok(log_line) = log_rx.recv() => {
                        let level = if log_line.contains("ERROR") || log_line.contains("FATAL") {
                            "ERROR".to_string()
                        } else if log_line.contains("WARN") {
                            "WARN".to_string()
                        } else if log_line.contains("DEBUG") {
                            "DEBUG".to_string()
                        } else {
                            "INFO".to_string()
                        };

                        let notif = narya_ipc::IpcNotification::LogLine { level, message: log_line };
                        if let Ok(res_json) = serde_json::to_vec(&notif) {
                            let _ = socket.write_all(&res_json).await;
                        }
                    }
                }
            }
        });
    }
}

async fn handle_request(request: IpcRequest, state: &Arc<Mutex<DaemonState>>) -> IpcResponse {
    match handle_request_inner(&request, state).await {
        Ok(value) => IpcResponse {
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(error) => IpcResponse {
            id: request.id,
            result: None,
            error: Some(error.to_string()),
        },
    }
}

async fn handle_request_inner(
    request: &IpcRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> Result<serde_json::Value> {
    let mut state = state.lock().await;
    match request.method.as_str() {
        "SetSystemProxy" => {
            let enabled = request.params.as_bool().unwrap_or(false);
            state.proxy.set_enabled(enabled).await?;
            Ok(serde_json::json!(true))
        }
        "StartKernel" => {
            let node = serde_json::from_value::<narya_core::Node>(request.params.clone())?;
            let config_json = crate::config_gen::ConfigGenerator::generate_json(&node)?;
            let config_path = narya_ipc::kernel_config_path();
            write_private_config(&config_path, &config_json)?;
            let log_tx_clone = state.log_tx.clone();
            state
                .kernel
                .start(
                    "sing-box",
                    config_path.to_string_lossy().as_ref(),
                    log_tx_clone,
                )
                .await?;
            Ok(serde_json::json!(true))
        }
        "StopKernel" => {
            state.kernel.stop().await?;
            Ok(serde_json::json!(true))
        }
        "GetKernelStatus" => Ok(serde_json::to_value(kernel_status(
            state.kernel.is_running(),
        ))?),
        "InstallKernel" => anyhow::bail!("kernel installation is not implemented yet"),
        _ => anyhow::bail!("Unknown method"),
    }
}

fn write_private_config(path: &Path, config_json: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(config_json)?)?;
    Ok(())
}

fn kernel_status(sing_box_running: bool) -> Vec<narya_ipc::KernelInfo> {
    vec![
        kernel_info("sing-box", &["version"], sing_box_running),
        kernel_info("mihomo", &["-v"], false),
        kernel_info("xray", &["-version"], false).with_name("xray-core"),
    ]
}

fn kernel_info(binary: &str, args: &[&str], running: bool) -> narya_ipc::KernelInfo {
    let version = std::process::Command::new(binary)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .next()
                .unwrap_or("installed")
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty());

    narya_ipc::KernelInfo {
        name: binary.to_string(),
        installed: version.is_some(),
        version,
        running,
    }
}

trait KernelInfoNameExt {
    fn with_name(self, name: &str) -> Self;
}

impl KernelInfoNameExt for narya_ipc::KernelInfo {
    fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}
