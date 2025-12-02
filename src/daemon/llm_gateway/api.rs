//! HTTP API endpoints for the LLM Gateway
//!
//! Provides Anthropic-compatible API endpoints:
//! - POST /v1/messages - Main completion endpoint (supports streaming via SSE)
//! - GET /gateway/health - Gateway health check
//! - GET /gateway/metrics - Cost metrics and statistics
//! - GET /gateway/audit - Search audit logs

use axum::{
    body::Body,
    extract::{Json, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{CompletionRequest, GatewayState, LlmGateway};

/// Create the gateway router
pub fn gateway_router() -> Router<GatewayState> {
    Router::new()
        // Anthropic-compatible endpoint
        .route("/v1/messages", post(complete_handler))
        // Gateway management endpoints
        .route("/gateway/health", get(health_handler))
        .route("/gateway/metrics", get(metrics_handler))
        .route("/gateway/metrics/reset", post(reset_metrics_handler))
        .route("/gateway/providers", get(providers_handler))
        .route("/gateway/audit", get(audit_handler))
}

/// Create a stateful gateway router
pub fn gateway_router_with_state(gateway: Arc<LlmGateway>) -> Router {
    gateway_router().with_state(gateway)
}

// ============================================================================
// Handlers
// ============================================================================

/// Main completion endpoint (Anthropic Messages API compatible)
/// Supports both streaming (SSE) and non-streaming (JSON) responses
async fn complete_handler(
    State(gateway): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<CompletionRequest>,
) -> Response {
    // Log incoming headers for debugging
    let auth_from_authorization = headers.get("authorization").and_then(|v| v.to_str().ok());
    let auth_from_x_api_key = headers.get("x-api-key").and_then(|v| v.to_str().ok());
    let anthropic_beta = headers.get("anthropic-beta").and_then(|v| v.to_str().ok());

    tracing::info!(
        model = %request.model,
        agent_type = ?request.agent_type,
        stream = request.stream,
        has_authorization = auth_from_authorization.is_some(),
        has_x_api_key = auth_from_x_api_key.is_some(),
        has_anthropic_beta = anthropic_beta.is_some(),
        anthropic_beta = ?anthropic_beta,
        authorization_prefix = auth_from_authorization.map(|s| &s[..s.len().min(20)]),
        "Received completion request with auth headers"
    );

    // Extract headers for passthrough
    let auth_header = auth_from_authorization
        .or(auth_from_x_api_key)
        .map(|s| s.to_string());
    let beta_header = anthropic_beta.map(|s| s.to_string());

    // Check if streaming is requested
    if request.stream {
        return handle_streaming_request(gateway, request, auth_header, beta_header).await;
    }

    // Non-streaming path
    match gateway.complete(request, auth_header, beta_header).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Completion request failed");
            GatewayError::ProviderError(e.to_string()).into_response()
        }
    }
}

/// Handle streaming completion requests
/// Returns SSE (Server-Sent Events) response forwarded from the provider or LiteLLM
async fn handle_streaming_request(
    gateway: GatewayState,
    request: CompletionRequest,
    auth_header: Option<String>,
    beta_header: Option<String>,
) -> Response {
    // Get the byte stream - use LiteLLM if available, otherwise direct provider
    let byte_stream = if let Some(ref litellm) = gateway.litellm_client {
        tracing::debug!("Using LiteLLM client for streaming request");
        match litellm.complete_stream(request, auth_header, beta_header).await {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!(error = %e, "Failed to start LiteLLM streaming");
                return GatewayError::ProviderError(e.to_string()).into_response();
            }
        }
    } else {
        // Get the provider for this request
        let route = gateway.router.route(&request);
        tracing::debug!(provider = %route.provider, "Using direct provider for streaming");

        let provider = match gateway.providers.get(&route.provider) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(error = %e, "Failed to get provider for streaming");
                return GatewayError::ProviderError(e.to_string()).into_response();
            }
        };

        // Get the byte stream from the provider
        match provider.complete_stream(request, auth_header, beta_header).await {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!(error = %e, "Failed to start streaming");
                return GatewayError::ProviderError(e.to_string()).into_response();
            }
        }
    };

    tracing::info!(
        using_litellm = gateway.litellm_client.is_some(),
        "Streaming response started"
    );

    // Convert the byte stream to an axum Body stream
    // Map reqwest errors to std::io::Error for compatibility
    let body_stream = byte_stream.map(|result| {
        result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    });

    // Build SSE response with proper headers
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .header("x-accel-buffering", "no") // Disable nginx buffering
        .body(Body::from_stream(body_stream))
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "Failed to build streaming response");
            GatewayError::ProviderError(e.to_string()).into_response()
        })
}

