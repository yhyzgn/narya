use anyhow::{anyhow, bail, Context, Result};
use narya_core::Node;
use narya_kernel::KernelId;
use narya_platform::{ProxyMode, RoutingPlan, SystemProxyPlan};
use narya_rules::{Action, Condition, GroupStrategy, RoutingGroup, RuleSet, RuleSetSource};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::net::IpAddr;

mod mihomo;
mod xray;

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
    pub groups: Vec<RoutingGroup>,
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
            groups: vec![RoutingGroup::default_proxy()],
            rule_sets: Vec::new(),
            dns: DnsConfig::default(),
        }
    }
}

pub struct ConfigGenerator;

/// Rule-set bytes must be verified by Narya before a kernel is allowed to
/// consume them. Remote URLs are intentionally rejected here; a future
/// managed downloader can populate a verified local cache without delegating
/// trust to an arbitrary kernel process.
pub fn validate_rule_set_sources(rule_sets: &[RuleSetSource]) -> Result<()> {
    for source in rule_sets {
        source
            .validate()
            .map_err(|error| anyhow!("ruleset {} is invalid: {error}", source.id))?;
        if !source.enabled {
            continue;
        }
        let path = if source.source.starts_with("https://") {
            narya_ipc::ruleset_cache_dir()
                .join(&source.id)
                .join("current")
        } else {
            let source_path = source
                .source
                .strip_prefix("file://")
                .unwrap_or(&source.source);
            let source_path = std::path::Path::new(source_path);
            if !source_path.is_absolute() {
                bail!(
                    "ruleset {}: source must be an absolute local path",
                    source.id
                );
            }
            source_path.to_path_buf()
        };
        let bytes = std::fs::read(&path).with_context(|| {
            format!(
                "ruleset {}: verified cache is unavailable at {}",
                source.id,
                path.display()
            )
        })?;
        let actual = format!("{:x}", Sha256::digest(bytes));
        if !actual.eq_ignore_ascii_case(&source.sha256) {
            bail!(
                "ruleset {}: cached checksum mismatch, expected {}, got {}",
                source.id,
                source.sha256,
                actual
            );
        }
    }
    Ok(())
}

