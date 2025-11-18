//! Daemon HTTP Server
//!
//! Provides HTTP API endpoints for the daemon, including:
//! - /api/classify - CRUD classification of shell commands
//! - /api/hooks/permission-request - Permission request handling
//! - /api/hooks/decisions - Decision history and statistics
//! - /health - Health check with hooks status
//! - /api/shutdown - Graceful shutdown endpoint

use axum::{
    extract::{Json, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{debug, info, warn};

use super::config::DaemonConfig;
use super::hooks::{
    CrudClassification, CrudClassifier, Decision, DecisionDatabase, HookExecutor,
    HookRegistry, PermissionHandler, PermissionResponse,
    SqliteAuditDatabase,
};
use super::knowledge::{knowledge_router_without_state, KnowledgeState, KnowledgeStore};
use super::security::{TokenManager, CredentialDetector};

/// Classification decision tracking
#[derive(Debug, Clone, Serialize)]
pub struct ClassificationDecision {
    pub command: String,
    pub classification: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub decision: String,
    pub confidence_score: f32,
}

/// Decision statistics
#[derive(Debug, Clone, Default, Serialize)]
pub struct DecisionStatistics {
    pub read_count: u64,
    pub create_count: u64,
    pub update_count: u64,
    pub delete_count: u64,
    pub total_requests: u64,
}

/// Daemon server state shared across handlers
#[derive(Clone)]
pub struct DaemonState {
    pub config: DaemonConfig,
    pub hooks_registry: Arc<HookRegistry>,
    pub hook_executor: HookExecutor,
    pub crud_classifier: Option<Arc<CrudClassifier>>,
    pub permission_handler: Arc<PermissionHandler>,
    pub audit_db: Option<Arc<SqliteAuditDatabase>>,
    pub start_time: std::time::Instant,
    pub recent_decisions: Arc<Mutex<VecDeque<ClassificationDecision>>>,
    pub decision_stats: Arc<Mutex<DecisionStatistics>>,
    pub last_classification_ms: Arc<Mutex<Option<u32>>>,
    pub knowledge_store: Option<KnowledgeState>,
    pub token_manager: Option<Arc<TokenManager>>,
    pub credential_detector: Arc<CredentialDetector>,
}

impl DaemonState {
    /// Create new daemon state with hooks initialization
    pub async fn new(config: DaemonConfig) -> anyhow::Result<Self> {
        let hooks_registry = Arc::new(HookRegistry::new());
        let hook_executor = HookExecutor::with_config(
            hooks_registry.clone(),
            Duration::from_millis(config.hooks.timeout_ms),
            config.hooks.max_retries,
        );

        // Initialize permission handler
        let permission_handler = Arc::new(PermissionHandler::new());

        // Initialize audit database if hooks enabled
        let audit_db = if config.hooks.is_enabled() {
            let db_path = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".cco")
                .join("decisions.db");

            match SqliteAuditDatabase::new(db_path).await {
                Ok(db) => {
                    info!("✅ Audit database initialized successfully");

                    // Cleanup old decisions (older than 7 days)
                    match db.cleanup_old_decisions(7).await {
                        Ok(deleted) if deleted > 0 => {
                            info!("Cleaned up {} old decisions", deleted);
                        }
                        Ok(_) => debug!("No old decisions to clean up"),
                        Err(e) => warn!("Failed to cleanup old decisions: {}", e),
                    }

                    Some(Arc::new(db))
                }
                Err(e) => {
                    warn!("Failed to initialize audit database: {}", e);
                    warn!("Decision history will not be persisted");
                    None
                }
            }
        } else {
            info!("Hooks system disabled - audit database not initialized");
            None
        };

        // Initialize classifier if hooks enabled
        let crud_classifier = if config.hooks.is_enabled() {
            match CrudClassifier::new(config.hooks.llm.clone()).await {
                Ok(classifier) => {
                    info!("✅ CRUD classifier initialized successfully");

                    // Ensure model is available (download if needed)
                    if let Err(e) = classifier.ensure_model_available().await {
                        warn!("Failed to ensure model availability: {}", e);
                        warn!("Classifier will use fallback (CREATE) on first requests");
                    } else {
                        info!("✅ Model verified and ready");
                    }

                    Some(Arc::new(classifier))
                }
                Err(e) => {
                    warn!("Failed to initialize CRUD classifier: {}", e);
                    warn!("Classification endpoint will return service unavailable");
                    None
                }
            }
        } else {
            info!("Hooks system disabled - classifier not initialized");
            None
        };

        // Initialize token manager
        let token_manager = {
            let token_storage_path = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".cco")
                .join("tokens.json");

            match TokenManager::new(&token_storage_path) {
                Ok(manager) => {
                    info!("✅ Token manager initialized successfully");
                    Some(Arc::new(manager))
                }
                Err(e) => {
                    warn!("Failed to initialize token manager: {}", e);
                    warn!("Knowledge API authentication will not be available");
                    None
                }
            }
        };

        // Initialize credential detector
        let credential_detector = Arc::new(CredentialDetector::new());
        info!("✅ Credential detector initialized");

        // Initialize knowledge store
        let knowledge_store = {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let base_dir = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".cco")
                .join("knowledge");

            let mut store = KnowledgeStore::new(
                &cwd,
                Some(&base_dir),
                Some("orchestra_knowledge".to_string()),
            );

            match store.initialize().await {
                Ok(_) => {
                    info!("✅ Knowledge store initialized successfully");
                    Some(Arc::new(tokio::sync::Mutex::new(store)))
                }
                Err(e) => {
                    warn!("Failed to initialize knowledge store: {}", e);
                    warn!("Knowledge API endpoints will not be available");
                    None
                }
            }
        };

        Ok(Self {
            config,
            hooks_registry,
            hook_executor,
            crud_classifier,
            permission_handler,
            audit_db,
            start_time: std::time::Instant::now(),
            recent_decisions: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            decision_stats: Arc::new(Mutex::new(DecisionStatistics::default())),
            last_classification_ms: Arc::new(Mutex::new(None)),
            knowledge_store,
            token_manager,
            credential_detector,
        })
    }
}

