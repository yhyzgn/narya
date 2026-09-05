use super::{
    split_host_port, split_shadowsocks_credentials, validate_routing_plan,
    validate_rule_set_references, RoutingConfig,
};
use anyhow::{anyhow, bail, Result};
use narya_core::Node;
use narya_platform::ProxyMode;
use narya_rules::{Action, Condition};
use serde_json::{json, Map, Value};

pub(super) fn generate(node: &Node, config: &RoutingConfig) -> Result<Value> {
    validate_routing_plan(config)?;
    validate_rule_set_references(&config.rules, &config.rule_sets)?;
    if config.mode == ProxyMode::Tun {
        bail!("xray-core adapter does not support TUN mode yet");
    }
    let proxy = proxy_outbound(node)?;
    let outbounds = vec![
        proxy,
        json!({"protocol": "freedom", "tag": "direct"}),
        json!({"protocol": "blackhole", "tag": "block"}),
    ];

    let mut balancers = Vec::new();
    for group in &config.groups {
        group
            .validate()
            .map_err(|error| anyhow!("group {} is invalid: {error}", group.id))?;
        balancers.push(json!({
            "tag": group.id,
            "selector": group.members,
        }));
    }

    let mut routing_rules = Vec::new();
    for rule in config.rules.rules() {
        if rule.conditions.len() != 1 {
            bail!(
                "rule {}: xray adapter requires one condition per rule",
                rule.id
            );
        }
        let mut value = Map::new();
        match &rule.conditions[0] {
            Condition::Domain(domain) | Condition::DomainSuffix(domain) => {
                value.insert("domain".into(), json!([format!("domain:{domain}")]))
            }
            Condition::IpCidr { network, prefix } => {
                value.insert("ip".into(), json!([format!("{network}/{prefix}")]))
            }
            Condition::RuleSet(_) => bail!(
                "rule {}: xray adapter requires downloaded rule-set translation",
                rule.id
            ),
            Condition::Any => None,
            Condition::Port(_) | Condition::Process(_) => bail!(
                "rule {}: xray adapter does not support port/process conditions",
                rule.id
            ),
        };
        match &rule.action {
            Action::Proxy(group) => {
                value.insert("balancerTag".into(), Value::String(group.clone()));
            }
            Action::Direct => {
                value.insert("outboundTag".into(), Value::String("direct".into()));
            }
            Action::Block => {
                value.insert("outboundTag".into(), Value::String("block".into()));
            }
            Action::Dns(_) => bail!(
                "rule {}: xray adapter does not support DNS actions",
                rule.id
            ),
        }
        routing_rules.push(Value::Object(value));
    }
    routing_rules.push(json!({"outboundTag": "block"}));

    let mut root = json!({
        "log": {"loglevel": "warning"},
        "inbounds": [
            {"listen": config.plan.system_proxy.socks_host, "port": config.plan.system_proxy.socks_port, "protocol": "socks", "settings": {"udp": true}},
            {"listen": config.plan.system_proxy.http_host, "port": config.plan.system_proxy.http_port, "protocol": "http"}
        ],
        "outbounds": outbounds,
        "routing": {"domainStrategy": "AsIs", "rules": routing_rules}
    });
    if !balancers.is_empty() {
        root["routing"]["balancers"] = Value::Array(balancers);
    }
    Ok(root)
}

