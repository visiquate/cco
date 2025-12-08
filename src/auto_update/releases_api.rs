//! GitHub Releases integration for stable-only updates
//!
//! Fetches release metadata from the public GitHub releases API, selects the
//! platform-specific asset, downloads `checksums.txt` to obtain the expected
//! SHA256, and returns a `ReleaseInfo` used by the updater.
//!
//! Notes:
//! - Stable channel only (beta/prerelease not supported).
//! - Optional embedded PAT (XOR-obfuscated at build time) may be used for
//!   metadata fetch if available; downloads remain unauthenticated.
//! - Checksum verification is mandatory; missing or mismatched entries fail.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

// GitHub repository for releases
const GITHUB_OWNER: &str = "visiquate";
const GITHUB_REPO: &str = "cco";
const GITHUB_API_URL: &str = "https://api.github.com";

// Maximum download size to prevent DoS attacks
const MAX_BINARY_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

// XOR key must match build.rs
const XOR_KEY: u8 = 0xA7;

/// Release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Version string (e.g., "2025.12.28+<commit>")
    pub version: String,

    /// Release notes/description
    pub release_notes: String,

    /// Download URL (GitHub asset)
    pub download_url: String,

    /// Expected SHA256 checksum (mandatory)
    pub checksum: String,

    /// File size in bytes
    pub size: u64,

    /// Archive filename
    pub filename: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: Option<String>,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
    #[allow(dead_code)]
    prerelease: bool,
    #[allow(dead_code)]
    draft: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Platform identifier
#[derive(Debug, Clone, Copy)]
pub enum Platform {
    DarwinArm64,
    DarwinX86_64,
    LinuxX86_64,
    LinuxAarch64,
    WindowsX86_64,
}

impl Platform {
    /// Detect current platform (returns Rust target triple form)
    pub fn detect() -> Result<Self> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        let platform = match (os, arch) {
            ("macos", "aarch64") => Platform::DarwinArm64,
            ("macos", "x86_64") => Platform::DarwinX86_64,
            ("linux", "x86_64") => Platform::LinuxX86_64,
            ("linux", "aarch64") => Platform::LinuxAarch64,
            ("windows", "x86_64") => Platform::WindowsX86_64,
            _ => return Err(anyhow!("Unsupported platform: {}-{}", os, arch)),
        };

        Ok(platform)
    }

    /// Target triple string used in asset names
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::DarwinArm64 => "aarch64-apple-darwin",
            Platform::DarwinX86_64 => "x86_64-apple-darwin",
            Platform::LinuxX86_64 => "x86_64-unknown-linux-gnu",
            Platform::LinuxAarch64 => "aarch64-unknown-linux-gnu",
            Platform::WindowsX86_64 => "x86_64-pc-windows-msvc",
        }
    }

    /// Archive extension for this platform
    pub fn archive_extension(&self) -> &'static str {
        match self {
            Platform::WindowsX86_64 => "zip",
            _ => "tar.gz",
        }
    }

    /// Expected asset filename (matches current release naming)
    pub fn asset_name(&self) -> String {
        format!("cco-{}.{}", self.as_str(), self.archive_extension())
    }
}

/// Fetch latest stable release from GitHub
pub async fn fetch_latest_release(channel: &str) -> Result<ReleaseInfo> {
    if channel != "stable" {
        return Err(anyhow!("Only the stable channel is supported"));
    }
    fetch_release(None).await
}

/// Fetch a specific release by version/tag (expects raw version like 2025.12.28)
pub async fn fetch_release_by_version(version: &str) -> Result<ReleaseInfo> {
    let tag = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    };
    fetch_release(Some(&tag)).await
}

