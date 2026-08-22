use anyhow::{anyhow, bail, Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use narya_kernel::KernelId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelCatalogEntry {
    pub kernel: KernelId,
    pub version: String,
    pub platform: String,
    pub architecture: String,
    pub source: String,
    pub sha256: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelCatalogDocument {
    pub schema: u32,
    pub entries: Vec<KernelCatalogEntry>,
    pub signature: String,
    pub public_key: String,
}

impl KernelCatalogDocument {
    pub fn canonical_payload(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&serde_json::json!({
            "schema": self.schema,
            "entries": self.entries,
        }))
        .map_err(Into::into)
    }

    pub fn verify(&self, trusted_key: &str) -> Result<()> {
        if self.schema != 1 {
            bail!("unsupported kernel catalog schema {}", self.schema);
        }
        let trusted = decode_hex(trusted_key, 32, "trusted catalog public key")?;
        let public_key = decode_hex(&self.public_key, 32, "catalog public key")?;
        if trusted != public_key {
            bail!("kernel catalog public key does not match the local trust root");
        }
        let signature = decode_hex(&self.signature, 64, "catalog signature")?;
        let key = VerifyingKey::from_bytes(
            &public_key
                .try_into()
                .map_err(|_| anyhow!("invalid catalog public key length"))?,
        )
        .map_err(|error| anyhow!("invalid catalog public key: {error}"))?;
        key.verify(
            &self.canonical_payload()?,
            &Signature::from_bytes(
                &signature
                    .try_into()
                    .map_err(|_| anyhow!("invalid catalog signature length"))?,
            ),
        )
        .map_err(|error| anyhow!("kernel catalog signature verification failed: {error}"))?;
        let mut identities = HashSet::with_capacity(self.entries.len());
        for entry in &self.entries {
            validate_entry(entry)?;
            let identity = format!(
                "{}\0{}\0{}\0{}",
                entry.kernel, entry.version, entry.platform, entry.architecture
            );
            if !identities.insert(identity) {
                bail!(
                    "kernel catalog contains duplicate artifact identity for {} {} {} {}",
                    entry.kernel,
                    entry.version,
                    entry.platform,
                    entry.architecture
                );
            }
        }
        Ok(())
    }
}

pub async fn fetch_and_store(source: &str, trusted_key: &str) -> Result<KernelCatalogDocument> {
    let bytes = if source.starts_with("https://") {
        let response = reqwest::get(source).await?;
        if !response.status().is_success() {
            bail!(
                "kernel catalog download returned HTTP {}",
                response.status()
            );
        }
        response.bytes().await?.to_vec()
    } else {
        fs::read(source).await?
    };
    if bytes.len() > 16 * 1024 * 1024 {
        bail!("kernel catalog is too large");
    }
    let document: KernelCatalogDocument =
        serde_json::from_slice(&bytes).context("kernel catalog is not valid JSON")?;
    document.verify(trusted_key)?;
    let root = narya_ipc::kernel_catalog_dir();
    fs::create_dir_all(&root).await?;
    let temp = root.join(format!(".narya-{}.tmp", nonce()));
    let target = root.join("catalog.json");
    let trust_temp = root.join(format!(".narya-{}.trust.tmp", nonce()));
    let trust_target = root.join("trusted-public-key");
    let target_backup = root.join(format!(".narya-{}.catalog.backup", nonce()));
    let trust_backup = root.join(format!(".narya-{}.trust.backup", nonce()));
    if let Err(error) = fs::write(&temp, &bytes).await {
        let _ = fs::remove_file(&temp).await;
        return Err(error.into());
    }
    if let Err(error) = fs::write(&trust_temp, trusted_key.trim()).await {
        let _ = fs::remove_file(&temp).await;
        let _ = fs::remove_file(&trust_temp).await;
        return Err(error.into());
    }
    let had_target = target.is_file();
    let had_trust_target = trust_target.is_file();
    if had_target {
        fs::rename(&target, &target_backup).await?;
    }
    if had_trust_target {
        if let Err(error) = fs::rename(&trust_target, &trust_backup).await {
            let _ = fs::rename(&target_backup, &target).await;
            let _ = fs::remove_file(&temp).await;
            let _ = fs::remove_file(&trust_temp).await;
            return Err(error.into());
        }
    }
    if let Err(error) = fs::rename(&temp, &target).await {
        if had_target {
            let _ = fs::rename(&target_backup, &target).await;
        }
        if had_trust_target {
            let _ = fs::rename(&trust_backup, &trust_target).await;
        }
        let _ = fs::remove_file(&temp).await;
        let _ = fs::remove_file(&trust_temp).await;
        return Err(error.into());
    }
    if let Err(error) = fs::rename(&trust_temp, &trust_target).await {
        let _ = fs::remove_file(&target).await;
        if had_target {
            let _ = fs::rename(&target_backup, &target).await;
        }
        if had_trust_target {
            let _ = fs::rename(&trust_backup, &trust_target).await;
        }
        let _ = fs::remove_file(&trust_temp).await;
        return Err(error.into());
    }
    let _ = fs::remove_file(&target_backup).await;
    let _ = fs::remove_file(&trust_backup).await;
    Ok(document)
}

pub async fn load_verified(trusted_key: &str) -> Result<KernelCatalogDocument> {
    let path = narya_ipc::kernel_catalog_dir().join("catalog.json");
    let bytes = fs::read(&path).await.with_context(|| {
        format!(
            "verified kernel catalog is unavailable at {}",
            path.display()
        )
    })?;
    let document: KernelCatalogDocument = serde_json::from_slice(&bytes)?;
    document.verify(trusted_key)?;
    Ok(document)
}

pub fn find_entry(
    document: &KernelCatalogDocument,
    kernel: KernelId,
    version: &str,
    platform: &str,
    architecture: &str,
) -> Result<KernelCatalogEntry> {
    document
        .entries
        .iter()
        .find(|entry| {
            entry.kernel == kernel
                && entry.version == version
                && entry.platform == platform
                && entry.architecture == architecture
        })
        .cloned()
        .ok_or_else(|| anyhow!("kernel catalog has no matching verified artifact"))
}

fn validate_entry(entry: &KernelCatalogEntry) -> Result<()> {
    if entry.version.trim().is_empty()
        || entry.platform.trim().is_empty()
        || entry.architecture.trim().is_empty()
        || !entry.source.starts_with("https://")
    {
        bail!("kernel catalog entry has invalid identity or source");
    }
    if entry.sha256.len() != 64 || !entry.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("kernel catalog entry has an invalid SHA-256");
    }
    decode_hex(&entry.signature, 64, "kernel artifact signature")?;
    decode_hex(&entry.public_key, 32, "kernel artifact public key")?;
    Ok(())
}

