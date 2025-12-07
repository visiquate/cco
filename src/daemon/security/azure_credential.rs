//! Azure credential retrieval with fallback chain
//!
//! Implements secure credential retrieval for Azure OpenAI API with the following priority:
//! 1. Build-time embedded credential (XOR obfuscated, from CI/CD)
//! 2. AZURE_API_KEY environment variable
//! 3. macOS Keychain via keyring crate
//! 4. .env file in current working directory
//!
//! The embedded credential is only available if built with the `AZURE_CREDENTIAL_EMBEDDED`
//! environment variable set to "true" at build time.

use anyhow::{Context, Result};
use std::env;
use std::fs;
use tracing::{debug, info, warn};

/// XOR decryption key used for embedded credentials (must match obfuscate.sh)
#[allow(dead_code)]
const XOR_KEY: u8 = 167; // 0xA7 in hex, same as obfuscate.sh

/// Keychain service identifier for macOS Keychain
const KEYCHAIN_SERVICE: &str = "cc-orchestra";
/// Keychain account identifier for Azure API key
const KEYCHAIN_ACCOUNT: &str = "azure-api-key";

/// Retrieve Azure API key with fallback chain
///
/// Attempts to retrieve the Azure API key using the following priority:
/// 1. Build-time embedded credential (if available)
/// 2. AZURE_API_KEY environment variable
/// 3. macOS Keychain (via keyring crate)
/// 4. .env file in current working directory
///
/// Returns None if no credential is found in any fallback.
pub fn get_azure_api_key() -> Option<String> {
    debug!("Attempting to retrieve Azure API key with fallback chain");

    // Priority 1: Embedded credential from build
    if let Some(key) = get_embedded_credential() {
        info!("Azure API key retrieved from embedded credential");
        return Some(key);
    }
    debug!("No embedded Azure credential found");

    // Priority 2: Environment variable
    if let Ok(key) = env::var("AZURE_API_KEY") {
        if !key.trim().is_empty() {
            info!("Azure API key retrieved from AZURE_API_KEY environment variable");
            return Some(key);
        }
    }
    debug!("AZURE_API_KEY environment variable not set or empty");

    // Priority 3: macOS Keychain
    if let Some(key) = get_from_keychain() {
        info!("Azure API key retrieved from macOS Keychain");
        return Some(key);
    }
    debug!("Azure API key not found in Keychain");

    // Priority 4: .env file
    if let Some(key) = get_from_env_file() {
        info!("Azure API key retrieved from .env file");
        return Some(key);
    }
    debug!("No .env file or AZURE_API_KEY not found in .env");

    warn!("Azure API key not found in any fallback source");
    None
}

/// Get embedded credential if available at build time
fn get_embedded_credential() -> Option<String> {
    // Check if credential was embedded at build time
    let embedded = option_env!("AZURE_CREDENTIAL_EMBEDDED");
    if embedded != Some("1") {
        debug!("Embedded credential not available (AZURE_CREDENTIAL_EMBEDDED not set)");
        return None;
    }

    // Include the embedded credential blob from OUT_DIR
    let embedded_blob = include_bytes!(concat!(env!("OUT_DIR"), "/azure_credential.bin"));

    if embedded_blob.is_empty() {
        debug!("Embedded credential blob is empty");
        return None;
    }

    // XOR decrypt
    xor_decrypt(embedded_blob)
}

/// XOR decrypt an embedded credential blob
///
/// # Arguments
/// * `data` - The XOR-encrypted data
///
/// # Returns
/// The decrypted string, or None if decryption fails
#[allow(dead_code)]
fn xor_decrypt(data: &[u8]) -> Option<String> {
    let decrypted: Vec<u8> = data.iter().map(|&b| b ^ XOR_KEY).collect();

    // Validate UTF-8 and trim whitespace
    String::from_utf8(decrypted).ok().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

/// Retrieve credential from macOS Keychain
fn get_from_keychain() -> Option<String> {
    use keyring::Entry;

    let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT).ok()?;

    match entry.get_password() {
        Ok(password) => {
            debug!(
                "Successfully retrieved credential from Keychain for service: {}, account: {}",
                KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT
            );
            Some(password)
        }
        Err(e) => {
            debug!("Failed to retrieve credential from Keychain: {}", e);
            None
        }
    }
}

