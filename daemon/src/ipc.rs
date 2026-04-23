use crate::controller::NaryaDaemon;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

pub struct IpcServer {
    daemon: Arc<NaryaDaemon>,
    socket_path: String,
}

impl IpcServer {
    pub fn new(daemon: Arc<NaryaDaemon>, socket_path: &str) -> Self {
        Self {
            daemon,
            socket_path: socket_path.to_string(),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let _ = std::fs::remove_file(&self.socket_path);
        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("IPC Server listening on {}", self.socket_path);

        loop {
            let (stream, _) = listener.accept().await?;
            let daemon = self.daemon.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(stream, daemon).await {
                    tracing::error!("Error handling IPC client: {}", e);
                }
            });
        }
    }

    async fn handle_client(mut stream: UnixStream, daemon: Arc<NaryaDaemon>) -> Result<()> {
        let mut buffer = [0; 8192]; // 进一步调大缓冲区以接收海量 App 规则
        loop {
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let request = String::from_utf8_lossy(&buffer[..n]);
            let request_trimmed = request.trim();
            tracing::info!("IPC Request: {}", request_trimmed);

            let response = if request_trimmed == "status" {
                "running\n".to_string()
            } else if request_trimmed == "get_apps" {
                match daemon.get_tracker().list_network_apps().await {
                    Ok(apps) => {
                        serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&apps)?)?
                            .to_string()
                            + "\n"
                    }
                    Err(e) => format!("error: {}\n", e),
                }
            } else if request_trimmed == "get_connections" {
                match daemon.get_tracker().get_active_connections().await {
                    Ok(conns) => serde_json::to_string(&conns)? + "\n",
                    Err(e) => format!("error: {}\n", e),
                }
            } else if request_trimmed.starts_with("update_rules ") {
                let json_part = &request_trimmed["update_rules ".len()..];
                match serde_json::from_str::<api::tracker::BypassRules>(json_part) {
                    Ok(rules) => {
                        // 1. 持久化保存
                        let _ = daemon.get_config_manager().update_rules(rules.clone());
                        // 2. 应用到 eBPF
                        match daemon.get_tracker().update_bypass_rules(&rules).await {
                            Ok(_) => "ok\n".to_string(),
                            Err(e) => format!("error: {}\n", e),
                        }
                    }
                    Err(e) => format!("invalid json: {}\n", e),
                }
            } else if request_trimmed.starts_with("select_proxy ") {
                let proxy_name = &request_trimmed["select_proxy ".len()..];
                tracing::info!("Switching active proxy to: {}", proxy_name);
                // 1. 持久化选中的节点
                let _ = daemon
                    .get_config_manager()
                    .update_active_node(proxy_name.to_string());
                "ok\n".to_string()
            } else if request_trimmed == "start" {
                daemon.start().await?;
                "started\n".to_string()
            } else if request_trimmed == "stop" {
                daemon.stop().await?;
                "stopped\n".to_string()
            } else {
                "unknown command\n".to_string()
            };

            stream.write_all(response.as_bytes()).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn test_ipc_communication() {
        let daemon = Arc::new(NaryaDaemon::new());
        let socket_path = "/tmp/narya_test.sock";
        let server = IpcServer::new(daemon, socket_path);

        let server_handle = tokio::spawn(async move {
            server.start().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut stream = UnixStream::connect(socket_path).await.unwrap();
        stream.write_all(b"status").await.unwrap();

        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..n], b"running\n");

        server_handle.abort();
        let _ = std::fs::remove_file(socket_path);
    }
}
