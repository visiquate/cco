//! Linux Secret Service backend implementation

use crate::credentials::keyring::{
    backend::PlatformBackend, Credential, CredentialError, CredentialResult,
};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use tracing::{debug, info, warn};

/// Linux Secret Service backend
pub struct LinuxSecretServiceBackend {
    service_name: String,
}

impl LinuxSecretServiceBackend {
    /// Create a new Linux Secret Service backend
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }
}

#[async_trait]
impl PlatformBackend for LinuxSecretServiceBackend {
    async fn store(&mut self, key: &str, credential: &Credential) -> CredentialResult<()> {
        debug!("Storing credential in Linux Secret Service: {}", key);

        #[cfg(target_os = "linux")]
        {
            // Use the keyring crate which abstracts Secret Service
            keyring::Entry::new(&self.service_name, key)
                .map_err(|e| {
                    CredentialError::BackendUnavailable(format!("Secret Service error: {}", e))
                })?
                .set_password(credential.secret.expose_secret())
                .map_err(|e| CredentialError::EncryptionError(format!("Failed to store: {}", e)))?;

            info!("Stored credential in Secret Service: {}", key);
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on Linux".to_string(),
            ))
        }
    }

    async fn retrieve(&mut self, key: &str) -> CredentialResult<Credential> {
        debug!("Retrieving credential from Linux Secret Service: {}", key);

        #[cfg(target_os = "linux")]
        {
            let entry = keyring::Entry::new(&self.service_name, key).map_err(|e| {
                CredentialError::BackendUnavailable(format!("Secret Service error: {}", e))
            })?;

            let password = entry.get_password().map_err(|e| {
                if e.to_string().contains("not found") {
                    CredentialError::NotFound(key.to_string())
                } else {
                    CredentialError::DecryptionError(format!("Failed to retrieve: {}", e))
                }
            })?;

            // Reconstruct credential from stored data
            let mut credential = Credential::new(key.to_string(), password, Default::default());

            credential.last_accessed = chrono::Utc::now();
            Ok(credential)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on Linux".to_string(),
            ))
        }
    }

    async fn delete(&mut self, key: &str) -> CredentialResult<bool> {
        debug!("Deleting credential from Linux Secret Service: {}", key);

        #[cfg(target_os = "linux")]
        {
            let entry = keyring::Entry::new(&self.service_name, key).map_err(|e| {
                CredentialError::BackendUnavailable(format!("Secret Service error: {}", e))
            })?;

            match entry.delete_password() {
                Ok(_) => {
                    info!("Deleted credential from Secret Service: {}", key);
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

        #[cfg(not(target_os = "linux"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on Linux".to_string(),
            ))
        }
    }

    async fn list(&self) -> CredentialResult<Vec<String>> {
        // Note: Secret Service API doesn't provide a standard way to list items
        // This is a platform limitation
        debug!("Listing credentials from Linux Secret Service");
        warn!("Secret Service does not support listing credentials");
        Ok(Vec::new())
    }

    async fn exists(&self, key: &str) -> CredentialResult<bool> {
        debug!(
            "Checking if credential exists in Linux Secret Service: {}",
            key
        );

        #[cfg(target_os = "linux")]
        {
            let entry = keyring::Entry::new(&self.service_name, key).map_err(|e| {
                CredentialError::BackendUnavailable(format!("Secret Service error: {}", e))
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

        #[cfg(not(target_os = "linux"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on Linux".to_string(),
            ))
        }
    }

    fn backend_name(&self) -> &'static str {
        "Linux Secret Service"
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "linux")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_availability() {
        let backend = LinuxSecretServiceBackend::new("com.test.app");
        #[cfg(target_os = "linux")]
        assert!(backend.is_available());
        #[cfg(not(target_os = "linux"))]
        assert!(!backend.is_available());
    }

    #[test]
    fn test_backend_name() {
        let backend = LinuxSecretServiceBackend::new("com.test.app");
        assert_eq!(backend.backend_name(), "Linux Secret Service");
    }
}