/// Request payload for /api/classify endpoint
#[derive(Debug, Deserialize)]
pub struct ClassifyRequest {
    pub command: String,
    #[serde(default)]
    pub context: Option<String>,
}

/// Response from /api/classify endpoint
#[derive(Debug, Serialize)]
pub struct ClassifyResponse {
    pub classification: String,
    pub confidence: f32,
    pub reasoning: String,
    pub timestamp: String,
}

/// Health check response with hooks status
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub port: u16,
    pub hooks: HooksHealthStatus,
}

/// Hooks system health status
#[derive(Debug, Serialize)]
pub struct HooksHealthStatus {
    pub enabled: bool,
    pub classifier_available: bool,
    pub model_loaded: bool,
    pub model_name: String,
    pub classification_latency_ms: Option<u32>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

/// Custom error type for daemon server errors
#[derive(Debug)]
pub enum AppError {
    ClassificationFailed(String),
    ClassifierUnavailable,
    HookFailed(String),
    TimeoutError,
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error, details) = match self {
            AppError::ClassificationFailed(msg) => (
                StatusCode::BAD_REQUEST,
                "Classification failed".to_string(),
                Some(msg),
            ),
            AppError::ClassifierUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "CRUD classifier not available".to_string(),
                Some("Hooks system disabled or classifier failed to initialize".to_string()),
            ),
            AppError::HookFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Hook execution failed".to_string(),
                Some(msg),
            ),
            AppError::TimeoutError => (
                StatusCode::REQUEST_TIMEOUT,
                "Request timed out".to_string(),
                None,
            ),
            AppError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
                Some(msg),
            ),
        };

        let body = Json(ErrorResponse { error, details });
        (status, body).into_response()
    }
}