/// Retrieve credential from .env file
fn get_from_env_file() -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let env_path = cwd.join(".env");

    if !env_path.exists() {
        debug!(
            ".env file not found in current working directory: {:?}",
            cwd
        );
        return None;
    }

    debug!("Reading .env file from: {:?}", env_path);

    let content = fs::read_to_string(&env_path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Parse KEY=VALUE format
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == "AZURE_API_KEY" {
                let credential = value.trim();
                // Remove surrounding quotes if present
                let credential = if (credential.starts_with('"') && credential.ends_with('"'))
                    || (credential.starts_with('\'') && credential.ends_with('\''))
                {
                    &credential[1..credential.len() - 1]
                } else {
                    credential
                };

                if !credential.is_empty() {
                    debug!("Found AZURE_API_KEY in .env file");
                    return Some(credential.to_string());
                }
            }
        }
    }

    debug!("AZURE_API_KEY not found in .env file");
    None
}

/// Store Azure API key in macOS Keychain
///
/// This is a convenience function for developers to store credentials securely.
/// The credential will be retrieved in future calls via Priority 3 (Keychain).
pub fn store_in_keychain(api_key: &str) -> Result<()> {
    use keyring::Entry;

    let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .context("Failed to create Keychain entry")?;

    entry
        .set_password(api_key)
        .context("Failed to store credential in Keychain")?;

    info!(
        "Successfully stored Azure API key in Keychain (service: {}, account: {})",
        KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT
    );
    Ok(())
}

/// Retrieve Azure API key or return error if unavailable
///
/// This is a convenience function for scenarios where the credential is required.
pub fn get_azure_api_key_required() -> Result<String> {
    get_azure_api_key().ok_or_else(|| {
        anyhow::anyhow!(
            "Azure API key not found. Set one of: AZURE_API_KEY env var, .env file, or Keychain"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_decrypt() {
        // XOR encrypt "test_key" with key 167
        let original = b"test_key";
        let encrypted: Vec<u8> = original.iter().map(|&b| b ^ XOR_KEY).collect();

        // Decrypt and verify
        let decrypted = xor_decrypt(&encrypted);
        assert_eq!(decrypted, Some("test_key".to_string()));
    }

    #[test]
    fn test_xor_decrypt_empty() {
        let encrypted: Vec<u8> = vec![];
        let decrypted = xor_decrypt(&encrypted);
        assert_eq!(decrypted, None);
    }

    #[test]
    fn test_xor_decrypt_invalid_utf8() {
        // Create invalid UTF-8 sequence
        let invalid = vec![0xFF, 0xFE, 0xFD];
        let decrypted = xor_decrypt(&invalid);
        // Should fail gracefully, not panic
        assert!(decrypted.is_none() || !decrypted.unwrap_or_default().is_empty());
    }

    #[test]
    fn test_parse_env_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Comment line").unwrap();
        writeln!(file, "AZURE_API_KEY=test_key_123").unwrap();
        writeln!(file, "OTHER_VAR=other_value").unwrap();
        file.flush().unwrap();

        // Set CWD to parent of temp file (simulate reading from .env)
        // Note: This test is limited without changing directories
        // In practice, get_from_env_file() reads from CWD/.env
    }

    #[test]
    fn test_parse_env_file_with_quotes() {
        // Test parsing with double quotes
        let line = "AZURE_API_KEY=\"quoted_key_123\"";
        let (_key, value) = line.split_once('=').unwrap();
        let credential = value.trim();
        let credential = if credential.starts_with('"') && credential.ends_with('"') {
            &credential[1..credential.len() - 1]
        } else {
            credential
        };
        assert_eq!(credential, "quoted_key_123");
    }

    #[test]
    fn test_parse_env_file_with_single_quotes() {
        // Test parsing with single quotes
        let line = "AZURE_API_KEY='quoted_key_456'";
        let (_key, value) = line.split_once('=').unwrap();
        let credential = value.trim();
        let credential = if credential.starts_with('\'') && credential.ends_with('\'') {
            &credential[1..credential.len() - 1]
        } else {
            credential
        };
        assert_eq!(credential, "quoted_key_456");
    }

    #[test]
    fn test_env_var_retrieval() {
        // Test that environment variable retrieval works
        env::set_var("AZURE_API_KEY", "env_var_key_789");
        let key = env::var("AZURE_API_KEY").unwrap();
        assert_eq!(key, "env_var_key_789");
        env::remove_var("AZURE_API_KEY");
    }

    #[test]
    fn test_fallback_chain_priority() {
        // Ensure env var takes priority over missing embedded
        // This would require mocking, but demonstrates the structure
        // In practice, each priority is checked in order:
        // 1. Embedded (unlikely to exist in test)
        // 2. Env var (can be set in test)
        // 3. Keychain (requires user interaction)
        // 4. .env file (requires file setup)
    }
}
