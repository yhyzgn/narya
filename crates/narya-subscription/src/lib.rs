use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use narya_core::{Node, NodeDetails};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
struct ClashConfig {
    proxies: Option<Vec<serde_yaml::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SingBoxConfig {
    outbounds: Option<Vec<serde_json::Value>>,
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
    let client = reqwest::Client::builder()
        .user_agent("Clash/1.0 Narya/1.0")
        .timeout(Duration::from_secs(15))
        .build()?;

    let response = client.get(url).send().await?;
    let content = response.text().await?;
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
    let format = detect_format(content);
    let nodes = match format {
        SubscriptionFormat::ClashYaml => parse_clash_yaml(content)?,
        SubscriptionFormat::SingBoxJson => parse_singbox_json(content)?,
        SubscriptionFormat::V2RayBase64 => parse_v2ray_base64(content)?,
        SubscriptionFormat::Unknown => return Err(anyhow!("Unsupported subscription format")),
    };
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

            let encryption = if proxy_type.eq_ignore_ascii_case("ss")
                || proxy_type.eq_ignore_ascii_case("shadowsocks")
            {
                if !cipher.is_empty() && !password.is_empty() {
                    format!("{}:{}", cipher, password)
                } else {
                    "none".to_string()
                }
            } else if !cipher.is_empty() {
                cipher
            } else if !uuid.is_empty() {
                "auto (uuid)".to_string()
            } else {
                "none".to_string()
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

            let details = NodeDetails {
                address: format!("{}:{}", server, port),
                encryption: "auto".to_string(),
                udp: true, // Singbox usually enables UDP inherently depending on protocol
                tls,
                skip_cert_verify: tls_obj
                    .and_then(|t| t.get("insecure").and_then(|v| v.as_bool()))
                    .unwrap_or(false),
                transport,
                last_test: "Never".to_string(),
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
            encryption: "auto".to_string(),
            udp: true,
            tls,
            skip_cert_verify: false,
            transport,
            last_test: "Never".to_string(),
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
                encryption: "auto".to_string(),
                udp: true,
                tls,
                skip_cert_verify: false,
                transport,
                last_test: "Never".to_string(),
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
}
