use anyhow::{anyhow, bail, Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use narya_rules::RuleSetSource;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;

const MAX_RULESET_SIZE: usize = 128 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct CachedRuleSet {
    pub id: String,
    pub version: String,
    pub path: PathBuf,
    pub sha256: String,
}

pub async fn fetch_and_cache(source: &RuleSetSource) -> Result<CachedRuleSet> {
    source
        .validate()
        .map_err(|error| anyhow!("ruleset {} is invalid: {error}", source.id))?;
    let bytes = if source.source.starts_with("https://") {
        let response = reqwest::get(&source.source)
            .await
            .with_context(|| format!("failed to download ruleset {}", source.id))?;
        if !response.status().is_success() {
            bail!(
                "ruleset {} download returned HTTP {}",
                source.id,
                response.status()
            );
        }
        response.bytes().await?.to_vec()
    } else {
        let path = source
            .source
            .strip_prefix("file://")
            .unwrap_or(&source.source);
        fs::read(path)
            .await
            .with_context(|| format!("failed to read ruleset {}", source.id))?
    };
    verify_bytes(source, &bytes)?;
    let root = narya_ipc::ruleset_cache_dir().join(&source.id);
    fs::create_dir_all(&root).await?;
    let suffix = format!("{}-{}", std::process::id(), cache_nonce());
    let temp = root.join(format!(".narya-{suffix}.tmp"));
    let target = root.join("current");
    let metadata = root.join("metadata.json");
    let metadata_temp = root.join(format!(".narya-{suffix}.metadata.tmp"));
    fs::write(&temp, &bytes).await?;
    fs::write(
        &metadata_temp,
        serde_json::to_vec_pretty(&serde_json::json!({
            "id": source.id,
            "version": source.version,
            "sha256": source.sha256.to_ascii_lowercase(),
        }))?,
    )
    .await?;
    fs::rename(&temp, &target)
        .await
        .with_context(|| format!("failed to atomically replace cached ruleset {}", source.id))?;
    if let Err(error) = fs::rename(&metadata_temp, &metadata).await {
        let _ = fs::remove_file(&target).await;
        return Err(error).context("failed to persist ruleset cache metadata");
    }
    Ok(CachedRuleSet {
        id: source.id.clone(),
        version: source.version.clone(),
        path: target,
        sha256: source.sha256.to_ascii_lowercase(),
    })
}

pub async fn ensure_cached(source: &RuleSetSource) -> Result<CachedRuleSet> {
    source
        .validate()
        .map_err(|error| anyhow!("ruleset {} is invalid: {error}", source.id))?;
    let root = narya_ipc::ruleset_cache_dir().join(&source.id);
    let target = root.join("current");
    let metadata = root.join("metadata.json");
    let bytes = fs::read(&target).await.with_context(|| {
        format!(
            "ruleset {} is not cached; import or update it first",
            source.id
        )
    })?;
    let metadata_json: serde_json::Value = serde_json::from_slice(&fs::read(&metadata).await?)?;
    let cached_version = metadata_json["version"].as_str().unwrap_or_default();
    if cached_version != source.version {
        bail!(
            "ruleset {} cache version mismatch: expected {}, got {}",
            source.id,
            source.version,
            cached_version
        );
    }
    verify_bytes(source, &bytes)?;
    Ok(CachedRuleSet {
        id: source.id.clone(),
        version: source.version.clone(),
        path: target,
        sha256: source.sha256.to_ascii_lowercase(),
    })
}

fn verify_bytes(source: &RuleSetSource, bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_RULESET_SIZE {
        bail!("ruleset {} is too large", source.id);
    }
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(&source.sha256) {
        bail!(
            "ruleset {} checksum mismatch: expected {}, got {}",
            source.id,
            source.sha256,
            actual
        );
    }
    if source.signature.trim().is_empty() {
        return Ok(());
    }
    let public_key = decode_hex(&source.public_key, 32, "ruleset public key")?;
    let signature = decode_hex(&source.signature, 64, "ruleset signature")?;
    let key = VerifyingKey::from_bytes(
        &public_key
            .try_into()
            .map_err(|_| anyhow!("invalid ruleset public key length"))?,
    )
    .map_err(|error| anyhow!("invalid ruleset public key: {error}"))?;
    let signature = Signature::from_bytes(
        &signature
            .try_into()
            .map_err(|_| anyhow!("invalid ruleset signature length"))?,
    );
    key.verify(bytes, &signature).map_err(|error| {
        anyhow!(
            "ruleset {} signature verification failed: {error}",
            source.id
        )
    })
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

fn cache_nonce() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[tokio::test]
    async fn local_signed_ruleset_is_cached_and_reverified() {
        let root = std::env::temp_dir().join(format!("narya-ruleset-test-{}", cache_nonce()));
        fs::create_dir_all(&root).await.unwrap();
        let source_path = root.join("rules.srs");
        let bytes = b"ruleset";
        fs::write(&source_path, bytes).await.unwrap();
        let signing = SigningKey::from_bytes(&[9u8; 32]);
        let source = RuleSetSource {
            id: format!("test-{}", cache_nonce()),
            source: source_path.to_string_lossy().into_owned(),
            version: "1".into(),
            sha256: format!("{:x}", Sha256::digest(bytes)),
            enabled: true,
            signature: hex(signing.sign(bytes).to_bytes()),
            public_key: hex(signing.verifying_key().to_bytes()),
        };
        let cached = fetch_and_cache(&source).await.unwrap();
        assert_eq!(fs::read(&cached.path).await.unwrap(), bytes);
        let verified = ensure_cached(&source).await.unwrap();
        assert_eq!(verified.sha256, source.sha256);
        let _ = fs::remove_dir_all(root).await;
        let _ = fs::remove_dir_all(narya_ipc::ruleset_cache_dir().join(source.id)).await;
    }

    fn hex(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}
