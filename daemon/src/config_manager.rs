use anyhow::{Context, Result};
use config::persistent::PersistentState;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct ConfigManager {
    config_path: PathBuf,
    state: Arc<Mutex<PersistentState>>,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let mut path = dirs::home_dir().context("Cannot find home directory")?;
        path.push(".config/narya");
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        path.push("state.json");

        let state = if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            PersistentState::default()
        };

        Ok(Self {
            config_path: path,
            state: Arc::new(Mutex::new(state)),
        })
    }

    pub fn get_state(&self) -> PersistentState {
        self.state.lock().unwrap().clone()
    }

    pub fn update_rules(&self, rules: api::tracker::BypassRules) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.bypass_rules = rules;
        self.save(&state)
    }

    pub fn update_active_node(&self, node: String) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.active_node = Some(node);
        self.save(&state)
    }

    fn save(&self, state: &PersistentState) -> Result<()> {
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
}
