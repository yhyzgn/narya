use super::{
    split_host_port, split_shadowsocks_credentials, validate_routing_plan,
    validate_rule_set_references, RoutingConfig,
};
use anyhow::{anyhow, bail, Result};
use narya_core::Node;
use narya_rules::{Action, Condition, GroupStrategy, Rule, RuleSet, RuleSetSource};
use serde_json::{json, Map, Value};
use std::collections::HashSet;

pub(super) fn generate(node: &Node, config: &RoutingConfig) -> Result<Value> {
    validate_routing_plan(config)?;
    validate_rule_set_references(&config.rules, &config.rule_sets)?;
    let proxies = vec![proxy(node)?];
    let mut groups = Vec::new();
    for group in &config.groups {
        group
            .validate()
            .map_err(|error| anyhow!("group {} is invalid: {error}", group.id))?;
        let group_type = match group.strategy {
            GroupStrategy::Select => "select",
            GroupStrategy::UrlTest => "url-test",
            GroupStrategy::Fallback => "fallback",
            GroupStrategy::LoadBalance => "load-balance",
        };
        let mut value = json!({
            "name": group.id,
            "type": group_type,
            "proxies": group.members,
        });
        if let Some(url) = &group.url {
            value["url"] = Value::String(url.clone());
        }
        if let Some(interval) = group.interval_secs {
            value["interval"] = Value::Number(interval.into());
        }
        groups.push(value);
    }

    let mut rules = config
        .rules
        .rules()
        .iter()
        .map(compile_rule)
        .collect::<Result<Vec<_>>>()?;
    if rules.is_empty() || !rules.iter().any(|rule| rule.ends_with(",REJECT")) {
        rules.push("MATCH,REJECT".into());
    }

    let mut root = json!({
        "port": config.plan.system_proxy.http_port,
        "socks-port": config.plan.system_proxy.socks_port,
        "bind-address": config.plan.system_proxy.http_host,
        "allow-lan": false,
        "mode": "rule",
        "log-level": "info",
        "proxies": proxies,
        "proxy-groups": groups,
        "rules": rules,
        "dns": {
            "enable": true,
            "listen": "127.0.0.1:1053",
            "nameserver": config.dns.proxy,
            "fallback": config.dns.resolver,
            "respect-rules": true
        }
    });
    let providers = rule_providers(&config.rules, &config.rule_sets)?;
    if !providers.is_empty() {
        root["rule-providers"] = Value::Object(providers);
    }
    if let Some(tun) = &config.plan.tun {
        root["tun"] = json!({
            "enable": true,
            "stack": "system",
            "auto-route": tun.auto_route,
            "strict-route": tun.strict_route,
            "dns-hijack": if tun.hijack_dns { vec!["any:53"] } else { Vec::new() },
            "route-exclude-address": tun.excluded_routes,
        });
    }
    Ok(root)
}

fn rule_providers(rules: &RuleSet, sources: &[RuleSetSource]) -> Result<Map<String, Value>> {
    let referenced = rules
        .rules()
        .iter()
        .flat_map(|rule| rule.conditions.iter())
        .filter_map(|condition| match condition {
            Condition::RuleSet(id) => Some(id.as_str()),
            _ => None,
        })
        .collect::<HashSet<_>>();
    let mut providers = Map::new();
    for source in sources
        .iter()
        .filter(|source| source.enabled && referenced.contains(source.id.as_str()))
    {
        let behavior = source.format.mihomo_behavior().ok_or_else(|| {
            anyhow!(
                "ruleset {} uses sing-box binary format, which mihomo cannot consume",
                source.id
            )
        })?;
        providers.insert(
            source.id.clone(),
            json!({
                "type": "file",
                "behavior": behavior,
                "format": "text",
                "path": rule_set_local_path(source),
                "interval": 0
            }),
        );
    }
    Ok(providers)
}

fn rule_set_local_path(source: &RuleSetSource) -> String {
    if source.source.starts_with("https://") {
        narya_ipc::ruleset_cache_dir()
            .join(&source.id)
            .join("current")
            .display()
            .to_string()
    } else {
        source
            .source
            .strip_prefix("file://")
            .unwrap_or(&source.source)
            .to_string()
    }
}

