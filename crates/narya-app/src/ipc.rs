use anyhow::Result;
use narya_ipc::{decode_frame, encode_frame, IpcNotification, IpcRequest, IpcResponse};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UnixStream};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time::timeout;

pub fn ensure_daemon() {
    let default_socket = narya_ipc::socket_path();
    let Ok(current_exe) = std::env::current_exe() else {
        eprintln!("failed to locate narya executable for daemon startup");
        return;
    };
    let Some(parent) = current_exe.parent() else {
        eprintln!("failed to locate narya executable directory for daemon startup");
        return;
    };
    let daemon_name = if cfg!(windows) {
        "narya-daemon.exe"
    } else {
        "narya-daemon"
    };
    let daemon = parent.join(daemon_name);
    #[cfg(debug_assertions)]
    if let Err(error) = rebuild_daemon_for_development(&daemon) {
        eprintln!("failed to prepare narya-daemon: {error}");
    }
    if !daemon.is_file() {
        eprintln!("narya-daemon is not available at {}", daemon.display());
        return;
    }
    if cfg!(not(debug_assertions)) && daemon_socket_is_compatible(&default_socket) {
        let _ = selected_socket().set(default_socket);
        return;
    }
    let socket = fingerprinted_socket(&daemon).unwrap_or(default_socket);
    let _ = selected_socket().set(socket.clone());
    if daemon_socket_is_compatible(&socket) {
        return;
    }
    let socket_name = socket.file_name().map(|name| name.to_os_string());
    if let Err(error) = std::process::Command::new(&daemon)
        .envs(socket_name.map(|name| ("NARYA_SOCKET_NAME", name)))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        eprintln!("failed to start narya-daemon: {error}");
        return;
    }
    for _ in 0..40 {
        if daemon_socket_is_compatible(&socket) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    eprintln!("narya-daemon did not become compatible within the startup timeout");
}

fn rebuild_daemon_for_development(daemon: &Path) -> Result<(), String> {
    let workspace = daemon
        .parent()
        .and_then(|parent| {
            parent.ancestors().find(|candidate| {
                candidate.join("Cargo.toml").is_file()
                    && candidate.join("crates/narya-daemon/Cargo.toml").is_file()
            })
        })
        .ok_or_else(|| "daemon is not inside a Narya development workspace".to_string())?;
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = std::process::Command::new(cargo)
        .args(["build", "-p", "narya-daemon"])
        .current_dir(workspace)
        .status()
        .map_err(|error| format!("failed to rebuild narya-daemon: {error}"))?;
    if !status.success() {
        return Err(format!("narya-daemon rebuild exited with {status}"));
    }
    Ok(())
}

fn selected_socket() -> &'static OnceLock<std::path::PathBuf> {
    static SOCKET: OnceLock<std::path::PathBuf> = OnceLock::new();
    &SOCKET
}

fn effective_socket_path() -> std::path::PathBuf {
    selected_socket()
        .get()
        .cloned()
        .unwrap_or_else(narya_ipc::socket_path)
}

fn fingerprinted_socket(daemon: &Path) -> Option<std::path::PathBuf> {
    use std::hash::{Hash, Hasher};
    let metadata = daemon.metadata().ok()?;
    let modified = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    metadata.len().hash(&mut hasher);
    modified.hash(&mut hasher);
    Some(narya_ipc::runtime_dir().join(format!("narya-{:08x}.sock", hasher.finish() as u32)))
}

fn daemon_socket_is_compatible(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::io::{Read, Write};
        let Ok(mut stream) = std::os::unix::net::UnixStream::connect(path) else {
            return false;
        };
        let request = IpcRequest {
            version: narya_ipc::PROTOCOL_VERSION,
            id: u64::MAX,
            method: "GetDaemonInfo".into(),
            params: serde_json::json!({}),
        };
        let Ok(frame) = encode_frame(&request) else {
            return false;
        };
        if stream
            .set_read_timeout(Some(std::time::Duration::from_millis(250)))
            .is_err()
            || stream.write_all(&frame).is_err()
        {
            return false;
        }
        let mut header = [0u8; narya_ipc::FRAME_HEADER_LEN];
        if stream.read_exact(&mut header).is_err() {
            return false;
        }
        let size = u32::from_be_bytes(header) as usize;
        if size > narya_ipc::MAX_FRAME_SIZE {
            return false;
        }
        let mut payload = vec![0u8; size];
        if stream.read_exact(&mut payload).is_err() {
            return false;
        }
        let Ok(response) = decode_frame::<IpcResponse>(&payload) else {
            return false;
        };
        let Some(result) = response.result else {
            return false;
        };
        result.get("version").and_then(serde_json::Value::as_str) == Some(env!("CARGO_PKG_VERSION"))
            && result
                .get("capabilities")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|capabilities| {
                    capabilities
                        .iter()
                        .any(|capability| capability.as_str() == Some("InstallOfficialKernel"))
                })
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

pub struct IpcClient {
    stream: Arc<Mutex<UnixStream>>,
}

impl IpcClient {
    pub async fn connect_default() -> Result<Self> {
        Self::connect(effective_socket_path().to_string_lossy().as_ref()).await
    }

    pub async fn connect(path: &str) -> Result<Self> {
        let path = path.to_string();
        let stream = ipc_runtime()
            .spawn(async move { UnixStream::connect(path).await })
            .await??;
        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    pub async fn send_request(&mut self, request: IpcRequest) -> Result<IpcResponse> {
        let stream = Arc::clone(&self.stream);
        ipc_runtime()
            .spawn(async move {
                let frame = encode_frame(&request)?;
                let mut stream = stream.lock().await;
                stream.write_all(&frame).await?;

                loop {
                    let payload = read_frame(&mut stream).await?;
                    if let Ok(response) = decode_frame::<IpcResponse>(&payload) {
                        if response.id == request.id {
                            return Ok::<IpcResponse, anyhow::Error>(response);
                        }
                    }
                    // Ignore unsolicited notifications while waiting for the matching response.
                }
            })
            .await?
    }

    pub async fn next_notification(&mut self) -> Result<IpcNotification> {
        let stream = Arc::clone(&self.stream);
        ipc_runtime()
            .spawn(async move {
                let mut stream = stream.lock().await;
                let payload = read_frame(&mut stream).await?;
                Ok::<_, anyhow::Error>(decode_frame(&payload)?)
            })
            .await?
    }
}

fn ipc_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("failed to create Narya IPC runtime"))
}

pub async fn measure_tcp_latency(address: String) -> Result<u32> {
    ipc_runtime()
        .spawn(async move {
            let started = std::time::Instant::now();
            timeout(
                std::time::Duration::from_secs(3),
                TcpStream::connect(address),
            )
            .await??;
            u32::try_from(started.elapsed().as_millis())
                .map_err(|_| anyhow::anyhow!("latency measurement overflow"))
        })
        .await?
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
