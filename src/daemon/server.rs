//! Daemon HTTP Server
//!
//! Provides HTTP API endpoints for the daemon, including:
//! - /api/classify - CRUD classification of shell commands
//! - /api/hooks/permission-request - Permission request handling
//! - /api/hooks/decisions - Decision history and statistics
//! - /health - Health check with hooks status
//! - /api/shutdown - Graceful shutdown endpoint

use axum::{
    extract::{Json, Query, State},
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
    CrudClassification, CrudClassifier, Decision, DecisionDatabase, HookExecutor, HookRegistry,
    PermissionHandler, PermissionResponse, SqliteAuditDatabase,
};
use super::knowledge::{knowledge_router_without_state, KnowledgeState, KnowledgeStore};
use super::log_watcher::LogWatcher;
use super::orchestration_routes::{
    create_orchestration_router, init_orchestration_state, OrchestrationHandlerState,
};
use super::security::TokenManager;
use crate::orchestration::OrchestrationState;

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
    // Note: CredentialDetector is created on-demand in knowledge/api.rs and hooks/audit.rs
    // rather than shared from DaemonState. Each instance compiles regex patterns, so this
    // could be optimized by sharing a single instance if performance becomes an issue.
    pub persistence: Option<Arc<crate::persistence::PersistenceLayer>>,
    pub metrics_cache: Arc<super::metrics_cache::MetricsCache>,
    pub llm_router_state: Option<super::llm_router::api::LlmRouterState>,
    pub proxy_port: Arc<Mutex<Option<u16>>>,
    pub orchestration_state: Option<Arc<OrchestrationState>>,
    pub llm_gateway: Option<super::llm_gateway::GatewayState>,
    pub gateway_port: Arc<Mutex<Option<u16>>>,
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

                    // Eagerly load model into memory for instant classification
                    info!("📥 Loading CRUD classifier model into memory...");
                    match classifier.preload_model().await {
                        Ok(()) => {
                            info!("✅ CRUD classifier model loaded and ready for instant classification");
                        }
                        Err(e) => {
                            warn!("Failed to preload model: {}", e);
                            warn!("Model will be lazy-loaded on first request (2s+ delay expected)");
                        }
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

        // Initialize persistence layer for metrics storage
        let persistence = {
            let db_path = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".cco")
                .join("metrics.db");

            match crate::persistence::PersistenceLayer::new(&db_path).await {
                Ok(p) => {
                    info!("✅ Metrics persistence layer initialized: {:?}", db_path);
                    Some(Arc::new(p))
                }
                Err(e) => {
                    warn!("Failed to initialize metrics persistence: {}", e);
                    warn!("Claude metrics will not be persisted to database");
                    None
                }
            }
        };

        // Initialize metrics cache (1 hour of 1-second samples)
        let metrics_cache = Arc::new(super::metrics_cache::MetricsCache::new(3600));
        info!("✅ Metrics cache initialized (capacity: 3600 samples)");

        // Initialize LLM router
        let llm_router_state = match super::llm_router::LlmRouter::from_orchestra_config(None) {
            Ok(router) => {
                info!("✅ LLM router initialized successfully");
                Some(super::llm_router::api::LlmRouterState::new(router))
            }
            Err(e) => {
                info!("ℹ️  LLM router not available: {}", e);
                info!("   LLM routing API endpoints will not be available");
                None
            }
        };

        // Initialize orchestration state (for multi-agent coordination)
        let orchestration_state = match init_orchestration_state(None) {
            Ok(state) => {
                info!("✅ Orchestration system initialized successfully");
                Some(state)
            }
            Err(e) => {
                warn!("Failed to initialize orchestration system: {}", e);
                warn!("Orchestration API endpoints will not be available");
                None
            }
        };

        // Initialize LLM Gateway (unified gateway for all LLM requests)
        let llm_gateway = match super::llm_gateway::config::load_from_orchestra_config(None) {
            Ok(gateway_config) => {
                match super::llm_gateway::LlmGateway::new(gateway_config).await {
                    Ok(gateway) => {
                        info!("✅ LLM Gateway initialized successfully");
                        Some(Arc::new(gateway))
                    }
                    Err(e) => {
                        warn!("Failed to initialize LLM Gateway: {}", e);
                        warn!("LLM Gateway API endpoints will not be available");
                        None
                    }
                }
            }
            Err(e) => {
                info!("ℹ️  LLM Gateway config not found: {}", e);
                info!("   LLM Gateway will not be available (add llmGateway section to orchestra-config.json)");
                None
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
            persistence,
            metrics_cache,
            llm_router_state,
            proxy_port: Arc::new(Mutex::new(None)),
            orchestration_state,
            llm_gateway,
            gateway_port: Arc::new(Mutex::new(None)),
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
    let (classifier_available, model_loaded, model_name) =
        if let Some(ref classifier) = state.crud_classifier {
            let loaded = classifier.is_model_loaded().await;
            (true, loaded, state.config.hooks.llm.model_name.clone())
        } else {
            (false, false, "none".to_string())
        };

    // Get actual port from PID file (not config port which may be 0)
    let actual_port = super::read_daemon_port().unwrap_or(state.config.port);

    // Log health check at debug level to avoid spam
    debug!(
        endpoint = "/health",
        status = "ok",
        uptime_seconds = %uptime,
        port = %actual_port,
        "Health check"
    );

    // Get last classification latency from state
    let classification_latency_ms = state
        .last_classification_ms
        .lock()
        .ok()
        .and_then(|guard| *guard);

    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::version::DateVersion::current().to_string(),
        uptime_seconds: uptime,
        port: actual_port,
        hooks: HooksHealthStatus {
            enabled: state.config.hooks.is_enabled(),
            classifier_available,
            model_loaded,
            model_name,
            classification_latency_ms,
        },
    })
}

