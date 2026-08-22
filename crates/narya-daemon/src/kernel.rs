use anyhow::{anyhow, Result};
use narya_kernel::{KernelId, KernelRecord, KernelRegistry, KernelState};
use std::process::Stdio;
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
        Self {
            registry: KernelRegistry::probe(),
            child: None,
            active: None,
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
        self.refresh_health();
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
