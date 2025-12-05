//! Platform backend trait definition

use crate::credentials::keyring::{Credential, CredentialResult};
use async_trait::async_trait;

/// Platform-specific credential storage backend
///
/// This trait defines the interface that all platform-specific backends
/// must implement. Implementations include:
///
/// - **MacOSKeychainBackend**: macOS Keychain Access
/// - **LinuxSecretServiceBackend**: Linux Secret Service API
/// - **WindowsDpapiBackend**: Windows DPAPI
/// - **EncryptedFileBackend**: FIPS-compliant encrypted file fallback
#[async_trait]
pub trait PlatformBackend: Send + Sync {
    /// Store a credential
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the credential
    /// * `credential` - The credential to store
    ///
    /// # Returns
    ///
    /// * `Ok(())` if successful
    /// * `Err(CredentialError)` if storage failed
    async fn store(&mut self, key: &str, credential: &Credential) -> CredentialResult<()>;

    /// Retrieve a credential by key
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the credential
    ///
    /// # Returns
    ///
    /// * `Ok(Credential)` if found
    /// * `Err(CredentialError::NotFound)` if not found
    /// * `Err(CredentialError)` for other errors
    async fn retrieve(&mut self, key: &str) -> CredentialResult<Credential>;

    /// Delete a credential
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the credential
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if deleted
    /// * `Ok(false)` if not found
    /// * `Err(CredentialError)` if deletion failed
    async fn delete(&mut self, key: &str) -> CredentialResult<bool>;

    /// List all credential keys (not secrets)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of credential keys
    /// * `Err(CredentialError)` if listing failed
    async fn list(&self) -> CredentialResult<Vec<String>>;

    /// Check if a credential exists
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the credential
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if exists
    /// * `Ok(false)` if not found
    async fn exists(&self, key: &str) -> CredentialResult<bool>;

    /// Get backend name for logging
    fn backend_name(&self) -> &'static str;

    /// Check if backend is available on this platform
    fn is_available(&self) -> bool;

    /// Initialize backend (called once on first use)
    async fn initialize(&mut self) -> CredentialResult<()> {
        Ok(())
    }

    /// Perform health check
    async fn health_check(&self) -> CredentialResult<bool> {
        Ok(self.is_available())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::keyring::CredentialMetadata;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock backend for testing
    struct MockBackend {
        storage: Arc<Mutex<HashMap<String, Credential>>>,
    }

    impl MockBackend {
        fn new() -> Self {
            Self {
                storage: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl PlatformBackend for MockBackend {
        async fn store(&mut self, key: &str, credential: &Credential) -> CredentialResult<()> {
            let mut storage = self.storage.lock().unwrap();
            storage.insert(key.to_string(), credential.clone());
            Ok(())
        }

        async fn retrieve(&mut self, key: &str) -> CredentialResult<Credential> {
            let storage = self.storage.lock().unwrap();
            storage.get(key).cloned().ok_or_else(|| {
                crate::credentials::keyring::CredentialError::NotFound(key.to_string())
            })
        }

        async fn delete(&mut self, key: &str) -> CredentialResult<bool> {
            let mut storage = self.storage.lock().unwrap();
            Ok(storage.remove(key).is_some())
        }

        async fn list(&self) -> CredentialResult<Vec<String>> {
            let storage = self.storage.lock().unwrap();
            Ok(storage.keys().cloned().collect())
        }

        async fn exists(&self, key: &str) -> CredentialResult<bool> {
            let storage = self.storage.lock().unwrap();
            Ok(storage.contains_key(key))
        }

        fn backend_name(&self) -> &'static str {
            "Mock Backend"
        }

        fn is_available(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_mock_backend() {
        let mut backend = MockBackend::new();

        let credential = Credential::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            CredentialMetadata::default(),
        );

        // Store
        backend.store("test_key", &credential).await.unwrap();

        // Exists
        assert!(backend.exists("test_key").await.unwrap());

        // Retrieve
        let retrieved = backend.retrieve("test_key").await.unwrap();
        assert_eq!(retrieved.key, "test_key");

        // List
        let keys = backend.list().await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "test_key");

        // Delete
        assert!(backend.delete("test_key").await.unwrap());
        assert!(!backend.exists("test_key").await.unwrap());
    }
}
