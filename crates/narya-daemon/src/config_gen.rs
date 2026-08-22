use anyhow::{anyhow, bail, Result};
use narya_core::Node;
use narya_platform::{ProxyMode, RoutingPlan};
use narya_rules::{Action, Condition, RuleSet, RuleSetSource};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::net::IpAddr;

/// Explicit DNS paths used by the generated sing-box configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsConfig {
    pub resolver: Vec<String>,
    pub direct: Vec<String>,
    pub proxy: Vec<String>,
    pub outbound: Vec<String>,
    pub final_server: DnsServerTarget,
    pub hijack: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsServerTarget {
    Resolver,
    Direct,
    Proxy,
    Outbound,
}

impl DnsServerTarget {
    fn tag(self) -> &'static str {
        match self {
            Self::Resolver => "dns-resolver",
            Self::Direct => "dns-direct",
            Self::Proxy => "dns-proxy",
            Self::Outbound => "dns-outbound",
        }
    }
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            resolver: vec!["https://1.1.1.1/dns-query".into()],
            direct: vec!["local".into()],
            proxy: vec!["https://1.1.1.1/dns-query".into()],
            outbound: vec!["https://8.8.8.8/dns-query".into()],
            final_server: DnsServerTarget::Proxy,
            hijack: false,
        }
    }
}

/// All inputs needed to compile the shared routing semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub mode: ProxyMode,
    pub plan: RoutingPlan,
    pub rules: RuleSet,
    pub rule_sets: Vec<RuleSetSource>,
    pub dns: DnsConfig,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Disabled,
            plan: RoutingPlan {
                mode: ProxyMode::Disabled,
                system_proxy: narya_platform::SystemProxyPlan {
                    http_host: "127.0.0.1".into(),
                    http_port: 2080,
                    socks_host: "127.0.0.1".into(),
                    socks_port: 1080,
                    bypass_domains: vec!["localhost".into(), "127.0.0.1".into(), "::1".into()],
                },
                tun: None,
                dns: narya_platform::DnsPlan {
                    resolver: vec!["https://1.1.1.1/dns-query".into()],
                    direct: vec!["local".into()],
                    proxy: vec!["https://1.1.1.1/dns-query".into()],
                    hijack: false,
                },
            },
            rules: RuleSet::empty(),
            rule_sets: Vec::new(),
            dns: DnsConfig::default(),
        }
    }
}

pub struct ConfigGenerator;

impl ConfigGenerator {
    /// Compile a node and the shared routing model for sing-box.
    pub fn generate_json_with_config(node: &Node, config: &RoutingConfig) -> Result<Value> {
        if config.mode != config.plan.mode {
            bail!(
                "routing config mode mismatch: mode={} plan={}",
                config.mode.as_str(),
                config.plan.mode.as_str()
            );
        }

        let proxy = proxy_outbound(node)?;
        let mut inbounds = vec![
            json!({
                "type": "socks",
                "tag": "socks-in",
                "listen": "127.0.0.1",
                "listen_port": config.plan.system_proxy.socks_port
            }),
            json!({
                "type": "http",
                "tag": "http-in",
                "listen": "127.0.0.1",
                "listen_port": config.plan.system_proxy.http_port
            }),
        ];
        if config.mode == ProxyMode::Tun {
            let tun = config
                .plan
                .tun
                .as_ref()
                .ok_or_else(|| anyhow!("TUN mode requires an explicit TUN plan"))?;
            inbounds.push(tun_inbound(tun)?);
        }

        let dns = effective_dns_config(config);
        let rules = compile_route_rules(&config.rules, dns.hijack)?;
        let outbounds = vec![
            proxy,
            json!({"type": "direct", "tag": "direct"}),
            json!({"type": "block", "tag": "block"}),
            json!({"type": "dns", "tag": "dns-out"}),
        ];

        let mut root = Map::new();
        root.insert("log".into(), json!({"level": "info", "timestamp": true}));
        root.insert("inbounds".into(), Value::Array(inbounds));
        root.insert("outbounds".into(), Value::Array(outbounds));
        root.insert(
            "dns".into(),
            dns_config(&dns, &compile_dns_rules(&config.rules, &dns)?)?,
        );
        let mut route = json!({
            "rules": rules,
            // No implicit direct fallback: an unmatched request is rejected.
            "final": "block",
            "auto_detect_interface": config.mode == ProxyMode::Tun
        });
        if !config.rule_sets.is_empty() {
            route["rule_set"] = rule_set_metadata(&config.rule_sets)?;
        }
        root.insert("route".into(), route);
        Ok(Value::Object(root))
    }
}

