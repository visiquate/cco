//! HTTP server for CCO proxy

use crate::agents_config::{load_agents_from_embedded, Agent, AgentsConfig};
use crate::analytics::{AnalyticsEngine, ApiCallRecord, ActivityEvent};
use crate::cache::MokaCache;
use crate::proxy::{ChatRequest, ChatResponse, ProxyServer};
use crate::router::ModelRouter;
use crate::security::{
    localhost_only_middleware, validate_message_size, validate_terminal_dimensions,
    validate_utf8, ConnectionTracker,
};
use crate::terminal::TerminalSession;
use crate::version::DateVersion;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Json, State, Path as AxumPath,
    },
    http::{header, StatusCode},
    middleware,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Response,
    },
    routing::{get, post},
    Router,
};
use chrono::Utc;
use dirs::data_local_dir;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tower_http::cors::CorsLayer;
use tracing::{trace, error, info, warn};

/// Agent configuration from orchestration config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub r#type: String,
    pub model: String,
}

/// Server state shared across handlers
#[derive(Clone)]
pub struct ServerState {
    pub cache: MokaCache,
    pub router: ModelRouter,
    pub analytics: Arc<AnalyticsEngine>,
    pub proxy: Arc<ProxyServer>,
    pub start_time: Instant,
    pub model_overrides: Arc<HashMap<String, String>>,
    pub agent_models: Arc<HashMap<String, String>>, // agent type -> configured model
    pub agents: Arc<AgentsConfig>,                   // agent definitions from ~/.claude/agents/
    pub connection_tracker: ConnectionTracker,       // connection tracking for rate limiting
}

/// PID file metadata structure
#[derive(Debug, Serialize, Deserialize)]
struct PidInfo {
    pid: u32,
    port: u16,
    started_at: chrono::DateTime<Utc>,
    version: String,
}

/// Get the PID directory path
fn get_pid_dir() -> anyhow::Result<PathBuf> {
    let data_dir =
        data_local_dir().ok_or_else(|| anyhow::anyhow!("Failed to get local data directory"))?;

    let pid_dir = data_dir.join("cco").join("pids");

    // Create directory if it doesn't exist
    fs::create_dir_all(&pid_dir)?;

    Ok(pid_dir)
}

/// Get the PID file path for a specific port
fn get_pid_file(port: u16) -> anyhow::Result<PathBuf> {
    let pid_dir = get_pid_dir()?;
    Ok(pid_dir.join(format!("cco-{}.pid", port)))
}

/// Write PID file
fn write_pid_file(port: u16) -> anyhow::Result<()> {
    let pid_file = get_pid_file(port)?;

    let pid_info = PidInfo {
        pid: std::process::id(),
        port,
        started_at: Utc::now(),
        version: DateVersion::current().to_string(),
    };

    let json = serde_json::to_string_pretty(&pid_info)?;
    fs::write(&pid_file, json)?;

    Ok(())
}

/// Remove PID file
fn remove_pid_file(port: u16) -> anyhow::Result<()> {
    let pid_file = get_pid_file(port)?;

    if pid_file.exists() {
        fs::remove_file(&pid_file)?;
    }

    Ok(())
}

/// Get the logs directory path
fn get_logs_dir() -> anyhow::Result<PathBuf> {
    let data_dir =
        data_local_dir().ok_or_else(|| anyhow::anyhow!("Failed to get local data directory"))?;

    let logs_dir = data_dir.join("cco").join("logs");

    // Create directory if it doesn't exist
    fs::create_dir_all(&logs_dir)?;

    Ok(logs_dir)
}

/// Get the current project path for Claude history
fn get_current_project_path() -> anyhow::Result<String> {
    // 1. Try environment variable CCO_PROJECT_PATH first
    if let Ok(path) = std::env::var("CCO_PROJECT_PATH") {
        info!("✅ Using CCO_PROJECT_PATH: {}", path);

        // Verify path exists
        if std::path::Path::new(&path).exists() {
            info!("✓ Project path exists and is accessible");
        } else {
            tracing::warn!("⚠️  Project path does not exist: {}", path);
        }

        return Ok(path);
    }

    info!("⚠️  CCO_PROJECT_PATH not set, falling back to current directory");

    // 2. Fall back to current working directory
    let cwd = std::env::current_dir()?
        .to_string_lossy()
        .to_string();

    info!("📁 Current working directory: {}", cwd);

    // 3. Encode the path: /Users/brent/git/cc-orchestra -> -Users-brent-git-cc-orchestra
    let encoded = format!("-{}", cwd
        .trim_start_matches('/')
        .replace('/', "-"));

    info!("🔤 Encoded path: {}", encoded);

    // 4. Return full project path
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let project_path = format!(
        "{}/.claude/projects/{}/",
        home,
        encoded
    );

    info!("🎯 Derived project path: {}", project_path);

    // Verify path exists
    if std::path::Path::new(&project_path).exists() {
        info!("✓ Derived project path exists");
    } else {
        tracing::warn!("⚠️  Derived project path does not exist: {}", project_path);
        tracing::warn!("   Expected at: {}", project_path);
        tracing::warn!("   To fix: export CCO_PROJECT_PATH='<correct-path>'");
    }

    Ok(project_path)
}

/// Setup logging to file
fn setup_file_logging(port: u16) -> anyhow::Result<()> {
    let logs_dir = get_logs_dir()?;
    let log_file = logs_dir.join(format!("cco-{}.log", port));

    // For now, just ensure the directory exists
    // Full file logging will be implemented with proper tracing subscriber
    info!("Logs will be written to: {:?}", log_file);

    Ok(())
}

/// Health check response
#[derive(serde::Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    cache_stats: CacheMetrics,
    uptime: u64,
}

