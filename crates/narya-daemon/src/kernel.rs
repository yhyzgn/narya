use crate::installer::{self, InstalledKernel, KernelArtifactRequest};
use anyhow::{anyhow, bail, Context, Result};
use narya_kernel::{KernelId, KernelRecord, KernelRegistry, KernelState};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::sync::broadcast;
use tokio::time::timeout;

pub struct KernelManager {
    registry: KernelRegistry,
    child: Option<Child>,
    active: Option<KernelId>,
    active_config: Option<(PathBuf, Vec<u8>)>,
    active_listeners: Vec<ListenerTarget>,
}

impl KernelManager {
    pub fn new() -> Self {
        let mut registry = KernelRegistry::probe();
        discover_managed_kernels(&mut registry);
        Self {
            registry,
            child: None,
            active: None,
            active_config: None,
            active_listeners: Vec::new(),
        }
    }

    pub async fn install(
        &mut self,
        request: &KernelArtifactRequest,
        log_tx: broadcast::Sender<String>,
        requested_upgrade: bool,
    ) -> Result<InstalledKernel> {
        if self.active.is_some() {
            anyhow::bail!("cannot install or upgrade a kernel while another kernel is running")
        }
        let upgrading = self.registry.record(request.kernel).binary_path.is_some();
        if requested_upgrade && !upgrading {
            anyhow::bail!(
                "cannot upgrade kernel {} because it is not installed",
                request.kernel
            );
        }
        if request.source.starts_with("https://") {
            let trusted_key = std::env::var("NARYA_KERNEL_CATALOG_PUBLIC_KEY")
                .or_else(|_| {
                    std::fs::read_to_string(
                        narya_ipc::kernel_catalog_dir().join("trusted-public-key"),
                    )
                })
                .map_err(|_| {
                    anyhow!("HTTPS kernel artifacts require a configured catalog trust root")
                })?;
            let catalog = crate::kernel_catalog::load_verified(trusted_key.trim()).await?;
            let platform = if request.catalog_platform.trim().is_empty() {
                crate::kernel_catalog::default_platform()
            } else {
                request.catalog_platform.trim()
            };
            let architecture = if request.catalog_architecture.trim().is_empty() {
                crate::kernel_catalog::default_architecture()
            } else {
                request.catalog_architecture.trim()
            };
            let entry = crate::kernel_catalog::find_entry(
                &catalog,
                request.kernel,
                request.catalog_version.trim(),
                platform,
                architecture,
            )?;
            if entry.source != request.source
                || !entry.sha256.eq_ignore_ascii_case(&request.sha256)
                || entry.signature != request.signature
                || entry.public_key != request.public_key
            {
                anyhow::bail!("kernel artifact does not match the verified catalog entry");
            }
        }
        if !requested_upgrade && upgrading {
            anyhow::bail!(
                "kernel {} is already installed; use UpgradeKernel to replace it",
                request.kernel
            );
        }
        self.registry.set_state(
            request.kernel,
            if upgrading {
                KernelState::Upgrading
            } else {
                KernelState::Installing
            },
            false,
            None,
        );
        match installer::install(request, &narya_ipc::kernel_install_dir(), upgrading).await {
            Ok(installed) => {
                self.registry.set_installed(
                    installed.kernel,
                    installed.binary_path.clone(),
                    installed.version.clone(),
                );
                let _ = log_tx.send(format!(
                    "INFO kernel {} {} successfully {}",
                    installed.kernel,
                    installed.version,
                    if upgrading { "upgraded" } else { "installed" }
                ));
                Ok(installed)
            }
            Err(error) => {
                let previous = self.registry.record(request.kernel).binary_path.is_some();
                self.registry.set_state(
                    request.kernel,
                    if previous {
                        KernelState::Installed
                    } else {
                        KernelState::Failed
                    },
                    false,
                    Some(error.to_string()),
                );
                Err(error)
            }
        }
    }

    pub fn records(&mut self) -> Vec<KernelRecord> {
        self.refresh_health();
        self.registry.records().cloned().collect()
    }

