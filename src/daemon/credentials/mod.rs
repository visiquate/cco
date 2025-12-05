//! Credential management API for the daemon
//!
//! Provides HTTP endpoints for secure credential storage and retrieval using
//! the native platform keyring system.

pub mod api;

pub use api::{
    credentials_router, credentials_router_without_state, CredentialState,
    ListCredentialsResponse, RetrieveCredentialResponse, RotationCheckResponse,
    StoreCredentialRequest, StoreCredentialResponse,
};
