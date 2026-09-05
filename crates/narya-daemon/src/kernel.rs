use crate::installer::{self, InstalledKernel, KernelArtifactRequest};
use anyhow::{anyhow, bail, Context, Result};
use narya_kernel::{KernelId, KernelRecord, KernelRegistry, KernelState};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
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
        let registry = KernelRegistry::probe_managed(&narya_ipc::kernel_install_dir());
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
        if self.active == Some(request.kernel) {
            anyhow::bail!(
                "cannot install or upgrade the active kernel {}; switch to another installed kernel first",
                request.kernel
            )
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

    pub async fn install_official(
        &mut self,
        kernel: KernelId,
        log_tx: broadcast::Sender<String>,
        requested_upgrade: bool,
    ) -> Result<InstalledKernel> {
        if self.active == Some(kernel) {
            bail!(
                "cannot install or upgrade the active kernel {}; switch to another installed kernel first",
                kernel
            );
        }
        let upgrading = self.registry.record(kernel).binary_path.is_some();
        if requested_upgrade && !upgrading {
            bail!("cannot upgrade kernel {kernel} because it is not installed");
        }
        if !requested_upgrade && upgrading {
            bail!("kernel {kernel} is already installed; use UpgradeOfficialKernel to replace it");
        }
        self.registry.set_state(
            kernel,
            if upgrading {
                KernelState::Upgrading
            } else {
                KernelState::Installing
            },
            false,
            None,
        );
        let artifact = match crate::official_release::download_latest(kernel).await {
            Ok(artifact) => artifact,
            Err(error) => {
                self.registry.set_state(
                    kernel,
                    if upgrading {
                        KernelState::Installed
                    } else {
                        KernelState::Failed
                    },
                    false,
                    Some(error.to_string()),
                );
                return Err(error);
            }
        };
        let kernel_dir = narya_ipc::kernel_install_dir().join(kernel.as_str());
        tokio::fs::create_dir_all(&kernel_dir).await?;
        let staged = kernel_dir.join(format!(
            ".narya-official-{}-{}.download",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
        ));
        tokio::fs::write(&staged, &artifact.binary).await?;
        let request = KernelArtifactRequest {
            kernel,
            version: artifact.version,
            source: staged.to_string_lossy().into_owned(),
            sha256: artifact.sha256,
            signature: String::new(),
            public_key: String::new(),
            catalog_version: String::new(),
            catalog_platform: String::new(),
            catalog_architecture: String::new(),
        };
        let _ = log_tx.send(format!(
            "INFO installing official {} artifact from {}",
            artifact.kernel, artifact.source
        ));
        let result = self.install(&request, log_tx, requested_upgrade).await;
        let _ = tokio::fs::remove_file(staged).await;
        result
    }

    pub async fn uninstall(
        &mut self,
        id: KernelId,
        log_tx: broadcast::Sender<String>,
    ) -> Result<()> {
        if self.active == Some(id) {
            bail!("cannot uninstall the active kernel {id}; switch to another kernel first");
        }
        let managed_path = narya_ipc::kernel_install_dir()
            .join(id.as_str())
            .join("current");
        let record = self.registry.record(id).clone();
        if record.binary_path.as_deref() != Some(managed_path.as_path()) {
            bail!("kernel {id} is not installed in Narya's managed directory");
        }
        self.registry
            .set_state(id, KernelState::Uninstalling, false, None);
        match installer::uninstall(id, &narya_ipc::kernel_install_dir()).await {
            Ok(()) => {
                self.registry.set_not_installed(id);
                let _ = log_tx.send(format!(
                    "INFO kernel {id} uninstalled from Narya managed storage"
                ));
                Ok(())
            }
            Err(error) => {
                self.registry
                    .set_state(id, KernelState::Installed, false, Some(error.to_string()));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListenerKind {
    Http,
    Socks,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListenerTarget {
    host: String,
    port: u16,
    kind: ListenerKind,
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
        let mut reachable = 0;
        for target in &targets {
            if listener_reachable_async(target).await {
                reachable += 1;
            }
        }
        if reachable == targets.len() {
            return Ok(targets);
        }
        if tokio::time::Instant::now() >= deadline {
            bail!(
                "kernel readiness failed: only {reachable}/{} configured local listeners accepted a connection ({})",
                targets.len(),
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

async fn listener_reachable_async(target: &ListenerTarget) -> bool {
    let address = if target.host.contains(':') {
        format!("[{}]:{}", target.host, target.port)
    } else {
        format!("{}:{}", target.host, target.port)
    };
    let Ok(Ok(mut stream)) =
        timeout(Duration::from_millis(100), TcpStream::connect(&address)).await
    else {
        return false;
    };
    timeout(
        Duration::from_millis(150),
        probe_listener_async(&mut stream, target.kind),
    )
    .await
    .is_ok_and(|result| result.is_ok_and(|healthy| healthy))
}

fn listeners_reachable(targets: &[ListenerTarget]) -> bool {
    targets.iter().all(listener_healthy_sync)
}

async fn probe_listener_async(stream: &mut TcpStream, kind: ListenerKind) -> std::io::Result<bool> {
    match kind {
        ListenerKind::Socks => {
            stream.write_all(&[5, 1, 0]).await?;
            let mut response = [0_u8; 2];
            stream.read_exact(&mut response).await?;
            Ok(response == [5, 0])
        }
        ListenerKind::Http => {
            stream
                .write_all(
                    b"CONNECT 0.0.0.0:0 HTTP/1.1\r\nHost: 0.0.0.0:0\r\nConnection: close\r\n\r\n",
                )
                .await?;
            let mut response = [0_u8; 256];
            let size = stream.read(&mut response).await?;
            Ok(valid_http_proxy_response(&response[..size]))
        }
        ListenerKind::Generic => Ok(true),
    }
}

fn listener_healthy_sync(target: &ListenerTarget) -> bool {
    let address = if target.host.contains(':') {
        format!("[{}]:{}", target.host, target.port)
    } else {
        format!("{}:{}", target.host, target.port)
    };
    let Ok(address) = address.parse() else {
        return false;
    };
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&address, Duration::from_millis(100))
    else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(150)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(150)));
    match target.kind {
        ListenerKind::Socks => {
            use std::io::{Read, Write};
            if stream.write_all(&[5, 1, 0]).is_err() {
                return false;
            }
            let mut response = [0_u8; 2];
            stream.read_exact(&mut response).is_ok() && response == [5, 0]
        }
        ListenerKind::Http => {
            use std::io::{Read, Write};
            if stream
                .write_all(
                    b"CONNECT 0.0.0.0:0 HTTP/1.1\r\nHost: 0.0.0.0:0\r\nConnection: close\r\n\r\n",
                )
                .is_err()
            {
                return false;
            }
            let mut response = [0_u8; 256];
            let size = stream.read(&mut response).unwrap_or(0);
            valid_http_proxy_response(&response[..size])
        }
        ListenerKind::Generic => true,
    }
}

fn valid_http_proxy_response(response: &[u8]) -> bool {
    let Some(line) = response
        .split(|byte| *byte == b'\n')
        .next()
        .map(|line| line.strip_suffix(&[b'\r'][..]).unwrap_or(line))
    else {
        return false;
    };
    let mut fields = line.split(|byte| *byte == b' ' || *byte == b'\t');
    let Some(version) = fields.next() else {
        return false;
    };
    let Some(status) = fields.next().and_then(|value| {
        std::str::from_utf8(value)
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
    }) else {
        return false;
    };
    version.starts_with(b"HTTP/1.") && (100..=599).contains(&status)
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
            let kind = match inbound.get("type").and_then(serde_json::Value::as_str) {
                Some("http") => ListenerKind::Http,
                Some("socks") => ListenerKind::Socks,
                _ => ListenerKind::Generic,
            };
            targets.push(ListenerTarget { host, port, kind });
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
                kind: if key == "port" {
                    ListenerKind::Http
                } else {
                    ListenerKind::Socks
                },
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
                    port: 1080,
                    kind: ListenerKind::Socks,
                },
                ListenerTarget {
                    host: "127.0.0.1".into(),
                    port: 2080,
                    kind: ListenerKind::Http,
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

    #[test]
    fn health_requires_all_declared_listeners() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let targets = vec![
            ListenerTarget {
                host: "127.0.0.1".into(),
                port,
                kind: ListenerKind::Generic,
            },
            ListenerTarget {
                host: "127.0.0.1".into(),
                port: port.saturating_add(1),
                kind: ListenerKind::Generic,
            },
        ];
        assert!(!listeners_reachable(&targets));
        drop(listener);
    }

    #[tokio::test]
    async fn active_kernel_cannot_be_uninstalled() {
        let (log_tx, _) = broadcast::channel(1);
        let mut manager = KernelManager::new();
        manager.active = Some(KernelId::SingBox);
        let error = manager
            .uninstall(KernelId::SingBox, log_tx)
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("cannot uninstall the active kernel"));
    }

    #[test]
    fn protocol_probes_require_socks_and_http_handshakes() {
        use std::io::{Read, Write};
        use std::thread;

        let socks_listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let socks_port = socks_listener.local_addr().unwrap().port();
        let socks_thread = thread::spawn(move || {
            let (mut stream, _) = socks_listener.accept().unwrap();
            let mut request = [0_u8; 3];
            stream.read_exact(&mut request).unwrap();
            assert_eq!(request, [5, 1, 0]);
            stream.write_all(&[5, 0]).unwrap();
        });
        assert!(listener_healthy_sync(&ListenerTarget {
            host: "127.0.0.1".into(),
            port: socks_port,
            kind: ListenerKind::Socks,
        }));
        socks_thread.join().unwrap();

        let http_listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let http_port = http_listener.local_addr().unwrap().port();
        let http_thread = thread::spawn(move || {
            let (mut stream, _) = http_listener.accept().unwrap();
            let mut request = [0_u8; 128];
            let _ = stream.read(&mut request).unwrap();
            stream
                .write_all(b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n")
                .unwrap();
        });
        assert!(listener_healthy_sync(&ListenerTarget {
            host: "127.0.0.1".into(),
            port: http_port,
            kind: ListenerKind::Http,
        }));
        http_thread.join().unwrap();
        assert!(!valid_http_proxy_response(b"not http\r\n"));
    }
}