    pub async fn start(
        &mut self,
        id: KernelId,
        config_path: &str,
        log_tx: broadcast::Sender<String>,
    ) -> Result<()> {
        let config_path = Path::new(config_path).to_path_buf();
        let config_bytes = tokio::fs::read(&config_path)
            .await
            .map_err(|error| anyhow!("failed to read kernel configuration: {error}"))?;
        let previous = self.active.zip(self.active_config.clone());
        if self.active.is_some() {
            self.stop().await?;
        }

        match self
            .start_process(id, &config_path, config_bytes, log_tx.clone())
            .await
        {
            Ok(()) => Ok(()),
            Err(start_error) => {
                let Some((old_id, (old_path, old_config))) = previous else {
                    return Err(start_error);
                };
                tokio::fs::write(&old_path, &old_config)
                    .await
                    .map_err(|restore_error| {
                        anyhow!(
                            "kernel switch failed: {start_error}; restoring previous configuration failed: {restore_error}"
                        )
                    })?;
                self.start_process(old_id, &old_path, old_config, log_tx)
                    .await
                    .map_err(|restore_error| {
                        anyhow!(
                            "kernel switch failed: {start_error}; previous kernel restoration failed: {restore_error}"
                        )
                    })?;
                Err(anyhow!(
                    "kernel switch failed: {start_error}; previous kernel restored"
                ))
            }
        }
    }

