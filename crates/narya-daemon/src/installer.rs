use anyhow::{anyhow, bail, Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use narya_kernel::KernelId;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;

const MAX_ARTIFACT_SIZE: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
pub struct KernelArtifactRequest {
    pub kernel: KernelId,
    pub version: String,
    pub source: String,
    pub sha256: String,
    #[serde(default)]
    pub signature: String,
    #[serde(default)]
    pub public_key: String,
    #[serde(default)]
    pub catalog_version: String,
    #[serde(default)]
    pub catalog_platform: String,
    #[serde(default)]
    pub catalog_architecture: String,
}

#[derive(Debug, Clone)]
pub struct InstalledKernel {
    pub kernel: KernelId,
    pub version: String,
    pub binary_path: PathBuf,
}

/// Verify the immutable integrity record written next to a managed kernel.
///
/// This is intentionally performed immediately before spawning a kernel. An
/// install directory is user-writable on some platforms, so discovery alone
/// must not be treated as proof that the executable is still the verified
/// artifact.
pub async fn verify_installed(binary_path: &Path) -> Result<()> {
    let digest_path = binary_path.with_file_name("sha256");
    for path in [binary_path, digest_path.as_path()] {
        let metadata = fs::symlink_metadata(path)
            .await
            .with_context(|| format!("missing managed kernel file {}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!(
                "managed kernel path is not a regular file: {}",
                path.display()
            );
        }
    }
    let expected = fs::read_to_string(&digest_path)
        .await
        .with_context(|| format!("missing integrity record for {}", binary_path.display()))?;
    let expected = expected.trim();
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!(
            "invalid integrity record for {}: expected a SHA-256 digest",
            binary_path.display()
        );
    }
    let bytes = fs::read(binary_path)
        .await
        .with_context(|| format!("failed to read managed kernel {}", binary_path.display()))?;
    verify_sha256(&bytes, expected)
}

pub async fn install(
    request: &KernelArtifactRequest,
    install_root: &Path,
    upgrading: bool,
) -> Result<InstalledKernel> {
    validate_request(request)?;
    let bytes = fetch_artifact(&request.source).await?;
    if bytes.len() > MAX_ARTIFACT_SIZE {
        bail!(
            "kernel artifact is too large: {} bytes (limit {})",
            bytes.len(),
            MAX_ARTIFACT_SIZE
        );
    }
    verify_sha256(&bytes, &request.sha256)?;
    verify_signature_if_present(request, &bytes)?;

    let kernel_dir = install_root.join(request.kernel.as_str());
    fs::create_dir_all(&kernel_dir).await?;
    let suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default()
    );
    let temp_path = kernel_dir.join(format!(".narya-{suffix}.tmp"));
    let binary_path = kernel_dir.join("current");
    let version_path = kernel_dir.join("version");
    let sha_path = kernel_dir.join("sha256");
    let temp_version_path = kernel_dir.join(format!(".narya-{suffix}.version.tmp"));
    let temp_sha_path = kernel_dir.join(format!(".narya-{suffix}.sha256.tmp"));
    let backup_binary_path = kernel_dir.join(format!(".narya-{suffix}.current.backup"));
    let backup_version_path = kernel_dir.join(format!(".narya-{suffix}.version.backup"));
    let backup_sha_path = kernel_dir.join(format!(".narya-{suffix}.sha256.backup"));

    fs::write(&temp_path, &bytes)
        .await
        .with_context(|| format!("failed to write temporary {} artifact", request.kernel))?;
    set_executable(&temp_path).await?;
    fs::write(&temp_version_path, request.version.as_bytes()).await?;
    fs::write(
        &temp_sha_path,
        request.sha256.to_ascii_lowercase().as_bytes(),
    )
    .await?;

    // Stage all files first, then replace the active set as one rollbackable
    // transaction. In particular, a metadata failure must never destroy the
    // previous executable during an upgrade.
    let had_binary = binary_path.is_file();
    let had_version = version_path.is_file();
    let had_sha = sha_path.is_file();
    let cleanup_temps = || async {
        let _ = fs::remove_file(&temp_path).await;
        let _ = fs::remove_file(&temp_version_path).await;
        let _ = fs::remove_file(&temp_sha_path).await;
    };
    let restore = || async {
        let _ = fs::remove_file(&binary_path).await;
        let _ = fs::remove_file(&version_path).await;
        let _ = fs::remove_file(&sha_path).await;
        if had_binary {
            let _ = fs::rename(&backup_binary_path, &binary_path).await;
        }
        if had_version {
            let _ = fs::rename(&backup_version_path, &version_path).await;
        }
        if had_sha {
            let _ = fs::rename(&backup_sha_path, &sha_path).await;
        }
        cleanup_temps().await;
    };

    if had_binary {
        fs::rename(&binary_path, &backup_binary_path).await?;
    }
    if had_version {
        if let Err(error) = fs::rename(&version_path, &backup_version_path).await {
            restore().await;
            return Err(error).context("failed to stage previous kernel version");
        }
    }
    if had_sha {
        if let Err(error) = fs::rename(&sha_path, &backup_sha_path).await {
            restore().await;
            return Err(error).context("failed to stage previous kernel integrity record");
        }
    }
    if let Err(error) = fs::rename(&temp_path, &binary_path).await {
        restore().await;
        return Err(error).with_context(|| {
            format!(
                "failed to atomically {} kernel {}",
                if upgrading { "upgrade" } else { "install" },
                request.kernel
            )
        });
    }
    if let Err(error) = fs::rename(&temp_version_path, &version_path).await {
        restore().await;
        return Err(error).context("failed to persist installed kernel version");
    }
    if let Err(error) = fs::rename(&temp_sha_path, &sha_path).await {
        restore().await;
        return Err(error).context("failed to persist installed kernel integrity record");
    }
    let _ = fs::remove_file(&backup_binary_path).await;
    let _ = fs::remove_file(&backup_version_path).await;
    let _ = fs::remove_file(&backup_sha_path).await;

    Ok(InstalledKernel {
        kernel: request.kernel,
        version: request.version.clone(),
        binary_path,
    })
}

