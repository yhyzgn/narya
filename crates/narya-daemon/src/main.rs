mod config_gen;
mod kernel;
mod proxy;

use crate::kernel::KernelManager;
use crate::proxy::{LinuxGSettings, MacOSNetworkSetup, ProxyBackend, SystemProxy};
use anyhow::{Context, Result};
use narya_ipc::{decode_frame, encode_frame, IpcRequest, IpcResponse};
use narya_kernel::KernelId;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
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
    println!(
        "Daemon listening on {} (IPC protocol v{})",
        socket_path.display(),
        narya_ipc::PROTOCOL_VERSION
    );

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
            loop {
                tokio::select! {
                    res = read_frame(&mut socket) => {
                        match res {
                            Ok(payload) => {
                                let Ok(request) = decode_frame::<IpcRequest>(&payload) else {
                                    break;
                                };
                                let response = handle_request(request, &state).await;
                                if write_frame(&mut socket, &response).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    Ok(log_line) = log_rx.recv() => {
                        let level = if log_line.contains("ERROR") || log_line.contains("FATAL") {
                            "ERROR"
                        } else if log_line.contains("WARN") {
                            "WARN"
                        } else if log_line.contains("DEBUG") {
                            "DEBUG"
                        } else {
                            "INFO"
                        };
                        let notification = narya_ipc::IpcNotification::LogLine {
                            level: level.to_string(),
                            message: log_line,
                        };
                        if write_frame(&mut socket, &notification).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
    }
}

async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
    let mut header = [0u8; narya_ipc::FRAME_HEADER_LEN];
    reader.read_exact(&mut header).await?;
    let size = u32::from_be_bytes(header) as usize;
    if size > narya_ipc::MAX_FRAME_SIZE {
        anyhow::bail!("IPC frame exceeds maximum size: {size} bytes");
    }
    let mut payload = vec![0u8; size];
    reader.read_exact(&mut payload).await?;
    Ok(payload)
}

async fn write_frame<W: AsyncWrite + Unpin, T: serde::Serialize>(
    writer: &mut W,
    message: &T,
) -> Result<()> {
    let frame = encode_frame(message).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    writer.write_all(&frame).await?;
    writer.flush().await?;
    Ok(())
}

async fn handle_request(request: IpcRequest, state: &Arc<Mutex<DaemonState>>) -> IpcResponse {
    let id = request.id;
    let version = narya_ipc::PROTOCOL_VERSION;
    if request.version != version {
        return IpcResponse {
            version,
            id,
            result: None,
            error: Some(format!(
                "unsupported IPC protocol version {}; expected {}",
                request.version, version
            )),
        };
    }
    match handle_request_inner(&request, state).await {
        Ok(value) => IpcResponse {
            version,
            id,
            result: Some(value),
            error: None,
        },
        Err(error) => IpcResponse {
            version,
            id,
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
        "Ping" => Ok(serde_json::json!({
            "protocol_version": narya_ipc::PROTOCOL_VERSION,
            "daemon": "narya",
        })),
        "SetSystemProxy" => {
            let enabled = request
                .params
                .as_bool()
                .ok_or_else(|| anyhow::anyhow!("SetSystemProxy requires a boolean parameter"))?;
            state.proxy.set_enabled(enabled).await?;
            Ok(serde_json::json!(true))
        }
        "StartKernel" => {
            let (kernel_id, node) = parse_start_params(&request.params)?;
            if kernel_id != KernelId::SingBox {
                anyhow::bail!(
                    "configuration generation for kernel {kernel_id} is not available yet"
                );
            }
            let config_json = crate::config_gen::ConfigGenerator::generate_json(&node)?;
            let config_path = narya_ipc::kernel_config_path();
            write_private_config(&config_path, &config_json)?;
            let log_tx = state.log_tx.clone();
            state
                .kernel
                .start(kernel_id, config_path.to_string_lossy().as_ref(), log_tx)
                .await?;
            let healthy = state.kernel.records().into_iter().any(|record| {
                record.id == kernel_id
                    && record.healthy
                    && record.state == narya_kernel::KernelState::Running
            });
            if !healthy {
                anyhow::bail!("kernel {kernel_id} started without a healthy process");
            }
            Ok(serde_json::json!({"kernel": kernel_id, "healthy": true}))
        }
        "StopKernel" => {
            state.kernel.stop().await?;
            Ok(serde_json::json!(true))
        }
        "GetKernelStatus" => Ok(serde_json::to_value(kernel_status(&mut state.kernel))?),
        "InstallKernel" | "UpgradeKernel" => {
            let kernel = request
                .params
                .get("kernel")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let id: KernelId = kernel.parse()?;
            anyhow::bail!(
                "verified artifact source is required before installing or upgrading {id}"
            )
        }
        _ => anyhow::bail!("Unknown method: {}", request.method),
    }
}

fn parse_start_params(params: &serde_json::Value) -> Result<(KernelId, narya_core::Node)> {
    if let Some(node) = params.get("node") {
        let kernel = params
            .get("kernel")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("sing-box")
            .parse()?;
        return Ok((kernel, serde_json::from_value(node.clone())?));
    }
    Ok((KernelId::SingBox, serde_json::from_value(params.clone())?))
}

fn write_private_config(path: &Path, config_json: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(config_json)?)?;
    Ok(())
}

fn kernel_status(kernel: &mut KernelManager) -> Vec<narya_ipc::KernelInfo> {
    kernel
        .records()
        .into_iter()
        .map(|record| narya_ipc::KernelInfo {
            name: record.id.to_string(),
            installed: record.binary_path.is_some(),
            version: record.version,
            running: matches!(record.state, narya_kernel::KernelState::Running),
            healthy: record.healthy,
            state: record.state.as_str().to_string(),
            failure: record.failure,
        })
        .collect()
}
