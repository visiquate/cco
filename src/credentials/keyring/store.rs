//! Main credential store implementation

use crate::credentials::keyring::{
    audit::{AuditAction, AuditLogger},
    backend::PlatformBackend,
    models::{Credential, CredentialMetadata},
    platform::create_backend,
    CredentialError, CredentialResult,
};
use secrecy::SecretString;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Main credential store
pub struct CredentialStore {
    backend: Box<dyn PlatformBackend>,
    audit_logger: Option<Arc<AuditLogger>>,
    rate_limiter: Arc<RateLimiter>,
}

impl CredentialStore {
    /// Create a new credential store with platform detection
    ///
    /// # Arguments
    ///
    /// * `audit_enabled` - Whether to enable audit logging
    ///
    /// # Returns
    ///
    /// * `CredentialResult<Self>` - Initialized store or error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let store = cco::credentials::keyring::CredentialStore::new(true)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(audit_enabled: bool) -> CredentialResult<Self> {
        let backend = create_backend();
        Self::with_backend(backend, audit_enabled)
    }

    /// Create store with explicit backend (for testing)
    pub fn with_backend(
        backend: Box<dyn PlatformBackend>,
        audit_enabled: bool,
    ) -> CredentialResult<Self> {
        info!(
            "Initializing credential store with backend: {}",
            backend.backend_name()
        );

        let audit_logger = if audit_enabled {
            Some(Arc::new(AuditLogger::new()?))
        } else {
            None
        };

        let rate_limiter = Arc::new(RateLimiter::new(10, 60)); // 10 attempts per 60 seconds

        Ok(Self {
            backend,
            audit_logger,
            rate_limiter,
        })
    }

    /// Store a credential securely
    ///
    /// # Arguments
    ///
    /// * `credential` - The credential to store
    ///
    /// # Returns
    ///
    /// * `CredentialResult<()>` - Success or error
    pub async fn store_credential(&mut self, credential: Credential) -> CredentialResult<()> {
        debug!("Storing credential: {}", credential.key);

        // Store via backend
        let result = self.backend.store(&credential.key, &credential).await;

        // Audit log
        if let Some(audit) = &self.audit_logger {
            match &result {
                Ok(_) => {
                    audit
                        .log_access(&credential.key, AuditAction::Store, true, None)
                        .await?;
                }
                Err(e) => {
                    audit
                        .log_access(
                            &credential.key,
                            AuditAction::Store,
                            false,
                            Some(&e.to_string()),
                        )
                        .await?;
                }
            }
        }

        result?;
        info!("Successfully stored credential: {}", credential.key);
        Ok(())
    }

    /// Retrieve a credential by key
    ///
    /// # Arguments
    ///
    /// * `key` - The credential key to retrieve
    ///
    /// # Returns
    ///
    /// * `CredentialResult<Credential>` - The credential or error
    pub async fn retrieve_credential(&mut self, key: &str) -> CredentialResult<Credential> {
        debug!("Retrieving credential: {}", key);

        // Rate limiting
        self.rate_limiter.check_rate_limit(key)?;

        // Retrieve from backend
        let result = self.backend.retrieve(key).await;

        // Audit log
        if let Some(audit) = &self.audit_logger {
            match &result {
                Ok(_) => {
                    audit
                        .log_access(key, AuditAction::Retrieve, true, None)
                        .await?;
                }
                Err(e) => {
                    audit
                        .log_access(key, AuditAction::Retrieve, false, Some(&e.to_string()))
                        .await?;
                }
            }
        }

        let credential = result?;

        // Check expiration
        if credential.is_expired() {
            warn!("Credential expired: {}", key);
            return Err(CredentialError::CredentialExpired(key.to_string()));
        }

        Ok(credential)
    }

    /// Delete a credential
    ///
    /// # Arguments
    ///
    /// * `key` - The credential key to delete
    ///
    /// # Returns
    ///
    /// * `CredentialResult<bool>` - true if deleted, false if not found
    pub async fn delete_credential(&mut self, key: &str) -> CredentialResult<bool> {
        debug!("Deleting credential: {}", key);

        let result = self.backend.delete(key).await;

        // Audit log
        if let Some(audit) = &self.audit_logger {
            match &result {
                Ok(deleted) => {
                    audit
                        .log_access(key, AuditAction::Delete, *deleted, None)
                        .await?;
                }
                Err(e) => {
                    audit
                        .log_access(key, AuditAction::Delete, false, Some(&e.to_string()))
                        .await?;
                }
            }
        }

        result
    }

    /// List all credential keys (not secrets)
    pub async fn list_credentials(&self) -> CredentialResult<Vec<String>> {
        self.backend.list().await
    }

    /// Check if a credential exists
    pub async fn credential_exists(&self, key: &str) -> CredentialResult<bool> {
        self.backend.exists(key).await
    }

    /// Get credentials that need rotation
    pub async fn check_rotation_needed(&mut self) -> CredentialResult<Vec<String>> {
        let keys = self.backend.list().await?;
        let mut needs_rotation = Vec::new();

        for key in keys {
            if let Ok(credential) = self.backend.retrieve(&key).await {
                if credential.needs_rotation() {
                    needs_rotation.push(key);
                }
            }
        }

        Ok(needs_rotation)
    }

    /// Get expired credentials
    pub async fn get_expired_credentials(&mut self) -> CredentialResult<Vec<String>> {
        let keys = self.backend.list().await?;
        let mut expired = Vec::new();

        for key in keys {
            if let Ok(credential) = self.backend.retrieve(&key).await {
                if credential.is_expired() {
                    expired.push(key);
                }
            }
        }

        Ok(expired)
    }

    /// Update credential metadata (without changing secret)
    pub async fn update_metadata(
        &mut self,
        key: &str,
        metadata: CredentialMetadata,
    ) -> CredentialResult<()> {
        let mut credential = self.backend.retrieve(key).await?;
        credential.metadata = metadata;
        self.backend.store(key, &credential).await?;

        if let Some(audit) = &self.audit_logger {
            audit
                .log_access(key, AuditAction::Update, true, None)
                .await?;
        }

        Ok(())
    }

    /// Rotate a credential (update secret, track rotation)
    pub async fn rotate_credential(
        &mut self,
        key: &str,
        new_secret: SecretString,
    ) -> CredentialResult<()> {
        let mut credential = self.backend.retrieve(key).await?;
        credential.secret = new_secret;
        credential.last_rotated = Some(chrono::Utc::now());
        self.backend.store(key, &credential).await?;

        if let Some(audit) = &self.audit_logger {
            audit
                .log_access(key, AuditAction::Rotate, true, None)
                .await?;
        }

        info!("Rotated credential: {}", key);
        Ok(())
    }
}

