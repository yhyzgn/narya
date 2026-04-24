use crate::config_manager::ConfigManager;
use crate::tracker::ProcessTracker;
use anyhow::Result;
use config::diff::ConfigDiff;
use config::matcher::RuleEngine;
use config::model::{Action, NaryaConfig};
use config::transformer::Transformer;
use narya_core::singbox::{SingBoxCore, SingBoxFfi};
use platform::{LinuxAdapter, PlatformAdapter};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NaryaDaemon {
    config: RwLock<NaryaConfig>,
    engine: RwLock<RuleEngine>,
    core: Arc<dyn SingBoxCore>,
    platform: Arc<dyn PlatformAdapter>,
    tracker: Arc<dyn ProcessTracker>,
    config_manager: Arc<ConfigManager>,
}

impl Default for NaryaDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl NaryaDaemon {
    pub fn new() -> Self {
        let config_manager = Arc::new(ConfigManager::new().expect("Failed to init ConfigManager"));

        let initial_config = NaryaConfig::default();
        let engine = RuleEngine::new(initial_config.rules.clone(), Action::Direct);

        #[cfg(target_os = "linux")]
        let tracker = Arc::new(crate::tracker::EbpfProcessTracker::new());
        #[cfg(not(target_os = "linux"))]
        let tracker = std::sync::Arc::new(crate::tracker::SystemProcessTracker::new());

        Self {
            config: RwLock::new(initial_config),
            engine: RwLock::new(engine),
            core: Arc::new(SingBoxFfi),
            platform: Arc::new(LinuxAdapter),
            tracker,
            config_manager,
        }
    }

    pub fn get_tracker(&self) -> Arc<dyn ProcessTracker> {
        self.tracker.clone()
    }

    pub fn get_config_manager(&self) -> Arc<ConfigManager> {
        self.config_manager.clone()
    }

    pub async fn get_config(&self) -> NaryaConfig {
        self.config.read().await.clone()
    }

    pub async fn start(&self) -> Result<()> {
        let config = self.config.read().await;

        // 启动时应用持久化规则到 eBPF
        let state = self.config_manager.get_state();
        self.tracker
            .update_bypass_rules(&state.bypass_rules)
            .await?;

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

        let active_node = state.active_node.as_deref();
        let config_json = Transformer::transform(&config, active_node);
        tracing::info!("Starting sing-box with config: {}", config_json);
        self.core.start(&config_json)?;

        tracing::info!("Narya Daemon started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let config = self.config.read().await;
        self.core.stop()?;
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

            let state = self.config_manager.get_state();
            let active_node = state.active_node.as_deref();
            let new_config_json = Transformer::transform(&new_config, active_node);
            self.core.reload(diff, &new_config_json)?;

            let mut engine_lock = self.engine.write().await;
            *engine_lock = RuleEngine::new(new_config.rules.clone(), Action::Direct);

            *config_lock = new_config;
            tracing::info!("Configuration and RuleEngine updated");
        }
        Ok(())
    }
}
