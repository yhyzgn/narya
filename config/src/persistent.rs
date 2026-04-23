use api::tracker::BypassRules;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub subscription_url: String,
    pub active_node: Option<String>,
    pub bypass_rules: BypassRules,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            subscription_url:
                "https://jsjc.cfd/api/v1/client/subscribe?token=a6db043ed2bd5771205036c514290aa0"
                    .to_string(),
            active_node: None,
            bypass_rules: BypassRules {
                whitelist: vec![],
                blacklist: vec![],
            },
        }
    }
}
