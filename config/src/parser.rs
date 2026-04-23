use crate::model::{GroupType, NaryaConfig, Proxy, ProxyGroup};
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};

pub struct SubscriptionParser;

impl SubscriptionParser {
    pub async fn fetch_and_parse(url: &str) -> Result<NaryaConfig> {
        let client = reqwest::Client::new();
        let content = client.get(url).send().await?.text().await?;
        Self::parse(&content).await
    }

    pub async fn parse(content: &str) -> Result<NaryaConfig> {
        let trimmed = content.trim();
        if trimmed.starts_with("proxies:") || trimmed.contains("proxy-groups:") {
            Self::parse_clash(trimmed)
        } else {
            if let Ok(decoded_bytes) =
                general_purpose::STANDARD.decode(trimmed.replace("\n", "").replace("\r", ""))
            {
                let decoded_str = String::from_utf8(decoded_bytes)?;
                Self::parse_plain_links(&decoded_str)
            } else {
                Self::parse_plain_links(trimmed)
            }
        }
    }

    fn parse_clash(content: &str) -> Result<NaryaConfig> {
        tracing::info!("Parsing Clash format...");
        let mut narya_config = NaryaConfig::default();
        let yaml: serde_yaml::Value = serde_yaml::from_str(content)?;

        // 解析 proxies
        if let Some(proxies) = yaml.get("proxies").and_then(|v| v.as_sequence()) {
            for p in proxies {
                if let (Some(name), Some(p_type), Some(server), Some(port)) = (
                    p.get("name").and_then(|v| v.as_str()),
                    p.get("type").and_then(|v| v.as_str()),
                    p.get("server").and_then(|v| v.as_str()),
                    p.get("port").and_then(|v| v.as_u64()),
                ) {
                    narya_config.proxies.push(Proxy {
                        name: name.to_string(),
                        proxy_type: p_type.to_string(),
                        server: server.to_string(),
                        port: port as u16,
                    });
                }
            }
        }

        // 解析 groups
        if let Some(groups) = yaml.get("proxy-groups").and_then(|v| v.as_sequence()) {
            for g in groups {
                if let (Some(name), Some(g_type)) = (
                    g.get("name").and_then(|v| v.as_str()),
                    g.get("type").and_then(|v| v.as_str()),
                ) {
                    narya_config.groups.push(ProxyGroup {
                        name: name.to_string(),
                        group_type: match g_type.to_lowercase().as_str() {
                            "select" => GroupType::Select,
                            "url-test" => GroupType::UrlTest,
                            "fallback" => GroupType::Fallback,
                            "load-balance" => GroupType::LoadBalance,
                            "relay" => GroupType::Relay,
                            _ => GroupType::Select,
                        },
                        proxies: g
                            .get("proxies")
                            .and_then(|v| v.as_sequence())
                            .map(|s| {
                                s.iter()
                                    .filter_map(|p| p.as_str().map(|ps| ps.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    });
                }
            }
        }
        Ok(narya_config)
    }

    fn parse_plain_links(content: &str) -> Result<NaryaConfig> {
        tracing::info!("Parsing plain links format...");
        let mut config = NaryaConfig::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("ss://")
                || line.starts_with("vmess://")
                || line.starts_with("trojan://")
                || line.starts_with("vless://")
            {
                let name = if let Some(hash_pos) = line.find('#') {
                    let raw_name = &line[hash_pos + 1..];
                    percent_encoding::percent_decode_str(raw_name)
                        .decode_utf8_lossy()
                        .into_owned()
                } else {
                    format!("{}-node", line.split("://").next().unwrap_or("node"))
                };

                // 简单的正则或者 URL 解析来提取 server 和 port
                // 这里我们先做一个极其简单的 Mock 解析，实际中应使用更严谨的 URL parser
                let parts: Vec<&str> = line.split('@').collect();
                let server_info = if parts.len() > 1 {
                    parts[1].split('#').next().unwrap_or("")
                } else {
                    line.split("://")
                        .nth(1)
                        .unwrap_or("")
                        .split('#')
                        .next()
                        .unwrap_or("")
                };

                let server_parts: Vec<&str> = server_info.split(':').collect();
                let (server, port) = if server_parts.len() >= 2 {
                    (
                        server_parts[0].to_string(),
                        server_parts[1].parse::<u16>().unwrap_or(443),
                    )
                } else {
                    ("unknown".to_string(), 443)
                };

                config.proxies.push(Proxy {
                    name: name.clone(),
                    proxy_type: line.split("://").next().unwrap_or("unknown").to_string(),
                    server,
                    port,
                });
            }
        }

        if !config.proxies.is_empty() {
            config.groups.push(ProxyGroup {
                name: "Auto-Parsed".to_string(),
                group_type: GroupType::Select,
                proxies: config.proxies.iter().map(|p| p.name.clone()).collect(),
            });
        }

        Ok(config)
    }
}
