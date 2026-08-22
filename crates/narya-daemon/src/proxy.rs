use anyhow::{anyhow, Result};
use narya_platform::{SystemProxyPlan, SystemProxyState, TunPlan};
use tokio::process::Command;

pub trait SystemProxy: Send + Sync {
    async fn set_enabled(&self, enabled: bool) -> Result<()>;
}

pub struct LinuxGSettings;

impl SystemProxy for LinuxGSettings {
    async fn set_enabled(&self, enabled: bool) -> Result<()> {
        let mode = if enabled { "manual" } else { "none" };
        run_gsettings(["set", "org.gnome.system.proxy", "mode", mode]).await
    }
}

impl LinuxGSettings {
    async fn capture(&self) -> Result<SystemProxyState> {
        let mode = gsettings_get("org.gnome.system.proxy", "mode").await?;
        let enabled = mode.trim_matches('\'') == "manual";
        Ok(SystemProxyState {
            enabled,
            http_host: Some(parse_string(
                &gsettings_get("org.gnome.system.proxy.http", "host").await?,
            )),
            http_port: Some(parse_port(
                &gsettings_get("org.gnome.system.proxy.http", "port").await?,
            )?),
            https_host: Some(parse_string(
                &gsettings_get("org.gnome.system.proxy.https", "host").await?,
            )),
            https_port: Some(parse_port(
                &gsettings_get("org.gnome.system.proxy.https", "port").await?,
            )?),
            socks_host: Some(parse_string(
                &gsettings_get("org.gnome.system.proxy.socks", "host").await?,
            )),
            socks_port: Some(parse_port(
                &gsettings_get("org.gnome.system.proxy.socks", "port").await?,
            )?),
            bypass_domains: parse_ignore_hosts(
                &gsettings_get("org.gnome.system.proxy", "ignore-hosts").await?,
            ),
        })
    }

    async fn apply(&self, plan: &SystemProxyPlan) -> Result<()> {
        run_gsettings(["set", "org.gnome.system.proxy", "mode", "manual"]).await?;
        set_gsettings("org.gnome.system.proxy.http", "host", &plan.http_host).await?;
        set_gsettings(
            "org.gnome.system.proxy.http",
            "port",
            &plan.http_port.to_string(),
        )
        .await?;
        set_gsettings("org.gnome.system.proxy.https", "host", &plan.http_host).await?;
        set_gsettings(
            "org.gnome.system.proxy.https",
            "port",
            &plan.http_port.to_string(),
        )
        .await?;
        set_gsettings("org.gnome.system.proxy.socks", "host", &plan.socks_host).await?;
        set_gsettings(
            "org.gnome.system.proxy.socks",
            "port",
            &plan.socks_port.to_string(),
        )
        .await?;
        let ignore_hosts = format_gvariant_strings(&plan.bypass_domains);
        set_gsettings("org.gnome.system.proxy", "ignore-hosts", &ignore_hosts).await
    }

    async fn restore(&self, state: &SystemProxyState) -> Result<()> {
        if !state.enabled {
            return run_gsettings(["set", "org.gnome.system.proxy", "mode", "none"]).await;
        }
        let http_host = state
            .http_host
            .as_deref()
            .ok_or_else(|| anyhow!("saved HTTP proxy host is missing"))?;
        let http_port = state
            .http_port
            .ok_or_else(|| anyhow!("saved HTTP proxy port is missing"))?;
        let socks_host = state
            .socks_host
            .as_deref()
            .ok_or_else(|| anyhow!("saved SOCKS proxy host is missing"))?;
        let socks_port = state
            .socks_port
            .ok_or_else(|| anyhow!("saved SOCKS proxy port is missing"))?;
        let https_host = state
            .https_host
            .as_deref()
            .ok_or_else(|| anyhow!("saved HTTPS proxy host is missing"))?;
        let https_port = state
            .https_port
            .ok_or_else(|| anyhow!("saved HTTPS proxy port is missing"))?;
        let plan = SystemProxyPlan {
            http_host: http_host.to_string(),
            http_port,
            socks_host: socks_host.to_string(),
            socks_port,
            bypass_domains: state.bypass_domains.clone(),
        };
        self.apply(&plan).await?;
        set_gsettings("org.gnome.system.proxy.https", "host", https_host).await?;
        set_gsettings(
            "org.gnome.system.proxy.https",
            "port",
            &https_port.to_string(),
        )
        .await
    }
}

