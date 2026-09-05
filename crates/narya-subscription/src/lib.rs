use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use narya_core::{Node, NodeDetails};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

const MAX_SUBSCRIPTION_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Deserialize, Serialize)]
struct ClashConfig {
    proxies: Option<Vec<serde_yaml::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SingBoxConfig {
    outbounds: Option<Vec<serde_json::Value>>,
}

fn yaml_string(value: &serde_yaml::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

fn yaml_strings(value: &serde_yaml::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|value| value.as_sequence())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn yaml_nested_string(value: &serde_yaml::Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionFormat {
    ClashYaml,
    SingBoxJson,
    V2RayBase64,
    Unknown,
}

impl SubscriptionFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionFormat::ClashYaml => "Clash YAML",
            SubscriptionFormat::SingBoxJson => "Sing-box JSON",
            SubscriptionFormat::V2RayBase64 => "V2Ray Base64",
            SubscriptionFormat::Unknown => "Unknown",
        }
    }
}

pub async fn fetch_remote_subscription(url: &str) -> Result<String> {
    let parsed = Url::parse(url).map_err(|e| anyhow!("Invalid subscription URL: {}", e))?;
    if parsed.scheme() != "https" {
        return Err(anyhow!("Only https subscription URLs are allowed"));
    }

    let client = reqwest::Client::builder()
        .user_agent("Clash/1.0 Narya/1.0")
        .timeout(Duration::from_secs(15))
        .build()?;

    let mut response = client.get(parsed).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "Subscription fetch failed with HTTP status {}",
            response.status()
        ));
    }

    let mut content = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        content.extend_from_slice(&chunk);
        if content.len() > MAX_SUBSCRIPTION_BYTES {
            return Err(anyhow!("Subscription exceeds 8 MiB limit"));
        }
    }

    let content = String::from_utf8(content)
        .map_err(|e| anyhow!("Subscription is not valid UTF-8: {}", e))?;
    Ok(content)
}

pub fn detect_format(content: &str) -> SubscriptionFormat {
    let content = content.trim();
    if content.starts_with('{') {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            if json.get("outbounds").is_some() {
                return SubscriptionFormat::SingBoxJson;
            }
        }
    }

    if (content.contains("proxies:") || content.contains("Proxy:"))
        && serde_yaml::from_str::<serde_yaml::Value>(content).is_ok()
    {
        return SubscriptionFormat::ClashYaml;
    }

    // Heuristics for Base64: typical base64 strings don't have spaces and are alphanumeric + '+', '/', '='
    // We check if it can be decoded as Base64 or if it looks like a list of URIs
    if content.lines().any(|l| {
        l.starts_with("vmess://")
            || l.starts_with("vless://")
            || l.starts_with("ss://")
            || l.starts_with("trojan://")
    }) {
        return SubscriptionFormat::V2RayBase64;
    }

    if general_purpose::STANDARD.decode(content).is_ok()
        || general_purpose::URL_SAFE_NO_PAD.decode(content).is_ok()
    {
        return SubscriptionFormat::V2RayBase64;
    }

    SubscriptionFormat::Unknown
}

pub fn parse_subscription(content: &str) -> Result<(SubscriptionFormat, Vec<Node>)> {
    if content.trim().is_empty() {
        return Err(anyhow!("Empty subscription content"));
    }

    let format = detect_format(content);
    let nodes = match format {
        SubscriptionFormat::ClashYaml => parse_clash_yaml(content)?,
        SubscriptionFormat::SingBoxJson => parse_singbox_json(content)?,
        SubscriptionFormat::V2RayBase64 => parse_v2ray_base64(content)?,
        SubscriptionFormat::Unknown => return Err(anyhow!("Unsupported subscription format")),
    };
    if nodes.is_empty() {
        return Err(anyhow!("Subscription parsed zero nodes"));
    }
    Ok((format, nodes))
}

