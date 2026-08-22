use anyhow::{anyhow, bail, Context, Result};
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
}

#[derive(Debug, Clone)]
pub struct InstalledKernel {
    pub kernel: KernelId,
    pub version: String,
    pub binary_path: PathBuf,
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

    let kernel_dir = install_root.join(request.kernel.as_str());
    fs::create_dir_all(&kernel_dir).await?;
    let temp_path = kernel_dir.join(format!(".narya-{}.tmp", std::process::id()));
    let binary_path = kernel_dir.join("current");
    let version_path = kernel_dir.join("version");

    fs::write(&temp_path, &bytes)
        .await
        .with_context(|| format!("failed to write temporary {} artifact", request.kernel))?;
    set_executable(&temp_path).await?;

    // Rename is atomic on the same filesystem. The previous binary remains
    // untouched until checksum verification and the temporary write succeed.
    if let Err(error) = fs::rename(&temp_path, &binary_path).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(error).with_context(|| {
            format!(
                "failed to atomically {} kernel {}",
                if upgrading { "upgrade" } else { "install" },
                request.kernel
            )
        });
    }
    if let Err(error) = fs::write(&version_path, request.version.as_bytes()).await {
        let _ = fs::remove_file(&binary_path).await;
        return Err(error).context("failed to persist installed kernel version");
    }

    Ok(InstalledKernel {
        kernel: request.kernel,
        version: request.version.clone(),
        binary_path,
    })
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
    Ok(())
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "narya-installer-test-{}",
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
            sha256: checksum,
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
        };
        assert!(install(&request, &root.join("kernels"), true)
            .await
            .is_err());
        assert_eq!(fs::read(kernel_dir.join("current")).await.unwrap(), b"old");
        let _ = fs::remove_dir_all(root).await;
    }
}
