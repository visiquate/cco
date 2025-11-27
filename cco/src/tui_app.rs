//! TUI Application for daemon management and monitoring
//!
//! Provides an interactive terminal interface for:
//! - Auto-starting daemon if not running
//! - Monitoring daemon status and health
//! - Displaying real-time agent and metrics information
//! - Keyboard controls for daemon restart and shutdown

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, SystemTime};
use std::path::PathBuf;
use tokio::time::{sleep, interval};
use tokio::sync::mpsc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::api_client::{ApiClient, HealthResponse};
use crate::daemon::{DaemonConfig, DaemonManager};

/// Cost breakdown by tier
#[derive(Debug, Clone)]
pub struct CostByTier {
    pub sonnet_cost: f64,
    pub sonnet_pct: f64,
    pub sonnet_calls: u64,
    pub sonnet_tokens: TokenStats,
    pub opus_cost: f64,
    pub opus_pct: f64,
    pub opus_calls: u64,
    pub opus_tokens: TokenStats,
    pub haiku_cost: f64,
    pub haiku_pct: f64,
    pub haiku_calls: u64,
    pub haiku_tokens: TokenStats,
    pub total_cost: f64,
    pub total_calls: u64,
    pub total_tokens: TokenStats,
}

/// Model summary from API
#[derive(Debug, Clone)]
pub struct ModelSummary {
    pub model: String,
    pub cost: f64,
    pub tokens: u64,
    pub messages: u64,
    pub percentage: f64,
}

/// Token statistics per tier
#[derive(Debug, Clone, Default)]
pub struct TokenStats {
    pub input: u64,
    pub output: u64,
    pub cache_write: u64,
    pub cache_read: u64,
}

impl Default for CostByTier {
    fn default() -> Self {
        Self {
            sonnet_cost: 0.0,
            sonnet_pct: 0.0,
            sonnet_calls: 0,
            sonnet_tokens: TokenStats { input: 0, output: 0, cache_write: 0, cache_read: 0 },
            opus_cost: 0.0,
            opus_pct: 0.0,
            opus_calls: 0,
            opus_tokens: TokenStats { input: 0, output: 0, cache_write: 0, cache_read: 0 },
            haiku_cost: 0.0,
            haiku_pct: 0.0,
            haiku_calls: 0,
            haiku_tokens: TokenStats { input: 0, output: 0, cache_write: 0, cache_read: 0 },
            total_cost: 0.0,
            total_calls: 0,
            total_tokens: TokenStats { input: 0, output: 0, cache_write: 0, cache_read: 0 },
        }
    }
}

/// Time range filter for metrics
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeRange {
    Today,
    Week,
    Month,
    AllTime,
}

impl TimeRange {
    /// Get display name for the time range
    pub fn display_name(&self) -> &str {
        match self {
            TimeRange::Today => "Today",
            TimeRange::Week => "This Week",
            TimeRange::Month => "This Month",
            TimeRange::AllTime => "All Time",
        }
    }

    /// Cycle to the next time range
    pub fn next(&self) -> Self {
        match self {
            TimeRange::Today => TimeRange::Week,
            TimeRange::Week => TimeRange::Month,
            TimeRange::Month => TimeRange::AllTime,
            TimeRange::AllTime => TimeRange::Today,
        }
    }

    /// Get query parameter value for API calls
    pub fn as_query_param(&self) -> &str {
        match self {
            TimeRange::Today => "today",
            TimeRange::Week => "week",
            TimeRange::Month => "month",
            TimeRange::AllTime => "all",
        }
    }
}

impl Default for TimeRange {
    fn default() -> Self {
        TimeRange::AllTime
    }
}

/// Recent API call information
#[derive(Debug, Clone)]
pub struct RecentCall {
    pub timestamp: String,
    pub model: String,
    pub tokens: u64,
    pub cost: f64,
}

/// Overall summary from metrics
#[derive(Debug, Clone, Default)]
pub struct OverallSummary {
    pub total_cost: f64,
    pub total_tokens: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_calls: u64,
    pub opus_cost: f64,
    pub sonnet_cost: f64,
    pub haiku_cost: f64,
}

/// Per-project summary
#[derive(Debug, Clone)]
pub struct ProjectSummary {
    pub name: String,
    pub cost: f64,
    pub tokens: u64,
    pub calls: u64,
}

/// Application state tracking
#[derive(Debug, Clone)]
pub enum AppState {
    /// Initial startup - checking daemon status
    Initializing {
        message: String,
    },
    /// Daemon is starting up
    DaemonStarting {
        progress: u8,
    },
    /// Connected and running normally
    Connected {
        cost_by_tier: CostByTier,
        recent_calls: Vec<RecentCall>,
        health: HealthResponse,
        is_active: bool,
        overall_summary: OverallSummary,
        project_summaries: Vec<ProjectSummary>,
        model_summaries: Vec<ModelSummary>,
        last_updated: SystemTime,
        time_range: TimeRange,
        history_start_date: Option<SystemTime>,
    },
    /// Error state with message
    Error(String),
    /// Shutting down
    Shutting {
        message: String,
    },
}

/// Stats update message for background refresh
#[derive(Debug, Clone)]
struct StatsUpdate {
    cost_by_tier: CostByTier,
    recent_calls: Vec<RecentCall>,
    health: HealthResponse,
    is_active: bool,
    overall_summary: OverallSummary,
    project_summaries: Vec<ProjectSummary>,
    model_summaries: Vec<ModelSummary>,
    history_start_date: Option<SystemTime>,
}

/// Main TUI application
pub struct TuiApp {
    /// Current application state
    state: AppState,
    /// API client for daemon communication
    client: ApiClient,
    /// Daemon manager for lifecycle control
    daemon_manager: DaemonManager,
    /// Ratatui terminal
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// Should quit flag
    should_quit: bool,
    /// Status message
    status_message: String,
}