fn effective_dns_config(config: &RoutingConfig) -> DnsConfig {
    let mut dns = config.dns.clone();
    if config.plan.tun.as_ref().is_some_and(|tun| tun.hijack_dns) {
        dns.hijack = true;
    }
    dns
}

fn proxy_outbound(node: &Node) -> Result<Value> {
    let (server, port) = split_host_port(&node.details.address)?;
    match node.protocol.to_ascii_lowercase().as_str() {
        "shadowsocks" | "ss" => {
            let (method, password) = split_shadowsocks_credentials(&node.details.encryption)?;
            Ok(json!({
                "type": "shadowsocks",
                "tag": "proxy",
                "server": server,
                "server_port": port,
                "method": method,
                "password": password,
                "udp_over_tcp": false
            }))
        }
        protocol => Err(anyhow!(
            "unsupported proxy protocol for sing-box config generation: {protocol}"
        )),
    }
}

fn compile_route_rules(rules: &RuleSet, hijack_dns: bool) -> Result<Vec<Value>> {
    let mut compiled = Vec::with_capacity(rules.rules().len() + usize::from(hijack_dns));
    if hijack_dns {
        compiled.push(json!({"protocol": "dns", "action": "hijack-dns"}));
    }
    for rule in rules.rules() {
        if matches!(rule.action, Action::Dns(_)) {
            continue;
        }
        let mut value = Map::new();
        for condition in &rule.conditions {
            append_condition(&mut value, condition, &rule.id)?;
        }
        append_action(&mut value, &rule.action, &rule.id)?;
        compiled.push(Value::Object(value));
    }
    Ok(compiled)
}

fn compile_dns_rules(rules: &RuleSet, dns: &DnsConfig) -> Result<Vec<Value>> {
    let mut compiled = Vec::new();
    if dns.hijack {
        compiled.push(json!({"action": "hijack-dns"}));
    }
    for rule in rules.rules() {
        let Action::Dns(target_name) = &rule.action else {
            continue;
        };
        let server = match target_name.as_str() {
            "resolver" => DnsServerTarget::Resolver,
            "direct" => DnsServerTarget::Direct,
            "proxy" => DnsServerTarget::Proxy,
            "outbound" => DnsServerTarget::Outbound,
            other => bail!("rule {}: unsupported DNS target {other:?}", rule.id),
        };
        if dns_servers(dns, server).is_empty() {
            bail!(
                "rule {}: DNS target {target_name:?} has no configured servers",
                rule.id
            );
        }
        let mut value = Map::new();
        for condition in &rule.conditions {
            append_dns_condition(&mut value, condition, &rule.id)?;
        }
        value.insert("server".into(), Value::String(server.tag().into()));
        compiled.push(Value::Object(value));
    }
    Ok(compiled)
}

fn append_dns_condition(
    target: &mut Map<String, Value>,
    condition: &Condition,
    rule_id: &str,
) -> Result<()> {
    match condition {
        Condition::Domain(value) => append_string(target, "domain", value),
        Condition::DomainSuffix(value) => append_string(target, "domain_suffix", value),
        Condition::IpCidr { network, prefix } => {
            let max = if matches!(network, IpAddr::V4(_)) {
                32
            } else {
                128
            };
            if *prefix > max {
                bail!("rule {rule_id}: unsupported ip_cidr prefix {prefix}");
            }
            append_string(target, "ip_cidr", &format!("{network}/{prefix}"))
        }
        Condition::Any => Ok(()),
        Condition::Port(_) | Condition::Process(_) => {
            bail!("rule {rule_id}: DNS action does not support port/process conditions")
        }
    }
}

fn append_condition(
    target: &mut Map<String, Value>,
    condition: &Condition,
    rule_id: &str,
) -> Result<()> {
    match condition {
        Condition::Any => Ok(()),
        Condition::Domain(value) => append_string(target, "domain", value),
        Condition::DomainSuffix(value) => append_string(target, "domain_suffix", value),
        Condition::IpCidr { network, prefix } => {
            let max = if matches!(network, IpAddr::V4(_)) {
                32
            } else {
                128
            };
            if *prefix > max {
                bail!("rule {rule_id}: unsupported ip_cidr prefix {prefix}");
            }
            append_string(target, "ip_cidr", &format!("{network}/{prefix}"))
        }
        Condition::Port(port) => {
            target.insert("port".into(), json!(*port));
            Ok(())
        }
        Condition::Process(process) => append_string(target, "process_name", process),
    }
}

fn append_string(target: &mut Map<String, Value>, key: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("condition {key} must not be empty");
    }
    target.insert(
        key.into(),
        Value::Array(vec![Value::String(value.to_string())]),
    );
    Ok(())
}

