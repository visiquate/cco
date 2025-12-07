//! Orchestra API Endpoints
//!
//! HTTP API endpoints for orchestra conductor functionality.

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use super::conductor::{OrchestraConductor, OrchestraStats};
use super::instructions::AgentInstructions;
use super::workflow::Workflow;

/// Shared state for orchestra API
#[derive(Clone)]
pub struct OrchestraState {
    pub conductor: Arc<OrchestraConductor>,
}

impl OrchestraState {
    /// Create new orchestra state from config file path
    pub fn new(config_path: &str) -> Result<Self, anyhow::Error> {
        let conductor = OrchestraConductor::from_config_file(config_path)?;
        Ok(Self {
            conductor: Arc::new(conductor),
        })
    }
}

/// Request to generate agent spawn instructions
#[derive(Debug, Deserialize)]
pub struct GenerateInstructionsRequest {
    pub requirement: String,
}

/// Response with agent spawn instructions
#[derive(Debug, Serialize)]
pub struct GenerateInstructionsResponse {
    pub instructions: AgentInstructions,
    pub total_agents: usize,
}

/// Request to generate a complete workflow
#[derive(Debug, Deserialize)]
pub struct GenerateWorkflowRequest {
    pub requirement: String,
}

/// Response with complete workflow
#[derive(Debug, Serialize)]
pub struct GenerateWorkflowResponse {
    pub workflow: Workflow,
    pub summary: String,
}

/// Orchestra statistics response
#[derive(Debug, Serialize)]
pub struct OrchestraStatsResponse {
    pub stats: OrchestraStats,
    pub agent_types: AgentTypeBreakdown,
}

#[derive(Debug, Serialize)]
pub struct AgentTypeBreakdown {
    pub architect: String,
    pub coding_agents: Vec<String>,
    pub integration_agents: Vec<String>,
    pub support_agents: Vec<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let body = Json(self);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

/// Create orchestra API router
pub fn orchestra_router(state: OrchestraState) -> Router {
    Router::new()
        .route("/api/orchestra/generate", post(generate_instructions))
        .route("/api/orchestra/workflow", post(generate_workflow))
        .route("/api/orchestra/stats", get(get_stats))
        .with_state(state)
}

/// POST /api/orchestra/generate
/// Generate agent spawn instructions for a requirement
async fn generate_instructions(
    State(state): State<OrchestraState>,
    Json(request): Json<GenerateInstructionsRequest>,
) -> Result<Json<GenerateInstructionsResponse>, ErrorResponse> {
    info!(
        "Generating agent spawn instructions for: {}",
        request.requirement
    );

    let instructions = state
        .conductor
        .generate_agent_spawn_instructions(&request.requirement);
    let total_agents = instructions.total_count();

    info!("Generated instructions for {} agents", total_agents);

    Ok(Json(GenerateInstructionsResponse {
        instructions,
        total_agents,
    }))
}

/// POST /api/orchestra/workflow
/// Generate complete workflow for a requirement
async fn generate_workflow(
    State(state): State<OrchestraState>,
    Json(request): Json<GenerateWorkflowRequest>,
) -> Result<Json<GenerateWorkflowResponse>, ErrorResponse> {
    info!("Generating workflow for: {}", request.requirement);

    let workflow = state.conductor.generate_workflow(&request.requirement);
    let summary = workflow.summary();

    info!(
        "Generated workflow with {} agents",
        workflow.phase1_agent_spawn.agents.total_count()
    );

    Ok(Json(GenerateWorkflowResponse { workflow, summary }))
}

/// GET /api/orchestra/stats
/// Get orchestra statistics
async fn get_stats(
    State(state): State<OrchestraState>,
) -> Result<Json<OrchestraStatsResponse>, ErrorResponse> {
    info!("Fetching orchestra statistics");

    let stats = state.conductor.get_stats();

    // Get agent type names
    let config = &state.conductor.config;
    let agent_types = AgentTypeBreakdown {
        architect: config.architect.name.clone(),
        coding_agents: config
            .coding_agents
            .iter()
            .map(|a| a.name.clone())
            .collect(),
        integration_agents: config
            .integration_agents
            .iter()
            .map(|a| a.name.clone())
            .collect(),
        support_agents: config
            .support_agents
            .iter()
            .map(|a| a.name.clone())
            .collect(),
    };

    Ok(Json(OrchestraStatsResponse { stats, agent_types }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    // Helper to create test state
    fn create_test_state() -> OrchestraState {
        let config_path = "/Users/brent/git/cc-orchestra/config/orchestra-config.json";
        OrchestraState::new(config_path).expect("Failed to create test state")
    }

    #[tokio::test]
    async fn test_generate_instructions_endpoint() {
        let state = create_test_state();
        let app = orchestra_router(state);

        let request_body = serde_json::json!({
            "requirement": "Build a Python API with authentication"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/orchestra/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_generate_workflow_endpoint() {
        let state = create_test_state();
        let app = orchestra_router(state);

        let request_body = serde_json::json!({
            "requirement": "Create a mobile app with backend"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/orchestra/workflow")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_stats_endpoint() {
        let state = create_test_state();
        let app = orchestra_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/orchestra/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