#[derive(serde::Serialize)]
pub struct CacheMetrics {
    hit_rate: f64,
    hits: u64,
    misses: u64,
    entries: u64,
    total_savings: f64,
}

/// Analytics stats response
#[derive(serde::Serialize)]
pub struct StatsResponse {
    cache: CacheStats,
    models: Vec<ModelStats>,
    totals: TotalStats,
}

#[derive(serde::Serialize)]
pub struct CacheStats {
    hit_rate: f64,
    hits: u64,
    misses: u64,
    entries: u64,
    total_savings: f64,
}

#[derive(serde::Serialize)]
pub struct ModelStats {
    model: String,
    requests: u64,
    cache_hits: u64,
    cache_misses: u64,
    actual_cost: f64,
    would_be_cost: f64,
    savings: f64,
}

#[derive(serde::Serialize)]
pub struct TotalStats {
    requests: u64,
    actual_cost: f64,
    would_be_cost: f64,
    total_savings: f64,
}

/// Error response
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    error: String,
}

/// All agents response
#[derive(serde::Serialize)]
pub struct AgentsListResponse {
    agents: Vec<Agent>,
}

/// Agent not found error response
#[derive(serde::Serialize)]
pub struct AgentNotFoundResponse {
    error: String,
}

/// Custom error type for server errors
pub enum ServerError {
    Internal(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ServerError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

/// Health check endpoint with analytics
async fn health(State(state): State<Arc<ServerState>>) -> Json<HealthResponse> {
    let cache_metrics = state.cache.get_metrics().await;
    let uptime = state.start_time.elapsed().as_secs();

    Json(HealthResponse {
        status: "ok".to_string(),
        version: DateVersion::current().to_string(),
        cache_stats: CacheMetrics {
            hit_rate: cache_metrics.hit_rate,
            hits: cache_metrics.hits,
            misses: cache_metrics.misses,
            entries: cache_metrics.hits + cache_metrics.misses,
            total_savings: cache_metrics.total_savings,
        },
        uptime,
    })
}

/// Shutdown endpoint - gracefully shuts down the server
async fn shutdown_handler() -> Json<serde_json::Value> {
    info!("🛑 Shutdown request received");

    // Spawn a task to exit after sending the response
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        std::process::exit(0);
    });

    Json(serde_json::json!({
        "status": "shutdown_initiated",
        "message": "Server shutting down..."
    }))
}

/// Get all available agents
async fn list_agents(State(state): State<Arc<ServerState>>) -> Json<AgentsListResponse> {
    let agents = state.agents.all();
    info!("Returning {} agents", agents.len());
    Json(AgentsListResponse { agents })
}

/// Get a specific agent by name
async fn get_agent(
    State(state): State<Arc<ServerState>>,
    AxumPath(agent_name): AxumPath<String>,
) -> Result<Json<Agent>, (StatusCode, Json<AgentNotFoundResponse>)> {
    match state.agents.get(&agent_name) {
        Some(agent) => {
            info!("Agent found: {}", agent_name);
            Ok(Json(agent.clone()))
        }
        None => {
            info!("Agent not found: {}", agent_name);
            Err((
                StatusCode::NOT_FOUND,
                Json(AgentNotFoundResponse {
                    error: format!("Agent not found: {}", agent_name),
                }),
            ))
        }
    }
}

/// Detect agent type from conversation content
///
/// Analyzes the system message and conversation to identify which agent is making the request.
/// Returns the agent type if detected, or None if unrecognized.
fn detect_agent_from_conversation(messages: &[crate::proxy::Message]) -> Option<String> {
    // Find the system message (first message with role "system")
    let system_message = messages
        .iter()
        .find(|m| m.role.to_lowercase() == "system")
        .map(|m| m.content.clone());

    if let Some(system_msg) = system_message {
        let lower = system_msg.to_lowercase();

        // Pattern matching for known agents
        // These patterns match the agent descriptions from orchestra-config.json
        let patterns = vec![
            ("chief-architect", vec!["chief architect", "strategic decision"]),
            ("tdd-coding-agent", vec!["tdd", "test-driven", "test-first"]),
            ("python-specialist", vec!["python specialist", "fastapi", "django"]),
            ("swift-specialist", vec!["swift specialist", "swiftui", "ios"]),
            ("rust-specialist", vec!["rust specialist", "systems programming"]),
            ("go-specialist", vec!["go specialist", "golang", "microservice"]),
            ("flutter-specialist", vec!["flutter specialist", "cross-platform mobile"]),
            ("frontend-developer", vec!["frontend developer", "react", "javascript"]),
            ("fullstack-developer", vec!["full-stack", "fullstack"]),
            ("devops-engineer", vec!["devops", "docker", "kubernetes", "deployment"]),
            ("test-engineer", vec!["test engineer", "qa", "testing", "test automation"]),
            ("test-automator", vec!["test automator", "test automation"]),
            ("documentation-expert", vec!["documentation", "technical writer", "api documenting"]),
            ("security-auditor", vec!["security", "vulnerability", "penetration"]),
            ("database-architect", vec!["database architect", "schema design"]),
            ("backend-architect", vec!["backend architect", "api design"]),
            ("code-reviewer", vec!["code review", "code quality"]),
            ("architecture-modernizer", vec!["architecture", "modernization", "refactor"]),
            ("debugger", vec!["debugging", "error analysis"]),
            ("performance-engineer", vec!["performance", "optimization", "profiling"]),
        ];

        for (agent_type, keywords) in patterns {
            for keyword in keywords {
                if lower.contains(keyword) {
                    return Some(agent_type.to_string());
                }
            }
        }
    }

    None
}

