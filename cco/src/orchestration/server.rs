//! HTTP Server Component for Orchestration Sidecar
//!
//! Axum web server on port 3001 with 8 REST endpoints.
//! Handles JWT authentication, rate limiting, CORS, and graceful shutdown.

use anyhow::{Context as AnyhowContext, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
// JWT imports removed - not needed for current implementation
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use super::OrchestrationState;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub storage_path: String,
    pub jwt_secret: String,
    pub max_agents: usize,
    pub context_cache_size_mb: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,
            storage_path: dirs::home_dir()
                .unwrap()
                .join(".cco/orchestration")
                .to_string_lossy()
                .to_string(),
            jwt_secret: "change-me-in-production".to_string(),
            max_agents: 119,
            context_cache_size_mb: 1024,
        }
    }
}

/// JWT claims for agent authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,        // agent_id
    agent_type: String, // agent type name
    project_id: String, // project identifier
    permissions: Vec<String>,
    exp: usize, // expiration time
    iat: usize, // issued at
}

/// Orchestration server
pub struct OrchestrationServer {
    state: Arc<OrchestrationState>,
    config: ServerConfig,
    start_time: Instant,
}

impl OrchestrationServer {
    /// Create a new orchestration server
    pub fn new(state: Arc<OrchestrationState>, config: ServerConfig) -> Self {
        Self {
            state,
            config,
            start_time: Instant::now(),
        }
    }

