use crate::model::{GroupType, NaryaConfig, ProxyGroup};
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
            // Try base64 decoding first
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

        // Extract groups as a simple example
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
        let mut nodes = Vec::new();

        // Simple line-by-line link parser (vmess://, ss://, etc.)
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
                // 尝试提取节点名称：通常在 # 后面
                let name = if let Some(hash_pos) = line.find('#') {
                    let raw_name = &line[hash_pos + 1..];
                    // 处理 URL 编码
                    percent_encoding::percent_decode_str(raw_name)
                        .decode_utf8_lossy()
                        .into_owned()
                } else {
                    format!(
                        "{}-{}",
                        line.split("://").next().unwrap_or("node"),
                        nodes.len() + 1
                    )
                };

                nodes.push(name);
            }
        }

        if !nodes.is_empty() {
            // 将所有识别出的节点放入一个名为 "Auto-Parsed" 的组中
            config.groups.push(ProxyGroup {
                name: "Auto-Parsed".to_string(),
                group_type: GroupType::Select,
                proxies: nodes,
            });
        }

        Ok(config)
    }
}
