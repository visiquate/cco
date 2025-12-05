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
    response::{IntoResponse, Response, sse::{Event, Sse}},
    routing::{get, post},
    Router,
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::convert::Infallible;

use super::{CompletionRequest, GatewayState, LlmGateway};
use crate::sse::parser::SseParser;

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
        // Streaming events for TUI
        .route("/api/streams/subscribe", get(stream_subscribe_handler))
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
/// Also parses SSE events and broadcasts them to TUI subscribers
async fn handle_streaming_request(
    gateway: GatewayState,
    request: CompletionRequest,
    auth_header: Option<String>,
    beta_header: Option<String>,
) -> Response {
    // Generate request ID for tracking
    let request_id = uuid::Uuid::new_v4().to_string();

    // Detect OAuth: check for Bearer auth OR CLAUDE_CODE_OAUTH_TOKEN env var
    let has_bearer_auth = auth_header
        .as_ref()
        .map(|h| h.to_lowercase().starts_with("bearer "))
        .unwrap_or(false);
    let has_oauth_env = std::env::var("CLAUDE_CODE_OAUTH_TOKEN").map(|t| !t.is_empty()).unwrap_or(false);
    let is_oauth = has_bearer_auth || has_oauth_env;

    // IMPORTANT: For OAuth authentication, bypass LiteLLM and use direct Anthropic provider
    // LiteLLM doesn't handle OAuth Bearer tokens correctly (sends as x-api-key instead of Authorization)
    let use_litellm = gateway.litellm_client.is_some() && !is_oauth;

    tracing::info!(
        request_id = %request_id,
        has_bearer_auth = has_bearer_auth,
        has_oauth_env = has_oauth_env,
        is_oauth = is_oauth,
        litellm_available = gateway.litellm_client.is_some(),
        use_litellm = use_litellm,
        "OAuth detection for streaming request"
    );

    // Get the byte stream - use LiteLLM if available AND not OAuth, otherwise direct provider
    let byte_stream = if use_litellm {
        let litellm = gateway.litellm_client.as_ref().unwrap();
        tracing::debug!(request_id = %request_id, "Using LiteLLM client for streaming request");
        match litellm.complete_stream(request.clone(), auth_header, beta_header).await {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!(request_id = %request_id, error = %e, "Failed to start LiteLLM streaming");
                // Emit error event
                gateway.stream_broadcaster.send(super::sse_broadcast::TuiStreamEvent::Error {
                    request_id: request_id.clone(),
                    message: e.to_string(),
                });
                return GatewayError::ProviderError(e.to_string()).into_response();
            }
        }
    } else {
        // Get the provider for this request
        let route = gateway.router.route(&request);
        tracing::debug!(request_id = %request_id, provider = %route.provider, "Using direct provider for streaming");

        let provider = match gateway.providers.get(&route.provider) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(request_id = %request_id, error = %e, "Failed to get provider for streaming");
                // Emit error event
                gateway.stream_broadcaster.send(super::sse_broadcast::TuiStreamEvent::Error {
                    request_id: request_id.clone(),
                    message: e.to_string(),
                });
                return GatewayError::ProviderError(e.to_string()).into_response();
            }
        };

        // Get the byte stream from the provider
        match provider.complete_stream(request.clone(), auth_header, beta_header).await {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!(request_id = %request_id, error = %e, "Failed to start streaming");
                // Emit error event
                gateway.stream_broadcaster.send(super::sse_broadcast::TuiStreamEvent::Error {
                    request_id: request_id.clone(),
                    message: e.to_string(),
                });
                return GatewayError::ProviderError(e.to_string()).into_response();
            }
        }
    };

    tracing::info!(
        request_id = %request_id,
        using_litellm = use_litellm,
        is_oauth = is_oauth,
        "Streaming response started"
    );

    // Emit Started event
    gateway.stream_broadcaster.send(super::sse_broadcast::TuiStreamEvent::Started {
        request_id: request_id.clone(),
        model: request.model.clone(),
        agent_type: request.agent_type.clone(),
    });

    // Create a stream that:
    // 1. Parses SSE events from the bytes
    // 2. Emits TuiStreamEvents to broadcaster
    // 3. Passes bytes through unchanged
    let broadcaster = gateway.stream_broadcaster.clone();
    let req_id = request_id.clone();

    // Track usage for final completion event using Arc<Mutex<_>> for interior mutability
    use std::sync::Mutex;
    let usage_tracker = Arc::new(Mutex::new((0u32, 0u32))); // (input_tokens, output_tokens)

    let passthrough_stream = byte_stream.map(move |chunk_result| {
        match chunk_result {
            Ok(bytes) => {
                // Parse SSE events from this chunk
                let text = match std::str::from_utf8(&bytes) {
                    Ok(t) => t,
                    Err(_) => {
                        // If not valid UTF-8, just pass through
                        return Ok(bytes);
                    }
                };

                // Parse SSE events (using a temporary parser for each chunk)
                // Note: This is a simplified approach - in production you'd want to maintain
                // parser state across chunks, but for our use case (broadcasting deltas),
                // this is sufficient since we're primarily interested in text_delta events
                let mut parser = SseParser::new();
                let events = parser.process_chunk(text);

                for event in events {
                    // Try to parse as JSON
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        // Handle content_block_delta events with text deltas
                        if json.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                            if let Some(delta_text) = json
                                .get("delta")
                                .and_then(|d| d.get("text"))
                                .and_then(|t| t.as_str())
                            {
                                broadcaster.send(super::sse_broadcast::TuiStreamEvent::TextDelta {
                                    request_id: req_id.clone(),
                                    text: delta_text.to_string(),
                                });
                            }
                        }
                        // Handle message_delta for usage updates
                        else if json.get("type").and_then(|t| t.as_str()) == Some("message_delta") {
                            if let Some(usage) = json.get("usage") {
                                if let Ok(mut tracker) = usage_tracker.lock() {
                                    if let Some(input) = usage.get("input_tokens").and_then(|t| t.as_u64()) {
                                        tracker.0 = input as u32;
                                    }
                                    if let Some(output) = usage.get("output_tokens").and_then(|t| t.as_u64()) {
                                        tracker.1 = output as u32;
                                    }
                                }
                            }
                        }
                    }

                    // Check for termination
                    if event.is_done() {
                        // Get final token counts
                        let (input_tokens, output_tokens) = if let Ok(tracker) = usage_tracker.lock() {
                            *tracker
                        } else {
                            (0, 0)
                        };

                        // Calculate cost (simplified - in production use actual cost calculation)
                        let cost_usd = (input_tokens as f64 * 0.00003) + (output_tokens as f64 * 0.00015);

                        broadcaster.send(super::sse_broadcast::TuiStreamEvent::Completed {
                            request_id: req_id.clone(),
                            input_tokens,
                            output_tokens,
                            cost_usd,
                        });
                    }
                }

                // Pass bytes through unchanged
                Ok(bytes)
            }
            Err(e) => {
                // Emit error event
                broadcaster.send(super::sse_broadcast::TuiStreamEvent::Error {
                    request_id: req_id.clone(),
                    message: e.to_string(),
                });
                Err(e)
            }
        }
    });

    // Convert to io::Error for axum Body compatibility
    let body_stream = passthrough_stream.map(|result| {
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
            tracing::error!(request_id = %request_id, error = %e, "Failed to build streaming response");
            // Emit error event
            gateway.stream_broadcaster.send(super::sse_broadcast::TuiStreamEvent::Error {
                request_id,
                message: e.to_string(),
            });
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

/// Stream subscribe endpoint - streams TuiStreamEvents as SSE
/// This allows the TUI to receive real-time updates about streaming LLM requests
async fn stream_subscribe_handler(
    State(gateway): State<GatewayState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("New TUI stream subscriber connected");

    // Subscribe to the broadcaster
    let mut receiver = gateway.stream_broadcaster.subscribe();

    // Create a stream that converts TuiStreamEvents to SSE Events
    let stream = async_stream::stream! {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    // Serialize the event to JSON
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            // Create SSE event with the serialized JSON
                            let sse_event = Event::default()
                                .event("stream_event")
                                .data(json);
                            yield Ok(sse_event);
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to serialize TuiStreamEvent");
                            // Don't break the stream, just skip this event
                            continue;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!(skipped = %skipped, "TUI subscriber lagged, skipped events");
                    // Continue receiving, but notify about lag
                    let lag_event = Event::default()
                        .event("lag")
                        .data(format!("{{\"skipped\":{}}}", skipped));
                    yield Ok(lag_event);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    tracing::info!("Stream broadcaster closed, ending TUI subscription");
                    break;
                }
            }
        }
    };

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(15))
                .text("ping")
        )
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