pub fn parse_clash_yaml(content: &str) -> Result<Vec<Node>> {
    let config: ClashConfig =
        serde_yaml::from_str(content).map_err(|e| anyhow!("Clash YAML parse error: {}", e))?;
    let mut nodes = Vec::new();

    if let Some(proxies) = config.proxies {
        for p in proxies {
            let name = p
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Node")
                .to_string();
            let proxy_type = p
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let server = p
                .get("server")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let port = p.get("port").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
            let cipher = p
                .get("cipher")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let password = p
                .get("password")
                .or_else(|| p.get("passwd"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let uuid = p
                .get("uuid")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let udp = p.get("udp").and_then(|v| v.as_bool()).unwrap_or(false);
            let tls = p.get("tls").and_then(|v| v.as_bool()).unwrap_or(false);

            let network = p
                .get("network")
                .and_then(|v| v.as_str())
                .unwrap_or("tcp")
                .to_string();

            let (encryption, vmess_security) = if proxy_type.eq_ignore_ascii_case("ss")
                || proxy_type.eq_ignore_ascii_case("shadowsocks")
            {
                if !cipher.is_empty() && !password.is_empty() {
                    (format!("{}:{}", cipher, password), None)
                } else {
                    ("none".to_string(), None)
                }
            } else if proxy_type.eq_ignore_ascii_case("vmess")
                || proxy_type.eq_ignore_ascii_case("vless")
            {
                if !uuid.is_empty() {
                    (
                        format!("uuid:{uuid}"),
                        (!cipher.is_empty()).then_some(cipher.clone()),
                    )
                } else {
                    ("none".to_string(), None)
                }
            } else if proxy_type.eq_ignore_ascii_case("trojan")
                || proxy_type.eq_ignore_ascii_case("hysteria2")
                || proxy_type.eq_ignore_ascii_case("hy2")
            {
                if !password.is_empty() {
                    (format!("password:{password}"), None)
                } else {
                    ("none".to_string(), None)
                }
            } else if !cipher.is_empty() {
                (cipher.clone(), None)
            } else if !uuid.is_empty() {
                ("auto (uuid)".to_string(), None)
            } else {
                ("none".to_string(), None)
            };

            let details = NodeDetails {
                address: format!("{}:{}", server, port),
                encryption,
                udp,
                tls,
                skip_cert_verify: p
                    .get("skip-cert-verify")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                transport: network,
                last_test: "Never".to_string(),
                options: narya_core::ProtocolOptions {
                    server_name: yaml_string(&p, "servername").or_else(|| yaml_string(&p, "sni")),
                    alpn: yaml_strings(&p, "alpn"),
                    transport_path: yaml_nested_string(&p, &["ws-opts", "path"]),
                    transport_host: yaml_nested_string(&p, &["ws-opts", "headers", "Host"])
                        .or_else(|| yaml_nested_string(&p, &["ws-opts", "headers", "host"])),
                    grpc_service_name: yaml_nested_string(&p, &["grpc-opts", "grpc-service-name"]),
                    flow: yaml_string(&p, "flow"),
                    reality_public_key: yaml_nested_string(&p, &["reality-opts", "public-key"]),
                    reality_short_id: yaml_nested_string(&p, &["reality-opts", "short-id"]),
                    vmess_security,
                    vmess_alter_id: p
                        .get("alterId")
                        .or_else(|| p.get("alter-id"))
                        .and_then(|value| value.as_u64())
                        .and_then(|value| u32::try_from(value).ok())
                        .unwrap_or(0),
                },
            };

            nodes.push(Node {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                country_code: "UN".to_string(), // Can be extracted via regex on name later
                protocol: proxy_type,
                tag: None,
                latency: None,
                usage_pct: 0,
                download_speed: 0.0,
                upload_speed: 0.0,
                details,
            });
        }
    }

    Ok(nodes)
}

pub fn parse_singbox_json(content: &str) -> Result<Vec<Node>> {
    let config: SingBoxConfig =
        serde_json::from_str(content).map_err(|e| anyhow!("Sing-box JSON parse error: {}", e))?;
    let mut nodes = Vec::new();

    if let Some(outbounds) = config.outbounds {
        for o in outbounds {
            let out_type = o
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            // Skip internal routing outbounds
            if out_type == "direct"
                || out_type == "block"
                || out_type == "dns"
                || out_type == "selector"
                || out_type == "urltest"
            {
                continue;
            }

            let name = o
                .get("tag")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Node")
                .to_string();
            let server = o
                .get("server")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let port = o.get("server_port").and_then(|v| v.as_u64()).unwrap_or(0) as u16;

            let tls_obj = o.get("tls");
            let tls = tls_obj
                .map(|t| t.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false))
                .unwrap_or(false);

            let transport_obj = o.get("transport");
            let transport = transport_obj
                .and_then(|t| t.get("type").and_then(|v| v.as_str()))
                .unwrap_or("tcp")
                .to_string();

            let (credentials, vmess_security) = match out_type.as_str() {
                "shadowsocks" | "shadowsocksr" => {
                    let method = o.get("method").and_then(|v| v.as_str()).unwrap_or("");
                    let password = o.get("password").and_then(|v| v.as_str()).unwrap_or("");
                    if method.is_empty() || password.is_empty() {
                        ("none".to_string(), None)
                    } else {
                        (format!("{method}:{password}"), None)
                    }
                }
                "vmess" | "vless" => o
                    .get("uuid")
                    .and_then(|v| v.as_str())
                    .filter(|uuid| !uuid.is_empty())
                    .map(|uuid| {
                        (
                            format!("uuid:{uuid}"),
                            o.get("security")
                                .and_then(|value| value.as_str())
                                .map(ToOwned::to_owned),
                        )
                    })
                    .unwrap_or_else(|| ("none".to_string(), None)),
                "trojan" | "hysteria2" => o
                    .get("password")
                    .and_then(|v| v.as_str())
                    .filter(|password| !password.is_empty())
                    .map(|password| (format!("password:{password}"), None))
                    .unwrap_or_else(|| ("none".to_string(), None)),
                _ => ("auto".to_string(), None),
            };

            let details = NodeDetails {
                address: format!("{}:{}", server, port),
                encryption: credentials,
                udp: true, // Singbox usually enables UDP inherently depending on protocol
                tls,
                skip_cert_verify: tls_obj
                    .and_then(|t| t.get("insecure").and_then(|v| v.as_bool()))
                    .unwrap_or(false),
                transport,
                last_test: "Never".to_string(),
                options: narya_core::ProtocolOptions {
                    server_name: tls_obj
                        .and_then(|value| value.get("server_name"))
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned),
                    alpn: tls_obj
                        .and_then(|value| value.get("alpn"))
                        .and_then(|value| value.as_array())
                        .map(|values| {
                            values
                                .iter()
                                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                                .collect()
                        })
                        .unwrap_or_default(),
                    transport_path: transport_obj
                        .and_then(|value| value.get("path"))
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned),
                    transport_host: transport_obj
                        .and_then(|value| value.get("headers"))
                        .and_then(|value| value.get("Host").or_else(|| value.get("host")))
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned),
                    grpc_service_name: transport_obj
                        .and_then(|value| value.get("service_name"))
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned),
                    flow: o
                        .get("flow")
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned),
                    reality_public_key: tls_obj
                        .and_then(|value| value.get("reality"))
                        .and_then(|value| value.get("public_key"))
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned),
                    reality_short_id: tls_obj
                        .and_then(|value| value.get("reality"))
                        .and_then(|value| value.get("short_id"))
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned),
                    vmess_security,
                    vmess_alter_id: o
                        .get("alter_id")
                        .and_then(|value| value.as_u64())
                        .and_then(|value| u32::try_from(value).ok())
                        .unwrap_or(0),
                },
            };

            nodes.push(Node {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                country_code: "UN".to_string(),
                protocol: out_type,
                tag: None,
                latency: None,
                usage_pct: 0,
                download_speed: 0.0,
                upload_speed: 0.0,
                details,
            });
        }
    }

    Ok(nodes)
}

