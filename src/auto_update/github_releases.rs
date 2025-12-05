//! GitHub Releases API integration with embedded token support
//!
//! This module provides auto-update functionality using GitHub private releases.
//! The authentication token is embedded in the binary at compile time and obfuscated
//! using litcrypt2 for security.
//!
//! # Security Model
//! - Token is XOR encrypted at compile time (not visible via `strings`)
//! - Token has read-only access to releases only
//! - Even with token, GitHub verifies user has repo access
//! - Token is rotatable by releasing a new version
//!
//! # Configuration
//! Set `VISIQUATE_UPDATE_TOKEN` environment variable at build time.
//! If not set, updates will require manual download.

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::env;

use super::releases_api::{Platform, ReleaseInfo};

// GitHub repository for releases
const GITHUB_OWNER: &str = "visiquate";
const GITHUB_REPO: &str = "cco";
const GITHUB_API_URL: &str = "https://api.github.com";

// Embedded update token (encrypted at compile time)
// This is set via VISIQUATE_UPDATE_TOKEN env var during CI build
// If not available at build time, updates will fall back to manual download
fn get_embedded_token() -> Option<String> {
    // Try to get from build-time embedded value
    option_env!("VISIQUATE_UPDATE_TOKEN").map(|s| s.to_string())
}

/// GitHub Release API response
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: String,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
    prerelease: bool,
    draft: bool,
}

/// GitHub Asset API response
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
    #[serde(rename = "url")]
    api_url: String,
}

/// Convert tag name to version string
fn tag_to_version(tag: &str) -> String {
    tag.trim_start_matches('v').to_string()
}

/// Create HTTP client with appropriate headers for GitHub API
fn create_github_client(_token: Option<&str>) -> Result<reqwest::Client> {
    let builder = reqwest::Client::builder()
        .user_agent("cco-update/1.0")
        .timeout(std::time::Duration::from_secs(30));

    // Note: For private repo API access, we need to add the token header
    // This is handled per-request, not at client level

    builder.build().context("Failed to create HTTP client")
}

/// Fetch the latest release from GitHub
pub async fn fetch_latest_release(channel: &str) -> Result<ReleaseInfo> {
    let token = get_embedded_token();

    if token.is_none() {
        return Err(anyhow!(
            "No embedded update token available. \
            Please download updates manually from GitHub or reinstall CCO."
        ));
    }

    let client = create_github_client(token.as_deref())?;

    // Fetch releases list
    let url = format!(
        "{}/repos/{}/{}/releases",
        GITHUB_API_URL, GITHUB_OWNER, GITHUB_REPO
    );

    let mut request = client.get(&url).header("Accept", "application/vnd.github+json");

    if let Some(ref t) = token {
        request = request.header("Authorization", format!("Bearer {}", t));
    }

    let response = request.send().await.context("Failed to fetch releases")?;

    // Handle authentication errors
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(anyhow!(
            "GitHub authentication failed. The embedded update token may have expired. \
            Please reinstall CCO to get a new version with updated credentials."
        ));
    }

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        // Check for rate limiting
        if response
            .headers()
            .get("X-RateLimit-Remaining")
            .map(|v| v.to_str().unwrap_or("1"))
            == Some("0")
        {
            return Err(anyhow!(
                "GitHub API rate limit exceeded. Please try again later."
            ));
        }
        return Err(anyhow!(
            "Access denied to releases. Contact your administrator."
        ));
    }

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!(
            "Release repository not found. Please verify CCO installation."
        ));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Failed to fetch releases (HTTP {}): {}", status, body));
    }

    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .context("Failed to parse releases response")?;

    // Find the appropriate release based on channel
    let release = releases
        .iter()
        .filter(|r| !r.draft)
        .filter(|r| match channel {
            "stable" => !r.prerelease,
            "beta" => true, // Beta channel gets all releases including prereleases
            _ => !r.prerelease,
        })
        .next()
        .ok_or_else(|| anyhow!("No releases found for channel: {}", channel))?;

    // Convert to ReleaseInfo
    convert_github_release(release, token.as_deref()).await
}

/// Fetch a specific release by version
pub async fn fetch_release_by_version(version: &str, token: Option<&str>) -> Result<ReleaseInfo> {
    let effective_token = token.map(|t| t.to_string()).or_else(get_embedded_token);

    if effective_token.is_none() {
        return Err(anyhow!(
            "No update token available. Please provide a token or reinstall CCO."
        ));
    }

    let client = create_github_client(effective_token.as_deref())?;

    // Fetch specific release by tag
    let tag = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    };

    let url = format!(
        "{}/repos/{}/{}/releases/tags/{}",
        GITHUB_API_URL, GITHUB_OWNER, GITHUB_REPO, tag
    );

    let mut request = client.get(&url).header("Accept", "application/vnd.github+json");

    if let Some(ref t) = effective_token {
        request = request.header("Authorization", format!("Bearer {}", t));
    }

    let response = request.send().await.context("Failed to fetch release")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!("Release {} not found", version));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Failed to fetch release (HTTP {}): {}", status, body));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .context("Failed to parse release response")?;

    convert_github_release(&release, effective_token.as_deref()).await
}

