//! Error types for credential operations

use thiserror::Error;

/// Errors that can occur during credential operations
#[derive(Error, Debug)]
pub enum CredentialError {
    /// Credential not found
    #[error("Credential not found: {0}")]
    NotFound(String),

    /// Backend unavailable or failed to initialize
    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),

    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded for key: {0}")]
    RateLimitExceeded(String),

    /// Credential has expired
    #[error("Credential expired: {0}")]
    CredentialExpired(String),

    /// Invalid credential format
    #[error("Invalid credential format: {0}")]
    InvalidFormat(String),

    /// FIPS mode not available
    #[error("FIPS 140-2 mode is not available or not enabled")]
    FipsNotAvailable,

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Generic error
    #[error("Error: {0}")]
    Other(String),
}

/// Result type for credential operations
pub type CredentialResult<T> = Result<T, CredentialError>;

impl CredentialError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::RateLimitExceeded(_) | Self::BackendUnavailable(_)
        )
    }

    /// Check if error is a "not found" error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if error is security-related
    pub fn is_security_error(&self) -> bool {
        matches!(
            self,
            Self::EncryptionError(_)
                | Self::DecryptionError(_)
                | Self::PermissionDenied(_)
                | Self::FipsNotAvailable
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let err = CredentialError::NotFound("test".to_string());
        assert!(err.is_not_found());
        assert!(!err.is_recoverable());

        let err = CredentialError::RateLimitExceeded("test".to_string());
        assert!(err.is_recoverable());
        assert!(!err.is_not_found());

        let err = CredentialError::EncryptionError("test".to_string());
        assert!(err.is_security_error());
    }

    #[test]
    fn test_error_display() {
        let err = CredentialError::NotFound("my_key".to_string());
        assert_eq!(err.to_string(), "Credential not found: my_key");
    }
}