fn decode_hex(value: &str, bytes: usize, label: &str) -> Result<Vec<u8>> {
    if value.len() != bytes * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} must be {bytes} bytes encoded as hexadecimal");
    }
    (0..value.len())
        .step_by(2)
        .map(|index| Ok(u8::from_str_radix(&value[index..index + 2], 16)?))
        .collect()
}

fn nonce() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

pub fn catalog_digest(document: &KernelCatalogDocument) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(document.canonical_payload()?)
    ))
}

pub fn default_platform() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    }
}

pub fn default_architecture() -> &'static str {
    std::env::consts::ARCH
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn signed_catalog_matches_trust_root_and_entry() {
        let signing = SigningKey::from_bytes(&[4u8; 32]);
        let mut document = KernelCatalogDocument {
            schema: 1,
            entries: vec![KernelCatalogEntry {
                kernel: KernelId::SingBox,
                version: "1".into(),
                platform: "linux".into(),
                architecture: "x86_64".into(),
                source: "https://example.invalid/sing-box".into(),
                sha256: "ab".repeat(32),
                signature: "cd".repeat(64),
                public_key: hex(signing.verifying_key().to_bytes()),
            }],
            signature: String::new(),
            public_key: hex(signing.verifying_key().to_bytes()),
        };
        document.signature = hex(signing
            .sign(&document.canonical_payload().unwrap())
            .to_bytes());
        document
            .verify(&hex(signing.verifying_key().to_bytes()))
            .unwrap();
        assert!(find_entry(&document, KernelId::SingBox, "1", "linux", "x86_64").is_ok());
    }

    #[test]
    fn catalog_rejects_duplicate_artifact_identity() {
        let signing = SigningKey::from_bytes(&[9u8; 32]);
        let entry = KernelCatalogEntry {
            kernel: KernelId::SingBox,
            version: "1".into(),
            platform: "linux".into(),
            architecture: "x86_64".into(),
            source: "https://example.invalid/one".into(),
            sha256: "ab".repeat(32),
            signature: "cd".repeat(64),
            public_key: hex(signing.verifying_key().to_bytes()),
        };
        let mut document = KernelCatalogDocument {
            schema: 1,
            entries: vec![entry.clone(), entry],
            signature: String::new(),
            public_key: hex(signing.verifying_key().to_bytes()),
        };
        document.signature = hex(signing
            .sign(&document.canonical_payload().unwrap())
            .to_bytes());
        assert!(document
            .verify(&hex(signing.verifying_key().to_bytes()))
            .unwrap_err()
            .to_string()
            .contains("duplicate artifact identity"));
    }

    fn hex(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}
