use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyMode {
    Disabled,
    SystemProxy,
    Tun,
}

impl ProxyMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::SystemProxy => "system_proxy",
            Self::Tun => "tun",
        }
    }
}

impl std::str::FromStr for ProxyMode {
    type Err = PlatformError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "disabled" => Ok(Self::Disabled),
            "system_proxy" => Ok(Self::SystemProxy),
            "tun" => Ok(Self::Tun),
            _ => Err(PlatformError::Unsupported("unknown routing mode")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemProxyPlan {
    pub http_host: String,
    pub http_port: u16,
    pub socks_host: String,
    pub socks_port: u16,
    pub bypass_domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunPlan {
    pub interface_name: String,
    pub address: String,
    pub auto_route: bool,
    pub strict_route: bool,
    pub hijack_dns: bool,
    pub excluded_routes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsPlan {
    pub resolver: Vec<String>,
    pub direct: Vec<String>,
    pub proxy: Vec<String>,
    pub hijack: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingPlan {
    pub mode: ProxyMode,
    pub system_proxy: SystemProxyPlan,
    pub tun: Option<TunPlan>,
    pub dns: DnsPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemProxyState {
    pub enabled: bool,
    pub http_host: Option<String>,
    pub http_port: Option<u16>,
    pub https_host: Option<String>,
    pub https_port: Option<u16>,
    pub socks_host: Option<String>,
    pub socks_port: Option<u16>,
    pub bypass_domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunState {
    pub enabled: bool,
    pub interface_name: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformSnapshot {
    pub system_proxy: SystemProxyState,
    pub tun: TunState,
    pub dns: DnsPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    Unsupported(&'static str),
    Apply(String),
    Rollback(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(feature) => write!(f, "platform feature is unsupported: {feature}"),
            Self::Apply(error) => write!(f, "platform apply failed: {error}"),
            Self::Rollback(error) => write!(f, "platform rollback failed: {error}"),
        }
    }
}

impl std::error::Error for PlatformError {}

pub trait PlatformAdapter {
    fn snapshot(&mut self) -> Result<PlatformSnapshot, PlatformError>;
    fn apply_system_proxy(&mut self, plan: &SystemProxyPlan) -> Result<(), PlatformError>;
    fn apply_tun(&mut self, plan: &TunPlan) -> Result<(), PlatformError>;
    fn apply_dns(&mut self, plan: &DnsPlan) -> Result<(), PlatformError>;
    fn restore(&mut self, snapshot: &PlatformSnapshot) -> Result<(), PlatformError>;
}

pub fn apply_routing<A: PlatformAdapter>(
    adapter: &mut A,
    plan: &RoutingPlan,
) -> Result<PlatformSnapshot, PlatformError> {
    let snapshot = adapter.snapshot()?;
    let apply_result = (|| {
        match plan.mode {
            ProxyMode::Disabled => {}
            ProxyMode::SystemProxy => adapter.apply_system_proxy(&plan.system_proxy)?,
            ProxyMode::Tun => {
                let tun = plan
                    .tun
                    .as_ref()
                    .ok_or(PlatformError::Apply("TUN mode requires a TUN plan".into()))?;
                adapter.apply_tun(tun)?;
            }
        }
        adapter.apply_dns(&plan.dns)
    })();

    match apply_result {
        Ok(()) => Ok(snapshot),
        Err(error) => match adapter.restore(&snapshot) {
            Ok(()) => Err(error),
            Err(rollback) => Err(PlatformError::Rollback(format!(
                "{error}; restore failed: {rollback}"
            ))),
        },
    }
}

pub fn restore_routing<A: PlatformAdapter>(
    adapter: &mut A,
    snapshot: &PlatformSnapshot,
) -> Result<(), PlatformError> {
    adapter.restore(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(mode: ProxyMode) -> RoutingPlan {
        RoutingPlan {
            mode,
            system_proxy: SystemProxyPlan {
                http_host: "127.0.0.1".into(),
                http_port: 2080,
                socks_host: "127.0.0.1".into(),
                socks_port: 1080,
                bypass_domains: vec!["localhost".into()],
            },
            tun: Some(TunPlan {
                interface_name: "narya0".into(),
                address: "198.18.0.1/30".into(),
                auto_route: true,
                strict_route: true,
                hijack_dns: true,
                excluded_routes: vec!["127.0.0.0/8".into()],
            }),
            dns: DnsPlan {
                resolver: vec!["https://1.1.1.1/dns-query".into()],
                direct: vec!["local".into()],
                proxy: vec!["https://8.8.8.8/dns-query".into()],
                hijack: true,
            },
        }
    }

    #[derive(Debug)]
    struct FakeAdapter {
        state: PlatformSnapshot,
        fail_on: Option<&'static str>,
        restored: bool,
    }

    impl FakeAdapter {
        fn new() -> Self {
            Self {
                state: PlatformSnapshot {
                    system_proxy: SystemProxyState {
                        enabled: false,
                        http_host: None,
                        http_port: None,
                        https_host: None,
                        https_port: None,
                        socks_host: None,
                        socks_port: None,
                        bypass_domains: Vec::new(),
                    },
                    tun: TunState {
                        enabled: false,
                        interface_name: None,
                        address: None,
                    },
                    dns: DnsPlan {
                        resolver: vec!["system".into()],
                        direct: vec![],
                        proxy: vec![],
                        hijack: false,
                    },
                },
                fail_on: None,
                restored: false,
            }
        }

        fn fail_on(mut self, step: &'static str) -> Self {
            self.fail_on = Some(step);
            self
        }

        fn should_fail(&self, step: &'static str) -> Result<(), PlatformError> {
            if self.fail_on == Some(step) {
                Err(PlatformError::Apply(step.into()))
            } else {
                Ok(())
            }
        }
    }

    impl PlatformAdapter for FakeAdapter {
        fn snapshot(&mut self) -> Result<PlatformSnapshot, PlatformError> {
            Ok(self.state.clone())
        }

        fn apply_system_proxy(&mut self, plan: &SystemProxyPlan) -> Result<(), PlatformError> {
            self.should_fail("proxy")?;
            self.state.system_proxy = SystemProxyState {
                enabled: true,
                http_host: Some(plan.http_host.clone()),
                http_port: Some(plan.http_port),
                https_host: Some(plan.http_host.clone()),
                https_port: Some(plan.http_port),
                socks_host: Some(plan.socks_host.clone()),
                socks_port: Some(plan.socks_port),
                bypass_domains: plan.bypass_domains.clone(),
            };
            Ok(())
        }

        fn apply_tun(&mut self, plan: &TunPlan) -> Result<(), PlatformError> {
            self.should_fail("tun")?;
            self.state.tun = TunState {
                enabled: true,
                interface_name: Some(plan.interface_name.clone()),
                address: Some(plan.address.clone()),
            };
            Ok(())
        }

        fn apply_dns(&mut self, plan: &DnsPlan) -> Result<(), PlatformError> {
            self.should_fail("dns")?;
            self.state.dns = plan.clone();
            Ok(())
        }

        fn restore(&mut self, snapshot: &PlatformSnapshot) -> Result<(), PlatformError> {
            self.state = snapshot.clone();
            self.restored = true;
            Ok(())
        }
    }

    #[test]
    fn successful_system_proxy_apply_returns_restore_snapshot() {
        let mut adapter = FakeAdapter::new();
        let snapshot = apply_routing(&mut adapter, &plan(ProxyMode::SystemProxy)).unwrap();
        assert!(!snapshot.system_proxy.enabled);
        assert!(adapter.state.system_proxy.enabled);
        assert!(adapter.state.dns.hijack);
        restore_routing(&mut adapter, &snapshot).unwrap();
        assert!(!adapter.state.system_proxy.enabled);
    }

    #[test]
    fn dns_failure_restores_proxy_and_dns() {
        let mut adapter = FakeAdapter::new().fail_on("dns");
        let error = apply_routing(&mut adapter, &plan(ProxyMode::SystemProxy)).unwrap_err();
        assert!(matches!(error, PlatformError::Apply(_)));
        assert!(adapter.restored);
        assert!(!adapter.state.system_proxy.enabled);
        assert!(!adapter.state.dns.hijack);
    }

    #[test]
    fn tun_requires_explicit_plan() {
        let mut adapter = FakeAdapter::new();
        let mut config = plan(ProxyMode::Tun);
        config.tun = None;
        let error = apply_routing(&mut adapter, &config).unwrap_err();
        assert!(matches!(error, PlatformError::Apply(_)));
        assert!(adapter.restored);
    }

    #[test]
    fn tun_failure_rolls_back_without_touching_dns() {
        let mut adapter = FakeAdapter::new().fail_on("tun");
        let error = apply_routing(&mut adapter, &plan(ProxyMode::Tun)).unwrap_err();
        assert!(matches!(error, PlatformError::Apply(_)));
        assert!(adapter.restored);
        assert!(!adapter.state.tun.enabled);
        assert!(!adapter.state.dns.hijack);
    }
}
