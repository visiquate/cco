//! Credential management module
//!
//! Provides secure credential storage and retrieval functionality.

pub mod keyring;

pub use keyring::{
    Credential, CredentialError, CredentialMetadata, CredentialResult, CredentialStore,
    CredentialType,
};