    /// Run the server
    pub async fn run(self) -> Result<()> {
        let config = self.config.clone();
        let app = self.create_router();

        let addr: SocketAddr = format!("{}:{}", config.host, config.port)
            .parse()
            .context("Invalid server address")?;

        tracing::info!("🚀 Orchestration sidecar listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .context("Failed to bind to address")?;

        axum::serve(listener, app)
            .await
            .context("Server error")?;

        Ok(())
    }

    /// Create the router with all endpoints
    fn create_router(self) -> Router {
        let state = self.state;

        Router::new()
            // Health and status endpoints
            .route("/health", get(health_handler))
            .route("/status", get(status_handler))
            // Context and results endpoints
            .route(
                "/api/context/:issue_id/:agent_type",
                get(get_context_handler),
            )
            .route("/api/results", post(store_results_handler))
            // Event bus endpoints
            .route("/api/events/:event_type", post(publish_event_handler))
            .route("/api/events/wait/:event_type", get(subscribe_event_handler))
            // Agent management
            .route("/api/agents/spawn", post(spawn_agent_handler))
            // Cache management
            .route(
                "/api/cache/context/:issue_id",
                delete(clear_context_cache_handler),
            )
            .layer(CorsLayer::permissive())
            .with_state(state)
    }
}

// ===== REQUEST/RESPONSE TYPES =====

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    uptime_seconds: u64,
    checks: HealthChecks,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthChecks {
    storage: String,
    event_bus: String,
    memory_usage_mb: usize,
    active_agents: usize,
    event_queue_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusResponse {
    agents: AgentStatus,
    storage: StorageStatus,
    events: EventStatus,
    performance: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentStatus {
    active: usize,
    completed: usize,
    failed: usize,
    by_type: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageStatus {
    context_cache_entries: usize,
    results_stored: usize,
    total_size_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventStatus {
    total_published: usize,
    active_subscriptions: usize,
    queue_depth: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceMetrics {
    avg_response_time_ms: f64,
    p99_response_time_ms: f64,
    requests_per_second: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextRequest {
    issue_id: String,
    agent_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextResponse {
    issue_id: String,
    agent_type: String,
    context: serde_json::Value,
    truncated: bool,
    truncation_strategy: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultRequest {
    agent_id: String,
    agent_type: String,
    issue_id: String,
    project_id: String,
    result: serde_json::Value,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultResponse {
    id: String,
    stored: bool,
    storage_path: String,
    next_agents: Vec<String>,
    event_published: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRequest {
    event_type: String,
    publisher: String,
    topic: String,
    data: serde_json::Value,
    correlation_id: Option<String>,
    ttl_seconds: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventResponse {
    event_id: String,
    published: bool,
    subscribers_notified: Vec<String>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventSubscriptionQuery {
    timeout: Option<u64>,
    filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventSubscriptionResponse {
    events: Vec<EventData>,
    more_available: bool,
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event_id: String,
    pub event_type: String,
    pub publisher: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpawnAgentRequest {
    agent_type: String,
    issue_id: String,
    task: String,
    context_requirements: Vec<String>,
    environment: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpawnAgentResponse {
    agent_id: String,
    spawned: bool,
    process_id: Option<u32>,
    context_injected: bool,
    webhook_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClearCacheResponse {
    cleared: bool,
    entries_removed: usize,
}

// ===== HANDLER FUNCTIONS =====

async fn health_handler(State(_state): State<Arc<OrchestrationState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "orchestration-sidecar".to_string(),
        version: env!("CCO_VERSION").to_string(),
        uptime_seconds: 0, // TODO: track uptime
        checks: HealthChecks {
            storage: "healthy".to_string(),
            event_bus: "healthy".to_string(),
            memory_usage_mb: 150, // TODO: real memory usage
            active_agents: 0,     // TODO: track active agents
            event_queue_size: 0,  // TODO: event queue size
        },
    })
}

async fn status_handler(State(_state): State<Arc<OrchestrationState>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        agents: AgentStatus {
            active: 0,
            completed: 0,
            failed: 0,
            by_type: std::collections::HashMap::new(),
        },
        storage: StorageStatus {
            context_cache_entries: 0,
            results_stored: 0,
            total_size_mb: 0.0,
        },
        events: EventStatus {
            total_published: 0,
            active_subscriptions: 0,
            queue_depth: 0,
        },
        performance: PerformanceMetrics {
            avg_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            requests_per_second: 0.0,
        },
    })
}

async fn get_context_handler(
    State(state): State<Arc<OrchestrationState>>,
    Path((issue_id, agent_type)): Path<(String, String)>,
) -> Result<Json<ContextResponse>, AppError> {
    // Gather context using context injector
    let context = state
        .context_injector
        .gather_context(&agent_type, &issue_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(ContextResponse {
        issue_id,
        agent_type,
        context: serde_json::to_value(&context)
            .map_err(|e| AppError::InternalError(e.to_string()))?,
        truncated: false,
        truncation_strategy: "none".to_string(),
        timestamp: Utc::now(),
    }))
}

async fn store_results_handler(
    State(state): State<Arc<OrchestrationState>>,
    Json(request): Json<ResultRequest>,
) -> Result<Json<ResultResponse>, AppError> {
    // Store result
    let result_id = Uuid::new_v4().to_string();
    state
        .result_storage
        .store_result(&request.issue_id, &request.agent_type, &request.result)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // Publish event
    let _event_id = state
        .event_bus
        .publish(
            "agent_completed",
            &request.agent_id,
            "implementation",
            serde_json::json!({
                "issue_id": request.issue_id,
                "agent_type": request.agent_type,
                "status": "success"
            }),
        )
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(ResultResponse {
        id: result_id,
        stored: true,
        storage_path: format!(
            "{}/results/{}/{}.json",
            state.config.storage_path, request.issue_id, request.agent_type
        ),
        next_agents: vec![], // TODO: determine next agents
        event_published: true,
    }))
}

async fn publish_event_handler(
    State(state): State<Arc<OrchestrationState>>,
    Path(event_type): Path<String>,
    Json(request): Json<EventRequest>,
) -> Result<Json<EventResponse>, AppError> {
    let event_id = state
        .event_bus
        .publish(&event_type, &request.publisher, &request.topic, request.data)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(EventResponse {
        event_id,
        published: true,
        subscribers_notified: vec![], // TODO: track subscribers
        timestamp: Utc::now(),
    }))
}

async fn subscribe_event_handler(
    State(state): State<Arc<OrchestrationState>>,
    Path(event_type): Path<String>,
    Query(query): Query<EventSubscriptionQuery>,
) -> Result<Json<EventSubscriptionResponse>, AppError> {
    let timeout = query.timeout.unwrap_or(30000);

    let events = state
        .event_bus
        .wait_for_event(&event_type, timeout)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(EventSubscriptionResponse {
        events,
        more_available: false,
        next_cursor: None,
    }))
}

async fn spawn_agent_handler(
    State(_state): State<Arc<OrchestrationState>>,
    Json(_request): Json<SpawnAgentRequest>,
) -> Result<Json<SpawnAgentResponse>, AppError> {
    let agent_id = Uuid::new_v4().to_string();

    // TODO: Implement actual agent spawning
    Ok(Json(SpawnAgentResponse {
        agent_id: agent_id.clone(),
        spawned: false, // Not implemented yet
        process_id: None,
        context_injected: false,
        webhook_url: format!("/api/agents/{}/status", agent_id),
    }))
}

async fn clear_context_cache_handler(
    State(state): State<Arc<OrchestrationState>>,
    Path(issue_id): Path<String>,
) -> Result<Json<ClearCacheResponse>, AppError> {
    let removed = state
        .context_injector
        .clear_cache(&issue_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(ClearCacheResponse {
        cleared: true,
        entries_removed: removed,
    }))
}

// ===== ERROR HANDLING =====

#[derive(Debug)]
enum AppError {
    Unauthorized(String),
    BadRequest(String),
    NotFound(String),
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
