//! Orchestration API Routes for Daemon
//!
//! Provides HTTP endpoints for multi-agent coordination, previously served
//! by the standalone orchestration sidecar on port 3001. Now integrated
//! into the main daemon server.
//!
//! Endpoints:
//! - GET  /api/orchestration/status - Orchestration system status
//! - GET  /api/orchestration/context/:issue_id/:agent_type - Get agent context
//! - POST /api/orchestration/results - Store agent results
//! - POST /api/orchestration/events/:event_type - Publish event
//! - GET  /api/orchestration/events/wait/:event_type - Subscribe to events
//! - POST /api/orchestration/agents/spawn - Spawn agent
//! - GET  /api/orchestration/agents/:agent_id/status - Get agent status
//! - DELETE /api/orchestration/cache/context/:issue_id - Clear context cache

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use crate::orchestration::{OrchestrationState, ServerConfig};

/// Combined state for orchestration handlers
#[derive(Clone)]
pub struct OrchestrationHandlerState {
    pub orchestration: Arc<OrchestrationState>,
    pub active_agents: Arc<AtomicUsize>,
    pub start_time: std::time::Instant,
}

impl OrchestrationHandlerState {
    pub fn new(orchestration: Arc<OrchestrationState>) -> Self {
        Self {
            orchestration,
            active_agents: Arc::new(AtomicUsize::new(0)),
            start_time: std::time::Instant::now(),
        }
    }
}

// ===== REQUEST/RESPONSE TYPES =====

#[derive(Debug, Serialize)]
pub struct OrchestrationStatusResponse {
    pub status: String,
    pub agents: AgentStatus,
    pub storage: StorageStatus,
    pub events: EventStatus,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Serialize)]
