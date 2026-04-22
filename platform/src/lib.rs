use anyhow::{Result, bail};
use async_trait::async_trait;
use std::process::Command;

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    async fn setup_tun(&self, name: &str) -> Result<()>;
    async fn remove_tun(&self, name: &str) -> Result<()>;
    async fn set_system_proxy(&self, host: &str, port: u16) -> Result<()>;
    async fn clear_system_proxy(&self) -> Result<()>;
}

pub struct LinuxAdapter;

impl LinuxAdapter {
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(cmd).args(args).status()?;
        if !status.success() {
            bail!("Command {} {:?} failed", cmd, args);
        }
        Ok(())
    }
}

#[async_trait]
impl PlatformAdapter for LinuxAdapter {
    async fn setup_tun(&self, name: &str) -> Result<()> {
        tracing::info!("Setting up TUN device: {}", name);

        // 创建 TUN 设备
        let _ = self.run_command("ip", &["tuntap", "add", "mode", "tun", "name", name]);
        // 激活网卡
        let _ = self.run_command("ip", &["link", "set", "dev", name, "up"]);
        // 配置 IP 地址 (示例: 10.0.0.1)
        let _ = self.run_command("ip", &["addr", "add", "10.0.0.1/24", "dev", name]);

        Ok(())
    }

    async fn remove_tun(&self, name: &str) -> Result<()> {
        tracing::info!("Removing TUN device: {}", name);
        let _ = self.run_command("ip", &["tuntap", "del", "mode", "tun", "name", name]);
        Ok(())
    }

    async fn set_system_proxy(&self, host: &str, port: u16) -> Result<()> {
        tracing::info!("Setting Linux system proxy to {}:{}", host, port);

        // 针对 GNOME 环境使用 gsettings
        let port_str = port.to_string();
        let _ = self.run_command(
            "gsettings",
            &["set", "org.gnome.system.proxy", "mode", "manual"],
        );
        let _ = self.run_command(
            "gsettings",
            &["set", "org.gnome.system.proxy.http", "host", host],
        );
        let _ = self.run_command(
            "gsettings",
            &["set", "org.gnome.system.proxy.http", "port", &port_str],
        );
        let _ = self.run_command(
            "gsettings",
            &["set", "org.gnome.system.proxy.https", "host", host],
        );
        let _ = self.run_command(
            "gsettings",
            &["set", "org.gnome.system.proxy.https", "port", &port_str],
        );

        Ok(())
    }

    async fn clear_system_proxy(&self) -> Result<()> {
        tracing::info!("Clearing Linux system proxy");
        let _ = self.run_command(
            "gsettings",
            &["set", "org.gnome.system.proxy", "mode", "none"],
        );
        Ok(())
    }
}
