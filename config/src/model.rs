use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NaryaConfig {
    pub subscriptions: Vec<Subscription>,
    pub groups: Vec<ProxyGroup>,
    pub rules: Vec<Rule>,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Subscription {
    pub name: String,
    pub url: String,
    pub tag: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProxyGroup {
    pub name: String,
    pub group_type: GroupType,
    pub proxies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GroupType {
    Select,
    UrlTest,
    Fallback,
    LoadBalance,
    Relay,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub rule_type: RuleType,
    pub payload: Vec<String>,
    pub action: Action,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RuleType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    DomainRegex,
    IpCidr,
    Geoip,
    ProcessName,
    PackageName,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Direct,
    Proxy(String), // Proxy group name
    Reject,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Settings {
    pub tun: TunSettings,
    pub system_proxy: bool,
    pub mixed_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TunSettings {
    pub enabled: bool,
    pub stack: String, // gVisor, System, etc.
    pub device: String,
    pub auto_route: bool,
}

impl Default for NaryaConfig {
    fn default() -> Self {
        Self {
            subscriptions: vec![],
            groups: vec![],
            rules: vec![],
            settings: Settings {
                tun: TunSettings {
                    enabled: false,
                    stack: "gvisor".to_string(),
                    device: "utun".to_string(),
                    auto_route: true,
                },
                system_proxy: true,
                mixed_port: 7890,
            },
        }
    }
}