/// Gateway health check
async fn health_handler(State(gateway): State<GatewayState>) -> Json<HealthResponse> {
    let provider_health = gateway.providers.health_check_all().await;

    let all_healthy = provider_health.values().all(|&healthy| healthy);

    Json(HealthResponse {
        status: if all_healthy { "healthy" } else { "degraded" }.to_string(),
        providers: provider_health,
        routing: RoutingHealth {
            default_provider: gateway.config.routing.default_provider.clone(),
            fallback_chain: gateway.config.routing.fallback_chain.clone(),
        },
        audit: AuditHealth {
            enabled: gateway.config.audit.enabled,
            log_request_bodies: gateway.config.audit.log_request_bodies,
            log_response_bodies: gateway.config.audit.log_response_bodies,
        },
    })
}

/// Cost metrics endpoint
async fn metrics_handler(
    State(gateway): State<GatewayState>,
    Query(params): Query<MetricsQuery>,
) -> Json<MetricsResponse> {
    let summary = gateway.cost_tracker.summary();

    // Get breakdown by requested dimension
    let by_agent = if params.include_agent.unwrap_or(true) {
        Some(gateway.cost_tracker.by_agent())
    } else {
        None
    };

    let by_provider = if params.include_provider.unwrap_or(true) {
        Some(gateway.cost_tracker.by_provider())
    } else {
        None
    };

    let by_model = if params.include_model.unwrap_or(true) {
        Some(gateway.cost_tracker.by_model())
    } else {
        None
    };

    let by_project = if params.include_project.unwrap_or(false) {
        Some(gateway.cost_tracker.by_project())
    } else {
        None
    };

    // Get recent requests for display
    let limit = params.recent_limit.unwrap_or(20);
    let recent = gateway.cost_tracker.recent_requests(limit);

    Json(MetricsResponse {
        summary: MetricsSummary {
            total_requests: summary.total_requests,
            total_cost_usd: summary.total_cost_usd,
            total_input_tokens: summary.total_input_tokens,
            total_output_tokens: summary.total_output_tokens,
            total_cache_write_tokens: summary.total_cache_write_tokens,
            total_cache_read_tokens: summary.total_cache_read_tokens,
            avg_latency_ms: summary.avg_latency_ms,
        },
        by_agent,
        by_provider,
        by_model,
        by_project,
        recent: recent
            .into_iter()
            .map(|r| RecentRequest {
                request_id: r.request_id,
                timestamp: r.timestamp.to_rfc3339(),
                provider: r.provider,
                model: r.model,
                agent_type: r.agent_type,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                cost_usd: r.cost_usd,
                latency_ms: r.latency_ms,
            })
            .collect(),
    })
}

/// Reset metrics endpoint
async fn reset_metrics_handler(State(gateway): State<GatewayState>) -> Json<ResetResponse> {
    gateway.cost_tracker.reset();

    Json(ResetResponse {
        status: "ok".to_string(),
        message: "Metrics reset successfully".to_string(),
    })
}

/// List providers endpoint
async fn providers_handler(State(gateway): State<GatewayState>) -> Json<ProvidersResponse> {
    let provider_names = gateway.providers.list();
    let provider_health = gateway.providers.health_check_all().await;

    let providers: Vec<ProviderInfo> = provider_names
        .into_iter()
        .map(|name| {
            let healthy = provider_health.get(&name).copied().unwrap_or(false);
            ProviderInfo {
                name: name.clone(),
                healthy,
            }
        })
        .collect();

    Json(ProvidersResponse { providers })
}