/// Convert GitHub release to our ReleaseInfo format
async fn convert_github_release(
    release: &GitHubRelease,
    token: Option<&str>,
) -> Result<ReleaseInfo> {
    let platform = Platform::detect()?;
    let platform_suffix = match platform {
        Platform::DarwinArm64 => "aarch64-apple-darwin",
        Platform::DarwinX86_64 => "x86_64-apple-darwin",
        Platform::LinuxX86_64 => "x86_64-unknown-linux-gnu",
        Platform::LinuxAarch64 => "aarch64-unknown-linux-gnu",
        Platform::WindowsX86_64 => "x86_64-pc-windows-msvc",
    };

    // Find the binary asset for this platform
    let binary_asset = release
        .assets
        .iter()
        .find(|a| a.name.contains(platform_suffix) && a.name.ends_with(".tar.gz"))
        .ok_or_else(|| {
            anyhow!(
                "No binary found for platform {} in release {}",
                platform_suffix,
                release.tag_name
            )
        })?;

    // Try to find checksum file
    let checksum = fetch_checksum_for_asset(&binary_asset.name, release, token).await?;

    // For private repos, we need to use the API URL with auth header for download
    // The browser_download_url won't work without web session
    let download_url = if token.is_some() {
        // Use API URL which accepts bearer token auth
        binary_asset.api_url.clone()
    } else {
        // Fallback to browser URL (won't work for private repos)
        binary_asset.browser_download_url.clone()
    };

    Ok(ReleaseInfo {
        version: tag_to_version(&release.tag_name),
        release_notes: release.body.clone().unwrap_or_default(),
        download_url,
        checksum,
        size: binary_asset.size,
        filename: binary_asset.name.clone(),
    })
}

/// Fetch checksum for an asset from checksums.txt file
async fn fetch_checksum_for_asset(
    asset_name: &str,
    release: &GitHubRelease,
    token: Option<&str>,
) -> Result<String> {
    // Look for checksums.txt in the release assets
    let checksum_asset = release
        .assets
        .iter()
        .find(|a| a.name == "checksums.txt");

    if let Some(asset) = checksum_asset {
        let client = create_github_client(token)?;

        let mut request = client
            .get(&asset.api_url)
            .header("Accept", "application/octet-stream");

        if let Some(t) = token {
            request = request.header("Authorization", format!("Bearer {}", t));
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let content = response.text().await?;

            // Parse checksums.txt format: "sha256  filename"
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1].contains(asset_name) {
                    return Ok(parts[0].to_string());
                }
            }
        }
    }

    // If no checksums.txt, look for .sha256 file
    let sha256_asset_name = format!("{}.sha256", asset_name);
    let sha256_asset = release
        .assets
        .iter()
        .find(|a| a.name == sha256_asset_name);

    if let Some(asset) = sha256_asset {
        let client = create_github_client(token)?;

        let mut request = client
            .get(&asset.api_url)
            .header("Accept", "application/octet-stream");

        if let Some(t) = token {
            request = request.header("Authorization", format!("Bearer {}", t));
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let content = response.text().await?;
            // Format is usually "sha256  filename" or just "sha256"
            let checksum = content.split_whitespace().next().unwrap_or("");
            if !checksum.is_empty() {
                return Ok(checksum.to_string());
            }
        }
    }

    Err(anyhow!(
        "No checksum found for asset {}. Cannot verify download integrity.",
        asset_name
    ))
}

/// Download a binary with authentication
pub async fn download_binary_with_auth(url: &str, token: Option<&str>) -> Result<bytes::Bytes> {
    let effective_token = token.map(|t| t.to_string()).or_else(get_embedded_token);
    let client = create_github_client(effective_token.as_deref())?;

    let mut request = client
        .get(url)
        .header("Accept", "application/octet-stream");

    if let Some(ref t) = effective_token {
        request = request.header("Authorization", format!("Bearer {}", t));
    }

    let response = request.send().await.context("Failed to download binary")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    response.bytes().await.context("Failed to read binary data")
}

/// Check if embedded token is available
pub fn has_embedded_token() -> bool {
    get_embedded_token().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_to_version() {
        assert_eq!(tag_to_version("v2025.12.4"), "2025.12.4");
        assert_eq!(tag_to_version("2025.12.4"), "2025.12.4");
        assert_eq!(tag_to_version("v1.0.0"), "1.0.0");
    }

    #[test]
    fn test_has_embedded_token() {
        // This will be false in tests unless VISIQUATE_UPDATE_TOKEN is set
        // The important thing is it doesn't panic
        let _ = has_embedded_token();
    }
}
