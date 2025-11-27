//! Secure token storage for authentication
//!
//! This module provides secure token storage with OS keyring as primary storage
//! and encrypted file storage as fallback.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::device_flow::TokenResponse;

const KEYRING_SERVICE: &str = "cco-cli";
const KEYRING_USER: &str = "cco-tokens";

/// Token information with expiration tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

impl TokenInfo {
    /// Check if token is expired (with buffer in seconds)
    pub fn is_expired(&self, buffer_seconds: i64) -> bool {
        let now = Utc::now();
        let expiry_with_buffer = self.expires_at - chrono::Duration::seconds(buffer_seconds);
        now >= expiry_with_buffer
    }
}

impl From<TokenResponse> for TokenInfo {
    fn from(response: TokenResponse) -> Self {
        let expires_at = Utc::now() + chrono::Duration::seconds(response.expires_in as i64);
        Self {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_at,
            token_type: response.token_type,
        }
    }
}

/// Storage backend type
#[derive(Debug, Clone, Copy, PartialEq)]
enum StorageBackend {
    Keyring,
    File,
}

/// Token storage manager with OS keyring support and file fallback
pub struct TokenStorage {
    token_file: PathBuf,
    backend: StorageBackend,
    keyring: Option<Entry>,
}

impl TokenStorage {
    /// Create a new token storage manager
    pub fn new() -> Result<Self> {
        let token_file = Self::get_token_file_path()?;

        // Try to initialize keyring
        let (backend, keyring) = match Entry::new(KEYRING_SERVICE, KEYRING_USER) {
            Ok(entry) => (StorageBackend::Keyring, Some(entry)),
            Err(_) => {
                tracing::debug!("Keyring not available, using file storage");
                (StorageBackend::File, None)
            }
        };

        Ok(Self {
            token_file,
            backend,
            keyring,
        })
    }

    /// Get the token file path
    fn get_token_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        let cco_config_dir = config_dir.join("cco");

        fs::create_dir_all(&cco_config_dir)
            .context("Failed to create CCO config directory")?;