    async fn start_process(
        &mut self,
        id: KernelId,
        config_path: &Path,
        config_bytes: Vec<u8>,
        log_tx: broadcast::Sender<String>,
    ) -> Result<()> {
        let record = self
            .registry
            .require_installed(id)
            .map_err(|error| anyhow!(error.to_string()))?;
        let binary_path = record
            .binary_path
            .clone()
            .ok_or_else(|| anyhow!("kernel {id} has no executable path"))?;
        if let Err(error) = installer::verify_installed(&binary_path).await {
            self.registry
                .set_state(id, KernelState::Failed, false, Some(error.to_string()));
            return Err(error);
        }
        self.registry
            .set_state(id, KernelState::Starting, false, None);

        let mut child = match Command::new(&binary_path)
            .args(id.config_args(config_path))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                self.registry
                    .set_state(id, KernelState::Failed, false, Some(error.to_string()));
                return Err(error.into());
            }
        };

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("failed to capture kernel stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("failed to capture kernel stderr"))?;

        let tx1 = log_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx1.send(line);
            }
        });

        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = log_tx.send(line);
            }
        });

        self.registry
            .set_state(id, KernelState::Starting, false, None);
        // A live process is not sufficient evidence of a usable VPN. Require
        // a declared local HTTP/SOCKS listener to accept a TCP connection.
        let listeners = match wait_for_configured_listeners(&config_bytes, &mut child).await {
            Ok(listeners) => listeners,
            Err(error) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                self.registry
                    .set_state(id, KernelState::Failed, false, Some(error.to_string()));
                return Err(error);
            }
        };
        self.child = Some(child);
        self.active = Some(id);
        self.active_config = Some((config_path.to_path_buf(), config_bytes));
        self.active_listeners = listeners;
        self.registry
            .set_state(id, KernelState::Running, false, None);
        self.refresh_health();
        if !self.registry.record(id).healthy {
            let failure = self
                .registry
                .record(id)
                .failure
                .clone()
                .unwrap_or_else(|| "kernel failed readiness check".into());
            let _ = self.stop().await;
            self.registry
                .set_state(id, KernelState::Failed, false, Some(failure.clone()));
            anyhow::bail!("kernel {id} failed readiness: {failure}");
        }
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        let Some(id) = self.active.take() else {
            self.child = None;
            self.active_config = None;
            self.active_listeners.clear();
            return Ok(());
        };
        self.registry
            .set_state(id, KernelState::Stopping, false, None);
        if let Some(mut child) = self.child.take() {
            if let Err(error) = child.kill().await {
                self.registry
                    .set_state(id, KernelState::Failed, false, Some(error.to_string()));
                return Err(error.into());
            }
            let _ = child.wait().await;
        }
        self.registry
            .set_state(id, KernelState::Installed, false, None);
        self.active_config = None;
        self.active_listeners.clear();
        Ok(())
    }

    fn refresh_health(&mut self) {
        let Some(id) = self.active else {
            return;
        };
        let Some(child) = self.child.as_mut() else {
            self.registry.set_state(
                id,
                KernelState::Failed,
                false,
                Some("missing child process".into()),
            );
            self.active = None;
            return;
        };
        match child.try_wait() {
            Ok(None) if listeners_reachable(&self.active_listeners) => {
                self.registry
                    .set_state(id, KernelState::Running, true, None)
            }
            Ok(None) => {
                if let Some(child) = self.child.as_mut() {
                    let _ = child.start_kill();
                }
                self.registry.set_state(
                    id,
                    KernelState::Failed,
                    false,
                    Some("kernel listener became unreachable".into()),
                );
                self.active = None;
                self.child = None;
                self.active_config = None;
                self.active_listeners.clear();
            }
            Ok(Some(status)) => {
                self.registry.set_state(
                    id,
                    KernelState::Failed,
                    false,
                    Some(format!("kernel exited with {status}")),
                );
                self.active = None;
                self.child = None;
                self.active_config = None;
                self.active_listeners.clear();
            }
            Err(error) => {
                self.registry
                    .set_state(id, KernelState::Failed, false, Some(error.to_string()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListenerTarget {
    host: String,
    port: u16,
}

async fn wait_for_configured_listeners(
    config_bytes: &[u8],
    child: &mut Child,
) -> Result<Vec<ListenerTarget>> {
    let targets = listener_targets(config_bytes)?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child.try_wait()? {
            bail!("kernel exited during readiness with {status}");
        }
        for target in &targets {
            let address = format!("{}:{}", target.host, target.port);
            if timeout(Duration::from_millis(100), TcpStream::connect(&address))
                .await
                .is_ok_and(|result| result.is_ok())
            {
                return Ok(targets);
            }
        }
        if tokio::time::Instant::now() >= deadline {
            bail!(
                "kernel readiness failed: no configured local listener accepted a connection ({})",
                targets
                    .iter()
                    .map(|target| format!("{}:{}", target.host, target.port))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn listeners_reachable(targets: &[ListenerTarget]) -> bool {
    targets.iter().any(|target| {
        let address = if target.host.contains(':') {
            format!("[{}]:{}", target.host, target.port)
        } else {
            format!("{}:{}", target.host, target.port)
        };
        address.parse().ok().is_some_and(|address| {
            std::net::TcpStream::connect_timeout(&address, Duration::from_millis(50)).is_ok()
        })
    })
}

fn listener_targets(config_bytes: &[u8]) -> Result<Vec<ListenerTarget>> {
    let root: serde_json::Value = serde_json::from_slice(config_bytes)
        .context("kernel configuration is not valid JSON for readiness probing")?;
    let mut targets = Vec::new();
    if let Some(inbounds) = root.get("inbounds").and_then(serde_json::Value::as_array) {
        for inbound in inbounds {
            let Some(port) = inbound
                .get("listen_port")
                .or_else(|| inbound.get("port"))
                .and_then(serde_json::Value::as_u64)
                .and_then(|port| u16::try_from(port).ok())
            else {
                continue;
            };
            let host = inbound
                .get("listen")
                .and_then(serde_json::Value::as_str)
                .filter(|host| !host.is_empty() && *host != "0.0.0.0" && *host != "::")
                .unwrap_or("127.0.0.1")
                .to_string();
            targets.push(ListenerTarget { host, port });
        }
    }
    for key in ["port", "socks-port"] {
        if let Some(port) = root
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .and_then(|port| u16::try_from(port).ok())
        {
            targets.push(ListenerTarget {
                host: "127.0.0.1".into(),
                port,
            });
        }
    }
    targets.sort_by(|left, right| left.port.cmp(&right.port).then(left.host.cmp(&right.host)));
    targets.dedup();
    if targets.is_empty() {
        bail!("kernel configuration declares no local readiness listener");
    }
    Ok(targets)
}

fn discover_managed_kernels(registry: &mut KernelRegistry) {
    let root = narya_ipc::kernel_install_dir();
    for id in KernelId::ALL {
        let path = root.join(id.as_str()).join("current");
        // Managed binaries are only discoverable when their persisted digest
        // exists. The digest is rechecked asynchronously before every start;
        // this prevents silently adopting an unverified or tampered binary
        // left by an older installer.
        if !path.is_file() || !path.with_file_name("sha256").is_file() {
            continue;
        }
        let version = std::fs::read_to_string(path.with_file_name("version"))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "managed-unknown".into());
        registry.set_installed(id, path, version);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_listeners_from_sing_box_and_mihomo_shapes() {
        let sing_box = serde_json::json!({
            "inbounds": [
                {"type": "socks", "listen": "127.0.0.1", "listen_port": 1080},
                {"type": "http", "listen": "127.0.0.1", "listen_port": 2080}
            ]
        });
        assert_eq!(
            listener_targets(&serde_json::to_vec(&sing_box).unwrap()).unwrap(),
            vec![
                ListenerTarget {
                    host: "127.0.0.1".into(),
                    port: 1080
                },
                ListenerTarget {
                    host: "127.0.0.1".into(),
                    port: 2080
                },
            ]
        );

        let mihomo = serde_json::json!({"port": 2080, "socks-port": 1080});
        assert_eq!(
            listener_targets(&serde_json::to_vec(&mihomo).unwrap())
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn readiness_probe_rejects_config_without_listener() {
        let error = listener_targets(br#"{"outbounds": []}"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("no local readiness listener"));
    }
}
