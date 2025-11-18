//! Security module for CCO Daemon
//!
//! Provides authentication, authorization, input validation, credential detection,
//! and rate limiting for the daemon HTTP API.

pub mod auth;
pub mod credential_detector;
pub mod rate_limiter;
pub mod validation;

pub use auth::{TokenManager, Token, AuthMiddleware};
pub use credential_detector::{CredentialDetector, CredentialMatch};
pub use rate_limiter::{RateLimiter, RateLimitError};
pub use validation::{ValidatedMetadata, ValidationError};
