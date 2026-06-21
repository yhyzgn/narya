use anyhow::{anyhow, Result};
use narya_core::Node;
use serde_json::{json, Value};

pub struct ConfigGenerator;

impl ConfigGenerator {
    pub fn generate_json(node: &Node) -> Result<Value> {
        let (server, port) = split_host_port(&node.details.address)?;
        let outbound = match node.protocol.to_ascii_lowercase().as_str() {
            "shadowsocks" | "ss" => {
                let (method, password) = split_shadowsocks_credentials(&node.details.encryption)?;
                json!({
                    "type": "shadowsocks",
                    "tag": "proxy",
                    "server": server,
                    "server_port": port,
                    "method": method,
                    "password": password
                })
            }
            protocol => {
                return Err(anyhow!(
                    "unsupported proxy protocol for sing-box config generation: {}",
                    protocol
                ));
            }
        };

        Ok(json!({
            "log": {
                "level": "info",
                "timestamp": true
            },
            "inbounds": [
                {
                    "type": "socks",
                    "tag": "socks-in",
                    "listen": "127.0.0.1",
                    "listen_port": 1080
                },
                {
                    "type": "http",
                    "tag": "http-in",
                    "listen": "127.0.0.1",
                    "listen_port": 2080
                }
            ],
            "outbounds": [
                outbound,
                {
                    "type": "direct",
                    "tag": "direct"
                },
                {
                    "type": "dns",
                    "tag": "dns-out"
                }
            ],
            "route": {
                "rules": [
                    {
                        "protocol": "dns",
                        "outbound": "dns-out"
                    }
                ]
            }
        }))
    }
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

    #[test]
    fn shadowsocks_uses_distinct_method_and_password() {
        let config = ConfigGenerator::generate_json(&node("ss", "aes-256-gcm:secret")).unwrap();
        let outbound = &config["outbounds"][0];
        assert_eq!(outbound["method"], "aes-256-gcm");
        assert_eq!(outbound["password"], "secret");
    }

    #[test]
    fn shadowsocks_missing_password_fails_closed() {
        let err = ConfigGenerator::generate_json(&node("ss", "aes-256-gcm")).unwrap_err();
        assert!(err.to_string().contains("method:password"));
    }

    #[test]
    fn unsupported_protocol_fails_closed() {
        let err = ConfigGenerator::generate_json(&node("vmess", "auto")).unwrap_err();
        assert!(err.to_string().contains("unsupported proxy protocol"));
    }
}
