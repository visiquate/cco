//! Windows DPAPI backend implementation

use crate::credentials::keyring::{
    backend::PlatformBackend, Credential, CredentialError, CredentialResult,
};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use std::path::PathBuf;
use tracing::{debug, info};

/// Windows DPAPI backend
pub struct WindowsDpapiBackend {
    service_name: String,
    credential_dir: PathBuf,
}

impl WindowsDpapiBackend {
    /// Create a new Windows DPAPI backend
    pub fn new(service_name: &str) -> CredentialResult<Self> {
        let credential_dir = dirs::data_local_dir()
            .ok_or(CredentialError::BackendUnavailable(
                "Could not find data directory".to_string(),
            ))?
            .join("CCO")
            .join("credentials");

        std::fs::create_dir_all(&credential_dir)?;

        Ok(Self {
            service_name: service_name.to_string(),
            credential_dir,
        })
    }

    fn credential_path(&self, key: &str) -> PathBuf {
        self.credential_dir.join(format!("{}.dpapi", key))
    }
}

#[async_trait]
impl PlatformBackend for WindowsDpapiBackend {
    async fn store(&mut self, key: &str, credential: &Credential) -> CredentialResult<()> {
        debug!("Storing credential with Windows DPAPI: {}", key);

        #[cfg(target_os = "windows")]
        {
            // Use the keyring crate which abstracts Windows DPAPI
            keyring::Entry::new(&self.service_name, key)
                .map_err(|e| CredentialError::BackendUnavailable(format!("DPAPI error: {}", e)))?
                .set_password(credential.secret.expose_secret())
                .map_err(|e| CredentialError::EncryptionError(format!("Failed to store: {}", e)))?;

            info!("Stored credential with DPAPI: {}", key);
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on Windows".to_string(),
            ))
        }
    }

    async fn retrieve(&mut self, key: &str) -> CredentialResult<Credential> {
        debug!("Retrieving credential from Windows DPAPI: {}", key);

        #[cfg(target_os = "windows")]
        {
            let entry = keyring::Entry::new(&self.service_name, key)
                .map_err(|e| CredentialError::BackendUnavailable(format!("DPAPI error: {}", e)))?;

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

        #[cfg(not(target_os = "windows"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on Windows".to_string(),
            ))
        }
    }

    async fn delete(&mut self, key: &str) -> CredentialResult<bool> {
        debug!("Deleting credential from Windows DPAPI: {}", key);

        #[cfg(target_os = "windows")]
        {
            let entry = keyring::Entry::new(&self.service_name, key)
                .map_err(|e| CredentialError::BackendUnavailable(format!("DPAPI error: {}", e)))?;

            match entry.delete_password() {
                Ok(_) => {
                    info!("Deleted credential from DPAPI: {}", key);
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

        #[cfg(not(target_os = "windows"))]
        {
            Err(CredentialError::BackendUnavailable(
                "Not running on Windows".to_string(),
            ))
        }
    }

    async fn list(&self) -> CredentialResult<Vec<String>> {
        debug!("Listing credentials from Windows DPAPI");

        #[cfg(target_os = "windows")]
        {
            let entries = std::fs::read_dir(&self.credential_dir)?;
            let keys = entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.extension() == Some(std::ffi::OsStr::new("dpapi")) {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            Ok(keys)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(Vec::new())
        }
    }

    async fn exists(&self, key: &str) -> CredentialResult<bool> {
        debug!("Checking if credential exists in Windows DPAPI: {}", key);

        #[cfg(target_os = "windows")]
        {
            let entry = keyring::Entry::new(&self.service_name, key)
                .map_err(|e| CredentialError::BackendUnavailable(format!("DPAPI error: {}", e)))?;

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

        #[cfg(not(target_os = "windows"))]
        {
            Ok(false)
        }
    }

    fn backend_name(&self) -> &'static str {
        "Windows DPAPI"
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "windows")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_availability() {
        let backend = WindowsDpapiBackend::new("com.test.app");
        match backend {
            Ok(b) => {
                #[cfg(target_os = "windows")]
                assert!(b.is_available());
                #[cfg(not(target_os = "windows"))]
                assert!(!b.is_available());
            }
            Err(_) => {
                // Backend creation might fail on non-Windows systems
                #[cfg(not(target_os = "windows"))]
                {} // Expected
            }
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_backend_name() {
        let backend = WindowsDpapiBackend::new("com.test.app").unwrap();
        assert_eq!(backend.backend_name(), "Windows DPAPI");
    }
}