/// Command classification endpoint
async fn classify_command(
    State(state): State<Arc<DaemonState>>,
    Json(request): Json<ClassifyRequest>,
) -> Result<Json<ClassifyResponse>, AppError> {
    let start = std::time::Instant::now();

    // Get classifier from state
    let classifier = match state.crud_classifier.as_ref() {
        Some(c) => c,
        None => {
            info!(
                endpoint = "/api/classify",
                command = %request.command,
                error = "classifier unavailable",
                "CLASSIFY request failed - classifier not available"
            );
            return Err(AppError::ClassifierUnavailable);
        }
    };

    // Classify the command
    let result = match classifier.classify(&request.command).await {
        result => result,
    };

    let latency_ms = start.elapsed().as_millis() as u32;
    let classification_str = format!("{:?}", result.classification);

    // Log the classification result with structured fields
    info!(
        endpoint = "/api/classify",
        command = %request.command,
        classification = %classification_str,
        confidence = %result.confidence,
        latency_ms = %latency_ms,
        "CLASSIFY request completed"
    );

    // Update latency tracking
    if let Ok(mut last_ms) = state.last_classification_ms.lock() {
        *last_ms = Some(latency_ms);
    }

    // Track decision (auto-approved for now)
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
        reasoning: result
            .reasoning
            .unwrap_or_else(|| "No reasoning provided".to_string()),
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
    let start = std::time::Instant::now();

    // Parse classification
    let classification: CrudClassification = match payload.classification.parse() {
        Ok(c) => c,
        Err(e) => {
            info!(
                endpoint = "/api/hooks/permission-request",
                command = %payload.command,
                classification = %payload.classification,
                error = %e,
                "PERMISSION request failed - invalid classification"
            );
            return Err(AppError::ClassificationFailed(format!(
                "Invalid classification: {}",
                e
            )));
        }
    };

    // Get classifier to get confidence score
    let classifier = match state.crud_classifier.as_ref() {
        Some(c) => c,
        None => {
            info!(
                endpoint = "/api/hooks/permission-request",
                command = %payload.command,
                classification = %classification,
                error = "classifier unavailable",
                "PERMISSION request failed - classifier not available"
            );
            return Err(AppError::ClassifierUnavailable);
        }
    };

    // Classify the command to get confidence
    let classification_result = classifier.classify(&payload.command).await;

    let latency_ms = start.elapsed().as_millis() as u32;

    // Process permission request
    let response = state
        .permission_handler
        .process_classification(&payload.command, classification_result.clone())
        .await;

    // Log permission decision with structured fields
    info!(
        endpoint = "/api/hooks/permission-request",
        command = %payload.command,
        classification = %classification,
        decision = %response.decision,
        confidence = %response.confidence.unwrap_or(0.0),
        latency_ms = %latency_ms,
        denied = %(response.decision == super::hooks::PermissionDecision::Denied),
        "PERMISSION request completed"
    );

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
            response_time_ms: Some(latency_ms as i32),
        };

        // Spawn async task to store decision (don't block response)
        let audit_db_clone = Arc::clone(audit_db);
        tokio::spawn(async move {
            if let Err(e) = audit_db_clone.store_decision(decision).await {
                warn!("Failed to store decision in audit database: {}", e);
            }
        });
    }

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

