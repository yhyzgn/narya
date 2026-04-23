use daemon::controller::NaryaDaemon;
use daemon::ipc::IpcServer;
use std::sync::Arc;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Narya Proxy Engine is starting...");

    let daemon = Arc::new(NaryaDaemon::new());

    // Start the daemon and IPC in a background thread or tokio runtime
    // Since GPUI also uses tokio internally (optionally),
    // we use the GPUI execution context if possible, or a separate thread.
    let daemon_clone = daemon.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            // Start the daemon
            let d_clone = daemon_clone.clone();
            tokio::spawn(async move {
                if let Err(e) = d_clone.start().await {
                    tracing::error!("Failed to start daemon: {}", e);
                }
            });

            // Start IPC Server
            let ipc_server = IpcServer::new(daemon_clone, "/tmp/narya.sock");
            if let Err(e) = ipc_server.start().await {
                tracing::error!("IPC Server error: {}", e);
            }
        });
    });

    // Start GPUI
    narya_ui::run_app();

    Ok(())
}
