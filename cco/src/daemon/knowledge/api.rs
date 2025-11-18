//! HTTP API endpoints for the Knowledge Store
//!
//! Provides REST endpoints matching the JavaScript knowledge-manager.js CLI interface

use super::models::*;
use super::store::KnowledgeStore;
use crate::daemon::security::auth::AuthContext;
use crate::daemon::security::credential_detector::CredentialDetector;
use crate::daemon::security::validation::ValidatedMetadata;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared state for API handlers
pub type KnowledgeState = Arc<Mutex<KnowledgeStore>>;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    PayloadTooLarge(String),
    CredentialDetected(String),
    InvalidMetadata(String),
    ValidationFailed(String),
    StorageError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, msg),
            ApiError::CredentialDetected(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::InvalidMetadata(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::ValidationFailed(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::StorageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Build the knowledge API router without state (caller must apply state)
pub fn knowledge_router_without_state() -> Router<KnowledgeState> {
    Router::new()
        .route("/api/knowledge/store", post(store_handler))
        .route("/api/knowledge/store-batch", post(store_batch_handler))
        .route("/api/knowledge/search", post(search_handler))
        .route(
            "/api/knowledge/project/:project_id",
            get(project_knowledge_handler),
        )
        .route(
            "/api/knowledge/pre-compaction",
            post(pre_compaction_handler),
        )
        .route(
            "/api/knowledge/post-compaction",
            post(post_compaction_handler),
        )
        .route("/api/knowledge/stats", get(stats_handler))
        .route("/api/knowledge/cleanup", post(cleanup_handler))
}

/// Build the knowledge API router with state
pub fn knowledge_router(store: KnowledgeState) -> Router {
    knowledge_router_without_state().with_state(store)
}

/// Store a single knowledge item
///
/// POST /api/knowledge/store
/// Body: StoreKnowledgeRequest
async fn store_handler(
    State(store): State<KnowledgeState>,
    Extension(_auth): Extension<AuthContext>,
    Json(request): Json<StoreKnowledgeRequest>,
) -> Result<Json<StoreKnowledgeResponse>, ApiError> {
    // Validate text size (10 MB max)
    if request.text.len() > 10 * 1024 * 1024 {
        return Err(ApiError::PayloadTooLarge(format!(
            "Text field exceeds 10 MB limit (got {} bytes)",
            request.text.len()
        )));
    }

    // Check for credentials
    let detector = CredentialDetector::new();
    if detector.contains_credentials(&request.text) {
        return Err(ApiError::CredentialDetected(
            "Request contains sensitive data (credentials, API keys, passwords)".to_string(),
        ));
    }

    // Validate metadata if present
    if let Some(ref metadata) = request.metadata {
        let metadata_value = serde_json::to_value(metadata)
            .map_err(|e| ApiError::InvalidMetadata(format!("Invalid JSON: {}", e)))?;
        ValidatedMetadata::from_json(metadata_value)
            .map_err(|e| ApiError::InvalidMetadata(e.to_string()))?;
    }

    // Store the knowledge item
    let mut store = store.lock().await;
    match store.store(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Store multiple knowledge items in batch
///
/// POST /api/knowledge/store-batch
/// Body: Vec<StoreKnowledgeRequest>
async fn store_batch_handler(
    State(store): State<KnowledgeState>,
    Extension(_auth): Extension<AuthContext>,
    Json(requests): Json<Vec<StoreKnowledgeRequest>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Validate total size (50 MB for batch)
    let total_size: usize = requests.iter().map(|r| r.text.len()).sum();
    if total_size > 50 * 1024 * 1024 {
        return Err(ApiError::PayloadTooLarge(format!(
            "Batch exceeds 50 MB limit (got {} bytes)",
            total_size
        )));
    }

    // Check for credentials in all items
    let detector = CredentialDetector::new();
    for (idx, request) in requests.iter().enumerate() {
        if detector.contains_credentials(&request.text) {
            return Err(ApiError::CredentialDetected(format!(
                "Item {} contains sensitive data",
                idx
            )));
        }

        // Validate metadata if present
        if let Some(ref metadata) = request.metadata {
            let metadata_value = serde_json::to_value(metadata)
                .map_err(|e| ApiError::InvalidMetadata(format!("Item {}: Invalid JSON: {}", idx, e)))?;
            ValidatedMetadata::from_json(metadata_value)
                .map_err(|e| ApiError::InvalidMetadata(format!("Item {}: {}", idx, e)))?;
        }
    }

    // Store batch
    let mut store = store.lock().await;
    match store.store_batch(requests).await {
        Ok(ids) => Ok(Json(json!({ "ids": ids, "count": ids.len() }))),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Search knowledge base
///
/// POST /api/knowledge/search
/// Body: SearchRequest
async fn search_handler(
    State(store): State<KnowledgeState>,
    Extension(_auth): Extension<AuthContext>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<Vec<SearchResult>>, ApiError> {
    // Validate query size (100 KB max)
    if request.query.len() > 100_000 {
        return Err(ApiError::PayloadTooLarge(
            "Search query exceeds 100 KB".to_string(),
        ));
    }

    let store = store.lock().await;
    match store.search(request).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Get project knowledge
///
/// GET /api/knowledge/project/:project_id?type=<type>&limit=<limit>
async fn project_knowledge_handler(
    State(store): State<KnowledgeState>,
    Extension(_auth): Extension<AuthContext>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<SearchResult>>, ApiError> {
    let store = store.lock().await;

    // TODO: Parse query parameters for type and limit
    match store.get_project_knowledge(&project_id, None, 100).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Pre-compaction knowledge capture
///
/// POST /api/knowledge/pre-compaction
/// Body: PreCompactionRequest
async fn pre_compaction_handler(
    State(store): State<KnowledgeState>,
    Extension(_auth): Extension<AuthContext>,
    Json(request): Json<PreCompactionRequest>,
) -> Result<Json<PreCompactionResponse>, ApiError> {
    let mut store = store.lock().await;

    match store.pre_compaction(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Post-compaction knowledge retrieval
///
/// POST /api/knowledge/post-compaction
/// Body: PostCompactionRequest
async fn post_compaction_handler(
    State(store): State<KnowledgeState>,
    Extension(_auth): Extension<AuthContext>,
    Json(request): Json<PostCompactionRequest>,
) -> Result<Json<PostCompactionResponse>, ApiError> {
    let store = store.lock().await;

    match store.post_compaction(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Get knowledge base statistics
///
/// GET /api/knowledge/stats
async fn stats_handler(
    State(store): State<KnowledgeState>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<StatsResponse>, ApiError> {
    let store = store.lock().await;

    match store.get_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

/// Cleanup old knowledge
///
/// POST /api/knowledge/cleanup
/// Body: CleanupRequest
async fn cleanup_handler(
    State(store): State<KnowledgeState>,
    Extension(_auth): Extension<AuthContext>,
    Json(request): Json<CleanupRequest>,
) -> Result<Json<CleanupResponse>, ApiError> {
    let store = store.lock().await;

    match store.cleanup(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err(ApiError::StorageError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::security::auth::Token;
    use tempfile::tempdir;

    async fn create_test_store() -> KnowledgeState {
        let temp_dir = tempdir().unwrap();
        let mut store = KnowledgeStore::new(
            temp_dir.path(),
            Some(temp_dir.path()),
            Some("test_knowledge".to_string()),
        );
        store.initialize().await.unwrap();
        Arc::new(Mutex::new(store))
    }

    fn create_test_auth() -> AuthContext {
        AuthContext {
            token: Token {
                token_hash: "test_hash".to_string(),
                created_at: chrono::Utc::now(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
                project_id: "test_project".to_string(),
                revoked: false,
            },
            project_id: "test_project".to_string(),
        }
    }

    #[tokio::test]
    async fn test_store_handler() {
        let store = create_test_store().await;
        let auth = create_test_auth();
        let request = StoreKnowledgeRequest {
            text: "Test knowledge".to_string(),
            knowledge_type: Some("decision".to_string()),
            project_id: None,
            session_id: None,
            agent: None,
            metadata: None,
        };

        let result = store_handler(State(store), Extension(auth), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_handler() {
        let store = create_test_store().await;
        let auth = create_test_auth();
        let request = SearchRequest {
            query: "test".to_string(),
            limit: 10,
            threshold: 0.5,
            project_id: None,
            knowledge_type: None,
            agent: None,
        };

        let result = search_handler(State(store), Extension(auth), Json(request)).await;
        assert!(result.is_ok());
    }
}