/// Chat completion endpoint
async fn chat_completion(
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServerError> {
    let original_model = request.model.clone();
    let request_start = std::time::Instant::now();

    // Detect agent from conversation content
    if let Some(agent_type) = detect_agent_from_conversation(&request.messages) {
        if let Some(configured_model) = state.agent_models.get(&agent_type) {
            info!(
                "🤖 Agent detected from conversation: '{}' | Model override: {} → {}",
                agent_type, original_model, configured_model
            );
            request.model = configured_model.clone();

            // Track the override in analytics
            state
                .analytics
                .record_model_override(&original_model, configured_model)
                .await;
        }
    }
    // If no agent detected, try blanket model override rules
    else if let Some(override_model) = state.model_overrides.get(&request.model) {
        info!(
            "🔄 Model override (blanket rule): {} → {}",
            original_model, override_model
        );
        request.model = override_model.clone();

        // Track the override in analytics
        state
            .analytics
            .record_model_override(&original_model, override_model)
            .await;
    }

    info!("📝 Processing chat request for model: {}", request.model);

    // Generate cache key
    let prompt = request
        .messages
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let cache_key = MokaCache::generate_key(
        &request.model,
        &prompt,
        request.temperature.unwrap_or(1.0),
        request.max_tokens.unwrap_or(4096),
    );

    // Check cache
    if let Some(cached) = state.cache.get(&cache_key).await {
        info!("Cache hit for model: {}", request.model);
        let latency_ms = request_start.elapsed().as_millis() as u64;

        // Calculate savings
        if let Some((actual_cost, would_be_cost, savings)) = state.router.calculate_cache_savings(
            &request.model,
            cached.input_tokens,
            cached.output_tokens,
        ) {
            state
                .analytics
                .record_api_call(ApiCallRecord {
                    model: request.model.clone(),
                    input_tokens: cached.input_tokens,
                    output_tokens: cached.output_tokens,
                    cache_hit: true,
                    actual_cost,
                    would_be_cost,
                    savings,
                })
                .await;

            // Record activity event for cache hit
            state
                .analytics
                .record_event(ActivityEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    event_type: "cache_hit".to_string(),
                    agent_name: None,
                    model: Some(request.model.clone()),
                    tokens: Some((cached.input_tokens + cached.output_tokens) as u64),
                    latency_ms: Some(latency_ms),
                    status: Some("success".to_string()),
                    cost: Some(0.0), // Cache hit = no additional cost
                })
                .await;
        }

        return Ok(Json(ChatResponse {
            id: format!("cache-{}", uuid::Uuid::new_v4()),
            model: cached.model.clone(),
            content: cached.content.clone(),
            input_tokens: cached.input_tokens,
            output_tokens: cached.output_tokens,
            usage: crate::proxy::Usage {
                input_tokens: cached.input_tokens,
                output_tokens: cached.output_tokens,
            },
            from_cache: true,
        }));
    }

    info!("Cache miss for model: {}", request.model);

    // Handle request via proxy (this simulates an API call)
    let response = state.proxy.handle_request(request.clone()).await;
    let latency_ms = request_start.elapsed().as_millis() as u64;

    // Calculate cost
    if let Some(cost) = state.router.calculate_cost(
        &request.model,
        response.input_tokens,
        response.output_tokens,
    ) {
        state
            .analytics
            .record_api_call(ApiCallRecord {
                model: request.model.clone(),
                input_tokens: response.input_tokens,
                output_tokens: response.output_tokens,
                cache_hit: false,
                actual_cost: cost,
                would_be_cost: cost,
                savings: 0.0,
            })
            .await;

        // Record activity event for API call
        state
            .analytics
            .record_event(ActivityEvent {
                timestamp: Utc::now().to_rfc3339(),
                event_type: "api_call".to_string(),
                agent_name: None,
                model: Some(request.model.clone()),
                tokens: Some((response.input_tokens + response.output_tokens) as u64),
                latency_ms: Some(latency_ms),
                status: Some("success".to_string()),
                cost: Some(cost),
            })
            .await;
    }

    // Store in cache
    state
        .cache
        .insert(
            cache_key,
            crate::cache::CachedResponse {
                content: response.content.clone(),
                model: response.model.clone(),
                input_tokens: response.input_tokens,
                output_tokens: response.output_tokens,
            },
        )
        .await;

    Ok(Json(response))
}

/// Dashboard root - serves static HTML
async fn dashboard_html() -> impl IntoResponse {
    let html = include_str!("../static/dashboard.html");
    Html(html)
}

/// Dashboard CSS
async fn dashboard_css() -> impl IntoResponse {
    let css = include_str!("../static/dashboard.css");
    ([(header::CONTENT_TYPE, "text/css")], css)
}

/// Dashboard JavaScript
async fn dashboard_js() -> impl IntoResponse {
    let js = include_str!("../static/dashboard.js");
    ([(header::CONTENT_TYPE, "application/javascript")], js)
}

/// Analytics stats endpoint
async fn stats(State(state): State<Arc<ServerState>>) -> Result<Json<StatsResponse>, ServerError> {
    // Get cache metrics
    let cache_metrics = state.cache.get_metrics().await;

    // Get analytics by model
    let metrics_by_model = state.analytics.get_metrics_by_model().await;

    // Calculate totals
    let total_requests = state.analytics.get_total_requests().await;
    let total_actual_cost = state.analytics.get_total_actual_cost().await;
    let total_would_be_cost = state.analytics.get_total_would_be_cost().await;
    let total_savings = state.analytics.get_total_savings().await;

    // Convert model metrics to response format
    let model_stats: Vec<ModelStats> = metrics_by_model
        .into_iter()
        .map(|(_, metrics)| ModelStats {
            model: metrics.model,
            requests: metrics.total_requests,
            cache_hits: metrics.cache_hits,
            cache_misses: metrics.cache_misses,
            actual_cost: metrics.actual_cost,
            would_be_cost: metrics.would_be_cost,
            savings: metrics.total_savings,
        })
        .collect();

    Ok(Json(StatsResponse {
        cache: CacheStats {
            hit_rate: cache_metrics.hit_rate,
            hits: cache_metrics.hits,
            misses: cache_metrics.misses,
            entries: cache_metrics.hits + cache_metrics.misses,
            total_savings: cache_metrics.total_savings,
        },
        models: model_stats,
        totals: TotalStats {
            requests: total_requests,
            actual_cost: total_actual_cost,
            would_be_cost: total_would_be_cost,
            total_savings,
        },
    }))
}