pub struct MacOSNetworkSetup;

impl SystemProxy for MacOSNetworkSetup {
    async fn set_enabled(&self, enabled: bool) -> Result<()> {
        let state = if enabled { "on" } else { "off" };
        let status = Command::new("networksetup")
            .args(["-setwebproxystate", "Wi-Fi", state])
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("failed to set macOS network proxy state");
        }
        Ok(())
    }
}

pub enum ProxyBackend {
    Linux(LinuxGSettings),
    MacOS(MacOSNetworkSetup),
}

impl ProxyBackend {
    pub async fn preflight_tun(&self, plan: &TunPlan) -> Result<()> {
        if plan.interface_name.trim().is_empty() || plan.address.trim().is_empty() {
            anyhow::bail!("TUN interface name and address are required");
        }
        match self {
            Self::Linux(_) => {
                let tun_device = std::path::Path::new("/dev/net/tun");
                if !tun_device.exists() {
                    anyhow::bail!("TUN device {} is unavailable", tun_device.display());
                }
                if Command::new("ip")
                    .arg("-V")
                    .output()
                    .await?
                    .status
                    .success()
                {
                    Ok(())
                } else {
                    anyhow::bail!("iproute2 is required for Linux TUN mode")
                }
            }
            Self::MacOS(_) => anyhow::bail!("macOS TUN backend is not available safely yet"),
        }
    }

    pub async fn capture(&self) -> Result<SystemProxyState> {
        match self {
            Self::Linux(backend) => backend.capture().await,
            Self::MacOS(_) => anyhow::bail!("macOS proxy snapshot is not implemented safely yet"),
        }
    }

    pub async fn apply_system_proxy(&self, plan: &SystemProxyPlan) -> Result<()> {
        match self {
            Self::Linux(backend) => backend.apply(plan).await,
            Self::MacOS(_) => {
                anyhow::bail!("macOS proxy transaction is not implemented safely yet")
            }
        }
    }

    pub async fn restore(&self, snapshot: &SystemProxyState) -> Result<()> {
        match self {
            Self::Linux(backend) => backend.restore(snapshot).await,
            Self::MacOS(_) => anyhow::bail!("macOS proxy restore is not implemented safely yet"),
        }
    }
}

impl SystemProxy for ProxyBackend {
    async fn set_enabled(&self, enabled: bool) -> Result<()> {
        match self {
            Self::Linux(backend) => backend.set_enabled(enabled).await,
            Self::MacOS(backend) => backend.set_enabled(enabled).await,
        }
    }
}

async fn gsettings_get(schema: &str, key: &str) -> Result<String> {
    let output = Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!("failed to read gsettings {schema} {key}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn set_gsettings(schema: &str, key: &str, value: &str) -> Result<()> {
    run_gsettings(["set", schema, key, value]).await
}

async fn run_gsettings<const N: usize>(args: [&str; N]) -> Result<()> {
    let status = Command::new("gsettings").args(args).status().await?;
    if !status.success() {
        anyhow::bail!("gsettings command failed");
    }
    Ok(())
}

fn parse_string(value: &str) -> String {
    value.trim().trim_matches('\'').to_string()
}

fn parse_port(value: &str) -> Result<u16> {
    value
        .split_whitespace()
        .last()
        .unwrap_or_default()
        .parse()
        .map_err(|error| anyhow!("invalid proxy port {value:?}: {error}"))
}

fn parse_ignore_hosts(value: &str) -> Vec<String> {
    value
        .split('\'')
        .enumerate()
        .filter(|(index, _)| index % 2 == 1)
        .map(|(_, part)| part.to_string())
        .collect()
}

fn format_gvariant_strings(values: &[String]) -> String {
    let escaped = values
        .iter()
        .map(|value| format!("'{}'", value.replace('\'', "\\'")))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{escaped}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gsettings_values_without_fallbacks() {
        assert_eq!(parse_string("'127.0.0.1'\n"), "127.0.0.1");
        assert_eq!(parse_port("uint16 7890").unwrap(), 7890);
        assert_eq!(
            parse_ignore_hosts("['localhost', '127.0.0.1']"),
            ["localhost", "127.0.0.1"]
        );
    }

    #[test]
    fn formats_bypass_hosts_as_gvariant_array() {
        assert_eq!(
            format_gvariant_strings(&["localhost".into()]),
            "['localhost']"
        );
    }
}
