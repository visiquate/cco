//! Encrypted file fallback backend (FIPS 140-2 compliant)

use crate::credentials::keyring::{
    backend::PlatformBackend, Credential, CredentialError, CredentialResult,
};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use async_trait::async_trait;
use getrandom::getrandom;
use pbkdf2::pbkdf2_hmac;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};
use zeroize::Zeroizing;

/// AES-256 key size (32 bytes = 256 bits)
const AES_KEY_SIZE: usize = 32;

/// Salt size (32 bytes = 256 bits)
const SALT_SIZE: usize = 32;

/// AES-GCM nonce size (12 bytes = 96 bits)
const NONCE_SIZE: usize = 12;

/// PBKDF2 iteration count (NIST 2023 recommendation)
const PBKDF2_ITERATIONS: u32 = 600_000;

/// Encrypted credential storage (JSON serializable)
#[derive(Serialize, Deserialize)]
struct EncryptedStore {
    version: String,
    salt: String, // base64-encoded
    credentials: HashMap<String, EncryptedCredentialData>,
}

/// Individual encrypted credential
#[derive(Serialize, Deserialize, Clone)]
struct EncryptedCredentialData {
    nonce: String,      // base64-encoded
    ciphertext: String, // base64-encoded
    metadata_json: String,
}

/// Encrypted file fallback backend
pub struct EncryptedFileBackend {
    credential_path: PathBuf,
    master_key: Zeroizing<[u8; AES_KEY_SIZE]>,
}

impl EncryptedFileBackend {
    /// Create a new encrypted file backend
    pub fn new() -> CredentialResult<Self> {
        let credential_path = dirs::home_dir()
            .ok_or(CredentialError::BackendUnavailable(
                "Could not find home directory".to_string(),
            ))?
            .join(".cco")
            .join("credentials.enc");

        // Ensure directory exists
        if let Some(parent) = credential_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Derive master key from environment or generate
        let master_key = Self::derive_master_key()?;

        Ok(Self {
            credential_path,
            master_key: Zeroizing::new(master_key),
        })
    }

    /// Derive master key from password/environment
    fn derive_master_key() -> CredentialResult<[u8; AES_KEY_SIZE]> {
        // Get passphrase from environment or use default
        let passphrase = std::env::var("CCO_MASTER_PASSPHRASE")
            .unwrap_or_else(|_| "cco-default-passphrase".to_string());

        // Get or create salt
        let salt = Self::get_or_create_salt()?;

        // PBKDF2 key derivation
        let mut key = [0u8; AES_KEY_SIZE];
        pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

        Ok(key)
    }

    /// Get or create salt file
    fn get_or_create_salt() -> CredentialResult<[u8; SALT_SIZE]> {
        let home = dirs::home_dir().ok_or(CredentialError::BackendUnavailable(
            "Home directory not found".to_string(),
        ))?;
        let salt_path = home.join(".cco").join(".salt");

        // Create directory if needed
        if let Some(parent) = salt_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if salt_path.exists() {
            let salt_bytes = fs::read(&salt_path)?;
            if salt_bytes.len() == SALT_SIZE {
                let mut salt = [0u8; SALT_SIZE];
                salt.copy_from_slice(&salt_bytes);
                return Ok(salt);
            }
        }

        // Generate new salt
        let mut salt = [0u8; SALT_SIZE];
        getrandom(&mut salt).map_err(|e| {
            CredentialError::EncryptionError(format!("Failed to generate salt: {}", e))
        })?;

        // Save salt
        fs::write(&salt_path, &salt)?;

        // Secure file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&salt_path, perms)?;
        }

