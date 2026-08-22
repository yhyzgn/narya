use crate::installer::{self, InstalledKernel, KernelArtifactRequest};
use anyhow::{anyhow, Result};
use narya_kernel::{KernelId, KernelRecord, KernelRegistry, KernelState};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::broadcast;

pub struct KernelManager {
    registry: KernelRegistry,
    child: Option<Child>,
    active: Option<KernelId>,
}

impl KernelManager {
    pub fn new() -> Self {
        let mut registry = KernelRegistry::probe();
        discover_managed_kernels(&mut registry);
        Self {
            registry,
            child: None,
            active: None,
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
        self.stop().await?;
        let record = self
            .registry
            .require_installed(id)
            .map_err(|error| anyhow!(error.to_string()))?;
        let binary_path = record
            .binary_path
            .clone()
            .ok_or_else(|| anyhow!("kernel {id} has no executable path"))?;
        let config_path = std::path::Path::new(config_path);
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

        self.child = Some(child);
        self.active = Some(id);
        self.registry
            .set_state(id, KernelState::Running, false, None);
        // Give the child a bounded readiness window before reporting it as a
        // usable kernel. This catches instant exits (bad binaries/configs)
        // and prevents a transient process from becoming a fake connection.
        tokio::time::sleep(Duration::from_millis(50)).await;
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
            Ok(None) => self
                .registry
                .set_state(id, KernelState::Running, true, None),
            Ok(Some(status)) => {
                self.registry.set_state(
                    id,
                    KernelState::Failed,
                    false,
                    Some(format!("kernel exited with {status}")),
                );
                self.active = None;
                self.child = None;
            }
            Err(error) => {
                self.registry
                    .set_state(id, KernelState::Failed, false, Some(error.to_string()))
            }
        }
    }
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