/// Remove one kernel from the application-private managed install root.
///
/// This intentionally accepts a kernel id and root separately instead of an
/// arbitrary path, so callers cannot turn the uninstall operation into a
/// system-wide file delete. Unknown files are preserved for recovery/debugging.
pub async fn uninstall(kernel: KernelId, install_root: &Path) -> Result<()> {
    let kernel_dir = install_root.join(kernel.as_str());
    let directory = fs::symlink_metadata(&kernel_dir)
        .await
        .with_context(|| format!("kernel {kernel} is not installed in managed storage"))?;
    if !directory.is_dir() {
        bail!(
            "managed kernel path is not a directory: {}",
            kernel_dir.display()
        );
    }

    let names = ["current", "version", "sha256"];
    let mut managed_files = Vec::new();
    for name in names {
        let path = kernel_dir.join(name);
        let metadata = match fs::symlink_metadata(&path).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("failed to inspect {}", path.display()))
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!(
                "refusing to uninstall unexpected managed path: {}",
                path.display()
            );
        }
        managed_files.push(path);
    }
    if !managed_files.iter().any(|path| path.ends_with("current")) {
        bail!("kernel {kernel} is not installed in managed storage");
    }

    let suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default()
    );
    let mut staged = Vec::new();
    for path in managed_files {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid managed kernel filename"))?;
        let backup = kernel_dir.join(format!(".narya-uninstall-{suffix}-{name}"));
        if let Err(error) = fs::rename(&path, &backup).await {
            for (original, staged_path) in staged.into_iter().rev() {
                let _ = fs::rename(staged_path, original).await;
            }
            return Err(error).with_context(|| {
                format!("failed to stage managed kernel file {}", path.display())
            });
        }
        staged.push((path, backup));
    }
    for (_, backup) in &staged {
        let _ = fs::remove_file(backup).await;
    }
    let mut entries = fs::read_dir(&kernel_dir).await?;
    if entries.next_entry().await?.is_none() {
        fs::remove_dir(&kernel_dir).await?;
    }
    Ok(())
}

fn validate_request(request: &KernelArtifactRequest) -> Result<()> {
    if request.version.trim().is_empty() {
        bail!("kernel artifact version must not be empty");
    }
    if request.source.trim().is_empty() {
        bail!("kernel artifact source must not be empty");
    }
    if request.sha256.len() != 64 || !request.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("kernel artifact sha256 must be 64 hexadecimal characters");
    }
    let has_signature = !request.signature.trim().is_empty();
    let has_public_key = !request.public_key.trim().is_empty();
    if has_signature != has_public_key {
        bail!("kernel artifact signature and public key must be supplied together");
    }
    if request.source.starts_with("https://") && !has_signature {
        bail!("HTTPS kernel artifacts require an Ed25519 signature and public key");
    }
    Ok(())
}

