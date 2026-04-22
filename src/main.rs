use daemon::controller::NaryaDaemon;
use daemon::ipc::IpcServer;
use std::sync::Arc;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Narya VPN is starting...");

    let daemon = Arc::new(NaryaDaemon::new());

    // Start the daemon in the background
    let daemon_clone = daemon.clone();
    tokio::spawn(async move {
        if let Err(e) = daemon_clone.start().await {
            tracing::error!("Failed to start daemon: {}", e);
        }
    });

    // Start IPC Server
    let ipc_server = IpcServer::new(daemon, "/tmp/narya.sock");
    ipc_server.start().await?;

    Ok(())
}