/// Rate limiter for credential access
pub struct RateLimiter {
    attempts: Arc<Mutex<HashMap<String, AttemptTracker>>>,
    max_attempts: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window_secs,
        }
    }

    pub fn check_rate_limit(&self, key: &str) -> CredentialResult<()> {
        let mut attempts = self.attempts.lock().unwrap();
        let tracker = attempts
            .entry(key.to_string())
            .or_insert_with(AttemptTracker::new);

        if tracker.is_rate_limited(self.max_attempts, self.window_secs) {
            warn!("Rate limit exceeded for key: {}", key);
            return Err(CredentialError::RateLimitExceeded(key.to_string()));
        }

        tracker.record_attempt();
        Ok(())
    }
}

struct AttemptTracker {
    attempts: Vec<Instant>,
}

impl AttemptTracker {
    fn new() -> Self {
        Self {
            attempts: Vec::new(),
        }
    }

    fn record_attempt(&mut self) {
        self.attempts.push(Instant::now());
    }

    fn is_rate_limited(&mut self, max_attempts: u32, window_secs: u64) -> bool {
        let cutoff = Instant::now() - Duration::from_secs(window_secs);

        // Remove old attempts
        self.attempts.retain(|&instant| instant > cutoff);

        self.attempts.len() >= max_attempts as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 1);

        // First 3 attempts should succeed
        assert!(limiter.check_rate_limit("test").is_ok());
        assert!(limiter.check_rate_limit("test").is_ok());
        assert!(limiter.check_rate_limit("test").is_ok());

        // 4th attempt should fail
        assert!(limiter.check_rate_limit("test").is_err());
    }
}