/// Fetch release metadata and build ReleaseInfo
async fn fetch_release(tag: Option<&str>) -> Result<ReleaseInfo> {
    let token = get_embedded_token();
    let client = reqwest::Client::builder()
        .user_agent("cco/client")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let url = match tag {
        Some(tag) => format!(
            "{}/repos/{}/{}/releases/tags/{}",
            GITHUB_API_URL, GITHUB_OWNER, GITHUB_REPO, tag
        ),
        None => format!(
            "{}/repos/{}/{}/releases/latest",
            GITHUB_API_URL, GITHUB_OWNER, GITHUB_REPO
        ),
    };

    let mut request = client
        .get(&url)
        .header("Accept", "application/vnd.github+json");

    if let Some(ref t) = token {
        request = request.header("Authorization", format!("Bearer {}", t));
    }

    let response = request
        .send()
        .await
        .context("Failed to fetch release information from GitHub")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!("Release not found"));
    }

    if !response.status().is_success() {
        return Err(anyhow!("GitHub API returned status: {}", response.status()));
    }

    let release: GitHubRelease = response.json().await?;

    // Extract version from tag (strip leading 'v')
    let version = release.tag_name.trim_start_matches('v').to_string();

    // Detect platform and expected asset name
    let platform = Platform::detect()?;
    let asset_name = platform.asset_name();

    // Find the platform asset
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            let available: Vec<_> = release.assets.iter().map(|a| a.name.as_str()).collect();
            anyhow!(
                "No release asset found for platform: {}. Expected: {}. Available: {:?}",
                platform.as_str(),
                asset_name,
                available
            )
        })?;

    if asset.size > MAX_BINARY_SIZE {
        return Err(anyhow!(
            "SECURITY: Release asset size ({} bytes) exceeds maximum ({} bytes)",
            asset.size,
            MAX_BINARY_SIZE
        ));
    }

    // Locate and parse checksums file
    let checksum_asset = release
        .assets
        .iter()
        .find(|a| {
            matches!(
                a.name.as_str(),
                "checksums.txt" | "checksums.sha256" | "SHA256SUMS"
            )
        })
        .ok_or_else(|| anyhow!("No checksums file found in release"))?;

    let checksum =
        fetch_checksum_for_asset(&client, &checksum_asset.browser_download_url, &asset.name)
            .await
            .context("Failed to obtain checksum")?;

    Ok(ReleaseInfo {
        version,
        release_notes: release.body.unwrap_or_default(),
        download_url: asset.browser_download_url.clone(),
        checksum,
        size: asset.size,
        filename: asset.name.clone(),
    })
}

/// Fetch and parse checksum for the given asset name
async fn fetch_checksum_for_asset(
    client: &reqwest::Client,
    checksum_url: &str,
    asset_name: &str,
) -> Result<String> {
    let response = client
        .get(checksum_url)
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .context("Failed to download checksums file")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to download checksums file (status: {})",
            response.status()
        ));
    }

    let content = response.text().await?;

    // Expected format: "<checksum>  <filename>" (GNU sha256sum style)
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == asset_name {
            return Ok(parts[0].to_string());
        }
    }

    Err(anyhow!(
        "No checksum entry found for asset {} in checksums file",
        asset_name
    ))
}

/// Get embedded update token (XOR-obfuscated at build time)
fn get_embedded_token() -> Option<String> {
    let obfuscated = include_bytes!(concat!(env!("OUT_DIR"), "/update_token.bin"));

    if obfuscated.is_empty() {
        return None;
    }

    let decrypted: Vec<u8> = obfuscated.iter().map(|&b| b ^ XOR_KEY).collect();
    String::from_utf8(decrypted).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        assert!(platform.is_ok(), "Should detect current platform");
    }

    #[test]
    fn test_platform_strings() {
        assert_eq!(Platform::DarwinArm64.as_str(), "aarch64-apple-darwin");
        assert_eq!(Platform::DarwinX86_64.as_str(), "x86_64-apple-darwin");
        assert_eq!(Platform::LinuxX86_64.as_str(), "x86_64-unknown-linux-gnu");
        assert_eq!(Platform::LinuxAarch64.as_str(), "aarch64-unknown-linux-gnu");
        assert_eq!(Platform::WindowsX86_64.as_str(), "x86_64-pc-windows-msvc");
    }

    #[test]
    fn test_archive_extensions() {
        assert_eq!(Platform::DarwinArm64.archive_extension(), "tar.gz");
        assert_eq!(Platform::LinuxX86_64.archive_extension(), "tar.gz");
        assert_eq!(Platform::WindowsX86_64.archive_extension(), "zip");
    }

    #[test]
    fn test_asset_name_generation() {
        let platform = Platform::DarwinArm64;
        assert_eq!(platform.asset_name(), "cco-aarch64-apple-darwin.tar.gz");

        let platform = Platform::WindowsX86_64;
        assert_eq!(platform.asset_name(), "cco-x86_64-pc-windows-msvc.zip");
    }
}