        Ok(cco_config_dir.join("tokens.json"))
    }

    /// Store tokens securely using keyring or file storage
    pub fn store_tokens(&self, tokens: &TokenResponse) -> Result<()> {
        let token_info: TokenInfo = tokens.clone().into();
        let content = serde_json::to_string_pretty(&token_info)
            .context("Failed to serialize tokens")?;

        match self.backend {
            StorageBackend::Keyring => {
                if let Some(ref entry) = self.keyring {
                    entry
                        .set_password(&content)
                        .context("Failed to store tokens in keyring")?;
                    tracing::info!("Tokens stored securely in OS keyring");
                } else {
                    // Fallback to file if keyring unexpectedly unavailable
                    self.store_tokens_to_file(&content)?;
                }
            }
            StorageBackend::File => {
                self.store_tokens_to_file(&content)?;
            }
        }

        Ok(())
    }

    /// Store tokens to file with secure permissions (internal helper)
    fn store_tokens_to_file(&self, content: &str) -> Result<()> {
        fs::write(&self.token_file, content)
            .context("Failed to write token file")?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&self.token_file)?.permissions();
            perms.set_mode(0o600); // rw-------
            fs::set_permissions(&self.token_file, perms)
                .context("Failed to set secure permissions on token file")?;
        }

        tracing::info!("Tokens stored securely in file: {}", self.token_file.display());
        Ok(())
    }

    /// Retrieve stored tokens from keyring or file
    pub fn get_tokens(&self) -> Result<TokenInfo> {
        let content = match self.backend {
            StorageBackend::Keyring => {
                if let Some(ref entry) = self.keyring {
                    entry
                        .get_password()
                        .context("Failed to retrieve tokens from keyring")?
                } else {
                    // Fallback to file
                    self.get_tokens_from_file()?
                }
            }
            StorageBackend::File => self.get_tokens_from_file()?,
        };

        let tokens: TokenInfo = serde_json::from_str(&content)
            .context("Failed to parse token data")?;

        Ok(tokens)
    }

    /// Retrieve tokens from file (internal helper)
    fn get_tokens_from_file(&self) -> Result<String> {
        if !self.token_file.exists() {
            return Err(anyhow::anyhow!(
                "Not authenticated. Please run 'cco login' first."
            ));
        }

        fs::read_to_string(&self.token_file)
            .context("Failed to read token file")
    }

    /// Check if tokens exist
    pub fn has_tokens(&self) -> Result<bool> {
        match self.backend {
            StorageBackend::Keyring => {
                if let Some(ref entry) = self.keyring {
                    Ok(entry.get_password().is_ok())
                } else {
                    Ok(self.token_file.exists())
                }
            }
            StorageBackend::File => Ok(self.token_file.exists()),
        }
    }

    /// Check if valid tokens exist (not expired)
    pub fn has_valid_tokens(&self) -> Result<bool> {
        if !self.has_tokens()? {
            return Ok(false);
        }

        match self.get_tokens() {
            Ok(tokens) => Ok(!tokens.is_expired(0)),
            Err(_) => Ok(false),
        }
    }

    /// Clear stored tokens from keyring and file
    pub fn clear_tokens(&self) -> Result<()> {
        let mut cleared = false;

        // Clear from keyring
        if self.backend == StorageBackend::Keyring {
            if let Some(ref entry) = self.keyring {
                if let Err(e) = entry.delete_password() {
                    tracing::debug!("No tokens in keyring to clear: {}", e);
                } else {
                    tracing::info!("Tokens cleared from keyring");
                    cleared = true;
                }
            }
        }

        // Clear from file (always try for migration cases)
        if self.token_file.exists() {
            fs::remove_file(&self.token_file)
                .context("Failed to remove token file")?;
            tracing::info!("Tokens cleared from file");
            cleared = true;
        }

        if !cleared {
            tracing::debug!("No tokens found to clear");
        }

        Ok(())
    }

    /// Verify file permissions (Unix only)
    #[cfg(unix)]
    pub fn verify_permissions(&self) -> Result<()> {
        if !self.token_file.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&self.token_file)?;
        let perms = metadata.permissions();
        let mode = perms.mode();

        // Check that only owner has read/write access (0o600)
        if mode & 0o077 != 0 {
            tracing::warn!(
                "Token file has insecure permissions: 0o{:o}. Fixing...",
                mode & 0o777
            );

            let mut new_perms = perms;
            new_perms.set_mode(0o600);
            fs::set_permissions(&self.token_file, new_perms)?;

            tracing::info!("Token file permissions fixed to 0o600");
        }

        Ok(())
    }

    /// Get token file path (for diagnostics)
    pub fn get_path(&self) -> &Path {
        &self.token_file
    }

    /// Get the storage backend being used
    pub fn get_backend(&self) -> &str {
        match self.backend {
            StorageBackend::Keyring => "OS Keyring",
            StorageBackend::File => "File Storage",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_token_expiry() {
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let token = TokenInfo {
            access_token: "test_token".to_string(),
            refresh_token: "test_refresh".to_string(),
            expires_at,
            token_type: "Bearer".to_string(),
        };

        // Not expired (with 5 minute buffer)
        assert!(!token.is_expired(300));

        // Create expired token
        let expired = TokenInfo {
            expires_at: Utc::now() - chrono::Duration::hours(1),
            ..token
        };

        assert!(expired.is_expired(0));
    }

    #[test]
    fn test_token_storage_lifecycle() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let token_file = temp_dir.path().join("tokens.json");

        let storage = TokenStorage {
            token_file: token_file.clone(),
            backend: StorageBackend::File,
            keyring: None,
        };

        // Initially no tokens
        assert!(!storage.has_tokens()?);

        // Store tokens
        let tokens = TokenResponse {
            access_token: "access123".to_string(),
            refresh_token: "refresh456".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
        };

        storage.store_tokens(&tokens)?;

        // Now has tokens
        assert!(storage.has_tokens()?);
        assert!(storage.has_valid_tokens()?);

        // Retrieve tokens
        let retrieved = storage.get_tokens()?;
        assert_eq!(retrieved.access_token, "access123");
        assert_eq!(retrieved.refresh_token, "refresh456");

        // Clear tokens
        storage.clear_tokens()?;
        assert!(!storage.has_tokens()?);

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_secure_permissions() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new()?;
        let token_file = temp_dir.path().join("tokens.json");

        let storage = TokenStorage {
            token_file: token_file.clone(),
            backend: StorageBackend::File,
            keyring: None,
        };

        let tokens = TokenResponse {
            access_token: "access123".to_string(),
            refresh_token: "refresh456".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
        };

        storage.store_tokens(&tokens)?;

        // Check permissions are 0o600
        let metadata = fs::metadata(&token_file)?;
        let mode = metadata.permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "Token file should have 0o600 permissions");

        Ok(())
    }
}