/// Project stats endpoint (currently same as general stats)
async fn project_stats(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<StatsResponse>, ServerError> {
    // For now, return the same data as general stats
    // In the future, this could be project-specific
    stats(State(state)).await
}

/// Machine stats endpoint (currently same as general stats)
async fn machine_stats(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<StatsResponse>, ServerError> {
    // For now, return the same data as general stats
    // In the future, this could aggregate across multiple projects
    stats(State(state)).await
}

/// Model override statistics endpoint
async fn override_stats(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let overrides = state.analytics.get_override_statistics().await;

    // Group overrides by model to show which models are being rewritten
    let mut by_model: HashMap<String, (String, usize)> = HashMap::new();

    for record in &overrides {
        by_model
            .entry(record.original_model.clone())
            .and_modify(|(_, count)| *count += 1)
            .or_insert_with(|| (record.override_to.clone(), 1));
    }

    Ok(Json(serde_json::json!({
        "total_overrides": overrides.len(),
        "overrides_by_model": by_model,
        "recent_overrides": overrides.iter().rev().take(10).collect::<Vec<_>>(),
    })))
}

/// Project metrics response
#[derive(serde::Serialize)]
pub struct ProjectMetric {
    pub cost: f64,
    pub tokens: u64,
    pub calls: u64,
    pub last_updated: String,
}

#[derive(serde::Serialize)]
pub struct ProjectDataRow {
    pub name: String,
    pub api_calls: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
    pub last_activity: String,
}

#[derive(serde::Serialize)]
pub struct ProjectMetricsResponse {
    pub projects: Vec<ProjectDataRow>,
}

/// Per-project metrics endpoint
async fn metrics_projects(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ProjectMetricsResponse>, ServerError> {
    // Calculate totals
    let total_actual_cost = state.analytics.get_total_actual_cost().await;
    let total_requests = state.analytics.get_total_requests().await;
    let metrics_by_model = state.analytics.get_metrics_by_model().await;

    // Calculate total input/output tokens from model breakdown
    let mut total_input_tokens = 0u64;
    let mut total_output_tokens = 0u64;
    for metric in metrics_by_model.values() {
        total_input_tokens += metric.total_requests; // This will need adjustment based on actual token data
        total_output_tokens += metric.total_requests; // This will need adjustment based on actual token data
    }

    let projects = vec![
        ProjectDataRow {
            name: "Claude Orchestra".to_string(),
            api_calls: total_requests,
            input_tokens: total_input_tokens,
            output_tokens: total_output_tokens,
            cost: total_actual_cost,
            last_activity: Utc::now().to_rfc3339(),
        },
    ];

    Ok(Json(ProjectMetricsResponse { projects }))
}

/// Chart data structures
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ChartDataPoint {
    pub date: String,
    pub cost: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProjectChartData {
    pub project: String,
    pub cost: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ModelDistribution {
    pub model: String,
    pub percentage: f64,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ChartData {
    pub cost_over_time: Vec<ChartDataPoint>,
    pub cost_by_project: Vec<ProjectChartData>,
    pub model_distribution: Vec<ModelDistribution>,
}

/// SSE stream response format with project, machine, and activity
#[derive(serde::Serialize)]
pub struct SseStreamResponse {
    pub project: ProjectInfo,
    pub machine: MachineInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_metrics: Option<crate::claude_history::ClaudeMetrics>,
    pub activity: Vec<ActivityEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_data: Option<ChartData>,
}

#[derive(serde::Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub cost: f64,
    pub tokens: u64,
    pub calls: u64,
    pub last_updated: String,
}

#[derive(serde::Serialize)]
pub struct MachineInfo {
    pub cpu: String,
    pub memory: String,
    pub uptime: u64,
    pub process_count: u64,
}

/// SSE stream endpoint for real-time analytics updates
async fn stream(
    State(state): State<Arc<ServerState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;

            // Calculate totals
            let total_requests = state.analytics.get_total_requests().await;
            let total_actual_cost = state.analytics.get_total_actual_cost().await;
            let total_would_be_cost = state.analytics.get_total_would_be_cost().await;

            // Get recent activity (last 20 events)
            let activity = state.analytics.get_recent_activity(20).await;

            // Calculate uptime
            let uptime = state.start_time.elapsed().as_secs();

            // Get process count (approximate - number of overrides as proxy for active agents)
            let overrides = state.analytics.get_override_statistics().await;
            let process_count = (overrides.len() / 10).max(1) as u64;

            // Load Claude project metrics
            let claude_metrics = get_current_project_path()
                .ok()
                .and_then(|path| {
                    // Try to load metrics, but don't fail the SSE stream if it errors
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(
                            crate::claude_history::load_claude_project_metrics(&path)
                        )
                    }).ok()
                });

            // Generate chart data
            let metrics_by_model = state.analytics.get_metrics_by_model().await;
            let _savings_by_model = state.analytics.get_savings_by_model().await;

            // Cost over time: For now, just show current cost over past 30 days (mock data)
            let today = chrono::Local::now();
            let mut cost_over_time = Vec::new();
            for i in 0..30 {
                let date = today - chrono::Duration::days(i);
                let daily_cost = total_actual_cost / 30.0; // Simple average
                cost_over_time.push(ChartDataPoint {
                    date: date.format("%Y-%m-%d").to_string(),
                    cost: daily_cost,
                });
            }
            cost_over_time.reverse(); // Oldest first

            // Cost by project: For now, just the single project
            let cost_by_project = vec![ProjectChartData {
                project: "Claude Orchestra".to_string(),
                cost: total_actual_cost,
            }];

            // Model distribution: Calculate percentage by model
            let total_model_cost: f64 = metrics_by_model.values().map(|m| m.actual_cost).sum();
            let model_distribution: Vec<ModelDistribution> = if total_model_cost > 0.0 {
                metrics_by_model
                    .iter()
                    .map(|(model_name, metrics)| {
                        let percentage = (metrics.actual_cost / total_model_cost) * 100.0;
                        ModelDistribution {
                            model: model_name.clone(),
                            percentage,
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

            let chart_data = Some(ChartData {
                cost_over_time,
                cost_by_project,
                model_distribution,
            });

            let response = SseStreamResponse {
                project: ProjectInfo {
                    name: "Claude Orchestra".to_string(),
                    cost: total_actual_cost,
                    tokens: total_would_be_cost as u64,
                    calls: total_requests,
                    last_updated: Utc::now().to_rfc3339(),
                },
                machine: MachineInfo {
                    cpu: "N/A".to_string(),
                    memory: "N/A".to_string(),
                    uptime,
                    process_count,
                },
                claude_metrics,
                activity,
                chart_data,
            };

            // Serialize to JSON
            if let Ok(json) = serde_json::to_string(&response) {
                yield Ok(Event::default().event("analytics").data(json));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// WebSocket terminal endpoint handler with security
async fn terminal_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> Response {
    let ip = addr.ip();

    trace!(
        ip = %ip,
        remote_addr = %addr,
        "WebSocket upgrade request received for terminal"
    );

    // Check connection limit for this IP
    if !state.connection_tracker.try_acquire(ip).await {
        let current_count = state.connection_tracker.get_count(ip).await;
        warn!(
            ip = %ip,
            current_connections = current_count,
            max_allowed = 10,
            "Terminal connection rejected: too many concurrent connections"
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many concurrent connections from this IP",
        )
            .into_response();
    }

    trace!(
        ip = %ip,
        "Connection slot acquired for terminal"
    );

    info!(
        ip = %ip,
        "Terminal connection accepted, initiating WebSocket upgrade"
    );

    // Clone state and IP for the socket handler
    let state_clone = state.clone();
    let ip_clone = ip;

    ws.on_upgrade(move |socket| async move {
        trace!(
            ip = %ip_clone,
            "WebSocket upgrade complete (101 Switching Protocols)"
        );
        handle_terminal_socket(socket, state_clone.clone(), ip_clone).await;
        // Release connection when socket closes
        state_clone.connection_tracker.release(ip_clone).await;
    })
}

/// Handle WebSocket terminal connection with real PTY and security validation
async fn handle_terminal_socket(
    socket: WebSocket,
    _state: Arc<ServerState>,
    ip: std::net::IpAddr,
) {
    use futures::{SinkExt, StreamExt};

    // Security constants
    const MAX_MESSAGE_SIZE: usize = 64 * 1024; // 64KB
    const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60); // 5 minutes

    trace!(
        ip = %ip,
        "Entering WebSocket handler for terminal connection"
    );

    // Spawn a real shell session
    let session = match TerminalSession::spawn_shell() {
        Ok(s) => s,
        Err(e) => {
            error!(
                ip = %ip,
                error = %e,
                "Failed to spawn PTY shell session"
            );
            return;
        }
    };

    let session_id = session.session_id().to_string();
    trace!(
        ip = %ip,
        session_id = %session_id,
        "PTY shell session spawned successfully"
    );

    info!(
        ip = %ip,
        session_id = %session_id,
        "Terminal session initialized and ready for I/O"
    );

    trace!(
        ip = %ip,
        session_id = %session_id,
        "Splitting WebSocket into sender and receiver"
    );

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // Send initial shell output
    trace!(
        ip = %ip,
        session_id = %session_id,
        "Reading initial shell output"
    );

    let mut output_buffer = [0u8; 4096];
    if let Ok(n) = session.read_output(&mut output_buffer).await {
        if n > 0 {
            trace!(
                ip = %ip,
                session_id = %session_id,
                bytes = n,
                "Initial shell output received, sending to client"
            );
            let msg = Message::Binary(output_buffer[..n].to_vec());
            if sender.lock().await.send(msg).await.is_err() {
                error!(
                    ip = %ip,
                    session_id = %session_id,
                    "Failed to send initial output to client"
                );
                let _ = session.close_session().await;
                return;
            }
        } else {
            trace!(
                ip = %ip,
                session_id = %session_id,
                "No initial shell output available"
            );
        }
    } else {
        warn!(
            ip = %ip,
            session_id = %session_id,
            "Error reading initial shell output"
        );
    }

    // Spawn a background task to read shell output and send to client
    let session_clone = session.clone();
    let session_id_clone = session_id.clone();
    let ip_clone = ip;
    let sender_clone = sender.clone();
    let sender_handle = tokio::spawn(async move {
        trace!(
            ip = %ip_clone,
            session_id = %session_id_clone,
            "Background task started for reading shell output"
        );

        let mut read_interval = interval(Duration::from_millis(10));
        let mut idle_timer = tokio::time::interval(Duration::from_secs(1));
        let mut last_activity = std::time::Instant::now();

        loop {
            tokio::select! {
                _ = read_interval.tick() => {
                    let mut buffer = [0u8; 4096];
                    match session_clone.read_output(&mut buffer).await {
                        Ok(n) if n > 0 => {
                            trace!(
                                ip = %ip_clone,
                                session_id = %session_id_clone,
                                bytes = n,
                                "Shell output received, sending to client"
                            );
                            last_activity = std::time::Instant::now();
                            // Send output to client via WebSocket
                            let msg = Message::Binary(buffer[..n].to_vec());
                            let mut sender_lock = sender_clone.lock().await;
                            if let Err(e) = sender_lock.send(msg).await {
                                warn!(
                                    ip = %ip_clone,
                                    session_id = %session_id_clone,
                                    error = %e,
                                    "Failed to send shell output to client"
                                );
                                break;
                            }
                        }
                        Ok(_) => {}, // No data available
                        Err(e) => {
                            warn!(
                                ip = %ip_clone,
                                session_id = %session_id_clone,
                                error = %e,
                                "Error reading from shell in background task"
                            );
                            break;
                        }
                    }
                }
                _ = idle_timer.tick() => {
                    // Check for idle timeout
                    if last_activity.elapsed() > IDLE_TIMEOUT {
                        info!(
                            ip = %ip_clone,
                            session_id = %session_id_clone,
                            idle_seconds = last_activity.elapsed().as_secs(),
                            timeout_secs = IDLE_TIMEOUT.as_secs(),
                            "Terminal session idle timeout reached"
                        );
                        break;
                    }

                    // Keep-alive: check if process is still running
                    match session_clone.is_running().await {
                        Ok(true) => {
                            trace!(
                                ip = %ip_clone,
                                session_id = %session_id_clone,
                                "Shell process health check: running"
                            );
                        },
                        Ok(false) => {
                            info!(
                                ip = %ip_clone,
                                session_id = %session_id_clone,
                                "Shell process no longer running"
                            );
                            break;
                        }
                        Err(e) => {
                            warn!(
                                ip = %ip_clone,
                                session_id = %session_id_clone,
                                error = %e,
                                "Error checking if shell is running"
                            );
                            break;
                        }
                    }
                }
            }
        }

        trace!(
            ip = %ip_clone,
            session_id = %session_id_clone,
            "Background task exiting"
        );
    });

    // Handle incoming WebSocket messages from client with security validation
    trace!(
        ip = %ip,
        session_id = %session_id,
        "Starting WebSocket message handling loop"
    );

    let mut last_message_time = std::time::Instant::now();

    while let Some(msg) = StreamExt::next(&mut receiver).await {
        // Update last activity time
        last_message_time = std::time::Instant::now();

        match msg {
            Ok(Message::Binary(data)) => {
                trace!(
                    ip = %ip,
                    session_id = %session_id,
                    size = data.len(),
                    "Binary message received from client"
                );

                // Validate message size
                if let Err(e) = validate_message_size(&data, MAX_MESSAGE_SIZE) {
                    warn!(
                        ip = %ip,
                        session_id = %session_id,
                        size = data.len(),
                        max_size = MAX_MESSAGE_SIZE,
                        error = %e,
                        "Binary message size limit exceeded"
                    );
                    let _ = sender.lock().await
                        .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                            code: axum::extract::ws::close_code::POLICY,
                            reason: std::borrow::Cow::from(e),
                        })))
                        .await;
                    break;
                }

                // Raw terminal input from client
                match session.write_input(&data).await {
                    Ok(n) => {
                        trace!(
                            ip = %ip,
                            session_id = %session_id,
                            bytes = n,
                            "Binary input written to shell stdin"
                        );
                    }
                    Err(e) => {
                        error!(
                            ip = %ip,
                            session_id = %session_id,
                            error = %e,
                            "Failed to write binary input to shell"
                        );
                        break;
                    }
                }
            }
            Ok(Message::Text(text)) => {
                trace!(
                    ip = %ip,
                    session_id = %session_id,
                    size = text.len(),
                    "Text message received from client"
                );

                // Validate message size
                if let Err(e) = validate_message_size(text.as_bytes(), MAX_MESSAGE_SIZE) {
                    warn!(
                        ip = %ip,
                        session_id = %session_id,
                        size = text.len(),
                        max_size = MAX_MESSAGE_SIZE,
                        error = %e,
                        "Text message size limit exceeded"
                    );
                    let _ = sender.lock().await
                        .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                            code: axum::extract::ws::close_code::POLICY,
                            reason: std::borrow::Cow::from(e),
                        })))
                        .await;
                    break;
                }

                // Validate UTF-8 (though axum already does this for Text messages)
                if let Err(e) = validate_utf8(text.as_bytes()) {
                    warn!(
                        ip = %ip,
                        session_id = %session_id,
                        error = %e,
                        "Invalid UTF-8 in text message"
                    );
                    continue;
                }

                // Check for resize message: "\x1b[RESIZE;COLS;ROWS"
                if text.starts_with("\x1b[RESIZE;") {
                    trace!(
                        ip = %ip,
                        session_id = %session_id,
                        "Terminal resize command received"
                    );

                    if let Some(rest) = text.strip_prefix("\x1b[RESIZE;") {
                        let parts: Vec<&str> = rest.split(';').collect();
                        if parts.len() >= 2 {
                            if let (Ok(cols), Ok(rows)) = (
                                parts[0].parse::<u16>(),
                                parts[1].trim().parse::<u16>(),
                            ) {
                                trace!(
                                    ip = %ip,
                                    session_id = %session_id,
                                    cols = cols,
                                    rows = rows,
                                    "Resize dimensions parsed"
                                );

                                // Validate dimensions
                                if let Err(e) = validate_terminal_dimensions(cols, rows) {
                                    warn!(
                                        ip = %ip,
                                        session_id = %session_id,
                                        cols = cols,
                                        rows = rows,
                                        error = %e,
                                        "Invalid terminal dimensions"
                                    );
                                    continue;
                                }

                                if let Err(e) = session.set_terminal_size(cols, rows).await {
                                    warn!(
                                        ip = %ip,
                                        session_id = %session_id,
                                        cols = cols,
                                        rows = rows,
                                        error = %e,
                                        "Terminal resize operation failed"
                                    );
                                } else {
                                    info!(
                                        ip = %ip,
                                        session_id = %session_id,
                                        cols = cols,
                                        rows = rows,
                                        "Terminal resized successfully"
                                    );
                                }
                            } else {
                                warn!(
                                    ip = %ip,
                                    session_id = %session_id,
                                    message = %text,
                                    "Failed to parse resize dimensions from command"
                                );
                            }
                        } else {
                            warn!(
                                ip = %ip,
                                session_id = %session_id,
                                parts_count = parts.len(),
                                "Resize command missing required parts"
                            );
                        }
                    }
                } else {
                    // Treat as text input
                    trace!(
                        ip = %ip,
                        session_id = %session_id,
                        size = text.len(),
                        "Text input received, writing to shell"
                    );

                    if let Err(e) = session.write_input(text.as_bytes()).await {
                        error!(
                            ip = %ip,
                            session_id = %session_id,
                            error = %e,
                            "Failed to write text input to shell"
                        );
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!(
                    ip = %ip,
                    session_id = %session_id,
                    "Client initiated WebSocket close"
                );
                break;
            }
            Err(e) => {
                warn!(
                    ip = %ip,
                    session_id = %session_id,
                    error = %e,
                    "WebSocket error occurred"
                );
                trace!("WebSocket error details: {}", e);
                break;
            }
            _ => {
                trace!(
                    ip = %ip,
                    session_id = %session_id,
                    "Other WebSocket message type received"
                );
            }
        }
    }

    trace!(
        ip = %ip,
        session_id = %session_id,
        "Exiting WebSocket message handling loop"
    );

    // Clean up
    trace!(
        ip = %ip,
        session_id = %session_id,
        "Aborting background task for shell output"
    );
    sender_handle.abort();

    trace!(
        ip = %ip,
        session_id = %session_id,
        "Closing terminal session and PTY"
    );

    if let Err(e) = session.close_session().await {
        warn!(
            ip = %ip,
            session_id = %session_id,
            error = %e,
            "Error closing terminal session"
        );
    } else {
        trace!(
            ip = %ip,
            session_id = %session_id,
            "Terminal session closed successfully"
        );
    }

    info!(
        ip = %ip,
        session_id = %session_id,
        duration_secs = last_message_time.elapsed().as_secs(),
        "Terminal connection cleanup complete"
    );
}

/// Load model override rules from configuration
///
/// Returns a HashMap mapping original model names to override model names.
/// For MVP, we hardcode the default rules. Future enhancement: load from TOML.
fn load_model_overrides() -> HashMap<String, String> {
    let mut overrides = HashMap::new();

    // Default model override rules (Sonnet → Haiku for 75% cost savings)
    overrides.insert(
        "claude-sonnet-4.5-20250929".to_string(),
        "claude-haiku-4-5-20251001".to_string(),
    );
    overrides.insert(
        "claude-sonnet-4".to_string(),
        "claude-haiku-4-5-20251001".to_string(),
    );
    overrides.insert(
        "claude-sonnet-3.5".to_string(),
        "claude-haiku-4-5-20251001".to_string(),
    );

    // Uncomment to enable Opus → Sonnet rewrites (additional cost savings)
    // overrides.insert(
    //     "claude-opus-4-1-20250805".to_string(),
    //     "claude-sonnet-4.5-20250929".to_string(),
    // );

    info!("📋 Loaded {} model override rules", overrides.len());

    overrides
}

/// Claude history metrics REST endpoint
async fn claude_history_metrics(
) -> Result<Json<crate::claude_history::ClaudeMetrics>, ServerError> {
    let project_path = get_current_project_path()
        .map_err(|e| ServerError::Internal(format!("Failed to determine project path: {}", e)))?;

    let metrics = crate::claude_history::load_claude_project_metrics(&project_path)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to load Claude metrics: {}", e)))?;

    Ok(Json(metrics))
}

/// Load agent model configurations from orchestration config file
///
/// Embeds config/orchestra-config.json at compile time and extracts agent type -> model mappings
/// This ensures the proxy respects the orchestration layer's model assignments
fn load_agent_models() -> HashMap<String, String> {
    let mut agent_models = HashMap::new();

    // Embed orchestra-config.json at compile time (same pattern as dashboard.html)
    // Path is relative to this source file: cco/src/server.rs → ../../config/orchestra-config.json
    const ORCHESTRA_CONFIG: &str = include_str!("../../config/orchestra-config.json");

    match serde_json::from_str::<serde_json::Value>(ORCHESTRA_CONFIG) {
        Ok(config) => {
            // Extract architect model
            if let Some(architect) = config.get("architect") {
                if let Some(model) = architect.get("model").and_then(|m| m.as_str()) {
                    agent_models.insert("chief-architect".to_string(), model.to_string());
                    info!("✓ Loaded architect model: {}", model);
                }
            }

            // Extract coding agents models
            if let Some(agents) = config.get("codingAgents").and_then(|a| a.as_array()) {
                for agent in agents {
                    if let (Some(agent_type), Some(model)) = (
                        agent.get("type").and_then(|t| t.as_str()),
                        agent.get("model").and_then(|m| m.as_str()),
                    ) {
                        agent_models.insert(agent_type.to_string(), model.to_string());
                        info!("✓ Loaded agent model: {} → {}", agent_type, model);
                    }
                }
            }

            // Extract support agents models
            if let Some(agents) = config.get("supportTeam").and_then(|a| a.as_array()) {
                for agent in agents {
                    if let (Some(agent_type), Some(model)) = (
                        agent.get("type").and_then(|t| t.as_str()),
                        agent.get("model").and_then(|m| m.as_str()),
                    ) {
                        agent_models.insert(agent_type.to_string(), model.to_string());
                        info!("✓ Loaded agent model: {} → {}", agent_type, model);
                    }
                }
            }

            info!(
                "📊 Loaded {} agent model configurations from embedded orchestration config",
                agent_models.len()
            );
        }
        Err(e) => {
            info!(
                "⚠️  Failed to parse embedded orchestra-config.json: {}. Using defaults.",
                e
            );
        }
    }

    agent_models
}

/// Run the HTTP server
pub async fn run_server(
    host: &str,
    port: u16,
    cache_size: u64,
    cache_ttl: u64,
    debug: bool,
) -> anyhow::Result<()> {
    info!(
        "🚀 Starting CCO Proxy Server v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("→ Host: {}", host);
    info!("→ Port: {}", port);
    info!(
        "→ Cache size: {} bytes ({} MB)",
        cache_size,
        cache_size / 1_000_000
    );
    info!(
        "→ Cache TTL: {} seconds ({} hours)",
        cache_ttl,
        cache_ttl / 3600
    );

    // Log debug mode status
    if debug {
        info!("🐛 Debug mode: ENABLED");
    }

    // Setup file logging
    setup_file_logging(port)?;

    // Write PID file
    write_pid_file(port)?;
    info!("→ PID file: {:?}", get_pid_file(port)?);

    // Initialize components
    let cache = MokaCache::new(cache_size, cache_ttl);
    let router = ModelRouter::new();
    let analytics = Arc::new(AnalyticsEngine::new());
    let proxy = Arc::new(ProxyServer::new());
    let start_time = Instant::now();
    let model_overrides = Arc::new(load_model_overrides());
    let agent_models = Arc::new(load_agent_models());
    let agents = Arc::new(load_agents_from_embedded());

    // Load persisted metrics on startup
    if let Ok(persisted_records) = AnalyticsEngine::load_from_disk().await {
        let record_count = persisted_records.len();
        for record in persisted_records {
            analytics.record_api_call(record).await;
        }
        if record_count > 0 {
            info!("✅ Loaded {} metrics from disk", record_count);
        }
    }

    // Spawn background task to save metrics every 60 seconds
    let analytics_clone = analytics.clone();
    let metrics_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = analytics_clone.save_to_disk().await {
                tracing::warn!("Failed to save metrics: {}", e);
            }
        }
    });

    // Initialize connection tracker (max 10 concurrent connections per IP)
    let connection_tracker = ConnectionTracker::new(10);

    let state = Arc::new(ServerState {
        cache,
        router,
        analytics,
        proxy,
        start_time,
        model_overrides,
        agent_models,
        agents,
        connection_tracker,
    });

    // Build terminal route with localhost-only security
    // Note: CORS middleware is NOT applied to WebSocket routes as it can interfere with the upgrade
    // ConnectInfo is provided globally via into_make_service_with_connect_info
    let terminal_route = Router::new()
        .route("/terminal", get(terminal_handler))
        .layer(middleware::from_fn(localhost_only_middleware))
        .with_state(state.clone());

    // Build main app router
    let app = Router::new()
        // Dashboard routes
        .route("/", get(dashboard_html))
        .route("/dashboard.css", get(dashboard_css))
        .route("/dashboard.js", get(dashboard_js))
        // API routes
        .route("/health", get(health))
        .route("/api/agents", get(list_agents))
        .route("/api/agents/:agent_name", get(get_agent))
        .route("/api/stats", get(stats))
        .route("/api/project/stats", get(project_stats))
        .route("/api/machine/stats", get(machine_stats))
        .route("/api/metrics/projects", get(metrics_projects))
        .route("/api/metrics/claude-history", get(claude_history_metrics))
        .route("/api/overrides/stats", get(override_stats))
        .route("/api/shutdown", post(shutdown_handler))
        .route("/api/stream", get(stream))
        .route("/v1/chat/completions", post(chat_completion))
        .layer(CorsLayer::permissive())
        .merge(terminal_route)
        .with_state(state);

    // Create listener
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;

    info!("✅ Server listening on http://{}", addr);
    info!("→ Dashboard: http://{}/", addr);
    info!("→ Health check: http://{}/health", addr);
    info!("→ Agent API: http://{}/api/agents", addr);
    info!("→ Agent Details: http://{}/api/agents/:name", addr);
    info!("→ Analytics API: http://{}/api/stats", addr);
    info!("→ Project Metrics: http://{}/api/metrics/projects", addr);
    info!("→ SSE Stream: http://{}/api/stream", addr);
    info!("→ WebSocket Terminal: ws://{}/terminal", addr);
    info!("→ Chat endpoint: http://{}/v1/chat/completions", addr);
    info!("");
    info!("Press Ctrl+C to stop");

    // Run server with graceful shutdown
    // Note: into_make_service_with_connect_info is required for ConnectInfo extraction
    let result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await;

    // Abort background metrics task to prevent hanging on shutdown
    info!("Aborting background metrics task...");
    metrics_handle.abort();

    // Cleanup PID file
    info!("Cleaning up PID file...");
    if let Err(e) = remove_pid_file(port) {
        eprintln!("Failed to remove PID file: {}", e);
    }

    info!("Server shut down gracefully");
    result?;
    Ok(())
}

/// Wait for Ctrl+C signal
async fn shutdown_signal() {
    use tokio::signal;

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
