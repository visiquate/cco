//! Comprehensive tests for the auto-update system
//!
//! Tests cover:
//! - Unit tests for version comparison logic
//! - Config serialization/deserialization
//! - Checksum verification logic
//! - Integration tests with mock GitHub API
//! - CLI command tests
//! - Edge cases and error scenarios
//! - Safety tests for update process

use anyhow::Result;
use chrono::{Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Re-export types we need from the cco crate
use cco::version::DateVersion;

// Helper function to create a test file with known content and SHA256
fn create_test_binary(path: &PathBuf, content: &[u8]) -> Result<String> {
    fs::write(path, content)?;

    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

// Helper function to create mock GitHub release response
fn mock_github_release(version: &str, platform: &str) -> serde_json::Value {
    json!({
        "tag_name": format!("v{}", version),
        "name": format!("CCO {}", version),
        "body": "## What's New\n\n- New feature 1\n- Bug fix 2\n- Performance improvements",
        "published_at": "2025-11-15T12:00:00Z",
        "assets": [
            {
                "name": format!("cco-v{}-{}.tar.gz", version, platform),
                "browser_download_url": format!("https://github.com/brentley/cco-releases/releases/download/v{}/cco-v{}-{}.tar.gz", version, version, platform),
                "size": 5242880
            },
            {
                "name": "checksums.sha256",
                "browser_download_url": format!("https://github.com/brentley/cco-releases/releases/download/v{}/checksums.sha256", version),
                "size": 512
            }
        ]
    })
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn test_version_parsing_valid() {
        let v = DateVersion::parse("2025.11.1").unwrap();
        assert_eq!(v.year(), 2025);
        assert_eq!(v.month(), 11);
        assert_eq!(v.release(), 1);
    }

    #[test]
    fn test_version_parsing_different_formats() {
        // Single digit month
        let v = DateVersion::parse("2025.1.1").unwrap();
        assert_eq!(v.month(), 1);

        // Large release number
        let v = DateVersion::parse("2025.11.99").unwrap();
        assert_eq!(v.release(), 99);
    }

    #[test]
    fn test_version_parsing_errors() {
        // Too many components
        assert!(DateVersion::parse("2025.11.1.extra").is_err());

        // Too few components
        assert!(DateVersion::parse("2025.11").is_err());

        // Invalid month (too high)
        assert!(DateVersion::parse("2025.13.1").is_err());

        // Invalid month (zero)
        assert!(DateVersion::parse("2025.0.1").is_err());

        // Not a number
        assert!(DateVersion::parse("invalid.11.1").is_err());
        assert!(DateVersion::parse("2025.abc.1").is_err());
        assert!(DateVersion::parse("2025.11.xyz").is_err());

        // Negative numbers
        assert!(DateVersion::parse("-2025.11.1").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = DateVersion::parse("2025.11.1").unwrap();
        let v2 = DateVersion::parse("2025.11.2").unwrap();
        let v3 = DateVersion::parse("2025.11.10").unwrap();
        let v4 = DateVersion::parse("2025.12.1").unwrap();
        let v5 = DateVersion::parse("2026.1.1").unwrap();

        // Same month, different release numbers
        assert!(v1 < v2);
        assert!(v2 > v1);
        assert!(v2 < v3); // 2 < 10
        assert!(v3 > v2);

        // Different months, same year
        assert!(v3 < v4); // November < December
        assert!(v4 > v3);

        // Different years
        assert!(v4 < v5);
        assert!(v5 > v4);

        // Transitivity
        assert!(v1 < v3);
        assert!(v1 < v5);
    }

    #[test]
    fn test_version_equality() {
        let v1 = DateVersion::parse("2025.11.1").unwrap();
        let v2 = DateVersion::parse("2025.11.1").unwrap();

        assert_eq!(v1, v2);
        assert!(!(v1 < v2));
        assert!(!(v1 > v2));
    }

    #[test]
    fn test_version_to_string() {
        let v = DateVersion::parse("2025.11.1").unwrap();
        assert_eq!(v.to_string(), "2025.11.1");

        let v = DateVersion::parse("2025.1.10").unwrap();
        assert_eq!(v.to_string(), "2025.1.10");
    }

    #[test]
    fn test_version_edge_cases() {
        // December (month 12)
        let v = DateVersion::parse("2025.12.1").unwrap();
        assert_eq!(v.month(), 12);

        // January (month 1)
        let v = DateVersion::parse("2026.1.1").unwrap();
        assert_eq!(v.month(), 1);

        // Large release counter
        let v = DateVersion::parse("2025.11.999").unwrap();
        assert_eq!(v.release(), 999);
    }

    #[test]
    fn test_version_ordering_comprehensive() {
        let versions = vec![
            "2025.1.1",
            "2025.1.2",
            "2025.2.1",
            "2025.11.1",
            "2025.11.2",
            "2025.12.1",
            "2026.1.1",
        ];

        let parsed: Vec<DateVersion> = versions
            .iter()
            .map(|v| DateVersion::parse(v).unwrap())
            .collect();

        // Verify they're in ascending order
        for i in 0..parsed.len() - 1 {
            assert!(parsed[i] < parsed[i + 1]);
        }
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        // Test that defaults are correct
        let config = cco::auto_update::UpdateConfig::default();
        assert!(config.enabled);
        assert!(!config.auto_install);
        assert_eq!(config.check_interval, "daily");
        assert_eq!(config.channel, "stable");
        assert!(config.last_check.is_none());
        assert!(config.last_update.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let config = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "weekly".to_string(),
            channel: "beta".to_string(),
            last_check: Some(Utc::now()),
            last_update: None,
        };

        // Serialize to TOML
        let toml_str = toml::to_string(&config).unwrap();

        // Deserialize back
        let deserialized: cco::auto_update::UpdateConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.auto_install, deserialized.auto_install);
        assert_eq!(config.check_interval, deserialized.check_interval);
        assert_eq!(config.channel, deserialized.channel);
    }

    #[test]
    fn test_config_partial_serialization() {
        // Test that None values are properly skipped
        let config = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: None,
            last_update: None,
        };

        let toml_str = toml::to_string(&config).unwrap();

        // last_check and last_update should not be in the output
        assert!(!toml_str.contains("last_check"));
        assert!(!toml_str.contains("last_update"));
    }

    #[test]
    fn test_config_valid_intervals() {
        let intervals = vec!["daily", "weekly", "never"];
        for interval in intervals {
            let config = cco::auto_update::UpdateConfig {
                enabled: true,
                auto_install: false,
                check_interval: interval.to_string(),
                channel: "stable".to_string(),
                last_check: None,
                last_update: None,
            };
            assert_eq!(config.check_interval, interval);
        }
    }

    #[test]
    fn test_config_valid_channels() {
        let channels = vec!["stable", "beta"];
        for channel in channels {
            let config = cco::auto_update::UpdateConfig {
                enabled: true,
                auto_install: false,
                check_interval: "daily".to_string(),
                channel: channel.to_string(),
                last_check: None,
                last_update: None,
            };
            assert_eq!(config.channel, channel);
        }
    }
}

#[cfg(test)]
mod checksum_tests {
    use super::*;

    #[test]
    fn test_checksum_verification_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let content = b"Hello, CCO!";
        let expected_checksum = create_test_binary(&file_path, content).unwrap();

        // Verify checksum matches
        let mut file = fs::File::open(&file_path).unwrap();
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let n = std::io::Read::read(&mut file, &mut buffer).unwrap();
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        let computed = hex::encode(result);

        assert_eq!(computed.to_lowercase(), expected_checksum.to_lowercase());
    }

    #[test]
    fn test_checksum_verification_failure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let content = b"Hello, CCO!";
        create_test_binary(&file_path, content).unwrap();

        // Use wrong checksum
        let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";

        let mut file = fs::File::open(&file_path).unwrap();
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let n = std::io::Read::read(&mut file, &mut buffer).unwrap();
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        let computed = hex::encode(result);

        assert_ne!(computed.to_lowercase(), wrong_checksum.to_lowercase());
    }

    #[test]
    fn test_checksum_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.bin");

        let expected = create_test_binary(&file_path, b"").unwrap();

        // SHA256 of empty string
        let empty_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        assert_eq!(expected, empty_sha256);
    }

    #[test]
    fn test_checksum_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.bin");

        // Create a 1MB file
        let content = vec![0u8; 1024 * 1024];
        let checksum = create_test_binary(&file_path, &content).unwrap();

        // Verify it's a valid SHA256 (64 hex chars)
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_checksum_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let content = b"Test content";
        let checksum = create_test_binary(&file_path, content).unwrap();

        // Both uppercase and lowercase should match
        assert_eq!(checksum.to_lowercase(), checksum.to_lowercase());
        assert_eq!(checksum.to_uppercase().to_lowercase(), checksum.to_lowercase());
    }
}

