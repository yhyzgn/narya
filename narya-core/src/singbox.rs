use anyhow::Result;
use config::diff::ConfigDiff;
use std::ffi::CString;
use libc::c_char;

// Actual Sing-box C-ABI definitions (linked from Go/Sing-box)
unsafe extern "C" {
    /// Starts the sing-box core with the given JSON configuration.
    /// Returns 0 on success, non-zero on failure.
    pub fn sing_box_start(config_json: *const c_char) -> i32;

    /// Stops the sing-box core.
    pub fn sing_box_stop() -> i32;

    /// Reloads the sing-box core with a new JSON configuration.
    /// This is a simplified view of the hot-reload mechanism.
    pub fn sing_box_reload(config_json: *const c_char) -> i32;
}

pub trait SingBoxCore: Send + Sync {
    fn start(&self, config_json: &str) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn reload(&self, diff: ConfigDiff, new_config_json: &str) -> Result<()>;
}

pub struct SingBoxFfi;

impl SingBoxCore for SingBoxFfi {
    fn start(&self, config_json: &str) -> Result<()> {
        tracing::info!("Starting sing-box via FFI");
        let c_config = CString::new(config_json)?;
        // In a real build, we would link against the sing-box C library
        // unsafe { sing_box_start(c_config.as_ptr()); }
        let _ = c_config;
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        tracing::info!("Stopping sing-box via FFI");
        // unsafe { sing_box_stop(); }
        Ok(())
    }

    fn reload(&self, _diff: ConfigDiff, new_config_json: &str) -> Result<()> {
        tracing::info!("Reloading sing-box via FFI");
        let c_config = CString::new(new_config_json)?;
        // unsafe { sing_box_reload(c_config.as_ptr()); }
        let _ = c_config;
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