pub fn parse_v2ray_base64(content: &str) -> Result<Vec<Node>> {
    let content = content.trim();
    let decoded = if let Ok(bytes) = general_purpose::STANDARD.decode(content) {
        String::from_utf8_lossy(&bytes).to_string()
    } else if let Ok(bytes) = general_purpose::URL_SAFE_NO_PAD.decode(content) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        content.to_string() // Try parsing raw lines just in case it wasn't actually encoded
    };

    let mut nodes = Vec::new();

    for line in decoded.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(node) = parse_uri(line) {
            nodes.push(node);
        }
    }

    Ok(nodes)
}

fn parse_uri(uri: &str) -> Option<Node> {
    if uri.starts_with("vmess://") {
        let b64_payload = uri.trim_start_matches("vmess://");
        let payload = general_purpose::STANDARD
            .decode(b64_payload)
            .ok()
            .or_else(|| general_purpose::URL_SAFE_NO_PAD.decode(b64_payload).ok())?;
        let json_str = String::from_utf8_lossy(&payload);
        let vmess: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        let name = vmess
            .get("ps")
            .and_then(|v| v.as_str())
            .unwrap_or("Vmess Node")
            .to_string();
        let server = vmess
            .get("add")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let port = match vmess.get("port") {
            Some(v) if v.is_u64() => v.as_u64().unwrap_or(0) as u16,
            Some(v) if v.is_string() => v.as_str().unwrap().parse().unwrap_or(0),
            _ => 0,
        };
        let tls = vmess.get("tls").and_then(|v| v.as_str()).unwrap_or("") == "tls";
        let transport = vmess
            .get("net")
            .and_then(|v| v.as_str())
            .unwrap_or("tcp")
            .to_string();

        let details = NodeDetails {
            address: format!("{}:{}", server, port),
            encryption: vmess
                .get("id")
                .and_then(|value| value.as_str())
                .filter(|value| !value.is_empty())
                .map(|value| format!("uuid:{value}"))
                .unwrap_or_else(|| "none".to_string()),
            udp: true,
            tls,
            skip_cert_verify: false,
            transport,
            last_test: "Never".to_string(),
            options: narya_core::ProtocolOptions {
                server_name: vmess
                    .get("sni")
                    .and_then(|value| value.as_str())
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned),
                transport_path: vmess
                    .get("path")
                    .and_then(|value| value.as_str())
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned),
                transport_host: vmess
                    .get("host")
                    .and_then(|value| value.as_str())
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned),
                vmess_security: vmess
                    .get("scy")
                    .and_then(|value| value.as_str())
                    .map(ToOwned::to_owned),
                vmess_alter_id: vmess
                    .get("aid")
                    .and_then(|value| {
                        value
                            .as_u64()
                            .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
                    })
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(0),
                ..narya_core::ProtocolOptions::default()
            },
        };

        return Some(Node {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            country_code: "UN".to_string(),
            protocol: "vmess".to_string(),
            tag: None,
            latency: None,
            usage_pct: 0,
            download_speed: 0.0,
            upload_speed: 0.0,
            details,
        });
    }

    if let Some(node) = parse_shadowsocks_uri(uri) {
        return Some(node);
    }

    if let Ok(parsed_url) = Url::parse(uri) {
        let scheme = parsed_url.scheme().to_string();
        if ["vless", "trojan"].contains(&scheme.as_str()) {
            let name = uri_fragment_name(&parsed_url).unwrap_or_else(|| format!("{} Node", scheme));
            let server = parsed_url.host_str().unwrap_or("").to_string();
            let port = parsed_url.port().unwrap_or(443);

            let query_pairs: std::collections::HashMap<_, _> =
                parsed_url.query_pairs().into_owned().collect();
            let tls = query_pairs
                .get("security")
                .map(|v| v == "tls" || v == "reality")
                .unwrap_or(false);
            let transport = query_pairs
                .get("type")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "tcp".to_string());

            let details = NodeDetails {
                address: format!("{}:{}", server, port),
                encryption: if scheme == "trojan" {
                    let username = parsed_url.username();
                    if username.is_empty() {
                        "none".to_string()
                    } else {
                        urlencoding::decode(username)
                            .ok()
                            .map(|password| format!("password:{password}"))
                            .unwrap_or_else(|| "none".to_string())
                    }
                } else {
                    let username = parsed_url.username();
                    if username.is_empty() {
                        "none".to_string()
                    } else {
                        format!("uuid:{username}")
                    }
                },
                udp: true,
                tls,
                skip_cert_verify: false,
                transport,
                last_test: "Never".to_string(),
                options: narya_core::ProtocolOptions {
                    server_name: query_pairs
                        .get("sni")
                        .or_else(|| query_pairs.get("serverName"))
                        .cloned(),
                    transport_path: query_pairs.get("path").cloned(),
                    transport_host: query_pairs.get("host").cloned(),
                    grpc_service_name: query_pairs
                        .get("serviceName")
                        .or_else(|| query_pairs.get("service_name"))
                        .cloned(),
                    flow: query_pairs.get("flow").cloned(),
                    reality_public_key: query_pairs.get("pbk").cloned(),
                    reality_short_id: query_pairs.get("sid").cloned(),
                    ..narya_core::ProtocolOptions::default()
                },
            };

            return Some(Node {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                country_code: "UN".to_string(),
                protocol: scheme,
                tag: None,
                latency: None,
                usage_pct: 0,
                download_speed: 0.0,
                upload_speed: 0.0,
                details,
            });
        }
    }

    None
}