fn append_action(target: &mut Map<String, Value>, action: &Action, rule_id: &str) -> Result<()> {
    match action {
        Action::Proxy(tag) => {
            if tag != "proxy" {
                bail!("rule {rule_id}: proxy outbound {tag:?} is not configured");
            }
            target.insert("outbound".into(), Value::String(tag.clone()));
        }
        Action::Direct => {
            target.insert("outbound".into(), Value::String("direct".into()));
        }
        Action::Block => {
            target.insert("action".into(), Value::String("reject".into()));
        }
        Action::Dns(_) => bail!("rule {rule_id}: DNS action must be compiled as a DNS rule"),
    }
    Ok(())
}

fn dns_config(config: &DnsConfig, rules: &[Value]) -> Result<Value> {
    let mut servers = Vec::new();
    for (target, detour) in [
        (DnsServerTarget::Resolver, "direct"),
        (DnsServerTarget::Direct, "direct"),
        (DnsServerTarget::Proxy, "proxy"),
        (DnsServerTarget::Outbound, "proxy"),
    ] {
        let addresses = dns_servers(config, target);
        if addresses.is_empty() {
            bail!("DNS target {} has no configured servers", target.tag());
        }
        for (index, address) in addresses.iter().enumerate() {
            let tag = if addresses.len() == 1 {
                target.tag().to_string()
            } else {
                format!("{}-{index}", target.tag())
            };
            servers.push(json!({"tag": tag, "address": address, "detour": detour}));
        }
    }
    let mut result = json!({
        "servers": servers,
        "final": config.final_server.tag(),
        "strategy": "prefer_ipv4"
    });
    if !rules.is_empty() {
        result["rules"] = Value::Array(rules.to_vec());
    }
    Ok(result)
}

fn dns_servers(config: &DnsConfig, target: DnsServerTarget) -> &[String] {
    match target {
        DnsServerTarget::Resolver => &config.resolver,
        DnsServerTarget::Direct => &config.direct,
        DnsServerTarget::Proxy => &config.proxy,
        DnsServerTarget::Outbound => &config.outbound,
    }
}

fn tun_inbound(plan: &narya_platform::TunPlan) -> Result<Value> {
    if plan.interface_name.trim().is_empty() || plan.address.trim().is_empty() {
        bail!("TUN interface_name and address are required");
    }
    Ok(json!({
        "type": "tun",
        "tag": "tun-in",
        "interface_name": plan.interface_name,
        "address": [plan.address],
        "auto_route": plan.auto_route,
        "strict_route": plan.strict_route,
        "route_exclude_address": plan.excluded_routes,
        "stack": "system"
    }))
}

