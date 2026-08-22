use anyhow::Result;
use narya_ipc::{decode_frame, encode_frame, IpcNotification, IpcRequest, IpcResponse};
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
        let frame = encode_frame(&request)?;
        self.stream.write_all(&frame).await?;

        loop {
            let payload = read_frame(&mut self.stream).await?;
            if let Ok(response) = decode_frame::<IpcResponse>(&payload) {
                if response.id == request.id {
                    return Ok(response);
                }
            }
            // Ignore unsolicited notifications while waiting for the matching response.
        }
    }

    pub async fn next_notification(&mut self) -> Result<IpcNotification> {
        let payload = read_frame(&mut self.stream).await?;
        Ok(decode_frame(&payload)?)
    }
}

async fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>> {
    let mut header = [0u8; narya_ipc::FRAME_HEADER_LEN];
    stream.read_exact(&mut header).await?;
    let size = u32::from_be_bytes(header) as usize;
    if size > narya_ipc::MAX_FRAME_SIZE {
        anyhow::bail!("IPC frame exceeds maximum size: {size} bytes");
    }
    let mut payload = vec![0u8; size];
    stream.read_exact(&mut payload).await?;
    Ok(payload)
}