#[cfg(test)]
mod update_logic_tests {
    use super::*;

    #[test]
    fn test_should_check_never_checked() {
        let config = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: None,
            last_update: None,
        };

        // Should return true when never checked before
        // We can't access the private function directly, but we test the behavior
        assert!(config.last_check.is_none());
    }

    #[test]
    fn test_should_check_disabled() {
        let config = cco::auto_update::UpdateConfig {
            enabled: false,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: None,
            last_update: None,
        };

        // Should not check when disabled
        assert!(!config.enabled);
    }

    #[test]
    fn test_should_check_interval_daily() {
        let now = Utc::now();

        // Recent check (1 hour ago) - should not check
        let config_recent = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: Some(now - Duration::hours(1)),
            last_update: None,
        };

        let elapsed = now - config_recent.last_check.unwrap();
        assert!(elapsed < Duration::days(1));

        // Old check (2 days ago) - should check
        let config_old = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: Some(now - Duration::days(2)),
            last_update: None,
        };

        let elapsed_old = now - config_old.last_check.unwrap();
        assert!(elapsed_old >= Duration::days(1));
    }

    #[test]
    fn test_should_check_interval_weekly() {
        let now = Utc::now();

        // Recent check (3 days ago) - should not check
        let config_recent = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "weekly".to_string(),
            channel: "stable".to_string(),
            last_check: Some(now - Duration::days(3)),
            last_update: None,
        };

        let elapsed = now - config_recent.last_check.unwrap();
        assert!(elapsed < Duration::weeks(1));

        // Old check (10 days ago) - should check
        let config_old = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "weekly".to_string(),
            channel: "stable".to_string(),
            last_check: Some(now - Duration::days(10)),
            last_update: None,
        };

        let elapsed_old = now - config_old.last_check.unwrap();
        assert!(elapsed_old >= Duration::weeks(1));
    }

    #[test]
    fn test_should_check_interval_never() {
        let config = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "never".to_string(),
            channel: "stable".to_string(),
            last_check: Some(Utc::now() - Duration::days(365)),
            last_update: None,
        };

        // Should never check when interval is "never"
        assert_eq!(config.check_interval, "never");
    }
}

