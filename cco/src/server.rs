//! HTTP server for CCO proxy

use crate::analytics::{AnalyticsEngine, ApiCallRecord};
use crate::cache::MokaCache;
use crate::proxy::{ChatRequest, ChatResponse, ProxyServer};
use crate::router::ModelRouter;
use crate::version::DateVersion;
use chrono::Utc;
use dirs::data_local_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, State,
    },
    http::{header, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Response,
    },
    routing::{get, post},
    Router,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

/// Server state shared across handlers
#[derive(Clone)]
pub struct ServerState {
    pub cache: MokaCache,
    pub router: ModelRouter,
    pub analytics: Arc<AnalyticsEngine>,
    pub proxy: Arc<ProxyServer>,
    pub start_time: Instant,
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
    let data_dir = data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get local data directory"))?;

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
    let data_dir = data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get local data directory"))?;

    let logs_dir = data_dir.join("cco").join("logs");

    // Create directory if it doesn't exist
    fs::create_dir_all(&logs_dir)?;

    Ok(logs_dir)
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

/// Chat completion endpoint
async fn chat_completion(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServerError> {
    info!("Received chat request for model: {}", request.model);

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

    // Calculate cost
    if let Some(cost) = state
        .router
        .calculate_cost(&request.model, response.input_tokens, response.output_tokens)
    {
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
    (
        [(header::CONTENT_TYPE, "text/css")],
        css,
    )
}

/// Dashboard JavaScript
async fn dashboard_js() -> impl IntoResponse {
    let js = include_str!("../static/dashboard.js");
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        js,
    )
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
async fn project_stats(State(state): State<Arc<ServerState>>) -> Result<Json<StatsResponse>, ServerError> {
    // For now, return the same data as general stats
    // In the future, this could be project-specific
    stats(State(state)).await
}

/// Machine stats endpoint (currently same as general stats)
async fn machine_stats(State(state): State<Arc<ServerState>>) -> Result<Json<StatsResponse>, ServerError> {
    // For now, return the same data as general stats
    // In the future, this could aggregate across multiple projects
    stats(State(state)).await
}

/// SSE stream endpoint for real-time analytics updates
async fn stream(
    State(state): State<Arc<ServerState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;

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

            let stats = StatsResponse {
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
            };

            // Serialize to JSON
            if let Ok(json) = serde_json::to_string(&stats) {
                yield Ok(Event::default().event("analytics").data(json));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// WebSocket terminal endpoint handler
async fn terminal_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_terminal_socket(socket, state))
}

/// Handle WebSocket terminal connection
async fn handle_terminal_socket(socket: WebSocket, _state: Arc<ServerState>) {
    use futures::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();

    // Send welcome message
    let welcome_msg = r#"{
        "type": "output",
        "data": "CCO Terminal v2025.11.2\nType 'help' for available commands.\n\n$ "
    }"#;

    if sender.send(Message::Text(welcome_msg.to_string())).await.is_err() {
        return;
    }

    // Handle incoming messages
    while let Some(msg) = StreamExt::next(&mut receiver).await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse command from JSON
                if let Ok(cmd_json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(command) = cmd_json.get("command").and_then(|v| v.as_str()) {
                        let response = execute_command(command).await;

                        let response_json = serde_json::json!({
                            "type": "output",
                            "data": response
                        });

                        if let Ok(json_str) = serde_json::to_string(&response_json) {
                            if sender.send(Message::Text(json_str)).await.is_err() {
                                break;
                            }
                        }

                        // Send prompt
                        let prompt_json = serde_json::json!({
                            "type": "output",
                            "data": "$ "
                        });

                        if let Ok(json_str) = serde_json::to_string(&prompt_json) {
                            if sender.send(Message::Text(json_str)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            _ => {}
        }
    }
}

/// Execute a terminal command
async fn execute_command(command: &str) -> String {
    let cmd = command.trim();

    match cmd {
        "" => String::new(),
        "help" => {
            r#"Available commands:
  help           - Show this help message
  version        - Show CCO version
  status         - Show system status
  cache stats    - Show cache statistics
  clear          - Clear screen
  exit           - Close terminal
"#.to_string()
        }
        "version" => {
            format!("CCO Proxy v{}\n", DateVersion::current())
        }
        "status" => {
            "Status: Running\nUptime: Active\nCache: Operational\n".to_string()
        }
        "cache stats" => {
            "Cache Statistics:\n  Hit Rate: N/A\n  Entries: N/A\n  Size: N/A\n".to_string()
        }
        "clear" => {
            "\x1b[2J\x1b[H".to_string() // ANSI clear screen
        }
        "exit" => {
            "Goodbye!\n".to_string()
        }
        _ => {
            format!("Unknown command: {}\nType 'help' for available commands.\n", cmd)
        }
    }
}

/// Run the HTTP server
pub async fn run_server(
    host: &str,
    port: u16,
    cache_size: u64,
    cache_ttl: u64,
) -> anyhow::Result<()> {
    info!("🚀 Starting CCO Proxy Server v{}", env!("CARGO_PKG_VERSION"));
    info!("→ Host: {}", host);
    info!("→ Port: {}", port);
    info!("→ Cache size: {} bytes ({} MB)", cache_size, cache_size / 1_000_000);
    info!("→ Cache TTL: {} seconds ({} hours)", cache_ttl, cache_ttl / 3600);

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

    let state = Arc::new(ServerState {
        cache,
        router,
        analytics,
        proxy,
        start_time,
    });

    // Build router
    let app = Router::new()
        // Dashboard routes
        .route("/", get(dashboard_html))
        .route("/dashboard.css", get(dashboard_css))
        .route("/dashboard.js", get(dashboard_js))
        // API routes
        .route("/health", get(health))
        .route("/api/stats", get(stats))
        .route("/api/project/stats", get(project_stats))
        .route("/api/machine/stats", get(machine_stats))
        .route("/api/stream", get(stream))
        .route("/terminal", get(terminal_handler))
        .route("/v1/chat/completions", post(chat_completion))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Create listener
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;

    info!("✅ Server listening on http://{}", addr);
    info!("→ Dashboard: http://{}/", addr);
    info!("→ Health check: http://{}/health", addr);
    info!("→ Analytics API: http://{}/api/stats", addr);
    info!("→ SSE Stream: http://{}/api/stream", addr);
    info!("→ WebSocket Terminal: ws://{}/terminal", addr);
    info!("→ Chat endpoint: http://{}/v1/chat/completions", addr);
    info!("");
    info!("Press Ctrl+C to stop");

    // Run server with graceful shutdown
    let result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;

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
