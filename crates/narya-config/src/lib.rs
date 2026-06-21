use anyhow::Result;
use narya_core::Node;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub nodes: Vec<Node>,
}

pub fn save_profile(path: PathBuf, profile: &Profile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(profile)?;
    fs::write(path, yaml)?;
    Ok(())
}

pub fn load_profile(path: PathBuf) -> Result<Profile> {
    let yaml = fs::read_to_string(path)?;
    let profile: Profile = serde_yaml::from_str(&yaml)?;
    Ok(profile)
}
