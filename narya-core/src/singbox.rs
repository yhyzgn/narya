use anyhow::{Result, bail};
use config::diff::ConfigDiff;
use std::ffi::CString;
use libc::c_char;

unsafe extern "C" {
    pub fn sing_box_start(config_json: *const c_char) -> i32;
    pub fn sing_box_stop() -> i32;
}

pub trait SingBoxCore: Send + Sync {
    fn start(&self, config_json: &str) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn reload(&self, diff: ConfigDiff, new_config_json: &str) -> Result<()>;
}

pub struct SingBoxFfi;

impl SingBoxCore for SingBoxFfi {
    fn start(&self, config_json: &str) -> Result<()> {
        tracing::info!("Starting sing-box via real FFI");
        let c_config = CString::new(config_json)?;
        let result = unsafe { sing_box_start(c_config.as_ptr()) };
        
        if result != 0 {
            bail!("Sing-box failed to start with code: {}", result);
        }
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        tracing::info!("Stopping sing-box via real FFI");
        let result = unsafe { sing_box_stop() };
        if result != 0 {
            bail!("Sing-box failed to stop with code: {}", result);
        }
        Ok(())
    }

    fn reload(&self, _diff: ConfigDiff, new_config_json: &str) -> Result<()> {
        tracing::info!("Reloading sing-box (restart mechanism)");
        self.stop()?;
        self.start(new_config_json)?;
        Ok(())
    }
}

pub struct MockSingBox;

impl MockSingBox {
    pub fn new() -> Self {
        Self
    }
}

impl SingBoxCore for MockSingBox {
    fn start(&self, _config_json: &str) -> Result<()> {
        tracing::info!("Mock: sing-box started");
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        tracing::info!("Mock: sing-box stopped");
        Ok(())
    }

    fn reload(&self, _diff: ConfigDiff, _new_config_json: &str) -> Result<()> {
        tracing::info!("Mock: sing-box reloaded");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singbox_ffi_lifecycle() {
        let sb = SingBoxFfi;
        // 提供一个包含 inbound 的配置，某些版本的 sing-box 可能需要至少一个 inbound
        let config = r#"{
            "log": {"level": "info"},
            "inbounds": [
                {
                    "type": "mixed",
                    "listen": "127.0.0.1",
                    "listen_port": 20086
                }
            ]
        }"#;
        
        let result = sb.start(config);
        assert!(result.is_ok());

        let stop_result = sb.stop();
        assert!(stop_result.is_ok());
    }
}
