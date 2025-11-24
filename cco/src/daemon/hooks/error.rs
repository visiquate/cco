//! Error types for the hooks system
//!
//! Provides comprehensive error handling for hook operations
//! with detailed error context and proper error chaining.

use std::time::Duration;
use thiserror::Error;

/// Result type for hook operations
pub type HookResult<T> = Result<T, HookError>;

/// Errors that can occur during hook execution
///
/// All hook errors are recoverable - they should be logged
/// but should not prevent the daemon from continuing operation.
#[derive(Error, Debug)]
pub enum HookError {
    /// Hook execution exceeded the configured timeout
    ///
    /// This indicates the hook took too long to complete.
    /// The hook was terminated to prevent blocking the daemon.
    #[error("Hook execution timed out after {duration:?} (hook: {hook_type})")]
    Timeout {
        /// The hook type that timed out
        hook_type: String,
        /// The timeout duration that was exceeded
        duration: Duration,
    },

    /// Hook execution failed with an error
    ///
    /// The hook callback returned an error or panicked.
    #[error("Hook execution failed: {message} (hook: {hook_type})")]
    ExecutionFailed {
        /// The hook type that failed
        hook_type: String,
        /// Error message from the hook
        message: String,
    },

    /// Invalid hook configuration
    ///
    /// The hook configuration is invalid or inconsistent.
    #[error("Invalid hook configuration: {message}")]
    InvalidConfig {
        /// Description of the configuration error
        message: String,
    },

    /// Hook panicked during execution
    ///
    /// The hook callback panicked. This is caught and
    /// converted to an error to prevent daemon crash.
    #[error("Hook panicked: {message} (hook: {hook_type})")]
    PanicRecovery {
        /// The hook type that panicked
        hook_type: String,
        /// Panic message if available
        message: String,
    },

    /// LLM service is unavailable
    ///
    /// The LLM service required for hook execution is not available.
    /// This may be temporary and can be retried.
    #[error("LLM service unavailable: {reason}")]
    LlmUnavailable {
        /// Reason for LLM unavailability
        reason: String,
    },

    /// Hook registration failed
    ///
    /// Failed to register a hook in the registry.
    #[error("Hook registration failed: {message}")]
    RegistrationFailed {
        /// Error message
        message: String,
    },

    /// Maximum retries exceeded
    ///
    /// The hook failed after all retry attempts.
    #[error("Hook exceeded maximum retries ({max_retries}) (hook: {hook_type})")]
    MaxRetriesExceeded {
        /// The hook type that exceeded retries
        hook_type: String,
        /// Maximum number of retries configured
        max_retries: u32,
    },
}

impl HookError {
    /// Create a timeout error
    pub fn timeout(hook_type: impl Into<String>, duration: Duration) -> Self {
        Self::Timeout {
            hook_type: hook_type.into(),
            duration,
        }
    }

    /// Create an execution failed error
    pub fn execution_failed(hook_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            hook_type: hook_type.into(),
            message: message.into(),
        }
    }

    /// Create an invalid config error
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Create a panic recovery error
    pub fn panic_recovery(hook_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::PanicRecovery {
            hook_type: hook_type.into(),
            message: message.into(),
        }
    }

    /// Create an LLM unavailable error
    pub fn llm_unavailable(reason: impl Into<String>) -> Self {
        Self::LlmUnavailable {
            reason: reason.into(),
        }
    }

    /// Create a registration failed error
    pub fn registration_failed(message: impl Into<String>) -> Self {
        Self::RegistrationFailed {
            message: message.into(),
        }
    }

    /// Create a max retries exceeded error
    pub fn max_retries_exceeded(hook_type: impl Into<String>, max_retries: u32) -> Self {
        Self::MaxRetriesExceeded {
            hook_type: hook_type.into(),
            max_retries,
        }
    }

    /// Check if this error is retryable
    ///
    /// Returns true if the operation that caused this error
    /// can be safely retried.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            HookError::Timeout { .. }
                | HookError::LlmUnavailable { .. }
                | HookError::ExecutionFailed { .. }
        )
    }

    /// Check if this error is a timeout
    pub fn is_timeout(&self) -> bool {
        matches!(self, HookError::Timeout { .. })
    }

    /// Check if this error is a panic
    pub fn is_panic(&self) -> bool {
        matches!(self, HookError::PanicRecovery { .. })
    }

    /// Get the hook type associated with this error, if any
    pub fn hook_type(&self) -> Option<&str> {
        match self {
            HookError::Timeout { hook_type, .. }
            | HookError::ExecutionFailed { hook_type, .. }
            | HookError::PanicRecovery { hook_type, .. }
            | HookError::MaxRetriesExceeded { hook_type, .. } => Some(hook_type),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_error() {
        let err = HookError::timeout("test_hook", Duration::from_secs(5));
        assert!(err.is_timeout());
        assert!(err.is_retryable());
        assert_eq!(err.hook_type(), Some("test_hook"));
    }

    #[test]
    fn test_execution_failed_error() {
        let err = HookError::execution_failed("test_hook", "test error");
        assert!(err.is_retryable());
        assert_eq!(err.hook_type(), Some("test_hook"));
    }

    #[test]
    fn test_invalid_config_error() {
        let err = HookError::invalid_config("invalid timeout");
        assert!(!err.is_retryable());
        assert_eq!(err.hook_type(), None);
    }

    #[test]
    fn test_panic_recovery_error() {
        let err = HookError::panic_recovery("test_hook", "panic message");
        assert!(err.is_panic());
        assert_eq!(err.hook_type(), Some("test_hook"));
    }

    #[test]
    fn test_llm_unavailable_error() {
        let err = HookError::llm_unavailable("service down");
        assert!(err.is_retryable());
        assert_eq!(err.hook_type(), None);
    }

    #[test]
    fn test_max_retries_exceeded() {
        let err = HookError::max_retries_exceeded("test_hook", 3);
        assert!(!err.is_retryable());
        assert_eq!(err.hook_type(), Some("test_hook"));
    }

    #[test]
    fn test_error_display() {
        let err = HookError::timeout("test_hook", Duration::from_secs(5));
        let display = format!("{}", err);
        assert!(display.contains("timed out"));
        assert!(display.contains("test_hook"));
    }
}
