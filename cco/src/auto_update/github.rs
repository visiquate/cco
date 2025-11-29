//! GitHub API integration for fetching release information
//!
//! Handles querying GitHub releases API without authentication
//! and extracting platform-specific binary information.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

const GITHUB_REPO: &str = "brentley/cco-releases";
const GITHUB_API_URL: &str = "https://api.github.com/repos";

// SECURITY: Expected repository owner for verification (prevents account takeover attacks)
const EXPECTED_REPO_OWNER: &str = "brentley";

/// Release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Version string (e.g., "2025.11.2")
    pub version: String,

    /// Release notes/description
    pub release_notes: String,

    /// Download URL for the current platform
    pub download_url: String,

    /// Expected SHA256 checksum (if available)
    pub checksum: Option<String>,

    /// File size in bytes
    pub size: u64,

    /// Archive filename
    pub filename: String,
}

/// GitHub Release API response
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: String,
    body: String,
    assets: Vec<GitHubAsset>,
    #[allow(dead_code)]
    published_at: String,
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

/// Verify repository is owned by legitimate account/organization
/// SECURITY: Prevents supply chain attacks via compromised GitHub accounts
async fn verify_repository_ownership() -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("cco/client") // LOW FIX #11: Generic user-agent for privacy
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    #[derive(Deserialize)]
    struct RepoInfo {
        owner: Owner,
    }

    #[derive(Deserialize)]
    struct Owner {
        login: String,
    }

    let url = format!("{}/{}", GITHUB_API_URL, GITHUB_REPO);
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("Failed to verify repository ownership")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to verify repository ownership (HTTP {})",
            response.status()
        ));
    }

    let repo_info: RepoInfo = response.json().await?;

    if repo_info.owner.login != EXPECTED_REPO_OWNER {
        return Err(anyhow!(
            "SECURITY ALERT: Repository owner mismatch! Expected '{}', found '{}'. Possible account takeover - refusing to download.",
            EXPECTED_REPO_OWNER,
            repo_info.owner.login
        ));
    }

    tracing::info!("Repository ownership verified: {}", repo_info.owner.login);
    Ok(())
}

/// Validate release tag format to prevent injection attacks
/// SECURITY: Prevents path traversal and command injection via malicious tags
fn validate_release_tag(tag: &str) -> Result<String> {
    // Must match version format: vYYYY.MM.N or vX.Y.Z
    let date_version_pattern = r"^v(\d{4}\.\d{1,2}\.\d+)$";
    let semver_pattern = r"^v(\d+\.\d+\.\d+(-[a-z0-9]+)?)$";

    let date_re =
        regex::Regex::new(date_version_pattern).context("Invalid date version pattern")?;
    let semver_re = regex::Regex::new(semver_pattern).context("Invalid semver pattern")?;

    if !date_re.is_match(tag) && !semver_re.is_match(tag) {
        return Err(anyhow!(
            "SECURITY: Invalid release tag format '{}' (expected vYYYY.MM.N or vX.Y.Z)",
            tag
        ));
    }

    // No path traversal or special characters
    if tag.contains("..") || tag.contains('/') || tag.contains('\\') || tag.contains(';') {
        return Err(anyhow!(
            "SECURITY: Release tag contains invalid characters: {}",
            tag
        ));
    }

    Ok(tag.to_string())
}

/// Validate asset name to prevent path traversal attacks
/// SECURITY: Ensures asset names don't escape temporary directory
fn validate_asset_name(name: &str) -> Result<()> {
    // No path traversal characters
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(anyhow!(
            "SECURITY: Asset name contains path traversal characters: {}",
            name
        ));
    }

    // Must match expected pattern for CCO release assets
    let valid_patterns = [
        r"^cco-v[\d.]+-[a-z0-9_-]+\.(tar\.gz|zip)$", // Binary archives
        r"^checksums\.sha256$",                      // Checksum file
        r"^SHA256SUMS$",                             // Alternative checksum name
    ];

    let is_valid = valid_patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(name))
            .unwrap_or(false)
    });

    if !is_valid {
        return Err(anyhow!(
            "SECURITY: Asset name does not match expected format: {}",
            name
        ));
    }

    Ok(())
}

/// Validate download URL to prevent SSRF attacks
/// SECURITY: Ensures downloads only come from GitHub domains
fn validate_download_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url).context("Invalid download URL format")?;

    // MUST be HTTPS
    if parsed.scheme() != "https" {
        return Err(anyhow!(
            "SECURITY: Download URL must use HTTPS, got: {}",
            parsed.scheme()
        ));
    }

    // MUST be GitHub domain
    if let Some(host) = parsed.host_str() {
        let allowed_hosts = ["github.com", "githubusercontent.com", "github.io"];
        let is_github = allowed_hosts
            .iter()
            .any(|&allowed| host == allowed || host.ends_with(&format!(".{}", allowed)));

        if !is_github {
            return Err(anyhow!(
                "SECURITY: Download URL must be from GitHub domains, got: {}",
                host
            ));
        }
    } else {
        return Err(anyhow!("SECURITY: Download URL has no host component"));
    }

    tracing::debug!("Download URL validated: {}", url);
    Ok(())
}

