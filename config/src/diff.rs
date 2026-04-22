use crate::model::NaryaConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ConfigDiff {
    pub subscriptions_changed: bool,
    pub groups_changed: Vec<String>, // Names of groups that changed
    pub rules_changed: bool,
    pub settings_changed: bool,
}

impl ConfigDiff {
    pub fn calculate(old: &NaryaConfig, new: &NaryaConfig) -> Self {
        let mut diff = ConfigDiff::default();

        if old.subscriptions != new.subscriptions {
            diff.subscriptions_changed = true;
        }

        // Compare groups
        for new_group in &new.groups {
            if let Some(old_group) = old.groups.iter().find(|g| g.name == new_group.name) {
                if old_group != new_group {
                    diff.groups_changed.push(new_group.name.clone());
                }
            } else {
                // New group added
                diff.groups_changed.push(new_group.name.clone());
            }
        }
        // Check for deleted groups
        for old_group in &old.groups {
            if !new.groups.iter().any(|g| g.name == old_group.name) {
                diff.groups_changed.push(old_group.name.clone());
            }
        }

        if old.rules != new.rules {
            diff.rules_changed = true;
        }

        if old.settings != new.settings {
            diff.settings_changed = true;
        }

        diff
    }

    pub fn has_changes(&self) -> bool {
        self.subscriptions_changed
            || !self.groups_changed.is_empty()
            || self.rules_changed
            || self.settings_changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    #[test]
    fn test_config_diff() {
        let old_config = NaryaConfig::default();
        let mut new_config = old_config.clone();

        assert!(!ConfigDiff::calculate(&old_config, &new_config).has_changes());

        new_config.settings.mixed_port = 8888;
        let diff = ConfigDiff::calculate(&old_config, &new_config);
        assert!(diff.settings_changed);
        assert!(diff.has_changes());

        let mut group_config = new_config.clone();
        group_config.groups.push(ProxyGroup {
            name: "TestGroup".to_string(),
            group_type: GroupType::Select,
            proxies: vec!["Proxy1".to_string()],
        });
        let diff = ConfigDiff::calculate(&new_config, &group_config);
        assert_eq!(diff.groups_changed, vec!["TestGroup".to_string()]);
    }
}