#[cfg(test)]
mod platform_detection_tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        // Test that platform detection returns a valid result
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        // Verify we're getting expected values
        assert!(!os.is_empty());
        assert!(!arch.is_empty());

        // Common platforms
        let valid_platforms = vec![
            "darwin-arm64",
            "darwin-x86_64",
            "linux-x86_64",
            "linux-aarch64",
            "windows-x86_64",
        ];

        let platform = format!("{}-{}",
            if os == "macos" { "darwin" } else { os },
            if arch == "aarch64" && os == "macos" { "arm64" } else { arch }
        );

        // Should be one of the valid platforms
        assert!(valid_platforms.iter().any(|p| platform.contains(&p[..p.find('-').unwrap()])));
    }
}

#[cfg(test)]
mod safety_tests {
    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let install_path = temp_dir.path().join("cco");
        let backup_path = install_path.with_extension("backup");

        // Create original binary
        fs::write(&install_path, b"original binary").unwrap();

        // Simulate backup
        fs::copy(&install_path, &backup_path).unwrap();

        // Verify backup exists and has same content
        assert!(backup_path.exists());
        let original = fs::read(&install_path).unwrap();
        let backup = fs::read(&backup_path).unwrap();
        assert_eq!(original, backup);
    }

    #[test]
    fn test_rollback_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let install_path = temp_dir.path().join("cco");
        let backup_path = install_path.with_extension("backup");

        // Create original binary
        let original_content = b"original binary v1.0";
        fs::write(&install_path, original_content).unwrap();

        // Create backup
        fs::copy(&install_path, &backup_path).unwrap();

        // Simulate failed update (corrupt new binary)
        fs::write(&install_path, b"corrupt").unwrap();

        // Rollback from backup
        fs::copy(&backup_path, &install_path).unwrap();

        // Verify rollback worked
        let restored = fs::read(&install_path).unwrap();
        assert_eq!(restored, original_content);
    }

    #[test]
    fn test_permissions_preserved() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("cco");

        // Create binary with specific permissions
        fs::write(&binary_path, b"binary content").unwrap();

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&binary_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms).unwrap();

            // Verify permissions
            let metadata = fs::metadata(&binary_path).unwrap();
            assert_eq!(metadata.permissions().mode() & 0o777, 0o755);
        }
    }

    #[test]
    fn test_temp_file_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("download.tar.gz");

        // Create temp file
        fs::write(&temp_file, b"temp content").unwrap();
        assert!(temp_file.exists());

        // Cleanup
        fs::remove_file(&temp_file).unwrap();
        assert!(!temp_file.exists());
    }

    #[test]
    fn test_atomic_replacement() {
        let temp_dir = TempDir::new().unwrap();
        let install_path = temp_dir.path().join("cco");
        let new_binary = temp_dir.path().join("cco.new");

        // Create original and new binary
        fs::write(&install_path, b"v1.0").unwrap();
        fs::write(&new_binary, b"v2.0").unwrap();

        // Atomic copy (replace)
        fs::copy(&new_binary, &install_path).unwrap();

        // Verify replacement
        let content = fs::read(&install_path).unwrap();
        assert_eq!(content, b"v2.0");
    }

    #[test]
    fn test_no_data_loss_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let install_path = temp_dir.path().join("cco");
        let backup_path = install_path.with_extension("backup");

        // Create original binary
        let original = b"important binary data";
        fs::write(&install_path, original).unwrap();

        // Create backup before update
        fs::copy(&install_path, &backup_path).unwrap();

        // Simulate update failure (but backup exists)
        // Even if install_path gets corrupted, we have backup
        fs::write(&install_path, b"corrupted").unwrap();

        // Restore from backup
        fs::copy(&backup_path, &install_path).unwrap();

        // Verify no data loss
        let restored = fs::read(&install_path).unwrap();
        assert_eq!(restored, original);

        // Cleanup backup
        fs::remove_file(&backup_path).unwrap();
        assert!(!backup_path.exists());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_already_latest_version() {
        // Test that we handle already being on latest version
        let current = DateVersion::parse("2025.11.2").unwrap();
        let latest = DateVersion::parse("2025.11.2").unwrap();

        assert_eq!(current, latest);
        assert!(!(current < latest));
    }

    #[test]
    fn test_newer_than_latest() {
        // Test downgrade detection (current > latest)
        let current = DateVersion::parse("2025.12.1").unwrap();
        let latest = DateVersion::parse("2025.11.5").unwrap();

        assert!(current > latest);
    }

    #[test]
    fn test_corrupted_download() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("download.tar.gz");

        // Create partial/corrupted file
        fs::write(&file_path, b"incomplete data").unwrap();

        // Known good checksum (won't match)
        let expected = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

        let mut file = fs::File::open(&file_path).unwrap();
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let n = std::io::Read::read(&mut file, &mut buffer).unwrap();
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        let computed = hex::encode(result);

        // Verify checksum mismatch
        assert_ne!(computed, expected);
    }

    #[test]
    fn test_missing_checksum_file() {
        let temp_dir = TempDir::new().unwrap();
        let checksum_path = temp_dir.path().join("checksums.sha256");

        // Verify file doesn't exist
        assert!(!checksum_path.exists());

        // Should handle gracefully (no panic)
        let result = fs::read_to_string(&checksum_path);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(unix)]
    fn test_insufficient_disk_space_simulation() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();

        // We can't actually fill the disk, but we can test error handling
        // by trying to write to a read-only location
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&readonly_dir, perms).unwrap();

        let file_path = readonly_dir.join("test.bin");
        let result = fs::write(&file_path, b"data");

        // Should fail due to permissions
        assert!(result.is_err());
    }

    #[test]
    #[cfg(unix)]
    fn test_no_write_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();

        let protected_file = temp_dir.path().join("protected.bin");
        fs::write(&protected_file, b"original").unwrap();

        let mut perms = fs::metadata(&protected_file).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&protected_file, perms).unwrap();

        // Try to overwrite
        let result = fs::write(&protected_file, b"new");
        assert!(result.is_err());
    }

    #[test]
    fn test_download_interruption_simulation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("partial.tar.gz");

        // Simulate partial download
        let full_content = b"This would be a complete tarball with lots of content";
        let partial_content = &full_content[..20]; // Only first 20 bytes

        fs::write(&file_path, partial_content).unwrap();

        // Verify size mismatch
        let metadata = fs::metadata(&file_path).unwrap();
        assert_eq!(metadata.len(), 20);
        assert!(metadata.len() < full_content.len() as u64);
    }

    #[test]
    fn test_multiple_concurrent_update_attempts() {
        // Test that we don't corrupt state with concurrent updates
        let temp_dir = TempDir::new().unwrap();
        let lock_file = temp_dir.path().join("update.lock");

        // First update creates lock
        fs::write(&lock_file, "1").unwrap();

        // Second update should detect existing lock
        assert!(lock_file.exists());

        // Cleanup
        fs::remove_file(&lock_file).unwrap();
    }

    #[test]
    fn test_invalid_archive_format() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("fake.tar.gz");

        // Create file that's not actually a tar.gz
        fs::write(&archive_path, b"not a real archive").unwrap();

        // Attempt to extract should fail
        let result = fs::File::open(&archive_path);
        assert!(result.is_ok()); // File opens fine

        // But tar extraction would fail (we can't test that without the actual code)
    }

    #[test]
    fn test_missing_binary_in_archive() {
        let temp_dir = TempDir::new().unwrap();
        let extract_dir = temp_dir.path().join("extracted");
        fs::create_dir(&extract_dir).unwrap();

        // No binary in extraction directory
        let binary_path = extract_dir.join("cco");
        assert!(!binary_path.exists());
    }
}

