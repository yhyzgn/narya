use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bridge_dir = PathBuf::from(manifest_dir).join("singbox-bridge");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=singbox-bridge/main.go");
    println!("cargo:rerun-if-changed=singbox-bridge/go.mod");
    println!("cargo:rerun-if-changed=singbox-bridge/go.sum");

    let lib_name = "singbox";
    let static_lib_name = format!("lib{}.a", lib_name);
    let static_lib_path = out_dir.join(&static_lib_name);

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

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", lib_name);

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
    } else if target_os == "linux" {
        println!("cargo:rustc-link-lib=pthread");
    }
}