fn proxy_outbound(node: &Node) -> Result<Value> {
    let (server, port) = split_host_port(&node.details.address)?;
    let protocol = node.protocol.to_ascii_lowercase();
    let mut outbound = match protocol.as_str() {
        "shadowsocks" | "ss" => {
            let (method, password) = split_shadowsocks_credentials(&node.details.encryption)?;
            json!({
                "protocol": "shadowsocks",
                "tag": "proxy-node",
                "settings": {"servers": [{"address": server, "port": port, "method": method, "password": password}]}
            })
        }
        "vmess" => json!({
            "protocol": "vmess",
            "tag": "proxy-node",
            "settings": {"vnext": [{"address": server, "port": port, "users": [{"id": credential(&node.details.encryption, "uuid")?, "security": node.details.options.vmess_security.as_deref().unwrap_or("auto") }]}]}
        }),
        "vless" | "vless reality" => json!({
            "protocol": "vless",
            "tag": "proxy-node",
            "settings": {"vnext": [{"address": server, "port": port, "users": [{
                "id": credential(&node.details.encryption, "uuid")?,
                "encryption": "none"
            }]}]}
        }),
        "trojan" => json!({
            "protocol": "trojan",
            "tag": "proxy-node",
            "settings": {"servers": [{"address": server, "port": port, "password": credential(&node.details.encryption, "password")?}]}
        }),
        other => bail!("xray-core adapter does not support proxy protocol {other}"),
    };
    if let Some(stream) = stream_settings(node, server)? {
        outbound["streamSettings"] = stream;
    }
    Ok(outbound)
}

fn credential<'a>(value: &'a str, prefix: &str) -> Result<&'a str> {
    value
        .strip_prefix(&format!("{prefix}:"))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("node is missing {prefix} credential"))
}

fn stream_settings(node: &Node, server: &str) -> Result<Option<Value>> {
    let transport = node.details.transport.to_ascii_lowercase();
    let options = &node.details.options;
    let mut stream = Map::new();
    let mut has_stream = false;

    match transport.as_str() {
        "" | "tcp" => {}
        "ws" | "websocket" => {
            has_stream = true;
            stream.insert("method".into(), json!("websocket"));
            let mut ws = Map::new();
            if let Some(path) = &options.transport_path {
                ws.insert("path".into(), json!(path));
            }
            if let Some(host) = &options.transport_host {
                ws.insert("host".into(), json!(host));
            }
            stream.insert("wsSettings".into(), Value::Object(ws));
        }
        "grpc" => {
            has_stream = true;
            stream.insert("method".into(), json!("grpc"));
            let service_name = options
                .grpc_service_name
                .as_deref()
                .ok_or_else(|| anyhow!("gRPC transport requires a service name"))?;
            let mut grpc = Map::new();
            if let Some(authority) = &options.transport_host {
                grpc.insert("authority".into(), json!(authority));
            }
            grpc.insert("serviceName".into(), json!(service_name));
            stream.insert("grpcSettings".into(), Value::Object(grpc));
        }
        other => bail!("xray-core adapter does not support transport {other}"),
    }

    let has_reality = options.reality_public_key.is_some() || options.reality_short_id.is_some();
    let has_tls = node.details.tls
        || node.details.skip_cert_verify
        || options.server_name.is_some()
        || !options.alpn.is_empty();
    if has_reality {
        if !matches!(transport.as_str(), "" | "tcp" | "grpc") {
            bail!("xray-core REALITY requires RAW or gRPC transport");
        }
        has_stream = true;
        stream.insert("security".into(), json!("reality"));
        let mut reality = Map::new();
        reality.insert(
            "serverName".into(),
            json!(options
                .server_name
                .clone()
                .unwrap_or_else(|| server.to_string())),
        );
        reality.insert("fingerprint".into(), json!("chrome"));
        if let Some(public_key) = &options.reality_public_key {
            reality.insert("password".into(), json!(public_key));
        }
        if let Some(short_id) = &options.reality_short_id {
            reality.insert("shortId".into(), json!(short_id));
        }
        stream.insert("realitySettings".into(), Value::Object(reality));
    } else if has_tls {
        has_stream = true;
        stream.insert("security".into(), json!("tls"));
        let mut tls = Map::new();
        tls.insert(
            "serverName".into(),
            json!(options
                .server_name
                .clone()
                .unwrap_or_else(|| server.to_string())),
        );
        if !options.alpn.is_empty() {
            tls.insert("alpn".into(), json!(options.alpn));
        }
        if node.details.skip_cert_verify {
            tls.insert("allowInsecure".into(), json!(true));
        }
        stream.insert("tlsSettings".into(), Value::Object(tls));
    }

    if has_stream {
        Ok(Some(Value::Object(stream)))
    } else {
        Ok(None)
    }
}
