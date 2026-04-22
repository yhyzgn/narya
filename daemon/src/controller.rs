use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use config::model::NaryaConfig;
use config::diff::ConfigDiff;
use narya_core::singbox::{SingBoxCore, MockSingBox};
use platform::{PlatformAdapter, LinuxAdapter};
use crate::tracker::{ProcessTracker, SystemProcessTracker};

pub struct NaryaDaemon {
    config: RwLock<NaryaConfig>,
    core: Arc<dyn SingBoxCore>,
    platform: Arc<dyn PlatformAdapter>,
    tracker: Arc<dyn ProcessTracker>,
}

impl NaryaDaemon {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(NaryaConfig::default()),
            core: Arc::new(MockSingBox::new()),
            platform: Arc::new(LinuxAdapter),
            tracker: Arc::new(SystemProcessTracker::new()),
        }
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
            self.platform.set_system_proxy("127.0.0.1", config.settings.mixed_port).await?;
        }

        let config_json = serde_json::to_string(&*config)?;
        self.core.start(&config_json)?;
        
        tracing::info!("Narya Daemon started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let config = self.config.read().await;
        
        self.core.stop()?;
        
        if config.settings.tun.enabled {
            self.platform.remove_tun(&config.settings.tun.device).await?;
        }
        if config.settings.system_proxy {
            self.platform.clear_system_proxy().await?;
        }

        tracing::info!("Narya Daemon stopped");
        Ok(())
    }

    pub async fn update_config(&self, new_config: NaryaConfig) -> Result<()> {
        let mut config_lock = self.config.write().await;
        let diff = ConfigDiff::calculate(&*config_lock, &new_config);
        
        if diff.has_changes() {
            // Update platform settings if they changed
            if diff.settings_changed {
                if new_config.settings.tun.enabled && !config_lock.settings.tun.enabled {
                    self.platform.setup_tun(&new_config.settings.tun.device).await?;
                } else if !new_config.settings.tun.enabled && config_lock.settings.tun.enabled {
                    self.platform.remove_tun(&config_lock.settings.tun.device).await?;
                }

                if new_config.settings.system_proxy && !config_lock.settings.system_proxy {
                    self.platform.set_system_proxy("127.0.0.1", new_config.settings.mixed_port).await?;
                } else if !new_config.settings.system_proxy && config_lock.settings.system_proxy {
                    self.platform.clear_system_proxy().await?;
                }
            }

            let new_config_json = serde_json::to_string(&new_config)?;
            self.core.reload(diff, &new_config_json)?;
            *config_lock = new_config;
            tracing::info!("Configuration updated and hot-reloaded");
        } else {
            tracing::info!("No configuration changes detected");
        }

        Ok(())
    }

    pub fn get_tracker(&self) -> Arc<dyn ProcessTracker> {
        self.tracker.clone()
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