/// Audit log search endpoint
async fn audit_handler(
    State(gateway): State<GatewayState>,
    Query(params): Query<AuditQueryParams>,
) -> Result<Json<AuditResponse>, GatewayError> {
    let logger = gateway.audit_logger.read().await;

    // Convert API params to internal query format
    let query = super::audit::AuditQuery {
        provider: params.provider.clone(),
        agent_type: params.agent_type.clone(),
        project_id: params.project_id.clone(),
        status: params.success.map(|s| if s { "success".to_string() } else { "error".to_string() }),
        from_timestamp: params.start_time.as_deref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
        to_timestamp: params.end_time.as_deref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
        limit: params.limit.map(|l| l as usize),
    };

    let audit_entries = logger
        .search(query)
        .await
        .map_err(|e| GatewayError::AuditError(e.to_string()))?;

    let include_bodies = params.include_bodies.unwrap_or(false);

    // Convert to response format
    let entries: Vec<AuditEntry> = audit_entries
        .into_iter()
        .map(|e| AuditEntry {
            id: e.id.clone(),
            request_id: e.id.clone(), // request_id is the same as id in our schema
            timestamp: e.timestamp,
            agent_type: e.agent_type,
            provider: e.provider,
            model: e.model,
            success: e.status == "success",
            latency_ms: e.latency_ms as i64,
            input_tokens: Some(e.input_tokens as i64),
            output_tokens: Some(e.output_tokens as i64),
            cost_usd: Some(e.cost_usd),
            error_message: e.error_message,
            request_body: if include_bodies { e.request_body } else { None },
            response_body: if include_bodies { e.response_body } else { None },
        })
        .collect();

    let total = entries.len() as i64;
    Ok(Json(AuditResponse { entries, total }))
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum GatewayError {
    ProviderError(String),
    AuditError(String),
    ValidationError(String),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            GatewayError::ProviderError(msg) => {
                (StatusCode::BAD_GATEWAY, "provider_error", msg)
            }
            GatewayError::AuditError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "audit_error", msg)
            }
            GatewayError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, "validation_error", msg)
            }
        };

        let body = ErrorResponse {
            error: ErrorDetail {
                error_type: error_type.to_string(),
                message,
            },
        };

        (status, Json(body)).into_response()
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub providers: std::collections::HashMap<String, bool>,
    pub routing: RoutingHealth,
    pub audit: AuditHealth,
}

#[derive(Debug, Serialize)]
pub struct RoutingHealth {
    pub default_provider: String,
    pub fallback_chain: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditHealth {
    pub enabled: bool,
    pub log_request_bodies: bool,
    pub log_response_bodies: bool,
}

/// Metrics query parameters
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub include_agent: Option<bool>,
    pub include_provider: Option<bool>,
    pub include_model: Option<bool>,
    pub include_project: Option<bool>,
    pub recent_limit: Option<usize>,
}

/// Metrics response
#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub summary: MetricsSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_agent: Option<std::collections::HashMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_provider: Option<std::collections::HashMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_model: Option<std::collections::HashMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_project: Option<std::collections::HashMap<String, f64>>,
    pub recent: Vec<RecentRequest>,
}

#[derive(Debug, Serialize)]
pub struct MetricsSummary {
    pub total_requests: u64,
    pub total_cost_usd: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_write_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct RecentRequest {
    pub request_id: String,
    pub timestamp: String,
    pub provider: String,
    pub model: String,
    pub agent_type: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
    pub latency_ms: u64,
}

/// Reset response
#[derive(Debug, Serialize)]
pub struct ResetResponse {
    pub status: String,
    pub message: String,
}

/// Providers response
#[derive(Debug, Serialize)]
pub struct ProvidersResponse {
    pub providers: Vec<ProviderInfo>,
}

#[derive(Debug, Serialize)]
pub struct ProviderInfo {
    pub name: String,
    pub healthy: bool,
}

/// Audit query parameters (API version)
#[derive(Debug, Deserialize)]
pub struct AuditQueryParams {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub agent_type: Option<String>,
    pub provider: Option<String>,
    pub project_id: Option<String>,
    pub success: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub include_bodies: Option<bool>,
}

/// Audit response
#[derive(Debug, Serialize)]
pub struct AuditResponse {
    pub entries: Vec<AuditEntry>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub id: String,
    pub request_id: String,
    pub timestamp: String,
    pub agent_type: Option<String>,
    pub provider: String,
    pub model: String,
    pub success: bool,
    pub latency_ms: i64,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_usd: Option<f64>,
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
}

/// Error response format (Anthropic-compatible)
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: ErrorDetail {
                error_type: "provider_error".to_string(),
                message: "Connection failed".to_string(),
            },
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"type\":\"provider_error\""));
        assert!(json.contains("\"message\":\"Connection failed\""));
    }

    #[test]
    fn test_metrics_query_defaults() {
        let json = "{}";
        let query: MetricsQuery = serde_json::from_str(json).unwrap();
        assert!(query.include_agent.is_none());
        assert!(query.include_provider.is_none());
        assert!(query.include_model.is_none());
    }

    #[test]
    fn test_audit_query_parsing() {
        let json = r#"{"agent_type":"code-reviewer","limit":50}"#;
        let query: AuditQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(query.agent_type, Some("code-reviewer".to_string()));
        assert_eq!(query.limit, Some(50));
    }
}
