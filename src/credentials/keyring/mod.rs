//! CCO Keyring Credential System
//!
//! Provides secure, cross-platform credential storage with native OS keyring
//! integration and FIPS 140-2 compliant encryption fallback.
//!
//! # Platform Support
//!
//! - **macOS**: Keychain Access integration
//! - **Linux**: Secret Service API (GNOME Keyring, KWallet)
//! - **Windows**: DPAPI (Data Protection API)
//! - **Fallback**: AES-256-GCM encrypted file storage
//!
//! # Example
//!
//! ```rust,no_run
//! use cco::credentials::keyring::{CredentialStore, Credential, CredentialMetadata, CredentialType};
//! use secrecy::SecretString;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create credential store
//!     let mut store = CredentialStore::new(true)?;
//!
//!     // Store a credential
//!     let credential = Credential {
//!         key: "salesforce_api_token".to_string(),
//!         secret: SecretString::new("sk_live_...".to_string()),
//!         created_at: chrono::Utc::now(),
//!         expires_at: None,
//!         last_accessed: chrono::Utc::now(),
//!         last_rotated: None,
//!         metadata: CredentialMetadata {
//!             service: "salesforce".to_string(),
//!             credential_type: CredentialType::ApiKey,
//!             description: "Production API token".to_string(),
//!             ..Default::default()
//!         },
//!     };
//!
//!     store.store_credential(credential).await?;
//!
//!     // Retrieve credential
//!     let retrieved = store.retrieve_credential("salesforce_api_token").await?;
//!     println!("Secret retrieved (length: {})", retrieved.secret.expose_secret().len());
//!
//!     Ok(())
//! }
//! ```

pub mod audit;
pub mod backend;
pub mod error;
pub mod models;
pub mod platform;
pub mod store;

pub use audit::{AuditAction, AuditEntry, AuditLogger};
pub use backend::PlatformBackend;
pub use error::{CredentialError, CredentialResult};
pub use models::{Credential, CredentialMetadata, CredentialType};
pub use store::CredentialStore;
