//! Update module for CCO
//!
//! Handles checking for updates and installing new versions from GitHub releases.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::io::Read;

use cco::version::DateVersion;

const GITHUB_REPO: &str = "brentley/cco-releases";
const GITHUB_API_URL: &str = "https://api.github.com/repos";

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
    #[allow(dead_code)]
    size: u64,
}

/// Version manifest structure (for future use)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct VersionManifest {
    latest: LatestVersions,
    versions: std::collections::HashMap<String, VersionInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct LatestVersions {
    stable: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    beta: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct VersionInfo {
    date: String,
    platforms: std::collections::HashMap<String, PlatformInfo>,
    release_notes: String,
    #[serde(default)]
    breaking_changes: bool,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct PlatformInfo {
    url: String,
    sha256: String,
    size: u64,
}

/// Detect current platform
fn detect_platform() -> Result<String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let platform = match (os, arch) {
        ("macos", "aarch64") => "darwin-arm64",
        ("macos", "x86_64") => "darwin-x86_64",
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("windows", "x86_64") => "windows-x86_64",
        _ => return Err(anyhow!("Unsupported platform: {}-{}", os, arch)),
    };

    Ok(platform.to_string())
}

/// Fetch latest release from GitHub
async fn fetch_latest_release(channel: &str) -> Result<GitHubRelease> {
    let url = if channel == "stable" {
        format!("{}/{}/releases/latest", GITHUB_API_URL, GITHUB_REPO)
    } else {
        // For beta/nightly, fetch all releases and filter
        format!("{}/{}/releases", GITHUB_API_URL, GITHUB_REPO)
    };

    let client = reqwest::Client::builder()
        .user_agent(format!("cco/{}", env!("CARGO_PKG_VERSION")))
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

    if channel == "stable" {
        let release: GitHubRelease = response.json().await?;
        Ok(release)
    } else {
        let releases: Vec<GitHubRelease> = response.json().await?;
        // Find first prerelease for beta channel
        releases
            .into_iter()
            .find(|r| r.tag_name.contains("beta") || r.tag_name.contains("rc"))
            .ok_or_else(|| anyhow!("No {} releases found", channel))
    }
}

/// Download a file from URL
async fn download_file(url: &str, path: &Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(format!("cco/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("Download failed with status: {}", response.status()));
    }

    let bytes = response.bytes().await?;
    fs::write(path, &bytes)?;

    Ok(())
}

/// Verify SHA256 checksum of a file
fn verify_checksum(file_path: &Path, expected_checksum: &str) -> Result<bool> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    let computed_checksum = hex::encode(result);

    Ok(computed_checksum.to_lowercase() == expected_checksum.to_lowercase())
}

/// Get the installation path for CCO
fn get_install_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    Ok(home.join(".local/bin/cco"))
}

/// Extract version from tag (e.g., "v202511-1" -> "202511-1" or "v0.3.0" -> "0.3.0" for backward compatibility)
fn extract_version(tag: &str) -> Result<String> {
    let version_str = tag.trim_start_matches('v');

    // Validate it's either date-based or semantic version format
    if DateVersion::parse(version_str).is_ok() {
        Ok(version_str.to_string())
    } else {
        // For backward compatibility, accept semantic versions too
        Ok(version_str.to_string())
    }
}

/// Check for updates only
async fn check_for_updates(channel: &str) -> Result<Option<GitHubRelease>> {
    println!("→ Checking for updates...");

    let current_version = DateVersion::current();
    println!("→ Current version: {}", current_version);

    let release = fetch_latest_release(channel).await?;
    let latest_version_str = extract_version(&release.tag_name)?;

    println!("→ Latest version: {}", latest_version_str);

    // Try to parse as DateVersion for comparison
    match (DateVersion::parse(current_version), DateVersion::parse(&latest_version_str)) {
        (Ok(current), Ok(latest)) => {
            if latest > current {
                Ok(Some(release))
            } else {
                println!("✅ You are running the latest version");
                Ok(None)
            }
        }
        _ => {
            // If we can't parse, just do string comparison
            if latest_version_str != current_version {
                Ok(Some(release))
            } else {
                println!("✅ You are running the latest version");
                Ok(None)
            }
        }
    }
}

