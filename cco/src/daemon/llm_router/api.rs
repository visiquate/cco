//! HTTP API endpoints for LLM routing

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{LlmClient, LlmOptions, LlmRouter, RoutingDecision};

/// Shared state for LLM router API
#[derive(Clone)]
pub struct LlmRouterState {
    pub router: Arc<LlmRouter>,
    pub client: Arc<LlmClient>,
}

impl LlmRouterState {
    pub fn new(router: LlmRouter) -> Self {
        Self {
            router: Arc::new(router),
            client: Arc::new(LlmClient::new()),
        }
    }
}

/// Request to route a task
#[derive(Debug, Deserialize)]
pub struct RouteRequest {
    pub agent_type: String,
    pub task_type: Option<String>,
}

/// Request to call an LLM
#[derive(Debug, Deserialize)]
pub struct CallRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Response from LLM call
#[derive(Debug, Serialize)]
pub struct CallResponse {
    pub text: String,
    pub model: Option<String>,
}

/// Create router with all LLM routing endpoints
pub fn llm_router_routes() -> Router<LlmRouterState> {
    Router::new()
        .route("/api/llm/route", post(route_task_handler))
        .route("/api/llm/call", post(call_llm_handler))
        .route("/api/llm/stats", get(stats_handler))
}

/// POST /api/llm/route - Get routing decision for agent/task
async fn route_task_handler(
    State(state): State<LlmRouterState>,
    Json(req): Json<RouteRequest>,
) -> Response {
    let decision = state.router.route_task(&req.agent_type, req.task_type.as_deref());

    Json(decision).into_response()
}

/// POST /api/llm/call - Call custom LLM with prompt
async fn call_llm_handler(
    State(state): State<LlmRouterState>,
    Json(req): Json<CallRequest>,
) -> Response {
    // Get coding endpoint
    let endpoint = match state.router.get_endpoint("coding") {
        Some(ep) if ep.enabled => ep,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Custom coding endpoint not configured or not enabled"
                })),
            )
                .into_response();
        }
    };

    // Parse endpoint URL to get hostname for bearer token lookup
    let hostname = if let Ok(url) = url::Url::parse(&endpoint.url) {
        url.host_str().map(String::from)
    } else {
        None
    };

    // Get bearer token if applicable
    let bearer_token = if let Some(host) = hostname {
        state.router.get_bearer_token(&host).await
    } else {
        None
    };

    // Build options
    let options = LlmOptions {
        model: req.model,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
    };

    // Call endpoint
    match state
        .client
        .call_endpoint(endpoint, &req.prompt, options, bearer_token)
        .await
    {
        Ok(response) => Json(CallResponse {
            text: response.text,
            model: response.model,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("LLM call failed: {}", e)
            })),
        )
            .into_response(),
    }
}

/// GET /api/llm/stats - Get routing statistics
async fn stats_handler(State(state): State<LlmRouterState>) -> Response {
    let stats = state.router.get_stats();
    Json(stats).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::llm_router::{
        endpoints::{EndpointConfig, EndpointType},
        router::LlmRoutingConfig,
    };
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::collections::HashMap;
    use tower::ServiceExt;

    fn test_router_state() -> LlmRouterState {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "coding".to_string(),
            EndpointConfig {
                enabled: true,
                url: "http://localhost:11434".to_string(),
                endpoint_type: Some(EndpointType::Ollama),
                default_model: Some("qwen2.5-coder:32b-instruct".to_string()),
                api_key: None,
                temperature: Some(0.7),
                max_tokens: Some(4096),
                headers: None,
                additional_params: None,
            },
        );

        let config = LlmRoutingConfig {
            endpoints,
            rules: HashMap::new(),
        };

        let router = LlmRouter::new(config);
        LlmRouterState::new(router)
    }

    #[tokio::test]
    async fn test_route_architecture_task() {
        let state = test_router_state();
        let app = llm_router_routes().with_state(state);

        let request = Request::builder()
            .method("POST")
            .uri("/api/llm/route")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"agent_type": "chief-architect", "task_type": "design system"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let decision: RoutingDecision = serde_json::from_slice(&body).unwrap();

        assert_eq!(decision.endpoint, "claude");
        assert!(decision.use_claude_code);
    }

    #[tokio::test]
    async fn test_route_coding_task() {
        let state = test_router_state();
        let app = llm_router_routes().with_state(state);

        let request = Request::builder()
            .method("POST")
            .uri("/api/llm/route")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"agent_type": "python-specialist", "task_type": "implement API"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let decision: RoutingDecision = serde_json::from_slice(&body).unwrap();

        assert_eq!(decision.endpoint, "custom");
        assert!(!decision.use_claude_code);
        assert!(decision.url.is_some());
    }

    #[tokio::test]
    async fn test_stats_endpoint() {
        let state = test_router_state();
        let app = llm_router_routes().with_state(state);

        let request = Request::builder()
            .method("GET")
            .uri("/api/llm/stats")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let stats: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(stats.get("endpoints").is_some());
        assert!(stats.get("architecture_tasks").is_some());
        assert!(stats.get("coding_tasks").is_some());
    }
}
