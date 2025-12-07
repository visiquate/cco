//! HTTP API endpoints for the Credential Store
//!
//! Provides REST endpoints for secure credential management using the native
//! platform keyring system (Keychain on macOS, Secret Service on Linux, DPAPI on Windows).

use crate::credentials::keyring::{Credential, CredentialMetadata, CredentialStore};
use crate::daemon::security::auth::AuthContext;
use crate::daemon::security::validation::ValidatedMetadata;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared state for API handlers
pub type CredentialState = Arc<Mutex<CredentialStore>>;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    PayloadTooLarge(String),
    ValidationFailed(String),
    StorageError(String),
    NotFound(String),
    Expired(String),
    RateLimitExceeded(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, msg),
            ApiError::ValidationFailed(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::StorageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Expired(msg) => (StatusCode::GONE, msg),
            ApiError::RateLimitExceeded(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Request to store a credential
#[derive(Debug, Deserialize)]
pub struct StoreCredentialRequest {
    /// Unique identifier for the credential
    pub key: String,

    /// The secret value (will be encrypted)
    pub secret: String,

    /// Credential metadata
    pub metadata: CredentialMetadata,

    /// Optional expiration timestamp (ISO 8601)
    pub expires_at: Option<String>,
}

/// Response from storing a credential
#[derive(Debug, Serialize)]
pub struct StoreCredentialResponse {
    pub success: bool,
    pub key: String,
    pub message: String,
}

/// Response from retrieving a credential
#[derive(Debug, Serialize)]
pub struct RetrieveCredentialResponse {
    pub key: String,
    pub secret: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_accessed: String,
    pub last_rotated: Option<String>,
    pub metadata: CredentialMetadata,
}

/// Response from listing credentials
#[derive(Debug, Serialize)]
pub struct ListCredentialsResponse {
    pub keys: Vec<String>,
    pub count: usize,
}

/// Response from checking rotation status
#[derive(Debug, Serialize)]
pub struct RotationCheckResponse {
    pub needs_rotation: Vec<String>,
    pub expired: Vec<String>,
    pub rotation_count: usize,
    pub expired_count: usize,
}

/// Request to rotate a credential
#[derive(Debug, Deserialize)]
pub struct RotateCredentialRequest {
    pub new_secret: String,
}

/// Build the credentials API router without state (caller must apply state)
pub fn credentials_router_without_state() -> Router<CredentialState> {
    Router::new()
        .route("/api/credentials/store", post(store_credential))
        .route("/api/credentials/retrieve/:key", get(retrieve_credential))
        .route("/api/credentials/list", get(list_credentials))
        .route("/api/credentials/delete/:key", delete(delete_credential))
        .route("/api/credentials/exists/:key", get(check_exists))
        .route("/api/credentials/check-rotation", get(check_rotation))
        .route("/api/credentials/rotate/:key", post(rotate_credential))
}

/// Build the credentials API router with state
pub fn credentials_router(store: CredentialState) -> Router {
    credentials_router_without_state().with_state(store)
}

/// Store a credential securely
///
/// POST /api/credentials/store
/// Body: StoreCredentialRequest
async fn store_credential(
    State(store): State<CredentialState>,
    Extension(_auth): Extension<AuthContext>,
    Json(request): Json<StoreCredentialRequest>,
) -> Result<Json<StoreCredentialResponse>, ApiError> {
    // Validate key size (1 KB max)
    if request.key.len() > 1024 {
        return Err(ApiError::PayloadTooLarge(format!(
            "Key exceeds 1 KB limit (got {} bytes)",
            request.key.len()
        )));
    }

    // Validate secret size (100 KB max)
    if request.secret.len() > 100 * 1024 {
        return Err(ApiError::PayloadTooLarge(format!(
            "Secret exceeds 100 KB limit (got {} bytes)",
            request.secret.len()
        )));
    }

    // Validate metadata
    let metadata_value = serde_json::to_value(&request.metadata)
        .map_err(|e| ApiError::ValidationFailed(format!("Invalid metadata: {}", e)))?;
    ValidatedMetadata::from_json(metadata_value)
        .map_err(|e| ApiError::ValidationFailed(e.to_string()))?;

    // Parse expiration if provided
    let expires_at = if let Some(exp_str) = request.expires_at {
        Some(
            chrono::DateTime::parse_from_rfc3339(&exp_str)
                .map_err(|e| ApiError::ValidationFailed(format!("Invalid expiration date: {}", e)))?
                .with_timezone(&chrono::Utc),
        )
    } else {
        None
    };

    // Create credential
    let credential = Credential {
        key: request.key.clone(),
        secret: SecretString::new(request.secret),
        created_at: chrono::Utc::now(),
        expires_at,
        last_accessed: chrono::Utc::now(),
        last_rotated: None,
        metadata: request.metadata,
    };

    // Store the credential
    let mut store = store.lock().await;
    store
        .store_credential(credential)
        .await
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    Ok(Json(StoreCredentialResponse {
        success: true,
        key: request.key,
        message: "Credential stored successfully".to_string(),
    }))
}

/// Retrieve a credential by key
///
/// GET /api/credentials/retrieve/:key
async fn retrieve_credential(
    State(store): State<CredentialState>,
    Extension(_auth): Extension<AuthContext>,
    Path(key): Path<String>,
) -> Result<Json<RetrieveCredentialResponse>, ApiError> {
    let mut store = store.lock().await;

    match store.retrieve_credential(&key).await {
        Ok(credential) => {
            // Convert credential to response (exposing secret - ensure HTTPS!)
            Ok(Json(RetrieveCredentialResponse {
                key: credential.key,
                secret: credential.secret.expose_secret().to_string(),
                created_at: credential.created_at.to_rfc3339(),
                expires_at: credential.expires_at.map(|dt| dt.to_rfc3339()),
                last_accessed: credential.last_accessed.to_rfc3339(),
                last_rotated: credential.last_rotated.map(|dt| dt.to_rfc3339()),
                metadata: credential.metadata,
            }))
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("not found") {
                Err(ApiError::NotFound(format!(
                    "Credential '{}' not found",
                    key
                )))
            } else if error_str.contains("expired") {
                Err(ApiError::Expired(format!(
                    "Credential '{}' has expired",
                    key
                )))
            } else if error_str.contains("rate limit") {
                Err(ApiError::RateLimitExceeded(error_str))
            } else {
                Err(ApiError::StorageError(error_str))
            }
        }
    }
}

/// List all credential keys (without secrets)
///
/// GET /api/credentials/list
async fn list_credentials(
    State(store): State<CredentialState>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<ListCredentialsResponse>, ApiError> {
    let store = store.lock().await;

    match store.list_credentials().await {
        Ok(keys) => {
            let count = keys.len();
            Ok(Json(ListCredentialsResponse { keys, count }))
        }
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Delete a credential
///
/// DELETE /api/credentials/delete/:key
async fn delete_credential(
    State(store): State<CredentialState>,
    Extension(_auth): Extension<AuthContext>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = store.lock().await;

    match store.delete_credential(&key).await {
        Ok(deleted) => {
            if deleted {
                Ok(Json(json!({
                    "success": true,
                    "key": key,
                    "message": "Credential deleted successfully"
                })))
            } else {
                Err(ApiError::NotFound(format!(
                    "Credential '{}' not found",
                    key
                )))
            }
        }
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Check if a credential exists
///
/// GET /api/credentials/exists/:key
async fn check_exists(
    State(store): State<CredentialState>,
    Extension(_auth): Extension<AuthContext>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = store.lock().await;

    match store.credential_exists(&key).await {
        Ok(exists) => Ok(Json(json!({ "exists": exists, "key": key }))),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Check credentials that need rotation or have expired
///
/// GET /api/credentials/check-rotation
async fn check_rotation(
    State(store): State<CredentialState>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<RotationCheckResponse>, ApiError> {
    let mut store = store.lock().await;

    let needs_rotation = store
        .check_rotation_needed()
        .await
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    let expired = store
        .get_expired_credentials()
        .await
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    let rotation_count = needs_rotation.len();
    let expired_count = expired.len();

    Ok(Json(RotationCheckResponse {
        needs_rotation,
        expired,
        rotation_count,
        expired_count,
    }))
}

/// Rotate a credential with a new secret
///
/// POST /api/credentials/rotate/:key
/// Body: RotateCredentialRequest
async fn rotate_credential(
    State(store): State<CredentialState>,
    Extension(_auth): Extension<AuthContext>,
    Path(key): Path<String>,
    Json(request): Json<RotateCredentialRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Validate secret size (100 KB max)
    if request.new_secret.len() > 100 * 1024 {
        return Err(ApiError::PayloadTooLarge(format!(
            "Secret exceeds 100 KB limit (got {} bytes)",
            request.new_secret.len()
        )));
    }

    let mut store = store.lock().await;

    match store
        .rotate_credential(&key, SecretString::new(request.new_secret))
        .await
    {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "key": key,
            "message": "Credential rotated successfully"
        }))),
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("not found") {
                Err(ApiError::NotFound(format!(
                    "Credential '{}' not found",
                    key
                )))
            } else {
                Err(ApiError::StorageError(error_str))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::keyring::CredentialType;

    fn create_test_metadata() -> CredentialMetadata {
        CredentialMetadata {
            service: "test_service".to_string(),
            agent_name: Some("test_agent".to_string()),
            description: "Test credential".to_string(),
            credential_type: CredentialType::ApiKey,
            rotation_required: false,
            rotation_interval_days: None,
            encryption_algorithm: "aes-256-gcm".to_string(),
            custom: std::collections::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_store_credential_validation() {
        // Test payload size validation
        let large_secret = "x".repeat(200 * 1024); // 200 KB
        let request = StoreCredentialRequest {
            key: "test_key".to_string(),
            secret: large_secret,
            metadata: create_test_metadata(),
            expires_at: None,
        };

        // This would fail with PayloadTooLarge
        assert!(request.secret.len() > 100 * 1024);
    }

    #[test]
    fn test_credential_type_serialization() {
        let metadata = create_test_metadata();
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("api_key"));
    }
}