/// Health check endpoint
async fn health(State(state): State<Arc<DaemonState>>) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();

    // Check classifier status
    let (classifier_available, model_loaded, model_name) = if let Some(ref classifier) = state.crud_classifier {
        let loaded = classifier.is_model_loaded().await;
        (true, loaded, state.config.hooks.llm.model_name.clone())
    } else {
        (false, false, "none".to_string())
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::version::DateVersion::current().to_string(),
        uptime_seconds: uptime,
        port: state.config.port,
        hooks: HooksHealthStatus {
            enabled: state.config.hooks.is_enabled(),
            classifier_available,
            model_loaded,
            model_name,
            classification_latency_ms: None, // TODO: Track metrics
        },
    })
}

/// Command classification endpoint
async fn classify_command(
    State(state): State<Arc<DaemonState>>,
    Json(request): Json<ClassifyRequest>,
) -> Result<Json<ClassifyResponse>, AppError> {
    info!("Classification request for command: {}", request.command);

    // Get classifier from state
    let classifier = state
        .crud_classifier
        .as_ref()
        .ok_or(AppError::ClassifierUnavailable)?;

    // Measure classification latency
    let start = std::time::Instant::now();

    // Classify the command
    let result = classifier.classify(&request.command).await;

    let latency_ms = start.elapsed().as_millis() as u32;

    // Update latency tracking
    if let Ok(mut last_ms) = state.last_classification_ms.lock() {
        *last_ms = Some(latency_ms);
    }

    // Track decision (auto-approved for now)
    let classification_str = format!("{:?}", result.classification);
    let decision = ClassificationDecision {
        command: request.command.clone(),
        classification: classification_str.clone(),
        timestamp: Utc::now(),
        decision: "APPROVED".to_string(),
        confidence_score: result.confidence,
    };

    // Add to recent decisions (keep last 100)
    if let Ok(mut decisions) = state.recent_decisions.lock() {
        decisions.push_front(decision);
        if decisions.len() > 100 {
            decisions.pop_back();
        }
    }

    // Update statistics
    if let Ok(mut stats) = state.decision_stats.lock() {
        stats.total_requests += 1;
        match result.classification {
            CrudClassification::Read => stats.read_count += 1,
            CrudClassification::Create => stats.create_count += 1,
            CrudClassification::Update => stats.update_count += 1,
            CrudClassification::Delete => stats.delete_count += 1,
        }
    }

    // Return response
    Ok(Json(ClassifyResponse {
        classification: classification_str,
        confidence: result.confidence,
        reasoning: result.reasoning.unwrap_or_else(|| "No reasoning provided".to_string()),
        timestamp: Utc::now().to_rfc3339(),
    }))
}

/// Permission request endpoint
#[derive(Debug, Deserialize)]
struct PermissionRequestPayload {
    command: String,
    classification: String,
}

async fn permission_request(
    State(state): State<Arc<DaemonState>>,
    Json(payload): Json<PermissionRequestPayload>,
) -> Result<Json<PermissionResponse>, AppError> {
    debug!(
        "Permission request for command: {} (classification: {})",
        payload.command, payload.classification
    );

    // Parse classification
    let classification: CrudClassification = payload.classification.parse().map_err(|e| {
        AppError::ClassificationFailed(format!("Invalid classification: {}", e))
    })?;

    // Get classifier to get confidence score
    let classifier = state
        .crud_classifier
        .as_ref()
        .ok_or(AppError::ClassifierUnavailable)?;

    // Measure response time
    let start = std::time::Instant::now();

    // Classify the command to get confidence
    let classification_result = classifier.classify(&payload.command).await;

    let response_time_ms = start.elapsed().as_millis() as i32;

    // Process permission request
    let response = state
        .permission_handler
        .process_classification(&payload.command, classification_result.clone())
        .await;

    // Store decision in audit database (async, non-blocking)
    if let Some(ref audit_db) = state.audit_db {
        let decision = Decision {
            id: 0, // Will be auto-generated by database
            command: payload.command.clone(),
            classification: classification_result.classification,
            timestamp: Utc::now(),
            user_decision: response.decision,
            reasoning: Some(response.reasoning.clone()),
            confidence_score: response.confidence,
            response_time_ms: Some(response_time_ms),
        };

        // Spawn async task to store decision (don't block response)
        let audit_db_clone = Arc::clone(audit_db);
        tokio::spawn(async move {
            if let Err(e) = audit_db_clone.store_decision(decision).await {
                warn!("Failed to store decision in audit database: {}", e);
            }
        });
    }

    info!(
        "Permission decision: {} for {} ({})",
        response.decision, payload.command, classification
    );

    Ok(Json(response))
}