fn verify_signature_if_present(request: &KernelArtifactRequest, bytes: &[u8]) -> Result<()> {
    if request.signature.trim().is_empty() {
        return Ok(());
    }
    let public_key = decode_hex(&request.public_key, 32, "public key")?;
    let signature = decode_hex(&request.signature, 64, "signature")?;
    let public_key = VerifyingKey::from_bytes(
        &public_key
            .try_into()
            .map_err(|_| anyhow!("invalid Ed25519 public key length"))?,
    )
    .map_err(|error| anyhow!("invalid Ed25519 public key: {error}"))?;
    let signature = Signature::from_bytes(
        &signature
            .try_into()
            .map_err(|_| anyhow!("invalid Ed25519 signature length"))?,
    );
    public_key
        .verify(bytes, &signature)
        .map_err(|error| anyhow!("kernel artifact signature verification failed: {error}"))
}

#[cfg(test)]
mod uninstall_tests {
    use super::*;

    fn test_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/narya-installer-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn uninstall_removes_only_managed_kernel_files() {
        let root = test_root();
        let dir = root.join(KernelId::Mihomo.as_str());
        fs::create_dir_all(&dir).await.unwrap();
        fs::write(dir.join("current"), b"binary").await.unwrap();
        fs::write(dir.join("version"), b"1.0.0").await.unwrap();
        fs::write(dir.join("sha256"), "0".repeat(64)).await.unwrap();
        fs::write(dir.join("keep.txt"), b"leave unknown files")
            .await
            .unwrap();

        uninstall(KernelId::Mihomo, &root).await.unwrap();
        assert!(!dir.join("current").exists());
        assert!(!dir.join("version").exists());
        assert!(!dir.join("sha256").exists());
        assert!(dir.join("keep.txt").exists());
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn uninstall_rejects_missing_managed_kernel() {
        let root = test_root();
        let error = uninstall(KernelId::Xray, &root).await.unwrap_err();
        assert!(error
            .to_string()
            .contains("not installed in managed storage"));
    }
}

fn decode_hex(value: &str, expected_len: usize, label: &str) -> Result<Vec<u8>> {
    if value.len() != expected_len * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} must be {expected_len} bytes encoded as hexadecimal");
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|error| anyhow!("invalid {label}: {error}"))
        })
        .collect()
}

async fn fetch_artifact(source: &str) -> Result<Vec<u8>> {
    if let Some(path) = source.strip_prefix("file://") {
        return fs::read(path)
            .await
            .with_context(|| format!("failed to read kernel artifact {path}"));
    }
    if !source.starts_with("https://") {
        let path = Path::new(source);
        if path.is_absolute() || path.components().count() > 1 {
            return fs::read(path)
                .await
                .with_context(|| format!("failed to read kernel artifact {}", path.display()));
        }
        bail!("kernel artifact source must be an absolute path, file:// URL, or HTTPS URL");
    }

    let response = reqwest::get(source)
        .await
        .with_context(|| format!("failed to download kernel artifact {source}"))?;
    if !response.status().is_success() {
        bail!(
            "kernel artifact download returned HTTP {}",
            response.status()
        );
    }
    let bytes = response.bytes().await?.to_vec();
    Ok(bytes)
}