/// Fetch latest release from GitHub
pub async fn fetch_latest_release(channel: &str) -> Result<ReleaseInfo> {
    // CRITICAL FIX #2: Verify repository ownership FIRST
    verify_repository_ownership().await?;

    let url = if channel == "stable" {
        format!("{}/{}/releases/latest", GITHUB_API_URL, GITHUB_REPO)
    } else {
        // For beta/nightly, fetch all releases and filter
        format!("{}/{}/releases", GITHUB_API_URL, GITHUB_REPO)
    };

    let client = reqwest::Client::builder()
        .user_agent("cco/client") // LOW FIX #11: Generic user-agent for privacy
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("Failed to fetch release information from GitHub")?;

    if !response.status().is_success() {
        return Err(anyhow!("GitHub API returned status: {}", response.status()));
    }

    let github_release = if channel == "stable" {
        response.json::<GitHubRelease>().await?
    } else {
        // Find first matching release for the channel
        let releases: Vec<GitHubRelease> = response.json().await?;
        releases
            .into_iter()
            .find(|r| {
                let tag = r.tag_name.to_lowercase();
                match channel {
                    "beta" => tag.contains("beta") || tag.contains("rc"),
                    _ => false,
                }
            })
            .ok_or_else(|| anyhow!("No {} releases found", channel))?
    };

    // HIGH FIX #6: Validate release tag BEFORE using it
    let validated_tag = validate_release_tag(&github_release.tag_name)?;

    // Extract version from tag (remove 'v' prefix if present)
    let version = validated_tag.trim_start_matches('v').to_string();

    // Detect current platform
    let platform = Platform::detect()?;

    // Find the appropriate asset for this platform
    let asset_name = platform.asset_name(&version);

    // HIGH FIX #5: Validate asset name before searching
    validate_asset_name(&asset_name)?;

    let asset = github_release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            anyhow!(
                "No release asset found for platform: {} (expected: {})",
                platform.as_str(),
                asset_name
            )
        })?;

    // HIGH FIX #5: Validate download URL before use
    validate_download_url(&asset.browser_download_url)?;

    // CRITICAL FIX #1: Checksum verification is MANDATORY (no optional fallback)
    let checksum = find_checksum(&github_release.assets, &asset_name)
        .await
        .ok_or_else(|| {
            anyhow!(
                "SECURITY: No checksum found for {} - refusing to install unverified binary. \
                All CCO releases must include checksums.sha256 file.",
                asset_name
            )
        })?;

    Ok(ReleaseInfo {
        version,
        release_notes: github_release.body,
        download_url: asset.browser_download_url.clone(),
        checksum: Some(checksum), // Always Some() now due to mandatory check
        size: asset.size,
        filename: asset.name.clone(),
    })
}

/// Find checksum for a specific asset
/// SECURITY: Downloads and parses checksum file with validation
async fn find_checksum(assets: &[GitHubAsset], asset_name: &str) -> Option<String> {
    // Look for checksums.sha256 or similar file
    let checksum_asset = assets
        .iter()
        .find(|a| a.name == "checksums.sha256" || a.name == "SHA256SUMS")?;

    // HIGH FIX #5: Validate checksum asset name
    if validate_asset_name(&checksum_asset.name).is_err() {
        tracing::error!(
            "Checksum asset name validation failed: {}",
            checksum_asset.name
        );
        return None;
    }

    // HIGH FIX #5: Validate checksum download URL
    if validate_download_url(&checksum_asset.browser_download_url).is_err() {
        tracing::error!(
            "Checksum URL validation failed: {}",
            checksum_asset.browser_download_url
        );
        return None;
    }

    // Download checksum file
    let client = reqwest::Client::builder()
        .user_agent("cco/client") // LOW FIX #11: Generic user-agent
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client
        .get(&checksum_asset.browser_download_url)
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let content = response.text().await.ok()?;

    // Parse checksum file (format: "<checksum>  <filename>")
    content
        .lines()
        .find(|line| line.contains(asset_name))
        .and_then(|line| line.split_whitespace().next())
        .map(|s| s.to_string())
}

/// Fetch release by specific version
pub async fn fetch_release_by_version(version: &str) -> Result<ReleaseInfo> {
    // CRITICAL FIX #2: Verify repository ownership FIRST
    verify_repository_ownership().await?;

    let tag = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    };

    // HIGH FIX #6: Validate tag format
    let validated_tag = validate_release_tag(&tag)?;

    let url = format!(
        "{}/{}/releases/tags/{}",
        GITHUB_API_URL, GITHUB_REPO, validated_tag
    );

    let client = reqwest::Client::builder()
        .user_agent("cco/client") // LOW FIX #11: Generic user-agent
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("Failed to fetch release information from GitHub")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Release {} not found (status: {})",
            version,
            response.status()
        ));
    }

    let github_release: GitHubRelease = response.json().await?;

    // HIGH FIX #6: Validate tag (already validated above, but double-check)
    let validated_tag = validate_release_tag(&github_release.tag_name)?;

    // Extract version from tag
    let version = validated_tag.trim_start_matches('v').to_string();

    // Detect current platform
    let platform = Platform::detect()?;
    let asset_name = platform.asset_name(&version);

    // HIGH FIX #5: Validate asset name
    validate_asset_name(&asset_name)?;

    let asset = github_release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| anyhow!("No release asset found for platform: {}", platform.as_str()))?;

    // HIGH FIX #5: Validate download URL
    validate_download_url(&asset.browser_download_url)?;

    // CRITICAL FIX #1: Mandatory checksum verification
    let checksum = find_checksum(&github_release.assets, &asset_name)
        .await
        .ok_or_else(|| {
            anyhow!(
                "SECURITY: No checksum found for {} - refusing to install unverified binary",
                asset_name
            )
        })?;

    Ok(ReleaseInfo {
        version,
        release_notes: github_release.body,
        download_url: asset.browser_download_url.clone(),
        checksum: Some(checksum), // Always Some() due to mandatory check
        size: asset.size,
        filename: asset.name.clone(),
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
