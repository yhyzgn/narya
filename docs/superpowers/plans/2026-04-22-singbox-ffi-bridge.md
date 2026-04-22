# Sing-box FFI Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a reliable C-ABI bridge between Rust (Narya Core) and Go (Sing-box) to enable real proxy routing.

**Architecture:** A lightweight Go wrapper will be created in a new `singbox-bridge` directory. This wrapper will export C-compatible functions (like `sing_box_start`) using `cgo`. During the Rust build process, a `build.rs` script will automatically compile the Go code into a static library (`libsingbox.a`) and link it to the `narya-core` crate. This achieves a single, statically-linked binary.

**Tech Stack:** Rust, Go (cgo), `build.rs` (Cargo build scripts).

---

### Task 1: Setup Go Bridge Module

**Files:**
- Create: `narya-core/singbox-bridge/go.mod`
- Create: `narya-core/singbox-bridge/main.go`

- [ ] **Step 1: Initialize the Go module**

```bash
mkdir -p narya-core/singbox-bridge
cd narya-core/singbox-bridge
go mod init narya-singbox
go get github.com/sagernet/sing-box@latest
```

- [ ] **Step 2: Write the Go C-API wrapper**

Write the following to `narya-core/singbox-bridge/main.go`:

```go
package main

/*
#include <stdlib.h>
*/
import "C"
import (
	"context"
	"fmt"
	"strings"
	"unsafe"

	"github.com/sagernet/sing-box/option"
	"github.com/sagernet/sing-box/box"
)

var (
	currentBox *box.Box
	cancelFunc context.CancelFunc
)

//export sing_box_start
func sing_box_start(configJson *C.char) C.int {
	if currentBox != nil {
		return -1 // Already running
	}

	jsonStr := C.GoString(configJson)
	
	// Create a minimal context
	ctx, cancel := context.WithCancel(context.Background())
	cancelFunc = cancel

	// Simple option parse
	var opts option.Options
	if err := opts.UnmarshalJSON([]byte(jsonStr)); err != nil {
		fmt.Printf("Failed to parse config: %v\n", err)
		return -2
	}

	b, err := box.New(box.Options{
		Context: ctx,
		Options: opts,
	})
	
	if err != nil {
		fmt.Printf("Failed to create box: %v\n", err)
		return -3
	}

	if err := b.Start(); err != nil {
		fmt.Printf("Failed to start box: %v\n", err)
		return -4
	}

	currentBox = b
	return 0 // Success
}

//export sing_box_stop
func sing_box_stop() C.int {
	if currentBox == nil {
		return 0
	}

	if err := currentBox.Close(); err != nil {
		fmt.Printf("Failed to close box: %v\n", err)
		return -1
	}

	if cancelFunc != nil {
		cancelFunc()
	}

	currentBox = nil
	return 0
}

// Main function required for c-archive
func main() {}
```

- [ ] **Step 3: Run go mod tidy**

Run: `cd narya-core/singbox-bridge && go mod tidy`
Expected: Download dependencies and update `go.sum`.

- [ ] **Step 4: Commit**

```bash
git add narya-core/singbox-bridge/
git commit -m "feat(narya-core): add go bridge for sing-box"
```

---

### Task 2: Configure Rust Build Script

**Files:**
- Create: `narya-core/build.rs`
- Modify: `narya-core/Cargo.toml`

- [ ] **Step 1: Add build.rs to Cargo.toml**

Update `narya-core/Cargo.toml` to include a build script.

```toml
[package]
name = "narya-core"
version = "0.1.0"
edition = "2024"
build = "build.rs"

[dependencies]
config = { path = "../config" }
anyhow = { workspace = true }
tracing = { workspace = true }
libc = { workspace = true }
```

- [ ] **Step 2: Write build.rs**

Create `narya-core/build.rs`:

```rust
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bridge_dir = PathBuf::from(manifest_dir).join("singbox-bridge");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Only rebuild if the Go code changes
    println!("cargo:rerun-if-changed=singbox-bridge/main.go");
    println!("cargo:rerun-if-changed=singbox-bridge/go.mod");
    println!("cargo:rerun-if-changed=singbox-bridge/go.sum");

    let lib_name = "singbox";
    let static_lib_name = format!("lib{}.a", lib_name);
    let static_lib_path = out_dir.join(&static_lib_name);

    // Run `go build -buildmode=c-archive -o <out_dir>/libsingbox.a`
    let status = Command::new("go")
        .current_dir(&bridge_dir)
        .env("CGO_ENABLED", "1")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(&static_lib_path)
        .status()
        .expect("Failed to execute go build");

    assert!(status.success(), "Go build failed");

    // Link the static library
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", lib_name);

    // Depending on the OS, we might need to link additional libraries (like CoreFoundation on macOS)
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
    } else if target_os == "linux" {
        println!("cargo:rustc-link-lib=pthread");
    }
}
```

- [ ] **Step 3: Test compilation**

Run: `cargo build -p narya-core`
Expected: Successful compilation without errors (the build script will compile the Go code).

- [ ] **Step 4: Commit**

```bash
git add narya-core/Cargo.toml narya-core/build.rs
git commit -m "build(narya-core): add build.rs to compile and link go bridge"
```

---

### Task 3: Implement Rust FFI Bindings

**Files:**
- Modify: `narya-core/src/singbox.rs`

- [ ] **Step 1: Write a test that fails (since we haven't wired up the real FFI struct yet)**

At the bottom of `narya-core/src/singbox.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singbox_ffi_lifecycle() {
        let sb = SingBoxFfi;
        // Invalid config should fail gracefully, but for now we just test the C-ABI call
        // A minimal valid JSON is needed to bypass JSON parsing errors.
        let config = r#"{"log": {"disabled": true}}"#;
        
        let result = sb.start(config);
        assert!(result.is_ok());

        let stop_result = sb.stop();
        assert!(stop_result.is_ok());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p narya-core`
Expected: The test will fail or panic if `sing_box_start` is missing or panics. Currently `SingBoxFfi` is partially mocked.

- [ ] **Step 3: Write minimal implementation**

Update `narya-core/src/singbox.rs` to use the real C functions:

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
        // Sing-box hot reload is complex. For phase 1, we restart.
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
        let config = r#"{"log": {"disabled": true}}"#;
        
        let result = sb.start(config);
        assert!(result.is_ok());

        let stop_result = sb.stop();
        assert!(stop_result.is_ok());
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p narya-core -- --nocapture`
Expected: PASS, with the actual CGo linkage functioning.

- [ ] **Step 5: Commit**

```bash
git add narya-core/src/singbox.rs
git commit -m "feat(narya-core): implement real FFI calls to sing-box"
```
