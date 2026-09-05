use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use narya_kernel::KernelId;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::time::Duration;

const MAX_DOWNLOAD_SIZE: usize = 256 * 1024 * 1024;
const MAX_BINARY_SIZE: usize = 128 * 1024 * 1024;
const MAX_DOWNLOAD_ATTEMPTS: usize = 3;

pub struct OfficialKernelArtifact {
    pub kernel: KernelId,
    pub version: String,
    pub source: String,
    pub sha256: String,
    pub binary: Vec<u8>,
}

#[derive(Clone, Copy)]
enum ArchiveKind {
    Gzip,
    TarGzip,
    Zip,
}

struct ReleaseSpec {
    owner: &'static str,
    repository: &'static str,
    asset_name: String,
    binary_name: &'static str,
    archive: ArchiveKind,
    has_digest: bool,
}

pub async fn download_latest(kernel: KernelId) -> Result<OfficialKernelArtifact> {
    let client = reqwest::Client::builder()
        .user_agent(format!("narya/{}", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(120))
        .redirect(reqwest::redirect::Policy::limited(8))
        .build()?;
    let (tag, version) = latest_release(&client, kernel).await?;
    let spec = release_spec(kernel, &version)?;
    let source = format!(
        "https://github.com/{}/{}/releases/download/{}/{}",
        spec.owner, spec.repository, tag, spec.asset_name
    );
    let archive = download_official_asset(&client, &source).await?;
    if spec.has_digest {
        let digest_source = format!("{source}.dgst");
        let digest = download_official_asset(&client, &digest_source).await?;
        verify_upstream_digest(&archive, &digest)?;
    }
    let binary = extract_binary(&archive, spec.archive, spec.binary_name)?;
    if binary.is_empty() {
        bail!("official {} archive contained an empty executable", kernel);
    }
    let sha256 = format!("{:x}", Sha256::digest(&binary));
    Ok(OfficialKernelArtifact {
        kernel,
        version,
        source,
        sha256,
        binary,
    })
}

async fn latest_release(client: &reqwest::Client, kernel: KernelId) -> Result<(String, String)> {
    let (owner, repository) = repository(kernel);
    let source = format!("https://github.com/{owner}/{repository}/releases/latest");
    let response =
        client.get(&source).send().await.map_err(|error| {
            anyhow!("failed to resolve latest official {kernel} release: {error}")
        })?;
    if !response.status().is_success() {
        bail!(
            "official {kernel} release lookup returned HTTP {}",
            response.status()
        );
    }
    validate_github_release_url(response.url(), owner, repository)?;
    let tag = response
        .url()
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|tag| !tag.is_empty() && *tag != "latest")
        .ok_or_else(|| anyhow!("official {kernel} release did not resolve to a version tag"))?
        .to_string();
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    if version.is_empty()
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        bail!("official {kernel} release returned an invalid version tag");
    }
    Ok((tag, version))
}

async fn download_official_asset(client: &reqwest::Client, source: &str) -> Result<Vec<u8>> {
    'attempt: for attempt in 1..=MAX_DOWNLOAD_ATTEMPTS {
        let response = match client.get(source).send().await {
            Ok(response) => response,
            Err(error)
                if attempt < MAX_DOWNLOAD_ATTEMPTS
                    && (error.is_connect() || error.is_timeout() || error.is_request()) =>
            {
                tokio::time::sleep(Duration::from_millis(250 * attempt as u64)).await;
                continue;
            }
            Err(error) => {
                bail!("failed to download official asset {source}: {error}");
            }
        };
        if (response.status().is_server_error() || response.status().as_u16() == 429)
            && attempt < MAX_DOWNLOAD_ATTEMPTS
        {
            tokio::time::sleep(Duration::from_millis(250 * attempt as u64)).await;
            continue;
        }
        if !response.status().is_success() {
            bail!(
                "official asset download returned HTTP {} for {source}",
                response.status()
            );
        }
        validate_release_asset_url(response.url())?;
        if response
            .content_length()
            .is_some_and(|length| length > MAX_DOWNLOAD_SIZE as u64)
        {
            bail!("official asset exceeds the download size limit");
        }
        let mut bytes = Vec::new();
        let mut response = response;
        loop {
            let chunk = match response.chunk().await {
                Ok(chunk) => chunk,
                Err(error)
                    if attempt < MAX_DOWNLOAD_ATTEMPTS
                        && (error.is_connect() || error.is_timeout() || error.is_request()) =>
                {
                    tokio::time::sleep(Duration::from_millis(250 * attempt as u64)).await;
                    continue 'attempt;
                }
                Err(error) => {
                    bail!("failed while downloading official asset {source}: {error}");
                }
            };
            let Some(chunk) = chunk else {
                break;
            };
            if bytes.len().saturating_add(chunk.len()) > MAX_DOWNLOAD_SIZE {
                bail!("official asset exceeds the download size limit");
            }
            bytes.extend_from_slice(&chunk);
        }
        return Ok(bytes);
    }
    unreachable!("official asset download attempts are exhausted")
}

fn validate_github_release_url(url: &reqwest::Url, owner: &str, repository: &str) -> Result<()> {
    if url.scheme() != "https" || url.host_str() != Some("github.com") {
        bail!("official release lookup redirected outside github.com");
    }
    let expected = format!("/{owner}/{repository}/releases/tag/");
    if !url.path().starts_with(&expected) {
        bail!("official release lookup redirected outside the configured repository");
    }
    Ok(())
}

fn validate_release_asset_url(url: &reqwest::Url) -> Result<()> {
    let allowed = matches!(
        url.host_str(),
        Some("github.com") | Some("release-assets.githubusercontent.com")
    );
    if url.scheme() != "https" || !allowed {
        bail!("official asset redirected outside GitHub release storage");
    }
    Ok(())
}

