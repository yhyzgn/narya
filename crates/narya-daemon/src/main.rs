mod config_gen;
mod installer;
mod kernel;
mod proxy;

use crate::kernel::KernelManager;
use crate::proxy::{LinuxGSettings, MacOSNetworkSetup, ProxyBackend, SystemProxy};
use anyhow::{Context, Result};
use narya_ipc::{decode_frame, encode_frame, IpcRequest, IpcResponse};
use narya_kernel::KernelId;
use narya_platform::{ProxyMode, RoutingPlan, SystemProxyPlan, SystemProxyState};
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
    proxy_snapshot: Option<SystemProxyState>,
    configured_routing: Option<RoutingPlan>,
    active_mode: ProxyMode,
}

struct StartParams {
    kernel: KernelId,
    node: narya_core::Node,
    routing: RoutingPlan,
    rules: Vec<narya_rules::Rule>,
    groups: Vec<narya_rules::RoutingGroup>,
    rule_sets: Vec<narya_rules::RuleSetSource>,
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
        proxy_snapshot: None,
        configured_routing: None,
        active_mode: ProxyMode::Disabled,
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
            apply_proxy_mode(
                &mut state,
                if enabled {
                    ProxyMode::SystemProxy
                } else {
                    ProxyMode::Disabled
                },
            )
            .await?;
            Ok(serde_json::json!(true))
        }
        "SetRoutingMode" => {
            let mode = request
                .params
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("SetRoutingMode requires a mode string"))?
                .parse::<ProxyMode>()?;
            apply_proxy_mode(&mut state, mode).await?;
            Ok(serde_json::json!({"mode": mode.as_str()}))
        }
        "StartKernel" => {
            let start = parse_start_params(&request.params)?;
            let kernel_id = start.kernel;
            let node = start.node;
            let routing = start.routing;
            let rules = start.rules;
            let groups = start.groups;
            let rule_sets = start.rule_sets;
            let mut generated = crate::config_gen::RoutingConfig {
                mode: routing.mode,
                plan: routing,
                ..crate::config_gen::RoutingConfig::default()
            };
            generated.rules =
                narya_rules::RuleSet::compile(rules).context("invalid routing rules")?;
            if !groups.is_empty() {
                generated.groups = groups;
            }
            generated.rule_sets = rule_sets;
            crate::config_gen::validate_rule_set_sources(&generated.rule_sets)?;
            let config_json = crate::config_gen::ConfigGenerator::generate_json_for_kernel(
                kernel_id, &node, &generated,
            )?;
            let config_path = narya_ipc::kernel_config_path();
            write_private_config(&config_path, &config_json)?;
            let log_tx = state.log_tx.clone();
            state
                .kernel
                .start(kernel_id, config_path.to_string_lossy().as_ref(), log_tx)
                .await?;
            state.configured_routing = Some(generated.plan.clone());
            let healthy = state.kernel.records().into_iter().any(|record| {
                record.id == kernel_id
                    && record.healthy
                    && record.state == narya_kernel::KernelState::Running
            });
            if !healthy {
                state.configured_routing = None;
                anyhow::bail!("kernel {kernel_id} started without a healthy process");
            }
            Ok(serde_json::json!({"kernel": kernel_id, "healthy": true}))
        }
        "StopKernel" => {
            // Keep the kernel alive if the system proxy cannot be restored; stopping
            // first would leave traffic in an unknown routing state.
            apply_proxy_mode(&mut state, ProxyMode::Disabled).await?;
            state.kernel.stop().await?;
            state.configured_routing = None;
            state.active_mode = ProxyMode::Disabled;
            Ok(serde_json::json!(true))
        }
        "GetKernelStatus" => Ok(serde_json::to_value(kernel_status(&mut state.kernel))?),
        "GetRoutingStatus" => Ok(serde_json::json!({
            "configured_mode": state
                .configured_routing
                .as_ref()
                .map(|plan| plan.mode.as_str()),
            "active_mode": state.active_mode.as_str(),
            "kernel_healthy": state.kernel.records().into_iter().any(|record| {
                record.healthy && record.state == narya_kernel::KernelState::Running
            })
        })),
        "InstallKernel" | "UpgradeKernel" => {
            let artifact: installer::KernelArtifactRequest =
                serde_json::from_value(request.params.clone())
                    .context("invalid kernel artifact request")?;
            let log_tx = state.log_tx.clone();
            let installed = state
                .kernel
                .install(&artifact, log_tx, request.method == "UpgradeKernel")
                .await?;
            Ok(serde_json::json!({
                "kernel": installed.kernel,
                "version": installed.version,
                "binary_path": installed.binary_path,
                "operation": request.method.to_ascii_lowercase()
            }))
        }
        _ => anyhow::bail!("Unknown method: {}", request.method),
    }
}