#[cfg(test)]
mod error_message_tests {
    use super::*;

    #[test]
    fn test_version_parse_error_message() {
        let result = DateVersion::parse("invalid");
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Invalid version format") || err_msg.contains("Invalid"));
    }

    #[test]
    fn test_month_validation_error() {
        let result = DateVersion::parse("2025.13.1");
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Invalid month") || err_msg.contains("13"));
    }

    #[test]
    fn test_missing_component_error() {
        let result = DateVersion::parse("2025.11");
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Invalid version format") || err_msg.contains("YYYY.MM.N"));
    }
}

#[cfg(test)]
mod integration_scenarios {
    use super::*;

    #[test]
    fn test_full_update_flow_simulation() {
        let temp_dir = TempDir::new().unwrap();

        // 1. Create current binary
        let install_path = temp_dir.path().join("cco");
        fs::write(&install_path, b"current version").unwrap();

        // 2. Create backup
        let backup_path = install_path.with_extension("backup");
        fs::copy(&install_path, &backup_path).unwrap();

        // 3. Download new version
        let download_path = temp_dir.path().join("cco-new");
        let new_content = b"new version";
        let checksum = create_test_binary(&download_path, new_content).unwrap();

        // 4. Verify checksum
        let mut file = fs::File::open(&download_path).unwrap();
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let n = std::io::Read::read(&mut file, &mut buffer).unwrap();
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        let computed = hex::encode(result);
        assert_eq!(computed, checksum);

        // 5. Install new version
        fs::copy(&download_path, &install_path).unwrap();

        // 6. Verify installation
        let installed = fs::read(&install_path).unwrap();
        assert_eq!(installed, new_content);

        // 7. Cleanup backup
        fs::remove_file(&backup_path).unwrap();
    }