fn verify_sha256(bytes: &[u8], expected: &str) -> Result<()> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(anyhow!(
            "kernel artifact checksum mismatch: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

async fn set_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).await?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/narya-installer-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn installs_local_artifact_after_checksum_verification() {
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let source = root.join("source");
        fs::write(&source, b"kernel-binary").await.unwrap();
        let checksum = format!("{:x}", Sha256::digest(b"kernel-binary"));
        let request = KernelArtifactRequest {
            kernel: KernelId::SingBox,
            version: "1.0.0".into(),
            source: source.to_string_lossy().into_owned(),
            sha256: checksum.clone(),
            signature: String::new(),
            public_key: String::new(),
            catalog_version: String::new(),
            catalog_platform: String::new(),
            catalog_architecture: String::new(),
        };
        let installed = install(&request, &root.join("kernels"), false)
            .await
            .unwrap();
        assert_eq!(
            fs::read(&installed.binary_path).await.unwrap(),
            b"kernel-binary"
        );
        assert_eq!(
            fs::read_to_string(root.join("kernels/sing-box/version"))
                .await
                .unwrap(),
            "1.0.0"
        );
        assert_eq!(
            fs::read_to_string(root.join("kernels/sing-box/sha256"))
                .await
                .unwrap(),
            checksum
        );
        verify_installed(&installed.binary_path).await.unwrap();
        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn checksum_failure_does_not_replace_existing_binary() {
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let kernel_dir = root.join("kernels/sing-box");
        fs::create_dir_all(&kernel_dir).await.unwrap();
        fs::write(kernel_dir.join("current"), b"old").await.unwrap();
        let source = root.join("source");
        fs::write(&source, b"new").await.unwrap();
        let request = KernelArtifactRequest {
            kernel: KernelId::SingBox,
            version: "2.0.0".into(),
            source: source.to_string_lossy().into_owned(),
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            signature: String::new(),
            public_key: String::new(),
            catalog_version: String::new(),
            catalog_platform: String::new(),
            catalog_architecture: String::new(),
        };
        assert!(install(&request, &root.join("kernels"), true)
            .await
            .is_err());
        assert_eq!(fs::read(kernel_dir.join("current")).await.unwrap(), b"old");
        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn signed_artifact_is_verified_before_install() {
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let source = root.join("source");
        let bytes = b"signed-kernel";
        fs::write(&source, bytes).await.unwrap();
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let signature = signing.sign(bytes);
        let request = KernelArtifactRequest {
            kernel: KernelId::SingBox,
            version: "3.0.0".into(),
            source: source.to_string_lossy().into_owned(),
            sha256: format!("{:x}", Sha256::digest(bytes)),
            signature: hex_bytes(signature.to_bytes()),
            public_key: hex_bytes(signing.verifying_key().to_bytes()),
            catalog_version: String::new(),
            catalog_platform: String::new(),
            catalog_architecture: String::new(),
        };
        let installed = install(&request, &root.join("kernels"), false)
            .await
            .unwrap();
        assert_eq!(fs::read(installed.binary_path).await.unwrap(), bytes);
        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn metadata_commit_failure_restores_previous_upgrade() {
        let root = temp_root();
        let kernel_dir = root.join("kernels/sing-box");
        fs::create_dir_all(&kernel_dir).await.unwrap();
        fs::write(kernel_dir.join("current"), b"old-kernel")
            .await
            .unwrap();
        fs::write(kernel_dir.join("version"), b"1.0.0")
            .await
            .unwrap();
        // A directory at the metadata destination makes the final metadata
        // rename fail after the new executable has already been staged.
        fs::create_dir(kernel_dir.join("sha256")).await.unwrap();

        let source = root.join("source");
        fs::write(&source, b"new-kernel").await.unwrap();
        let request = KernelArtifactRequest {
            kernel: KernelId::SingBox,
            version: "2.0.0".into(),
            source: source.to_string_lossy().into_owned(),
            sha256: format!("{:x}", Sha256::digest(b"new-kernel")),
            signature: String::new(),
            public_key: String::new(),
            catalog_version: String::new(),
            catalog_platform: String::new(),
            catalog_architecture: String::new(),
        };

        assert!(install(&request, &root.join("kernels"), true)
            .await
            .is_err());
        assert_eq!(
            fs::read(kernel_dir.join("current")).await.unwrap(),
            b"old-kernel"
        );
        assert_eq!(
            fs::read_to_string(kernel_dir.join("version"))
                .await
                .unwrap(),
            "1.0.0"
        );
        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn tampered_installed_artifact_is_rejected() {
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let binary = root.join("current");
        fs::write(&binary, b"kernel").await.unwrap();
        fs::write(
            root.join("sha256"),
            format!("{:x}", Sha256::digest(b"different")),
        )
        .await
        .unwrap();
        let error = verify_installed(&binary).await.unwrap_err().to_string();
        assert!(error.contains("checksum mismatch"));
        let _ = fs::remove_dir_all(root).await;
    }

    #[test]
    fn https_artifact_requires_signature_pair() {
        let request = KernelArtifactRequest {
            kernel: KernelId::SingBox,
            version: "1.0.0".into(),
            source: "https://example.invalid/kernel".into(),
            sha256: "a".repeat(64),
            signature: String::new(),
            public_key: String::new(),
            catalog_version: String::new(),
            catalog_platform: String::new(),
            catalog_architecture: String::new(),
        };
        assert!(validate_request(&request)
            .unwrap_err()
            .to_string()
            .contains("require an Ed25519 signature"));
    }

    fn hex_bytes(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}