fn uri_fragment_name(parsed_url: &Url) -> Option<String> {
    parsed_url.fragment().map(|s| {
        urlencoding::decode(s)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| s.to_string())
    })
}

fn parse_shadowsocks_uri(uri: &str) -> Option<Node> {
    let parsed_url = Url::parse(uri).ok()?;
    if parsed_url.scheme() != "ss" {
        return None;
    }

    let name = uri_fragment_name(&parsed_url).unwrap_or_else(|| "Shadowsocks Node".to_string());
    let server = parsed_url.host_str()?.to_string();
    let port = parsed_url.port().unwrap_or(8388);
    let user = parsed_url.username();
    let password = parsed_url.password().unwrap_or("");

    let credentials = if !user.is_empty() && !password.is_empty() {
        let method = urlencoding::decode(user).ok()?.into_owned();
        let password = urlencoding::decode(password).ok()?.into_owned();
        format!("{}:{}", method, password)
    } else if !user.is_empty() {
        decode_shadowsocks_userinfo(user)?
    } else {
        return None;
    };

    Some(Node {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        country_code: "UN".to_string(),
        protocol: "ss".to_string(),
        tag: None,
        latency: None,
        usage_pct: 0,
        download_speed: 0.0,
        upload_speed: 0.0,
        details: NodeDetails {
            address: format!("{}:{}", server, port),
            encryption: credentials,
            udp: true,
            tls: false,
            skip_cert_verify: false,
            transport: "tcp".to_string(),
            last_test: "Never".to_string(),
            options: narya_core::ProtocolOptions::default(),
        },
    })
}