fn rule_set_metadata(rule_sets: &[RuleSetSource]) -> Result<Value> {
    let values = rule_sets
        .iter()
        .map(|source| {
            source
                .validate()
                .map_err(|error| anyhow!("ruleset {} is invalid: {error}", source.id))?;
            Ok(json!({
                "tag": source.id,
                "format": "binary",
                "url": source.source,
                "download_detour": "direct",
                "version": source.version,
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Value::Array(values))
}

fn split_host_port(address: &str) -> Result<(&str, u16)> {
    let (host, port) = address
        .rsplit_once(':')
        .ok_or_else(|| anyhow!("node address must include host:port"))?;
    if host.trim().is_empty() {
        return Err(anyhow!("node address host is empty"));
    }
    let port = port.parse::<u16>()?;
    Ok((host, port))
}

fn split_shadowsocks_credentials(encryption: &str) -> Result<(&str, &str)> {
    let (method, password) = encryption
        .split_once(':')
        .ok_or_else(|| anyhow!("Shadowsocks credentials must be stored as method:password"))?;
    if method.trim().is_empty() || method == "none" {
        return Err(anyhow!("Shadowsocks node is missing encryption method"));
    }
    if password.is_empty() {
        return Err(anyhow!("Shadowsocks node is missing password"));
    }
    Ok((method, password))
}

#[cfg(test)]
mod tests {
    use super::*;
    use narya_core::{Node, NodeDetails};
    use narya_rules::{Rule, RuleSet};
    use std::net::{IpAddr, Ipv4Addr};

    fn node(protocol: &str, encryption: &str) -> Node {
        Node {
            id: "test".to_string(),
            name: "Test".to_string(),
            country_code: "UN".to_string(),
            protocol: protocol.to_string(),
            tag: None,
            latency: None,
            usage_pct: 0,
            download_speed: 0.0,
            upload_speed: 0.0,
            details: NodeDetails {
                address: "example.com:8388".to_string(),
                encryption: encryption.to_string(),
                udp: true,
                tls: false,
                skip_cert_verify: false,
                transport: "tcp".to_string(),
                last_test: "Never".to_string(),
            },
        }
    }

    fn routing(mode: ProxyMode) -> RoutingConfig {
        let mut config = RoutingConfig {
            mode,
            ..RoutingConfig::default()
        };
        config.plan.mode = mode;
        config.plan.tun = (mode == ProxyMode::Tun).then(|| narya_platform::TunPlan {
            interface_name: "narya0".into(),
            address: "198.18.0.1/30".into(),
            auto_route: true,
            strict_route: true,
            hijack_dns: true,
            excluded_routes: vec!["127.0.0.0/8".into()],
        });
        config
    }

    #[test]
    fn shadowsocks_uses_distinct_method_and_password() {
        let config = ConfigGenerator::generate_json_with_config(
            &node("ss", "aes-256-gcm:secret"),
            &RoutingConfig::default(),
        )
        .unwrap();
        let outbound = &config["outbounds"][0];
        assert_eq!(outbound["method"], "aes-256-gcm");
        assert_eq!(outbound["password"], "secret");
        assert_eq!(config["route"]["final"], "block");
    }

    #[test]
    fn shared_rule_order_is_identical_for_system_proxy_and_tun() {
        let mut system = routing(ProxyMode::SystemProxy);
        system.rules = RuleSet::compile(vec![
            Rule {
                id: "proxy-example".into(),
                priority: 20,
                conditions: vec![Condition::DomainSuffix("example.com".into())],
                action: Action::Proxy("proxy".into()),
            },
            Rule {
                id: "private".into(),
                priority: 10,
                conditions: vec![Condition::IpCidr {
                    network: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)),
                    prefix: 8,
                }],
                action: Action::Direct,
            },
        ])
        .unwrap();
        let mut tun = system.clone();
        tun.mode = ProxyMode::Tun;
        tun.plan.mode = ProxyMode::Tun;
        tun.plan.tun = routing(ProxyMode::Tun).plan.tun;
        let proxy_rules =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &system)
                .unwrap()["route"]["rules"]
                .clone();
        let tun_rules =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &tun)
                .unwrap()["route"]["rules"]
                .clone();
        assert_eq!(
            proxy_rules.as_array().unwrap(),
            &tun_rules.as_array().unwrap()[1..]
        );
        assert_eq!(proxy_rules[0]["ip_cidr"][0], "10.0.0.0/8");
        assert_eq!(proxy_rules[1]["domain_suffix"][0], "example.com");
    }

    #[test]
    fn tun_and_dns_paths_are_explicit() {
        let mut config = routing(ProxyMode::Tun);
        config.dns.hijack = true;
        config.rules = RuleSet::compile(vec![Rule {
            id: "cn-dns".into(),
            priority: 1,
            conditions: vec![Condition::DomainSuffix("cn".into())],
            action: Action::Dns("direct".into()),
        }])
        .unwrap();
        let generated =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap();
        assert_eq!(generated["inbounds"][2]["type"], "tun");
        assert_eq!(generated["route"]["rules"][0]["action"], "hijack-dns");
        assert_eq!(generated["dns"]["rules"][1]["server"], "dns-direct");
        assert_eq!(generated["dns"]["final"], "dns-proxy");
        assert!(generated["dns"]["servers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["tag"] == "dns-outbound"));
    }

    #[test]
    fn unsupported_proxy_target_fails_closed_with_rule_id() {
        let mut config = routing(ProxyMode::SystemProxy);
        config.rules = RuleSet::compile(vec![Rule {
            id: "unknown-outbound".into(),
            priority: 1,
            conditions: vec![Condition::Any],
            action: Action::Proxy("not-configured".into()),
        }])
        .unwrap();
        let error =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap_err();
        assert!(error.to_string().contains("unknown-outbound"));
    }

    #[test]
    fn ruleset_metadata_requires_integrity_fields() {
        let mut config = routing(ProxyMode::SystemProxy);
        config.rule_sets = vec![RuleSetSource {
            id: "geosite-cn".into(),
            source: "https://rules.invalid/geosite-cn.srs".into(),
            version: "2026-08-22".into(),
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        }];
        let generated =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap();
        assert_eq!(generated["route"]["rule_set"][0]["version"], "2026-08-22");
    }

    #[test]
    fn shadowsocks_missing_password_fails_closed() {
        let err = ConfigGenerator::generate_json_with_config(
            &node("ss", "aes-256-gcm"),
            &RoutingConfig::default(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("method:password"));
    }

    #[test]
    fn unsupported_protocol_fails_closed() {
        let err = ConfigGenerator::generate_json_with_config(
            &node("vmess", "auto"),
            &RoutingConfig::default(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("unsupported proxy protocol"));
    }
}
