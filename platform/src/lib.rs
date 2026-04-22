use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    async fn setup_tun(&self, name: &str) -> Result<()>;
    async fn remove_tun(&self, name: &str) -> Result<()>;
    async fn set_system_proxy(&self, host: &str, port: u16) -> Result<()>;
    async fn clear_system_proxy(&self) -> Result<()>;
}

pub struct LinuxAdapter;

#[async_trait]
impl PlatformAdapter for LinuxAdapter {
    async fn setup_tun(&self, name: &str) -> Result<()> {
        tracing::info!("Setting up TUN device: {}", name);
        Ok(())
    }

    async fn remove_tun(&self, name: &str) -> Result<()> {
        tracing::info!("Removing TUN device: {}", name);
        Ok(())
    }

    async fn set_system_proxy(&self, host: &str, port: u16) -> Result<()> {
        tracing::info!("Setting Linux system proxy to {}:{}", host, port);
        Ok(())
    }

    async fn clear_system_proxy(&self) -> Result<()> {
        tracing::info!("Clearing Linux system proxy");
        Ok(())
    }
}