fn decode_shadowsocks_userinfo(user: &str) -> Option<String> {
    let decoded_user = urlencoding::decode(user).ok()?.into_owned();
    if decoded_user.contains(':') {
        return Some(decoded_user);
    }
    for engine in [general_purpose::STANDARD, general_purpose::URL_SAFE_NO_PAD] {
        if let Ok(bytes) = engine.decode(decoded_user.as_bytes()) {
            if let Ok(text) = String::from_utf8(bytes) {
                if text.contains(':') {
                    return Some(text);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clash_shadowsocks_password_with_method() {
        let content = r#"
proxies:
  - name: Test SS
    type: ss
    server: example.com
    port: 8388
    cipher: aes-256-gcm
    password: secret
"#;
        let nodes = parse_clash_yaml(content).unwrap();
        assert_eq!(nodes[0].details.encryption, "aes-256-gcm:secret");
    }

    #[test]
    fn parses_plain_shadowsocks_uri_credentials() {
        let node = parse_uri("ss://aes-256-gcm:secret@example.com:8388#Test").unwrap();
        assert_eq!(node.protocol, "ss");
        assert_eq!(node.details.address, "example.com:8388");
        assert_eq!(node.details.encryption, "aes-256-gcm:secret");
    }

    #[test]
    fn parses_base64_shadowsocks_userinfo() {
        let node = parse_uri("ss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#Test").unwrap();
        assert_eq!(node.details.encryption, "aes-256-gcm:secret");
    }

    #[test]
    fn rejects_empty_subscription_content() {
        let err = parse_subscription("   ").unwrap_err();
        assert!(err.to_string().contains("Empty subscription content"));
    }

    #[test]
    fn rejects_zero_node_subscription() {
        let err = parse_subscription("proxies: []").unwrap_err();
        assert!(err.to_string().contains("zero nodes"));
    }

    #[test]
    fn parses_clash_vmess_vless_reality_ws_grpc() {
        let content = r#"
proxies:
  - name: VMess WS
    type: vmess
    server: vmess.example.com
    port: 443
    uuid: 11111111-1111-1111-1111-111111111111
    cipher: auto
    tls: true
    servername: sni.example.com
    network: ws
    ws-opts:
      path: /ws
      headers:
        Host: ws.example.com
  - name: VLESS gRPC Reality
    type: vless
    server: vless.example.com
    port: 443
    uuid: 22222222-2222-2222-2222-222222222222
    tls: true
    servername: reality.example.com
    flow: xtls-rprx-vision
    network: grpc
    grpc-opts:
      grpc-service-name: liora
    reality-opts:
      public-key: pubkey123
      short-id: sid123
"#;
        let nodes = parse_clash_yaml(content).unwrap();
        assert_eq!(nodes.len(), 2);
        let vmess = &nodes[0].details.options;
        assert_eq!(vmess.server_name.as_deref(), Some("sni.example.com"));
        assert_eq!(vmess.transport_path.as_deref(), Some("/ws"));
        assert_eq!(vmess.transport_host.as_deref(), Some("ws.example.com"));
        assert_eq!(vmess.vmess_security.as_deref(), Some("auto"));
        let vless = &nodes[1].details.options;
        assert_eq!(vless.server_name.as_deref(), Some("reality.example.com"));
        assert_eq!(vless.grpc_service_name.as_deref(), Some("liora"));
        assert_eq!(vless.flow.as_deref(), Some("xtls-rprx-vision"));
        assert_eq!(vless.reality_public_key.as_deref(), Some("pubkey123"));
        assert_eq!(vless.reality_short_id.as_deref(), Some("sid123"));
    }

    #[test]
    fn parses_singbox_tls_reality_transport() {
        let content = r#"
{
  "outbounds": [
    {
      "type": "vmess",
      "tag": "sb-vmess",
      "server": "sb.example.com",
      "server_port": 443,
      "uuid": "33333333-3333-3333-3333-333333333333",
      "security": "auto",
      "tls": {
        "enabled": true,
        "server_name": "sb-sni.example.com",
        "alpn": ["h2", "http/1.1"],
        "insecure": true,
        "reality": {
          "public_key": "sb-pub",
          "short_id": "sb-sid"
        }
      },
      "transport": {
        "type": "grpc",
        "service_name": "sb-grpc"
      },
      "flow": "xtls-rprx-vision"
    }
  ]
}
"#;
        let nodes = parse_singbox_json(content).unwrap();
        assert_eq!(nodes.len(), 1);
        let opts = &nodes[0].details.options;
        assert_eq!(opts.server_name.as_deref(), Some("sb-sni.example.com"));
        assert_eq!(opts.alpn, vec!["h2".to_string(), "http/1.1".to_string()]);
        assert_eq!(opts.grpc_service_name.as_deref(), Some("sb-grpc"));
        assert_eq!(opts.flow.as_deref(), Some("xtls-rprx-vision"));
        assert_eq!(opts.reality_public_key.as_deref(), Some("sb-pub"));
        assert_eq!(opts.reality_short_id.as_deref(), Some("sb-sid"));
        assert!(nodes[0].details.tls);
        assert!(nodes[0].details.skip_cert_verify);
    }

    #[test]
    fn parses_vless_and_trojan_uris() {
        let vless = parse_uri(
            "vless://11111111-1111-1111-1111-111111111111@vless.example.com:443?type=grpc&serviceName=liora&sni=sni.example.com&flow=xtls-rprx-vision&pbk=pubkey&sid=sid#VLESS",
        )
        .unwrap();
        assert_eq!(vless.protocol, "vless");
        assert_eq!(vless.details.address, "vless.example.com:443");
        assert_eq!(
            vless.details.encryption,
            "uuid:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(
            vless.details.options.grpc_service_name.as_deref(),
            Some("liora")
        );
        assert_eq!(
            vless.details.options.reality_public_key.as_deref(),
            Some("pubkey")
        );
        assert_eq!(
            vless.details.options.reality_short_id.as_deref(),
            Some("sid")
        );

        let trojan = parse_uri(
            "trojan://secret@trojan.example.com:443?type=ws&path=%2Fws&host=ws.example.com&sni=sni.example.com#Trojan",
        )
        .unwrap();
        assert_eq!(trojan.protocol, "trojan");
        assert_eq!(trojan.details.encryption, "password:secret");
        assert_eq!(
            trojan.details.options.server_name.as_deref(),
            Some("sni.example.com")
        );
        assert_eq!(
            trojan.details.options.transport_path.as_deref(),
            Some("/ws")
        );
        assert_eq!(
            trojan.details.options.transport_host.as_deref(),
            Some("ws.example.com")
        );
    }
}
