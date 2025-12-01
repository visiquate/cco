//! Releases API integration for fetching release information
//!
//! Handles querying cco-api.visiquate.com for releases with authentication.
//! Files are downloaded directly from the API endpoint with bearer token authentication.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

use crate::auth;

const RELEASES_API_URL: &str = "https://cco-api.visiquate.com";

// SECURITY: Maximum download size to prevent DoS attacks
const MAX_BINARY_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

/// Release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Version string (e.g., "2025.11.2")
    pub version: String,

    /// Release notes/description
    pub release_notes: String,

    /// Download URL (direct API endpoint)
    pub download_url: String,

    /// Expected SHA256 checksum (mandatory)
    pub checksum: String,

    /// File size in bytes
    pub size: u64,

    /// Archive filename
    pub filename: String,
}

/// Releases API response for latest release
#[derive(Debug, Deserialize)]
struct LatestReleaseResponse {
    version: String,
    release_notes: String,
    platforms: Vec<PlatformAsset>,
}

#[derive(Debug, Deserialize)]
struct PlatformAsset {
    platform: String,
    filename: String,
    size: u64,
    checksum: String,
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
    /// Detect current platform
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

    /// Get platform identifier string
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::DarwinArm64 => "darwin-arm64",
            Platform::DarwinX86_64 => "darwin-x86_64",
            Platform::LinuxX86_64 => "linux-x86_64",
            Platform::LinuxAarch64 => "linux-aarch64",
            Platform::WindowsX86_64 => "windows-x86_64",
        }
    }

    /// Get archive extension for this platform
    pub fn archive_extension(&self) -> &'static str {
        match self {
            Platform::WindowsX86_64 => "zip",
            _ => "tar.gz",
        }
    }

    /// Generate expected asset name for a version
    pub fn asset_name(&self, version: &str) -> String {
        format!(
            "cco-v{}-{}.{}",
            version,
            self.as_str(),
            self.archive_extension()
        )
    }
}

/// Fetch latest release from API
pub async fn fetch_latest_release(channel: &str) -> Result<ReleaseInfo> {
    // Check authentication
    if !auth::is_authenticated()? {
        return Err(anyhow!(
            "Not authenticated. Please run 'cco login' first to access releases."
        ));
    }

    // Get access token (will refresh if needed)
    let access_token = auth::get_access_token().await?;

    // Build API URL
    let url = format!("{}/releases/latest", RELEASES_API_URL);

    // Create HTTP client
    let client = reqwest::Client::builder()
        .user_agent("cco/client")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Fetch latest release
    let response = client
        .get(&url)
        .bearer_auth(&access_token)
        .query(&[("channel", channel)])
        .send()
        .await
        .context("Failed to fetch release information")?;

    // Handle authentication errors
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(anyhow!(
            "Authentication failed. Your session may have expired. Please run 'cco login' again."
        ));
    }

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(anyhow!(
            "Access denied. Your account does not have permission to access releases. Contact your administrator."
        ));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Failed to fetch release (HTTP {}): {}",
            status,
            body
        ));
    }

    let release_data: LatestReleaseResponse = response
        .json()
        .await
        .context("Failed to parse release response")?;

    // Detect current platform
    let platform = Platform::detect()?;

    // Find the appropriate asset for this platform
    let platform_str = platform.as_str();
    let asset = release_data
        .platforms
        .iter()
        .find(|a| a.platform == platform_str)
        .ok_or_else(|| anyhow!("No release asset found for platform: {}", platform_str))?;

    // Validate asset size
    if asset.size > MAX_BINARY_SIZE {
        return Err(anyhow!(
            "SECURITY: Release asset size ({} bytes) exceeds maximum ({} bytes). Possible attack - refusing to download.",
            asset.size,
            MAX_BINARY_SIZE
        ));
    }

    // Construct direct download URL for the API endpoint
    let download_url = format!(
        "{}/download/{}/{}",
        RELEASES_API_URL, release_data.version, platform_str
    );

    Ok(ReleaseInfo {
        version: release_data.version,
        release_notes: release_data.release_notes,
        download_url,
        checksum: asset.checksum.clone(),
        size: asset.size,
        filename: asset.filename.clone(),
    })
}

/// Fetch release by specific version
pub async fn fetch_release_by_version(version: &str) -> Result<ReleaseInfo> {
    // Check authentication
    if !auth::is_authenticated()? {
        return Err(anyhow!(
            "Not authenticated. Please run 'cco login' first to access releases."
        ));
    }

    // Get access token (will refresh if needed)
    let access_token = auth::get_access_token().await?;

    // Build API URL
    let url = format!("{}/releases/{}", RELEASES_API_URL, version);

    // Create HTTP client
    let client = reqwest::Client::builder()
        .user_agent("cco/client")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Fetch specific release
    let response = client
        .get(&url)
        .bearer_auth(&access_token)
        .send()
        .await
        .context("Failed to fetch release information")?;

    // Handle authentication errors
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(anyhow!(
            "Authentication failed. Please run 'cco login' again."
        ));
    }

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(anyhow!("Access denied. Contact your administrator."));
    }

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!("Release {} not found", version));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Failed to fetch release (HTTP {}): {}",
            status,
            body
        ));
    }

    let release_data: LatestReleaseResponse = response
        .json()
        .await
        .context("Failed to parse release response")?;

    // Detect current platform
    let platform = Platform::detect()?;
    let platform_str = platform.as_str();

    // Find the appropriate asset for this platform
    let asset = release_data
        .platforms
        .iter()
        .find(|a| a.platform == platform_str)
        .ok_or_else(|| anyhow!("No release asset found for platform: {}", platform_str))?;

    // Validate asset size
    if asset.size > MAX_BINARY_SIZE {
        return Err(anyhow!(
            "SECURITY: Release asset size exceeds maximum. Refusing to download."
        ));
    }

    // Construct direct download URL for the API endpoint
    let download_url = format!("{}/download/{}/{}", RELEASES_API_URL, version, platform_str);

    Ok(ReleaseInfo {
        version: release_data.version,
        release_notes: release_data.release_notes,
        download_url,
        checksum: asset.checksum.clone(),
        size: asset.size,
        filename: asset.filename.clone(),
    })
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
        assert_eq!(Platform::DarwinArm64.as_str(), "darwin-arm64");
        assert_eq!(Platform::DarwinX86_64.as_str(), "darwin-x86_64");
        assert_eq!(Platform::LinuxX86_64.as_str(), "linux-x86_64");
        assert_eq!(Platform::LinuxAarch64.as_str(), "linux-aarch64");
        assert_eq!(Platform::WindowsX86_64.as_str(), "windows-x86_64");
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
        assert_eq!(
            platform.asset_name("2025.11.2"),
            "cco-v2025.11.2-darwin-arm64.tar.gz"
        );

        let platform = Platform::WindowsX86_64;
        assert_eq!(
            platform.asset_name("2025.11.2"),
            "cco-v2025.11.2-windows-x86_64.zip"
        );
    }
}
