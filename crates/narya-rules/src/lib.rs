use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub priority: i32,
    pub conditions: Vec<Condition>,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Condition {
    Domain(String),
    DomainSuffix(String),
    IpCidr { network: IpAddr, prefix: u8 },
    Port(u16),
    Process(String),
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "outbound", rename_all = "snake_case")]
pub enum Action {
    Proxy(String),
    Direct,
    Block,
    Dns(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestContext {
    pub domain: Option<String>,
    pub ip: Option<IpAddr>,
    pub port: Option<u16>,
    pub process: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub rule_id: String,
    pub action: Action,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSet {
    rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleError {
    EmptyRuleId,
    EmptyCondition {
        rule_id: String,
    },
    EmptyValue {
        rule_id: String,
    },
    InvalidCidr {
        rule_id: String,
        reason: String,
    },
    NoMatch {
        domain: Option<String>,
        ip: Option<IpAddr>,
    },
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRuleId => write!(f, "rule id must not be empty"),
            Self::EmptyCondition { rule_id } => {
                write!(f, "rule {rule_id} must contain at least one condition")
            }
            Self::EmptyValue { rule_id } => write!(f, "rule {rule_id} contains an empty value"),
            Self::InvalidCidr { rule_id, reason } => {
                write!(f, "rule {rule_id} has invalid CIDR: {reason}")
            }
            Self::NoMatch { domain, ip } => {
                write!(f, "no rule matched domain={domain:?} ip={ip:?}")
            }
        }
    }
}

impl std::error::Error for RuleError {}

impl RuleSet {
    pub fn compile(mut rules: Vec<Rule>) -> Result<Self, RuleError> {
        for rule in &rules {
            validate_rule(rule)?;
        }
        rules.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(Self { rules })
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn decide(&self, request: &RequestContext) -> Result<Decision, RuleError> {
        for rule in &self.rules {
            if rule
                .conditions
                .iter()
                .all(|condition| condition_matches(condition, request))
            {
                return Ok(Decision {
                    rule_id: rule.id.clone(),
                    action: rule.action.clone(),
                    explanation: format!("matched rule {} at priority {}", rule.id, rule.priority),
                });
            }
        }
        Err(RuleError::NoMatch {
            domain: request.domain.clone(),
            ip: request.ip,
        })
    }
}

fn validate_rule(rule: &Rule) -> Result<(), RuleError> {
    if rule.id.trim().is_empty() {
        return Err(RuleError::EmptyRuleId);
    }
    if rule.conditions.is_empty() {
        return Err(RuleError::EmptyCondition {
            rule_id: rule.id.clone(),
        });
    }
    for condition in &rule.conditions {
        match condition {
            Condition::Domain(value)
            | Condition::DomainSuffix(value)
            | Condition::Process(value) => {
                if value.trim().is_empty() {
                    return Err(RuleError::EmptyValue {
                        rule_id: rule.id.clone(),
                    });
                }
            }
            Condition::IpCidr { network, prefix } => {
                let max_prefix = match network {
                    IpAddr::V4(_) => 32,
                    IpAddr::V6(_) => 128,
                };
                if *prefix > max_prefix {
                    return Err(RuleError::InvalidCidr {
                        rule_id: rule.id.clone(),
                        reason: format!("prefix {prefix} exceeds {max_prefix}"),
                    });
                }
            }
            Condition::Port(_) | Condition::Any => {}
        }
    }
    Ok(())
}

fn condition_matches(condition: &Condition, request: &RequestContext) -> bool {
    match condition {
        Condition::Any => true,
        Condition::Domain(expected) => request
            .domain
            .as_deref()
            .is_some_and(|actual| actual.eq_ignore_ascii_case(expected)),
        Condition::DomainSuffix(expected) => request.domain.as_deref().is_some_and(|actual| {
            let actual = actual.trim_end_matches('.').to_ascii_lowercase();
            let expected = expected
                .trim_start_matches('.')
                .trim_end_matches('.')
                .to_ascii_lowercase();
            actual == expected || actual.ends_with(&format!(".{expected}"))
        }),
        Condition::IpCidr { network, prefix } => request
            .ip
            .is_some_and(|actual| ip_in_cidr(actual, *network, *prefix)),
        Condition::Port(expected) => request.port == Some(*expected),
        Condition::Process(expected) => request
            .process
            .as_deref()
            .is_some_and(|actual| actual.eq_ignore_ascii_case(expected)),
    }
}

fn ip_in_cidr(ip: IpAddr, network: IpAddr, prefix: u8) -> bool {
    match (ip, network) {
        (IpAddr::V4(ip), IpAddr::V4(network)) => masked_equal(
            u32::from(ip) as u128,
            u32::from(network) as u128,
            prefix,
            32,
        ),
        (IpAddr::V6(ip), IpAddr::V6(network)) => {
            masked_equal(u128::from(ip), u128::from(network), prefix, 128)
        }
        _ => false,
    }
}

fn masked_equal(left: u128, right: u128, prefix: u8, bits: u8) -> bool {
    if prefix == 0 {
        return true;
    }
    let shift = u32::from(bits - prefix);
    (left >> shift) == (right >> shift)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn rule(id: &str, priority: i32, condition: Condition, action: Action) -> Rule {
        Rule {
            id: id.to_string(),
            priority,
            conditions: vec![condition],
            action,
        }
    }

    #[test]
    fn sorts_by_priority_then_id_and_returns_explanation() {
        let rules = RuleSet::compile(vec![
            rule("later", 20, Condition::Any, Action::Direct),
            rule(
                "first",
                10,
                Condition::DomainSuffix("example.com".into()),
                Action::Proxy("proxy".into()),
            ),
        ])
        .unwrap();
        let decision = rules
            .decide(&RequestContext {
                domain: Some("api.example.com".into()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(decision.rule_id, "first");
        assert!(decision.explanation.contains("priority 10"));
    }

    #[test]
    fn domain_suffix_respects_label_boundary() {
        let rules = RuleSet::compile(vec![rule(
            "suffix",
            1,
            Condition::DomainSuffix("example.com".into()),
            Action::Direct,
        )])
        .unwrap();
        assert!(rules
            .decide(&RequestContext {
                domain: Some("www.example.com".into()),
                ..Default::default()
            })
            .is_ok());
        assert!(matches!(
            rules.decide(&RequestContext {
                domain: Some("badexample.com".into()),
                ..Default::default()
            }),
            Err(RuleError::NoMatch { .. })
        ));
    }

    #[test]
    fn matches_ipv4_cidr_and_rejects_ipv6() {
        let rules = RuleSet::compile(vec![rule(
            "private",
            1,
            Condition::IpCidr {
                network: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)),
                prefix: 8,
            },
            Action::Block,
        )])
        .unwrap();
        assert!(rules
            .decide(&RequestContext {
                ip: Some(IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3))),
                ..Default::default()
            })
            .is_ok());
        assert!(matches!(
            rules.decide(&RequestContext {
                ip: Some("2001:db8::1".parse().unwrap()),
                ..Default::default()
            }),
            Err(RuleError::NoMatch { .. })
        ));
    }

    #[test]
    fn no_match_is_fail_closed() {
        let rules = RuleSet::compile(vec![rule(
            "port",
            1,
            Condition::Port(443),
            Action::Proxy("secure".into()),
        )])
        .unwrap();
        let error = rules.decide(&RequestContext::default()).unwrap_err();
        assert!(matches!(error, RuleError::NoMatch { .. }));
    }

    #[test]
    fn multiple_conditions_are_all_required() {
        let rules = RuleSet::compile(vec![Rule {
            id: "secure-web".into(),
            priority: 1,
            conditions: vec![
                Condition::DomainSuffix("example.com".into()),
                Condition::Port(443),
            ],
            action: Action::Proxy("secure".into()),
        }])
        .unwrap();
        assert!(rules
            .decide(&RequestContext {
                domain: Some("api.example.com".into()),
                port: Some(443),
                ..Default::default()
            })
            .is_ok());
        assert!(matches!(
            rules.decide(&RequestContext {
                domain: Some("api.example.com".into()),
                port: Some(80),
                ..Default::default()
            }),
            Err(RuleError::NoMatch { .. })
        ));
    }

    #[test]
    fn invalid_rule_is_rejected_before_runtime() {
        let error = RuleSet::compile(vec![rule(
            "broken",
            1,
            Condition::IpCidr {
                network: IpAddr::V4(Ipv4Addr::LOCALHOST),
                prefix: 33,
            },
            Action::Direct,
        )])
        .unwrap_err();
        assert!(matches!(error, RuleError::InvalidCidr { .. }));
    }
}