/// Stats request query parameters
#[derive(Debug, Deserialize)]
struct StatsQuery {
    time_range: Option<String>, // "today", "week", "month", "all"
}

/// Stats response for TUI
#[derive(Debug, Serialize)]
struct StatsResponse {
    project: ProjectStats,
    activity: ActivitySummary,
    projects: Vec<ProjectSummary>,
    models: Vec<ModelSummary>,
    history_start_date: Option<String>, // ISO 8601 timestamp of oldest conversation
}

#[derive(Debug, Serialize)]
struct ProjectStats {
    name: String,
    tokens: u64,
    cost: f64,
    messages: u64,
}

#[derive(Debug, Serialize)]
struct ActivitySummary {
    recent_calls: Vec<RecentCall>,
}

#[derive(Debug, Serialize)]
struct RecentCall {
    timestamp: String,
    model: String,
    tokens: u64,
    cost: f64,
}

#[derive(Debug, Serialize)]
struct ProjectSummary {
    name: String,
    cost: f64,
    tokens: u64,
    messages: u64,
}

#[derive(Debug, Serialize)]
struct ModelSummary {
    model: String,
    cost: f64,
    tokens: u64,
    messages: u64,
    percentage: f64,
}

/// Get statistics from metrics cache
async fn get_stats(
    State(_state): State<Arc<DaemonState>>,
    Query(params): Query<StatsQuery>,
) -> Result<Json<StatsResponse>, AppError> {
    // Parse time_range parameter with default to "all"
    let time_range = params.time_range.as_deref().unwrap_or("all");

    // Calculate date cutoff based on time range
    let cutoff_date = match time_range {
        "today" => Some(std::time::SystemTime::now() - std::time::Duration::from_secs(86400)),
        "week" => Some(std::time::SystemTime::now() - std::time::Duration::from_secs(7 * 86400)),
        "month" => Some(std::time::SystemTime::now() - std::time::Duration::from_secs(30 * 86400)),
        _ => None, // "all" = no filtering
    };

    // Parse Claude metrics from all projects using parallel parser with time filter
    info!(
        "Starting to load Claude metrics from all projects (parallel, time_range={})...",
        time_range
    );
    let start = std::time::Instant::now();

    let (metrics, history_start_date) =
        crate::claude_history::load_claude_metrics_with_time_filter(cutoff_date)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to load Claude metrics: {}", e))
            })?;

    let elapsed = start.elapsed();
    info!(
        "Loaded metrics in {:?} ({} projects, {} models)",
        elapsed,
        metrics.project_breakdown.len(),
        metrics.model_breakdown.len()
    );

    // Build project summaries (sorted by cost descending, top 20)
    let mut project_summaries: Vec<ProjectSummary> = metrics
        .project_breakdown
        .values()
        .map(|p| {
            let total_tokens = p.total_input_tokens
                + p.total_output_tokens
                + p.total_cache_creation_tokens
                + p.total_cache_read_tokens;

            ProjectSummary {
                name: p.name.clone(),
                cost: p.total_cost,
                tokens: total_tokens,
                messages: p.message_count,
            }
        })
        .collect();

    project_summaries.sort_by(|a, b| {
        b.cost
            .partial_cmp(&a.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    project_summaries.truncate(20);

    // Build model summaries (all models, sorted by cost descending)
    let mut model_summaries: Vec<ModelSummary> = metrics
        .model_breakdown
        .iter()
        .map(|(name, breakdown)| {
            let total_tokens = breakdown.input_tokens
                + breakdown.output_tokens
                + breakdown.cache_creation_tokens
                + breakdown.cache_read_tokens;

            ModelSummary {
                model: name.clone(),
                cost: breakdown.total_cost,
                tokens: total_tokens,
                messages: breakdown.message_count,
                percentage: if metrics.total_cost > 0.0 {
                    (breakdown.total_cost / metrics.total_cost) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    // Sort by cost descending for stable ordering across refreshes
    model_summaries.sort_by(|a, b| {
        b.cost
            .partial_cmp(&a.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Extract recent API calls from project files (last 10 calls across all projects)
    let recent_calls = extract_recent_api_calls(&metrics).await;

    // Convert history_start_date to ISO 8601 string if available
    let history_start_date_str = history_start_date.and_then(|st| {
        st.duration_since(std::time::UNIX_EPOCH).ok().and_then(|d| {
            chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                .map(|datetime| datetime.to_rfc3339())
        })
    });

    let latency_ms = start.elapsed().as_millis() as u32;

    // Build response
    let response = StatsResponse {
        project: ProjectStats {
            name: "All Projects".to_string(),
            tokens: metrics.total_input_tokens
                + metrics.total_output_tokens
                + metrics.total_cache_creation_tokens
                + metrics.total_cache_read_tokens,
            cost: metrics.total_cost,
            messages: metrics.messages_count,
        },
        activity: ActivitySummary { recent_calls },
        projects: project_summaries,
        models: model_summaries,
        history_start_date: history_start_date_str,
    };

    // Log stats request
    info!(
        endpoint = "/api/stats",
        time_range = %time_range,
        project_count = %response.projects.len(),
        model_count = %response.models.len(),
        total_cost = %response.project.cost,
        latency_ms = %latency_ms,
        "STATS request completed"
    );

    Ok(Json(response))
}

/// Extract recent API calls from all projects
async fn extract_recent_api_calls(
    metrics: &crate::claude_history::ClaudeMetrics,
) -> Vec<RecentCall> {
    use std::path::PathBuf;
    use tokio::fs;

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let projects_dir = PathBuf::from(&home).join(".claude").join("projects");

    if !projects_dir.exists() {
        return vec![];
    }

    let mut all_calls: Vec<RecentCall> = Vec::new();

    // Iterate through all projects and collect recent messages
    for (project_name, _project_breakdown) in &metrics.project_breakdown {
        let project_path = projects_dir.join(project_name);

        if !project_path.exists() {
            continue;
        }

        // Read the most recent JSONL file in the project
        if let Ok(mut entries) = fs::read_dir(&project_path).await {
            let mut jsonl_files: Vec<PathBuf> = Vec::new();

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    jsonl_files.push(path);
                }
            }

            // Sort by modification time (most recent first)
            jsonl_files.sort_by_cached_key(|path| {
                std::fs::metadata(path)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });
            jsonl_files.reverse();

            // Parse the most recent file (limit to 3 most recent files per project for efficiency)
            for file_path in jsonl_files.iter().take(3) {
                if let Ok((messages, _, _)) =
                    crate::claude_history::parse_jsonl_file_from_offset(file_path, 0).await
                {
                    for (model, usage, timestamp) in messages {
                        let total_tokens = usage.input_tokens.unwrap_or(0)
                            + usage.output_tokens.unwrap_or(0)
                            + usage.cache_creation_input_tokens.unwrap_or(0)
                            + usage.cache_read_input_tokens.unwrap_or(0);

                        // Calculate cost for this call
                        let (input_price, output_price, cache_write_price, cache_read_price) =
                            crate::claude_history::get_model_pricing(&model);

                        let input_cost = crate::claude_history::calculate_cost(
                            usage.input_tokens.unwrap_or(0),
                            input_price,
                        );
                        let output_cost = crate::claude_history::calculate_cost(
                            usage.output_tokens.unwrap_or(0),
                            output_price,
                        );
                        let cache_write_cost = crate::claude_history::calculate_cost(
                            usage.cache_creation_input_tokens.unwrap_or(0),
                            cache_write_price,
                        );
                        let cache_read_cost = crate::claude_history::calculate_cost(
                            usage.cache_read_input_tokens.unwrap_or(0),
                            cache_read_price,
                        );

                        let total_cost =
                            input_cost + output_cost + cache_write_cost + cache_read_cost;

                        all_calls.push(RecentCall {
                            timestamp: timestamp.unwrap_or_else(|| Utc::now().to_rfc3339()),
                            model: crate::claude_history::normalize_model_name(&model),
                            tokens: total_tokens,
                            cost: total_cost,
                        });
                    }
                }
            }
        }
    }

    // Sort by timestamp descending (most recent first)
    // Let TUI dynamically limit based on available height
    all_calls.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    all_calls
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
    let token_manager = state.token_manager.as_ref().ok_or(AppError::InternalError(
        "Token manager not available".to_string(),
    ))?;

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
        .route("/api/stats", get(get_stats))
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
                        super::security::AuthMiddleware::authenticate(token_manager, req, next)
                            .await
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

    // Mount LLM router routes if available
    if let Some(ref llm_router_state) = state.llm_router_state {
        info!("Mounting LLM router API routes at /api/llm/*");

        // Create LLM router routes
        let mut llm_routes = super::llm_router::llm_router_routes();

        if let Some(ref token_manager) = state.token_manager {
            // Apply authentication middleware to all LLM routes
            let auth_layer = middleware::from_fn({
                let token_manager = Arc::clone(token_manager);
                move |req, next| {
                    let token_manager = Arc::clone(&token_manager);
                    async move {
                        super::security::AuthMiddleware::authenticate(token_manager, req, next)
                            .await
                    }
                }
            });

            llm_routes = llm_routes.layer(auth_layer);
        } else {
            warn!("Token manager not available - LLM routes will NOT be authenticated");
        }

        // Apply LLM router state and nest under main router
        let llm_routes_with_state = llm_routes.with_state(llm_router_state.clone());
        router = router.nest("/", llm_routes_with_state);
    } else {
        info!("LLM router not available - skipping LLM routing API routes");
    }

    // Mount orchestration routes if orchestration state is available
    if let Some(ref orchestration_state) = state.orchestration_state {
        info!("Mounting orchestration API routes at /api/orchestration/*");

        // Create orchestration handler state
        let orchestration_handler_state = OrchestrationHandlerState::new(Arc::clone(orchestration_state));

        // Create orchestration routes with state
        let orchestration_routes = create_orchestration_router()
            .with_state(orchestration_handler_state);

        // Nest under /api/orchestration
        router = router.nest("/api/orchestration", orchestration_routes);
    } else {
        info!("Orchestration system not available - skipping orchestration API routes");
    }

    // Note: LLM Gateway runs on a separate port for Claude Code compatibility
    // See run_daemon_server() where it's started alongside the main daemon

    router.with_state(state)
}

/// Run the daemon HTTP server
///
/// Returns the actual port the server bound to (useful when port 0 is specified)
pub async fn run_daemon_server(config: DaemonConfig) -> anyhow::Result<u16> {
    info!("🚀 Starting CCO Daemon Server");
    info!("   Version: {}", crate::version::DateVersion::current());
    info!(
        "   Requested Port: {} (0 = random OS-assigned port)",
        config.port
    );
    info!("   Host: {}", config.host);
    info!("   Hooks enabled: {}", config.hooks.is_enabled());

    // Initialize daemon state
    let state = Arc::new(DaemonState::new(config.clone()).await?);

    // Create router (clone state for router while keeping reference for logging)
    let app = create_router(Arc::clone(&state));

    // Bind to address (port 0 means OS assigns random port)
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    // Get the actual bound port (important when port 0 is used)
    let actual_addr = listener
        .local_addr()
        .map_err(|e| anyhow::anyhow!("Failed to get local address: {}", e))?;
    let actual_port = actual_addr.port();

    // CRITICAL: Update PID file with actual port IMMEDIATELY after binding
    // This must happen BEFORE starting the server so clients can discover the port
    if let Err(e) = super::update_daemon_port(actual_port) {
        warn!("Failed to update PID file with actual port: {}", e);
        warn!("Clients may not be able to discover daemon port");
    } else {
        info!("✅ PID file updated with actual port: {}", actual_port);
    }

    // CRITICAL: Update settings file with actual port for Claude Code hooks
    // Claude Code reads .cco-orchestrator-settings to discover the daemon port
    let temp_manager = super::TempFileManager::new();
    if let Err(e) = temp_manager.update_settings_port(actual_port) {
        warn!("Failed to update settings file with actual port: {}", e);
        warn!("Claude Code hooks may fail to connect");
    } else {
        info!("✅ Settings file updated with actual port: {}", actual_port);
    }

    // Start the HTTP proxy server for model routing (random port)
    // This proxies Claude API requests and routes reviewer/tester agents to Azure
    let proxy_addr = "127.0.0.1:0"; // Random port
    match super::proxy::start_proxy_server(proxy_addr).await {
        Ok(proxy_port) => {
            info!("✅ Proxy server started on port {}", proxy_port);

            // Store proxy port in state
            if let Ok(mut port_guard) = state.proxy_port.lock() {
                *port_guard = Some(proxy_port);
            }

            // Update PID file with proxy port for launcher discovery
            if let Err(e) = super::update_proxy_port(proxy_port) {
                warn!("Failed to update PID file with proxy port: {}", e);
            } else {
                info!("✅ PID file updated with proxy port: {}", proxy_port);
            }
        }
        Err(e) => {
            warn!("Failed to start proxy server: {}", e);
            warn!("Model routing will be disabled - all requests go to Claude");
        }
    }

    // Start the LLM Gateway server if available (random port)
    // This provides Anthropic-compatible API with cost tracking and audit logging
    if let Some(ref llm_gateway) = state.llm_gateway {
        let gateway_addr = "127.0.0.1:0";
        let gateway_app = super::llm_gateway::api::gateway_router_with_state(Arc::clone(llm_gateway));

        match TcpListener::bind(gateway_addr).await {
            Ok(gateway_listener) => {
                match gateway_listener.local_addr() {
                    Ok(gateway_actual_addr) => {
                        let gateway_port = gateway_actual_addr.port();

                        // Store gateway port in state
                        if let Ok(mut port_guard) = state.gateway_port.lock() {
                            *port_guard = Some(gateway_port);
                        }

                        info!("✅ LLM Gateway started on port {}", gateway_port);
                        info!("   Set ANTHROPIC_BASE_URL=http://127.0.0.1:{} to use", gateway_port);

                        // Update PID file with gateway port for launcher discovery
                        if let Err(e) = super::update_gateway_port(gateway_port) {
                            warn!("Failed to update PID file with gateway port: {}", e);
                        } else {
                            info!("✅ PID file updated with gateway port: {}", gateway_port);
                        }

                        // Spawn gateway server in background
                        tokio::spawn(async move {
                            if let Err(e) = axum::serve(gateway_listener, gateway_app).await {
                                warn!("LLM Gateway server error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Failed to get gateway local address: {}", e);
                        warn!("LLM Gateway will not be available");
                    }
                }
            }
            Err(e) => {
                warn!("Failed to start LLM Gateway server: {}", e);
                warn!("LLM Gateway will not be available");
            }
        }
    }

    info!("✅ Daemon server listening on http://{}", actual_addr);
    info!("   Actual Port: {}", actual_port);
    info!("   Health: http://{}/health", actual_addr);
    info!("   Classify: http://{}/api/classify", actual_addr);
    info!(
        "   Permission: http://{}/api/hooks/permission-request",
        actual_addr
    );
    info!("   Decisions: http://{}/api/hooks/decisions", actual_addr);
    info!("   Stats: http://{}/api/stats", actual_addr);
    info!("   Shutdown: http://{}/api/shutdown", actual_addr);

    if state.token_manager.is_some() {
        info!(
            "   Token Generation: http://{}/api/token/generate",
            actual_addr
        );
    }

    if state.knowledge_store.is_some() {
        info!("   Knowledge API: http://{}/api/knowledge/*", actual_addr);
        info!("     - POST /api/knowledge/store - Store knowledge item");
        info!("     - POST /api/knowledge/search - Search knowledge base");
        info!("     - GET  /api/knowledge/stats - Knowledge statistics");

        if state.token_manager.is_some() {
            info!("     (All knowledge routes require Bearer token authentication)");
        } else {
            info!("     (WARNING: Authentication disabled - no token manager)");
        }
    }

    // Display proxy port if available
    if let Ok(port_guard) = state.proxy_port.lock() {
        if let Some(proxy_port) = *port_guard {
            info!(
                "   Proxy: http://127.0.0.1:{} (model router for Azure)",
                proxy_port
            );
        }
    }

    // Display orchestration endpoints if available
    if state.orchestration_state.is_some() {
        info!(
            "   Orchestration API: http://{}/api/orchestration/*",
            actual_addr
        );
        info!("     - GET  /api/orchestration/status - System status");
        info!("     - GET  /api/orchestration/context/:issue/:agent - Get agent context");
        info!("     - POST /api/orchestration/results - Store agent results");
        info!("     - POST /api/orchestration/events/:type - Publish event");
        info!("     - GET  /api/orchestration/events/wait/:type - Subscribe to events");
        info!("     - POST /api/orchestration/agents/spawn - Spawn agent");
        info!("     - GET  /api/orchestration/agents/:id/status - Agent status");
    }

    info!("");
    info!("Press Ctrl+C to stop");

    // Spawn background task to parse Claude logs with filesystem watching + periodic fallback
    if let Some(ref persistence) = state.persistence {
        let persistence_clone = Arc::clone(persistence);

        // Helper async function to parse and store metrics
        async fn parse_and_store_metrics(
            persistence: Arc<crate::persistence::PersistenceLayer>,
            metrics_cache: Arc<super::metrics_cache::MetricsCache>,
            trigger: &str,
        ) {
            match crate::claude_history::load_claude_metrics_from_home_dir_parallel().await {
                Ok(metrics) => {
                    debug!(
                        "Parsed Claude metrics ({}): {} messages, ${:.4} total cost",
                        trigger, metrics.messages_count, metrics.total_cost
                    );

                    // Calculate total tokens
                    let total_tokens = metrics.total_input_tokens
                        + metrics.total_output_tokens
                        + metrics.total_cache_creation_tokens
                        + metrics.total_cache_read_tokens;

                    // Update metrics cache (for /api/stats endpoint)
                    let snapshot = super::metrics_cache::StatsSnapshot {
                        timestamp: std::time::SystemTime::now(),
                        total_requests: metrics.messages_count,
                        successful_requests: metrics.messages_count, // Assume all successful for now
                        failed_requests: 0,
                        avg_response_time: 0.0, // Not tracked yet
                        uptime: std::time::Duration::from_secs(0), // Will be calculated by endpoint
                        port: 0,                // Will be set by endpoint
                        total_cost: metrics.total_cost,
                        total_tokens,
                        messages_count: metrics.messages_count,
                    };

                    metrics_cache.update(snapshot);
                    debug!("Updated metrics cache with latest stats");

                    // Get Claude history persistence interface
                    let claude_history = persistence.claude_history();

                    // Store aggregated metrics in database
                    if let Err(e) = claude_history.store_aggregated_metrics(&metrics).await {
                        warn!("Failed to store Claude metrics in database: {}", e);
                    } else {
                        debug!("Successfully persisted Claude metrics to database");
                    }
                }
                Err(e) => {
                    // Log at debug level to avoid spam - missing metrics file is expected
                    debug!("Failed to parse Claude metrics ({}): {}", trigger, e);
                }
            }
        }

        // Initialize filesystem watcher for immediate parsing on file changes
        let claude_history_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".claude")
            .join("projects");

        let metrics_cache_for_watcher = Arc::clone(&state.metrics_cache);
        match LogWatcher::new(claude_history_dir.clone()) {
            Ok((mut log_watcher, mut file_event_rx)) => {
                // Start watching
                if let Err(e) = log_watcher.start() {
                    warn!("Failed to start log watcher: {}", e);
                    warn!("Falling back to periodic polling only");
                } else {
                    info!(
                        "✅ Log watcher started for: {}",
                        claude_history_dir.display()
                    );

                    // Spawn task to handle file change events
                    let persistence_for_watcher = Arc::clone(&persistence_clone);
                    let cache_for_watcher = Arc::clone(&metrics_cache_for_watcher);
                    tokio::spawn(async move {
                        while let Some(changed_path) = file_event_rx.recv().await {
                            info!("📝 Detected change in log file: {:?}", changed_path);

                            // Parse immediately on file change
                            let persistence = Arc::clone(&persistence_for_watcher);
                            let cache = Arc::clone(&cache_for_watcher);
                            parse_and_store_metrics(persistence, cache, "file-change").await;
                        }
                    });
                }
            }
            Err(e) => {
                warn!("Failed to initialize log watcher: {}", e);
                warn!("Falling back to periodic polling only");
            }
        }

        // Perform initial full history scan on startup
        info!("🔄 Performing initial full history scan...");
        let initial_persistence = Arc::clone(&persistence_clone);
        let initial_cache = Arc::clone(&state.metrics_cache);
        parse_and_store_metrics(initial_persistence, initial_cache, "initial-scan").await;

        // Log completion with metrics
        if let Ok(metrics) =
            crate::claude_history::load_claude_metrics_from_home_dir_parallel().await
        {
            info!(
                "✅ Initial history scan complete: {} messages, ${:.2} total cost",
                metrics.messages_count, metrics.total_cost
            );
        }

        // Spawn periodic fallback task (5s interval) to catch anything the watcher misses
        let metrics_cache_for_periodic = Arc::clone(&state.metrics_cache);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                let persistence = Arc::clone(&persistence_clone);
                let cache = Arc::clone(&metrics_cache_for_periodic);
                parse_and_store_metrics(persistence, cache, "periodic").await;
            }
        });

        info!("✅ Background Claude log parser started (filesystem watcher + 5s fallback)");
    } else {
        warn!("Persistence layer not available - Claude log parsing disabled");
    }

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    // Clean up PID file on graceful shutdown
    if let Ok(pid_file) = super::get_daemon_pid_file() {
        if pid_file.exists() {
            if let Err(e) = std::fs::remove_file(&pid_file) {
                warn!("Failed to remove PID file on shutdown: {}", e);
            } else {
                info!("Cleaned up PID file on shutdown");
            }
        }
    }

    info!("Daemon server shut down gracefully");
    Ok(actual_port)
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