/// Load overall metrics from ~/.claude/metrics.json
async fn load_overall_metrics() -> Result<OverallSummary> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let metrics_path = PathBuf::from(&home).join(".claude").join("metrics.json");

    if !metrics_path.exists() {
        return Ok(OverallSummary::default());
    }

    let content = fs::read_to_string(&metrics_path).await?;
    let metrics: serde_json::Value = serde_json::from_str(&content)?;

    let summary = OverallSummary {
        total_cost: metrics.get("total_cost")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        total_tokens: metrics.get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        total_input_tokens: metrics.get("total_input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        total_output_tokens: metrics.get("total_output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        total_calls: metrics.get("messages_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        opus_cost: metrics.get("model_breakdown")
            .and_then(|mb| mb.get("claude-opus-4"))
            .and_then(|m| m.get("total_cost"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        sonnet_cost: metrics.get("model_breakdown")
            .and_then(|mb| mb.get("claude-sonnet-4-5"))
            .and_then(|m| m.get("total_cost"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        haiku_cost: metrics.get("model_breakdown")
            .and_then(|mb| mb.get("claude-haiku-4-5"))
            .and_then(|m| m.get("total_cost"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
    };

    Ok(summary)
}

/// Load project summaries from ~/.claude/projects/*/claude.jsonl
async fn load_project_summaries() -> Result<Vec<ProjectSummary>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let projects_dir = PathBuf::from(&home).join(".claude").join("projects");

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();
    let mut entries = fs::read_dir(&projects_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            let claude_jsonl = path.join("claude.jsonl");
            if claude_jsonl.exists() {
                if let Ok(summary) = load_project_from_jsonl(&claude_jsonl, &path).await {
                    projects.push(summary);
                }
            }
        }
    }

    // Sort by cost descending
    projects.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap_or(std::cmp::Ordering::Equal));

    Ok(projects)
}

/// Load a single project's metrics from its claude.jsonl file
async fn load_project_from_jsonl(jsonl_path: &PathBuf, project_dir: &PathBuf) -> Result<ProjectSummary> {
    let file = fs::File::open(jsonl_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut total_cost = 0.0;
    let mut total_tokens = 0u64;
    let mut calls = 0u64;

    while let Some(line) = lines.next_line().await? {
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(message) = msg.get("message") {
                if let Some(usage) = message.get("usage") {
                    calls += 1;

                    let input_tokens = usage.get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let output_tokens = usage.get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    total_tokens += input_tokens + output_tokens;

                    // Extract model name and estimate cost
                    if let Some(model) = message.get("model").and_then(|m| m.as_str()) {
                        let (input_price, output_price, _, _) = get_model_pricing(model);
                        total_cost += (input_tokens as f64 / 1_000_000.0) * input_price;
                        total_cost += (output_tokens as f64 / 1_000_000.0) * output_price;
                    }
                }
            }
        }
    }

    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(ProjectSummary {
        name: project_name,
        cost: total_cost,
        tokens: total_tokens,
        calls,
    })
}

/// Get model pricing for cost estimation (matches pricing from claude_history.rs)
fn get_model_pricing(model_name: &str) -> (f64, f64, f64, f64) {
    let normalized = normalize_model_name(model_name);
    match normalized.as_str() {
        "claude-sonnet-4-5" | "claude-3-5-sonnet" => (3.0, 15.0, 3.75, 0.30),
        "claude-haiku-4-5" | "claude-3-5-haiku" => (1.0, 5.0, 1.25, 0.10),
        "claude-opus-4" | "claude-opus-4-1" => (15.0, 75.0, 18.75, 1.50),
        _ => (3.0, 15.0, 3.75, 0.30), // Default to Sonnet
    }
}

/// Normalize model name for pricing lookup
fn normalize_model_name(model_name: &str) -> String {
    let parts: Vec<&str> = model_name.split('-').collect();
    if parts.len() >= 3 {
        if let Some(last) = parts.last() {
            if last.len() == 8 && last.chars().all(|c| c.is_ascii_digit()) {
                return parts[..parts.len() - 1].join("-");
            }
        }
    }
    model_name.to_string()
}

impl TuiApp {
    /// Create a new TUI application instance
    pub async fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;

        // Create daemon manager with default config
        let config = DaemonConfig::default();
        let daemon_manager = DaemonManager::new(config.clone());

        // Auto-discover daemon port from PID file (if daemon is running)
        // Otherwise fall back to default port (will trigger daemon start in ensure_daemon_running)
        let actual_port = crate::daemon::read_daemon_port().unwrap_or(config.port);

        // Create API client with discovered port
        let base_url = format!("http://{}:{}", config.host, actual_port);
        let client = ApiClient::new(base_url);

        Ok(Self {
            state: AppState::Initializing {
                message: "Checking daemon status...".to_string(),
            },
            client,
            daemon_manager,
            terminal,
            should_quit: false,
            status_message: String::new(),
        })
    }

    /// Run the TUI application main loop
    pub async fn run(&mut self) -> Result<()> {
        // 1. Ensure daemon is running
        self.ensure_daemon_running().await?;

        // 2. Load initial data from daemon
        self.load_agents_and_stats().await?;

        // 3. Spawn background stats fetcher task
        let (stats_tx, mut stats_rx) = mpsc::channel::<StatsUpdate>(10);
        let client_clone = self.client.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3));
            loop {
                interval.tick().await;

                // Fetch stats from daemon
                if let Ok(update) = Self::fetch_stats_update(&client_clone).await {
                    // Send update (non-blocking)
                    if stats_tx.send(update).await.is_err() {
                        // Receiver dropped, exit task
                        break;
                    }
                }
                // If fetch fails, silently continue - will retry in 3 seconds
            }
        });

        // 4. Enter main event loop
        loop {
            // Render current state
            self.render()?;

            // Handle user input with timeout
            if crossterm::event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_input(key).await? {
                        break; // Exit requested
                    }
                }
            }

            // Check for stats updates from background task
            if let Ok(update) = stats_rx.try_recv() {
                self.update_state_from_stats(update);
            }

            // Check if we should quit
            if self.should_quit {
                break;
            }
        }

        // 5. Cleanup on exit
        self.shutdown().await?;
        Ok(())
    }

    /// Ensure daemon is running, start if needed
    async fn ensure_daemon_running(&mut self) -> Result<()> {
        // Check current daemon status
        match self.daemon_manager.get_status().await {
            Ok(status) if status.is_running => {
                // Daemon is running, verify with health check
                self.state = AppState::Initializing {
                    message: format!("Connected to daemon (PID: {})", status.pid),
                };

                // Verify health
                match self.client.health().await {
                    Ok(_) => {
                        self.status_message = format!("Daemon running on port {}", status.port);
                    }
                    Err(e) => {
                        self.state = AppState::Error(format!(
                            "Daemon process exists but not responding: {}",
                            e
                        ));
                        return Err(anyhow::anyhow!(
                            "Daemon process exists but not responding"
                        ));
                    }
                }
            }
            _ => {
                // Daemon not running, start it
                self.state = AppState::DaemonStarting { progress: 0 };
                self.render()?;

                self.status_message = "Starting daemon...".to_string();
                self.daemon_manager
                    .start()
                    .await
                    .context("Failed to start daemon")?;

                // Wait for daemon to become ready
                self.wait_for_daemon_ready().await?;

                self.state = AppState::DaemonStarting { progress: 100 };
                self.status_message = "Daemon started successfully".to_string();
            }
        }

        Ok(())
    }

    /// Wait for daemon to become ready
    async fn wait_for_daemon_ready(&mut self) -> Result<()> {
        let max_wait = Duration::from_secs(45); // Increased from 30 to 45 seconds for slower systems
        let start = std::time::Instant::now();
        #[allow(unused_assignments)]
        let mut progress = 0u8;

        // Add initial delay before first poll - daemon needs time to initialize
        sleep(Duration::from_millis(2000)).await;

        while start.elapsed() < max_wait {
            // Update progress
            progress = ((start.elapsed().as_secs_f64() / max_wait.as_secs_f64()) * 100.0) as u8;
            progress = progress.min(95); // Cap at 95% until actually connected

            self.state = AppState::DaemonStarting { progress };
            self.render()?;

            // Try ready check (faster than health check)
            // The /ready endpoint just checks if daemon is responsive,
            // while /health includes analytics processing
            match self.client.ready().await {
                Ok(_) => {
                    return Ok(());
                }
                Err(_) => {
                    // If ready check fails, try once with health as fallback
                    match self.client.health().await {
                        Ok(health) if health.status == "ok" => {
                            return Ok(());
                        }
                        _ => {
                            // Neither ready nor health succeeded, wait and retry
                            sleep(Duration::from_millis(200)).await; // Faster polling (200ms vs 500ms)
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Timeout waiting for daemon to become ready"))
    }

    /// Load cost metrics and recent calls from daemon
    async fn load_agents_and_stats(&mut self) -> Result<()> {
        // Fetch data from daemon API
        let health = self.client.health().await;

        // Extract current time range if in Connected state, default to AllTime
        let time_range = if let AppState::Connected { time_range, .. } = self.state {
            time_range
        } else {
            TimeRange::AllTime
        };

        let time_range_param = time_range.as_query_param();
        let stats_url = format!("{}/api/stats?time_range={}", self.client.base_url, time_range_param);
        let stats_response: Result<serde_json::Value, _> = self.client.get_with_retry(&stats_url).await;

        match (health, stats_response) {
            (Ok(health), Ok(stats)) => {
                // Parse cost by tier from stats response
                let cost_by_tier = self.parse_cost_by_tier(&stats);

                // Parse recent calls from activity events
                let recent_calls = self.parse_recent_calls(&stats);

                // Determine if system is active (has recent activity)
                let is_active = !recent_calls.is_empty();

                // Parse model summaries from API response (needed for tier calculations)
                let model_summaries = self.parse_model_summaries(&stats);

                // Calculate tier costs from model summaries
                let mut opus_cost = 0.0;
                let mut sonnet_cost = 0.0;
                let mut haiku_cost = 0.0;

                for model in &model_summaries {
                    let model_lower = model.model.to_lowercase();
                    if model_lower.contains("opus") {
                        opus_cost += model.cost;
                    } else if model_lower.contains("sonnet") {
                        sonnet_cost += model.cost;
                    } else if model_lower.contains("haiku") {
                        haiku_cost += model.cost;
                    }
                }

                // Create overall metrics from /api/stats data
                let overall_summary = OverallSummary {
                    total_cost: stats.get("project")
                        .and_then(|p| p.get("cost"))
                        .and_then(|c| c.as_f64())
                        .unwrap_or(0.0),
                    total_tokens: stats.get("project")
                        .and_then(|p| p.get("tokens"))
                        .and_then(|t| t.as_u64())
                        .unwrap_or(0),
                    total_input_tokens: (stats.get("project")
                        .and_then(|p| p.get("tokens"))
                        .and_then(|t| t.as_u64())
                        .unwrap_or(0) as f64 * 0.6) as u64,
                    total_output_tokens: (stats.get("project")
                        .and_then(|p| p.get("tokens"))
                        .and_then(|t| t.as_u64())
                        .unwrap_or(0) as f64 * 0.4) as u64,
                    total_calls: stats.get("project")
                        .and_then(|p| p.get("messages"))
                        .and_then(|m| m.as_u64())
                        .unwrap_or(0),
                    opus_cost,
                    sonnet_cost,
                    haiku_cost,
                };

                // Parse project summaries from API response
                let project_summaries = self.parse_project_summaries(&stats);

                // Parse history_start_date from response
                let history_start_date = stats.get("history_start_date")
                    .and_then(|d| d.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| {
                        std::time::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64)
                    });

                self.state = AppState::Connected {
                    cost_by_tier,
                    recent_calls,
                    health,
                    is_active,
                    overall_summary,
                    project_summaries,
                    model_summaries,
                    last_updated: SystemTime::now(),
                    time_range,
                    history_start_date,
                };
            }
            (Err(e), _) | (_, Err(e)) => {
                self.state = AppState::Error(format!("Failed to load data: {}", e));
            }
        }
        Ok(())
    }

    /// Fetch stats update from daemon (used by background task)
    async fn fetch_stats_update(client: &ApiClient) -> Result<StatsUpdate> {
        let health = client.health().await?;

        // TODO: Pass time_range parameter from AppState to background task
        // For now, default to "all" until we wire up the parameter
        let time_range_param = "all";
        let stats_url = format!("{}/api/stats?time_range={}", client.base_url, time_range_param);
        let stats: serde_json::Value = client.get_with_retry(&stats_url).await?;

        // Parse cost by tier
        let cost_by_tier = Self::parse_cost_by_tier_static(&stats);

        // Parse recent calls
        let recent_calls = Self::parse_recent_calls_static(&stats);

        // Determine if system is active
        let is_active = !recent_calls.is_empty();

        // Parse model summaries first (needed for tier calculations)
        let model_summaries = Self::parse_model_summaries_static(&stats);

        // Calculate tier costs from model summaries
        let mut opus_cost = 0.0;
        let mut sonnet_cost = 0.0;
        let mut haiku_cost = 0.0;

        for model in &model_summaries {
            let model_lower = model.model.to_lowercase();
            if model_lower.contains("opus") {
                opus_cost += model.cost;
            } else if model_lower.contains("sonnet") {
                sonnet_cost += model.cost;
            } else if model_lower.contains("haiku") {
                haiku_cost += model.cost;
            }
        }

        // Create overall summary with calculated tier costs
        let overall_summary = OverallSummary {
            total_cost: stats.get("project")
                .and_then(|p| p.get("cost"))
                .and_then(|c| c.as_f64())
                .unwrap_or(0.0),
            total_tokens: stats.get("project")
                .and_then(|p| p.get("tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            total_input_tokens: (stats.get("project")
                .and_then(|p| p.get("tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as f64 * 0.6) as u64,
            total_output_tokens: (stats.get("project")
                .and_then(|p| p.get("tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as f64 * 0.4) as u64,
            total_calls: stats.get("project")
                .and_then(|p| p.get("messages"))
                .and_then(|m| m.as_u64())
                .unwrap_or(0),
            opus_cost,
            sonnet_cost,
            haiku_cost,
        };

        // Parse project summaries
        let project_summaries = Self::parse_project_summaries_static(&stats);

        // Parse history_start_date from response
        let history_start_date = stats.get("history_start_date")
            .and_then(|d| d.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| {
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64)
            });

        Ok(StatsUpdate {
            cost_by_tier,
            recent_calls,
            health,
            is_active,
            overall_summary,
            project_summaries,
            model_summaries,
            history_start_date,
        })
    }

    /// Update application state from stats update
    fn update_state_from_stats(&mut self, update: StatsUpdate) {
        if let AppState::Connected { time_range, .. } = self.state {
            self.state = AppState::Connected {
                cost_by_tier: update.cost_by_tier,
                recent_calls: update.recent_calls,
                health: update.health,
                is_active: update.is_active,
                overall_summary: update.overall_summary,
                project_summaries: update.project_summaries,
                model_summaries: update.model_summaries,
                last_updated: SystemTime::now(),
                time_range, // Preserve user's time range selection
                history_start_date: update.history_start_date,
            };
        }
    }

    /// Parse cost breakdown by tier from stats JSON (static version)
    fn parse_cost_by_tier_static(stats: &serde_json::Value) -> CostByTier {
        let mut cost_by_tier = CostByTier::default();

        // Get total cost and calls from stats.project
        let total_cost = stats.get("project")
            .and_then(|p| p.get("cost"))
            .and_then(|c| c.as_f64())
            .unwrap_or(0.0);
        let total_calls = stats.get("project")
            .and_then(|p| p.get("messages"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0);
        let total_tokens = stats.get("project")
            .and_then(|p| p.get("tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0);

        // Try to extract model_distribution from chart_data
        if let Some(chart_data) = stats.get("chart_data") {
            if let Some(model_distribution) = chart_data.get("model_distribution").and_then(|d| d.as_array()) {
                let mut sonnet_cost = 0.0;
                let mut opus_cost = 0.0;
                let mut haiku_cost = 0.0;
                let mut sonnet_calls = 0u64;
                let mut opus_calls = 0u64;
                let mut haiku_calls = 0u64;

                // Sum costs per model name to calculate totals
                for model_item in model_distribution {
                    if let Some(model_name) = model_item.get("model").and_then(|m| m.as_str()) {
                        // Estimate cost based on percentage if available
                        if let Some(percentage) = model_item.get("percentage").and_then(|p| p.as_f64()) {
                            let cost = (total_cost * percentage) / 100.0;
                            let calls = ((total_calls as f64 * percentage) / 100.0) as u64;

                            if model_name.to_lowercase().contains("sonnet") {
                                sonnet_cost += cost;
                                sonnet_calls += calls;
                            } else if model_name.to_lowercase().contains("opus") {
                                opus_cost += cost;
                                opus_calls += calls;
                            } else if model_name.to_lowercase().contains("haiku") {
                                haiku_cost += cost;
                                haiku_calls += calls;
                            }
                        }
                    }
                }

                // Calculate percentages (handle division by zero)
                let total_calculated = sonnet_cost + opus_cost + haiku_cost;
                let (sonnet_pct, opus_pct, haiku_pct) = if total_calculated > 0.0 {
                    (
                        (sonnet_cost / total_calculated) * 100.0,
                        (opus_cost / total_calculated) * 100.0,
                        (haiku_cost / total_calculated) * 100.0,
                    )
                } else {
                    (0.0, 0.0, 0.0)
                };

                // Extract token statistics per model from chart_data if available
                let (sonnet_tokens, opus_tokens, haiku_tokens) =
                    Self::extract_token_stats_per_model_static(stats, total_tokens);

                // Build total token stats
                let total_token_stats = TokenStats {
                    input: (total_tokens as f64 * 0.6) as u64,
                    output: (total_tokens as f64 * 0.4) as u64,
                    cache_write: 0,
                    cache_read: 0,
                };

                cost_by_tier = CostByTier {
                    sonnet_cost,
                    sonnet_pct,
                    sonnet_calls,
                    sonnet_tokens,
                    opus_cost,
                    opus_pct,
                    opus_calls,
                    opus_tokens,
                    haiku_cost,
                    haiku_pct,
                    haiku_calls,
                    haiku_tokens,
                    total_cost: total_calculated,
                    total_calls,
                    total_tokens: total_token_stats,
                };
            }
        }

        cost_by_tier
    }

    /// Parse cost breakdown by tier from stats JSON
    fn parse_cost_by_tier(&self, stats: &serde_json::Value) -> CostByTier {
        Self::parse_cost_by_tier_static(stats)
    }

    /// Extract token statistics per model from model_breakdown in chart_data (static version)
    fn extract_token_stats_per_model_static(stats: &serde_json::Value, total_tokens: u64) -> (TokenStats, TokenStats, TokenStats) {
        let mut sonnet_stats = TokenStats::default();
        let mut opus_stats = TokenStats::default();
        let mut haiku_stats = TokenStats::default();

        // Try to extract from model_breakdown if available
        if let Some(chart_data) = stats.get("chart_data") {
            if let Some(model_distribution) = chart_data.get("model_distribution").and_then(|d| d.as_array()) {
                for model_item in model_distribution {
                    if let Some(model_name) = model_item.get("model").and_then(|m| m.as_str()) {
                        if let Some(percentage) = model_item.get("percentage").and_then(|p| p.as_f64()) {
                            let model_tokens = ((total_tokens as f64 * percentage) / 100.0) as u64;
                            let estimated_input = (model_tokens as f64 * 0.6) as u64;
                            let estimated_output = (model_tokens as f64 * 0.4) as u64;

                            if model_name.to_lowercase().contains("sonnet") {
                                sonnet_stats.input += estimated_input;
                                sonnet_stats.output += estimated_output;
                            } else if model_name.to_lowercase().contains("opus") {
                                opus_stats.input += estimated_input;
                                opus_stats.output += estimated_output;
                            } else if model_name.to_lowercase().contains("haiku") {
                                haiku_stats.input += estimated_input;
                                haiku_stats.output += estimated_output;
                            }
                        }
                    }
                }
            }
        }

        (sonnet_stats, opus_stats, haiku_stats)
    }

    /// Extract token statistics per model from model_breakdown in chart_data
    fn extract_token_stats_per_model(&self, stats: &serde_json::Value, total_tokens: u64) -> (TokenStats, TokenStats, TokenStats) {
        Self::extract_token_stats_per_model_static(stats, total_tokens)
    }

    /// Parse recent API calls from activity events (static version)
    fn parse_recent_calls_static(stats: &serde_json::Value) -> Vec<RecentCall> {
        let mut calls = Vec::new();

        if let Some(recent_calls) = stats.get("activity")
            .and_then(|a| a.get("recent_calls"))
            .and_then(|rc| rc.as_array())
        {
            for call in recent_calls.iter() {
                if let (Some(timestamp), Some(model), Some(tokens), Some(cost)) = (
                    call.get("timestamp").and_then(|t| t.as_str()),
                    call.get("model").and_then(|m| m.as_str()),
                    call.get("tokens").and_then(|t| t.as_u64()),
                    call.get("cost").and_then(|c| c.as_f64()),
                ) {
                    calls.push(RecentCall {
                        timestamp: timestamp.to_string(),
                        model: model.to_string(),
                        tokens,
                        cost,
                    });
                }
            }
        }

        calls
    }

    /// Parse recent API calls from activity events
    fn parse_recent_calls(&self, stats: &serde_json::Value) -> Vec<RecentCall> {
        Self::parse_recent_calls_static(stats)
    }

    /// Parse project summaries from stats JSON (static version)
    fn parse_project_summaries_static(stats: &serde_json::Value) -> Vec<ProjectSummary> {
        let mut projects = Vec::new();

        if let Some(projects_array) = stats.get("projects").and_then(|p| p.as_array()) {
            for project in projects_array {
                if let (Some(name), Some(cost), Some(tokens), Some(messages)) = (
                    project.get("name").and_then(|n| n.as_str()),
                    project.get("cost").and_then(|c| c.as_f64()),
                    project.get("tokens").and_then(|t| t.as_u64()),
                    project.get("messages").and_then(|m| m.as_u64()),
                ) {
                    projects.push(ProjectSummary {
                        name: name.to_string(),
                        cost,
                        tokens,
                        calls: messages,
                    });
                }
            }
        }

        projects
    }

    /// Parse project summaries from stats JSON
    fn parse_project_summaries(&self, stats: &serde_json::Value) -> Vec<ProjectSummary> {
        Self::parse_project_summaries_static(stats)
    }

    /// Parse model summaries from stats JSON (static version)
    fn parse_model_summaries_static(stats: &serde_json::Value) -> Vec<ModelSummary> {
        let mut models = Vec::new();

        if let Some(models_array) = stats.get("models").and_then(|m| m.as_array()) {
            for model in models_array {
                if let (Some(name), Some(cost), Some(tokens), Some(messages), Some(percentage)) = (
                    model.get("model").and_then(|n| n.as_str()),
                    model.get("cost").and_then(|c| c.as_f64()),
                    model.get("tokens").and_then(|t| t.as_u64()),
                    model.get("messages").and_then(|m| m.as_u64()),
                    model.get("percentage").and_then(|p| p.as_f64()),
                ) {
                    models.push(ModelSummary {
                        model: name.to_string(),
                        cost,
                        tokens,
                        messages,
                        percentage,
                    });
                }
            }
        }

        models
    }

    /// Parse model summaries from stats JSON
    fn parse_model_summaries(&self, stats: &serde_json::Value) -> Vec<ModelSummary> {
        Self::parse_model_summaries_static(stats)
    }

    /// Handle keyboard input
    async fn handle_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.should_quit = true;
                return Ok(true);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Restart daemon
                self.status_message = "Restarting daemon...".to_string();
                self.state = AppState::Initializing {
                    message: "Restarting daemon...".to_string(),
                };
                self.render()?;

                if let Err(e) = self.daemon_manager.restart().await {
                    self.state = AppState::Error(format!("Failed to restart daemon: {}", e));
                } else {
                    self.wait_for_daemon_ready().await?;
                    self.load_agents_and_stats().await?;
                    self.status_message = "Daemon restarted successfully".to_string();
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                // Cycle through time ranges
                if let AppState::Connected { time_range, .. } = self.state {
                    let new_time_range = time_range.next();

                    // Update the state with new time range
                    if let AppState::Connected {
                        cost_by_tier,
                        recent_calls,
                        health,
                        is_active,
                        overall_summary,
                        project_summaries,
                        model_summaries,
                        last_updated,
                        history_start_date,
                        ..
                    } = self.state.clone()
                    {
                        self.state = AppState::Connected {
                            cost_by_tier,
                            recent_calls,
                            health,
                            is_active,
                            overall_summary,
                            project_summaries,
                            model_summaries,
                            last_updated,
                            time_range: new_time_range,
                            history_start_date,
                        };
                    }

                    self.status_message = format!("Loading {} data...", new_time_range.display_name());
                    self.render()?;

                    // Reload data from daemon with new time range filter
                    if let Err(e) = self.load_agents_and_stats().await {
                        self.state = AppState::Error(format!("Failed to reload data: {}", e));
                    } else {
                        self.status_message = format!("Time range: {}", new_time_range.display_name());
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    /// Render the current UI frame
    fn render(&mut self) -> Result<()> {
        let state = self.state.clone();
        let status_message = self.status_message.clone();

        self.terminal.draw(|f| {
            match &state {
                AppState::Initializing { message } => {
                    Self::render_initializing(f, message);
                }
                AppState::DaemonStarting { progress } => {
                    Self::render_starting(f, *progress);
                }
                AppState::Connected {
                    cost_by_tier,
                    recent_calls,
                    health,
                    is_active,
                    overall_summary,
                    project_summaries,
                    model_summaries,
                    last_updated,
                    time_range,
                    history_start_date,
                } => {
                    Self::render_connected(f, cost_by_tier, recent_calls, health, *is_active, overall_summary, project_summaries, model_summaries, &status_message, *last_updated, *time_range, *history_start_date);
                }
                AppState::Error(err) => {
                    Self::render_error(f, err);
                }
                AppState::Shutting { message } => {
                    Self::render_shutting(f, message);
                }
            }
        })?;
        Ok(())
    }

    /// Render initializing state
    fn render_initializing(f: &mut Frame, message: &str) {
        let area = f.size();
        let block = Block::default()
            .title("Claude Code Orchestra")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let text = vec![Line::from(Span::styled(
            message,
            Style::default().fg(Color::Yellow),
        ))];

        let para = Paragraph::new(text).alignment(Alignment::Center);
        f.render_widget(para, inner);
    }

    /// Render daemon starting state
    fn render_starting(f: &mut Frame, progress: u8) {
        let area = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        let block = Block::default()
            .title("Starting Daemon")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        f.render_widget(block, area);

        let progress_text = format!("{}%", progress);
        let progress_bar = "█".repeat((progress as usize * 50) / 100);

        let text = vec![
            Line::from(Span::raw(&progress_text)),
            Line::from(Span::styled(
                progress_bar,
                Style::default().fg(Color::Green),
            )),
        ];

        let para = Paragraph::new(text).alignment(Alignment::Center);
        f.render_widget(para, chunks[1]);
    }

    /// Render connected state (main dashboard with cost focus)
    fn render_connected(
        f: &mut Frame,
        cost_by_tier: &CostByTier,
        recent_calls: &[RecentCall],
        health: &HealthResponse,
        _is_active: bool,
        overall_summary: &OverallSummary,
        project_summaries: &[ProjectSummary],
        model_summaries: &[ModelSummary],
        status_message: &str,
        last_updated: SystemTime,
        time_range: TimeRange,
        history_start_date: Option<SystemTime>,
    ) {
        let area = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(3),  // Header
                    Constraint::Min(10),    // Main content
                    Constraint::Length(3),  // Footer
                ]
                .as_ref(),
            )
            .split(area);

        // Header with Status (server info, port, uptime, time range)
        Self::render_header(f, health, chunks[0], time_range, history_start_date);

        // Main content area layout with Overall Summary and expanded Project Summaries
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Overall Summary
                Constraint::Length(3 + (project_summaries.len() as u16).min(10)), // Project Summaries (expanded to show up to 10 projects)
                Constraint::Length(11),  // Cost summary table
                Constraint::Min(2),      // Recent calls list (dynamic height)
            ].as_ref())
            .split(chunks[1]);

        // Overall Summary (Section 1)
        Self::render_overall_summary(f, overall_summary, content_chunks[0]);

        // Project Summaries (Section 2) - now with more space
        if !project_summaries.is_empty() {
            Self::render_project_summaries(f, project_summaries, content_chunks[1]);
        }

        // Cost summary by tier (Section 3)
        Self::render_cost_summary(f, cost_by_tier, model_summaries, content_chunks[2]);

        // Recent API calls with dynamic height (Section 4)
        Self::render_recent_calls_dynamic(f, recent_calls, content_chunks[3]);

        // Footer
        Self::render_footer(f, chunks[2], status_message, last_updated, time_range);
    }

    /// Render header with title and status
    fn render_header(f: &mut Frame, health: &HealthResponse, area: Rect, time_range: TimeRange, history_start_date: Option<SystemTime>) {
        let uptime = health.uptime_seconds;
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        let seconds = uptime % 60;

        // Section 1: Status (server info, port, uptime)
        let port_str = if health.port == 0 {
            // Port not set in health response - should not happen in production
            // but provide fallback for resilience
            "unknown".to_string()
        } else {
            health.port.to_string()
        };

        // Format history start date
        let history_str = if let Some(start_time) = history_start_date {
            if let Ok(duration) = start_time.elapsed() {
                let days = duration.as_secs() / 86400;
                if days > 0 {
                    format!(" | History: {} days", days)
                } else {
                    " | History: <1 day".to_string()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let header_str = format!(
            "v{} | Port: {} | Uptime: {:02}:{:02}:{:02} | Filter: {}{}",
            health.version, port_str, hours, minutes, seconds, time_range.display_name(), history_str
        );

        let header_text = vec![Line::from(vec![
            Span::styled(
                "Claude Code Orchestra ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(header_str),
        ])];

        let header = Paragraph::new(header_text).block(
            Block::default()
                .title("Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(header, area);
    }

    /// Render overall summary from metrics
    fn render_overall_summary(f: &mut Frame, summary: &OverallSummary, area: Rect) {
        let total_tokens_formatted = if summary.total_tokens >= 1_000_000 {
            format!("{:.2}M", summary.total_tokens as f64 / 1_000_000.0)
        } else if summary.total_tokens >= 1_000 {
            format!("{:.1}K", summary.total_tokens as f64 / 1_000.0)
        } else {
            format!("{}", summary.total_tokens)
        };

        // Calculate percentages from the cost values
        let opus_pct = if summary.total_cost > 0.0 {
            (summary.opus_cost / summary.total_cost) * 100.0
        } else {
            0.0
        };

        let sonnet_pct = if summary.total_cost > 0.0 {
            (summary.sonnet_cost / summary.total_cost) * 100.0
        } else {
            0.0
        };

        let haiku_pct = if summary.total_cost > 0.0 {
            (summary.haiku_cost / summary.total_cost) * 100.0
        } else {
            0.0
        };

        let summary_line = format!(
            "Cost: ${:.5}  Tokens: {}  Calls: {}  | Opus: {:.0}% | Sonnet: {:.0}% | Haiku: {:.0}%",
            summary.total_cost,
            total_tokens_formatted,
            summary.total_calls,
            opus_pct,
            sonnet_pct,
            haiku_pct
        );

        let text = vec![Line::from(Span::styled(
            summary_line,
            Style::default().fg(Color::Cyan),
        ))];

        let para = Paragraph::new(text).block(
            Block::default()
                .title("Overall Summary")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );

        f.render_widget(para, area);
    }

    /// Render project summaries
    fn render_project_summaries(f: &mut Frame, projects: &[ProjectSummary], area: Rect) {
        // Calculate available width dynamically
        // Formula: total_width - borders(2) - cost_col(14) - tokens_col(11) - calls_col(6) - spacing(4)
        let total_width = area.width as usize;
        let fixed_width = 2 + 14 + 11 + 6 + 4; // borders + cost + tokens + calls + spacing
        let available_name_width = if total_width > fixed_width {
            total_width - fixed_width
        } else {
            26 // fallback to old minimum
        };

        let mut text = vec![Line::from(vec![
            Span::styled(
                format!("{:<width$}", "Project Name", width = available_name_width),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            ),
            Span::styled("Cost         ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Tokens    ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Calls", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ])];

        // Expanded to show up to 10 projects (was 5)
        for project in projects.iter().take(10) {
            let name = if project.name.len() > available_name_width {
                format!("{}...", &project.name[..(available_name_width.saturating_sub(3))])
            } else {
                format!("{:<width$}", project.name, width = available_name_width)
            };

            let tokens_formatted = if project.tokens >= 1_000_000 {
                format!("{:.2}M", project.tokens as f64 / 1_000_000.0)
            } else if project.tokens >= 1_000 {
                format!("{:.1}K", project.tokens as f64 / 1_000.0)
            } else {
                format!("{}", project.tokens)
            };

            let line = format!(
                "{} ${:>10.5}  {:>8}  {}",
                name, project.cost, tokens_formatted, project.calls
            );

            text.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::Yellow),
            )));
        }

        let para = Paragraph::new(text).block(
            Block::default()
                .title(format!("Cost Summary by Project ({} total)", projects.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

        f.render_widget(para, area);
    }

    /// Render cost summary by tier/model
    fn render_cost_summary(f: &mut Frame, _cost: &CostByTier, models: &[ModelSummary], area: Rect) {
        let mut text = vec![Line::from(vec![
            Span::styled("Model                     ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Cost       ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("%     ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Tokens    ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Calls", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ])];

        // Calculate totals
        let total_cost: f64 = models.iter().map(|m| m.cost).sum();
        let total_tokens: u64 = models.iter().map(|m| m.tokens).sum();
        let total_messages: u64 = models.iter().map(|m| m.messages).sum();

        // Sort models by cost descending for consistent display order
        let mut sorted_models = models.to_vec();
        sorted_models.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap_or(std::cmp::Ordering::Equal));

        // Display all models
        for model in &sorted_models {
            // Determine color based on model name
            let model_color = if model.model.contains("opus") {
                Color::Magenta
            } else if model.model.contains("sonnet") {
                Color::Cyan
            } else if model.model.contains("haiku") {
                Color::Blue
            } else {
                Color::White
            };

            // Truncate model name if too long
            let model_name = if model.model.len() > 25 {
                format!("{}...", &model.model[..22])
            } else {
                format!("{:<25}", model.model)
            };

            text.push(Line::from(vec![
                Span::styled(model_name, Style::default().fg(model_color)),
                Span::styled(
                    format!("${:>8.2} ", model.cost),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("{:>4.1}% ", model.percentage),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("{:>9}  ", Self::format_tokens(model.tokens)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{}", model.messages),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        // Add separator and total
        text.push(Line::from("─".repeat(80)));
        text.push(Line::from(vec![
            Span::styled(
                "TOTAL                    ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("${:>8.2} ", total_cost),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "100.0%",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:>9}  ", Self::format_tokens(total_tokens)),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", total_messages),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        let title = if models.is_empty() {
            "Cost Summary by Model (No data)".to_string()
        } else {
            format!("Cost Summary by Model ({} models)", models.len())
        };

        let para = Paragraph::new(text).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

        f.render_widget(para, area);
    }

    /// Format token count (e.g., 3480000 -> 3.48M)
    fn format_tokens(tokens: u64) -> String {
        if tokens >= 1_000_000 {
            format!("{:.2}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}K", tokens as f64 / 1_000.0)
        } else {
            format!("{}", tokens)
        }
    }

    /// Render recent API calls with dynamic height
    fn render_recent_calls_dynamic(f: &mut Frame, calls: &[RecentCall], area: Rect) {
        // Calculate how many calls can fit in the available area
        // Account for borders (top and bottom = 2 lines) and title (1 line)
        let available_height = area.height.saturating_sub(3) as usize;
        let display_count = if available_height > 0 {
            std::cmp::min(calls.len(), available_height)
        } else {
            0
        };

        let items: Vec<ListItem> = calls
            .iter()
            .take(display_count)
            .map(|call| {
                // Determine color based on model name
                let model_color = if call.model.contains("opus") {
                    Color::Magenta
                } else if call.model.contains("sonnet") {
                    Color::Cyan
                } else if call.model.contains("haiku") {
                    Color::Blue
                } else {
                    Color::White
                };

                // Parse timestamp and calculate relative time
                let relative_time = if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&call.timestamp) {
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(timestamp);

                    if duration.num_seconds() < 60 {
                        format!("{}s ago", duration.num_seconds())
                    } else if duration.num_minutes() < 60 {
                        format!("{}m ago", duration.num_minutes())
                    } else if duration.num_hours() < 24 {
                        format!("{}h ago", duration.num_hours())
                    } else {
                        format!("{}d ago", duration.num_days())
                    }
                } else {
                    "unknown".to_string()
                };

                // Format tokens (e.g., 3480 -> 3.5K)
                let tokens_str = Self::format_tokens(call.tokens);

                // Format: "5m ago  claude-sonnet-4-5  3.5K tokens  $0.0012"
                let content = format!(
                    "{:<8}  {:<22}  {:>6} tokens  ${:>7.4}",
                    relative_time,
                    call.model,
                    tokens_str,
                    call.cost
                );

                ListItem::new(content).style(Style::default().fg(model_color))
            })
            .collect();

        let title = if calls.is_empty() {
            "Recent API Calls (None)".to_string()
        } else {
            format!(
                "Recent API Calls ({} of {})",
                display_count,
                calls.len()
            )
        };

        let list = List::new(items).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(list, area);
    }

    /// Render footer with controls
    fn render_footer(f: &mut Frame, area: Rect, status_message: &str, last_updated: SystemTime, time_range: TimeRange) {
        // Calculate elapsed time since last update
        let elapsed = last_updated
            .elapsed()
            .unwrap_or(Duration::from_secs(0));
        let seconds_ago = elapsed.as_secs();

        let update_text = if seconds_ago < 1 {
            "Updated: just now".to_string()
        } else if seconds_ago < 60 {
            format!("Updated: {}s ago", seconds_ago)
        } else {
            format!("Updated: {}m ago", seconds_ago / 60)
        };

        let footer_text = if !status_message.is_empty() {
            format!("{} | {} | q: Quit | r: Restart | t: Time Range ({})", status_message, update_text, time_range.display_name())
        } else {
            format!("{} | q: Quit | r: Restart | t: Time Range ({})", update_text, time_range.display_name())
        };

        let footer = Paragraph::new(footer_text)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));

        f.render_widget(footer, area);
    }

    /// Render error state
    fn render_error(f: &mut Frame, err: &str) {
        let area = f.size();
        let block = Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let text = vec![
            Line::from(Span::styled(
                err,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::raw("Press 'q' to quit or 'r' to retry")),
        ];

        let para = Paragraph::new(text).alignment(Alignment::Center);
        f.render_widget(para, inner);
    }

    /// Render shutting down state
    fn render_shutting(f: &mut Frame, message: &str) {
        let area = f.size();
        let block = Block::default()
            .title("Shutting Down")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let text = vec![Line::from(Span::styled(
            message,
            Style::default().fg(Color::Yellow),
        ))];

        let para = Paragraph::new(text).alignment(Alignment::Center);
        f.render_widget(para, inner);
    }

    /// Shutdown and cleanup
    async fn shutdown(&mut self) -> Result<()> {
        self.state = AppState::Shutting {
            message: "Cleaning up...".to_string(),
        };
        self.render()?;

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        // Ensure terminal is restored even on panic
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