/// Hooks decisions endpoint - returns recent classifications and statistics
#[derive(Debug, Serialize)]
struct DecisionsResponse {
    recent: Vec<ClassificationDecision>,
    stats: DecisionStatistics,
    enabled: bool,
    model_loaded: bool,
    model_name: String,
    last_classification_ms: Option<u32>,
}

async fn get_hooks_decisions(State(state): State<Arc<DaemonState>>) -> Json<DecisionsResponse> {
    // Get classifier status
    let (model_loaded, model_name) = if let Some(ref classifier) = state.crud_classifier {
        let loaded = classifier.is_model_loaded().await;
        (loaded, state.config.hooks.llm.model_name.clone())
    } else {
        (false, "none".to_string())
    };

    // Get recent decisions (up to 20)
    let recent = if let Ok(decisions) = state.recent_decisions.lock() {
        decisions.iter().take(20).cloned().collect()
    } else {
        Vec::new()
    };

    // Get statistics
    let stats = if let Ok(stats) = state.decision_stats.lock() {
        stats.clone()
    } else {
        DecisionStatistics::default()
    };

    // Get last classification latency
    let last_classification_ms = if let Ok(last_ms) = state.last_classification_ms.lock() {
        *last_ms
    } else {
        None
    };

    Json(DecisionsResponse {
        recent,
        stats,
        enabled: state.config.hooks.is_enabled(),
        model_loaded,
        model_name,
        last_classification_ms,
    })
}

/// Graceful shutdown endpoint
async fn shutdown_handler() -> Json<serde_json::Value> {
    info!("🛑 Shutdown request received via API");

    // Spawn a task to exit after sending the response
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        info!("Initiating shutdown...");
        std::process::exit(0);
    });

    Json(serde_json::json!({
        "status": "shutdown_initiated",
        "message": "Daemon shutting down..."
    }))
}

/// Token generation request
#[derive(Debug, Deserialize)]
struct GenerateTokenRequest {
    project_id: String,
}

/// Token generation response
#[derive(Debug, Serialize)]
struct GenerateTokenResponse {
    token: String,
    expires_at: String,
    project_id: String,
}

/// Generate authentication token endpoint
async fn generate_token(
    State(state): State<Arc<DaemonState>>,
    Json(request): Json<GenerateTokenRequest>,
) -> Result<Json<GenerateTokenResponse>, AppError> {
    let token_manager = state
        .token_manager
        .as_ref()
        .ok_or(AppError::InternalError("Token manager not available".to_string()))?;

    let token = token_manager
        .generate_token(request.project_id.clone())
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to generate token: {}", e)))?;

    // Calculate expiration (24 hours from now)
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();

    info!("Generated token for project: {}", request.project_id);

    Ok(Json(GenerateTokenResponse {
        token,
        expires_at,
        project_id: request.project_id,
    }))
}