async fn apply_proxy_mode(state: &mut DaemonState, mode: ProxyMode) -> Result<()> {
    match mode {
        ProxyMode::Disabled => {
            if let Some(snapshot) = state.proxy_snapshot.take() {
                state.proxy.restore(&snapshot).await?
            } else {
                state.proxy.set_enabled(false).await?
            }
            state.active_mode = ProxyMode::Disabled;
            Ok(())
        }
        ProxyMode::SystemProxy => {
            if state.active_mode == ProxyMode::Tun {
                anyhow::bail!(
                    "cannot switch from TUN to system proxy without restarting the kernel configuration"
                );
            }
            let snapshot = state.proxy.capture().await?;
            let plan = SystemProxyPlan {
                http_host: "127.0.0.1".into(),
                http_port: 2080,
                socks_host: "127.0.0.1".into(),
                socks_port: 1080,
                bypass_domains: vec!["localhost".into(), "127.0.0.1".into(), "::1".into()],
            };
            if let Err(apply_error) = state.proxy.apply_system_proxy(&plan).await {
                return match state.proxy.restore(&snapshot).await {
                    Ok(()) => Err(apply_error),
                    Err(restore_error) => Err(anyhow::anyhow!(
                        "proxy apply failed: {apply_error}; restore failed: {restore_error}"
                    )),
                };
            }
            state.proxy_snapshot = Some(snapshot);
            state.active_mode = ProxyMode::SystemProxy;
            Ok(())
        }
        ProxyMode::Tun => {
            let tun = state.configured_routing.as_ref().ok_or_else(|| {
                anyhow::anyhow!("TUN mode requires a running kernel configuration")
            })?;
            let tun = tun
                .tun
                .as_ref()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("TUN mode requires an explicit TUN plan"))?;
            state.proxy.preflight_tun(&tun).await?;
            let healthy =
                state.kernel.records().into_iter().any(|record| {
                    record.healthy && record.state == narya_kernel::KernelState::Running
                });
            if !healthy {
                anyhow::bail!("TUN mode requires a healthy kernel process");
            }
            if let Some(snapshot) = state.proxy_snapshot.take() {
                state.proxy.restore(&snapshot).await?;
            }
            state.active_mode = ProxyMode::Tun;
            Ok(())
        }
    }
}

fn parse_start_params(params: &serde_json::Value) -> Result<StartParams> {
    if let Some(node) = params.get("node") {
        let kernel = params
            .get("kernel")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("sing-box")
            .parse()?;
        let routing = params
            .get("routing")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_else(|| crate::config_gen::RoutingConfig::default().plan);
        let rules = params
            .get("rules")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
        let groups = params
            .get("groups")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
        let rule_sets = params
            .get("rule_sets")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
        return Ok(StartParams {
            kernel,
            node: serde_json::from_value(node.clone())?,
            routing,
            rules,
            groups,
            rule_sets,
        });
    }
    Ok(StartParams {
        kernel: KernelId::SingBox,
        node: serde_json::from_value(params.clone())?,
        routing: crate::config_gen::RoutingConfig::default().plan,
        rules: Vec::new(),
        groups: Vec::new(),
        rule_sets: Vec::new(),
    })
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