pub struct AgentStatus {
    pub active: usize,
    pub completed: usize,
    pub failed: usize,
    pub by_type: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct StorageStatus {
    pub context_cache_entries: usize,
    pub results_stored: usize,
    pub total_size_mb: f64,
}

#[derive(Debug, Serialize)]
pub struct EventStatus {
    pub total_published: usize,
    pub active_subscriptions: usize,
    pub queue_depth: usize,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub avg_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub requests_per_second: f64,
}

#[derive(Debug, Serialize)]
pub struct ContextResponse {
    pub issue_id: String,
    pub agent_type: String,
    pub context: serde_json::Value,
    pub truncated: bool,
    pub truncation_strategy: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ResultRequest {
    pub agent_id: String,
    pub agent_type: String,
    pub issue_id: String,
    pub project_id: String,
    pub result: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ResultResponse {
    pub id: String,
    pub stored: bool,
    pub storage_path: String,
    pub next_agents: Vec<String>,
    pub event_published: bool,
}

#[derive(Debug, Deserialize)]
pub struct EventRequest {
    pub event_type: String,
    pub publisher: String,
    pub topic: String,
    pub data: serde_json::Value,
    pub correlation_id: Option<String>,
    pub ttl_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub event_id: String,
    pub published: bool,
    pub subscribers_notified: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct EventSubscriptionQuery {
    pub timeout: Option<u64>,
    pub filter: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EventSubscriptionResponse {
    pub events: Vec<crate::orchestration::EventData>,
    pub more_available: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpawnAgentRequest {
    pub agent_type: String,
    pub issue_id: String,
    pub task: String,
    pub context_requirements: Vec<String>,
    pub environment: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SpawnAgentResponse {
    pub agent_id: String,
    pub spawned: bool,
    pub process_id: Option<u32>,
    pub context_injected: bool,
    pub webhook_url: String,
}

#[derive(Debug, Serialize)]
pub struct AgentStatusResponse {
    pub agent_id: String,
    pub agent_type: String,
    pub issue_id: String,
    pub status: String,
    pub spawned_at: DateTime<Utc>,
    pub context_available: bool,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ClearCacheResponse {
    pub cleared: bool,
    pub entries_removed: usize,
}

// ===== ERROR HANDLING =====

#[derive(Debug)]
pub enum OrchestrationError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for OrchestrationError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            OrchestrationError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            OrchestrationError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            OrchestrationError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// ===== HANDLER FUNCTIONS =====

/// GET /api/orchestration/status - Orchestration system status
async fn status_handler(
    State(state): State<OrchestrationHandlerState>,
) -> Json<OrchestrationStatusResponse> {
    let active_agents = state.active_agents.load(Ordering::Relaxed);

    Json(OrchestrationStatusResponse {
        status: "healthy".to_string(),
        agents: AgentStatus {
            active: active_agents,
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

/// GET /api/orchestration/context/:issue_id/:agent_type - Get agent context
async fn get_context_handler(
    State(state): State<OrchestrationHandlerState>,
    Path((issue_id, agent_type)): Path<(String, String)>,
) -> Result<Json<ContextResponse>, OrchestrationError> {
    // Gather context using context injector
    let context = state
        .orchestration
        .context_injector
        .gather_context(&agent_type, &issue_id)
        .await
        .map_err(|e| OrchestrationError::InternalError(e.to_string()))?;

    Ok(Json(ContextResponse {
        issue_id,
        agent_type,
        context: serde_json::to_value(&context)
            .map_err(|e| OrchestrationError::InternalError(e.to_string()))?,
        truncated: false,
        truncation_strategy: "none".to_string(),
        timestamp: Utc::now(),
    }))
}

/// POST /api/orchestration/results - Store agent results
async fn store_results_handler(
    State(state): State<OrchestrationHandlerState>,
    Json(request): Json<ResultRequest>,
) -> Result<Json<ResultResponse>, OrchestrationError> {
    // Store result
    let result_id = Uuid::new_v4().to_string();
    state
        .orchestration
        .result_storage
        .store_result(&request.issue_id, &request.agent_type, &request.result)
        .await
        .map_err(|e| OrchestrationError::InternalError(e.to_string()))?;

    // Publish event
    let _event_id = state
        .orchestration
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
        .map_err(|e| OrchestrationError::InternalError(e.to_string()))?;

    Ok(Json(ResultResponse {
        id: result_id,
        stored: true,
        storage_path: format!(
            "{}/results/{}/{}.json",
            state.orchestration.config.storage_path, request.issue_id, request.agent_type
        ),
        next_agents: vec![],
        event_published: true,
    }))
}

/// POST /api/orchestration/events/:event_type - Publish event
async fn publish_event_handler(
    State(state): State<OrchestrationHandlerState>,
    Path(event_type): Path<String>,
    Json(request): Json<EventRequest>,
) -> Result<Json<EventResponse>, OrchestrationError> {
    let event_id = state
        .orchestration
        .event_bus
        .publish(
            &event_type,
            &request.publisher,
            &request.topic,
            request.data,
        )
        .await
        .map_err(|e| OrchestrationError::InternalError(e.to_string()))?;

    Ok(Json(EventResponse {
        event_id,
        published: true,
        subscribers_notified: vec![],
        timestamp: Utc::now(),
    }))
}

/// GET /api/orchestration/events/wait/:event_type - Subscribe to events (long-polling)
async fn subscribe_event_handler(
    State(state): State<OrchestrationHandlerState>,
    Path(event_type): Path<String>,
    Query(query): Query<EventSubscriptionQuery>,
) -> Result<Json<EventSubscriptionResponse>, OrchestrationError> {
    let timeout = query.timeout.unwrap_or(30000);

    let events = state
        .orchestration
        .event_bus
        .wait_for_event(&event_type, timeout)
        .await
        .map_err(|e| OrchestrationError::InternalError(e.to_string()))?;

    Ok(Json(EventSubscriptionResponse {
        events,
        more_available: false,
        next_cursor: None,
    }))
}

/// POST /api/orchestration/agents/spawn - Spawn agent
async fn spawn_agent_handler(
    State(state): State<OrchestrationHandlerState>,
    Json(request): Json<SpawnAgentRequest>,
) -> Result<Json<SpawnAgentResponse>, OrchestrationError> {
    let agent_id = Uuid::new_v4().to_string();

    // Gather context for the agent
    let context = state
        .orchestration
        .context_injector
        .gather_context(&request.agent_type, &request.issue_id)
        .await
        .map_err(|e| OrchestrationError::InternalError(format!("Failed to gather context: {}", e)))?;

    // Publish agent spawn event with all metadata
    let _event_id = state
        .orchestration
        .event_bus
        .publish(
            "agent_spawned",
            &agent_id,
            "orchestration",
            serde_json::json!({
                "agent_id": agent_id,
                "agent_type": request.agent_type,
                "issue_id": request.issue_id,
                "task": request.task,
                "context": context,
                "environment": request.environment,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "status": "ready"
            }),
        )
        .await
        .map_err(|e| OrchestrationError::InternalError(format!("Failed to publish spawn event: {}", e)))?;

    // Increment active agents counter
    state.active_agents.fetch_add(1, Ordering::Relaxed);

    tracing::info!(
        "Agent spawned: {} (type: {}, issue: {})",
        agent_id,
        request.agent_type,
        request.issue_id
    );

    Ok(Json(SpawnAgentResponse {
        agent_id: agent_id.clone(),
        spawned: true,
        process_id: Some(std::process::id()),
        context_injected: true,
        webhook_url: format!("/api/orchestration/agents/{}/status", agent_id),
    }))
}

/// GET /api/orchestration/agents/:agent_id/status - Get agent status
async fn get_agent_status_handler(
    State(state): State<OrchestrationHandlerState>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentStatusResponse>, OrchestrationError> {
    // Wait for agent spawn event (should already exist)
    let events = state
        .orchestration
        .event_bus
        .wait_for_event("agent_spawned", 1000)
        .await
        .map_err(|e| OrchestrationError::NotFound(format!("Agent not found: {}", e)))?;

    // Find the matching agent event
    let agent_event = events
        .iter()
        .find(|e| e.data["agent_id"].as_str() == Some(&agent_id))
        .ok_or_else(|| OrchestrationError::NotFound(format!("Agent {} not found", agent_id)))?;

    let agent_data = &agent_event.data;

    // Parse agent data
    let agent_type = agent_data["agent_type"]
        .as_str()
        .ok_or_else(|| OrchestrationError::InternalError("Invalid agent data".to_string()))?
        .to_string();

    let issue_id = agent_data["issue_id"]
        .as_str()
        .ok_or_else(|| OrchestrationError::InternalError("Invalid agent data".to_string()))?
        .to_string();

    let spawned_at = agent_data["timestamp"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .ok_or_else(|| OrchestrationError::InternalError("Invalid timestamp".to_string()))?;

    // Check if agent has submitted results
    let has_results = state
        .orchestration
        .result_storage
        .has_result(&issue_id, &agent_type)
        .await
        .unwrap_or(false);

    let status = if has_results { "completed" } else { "running" };

    Ok(Json(AgentStatusResponse {
        agent_id,
        agent_type,
        issue_id,
        status: status.to_string(),
        spawned_at,
        context_available: agent_data.get("context").is_some(),
        last_activity: if has_results { Some(Utc::now()) } else { None },
    }))
}

/// DELETE /api/orchestration/cache/context/:issue_id - Clear context cache
async fn clear_context_cache_handler(
    State(state): State<OrchestrationHandlerState>,
    Path(issue_id): Path<String>,
) -> Result<Json<ClearCacheResponse>, OrchestrationError> {
    let removed = state
        .orchestration
        .context_injector
        .clear_cache(&issue_id)
        .await
        .map_err(|e| OrchestrationError::InternalError(e.to_string()))?;

    Ok(Json(ClearCacheResponse {
        cleared: true,
        entries_removed: removed,
    }))
}

// ===== ROUTER CREATION =====

/// Create the orchestration router with all endpoints
///
/// All routes are prefixed with /api/orchestration when mounted
pub fn create_orchestration_router() -> Router<OrchestrationHandlerState> {
    Router::new()
        // Status endpoint
        .route("/status", get(status_handler))
        // Context endpoints
        .route("/context/:issue_id/:agent_type", get(get_context_handler))
        // Results endpoints
        .route("/results", post(store_results_handler))
        // Event bus endpoints
        .route("/events/:event_type", post(publish_event_handler))
        .route("/events/wait/:event_type", get(subscribe_event_handler))
        // Agent management
        .route("/agents/spawn", post(spawn_agent_handler))
        .route("/agents/:agent_id/status", get(get_agent_status_handler))
        // Cache management
        .route("/cache/context/:issue_id", delete(clear_context_cache_handler))
}

/// Initialize orchestration state from config
pub fn init_orchestration_state(config: Option<ServerConfig>) -> anyhow::Result<Arc<OrchestrationState>> {
    let config = config.unwrap_or_default();

    Ok(Arc::new(OrchestrationState {
        config: config.clone(),
        knowledge_broker: crate::orchestration::KnowledgeBroker::new(),
        event_bus: crate::orchestration::EventBus::new(),
        result_storage: crate::orchestration::ResultStorage::new(config.storage_path.clone())?,
        context_injector: crate::orchestration::ContextInjector::new(),
    }))
}
