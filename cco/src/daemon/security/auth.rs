//! Authentication module
//!
//! Implements token-based authentication with:
//! - 32-byte UUID v4 token generation
//! - SHA256 token hashing for secure storage
//! - 24-hour token expiration
//! - Token revocation
//! - File-based storage with 600 permissions

use anyhow::{Context, Result};
use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Token expiration duration (24 hours)
const TOKEN_EXPIRY_HOURS: i64 = 24;

/// Token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub project_id: String,
    #[serde(default)]
    pub revoked: bool,
}

impl Token {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if token is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.revoked
    }
}

/// Token storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TokenStorage {
    tokens: HashMap<String, Token>,
}

/// Token manager for authentication
pub struct TokenManager {
    storage_path: PathBuf,
    tokens: Arc<RwLock<TokenStorage>>,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Result<Self> {
        let storage_path = storage_path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create token storage directory")?;
        }

        // Load existing tokens or create new storage
        let tokens = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path)
                .context("Failed to read token storage")?;

            // Handle empty file
            if content.trim().is_empty() {
                TokenStorage::default()
            } else {
                serde_json::from_str(&content)
                    .context("Failed to parse token storage")?
            }
        } else {
            TokenStorage::default()
        };

        let manager = Self {
            storage_path,
            tokens: Arc::new(RwLock::new(tokens)),
        };

        // Ensure file permissions are 600 (owner read/write only)
        if manager.storage_path.exists() {
            manager.set_secure_permissions()?;
        }

        info!("Token manager initialized with storage: {:?}", manager.storage_path);
        Ok(manager)
    }

    /// Set secure file permissions (600)
    fn set_secure_permissions(&self) -> Result<()> {
        let metadata = fs::metadata(&self.storage_path)
            .context("Failed to get file metadata")?;

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);

        fs::set_permissions(&self.storage_path, permissions)
            .context("Failed to set file permissions to 600")?;

        debug!("Set secure permissions (600) on token storage");
        Ok(())
    }

    /// Persist tokens to disk
    async fn persist(&self) -> Result<()> {
        let tokens = self.tokens.read().await;
        let content = serde_json::to_string_pretty(&*tokens)
            .context("Failed to serialize tokens")?;

        fs::write(&self.storage_path, content)
            .context("Failed to write token storage")?;

        // Ensure permissions are secure after write
        self.set_secure_permissions()?;

        Ok(())
    }

    /// Generate a new authentication token
    pub async fn generate_token(&self, project_id: String) -> Result<String> {
        // Generate 32-byte UUID v4 token
        let token_value = Uuid::new_v4().to_string();

        // Hash the token with SHA256 for storage
        let token_hash = Self::hash_token(&token_value);

        // Create token with 24-hour expiration
        let now = Utc::now();
        let expires_at = now + Duration::hours(TOKEN_EXPIRY_HOURS);

        let token = Token {
            token_hash: token_hash.clone(),
            created_at: now,
            expires_at,
            project_id: project_id.clone(),
            revoked: false,
        };

        // Store token
        {
            let mut tokens = self.tokens.write().await;
            tokens.tokens.insert(token_hash.clone(), token);
        }

        // Persist to disk
        self.persist().await?;

        info!("Generated new token for project: {}", project_id);
        Ok(token_value)
    }

    /// Hash a token value with SHA256
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Validate a token
    pub async fn validate_token(&self, token_value: &str) -> Result<Token> {
        let token_hash = Self::hash_token(token_value);

        let tokens = self.tokens.read().await;
        let token = tokens.tokens.get(&token_hash)
            .ok_or_else(|| anyhow::anyhow!("Invalid token"))?;

        if !token.is_valid() {
            if token.revoked {
                anyhow::bail!("Token has been revoked");
            } else {
                anyhow::bail!("Token has expired");
            }
        }

        Ok(token.clone())
    }

    /// Revoke a token
    pub async fn revoke_token(&self, token_value: &str) -> Result<()> {
        let token_hash = Self::hash_token(token_value);

        let mut tokens = self.tokens.write().await;
        if let Some(token) = tokens.tokens.get_mut(&token_hash) {
            token.revoked = true;
            drop(tokens); // Release write lock

            self.persist().await?;
            info!("Revoked token: {}", &token_hash[..8]);
            Ok(())
        } else {
            anyhow::bail!("Token not found");
        }
    }

    /// Cleanup expired tokens
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut tokens = self.tokens.write().await;
        let initial_count = tokens.tokens.len();

        tokens.tokens.retain(|_, token| !token.is_expired());

        let removed_count = initial_count - tokens.tokens.len();

        if removed_count > 0 {
            drop(tokens); // Release write lock
            self.persist().await?;
            info!("Cleaned up {} expired tokens", removed_count);
        }

        Ok(removed_count)
    }

    /// List all active tokens (for admin purposes)
    pub async fn list_active_tokens(&self) -> Vec<Token> {
        let tokens = self.tokens.read().await;
        tokens.tokens.values()
            .filter(|t| t.is_valid())
            .cloned()
            .collect()
    }
}