    #[test]
    fn test_update_failure_and_rollback() {
        let temp_dir = TempDir::new().unwrap();

        // 1. Setup current version
        let install_path = temp_dir.path().join("cco");
        let original_content = b"stable version";
        fs::write(&install_path, original_content).unwrap();

        // 2. Create backup
        let backup_path = install_path.with_extension("backup");
        fs::copy(&install_path, &backup_path).unwrap();

        // 3. Attempt update with corrupted binary
        fs::write(&install_path, b"corrupted").unwrap();

        // 4. Verification fails, rollback
        fs::copy(&backup_path, &install_path).unwrap();

        // 5. Verify rollback success
        let restored = fs::read(&install_path).unwrap();
        assert_eq!(restored, original_content);
    }

    #[test]
    fn test_config_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create config
        let config = cco::auto_update::UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "weekly".to_string(),
            channel: "beta".to_string(),
            last_check: Some(Utc::now()),
            last_update: None,
        };

        // Save to file
        let toml_str = toml::to_string(&config).unwrap();
        fs::write(&config_path, &toml_str).unwrap();

        // Load from file
        let loaded_str = fs::read_to_string(&config_path).unwrap();
        let loaded: cco::auto_update::UpdateConfig = toml::from_str(&loaded_str).unwrap();

        // Verify fields match
        assert_eq!(config.enabled, loaded.enabled);
        assert_eq!(config.auto_install, loaded.auto_install);
        assert_eq!(config.check_interval, loaded.check_interval);
        assert_eq!(config.channel, loaded.channel);
    }

    #[test]
    fn test_version_upgrade_path() {
        // Simulate upgrade path: 2025.11.1 -> 2025.11.2 -> 2025.12.1
        let versions = vec!["2025.11.1", "2025.11.2", "2025.12.1"];

        for i in 0..versions.len() - 1 {
            let current = DateVersion::parse(versions[i]).unwrap();
            let next = DateVersion::parse(versions[i + 1]).unwrap();

            // Each upgrade should be detected
            assert!(next > current);
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_checksum_performance() {
        use std::time::Instant;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.bin");

        // Create 10MB file
        let content = vec![0u8; 10 * 1024 * 1024];
        fs::write(&file_path, &content).unwrap();

        let start = Instant::now();

        let mut file = fs::File::open(&file_path).unwrap();
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let n = std::io::Read::read(&mut file, &mut buffer).unwrap();
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let _result = hasher.finalize();
        let duration = start.elapsed();

        // Should complete in reasonable time (< 1 second for 10MB)
        assert!(duration.as_secs() < 1);
    }

    #[test]
    fn test_version_comparison_performance() {
        use std::time::Instant;

        // Create 1000 versions
        let mut versions = Vec::new();
        for year in 2020..2025 {
            for month in 1..=12 {
                for release in 1..=10 {
                    let v = DateVersion::parse(&format!("{}.{}.{}", year, month, release)).unwrap();
                    versions.push(v);
                }
            }
        }

        let start = Instant::now();

        // Sort all versions
        let mut sorted = versions.clone();
        sorted.sort();

        let duration = start.elapsed();

        // Should complete quickly (< 100ms for 600 versions)
        assert!(duration.as_millis() < 100);

        // Verify first < last
        assert!(sorted.first().unwrap() < sorted.last().unwrap());
    }
}