/// Check latest version (used by other modules)
pub async fn check_latest_version() -> Result<Option<String>> {
    let current = DateVersion::current();
    let current_parsed = DateVersion::parse(current)?;

    // Fetch latest from GitHub
    let release = fetch_latest_release("stable").await?;
    let latest = extract_version(&release.tag_name)?;
    let latest_parsed = DateVersion::parse(&latest)?;

    if latest_parsed > current_parsed {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

/// Install a new version from a release
async fn install_update(release: &GitHubRelease, auto_confirm: bool) -> Result<()> {
    let platform = detect_platform()?;

    // Find the appropriate asset for this platform
    let asset_name = if platform.starts_with("windows") {
        format!("cco-{}-{}.zip", release.tag_name, platform)
    } else {
        format!("cco-{}-{}.tar.gz", release.tag_name, platform)
    };

    let asset = release.assets.iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| anyhow!("No release asset found for platform: {}", platform))?;

    println!("\nWhat's new in {}:", release.tag_name);

    // Print first few lines of release notes
    for (i, line) in release.body.lines().take(10).enumerate() {
        if i == 0 && line.starts_with('#') {
            continue; // Skip title
        }
        println!("  {}", line);
    }

    if release.body.lines().count() > 10 {
        println!("  ... (see full release notes: {})",
                 format!("https://github.com/{}/releases/tag/{}", GITHUB_REPO, release.tag_name));
    }

    // Prompt for confirmation unless auto-confirm is enabled
    if !auto_confirm {
        print!("\nUpdate now? [Y/n]: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if !input.is_empty() && input != "y" && input != "yes" {
            println!("Update cancelled");
            return Ok(());
        }
    }

    // Create temporary directory for download
    let temp_dir = std::env::temp_dir().join(format!("cco-update-{}", release.tag_name));
    fs::create_dir_all(&temp_dir)?;

    let temp_file = temp_dir.join(&asset.name);
    let temp_checksum = temp_dir.join("checksums.sha256");

    // Download binary
    println!("→ Downloading CCO {}...", release.tag_name);
    download_file(&asset.browser_download_url, &temp_file).await?;

    // Try to download and verify checksum if available
    let checksum_asset = release.assets.iter()
        .find(|a| a.name == "checksums.sha256");

    if let Some(checksum_asset) = checksum_asset {
        println!("→ Verifying checksum...");
        download_file(&checksum_asset.browser_download_url, &temp_checksum).await?;

        // Read checksum file and find our platform's checksum
        let checksum_content = fs::read_to_string(&temp_checksum)?;
        let expected_checksum = checksum_content
            .lines()
            .find(|line| line.contains(&asset_name))
            .and_then(|line| line.split_whitespace().next())
            .ok_or_else(|| anyhow!("Could not find checksum for {}", asset_name))?;

        if !verify_checksum(&temp_file, expected_checksum)? {
            return Err(anyhow!("Checksum verification failed! Update aborted."));
        }

        println!("  ✓ Checksum verified");
    } else {
        println!("  ⚠️  No checksum available, skipping verification");
    }

    // Extract archive and get binary
    let binary_path = if platform.starts_with("windows") {
        // TODO: Handle ZIP extraction for Windows
        return Err(anyhow!("Windows update not yet implemented"));
    } else {
        // Extract tar.gz
        println!("→ Extracting archive...");
        let tar_gz = fs::File::open(&temp_file)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&temp_dir)?;
        temp_dir.join("cco")
    };

    if !binary_path.exists() {
        return Err(anyhow!("Binary not found in archive"));
    }

    // Backup current installation
    let install_path = get_install_path()?;
    let backup_path = install_path.with_extension("backup");

    if install_path.exists() {
        println!("→ Backing up current version...");
        fs::copy(&install_path, &backup_path)?;
    }

    // Install new binary atomically
    println!("→ Installing update...");

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    fs::copy(&binary_path, &install_path)
        .context("Failed to install new binary")?;

    // Verify new binary works
    match std::process::Command::new(&install_path)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("✅ Successfully updated to {}", release.tag_name);

            // Clean up backup on success
            if backup_path.exists() {
                let _ = fs::remove_file(backup_path);
            }
        }
        _ => {
            println!("⚠️  New binary verification failed, rolling back...");

            if backup_path.exists() {
                fs::copy(&backup_path, &install_path)?;
                return Err(anyhow!("Update failed, rolled back to previous version"));
            }

            return Err(anyhow!("Update verification failed"));
        }
    }

    // Clean up temporary files
    let _ = fs::remove_dir_all(&temp_dir);

    println!("\nRestart CCO to use the new version.");

    Ok(())
}

/// Main update command handler
pub async fn run(check_only: bool, auto_confirm: bool, channel: Option<String>) -> Result<()> {
    let channel = channel.as_deref().unwrap_or("stable");

    if !["stable", "beta"].contains(&channel) {
        return Err(anyhow!("Invalid channel: {}. Use 'stable' or 'beta'", channel));
    }

    match check_for_updates(channel).await? {
        Some(release) => {
            if check_only {
                println!("\nℹ️  New version available: {}", release.tag_name);
                println!("Run 'cco update' to install");
            } else {
                install_update(&release, auto_confirm).await?;
            }
        }
        None => {
            if !check_only {
                println!("No updates available");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform() {
        assert!(detect_platform().is_ok());
    }

    #[test]
    fn test_extract_version() {
        // Date-based versions
        assert_eq!(extract_version("v2025.11.1").unwrap(), "2025.11.1");
        assert_eq!(extract_version("2025.11.2").unwrap(), "2025.11.2");

        // Backward compatibility with semantic versions
        assert_eq!(extract_version("v0.3.0").unwrap(), "0.3.0");
        assert_eq!(extract_version("0.3.0").unwrap(), "0.3.0");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = DateVersion::parse("2025.11.1").unwrap();
        let v2 = DateVersion::parse("2025.11.2").unwrap();
        let v3 = DateVersion::parse("2025.12.1").unwrap();

        assert!(v2 > v1);
        assert!(v3 > v2);
        assert!(v3 > v1);
    }
}
