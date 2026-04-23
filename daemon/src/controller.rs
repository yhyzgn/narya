use crate::tracker::ProcessTracker;
use anyhow::Result;
use config::diff::ConfigDiff;
use config::matcher::RuleEngine;
use config::model::{Action, NaryaConfig};
use narya_core::singbox::{MockSingBox, SingBoxCore};
use platform::{LinuxAdapter, PlatformAdapter};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NaryaDaemon {
    config: RwLock<NaryaConfig>,
    engine: RwLock<RuleEngine>,
    core: Arc<dyn SingBoxCore>,
    platform: Arc<dyn PlatformAdapter>,
    tracker: Arc<dyn ProcessTracker>,
}

impl Default for NaryaDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl NaryaDaemon {
    pub fn new() -> Self {
        let initial_config = NaryaConfig::default();
        let engine = RuleEngine::new(initial_config.rules.clone(), Action::Direct);

        #[cfg(target_os = "linux")]
        let tracker = Arc::new(crate::tracker::EbpfProcessTracker::new());
        #[cfg(not(target_os = "linux"))]
        let tracker = std::sync::Arc::new(crate::tracker::SystemProcessTracker::new());

        Self {
            config: RwLock::new(initial_config),
            engine: RwLock::new(engine),
            core: Arc::new(MockSingBox::new()),
            platform: Arc::new(LinuxAdapter),
            tracker,
        }
    }

    pub fn get_tracker(&self) -> Arc<dyn ProcessTracker> {
        self.tracker.clone()
    }

    pub async fn start(&self) -> Result<()> {
        let config = self.config.read().await;

        // Anti-Loop: Detect physical interface
        tracing::info!("Anti-Loop: Detecting physical interfaces to bypass encrypted traffic...");

        // Setup Platform
        if config.settings.tun.enabled {
            self.platform.setup_tun(&config.settings.tun.device).await?;
        }
        if config.settings.system_proxy {
            self.platform
                .set_system_proxy("127.0.0.1", config.settings.mixed_port)
                .await?;
        }

        // Start Tracker
        self.tracker.start().await?;

        let config_json = serde_json::to_string(&*config)?;
        self.core.start(&config_json)?;

        tracing::info!("Narya Daemon started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let config = self.config.read().await;

        self.core.stop()?;

        // Stop Tracker
        self.tracker.stop().await?;

        if config.settings.tun.enabled {
            self.platform
                .remove_tun(&config.settings.tun.device)
                .await?;
        }
        if config.settings.system_proxy {
            self.platform.clear_system_proxy().await?;
        }

        tracing::info!("Narya Daemon stopped");
        Ok(())
    }

    pub async fn update_config(&self, new_config: NaryaConfig) -> Result<()> {
        let mut config_lock = self.config.write().await;
        let diff = ConfigDiff::calculate(&config_lock, &new_config);

        if diff.has_changes() {
            // Update platform settings if they changed
            if diff.settings_changed {
                if new_config.settings.tun.enabled && !config_lock.settings.tun.enabled {
                    self.platform
                        .setup_tun(&new_config.settings.tun.device)
                        .await?;
                } else if !new_config.settings.tun.enabled && config_lock.settings.tun.enabled {
                    self.platform
                        .remove_tun(&config_lock.settings.tun.device)
                        .await?;
                }

                if new_config.settings.system_proxy && !config_lock.settings.system_proxy {
                    self.platform
                        .set_system_proxy("127.0.0.1", new_config.settings.mixed_port)
                        .await?;
                } else if !new_config.settings.system_proxy && config_lock.settings.system_proxy {
                    self.platform.clear_system_proxy().await?;
                }
            }

            let new_config_json = serde_json::to_string(&new_config)?;
            self.core.reload(diff, &new_config_json)?;

            // Update RuleEngine
            let mut engine_lock = self.engine.write().await;
            *engine_lock = RuleEngine::new(new_config.rules.clone(), Action::Direct);

            *config_lock = new_config;
            tracing::info!("Configuration and RuleEngine updated");
        } else {
            tracing::info!("No configuration changes detected");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daemon_lifecycle() {
        let daemon = NaryaDaemon::new();
        daemon.start().await.unwrap();

        let mut new_config = NaryaConfig::default();
        new_config.settings.mixed_port = 9090;
        new_config.settings.tun.enabled = true;

        daemon.update_config(new_config).await.unwrap();

        let processes = daemon.get_tracker().list_running_processes().unwrap();
        assert!(!processes.is_empty());

        daemon.stop().await.unwrap();
    }
}
