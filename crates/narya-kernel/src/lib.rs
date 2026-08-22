use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KernelId {
    SingBox,
    Mihomo,
    Xray,
}

impl KernelId {
    pub const ALL: [Self; 3] = [Self::SingBox, Self::Mihomo, Self::Xray];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SingBox => "sing-box",
            Self::Mihomo => "mihomo",
            Self::Xray => "xray-core",
        }
    }

    pub fn binary_candidates(self) -> &'static [&'static str] {
        match self {
            Self::SingBox => &["sing-box"],
            Self::Mihomo => &["mihomo", "clash-meta"],
            Self::Xray => &["xray", "xray-core"],
        }
    }

    pub fn version_args(self) -> &'static [&'static str] {
        match self {
            Self::SingBox => &["version"],
            Self::Mihomo => &["-v"],
            Self::Xray => &["version"],
        }
    }

    pub fn config_args(self, config: &Path) -> Vec<String> {
        let config = config.to_string_lossy().into_owned();
        match self {
            Self::SingBox | Self::Xray => vec!["run".into(), "-c".into(), config],
            Self::Mihomo => vec!["-f".into(), config],
        }
    }
}

impl fmt::Display for KernelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for KernelId {
    type Err = KernelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "sing-box" | "singbox" => Ok(Self::SingBox),
            "mihomo" | "clash-meta" => Ok(Self::Mihomo),
            "xray" | "xray-core" => Ok(Self::Xray),
            other => Err(KernelError::Unknown(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelState {
    NotInstalled,
    Installed,
    Installing,
    Upgrading,
    Starting,
    Running,
    Stopping,
    Failed,
}

impl KernelState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotInstalled => "not_installed",
            Self::Installed => "installed",
            Self::Installing => "installing",
            Self::Upgrading => "upgrading",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelRecord {
    pub id: KernelId,
    pub binary_path: Option<PathBuf>,
    pub version: Option<String>,
    pub state: KernelState,
    pub healthy: bool,
    pub failure: Option<String>,
}

impl KernelRecord {
    fn unavailable(id: KernelId) -> Self {
        Self {
            id,
            binary_path: None,
            version: None,
            state: KernelState::NotInstalled,
            healthy: false,
            failure: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    Unknown(String),
    NotInstalled(KernelId),
    InvalidState(String),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(name) => write!(f, "unknown kernel: {name}"),
            Self::NotInstalled(id) => write!(f, "kernel {id} is not installed"),
            Self::InvalidState(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for KernelError {}

#[derive(Debug, Clone)]
pub struct KernelRegistry {
    records: BTreeMap<KernelId, KernelRecord>,
}

impl Default for KernelRegistry {
    fn default() -> Self {
        let records = KernelId::ALL
            .into_iter()
            .map(|id| (id, KernelRecord::unavailable(id)))
            .collect();
        Self { records }
    }
}

impl KernelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn probe() -> Self {
        let mut registry = Self::new();
        registry.refresh();
        registry
    }

    pub fn refresh(&mut self) {
        for id in KernelId::ALL {
            let record = self.records.get_mut(&id).expect("all kernels registered");
            let found = id
                .binary_candidates()
                .iter()
                .find_map(|candidate| find_on_path(candidate));
            match found {
                Some(path) => {
                    record.binary_path = Some(path.clone());
                    record.version = Command::new(&path)
                        .args(id.version_args())
                        .output()
                        .ok()
                        .filter(|output| output.status.success())
                        .and_then(|output| {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            stdout
                                .lines()
                                .chain(stderr.lines())
                                .map(str::trim)
                                .find(|line| !line.is_empty())
                                .map(ToOwned::to_owned)
                        });
                    record.state = KernelState::Installed;
                    record.healthy = false;
                    record.failure = None;
                }
                None => {
                    record.binary_path = None;
                    record.version = None;
                    record.state = KernelState::NotInstalled;
                    record.healthy = false;
                }
            }
        }
    }

    pub fn records(&self) -> impl Iterator<Item = &KernelRecord> {
        self.records.values()
    }

    pub fn record(&self, id: KernelId) -> &KernelRecord {
        self.records.get(&id).expect("all kernels registered")
    }

    pub fn record_mut(&mut self, id: KernelId) -> &mut KernelRecord {
        self.records.get_mut(&id).expect("all kernels registered")
    }

    pub fn require_installed(&self, id: KernelId) -> Result<&KernelRecord, KernelError> {
        let record = self.record(id);
        if record.binary_path.is_none() {
            return Err(KernelError::NotInstalled(id));
        }
        Ok(record)
    }

    pub fn set_installed(&mut self, id: KernelId, path: PathBuf, version: String) {
        let record = self.record_mut(id);
        record.binary_path = Some(path);
        record.version = Some(version);
        record.state = KernelState::Installed;
        record.healthy = false;
        record.failure = None;
    }

    pub fn set_state(
        &mut self,
        id: KernelId,
        state: KernelState,
        healthy: bool,
        failure: Option<String>,
    ) {
        let record = self.record_mut(id);
        record.state = state;
        record.healthy = healthy;
        record.failure = failure;
    }
}

fn find_on_path(candidate: &str) -> Option<PathBuf> {
    let candidate_path = Path::new(candidate);
    if candidate_path.components().count() > 1 && candidate_path.is_file() {
        return Some(candidate_path.to_path_buf());
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(candidate))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_have_distinct_commands_and_capabilities() {
        assert_eq!(KernelId::ALL.len(), 3);
        assert_eq!(
            KernelId::SingBox.config_args(Path::new("config.json")),
            ["run", "-c", "config.json"]
        );
        assert_eq!(
            KernelId::Mihomo.config_args(Path::new("config.yaml")),
            ["-f", "config.yaml"]
        );
        assert_eq!(KernelId::Xray.version_args(), ["version"]);
    }

    #[test]
    fn registry_starts_fail_closed() {
        let registry = KernelRegistry::new();
        assert!(matches!(
            registry.require_installed(KernelId::SingBox),
            Err(KernelError::NotInstalled(KernelId::SingBox))
        ));
        assert_eq!(
            registry.record(KernelId::Mihomo).state,
            KernelState::NotInstalled
        );
    }

    #[test]
    fn state_keeps_health_separate_from_running() {
        let mut registry = KernelRegistry::new();
        registry.set_state(KernelId::SingBox, KernelState::Running, false, None);
        let record = registry.record(KernelId::SingBox);
        assert_eq!(record.state, KernelState::Running);
        assert!(!record.healthy);
    }
}