/// Create the axum router with all daemon endpoints
fn create_router(state: Arc<DaemonState>) -> Router {
    let mut router = Router::new()
        .route("/health", get(health))
        .route("/api/classify", post(classify_command))
        .route("/api/hooks/permission-request", post(permission_request))
        .route("/api/hooks/decisions", get(get_hooks_decisions))
        .route("/api/shutdown", post(shutdown_handler));

    // Add token generation endpoint if token manager is available
    if state.token_manager.is_some() {
        router = router.route("/api/token/generate", post(generate_token));
        info!("Token generation endpoint available at /api/token/generate");
    }

    // Mount knowledge routes if knowledge store is available
    if let Some(ref knowledge_store) = state.knowledge_store {
        info!("Mounting knowledge API routes at /api/knowledge/*");

        // Create knowledge routes without state first
        let mut knowledge_routes = knowledge_router_without_state();

        if let Some(ref token_manager) = state.token_manager {
            // Apply authentication middleware to all knowledge routes
            let auth_layer = middleware::from_fn({
                let token_manager = Arc::clone(token_manager);
                move |req, next| {
                    let token_manager = Arc::clone(&token_manager);
                    async move {
                        super::security::AuthMiddleware::authenticate(token_manager, req, next).await
                    }
                }
            });

            knowledge_routes = knowledge_routes.layer(auth_layer);
        } else {
            warn!("Token manager not available - knowledge routes will NOT be authenticated");
        }

        // Apply knowledge store state and nest under main router
        let knowledge_routes_with_state = knowledge_routes.with_state(Arc::clone(knowledge_store));
        router = router.nest("/", knowledge_routes_with_state);
    } else {
        info!("Knowledge store not available - skipping knowledge API routes");
    }

    router.with_state(state)
}

/// Run the daemon HTTP server
pub async fn run_daemon_server(config: DaemonConfig) -> anyhow::Result<()> {
    info!("🚀 Starting CCO Daemon Server");
    info!("   Version: {}", crate::version::DateVersion::current());
    info!("   Port: {}", config.port);
    info!("   Host: {}", config.host);
    info!("   Hooks enabled: {}", config.hooks.is_enabled());

    // Initialize daemon state
    let state = Arc::new(DaemonState::new(config.clone()).await?);

    // Create router (clone state for router while keeping reference for logging)
    let app = create_router(Arc::clone(&state));

    // Bind to address
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    info!("✅ Daemon server listening on http://{}", addr);
    info!("   Health: http://{}/health", addr);
    info!("   Classify: http://{}/api/classify", addr);
    info!("   Permission: http://{}/api/hooks/permission-request", addr);
    info!("   Decisions: http://{}/api/hooks/decisions", addr);
    info!("   Shutdown: http://{}/api/shutdown", addr);

    if state.token_manager.is_some() {
        info!("   Token Generation: http://{}/api/token/generate", addr);
    }

    if state.knowledge_store.is_some() {
        info!("   Knowledge API: http://{}/api/knowledge/*", addr);
        info!("     - POST /api/knowledge/store - Store knowledge item");
        info!("     - POST /api/knowledge/search - Search knowledge base");
        info!("     - GET  /api/knowledge/stats - Knowledge statistics");

        if state.token_manager.is_some() {
            info!("     (All knowledge routes require Bearer token authentication)");
        } else {
            info!("     (WARNING: Authentication disabled - no token manager)");
        }
    }

    info!("");
    info!("Press Ctrl+C to stop");

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    info!("Daemon server shut down gracefully");
    Ok(())
}

/// Wait for Ctrl+C or SIGTERM signal
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }

    info!("Initiating graceful shutdown...");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daemon_state_creation() {
        let config = DaemonConfig::default();
        let state = DaemonState::new(config).await;
        assert!(state.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_state_with_disabled_hooks() {
        let mut config = DaemonConfig::default();
        config.hooks.enabled = false;

        let state = DaemonState::new(config).await.unwrap();
        assert!(state.crud_classifier.is_none());
    }

    #[test]
    fn test_classify_request_deserialization() {
        let json = r#"{"command":"ls -la"}"#;
        let req: ClassifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.command, "ls -la");
        assert!(req.context.is_none());
    }

    #[test]
    fn test_classify_request_with_context() {
        let json = r#"{"command":"mkdir test","context":"user-initiated"}"#;
        let req: ClassifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.command, "mkdir test");
        assert_eq!(req.context, Some("user-initiated".to_string()));
    }
}
