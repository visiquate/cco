//! macOS Keychain backend implementation

use crate::credentials::keyring::{
    backend::PlatformBackend, Credential, CredentialError, CredentialResult,
};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use tracing::{debug, info};

/// macOS Keychain backend
pub struct MacOSKeychainBackend {
    service_name: String,
}

impl MacOSKeychainBackend {
    /// Create a new macOS Keychain backend
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }
}

#[async_trait]
impl PlatformBackend for MacOSKeychainBackend {
    async fn store(&mut self, key: &str, credential: &Credential) -> CredentialResult<()> {
        debug!("Storing credential in macOS Keychain: {}", key);

        #[cfg(target_os = "macos")]
        {
            // Use the keyring crate for macOS Keychain access
            keyring::Entry::new(&self.service_name, key)
                .map_err(|e| CredentialError::BackendUnavailable(format!("Keychain error: {}", e)))?
                .set_password(credential.secret.expose_secret())
                .map_err(|e| CredentialError::EncryptionError(format!("Failed to store: {}", e)))?;

            info!("Stored credential in Keychain: {}", key);
            Ok(())
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on macOS".to_string(),
            ))
        }
    }

    async fn retrieve(&mut self, key: &str) -> CredentialResult<Credential> {
        debug!("Retrieving credential from macOS Keychain: {}", key);

        #[cfg(target_os = "macos")]
        {
            let entry = keyring::Entry::new(&self.service_name, key).map_err(|e| {
                CredentialError::BackendUnavailable(format!("Keychain error: {}", e))
            })?;

            let password = entry.get_password().map_err(|e| {
                if e.to_string().contains("not found") {
                    CredentialError::NotFound(key.to_string())
                } else {
                    CredentialError::DecryptionError(format!("Failed to retrieve: {}", e))
                }
            })?;

            // Reconstruct credential from stored data
            // Note: metadata is not stored in Keychain, using defaults
            let mut credential = Credential::new(key.to_string(), password, Default::default());

            credential.last_accessed = chrono::Utc::now();
            Ok(credential)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on macOS".to_string(),
            ))
        }
    }

    async fn delete(&mut self, key: &str) -> CredentialResult<bool> {
        debug!("Deleting credential from macOS Keychain: {}", key);

        #[cfg(target_os = "macos")]
        {
            let entry = keyring::Entry::new(&self.service_name, key).map_err(|e| {
                CredentialError::BackendUnavailable(format!("Keychain error: {}", e))
            })?;

            match entry.delete_password() {
                Ok(_) => {
                    info!("Deleted credential from Keychain: {}", key);
                    Ok(true)
                }
                Err(e) => {
                    if e.to_string().contains("not found") {
                        Ok(false)
                    } else {
                        Err(CredentialError::BackendUnavailable(format!(
                            "Delete error: {}",
                            e
                        )))
                    }
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on macOS".to_string(),
            ))
        }
    }

    async fn list(&self) -> CredentialResult<Vec<String>> {
        // Note: macOS Keychain doesn't provide a way to list items by service
        // This is a limitation of the platform API
        debug!("Listing credentials from macOS Keychain");
        Ok(Vec::new())
    }

    async fn exists(&self, key: &str) -> CredentialResult<bool> {
        debug!("Checking if credential exists in macOS Keychain: {}", key);

        #[cfg(target_os = "macos")]
        {
            let entry = keyring::Entry::new(&self.service_name, key).map_err(|e| {
                CredentialError::BackendUnavailable(format!("Keychain error: {}", e))
            })?;

            match entry.get_password() {
                Ok(_) => Ok(true),
                Err(e) => {
                    if e.to_string().contains("not found") {
                        Ok(false)
                    } else {
                        Err(CredentialError::BackendUnavailable(format!(
                            "Check error: {}",
                            e
                        )))
                    }
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on macOS".to_string(),
            ))
        }
    }

    fn backend_name(&self) -> &'static str {
        "macOS Keychain"
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_availability() {
        let backend = MacOSKeychainBackend::new("com.test.app");
        #[cfg(target_os = "macos")]
        assert!(backend.is_available());
        #[cfg(not(target_os = "macos"))]
        assert!(!backend.is_available());
    }

    #[test]
    fn test_backend_name() {
        let backend = MacOSKeychainBackend::new("com.test.app");
        assert_eq!(backend.backend_name(), "macOS Keychain");
    }
}