impl ConfigGenerator {
    /// Compile a node and the shared routing model for sing-box.
    pub fn generate_json_with_config(node: &Node, config: &RoutingConfig) -> Result<Value> {
        validate_routing_plan(config)?;

        let group_tags = config
            .groups
            .iter()
            .map(|group| group.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let proxy_tag = if config.groups.is_empty() {
            "proxy"
        } else {
            "proxy-node"
        };
        let proxy = proxy_outbound(node, proxy_tag)?;
        validate_rule_set_references(&config.rules, &config.rule_sets)?;
        let mut inbounds = vec![
            json!({
                "type": "socks",
                "tag": "socks-in",
                "listen": config.plan.system_proxy.socks_host,
                "listen_port": config.plan.system_proxy.socks_port
            }),
            json!({
                "type": "http",
                "tag": "http-in",
                "listen": config.plan.system_proxy.http_host,
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
        let rules = compile_route_rules(&config.rules, dns.hijack, &group_tags)?;
        let mut outbounds = vec![proxy];
        outbounds.extend(compile_groups(&config.groups)?);
        outbounds.extend([
            json!({"type": "direct", "tag": "direct"}),
            json!({"type": "block", "tag": "block"}),
            json!({"type": "dns", "tag": "dns-out"}),
        ]);

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
        if config.rule_sets.iter().any(|source| source.enabled) {
            route["rule_set"] = rule_set_metadata(&config.rule_sets)?;
        }
        root.insert("route".into(), route);
        Ok(Value::Object(root))
    }

    /// Compile the shared model for every kernel that Narya can start. Each
    /// adapter owns its schema translation but reuses the same validated rule
    /// and group semantics; unsupported conditions fail with the rule ID.
    pub fn generate_json_for_kernel(
        kernel: KernelId,
        node: &Node,
        config: &RoutingConfig,
    ) -> Result<Value> {
        match kernel {
            KernelId::SingBox => Self::generate_json_with_config(node, config),
            KernelId::Mihomo => mihomo::generate(node, config),
            KernelId::Xray | KernelId::V2Ray => xray::generate(node, config),
        }
    }
}

fn validate_system_proxy_plan(plan: &SystemProxyPlan) -> Result<()> {
    if plan.http_port == 0 || plan.socks_port == 0 {
        bail!("system proxy listener ports must be non-zero");
    }
    if plan.http_host != plan.socks_host {
        bail!("HTTP and SOCKS proxy listeners must use the same local bind host");
    }
    let host = plan.http_host.trim();
    let is_loopback = host == "localhost"
        || host
            .parse::<IpAddr>()
            .ok()
            .is_some_and(|address| address.is_loopback());
    if !is_loopback {
        bail!("system proxy listeners must bind to a loopback host");
    }
    Ok(())
}

fn validate_routing_plan(config: &RoutingConfig) -> Result<()> {
    if config.mode != config.plan.mode {
        bail!(
            "routing config mode mismatch: mode={} plan={}",
            config.mode.as_str(),
            config.plan.mode.as_str()
        );
    }
    if config.mode == ProxyMode::Tun && config.plan.tun.is_none() {
        bail!("TUN mode requires an explicit TUN plan");
    }
    if config.mode != ProxyMode::Tun && config.plan.tun.is_some() {
        bail!("TUN plan is only valid when routing mode is TUN");
    }
    validate_system_proxy_plan(&config.plan.system_proxy)
}

fn effective_dns_config(config: &RoutingConfig) -> DnsConfig {
    let mut dns = config.dns.clone();
    if config.plan.tun.as_ref().is_some_and(|tun| tun.hijack_dns) {
        dns.hijack = true;
    }
    dns
}

fn validate_rule_set_references(rules: &RuleSet, sources: &[RuleSetSource]) -> Result<()> {
    let known = sources
        .iter()
        .filter(|source| source.enabled)
        .map(|source| source.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    for rule in rules.rules() {
        for condition in &rule.conditions {
            if let Condition::RuleSet(id) = condition {
                if !known.contains(id.as_str()) {
                    bail!("rule {} references unknown ruleset {id}", rule.id);
                }
            }
        }
    }
    Ok(())
}

fn proxy_outbound(node: &Node, tag: &str) -> Result<Value> {
    let (server, port) = split_host_port(&node.details.address)?;
    let options = &node.details.options;
    match node.protocol.to_ascii_lowercase().as_str() {
        "shadowsocks" | "ss" => {
            let (method, password) = split_shadowsocks_credentials(&node.details.encryption)?;
            Ok(json!({
                "type": "shadowsocks",
                "tag": tag,
                "server": server,
                "server_port": port,
                "method": method,
                "password": password,
                "udp_over_tcp": false
            }))
        }
        "hysteria2" | "hy2" => {
            let password = prefixed_credential(&node.details.encryption, "password")?;
            let mut value = json!({
                "type": "hysteria2",
                "tag": tag,
                "server": server,
                "server_port": port,
                "password": password,
                "tls": {"enabled": node.details.tls, "insecure": node.details.skip_cert_verify}
            });
            apply_tls_options(&mut value, options);
            Ok(value)
        }
        "vmess" => {
            let uuid = prefixed_credential(&node.details.encryption, "uuid")?;
            let mut value = json!({
                "type": "vmess",
                "tag": tag,
                "server": server,
                "server_port": port,
                "uuid": uuid,
                "security": options.vmess_security.as_deref().unwrap_or("auto"),
                "tls": {"enabled": node.details.tls, "insecure": node.details.skip_cert_verify}
            });
            if options.vmess_alter_id > 0 {
                value["alter_id"] = json!(options.vmess_alter_id);
            }
            apply_transport_and_tls(&mut value, &node.details.transport, options)?;
            Ok(value)
        }
        "vless" | "vless reality" => {
            let uuid = prefixed_credential(&node.details.encryption, "uuid")?;
            let mut value = json!({
                "type": "vless",
                "tag": tag,
                "server": server,
                "server_port": port,
                "uuid": uuid,
                "tls": {"enabled": node.details.tls, "insecure": node.details.skip_cert_verify}
            });
            if let Some(flow) = &options.flow {
                value["flow"] = json!(flow);
            }
            apply_transport_and_tls(&mut value, &node.details.transport, options)?;
            Ok(value)
        }
        "trojan" => {
            let password = prefixed_credential(&node.details.encryption, "password")?;
            let mut value = json!({
                "type": "trojan",
                "tag": tag,
                "server": server,
                "server_port": port,
                "password": password,
                "tls": {"enabled": node.details.tls, "insecure": node.details.skip_cert_verify}
            });
            apply_tls_options(&mut value, options);
            Ok(value)
        }
        protocol => Err(anyhow!(
            "unsupported proxy protocol for sing-box config generation: {protocol}"
        )),
    }
}

fn prefixed_credential<'a>(value: &'a str, prefix: &str) -> Result<&'a str> {
    let credential = value
        .strip_prefix(&format!("{prefix}:"))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("node is missing {prefix} credential"))?;
    Ok(credential)
}

fn sing_box_transport(transport: &str) -> Result<Value> {
    match transport.to_ascii_lowercase().as_str() {
        "tcp" | "" => Ok(json!({"type": "tcp"})),
        "ws" | "websocket" => Ok(json!({"type": "ws"})),
        "grpc" => Ok(json!({"type": "grpc"})),
        other => bail!("unsupported sing-box transport {other}"),
    }
}

fn apply_transport_and_tls(
    value: &mut Value,
    transport: &str,
    options: &narya_core::ProtocolOptions,
) -> Result<()> {
    if !transport.eq_ignore_ascii_case("tcp") && !transport.is_empty() {
        let mut transport_value = sing_box_transport(transport)?;
        if let Some(path) = &options.transport_path {
            transport_value["path"] = json!(path);
        }
        if let Some(host) = &options.transport_host {
            transport_value["headers"] = json!({"Host": host});
        }
        if let Some(service) = &options.grpc_service_name {
            transport_value["service_name"] = json!(service);
        }
        value["transport"] = transport_value;
    }
    apply_tls_options(value, options);
    Ok(())
}

fn apply_tls_options(value: &mut Value, options: &narya_core::ProtocolOptions) {
    if let Some(server_name) = &options.server_name {
        value["tls"]["server_name"] = json!(server_name);
    }
    if !options.alpn.is_empty() {
        value["tls"]["alpn"] = json!(options.alpn);
    }
    if options.reality_public_key.is_some() || options.reality_short_id.is_some() {
        let mut reality = json!({"enabled": true});
        if let Some(key) = &options.reality_public_key {
            reality["public_key"] = json!(key);
        }
        if let Some(id) = &options.reality_short_id {
            reality["short_id"] = json!(id);
        }
        value["tls"]["reality"] = reality;
    }
}

fn compile_route_rules(
    rules: &RuleSet,
    hijack_dns: bool,
    group_tags: &std::collections::HashSet<&str>,
) -> Result<Vec<Value>> {
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
        append_action(&mut value, &rule.action, &rule.id, group_tags)?;
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
        Condition::RuleSet(value) => append_string(target, "rule_set", value),
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
        Condition::RuleSet(value) => append_string(target, "rule_set", value),
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

fn append_action(
    target: &mut Map<String, Value>,
    action: &Action,
    rule_id: &str,
    group_tags: &std::collections::HashSet<&str>,
) -> Result<()> {
    match action {
        Action::Proxy(tag) => {
            if !group_tags.is_empty() && !group_tags.contains(tag.as_str()) {
                bail!("rule {rule_id}: proxy outbound {tag:?} is not configured");
            }
            target.insert(
                "outbound".into(),
                Value::String(if group_tags.is_empty() {
                    "proxy".into()
                } else {
                    tag.clone()
                }),
            );
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

fn compile_groups(groups: &[RoutingGroup]) -> Result<Vec<Value>> {
    let mut tags = std::collections::HashSet::new();
    let mut values = Vec::with_capacity(groups.len());
    for group in groups {
        group
            .validate()
            .map_err(|error| anyhow!("group {} is invalid: {error}", group.id))?;
        if !tags.insert(group.id.as_str()) {
            bail!("duplicate outbound group tag {}", group.id);
        }
        let kind = match group.strategy {
            GroupStrategy::Select => "selector",
            GroupStrategy::UrlTest => "urltest",
            GroupStrategy::Fallback => "fallback",
            GroupStrategy::LoadBalance => "loadbalance",
        };
        let mut value = json!({
            "type": kind,
            "tag": group.id,
            "outbounds": group.members,
        });
        if let Some(url) = &group.url {
            value["url"] = Value::String(url.clone());
        }
        if let Some(interval) = group.interval_secs {
            value["interval"] = Value::String(format!("{interval}s"));
        }
        values.push(value);
    }
    Ok(values)
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
        .filter(|source| source.enabled)
        .map(|source| {
            source
                .validate()
                .map_err(|error| anyhow!("ruleset {} is invalid: {error}", source.id))?;
            Ok(json!({
                "tag": source.id,
                "format": source.format.sing_box_value(),
                "url": if source.source.starts_with("https://") {
                    format!("file://{}", narya_ipc::ruleset_cache_dir().join(&source.id).join("current").display())
                } else {
                    source.source.clone()
                },
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
    use narya_rules::{GroupStrategy, RoutingGroup, Rule, RuleSet, RuleSetFormat, RuleSetSource};
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
                options: narya_core::ProtocolOptions::default(),
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
            format: narya_rules::RuleSetFormat::Domain,
            enabled: true,
            signature: "11".repeat(64),
            public_key: "22".repeat(32),
        }];
        let generated =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap();
        assert_eq!(generated["route"]["rule_set"][0]["version"], "2026-08-22");
    }

    #[test]
    fn disabled_ruleset_is_excluded_and_referenced_disable_fails_closed() {
        let mut config = routing(ProxyMode::SystemProxy);
        config.rule_sets = vec![RuleSetSource {
            id: "disabled-set".into(),
            source: "/tmp/disabled-set.db".into(),
            version: "1".into(),
            sha256: "aa".repeat(32),
            format: narya_rules::RuleSetFormat::Domain,
            enabled: false,
            signature: String::new(),
            public_key: String::new(),
        }];
        let generated =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap();
        assert!(generated["route"].get("rule_set").is_none());

        config.rules = RuleSet::compile(vec![Rule {
            id: "uses-disabled".into(),
            priority: 1,
            conditions: vec![Condition::RuleSet("disabled-set".into())],
            action: Action::Block,
        }])
        .unwrap();
        let error =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap_err();
        assert!(error.to_string().contains("unknown ruleset"));
    }

    #[test]
    fn compiles_selector_group_and_rejects_unknown_group_target() {
        let mut config = RoutingConfig::default();
        config.groups.push(RoutingGroup {
            id: "streaming".into(),
            strategy: GroupStrategy::UrlTest,
            members: vec!["proxy-node".into()],
            url: Some("https://www.gstatic.com/generate_204".into()),
            interval_secs: Some(300),
        });
        config.rules = RuleSet::compile(vec![Rule {
            id: "streaming-rule".into(),
            priority: 10,
            conditions: vec![Condition::DomainSuffix("video.example".into())],
            action: Action::Proxy("streaming".into()),
        }])
        .unwrap();
        let generated =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap();
        assert!(generated["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .any(|outbound| { outbound["tag"] == "streaming" && outbound["type"] == "urltest" }));
        let mut invalid = config;
        invalid.rules = RuleSet::compile(vec![Rule {
            id: "unknown-group".into(),
            priority: 1,
            conditions: vec![Condition::Any],
            action: Action::Proxy("missing".into()),
        }])
        .unwrap();
        assert!(ConfigGenerator::generate_json_with_config(
            &node("ss", "aes-256-gcm:secret"),
            &invalid
        )
        .unwrap_err()
        .to_string()
        .contains("unknown-group"));
    }

    #[test]
    fn compiles_mihomo_and_xray_system_proxy_configs() {
        let config = RoutingConfig::default();
        let node = node("ss", "aes-256-gcm:secret");
        let mihomo =
            ConfigGenerator::generate_json_for_kernel(KernelId::Mihomo, &node, &config).unwrap();
        assert_eq!(mihomo["mode"], "rule");
        assert_eq!(mihomo["proxy-groups"][0]["name"], "proxy");
        let xray =
            ConfigGenerator::generate_json_for_kernel(KernelId::Xray, &node, &config).unwrap();
        assert_eq!(xray["routing"]["balancers"][0]["tag"], "proxy");
    }

    #[test]
    fn proxy_listeners_must_be_loopback_and_share_bind_host() {
        let mut config = RoutingConfig::default();
        config.plan.system_proxy.http_host = "0.0.0.0".into();
        config.plan.system_proxy.socks_host = "0.0.0.0".into();
        let error =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap_err();
        assert!(error.to_string().contains("loopback"));

        config.plan.system_proxy.http_host = "127.0.0.1".into();
        config.plan.system_proxy.socks_host = "127.0.0.2".into();
        let error =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap_err();
        assert!(error.to_string().contains("same local bind host"));
    }

    #[test]
    fn every_kernel_adapter_rejects_routing_plan_mismatch() {
        let mut config = RoutingConfig {
            mode: ProxyMode::Tun,
            ..RoutingConfig::default()
        };
        let node = node("ss", "aes-256-gcm:secret");
        for kernel in KernelId::ALL {
            let error =
                ConfigGenerator::generate_json_for_kernel(kernel, &node, &config).unwrap_err();
            assert!(error.to_string().contains("routing config mode mismatch"));
        }

        config.mode = ProxyMode::SystemProxy;
        config.plan.mode = ProxyMode::SystemProxy;
        config.plan.tun = Some(narya_platform::TunPlan {
            interface_name: "narya0".into(),
            address: "172.19.0.1/30".into(),
            auto_route: true,
            strict_route: true,
            hijack_dns: true,
            excluded_routes: Vec::new(),
        });
        let error = ConfigGenerator::generate_json_for_kernel(KernelId::Mihomo, &node, &config)
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("only valid when routing mode is TUN"));
    }

    #[test]
    fn mihomo_rule_provider_is_generated_for_text_ruleset() {
        let config = RoutingConfig {
            rule_sets: vec![RuleSetSource {
                id: "geosite-ai".into(),
                source: "/var/lib/narya/geosite-ai.txt".into(),
                version: "1".into(),
                sha256: "aa".repeat(32),
                format: RuleSetFormat::Domain,
                enabled: true,
                signature: String::new(),
                public_key: String::new(),
            }],
            rules: RuleSet::compile(vec![Rule {
                id: "ai".into(),
                priority: 1,
                conditions: vec![Condition::RuleSet("geosite-ai".into())],
                action: Action::Proxy("proxy".into()),
            }])
            .unwrap(),
            ..RoutingConfig::default()
        };
        let generated = ConfigGenerator::generate_json_for_kernel(
            KernelId::Mihomo,
            &node("ss", "aes-256-gcm:secret"),
            &config,
        )
        .unwrap();
        assert_eq!(
            generated["rule-providers"]["geosite-ai"]["behavior"],
            "domain"
        );
        assert_eq!(generated["rule-providers"]["geosite-ai"]["type"], "file");
        assert!(generated["rules"][0]
            .as_str()
            .unwrap()
            .starts_with("RULE-SET,geosite-ai"));
    }

    #[test]
    fn mihomo_rejects_sing_box_binary_ruleset() {
        let config = RoutingConfig {
            rule_sets: vec![RuleSetSource {
                id: "binary-set".into(),
                source: "/var/lib/narya/binary.mrs".into(),
                version: "1".into(),
                sha256: "bb".repeat(32),
                format: RuleSetFormat::SingBoxBinary,
                enabled: true,
                signature: String::new(),
                public_key: String::new(),
            }],
            rules: RuleSet::compile(vec![Rule {
                id: "binary-rule".into(),
                priority: 1,
                conditions: vec![Condition::RuleSet("binary-set".into())],
                action: Action::Block,
            }])
            .unwrap(),
            ..RoutingConfig::default()
        };
        let error = ConfigGenerator::generate_json_for_kernel(
            KernelId::Mihomo,
            &node("ss", "aes-256-gcm:secret"),
            &config,
        )
        .unwrap_err();
        assert!(error.to_string().contains("mihomo cannot consume"));
    }

    #[test]
    fn xray_tun_is_rejected_instead_of_falling_back_to_system_proxy() {
        let config = routing(ProxyMode::Tun);
        let error = ConfigGenerator::generate_json_for_kernel(
            KernelId::Xray,
            &node("ss", "aes-256-gcm:secret"),
            &config,
        )
        .unwrap_err();
        assert!(error.to_string().contains("does not support TUN"));
    }

    #[test]
    fn ruleset_conditions_require_declared_verified_metadata() {
        let config = RoutingConfig {
            rules: RuleSet::compile(vec![Rule {
                id: "geo-rule".into(),
                priority: 1,
                conditions: vec![Condition::RuleSet("geoip-private".into())],
                action: Action::Direct,
            }])
            .unwrap(),
            ..RoutingConfig::default()
        };
        let error =
            ConfigGenerator::generate_json_with_config(&node("ss", "aes-256-gcm:secret"), &config)
                .unwrap_err();
        assert!(error.to_string().contains("unknown ruleset"));
    }

    #[test]
    fn remote_ruleset_sources_are_rejected_until_locally_verified() {
        let source = RuleSetSource {
            id: "geoip".into(),
            source: "https://example.invalid/geoip.db".into(),
            version: "1".into(),
            sha256: "a".repeat(64),
            format: narya_rules::RuleSetFormat::IpCidr,
            enabled: true,
            signature: "11".repeat(64),
            public_key: "22".repeat(32),
        };
        let error = validate_rule_set_sources(&[source]).unwrap_err();
        assert!(error.to_string().contains("verified cache is unavailable"));
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
            &node("wireguard", "auto"),
            &RoutingConfig::default(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("unsupported proxy protocol"));
    }

    #[test]
    fn sing_box_supports_subscription_protocols_with_explicit_credentials() {
        let cases = [
            ("hysteria2", "password:secret", "hysteria2"),
            (
                "vmess",
                "uuid:00000000-0000-0000-0000-000000000001",
                "vmess",
            ),
            (
                "vless",
                "uuid:00000000-0000-0000-0000-000000000002",
                "vless",
            ),
            ("trojan", "password:secret", "trojan"),
        ];
        for (protocol, credentials, expected_type) in cases {
            let config = ConfigGenerator::generate_json_with_config(
                &node(protocol, credentials),
                &RoutingConfig::default(),
            )
            .unwrap();
            assert_eq!(config["outbounds"][0]["type"], expected_type);
        }
    }

    #[test]
    fn mihomo_and_xray_compile_supported_subscription_protocols() {
        let mihomo = ConfigGenerator::generate_json_for_kernel(
            KernelId::Mihomo,
            &node("hysteria2", "password:secret"),
            &RoutingConfig::default(),
        )
        .unwrap();
        assert_eq!(mihomo["proxies"][0]["type"], "hysteria2");

        for protocol in ["vmess", "vless"] {
            let xray = ConfigGenerator::generate_json_for_kernel(
                KernelId::Xray,
                &node(protocol, "uuid:00000000-0000-0000-0000-000000000005"),
                &RoutingConfig::default(),
            )
            .unwrap();
            assert_eq!(xray["outbounds"][0]["protocol"], protocol);
        }
        let xray = ConfigGenerator::generate_json_for_kernel(
            KernelId::Xray,
            &node("trojan", "password:secret"),
            &RoutingConfig::default(),
        )
        .unwrap();
        assert_eq!(xray["outbounds"][0]["protocol"], "trojan");
    }

    #[test]
    fn mihomo_emits_tls_reality_ws_and_grpc_fields() {
        let mut node = node("vless", "uuid:00000000-0000-0000-0000-000000000006");
        node.details.tls = true;
        node.details.transport = "ws".into();
        node.details.options = narya_core::ProtocolOptions {
            server_name: Some("sni.example.com".into()),
            alpn: vec!["h2".into(), "http/1.1".into()],
            transport_path: Some("/ws".into()),
            transport_host: Some("ws.example.com".into()),
            grpc_service_name: Some("liora".into()),
            flow: Some("xtls-rprx-vision".into()),
            reality_public_key: Some("pubkey".into()),
            reality_short_id: Some("sid".into()),
            ..narya_core::ProtocolOptions::default()
        };
        let ws = ConfigGenerator::generate_json_for_kernel(
            KernelId::Mihomo,
            &node,
            &RoutingConfig::default(),
        )
        .unwrap();
        assert_eq!(ws["proxies"][0]["servername"], "sni.example.com");
        assert_eq!(ws["proxies"][0]["alpn"][0], "h2");
        assert_eq!(ws["proxies"][0]["ws-opts"]["path"], "/ws");
        assert_eq!(
            ws["proxies"][0]["ws-opts"]["headers"]["Host"],
            "ws.example.com"
        );
        assert_eq!(ws["proxies"][0]["reality-opts"]["public-key"], "pubkey");
        assert_eq!(ws["proxies"][0]["reality-opts"]["short-id"], "sid");

        node.details.transport = "grpc".into();
        let grpc = ConfigGenerator::generate_json_for_kernel(
            KernelId::Mihomo,
            &node,
            &RoutingConfig::default(),
        )
        .unwrap();
        assert_eq!(grpc["proxies"][0]["network"], "grpc");
        assert_eq!(
            grpc["proxies"][0]["grpc-opts"]["grpc-service-name"],
            "liora"
        );
    }

    #[test]
    fn sing_box_emits_tls_reality_ws_and_grpc_fields() {
        let mut node = node("vmess", "uuid:00000000-0000-0000-0000-000000000008");
        node.details.tls = true;
        node.details.transport = "grpc".into();
        node.details.options = narya_core::ProtocolOptions {
            server_name: Some("sb.example.com".into()),
            alpn: vec!["h2".into()],
            grpc_service_name: Some("sb-grpc".into()),
            reality_public_key: Some("sb-public".into()),
            reality_short_id: Some("sb-short".into()),
            vmess_security: Some("auto".into()),
            vmess_alter_id: 17,
            ..narya_core::ProtocolOptions::default()
        };
        let reality =
            ConfigGenerator::generate_json_with_config(&node, &RoutingConfig::default()).unwrap();
        assert_eq!(reality["outbounds"][0]["transport"]["type"], "grpc");
        assert_eq!(
            reality["outbounds"][0]["transport"]["service_name"],
            "sb-grpc"
        );
        assert_eq!(
            reality["outbounds"][0]["tls"]["server_name"],
            "sb.example.com"
        );
        assert_eq!(reality["outbounds"][0]["tls"]["alpn"][0], "h2");
        assert_eq!(
            reality["outbounds"][0]["tls"]["reality"]["public_key"],
            "sb-public"
        );
        assert_eq!(
            reality["outbounds"][0]["tls"]["reality"]["short_id"],
            "sb-short"
        );
    }

    #[test]
    fn xray_emits_stream_settings_for_tls_reality_ws_and_grpc() {
        let mut node = node("vless", "uuid:00000000-0000-0000-0000-000000000007");
        node.details.tls = true;
        node.details.transport = "grpc".into();
        node.details.options = narya_core::ProtocolOptions {
            server_name: Some("reality.example.com".into()),
            alpn: vec!["h2".into()],
            grpc_service_name: Some("liora".into()),
            transport_host: Some("authority.example.com".into()),
            reality_public_key: Some("public-key".into()),
            reality_short_id: Some("short-id".into()),
            ..narya_core::ProtocolOptions::default()
        };
        let reality = ConfigGenerator::generate_json_for_kernel(
            KernelId::Xray,
            &node,
            &RoutingConfig::default(),
        )
        .unwrap();
        assert_eq!(
            reality["outbounds"][0]["streamSettings"]["security"],
            "reality"
        );
        assert_eq!(
            reality["outbounds"][0]["streamSettings"]["realitySettings"]["password"],
            "public-key"
        );
        assert_eq!(
            reality["outbounds"][0]["streamSettings"]["realitySettings"]["shortId"],
            "short-id"
        );
        assert_eq!(
            reality["outbounds"][0]["streamSettings"]["grpcSettings"]["serviceName"],
            "liora"
        );

        node.details.transport = "ws".into();
        node.details.options.reality_public_key = None;
        node.details.options.reality_short_id = None;
        node.details.options.transport_path = Some("/ws".into());
        node.details.options.transport_host = Some("ws.example.com".into());
        let ws = ConfigGenerator::generate_json_for_kernel(
            KernelId::Xray,
            &node,
            &RoutingConfig::default(),
        )
        .unwrap();
        assert_eq!(ws["outbounds"][0]["streamSettings"]["security"], "tls");
        assert_eq!(
            ws["outbounds"][0]["streamSettings"]["tlsSettings"]["serverName"],
            "reality.example.com"
        );
        assert_eq!(
            ws["outbounds"][0]["streamSettings"]["wsSettings"]["path"],
            "/ws"
        );
        assert_eq!(
            ws["outbounds"][0]["streamSettings"]["wsSettings"]["host"],
            "ws.example.com"
        );
    }

    #[test]
    fn sing_box_rejects_protocol_without_required_credentials() {
        let error = ConfigGenerator::generate_json_with_config(
            &node("vless", "none"),
            &RoutingConfig::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("uuid credential"));
    }
}
