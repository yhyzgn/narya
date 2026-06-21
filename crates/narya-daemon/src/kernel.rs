use anyhow::Result;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::broadcast;

pub struct KernelManager {
    child: Option<Child>,
}

impl KernelManager {
    pub fn new() -> Self {
        Self { child: None }
    }

    pub async fn start(
        &mut self,
        binary_path: &str,
        config_path: &str,
        log_tx: broadcast::Sender<String>,
    ) -> Result<()> {
        self.stop().await?;
        println!(
            "Starting kernel: {} with config: {}",
            binary_path, config_path
        );

        let mut child = Command::new(binary_path)
            .args(["run", "-c", config_path])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");

        let tx1 = log_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx1.send(line);
            }
        });

        let tx2 = log_tx;
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx2.send(line);
            }
        });

        self.child = Some(child);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            println!("Stopping kernel process...");
            child.kill().await?;
            // Wait for it to exit to avoid zombies
            let _ = child.wait().await;
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
}
