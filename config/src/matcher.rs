use crate::model::{Action, Rule, RuleType};
use anyhow::Result;
use api::tracker::ConnectionMeta;

pub trait Matcher: Send + Sync {
    fn matches(&self, _meta: &ConnectionMeta) -> bool;
}

pub struct RuleEngine {
    rules: Vec<(Rule, Box<dyn Matcher>)>,
    default_action: Action,
}

impl RuleEngine {
    pub fn new(config_rules: Vec<Rule>, default_action: Action) -> Self {
        let mut rules = Vec::new();
        for r in config_rules {
            if let Ok(matcher) = create_matcher(&r) {
                rules.push((r, matcher));
            }
        }
        Self {
            rules,
            default_action,
        }
    }

    pub fn match_rule(&self, meta: &ConnectionMeta) -> &Action {
        for (rule, matcher) in &self.rules {
            if matcher.matches(meta) {
                tracing::debug!("Rule matched: {} -> {:?}", rule.name, rule.action);
                return &rule.action;
            }
        }
        &self.default_action
    }
}

fn create_matcher(rule: &Rule) -> Result<Box<dyn Matcher>> {
    match rule.rule_type {
        RuleType::DomainSuffix => Ok(Box::new(DomainSuffixMatcher::new(rule.payload.clone()))),
        RuleType::ProcessName => Ok(Box::new(ProcessNameMatcher::new(rule.payload.clone()))),
        // 其他类型待实现...
        _ => anyhow::bail!("Unsupported rule type"),
    }
}

// 示例实现：域名后缀匹配
struct DomainSuffixMatcher {
    suffixes: Vec<String>,
}

impl DomainSuffixMatcher {
    fn new(suffixes: Vec<String>) -> Self {
        Self { suffixes }
    }
}

impl Matcher for DomainSuffixMatcher {
    fn matches(&self, meta: &ConnectionMeta) -> bool {
        // 在实际应用中，meta 需要携带域名信息（通过 FakeIP 还原或 DNS 解析）
        // 这里暂时通过 ConnectionMeta 的扩展字段或 DNS 缓存获取
        false
    }
}

// 示例实现：进程名匹配
struct ProcessNameMatcher {
    names: Vec<String>,
}

impl ProcessNameMatcher {
    fn new(names: Vec<String>) -> Self {
        Self { names }
    }
}

impl Matcher for ProcessNameMatcher {
    fn matches(&self, meta: &ConnectionMeta) -> bool {
        if let Some(ref name) = meta.process_name {
            return self.names.iter().any(|n| name.contains(n));
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Action, Rule, RuleType};
    use api::tracker::{ConnectionMeta, Protocol};
    use std::net::IpAddr;

    #[test]
    fn test_process_name_matching() {
        let rules = vec![Rule {
            name: "Proxy Telegram".to_string(),
            rule_type: RuleType::ProcessName,
            payload: vec!["telegram".to_string(), "Telegram".to_string()],
            action: Action::Proxy("Overseas".to_string()),
        }];
        let engine = RuleEngine::new(rules, Action::Direct);

        let mut meta = ConnectionMeta {
            protocol: Protocol::Tcp,
            src_ip: "127.0.0.1".parse().unwrap(),
            src_port: 12345,
            dst_ip: "1.1.1.1".parse().unwrap(),
            dst_port: 443,
            pid: Some(1234),
            process_name: Some("telegram-desktop".to_string()),
            process_path: None,
            package_name: None,
            bundle_id: None,
        };

        // Match case
        assert_eq!(
            engine.match_rule(&meta),
            &Action::Proxy("Overseas".to_string())
        );

        // No match case
        meta.process_name = Some("browser".to_string());
        assert_eq!(engine.match_rule(&meta), &Action::Direct);
    }
}