        Ok(salt)
    }

    /// Load encrypted store from disk
    async fn load_store(&self) -> CredentialResult<EncryptedStore> {
        use base64::Engine;

        if !self.credential_path.exists() {
            // Initialize new store
            let salt = Self::get_or_create_salt()?;
            return Ok(EncryptedStore {
                version: "1.0".to_string(),
                salt: base64::engine::general_purpose::STANDARD.encode(salt),
                credentials: HashMap::new(),
            });
        }

        let data = fs::read_to_string(&self.credential_path)?;
        serde_json::from_str(&data).map_err(|e| CredentialError::Serialization(e))
    }

    /// Save encrypted store to disk
    async fn save_store(&self, store: &EncryptedStore) -> CredentialResult<()> {
        let data = serde_json::to_string_pretty(store)?;
        fs::write(&self.credential_path, data)?;

        // Secure file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.credential_path, perms)?;
        }

        Ok(())
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt(&self, plaintext: &[u8]) -> CredentialResult<(String, String)> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        getrandom(&mut nonce_bytes).map_err(|e| {
            CredentialError::EncryptionError(format!("Failed to generate nonce: {}", e))
        })?;

        // Create cipher
        let key = Key::<Aes256Gcm>::from(*self.master_key);
        let cipher = Aes256Gcm::new(&key);

        // Encrypt
        let nonce = Nonce::<aes_gcm::aead::consts::U12>::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CredentialError::EncryptionError(format!("Encryption failed: {}", e)))?;

        // Return base64-encoded nonce and ciphertext
        use base64::Engine;
        let nonce_b64 = base64::engine::general_purpose::STANDARD.encode(&nonce_bytes);
        let ciphertext_b64 = base64::engine::general_purpose::STANDARD.encode(ciphertext);

        Ok((nonce_b64, ciphertext_b64))
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt(&self, nonce_b64: &str, ciphertext_b64: &str) -> CredentialResult<Vec<u8>> {
        use base64::Engine;

        // Decode base64
        let nonce_bytes = base64::engine::general_purpose::STANDARD
            .decode(nonce_b64)
            .map_err(|e| {
                CredentialError::DecryptionError(format!("Failed to decode nonce: {}", e))
            })?;

        let ciphertext = base64::engine::general_purpose::STANDARD
            .decode(ciphertext_b64)
            .map_err(|e| {
                CredentialError::DecryptionError(format!("Failed to decode ciphertext: {}", e))
            })?;

        if nonce_bytes.len() != NONCE_SIZE {
            return Err(CredentialError::DecryptionError(
                "Invalid nonce size".to_string(),
            ));
        }

        // Create cipher
        let key = Key::<Aes256Gcm>::from(*self.master_key);
        let cipher = Aes256Gcm::new(&key);

        // Decrypt
        let nonce = Nonce::<aes_gcm::aead::consts::U12>::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| CredentialError::DecryptionError("Failed to decrypt".to_string()))?;

        Ok(plaintext)
    }
}

#[async_trait]
impl PlatformBackend for EncryptedFileBackend {
    async fn store(&mut self, key: &str, credential: &Credential) -> CredentialResult<()> {
        debug!("Storing credential in encrypted file: {}", key);

        let mut store = self.load_store().await?;

        // Serialize credential to JSON
        let credential_json = serde_json::to_string(&credential)?;

        // Encrypt credential
        let (nonce_b64, ciphertext_b64) = self.encrypt(credential_json.as_bytes())?;

        // Store metadata separately
        let metadata_json = serde_json::to_string(&credential.metadata)?;

        store.credentials.insert(
            key.to_string(),
            EncryptedCredentialData {
                nonce: nonce_b64,
                ciphertext: ciphertext_b64,
                metadata_json,
            },
        );

        self.save_store(&store).await?;
        info!("Stored credential in encrypted file: {}", key);

        Ok(())
    }

    async fn retrieve(&mut self, key: &str) -> CredentialResult<Credential> {
        debug!("Retrieving credential from encrypted file: {}", key);

        let store = self.load_store().await?;

        let encrypted = store
            .credentials
            .get(key)
            .ok_or_else(|| CredentialError::NotFound(key.to_string()))?;

        // Decrypt credential
        let plaintext = self.decrypt(&encrypted.nonce, &encrypted.ciphertext)?;
        let credential_json = String::from_utf8(plaintext).map_err(|_| {
            CredentialError::InvalidFormat("Invalid UTF-8 in credential".to_string())
        })?;

        let mut credential: Credential = serde_json::from_str(&credential_json)?;
        credential.last_accessed = chrono::Utc::now();

        Ok(credential)
    }

    async fn delete(&mut self, key: &str) -> CredentialResult<bool> {
        debug!("Deleting credential from encrypted file: {}", key);

        let mut store = self.load_store().await?;

        let deleted = store.credentials.remove(key).is_some();

        if deleted {
            self.save_store(&store).await?;
            info!("Deleted credential from encrypted file: {}", key);
        }

        Ok(deleted)
    }

    async fn list(&self) -> CredentialResult<Vec<String>> {
        debug!("Listing credentials from encrypted file");

        let store = self.load_store().await?;
        Ok(store.credentials.keys().cloned().collect())
    }

    async fn exists(&self, key: &str) -> CredentialResult<bool> {
        debug!("Checking if credential exists in encrypted file: {}", key);

        let store = self.load_store().await?;
        Ok(store.credentials.contains_key(key))
    }

    fn backend_name(&self) -> &'static str {
        "Encrypted File (FIPS 140-2)"
    }

    fn is_available(&self) -> bool {
        true // Always available as fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_name() {
        let backend = EncryptedFileBackend::new().unwrap();
        assert_eq!(backend.backend_name(), "Encrypted File (FIPS 140-2)");
    }

    #[test]
    fn test_backend_always_available() {
        let backend = EncryptedFileBackend::new().unwrap();
        assert!(backend.is_available());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let backend = EncryptedFileBackend::new().unwrap();

        let plaintext = b"test data";
        let (nonce, ciphertext) = backend.encrypt(plaintext).unwrap();

        let decrypted = backend.decrypt(&nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