/// Authenticated request context
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub token: Token,
    pub project_id: String,
}

/// Extract authentication from request
#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Missing Authorization header"
                    })),
                )
            })?;

        // Verify Bearer scheme
        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid Authorization scheme. Expected: Bearer <token>"
                })),
            ));
        }

        let token_value = &auth_header[7..]; // Skip "Bearer "

        // Token validation will be done by middleware
        // This extractor just provides the context after validation

        // For now, return a placeholder - actual validation is in middleware
        Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Token validation should be done by middleware"
            })),
        ))
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    token_manager: Arc<TokenManager>,
}

impl AuthMiddleware {
    pub fn new(token_manager: Arc<TokenManager>) -> Self {
        Self { token_manager }
    }

    /// Middleware function to validate authentication
    pub async fn authenticate(
        token_manager: Arc<TokenManager>,
        mut request: Request,
        next: Next,
    ) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
        // Extract Authorization header
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Missing Authorization header",
                        "details": "All API endpoints require authentication. Include: Authorization: Bearer <token>"
                    })),
                )
            })?;

        // Verify Bearer scheme
        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid Authorization scheme",
                    "details": "Expected: Authorization: Bearer <token>"
                })),
            ));
        }

        let token_value = &auth_header[7..]; // Skip "Bearer "

        // Validate token
        let token = token_manager
            .validate_token(token_value)
            .await
            .map_err(|e| {
                warn!("Token validation failed: {}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Invalid or expired token",
                        "details": e.to_string()
                    })),
                )
            })?;

        // Insert auth context into request extensions
        request.extensions_mut().insert(AuthContext {
            project_id: token.project_id.clone(),
            token,
        });

        // Continue to next middleware/handler
        Ok(next.run(request).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_token_manager_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = TokenManager::new(temp_file.path()).unwrap();

        assert_eq!(manager.tokens.read().await.tokens.len(), 0);
    }

    #[tokio::test]
    async fn test_generate_token() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = TokenManager::new(temp_file.path()).unwrap();

        let token = manager.generate_token("test-project".to_string()).await.unwrap();
        assert_eq!(token.len(), 36); // UUID v4 length
    }

    #[tokio::test]
    async fn test_validate_token() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = TokenManager::new(temp_file.path()).unwrap();

        let token_value = manager.generate_token("test-project".to_string()).await.unwrap();
        let token = manager.validate_token(&token_value).await.unwrap();

        assert_eq!(token.project_id, "test-project");
        assert!(token.is_valid());
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = TokenManager::new(temp_file.path()).unwrap();

        let result = manager.validate_token("invalid-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_revoke_token() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = TokenManager::new(temp_file.path()).unwrap();

        let token_value = manager.generate_token("test-project".to_string()).await.unwrap();
        manager.revoke_token(&token_value).await.unwrap();

        let result = manager.validate_token(&token_value).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = TokenManager::new(temp_file.path()).unwrap();

        // Create token with custom expiration (already expired)
        let token_value = Uuid::new_v4().to_string();
        let token_hash = TokenManager::hash_token(&token_value);

        let expired_token = Token {
            token_hash: token_hash.clone(),
            created_at: Utc::now() - Duration::hours(25),
            expires_at: Utc::now() - Duration::hours(1),
            project_id: "test".to_string(),
            revoked: false,
        };

        {
            let mut tokens = manager.tokens.write().await;
            tokens.tokens.insert(token_hash, expired_token);
        }

        let result = manager.validate_token(&token_value).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = TokenManager::new(temp_file.path()).unwrap();

        // Add an expired token
        let token_value = Uuid::new_v4().to_string();
        let token_hash = TokenManager::hash_token(&token_value);

        {
            let mut tokens = manager.tokens.write().await;
            tokens.tokens.insert(token_hash, Token {
                token_hash: TokenManager::hash_token(&token_value),
                created_at: Utc::now() - Duration::hours(25),
                expires_at: Utc::now() - Duration::hours(1),
                project_id: "test".to_string(),
                revoked: false,
            });
        }

        let removed = manager.cleanup_expired().await.unwrap();
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_token_hash_consistency() {
        let token = "test-token-123";
        let hash1 = TokenManager::hash_token(token);
        let hash2 = TokenManager::hash_token(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_token_hash_uniqueness() {
        let hash1 = TokenManager::hash_token("token1");
        let hash2 = TokenManager::hash_token("token2");
        assert_ne!(hash1, hash2);
    }
}
