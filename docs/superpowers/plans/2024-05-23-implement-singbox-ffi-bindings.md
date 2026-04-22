# Implement Rust FFI Bindings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement real FFI bindings to sing-box in Rust and verify with tests.

**Architecture:** Update `narya-core/src/singbox.rs` to use `extern "C"` declarations that link to the sing-box library built via Go. Implement the `SingBoxCore` trait using these FFI calls.

**Tech Stack:** Rust, FFI, libc, anyhow, sing-box (Go)

---

### Task 1: Update narya-core/src/singbox.rs

**Files:**
- Modify: `narya-core/src/singbox.rs`

- [ ] **Step 1: Replace content of narya-core/src/singbox.rs with the provided implementation**

```rust
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
        // 提供一个极简的合法 JSON 配置
        let config = r#"{"log": {"disabled": true}}"#;
        
        let result = sb.start(config);
        assert!(result.is_ok());

        let stop_result = sb.stop();
        assert!(stop_result.is_ok());
    }
}
```

### Task 2: Verify and Commit

- [ ] **Step 1: Run tests for narya-core**

Run: `cargo test -p narya-core -- --nocapture`
Expected: PASS

- [ ] **Step 2: Commit the changes**

```bash
git add narya-core/src/singbox.rs
git commit -m "feat(narya-core): implement real FFI calls to sing-box"
```