fn repository(kernel: KernelId) -> (&'static str, &'static str) {
    match kernel {
        KernelId::SingBox => ("SagerNet", "sing-box"),
        KernelId::Mihomo => ("MetaCubeX", "mihomo"),
        KernelId::Xray => ("XTLS", "Xray-core"),
        KernelId::V2Ray => ("v2fly", "v2ray-core"),
    }
}

fn release_spec(kernel: KernelId, version: &str) -> Result<ReleaseSpec> {
    let architecture = linux_architecture()?;
    Ok(match kernel {
        KernelId::SingBox => ReleaseSpec {
            owner: "SagerNet",
            repository: "sing-box",
            asset_name: format!("sing-box-{version}-linux-{architecture}.tar.gz"),
            binary_name: "sing-box",
            archive: ArchiveKind::TarGzip,
            has_digest: false,
        },
        KernelId::Mihomo => ReleaseSpec {
            owner: "MetaCubeX",
            repository: "mihomo",
            asset_name: format!("mihomo-linux-{architecture}-v{version}.gz"),
            binary_name: "mihomo",
            archive: ArchiveKind::Gzip,
            has_digest: false,
        },
        KernelId::Xray => ReleaseSpec {
            owner: "XTLS",
            repository: "Xray-core",
            asset_name: format!("Xray-linux-{}.zip", v2ray_architecture()?),
            binary_name: "xray",
            archive: ArchiveKind::Zip,
            has_digest: true,
        },
        KernelId::V2Ray => ReleaseSpec {
            owner: "v2fly",
            repository: "v2ray-core",
            asset_name: format!("v2ray-linux-{}.zip", v2ray_architecture()?),
            binary_name: "v2ray",
            archive: ArchiveKind::Zip,
            has_digest: true,
        },
    })
}

fn linux_architecture() -> Result<&'static str> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("amd64"),
        "aarch64" => Ok("arm64"),
        architecture => bail!("official kernel installation does not support Linux {architecture}"),
    }
}

fn v2ray_architecture() -> Result<&'static str> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("64"),
        "aarch64" => Ok("arm64-v8a"),
        architecture => bail!("official kernel installation does not support Linux {architecture}"),
    }
}

fn verify_upstream_digest(bytes: &[u8], digest_document: &[u8]) -> Result<()> {
    let document = std::str::from_utf8(digest_document)
        .context("official digest document is not valid UTF-8")?;
    let expected = document
        .lines()
        .find_map(|line| line.strip_prefix("SHA2-256="))
        .map(str::trim)
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| anyhow!("official digest document has no valid SHA2-256 value"))?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        bail!("official archive SHA-256 verification failed");
    }
    Ok(())
}

fn extract_binary(bytes: &[u8], archive: ArchiveKind, binary_name: &str) -> Result<Vec<u8>> {
    match archive {
        ArchiveKind::Gzip => read_limited(GzDecoder::new(bytes), binary_name),
        ArchiveKind::TarGzip => {
            let decoder = GzDecoder::new(bytes);
            let mut archive = tar::Archive::new(decoder);
            for entry in archive.entries().context("invalid official tar archive")? {
                let mut entry = entry?;
                if !entry.header().entry_type().is_file() {
                    continue;
                }
                let matches = entry
                    .path()?
                    .file_name()
                    .is_some_and(|name| name == binary_name);
                if matches {
                    return read_limited(&mut entry, binary_name);
                }
            }
            bail!("official archive does not contain executable {binary_name}")
        }
        ArchiveKind::Zip => {
            let mut archive =
                zip::ZipArchive::new(Cursor::new(bytes)).context("invalid official zip archive")?;
            for index in 0..archive.len() {
                let mut entry = archive.by_index(index)?;
                let matches = entry
                    .enclosed_name()
                    .and_then(|path| path.file_name().map(|name| name == binary_name))
                    .unwrap_or(false);
                if matches && entry.is_file() {
                    return read_limited(&mut entry, binary_name);
                }
            }
            bail!("official archive does not contain executable {binary_name}")
        }
    }
}

fn read_limited(mut reader: impl Read, label: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(MAX_BINARY_SIZE as u64 + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to extract official executable {label}"))?;
    if bytes.len() > MAX_BINARY_SIZE {
        bail!("official executable exceeds the extracted size limit");
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    #[test]
    fn official_specs_are_fixed_to_expected_repositories() {
        let spec = release_spec(KernelId::SingBox, "1.14.0").unwrap();
        assert_eq!(spec.owner, "SagerNet");
        assert_eq!(spec.repository, "sing-box");
        assert!(spec.asset_name.starts_with("sing-box-1.14.0-linux-"));
        let spec = release_spec(KernelId::Mihomo, "1.19.30").unwrap();
        assert_eq!(spec.owner, "MetaCubeX");
        assert!(spec.asset_name.ends_with("-v1.19.30.gz"));
    }

    #[test]
    fn gzip_extraction_returns_only_the_binary() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"official-binary").unwrap();
        let archive = encoder.finish().unwrap();
        assert_eq!(
            extract_binary(&archive, ArchiveKind::Gzip, "mihomo").unwrap(),
            b"official-binary"
        );
    }

    #[test]
    fn digest_document_must_match_archive() {
        let bytes = b"archive";
        let digest = format!("SHA2-256= {:x}\n", Sha256::digest(bytes));
        verify_upstream_digest(bytes, digest.as_bytes()).unwrap();
        assert!(verify_upstream_digest(b"tampered", digest.as_bytes()).is_err());
    }
}
