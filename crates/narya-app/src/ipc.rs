use anyhow::Result;
use narya_ipc::{IpcNotification, IpcRequest, IpcResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    pub async fn connect_default() -> Result<Self> {
        Self::connect(narya_ipc::socket_path().to_string_lossy().as_ref()).await
    }

    pub async fn connect(path: &str) -> Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self { stream })
    }

    pub async fn send_request(&mut self, request: IpcRequest) -> Result<IpcResponse> {
        let json = serde_json::to_vec(&request)?;
        self.stream.write_all(&json).await?;

        loop {
            let mut buf = [0u8; 4096];
            let n = self.stream.read(&mut buf).await?;
            if n == 0 {
                anyhow::bail!("Connection closed");
            }
            if let Ok(response) = serde_json::from_slice::<IpcResponse>(&buf[..n]) {
                if response.id == request.id {
                    return Ok(response);
                }
            }
            // Ignore unsolicited notifications while waiting for the matching response.
        }
    }

    pub async fn next_notification(&mut self) -> Result<IpcNotification> {
        let mut buf = [0u8; 4096];
        let n = self.stream.read(&mut buf).await?;
        if n == 0 {
            anyhow::bail!("Connection closed");
        }
        // Note: In a real app, we'd use a length-prefixed codec. Current daemon writes one JSON object per read.
        let notification: IpcNotification = serde_json::from_slice(&buf[..n])?;
        Ok(notification)
    }
}
