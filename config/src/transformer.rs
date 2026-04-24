use crate::model::{Action, GroupType, NaryaConfig, RuleType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub struct Transformer;

#[derive(Serialize, Deserialize)]
struct SingBoxConfig {
    log: LogOptions,
    inbounds: Vec<Value>,
    outbounds: Vec<Value>,
    route: RouteOptions,
}

#[derive(Serialize, Deserialize)]
struct LogOptions {
    level: String,
}

#[derive(Serialize, Deserialize)]
struct RouteOptions {
    rules: Vec<Value>,
}

impl Transformer {
    pub fn transform(config: &NaryaConfig) -> String {
        let mut inbounds = Vec::new();
        let mut outbounds = Vec::new();
        let mut rules = Vec::new();

        // 1. Inbound: Mixed (HTTP/SOCKS)
        inbounds.push(json!({
            "type": "mixed",
            "listen": "127.0.0.1",
            "listen_port": config.settings.mixed_port
        }));

        // 2. Outbound: Direct (Default)
        outbounds.push(json!({
            "type": "direct",
            "tag": "direct"
        }));

        // 3. Outbounds: Proxies
        for proxy in &config.proxies {
            let outbound = match proxy.proxy_type.to_lowercase().as_str() {
                "ss" | "shadowsocks" => json!({
                    "type": "shadowsocks",
                    "tag": proxy.name,
                    "server": proxy.server,
                    "server_port": proxy.port,
                    "method": "aes-128-gcm", // 临时硬编码，后续需从解析器获取
                    "password": "password"   // 临时硬编码
                }),
                "trojan" => json!({
                    "type": "trojan",
                    "tag": proxy.name,
                    "server": proxy.server,
                    "server_port": proxy.port,
                    "password": "password"
                }),
                _ => json!({
                    "type": "direct",
                    "tag": proxy.name
                }),
            };
            outbounds.push(outbound);
        }

        // 4. Outbounds: Groups (Selector)
        for group in &config.groups {
            if group.group_type == GroupType::Select {
                outbounds.push(json!({
                    "type": "selector",
                    "tag": group.name,
                    "outbounds": group.proxies,
                    "default": group.proxies.first().cloned().unwrap_or_else(|| "direct".to_string())
                }));
            }
        }

        // 5. Rules
        for rule in &config.rules {
            let action_tag = match &rule.action {
                Action::Direct => "direct",
                Action::Proxy(tag) => tag,
                Action::Reject => "block", // Sing-box block outbound
            };

            if action_tag == "block" && !outbounds.iter().any(|o| o["tag"] == "block") {
                outbounds.push(json!({
                    "type": "block",
                    "tag": "block"
                }));
            }

            let mut rule_json = json!({
                "outbound": action_tag
            });

            match rule.rule_type {
                RuleType::Domain => {
                    rule_json["domain"] = json!(rule.payload);
                }
                RuleType::DomainSuffix => {
                    rule_json["domain_suffix"] = json!(rule.payload);
                }
                RuleType::IpCidr => {
                    rule_json["ip_cidr"] = json!(rule.payload);
                }
                _ => {}
            }
            rules.push(rule_json);
        }

        let sb_config = SingBoxConfig {
            log: LogOptions {
                level: "info".to_string(),
            },
            inbounds,
            outbounds,
            route: RouteOptions { rules },
        };

        serde_json::to_string_pretty(&sb_config).unwrap_or_default()
    }
}
