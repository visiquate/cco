//! Security module for CCO Daemon
//!
//! Provides authentication, authorization, input validation, credential detection,
//! rate limiting, and Azure credential retrieval for the daemon HTTP API.

pub mod auth;
pub mod azure_credential;
pub mod credential_detector;
pub mod rate_limiter;
pub mod validation;

pub use auth::{AuthMiddleware, Token, TokenManager};
pub use azure_credential::{get_azure_api_key, get_azure_api_key_required, store_in_keychain};
pub use credential_detector::{CredentialDetector, CredentialMatch};
pub use rate_limiter::{RateLimitError, RateLimiter};
pub use validation::{ValidatedMetadata, ValidationError};