fn proxy(node: &Node) -> Result<Value> {
    let (server, port) = split_host_port(&node.details.address)?;
    match node.protocol.to_ascii_lowercase().as_str() {
        "shadowsocks" | "ss" => {
            let (cipher, password) = split_shadowsocks_credentials(&node.details.encryption)?;
            Ok(json!({
                "name": "proxy-node",
                "type": "ss",
                "server": server,
                "port": port,
                "cipher": cipher,
                "password": password,
            }))
        }
        "vmess" => Ok(json!({
            "name": "proxy-node",
            "type": "vmess",
            "server": server,
            "port": port,
            "uuid": credential(&node.details.encryption, "uuid")?,
            "cipher": node.details.options.vmess_security.as_deref().unwrap_or("auto"),
            "alterId": node.details.options.vmess_alter_id,
            "tls": node.details.tls,
            "skip-cert-verify": node.details.skip_cert_verify,
            "udp": node.details.udp,
            "network": mihomo_network(&node.details.transport)?,
        }))
        .and_then(|mut value| {
            apply_tls_fields(&mut value, &node.details, &node.protocol)?;
            apply_transport_fields(&mut value, &node.details)?;
            Ok(value)
        }),
        "vless" | "vless reality" => Ok(json!({
            "name": "proxy-node",
            "type": "vless",
            "server": server,
            "port": port,
            "uuid": credential(&node.details.encryption, "uuid")?,
            "encryption": "none",
            "tls": node.details.tls,
            "skip-cert-verify": node.details.skip_cert_verify,
            "udp": node.details.udp,
            "network": mihomo_network(&node.details.transport)?,
        }))
        .and_then(|mut value| {
            if let Some(flow) = &node.details.options.flow {
                value["flow"] = json!(flow);
            }
            apply_tls_fields(&mut value, &node.details, &node.protocol)?;
            apply_transport_fields(&mut value, &node.details)?;
            Ok(value)
        }),
        "trojan" => Ok(json!({
            "name": "proxy-node",
            "type": "trojan",
            "server": server,
            "port": port,
            "password": credential(&node.details.encryption, "password")?,
            "skip-cert-verify": node.details.skip_cert_verify,
            "udp": node.details.udp,
            "tls": node.details.tls,
            "network": mihomo_network(&node.details.transport)?,
        }))
        .and_then(|mut value| {
            apply_tls_fields(&mut value, &node.details, &node.protocol)?;
            apply_transport_fields(&mut value, &node.details)?;
            Ok(value)
        }),
        "hysteria2" | "hy2" => Ok(json!({
            "name": "proxy-node",
            "type": "hysteria2",
            "server": server,
            "port": port,
            "password": credential(&node.details.encryption, "password")?,
            "skip-cert-verify": node.details.skip_cert_verify,
            "udp": node.details.udp,
            "tls": node.details.tls,
        }))
        .and_then(|mut value| {
            apply_tls_fields(&mut value, &node.details, &node.protocol)?;
            Ok(value)
        }),
        protocol => bail!("mihomo adapter does not support proxy protocol {protocol}"),
    }
}

fn apply_tls_fields(
    value: &mut Value,
    details: &narya_core::NodeDetails,
    protocol: &str,
) -> Result<()> {
    let protocol = protocol.to_ascii_lowercase();
    if let Some(server_name) = &details.options.server_name {
        let key = if protocol == "trojan" || protocol == "hysteria2" || protocol == "hy2" {
            "sni"
        } else {
            "servername"
        };
        value[key] = json!(server_name);
    }
    if !details.options.alpn.is_empty() {
        value["alpn"] = json!(details.options.alpn);
    }
    if protocol == "vmess"
        || protocol == "vless"
        || protocol == "vless reality"
        || protocol == "trojan"
    {
        if let Some(public_key) = &details.options.reality_public_key {
            let mut reality = Map::new();
            reality.insert("public-key".into(), json!(public_key));
            if let Some(short_id) = &details.options.reality_short_id {
                reality.insert("short-id".into(), json!(short_id));
            }
            value["reality-opts"] = Value::Object(reality);
        } else if let Some(short_id) = &details.options.reality_short_id {
            let mut reality = Map::new();
            reality.insert("short-id".into(), json!(short_id));
            value["reality-opts"] = Value::Object(reality);
        }
    }
    Ok(())
}

fn apply_transport_fields(value: &mut Value, details: &narya_core::NodeDetails) -> Result<()> {
    match mihomo_network(&details.transport)? {
        "tcp" => Ok(()),
        "ws" => {
            let mut ws = Map::new();
            if let Some(path) = &details.options.transport_path {
                ws.insert("path".into(), json!(path));
            }
            if let Some(host) = &details.options.transport_host {
                ws.insert("headers".into(), json!({"Host": host}));
            }
            value["ws-opts"] = Value::Object(ws);
            Ok(())
        }
        "grpc" => {
            let service = details
                .options
                .grpc_service_name
                .as_deref()
                .ok_or_else(|| anyhow!("gRPC transport requires a service name"))?;
            value["grpc-opts"] = json!({
                "grpc-service-name": service,
            });
            Ok(())
        }
        other => bail!("unsupported mihomo transport {other}"),
    }
}

fn credential<'a>(value: &'a str, prefix: &str) -> Result<&'a str> {
    value
        .strip_prefix(&format!("{prefix}:"))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("node is missing {prefix} credential"))
}

fn mihomo_network(transport: &str) -> Result<&'static str> {
    match transport.to_ascii_lowercase().as_str() {
        "tcp" | "" => Ok("tcp"),
        "ws" | "websocket" => Ok("ws"),
        "grpc" => Ok("grpc"),
        other => bail!("unsupported mihomo transport {other}"),
    }
}

fn compile_rule(rule: &Rule) -> Result<String> {
    if rule.conditions.len() != 1 {
        bail!(
            "rule {}: mihomo adapter requires one condition per rule",
            rule.id
        );
    }
    let target = match &rule.action {
        Action::Proxy(group) => group.clone(),
        Action::Direct => "DIRECT".into(),
        Action::Block => "REJECT".into(),
        Action::Dns(_) => bail!(
            "rule {}: mihomo adapter does not support DNS actions",
            rule.id
        ),
    };
    let condition = match &rule.conditions[0] {
        Condition::Domain(value) => format!("DOMAIN,{value},{target}"),
        Condition::DomainSuffix(value) => format!("DOMAIN-SUFFIX,{value},{target}"),
        Condition::IpCidr { network, prefix } => format!("IP-CIDR,{network}/{prefix},{target}"),
        Condition::RuleSet(value) => format!("RULE-SET,{value},{target}"),
        Condition::Any => format!("MATCH,{target}"),
        Condition::Port(_) | Condition::Process(_) => {
            bail!(
                "rule {}: mihomo adapter does not support port/process conditions",
                rule.id
            )
        }
    };
    Ok(condition)
}
