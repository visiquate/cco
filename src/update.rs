//! Command-line update flow using GitHub Releases (stable-only)
//!
//! This reuses the auto-update release metadata and updater routines:
//! - Metadata: GitHub Releases + checksums.txt (stable only)
//! - Integrity: mandatory SHA256 from checksums.txt
//! - Install: download, verify, extract, replace binary with rollback

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use cco::auto_update::releases_api::ReleaseInfo;
use cco::auto_update::{releases_api, updater};
use cco::version::DateVersion;

/// Detect multiple installations and warn user (best-effort)
async fn check_for_multiple_installations() -> Result<()> {
    let mut found_installations = Vec::new();

    // Check common paths
    let paths_to_check = vec![
        dirs::home_dir().map(|h| h.join(".local/bin/cco")),
        Some(PathBuf::from("/usr/local/bin/cco")),
        Some(PathBuf::from("/opt/cco/cco")),
        Some(PathBuf::from("/usr/bin/cco")),
    ];

    for path_opt in paths_to_check {
        if let Some(path) = path_opt {
            if path.exists() {
                if let Ok(output) = std::process::Command::new(&path).arg("--version").output() {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        found_installations.push((path, version));
                    }
                }
            }
        }
    }

    if found_installations.len() > 1 {
        println!("\n⚠️  WARNING: Multiple CCO installations detected:");
        for (path, version) in &found_installations {
            println!("  - {} (version: {})", path.display(), version);
        }
        println!("\nThis can cause PATH shadowing issues.\n");
    }

    Ok(())
}

/// Verify SHA256 checksum of a file
#[allow(dead_code)]
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

/// Check for updates (returns release info if update available)
async fn check_for_updates() -> Result<Option<ReleaseInfo>> {
    // Check for multiple installations (non-fatal)
    let _ = check_for_multiple_installations().await;

    println!("→ Checking for updates...");

    let current_version = DateVersion::current();
    println!("→ Current version: {}", current_version);

    let release = releases_api::fetch_latest_release("stable")
        .await
        .context("Failed to fetch latest release from GitHub")?;

    println!("→ Latest version: {}", release.version);

    match (
        DateVersion::parse(current_version),
        DateVersion::parse(&release.version),
    ) {
        (Ok(current), Ok(latest)) => {
            if latest > current {
                Ok(Some(release))
            } else {
                println!("✅ You are running the latest version");
                Ok(None)
            }
        }
        _ => {
            if release.version != current_version {
                Ok(Some(release))
            } else {
                println!("✅ You are running the latest version");
                Ok(None)
            }
        }
    }
}

/// Install a new version from release metadata
async fn install_update(release: &ReleaseInfo, auto_confirm: bool) -> Result<()> {
    println!("\nWhat's new in {}:", release.version);
    for (i, line) in release.release_notes.lines().take(10).enumerate() {
        if i == 0 && line.starts_with('#') {
            continue;
        }
        println!("  {}", line);
    }
    if release.release_notes.lines().count() > 10 {
        println!("  ... (see full release notes on GitHub)");
    }

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

    println!("→ Downloading CCO {}...", release.version);
    let binary_path = updater::download_and_verify(release).await?;

    println!("→ Installing update...");
    updater::replace_binary(&binary_path)
        .await
        .context("Failed to install update")?;

    println!("✅ Successfully updated to {}", release.version);
    println!("\nRestart CCO to use the new version.");

    Ok(())
}

/// Check for the latest version available (used by background update checks)
pub async fn check_latest_version() -> Result<Option<DateVersion>> {
    let release = releases_api::fetch_latest_release("stable").await?;

    // Parse and return the version
    match DateVersion::parse(&release.version) {
        Ok(version) => Ok(Some(version)),
        Err(_) => {
            // If we can't parse the version, return None
            tracing::warn!("Could not parse version from release: {}", release.version);
            Ok(None)
        }
    }
}

/// Main update command handler
pub async fn run(check_only: bool, auto_confirm: bool, channel: Option<String>) -> Result<()> {
    let channel = channel.as_deref().unwrap_or("stable");
    if channel != "stable" {
        return Err(anyhow!("Only the stable channel is supported"));
    }

    match check_for_updates().await? {
        Some(release) => {
            if check_only {
                println!("\nℹ️  New version available: {}", release.version);
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
    fn test_verify_checksum() {
        // Simple self-check
        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), b"hello").unwrap();
        let expected = {
            let mut hasher = Sha256::new();
            hasher.update(b"hello");
            hex::encode(hasher.finalize())
        };
        assert!(verify_checksum(tmp.path(), &expected).unwrap());
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
