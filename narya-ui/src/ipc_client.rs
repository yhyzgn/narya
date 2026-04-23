use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub async fn send_command(command: &str) -> Result<String> {
    let socket_path = "/tmp/narya.sock";
    let mut stream = UnixStream::connect(socket_path).await?;

    stream.write_all(command.as_bytes()).await?;
    // 在 tokio 中，异步 shutdown 不需要参数，且需要 await
    stream.shutdown().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    Ok(response)
}
