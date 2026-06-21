use anyhow::Result;
use tokio::process::Command;

pub trait SystemProxy: Send + Sync {
    async fn set_enabled(&self, enabled: bool) -> Result<()>;
}

pub struct LinuxGSettings;

impl SystemProxy for LinuxGSettings {
    async fn set_enabled(&self, enabled: bool) -> Result<()> {
        let mode = if enabled { "manual" } else { "none" };
        println!("Setting Linux system proxy mode to: {}", mode);

        let status = Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy", "mode", mode])
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("Failed to set gsettings proxy mode");
        }
        Ok(())
    }
}

pub struct MacOSNetworkSetup;

impl SystemProxy for MacOSNetworkSetup {
    async fn set_enabled(&self, enabled: bool) -> Result<()> {
        let state = if enabled { "on" } else { "off" };
        println!("Setting macOS system proxy state to: {}", state);

        let status = Command::new("networksetup")
            .args(["-setwebproxystate", "Wi-Fi", state])
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("Failed to set networksetup proxy state");
        }
        Ok(())
    }
}

pub enum ProxyBackend {
    Linux(LinuxGSettings),
    MacOS(MacOSNetworkSetup),
}

impl SystemProxy for ProxyBackend {
    async fn set_enabled(&self, enabled: bool) -> Result<()> {
        match self {
            Self::Linux(b) => b.set_enabled(enabled).await,
            Self::MacOS(b) => b.set_enabled(enabled).await,
        }
    }
}
