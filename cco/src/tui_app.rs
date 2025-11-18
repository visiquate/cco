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
use std::time::Duration;
use tokio::time::sleep;

use crate::api_client::{ApiClient, HealthResponse};
use crate::daemon::{DaemonConfig, DaemonManager};
use crate::analytics::ActivityEvent;

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

/// Recent API call information
#[derive(Debug, Clone)]
pub struct RecentCall {
    pub tier: String,
    pub cost: f64,
    pub file: String,
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
    },
    /// Error state with message
    Error(String),
    /// Shutting down
    Shutting {
        message: String,
    },
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

        // Create API client
        let base_url = format!("http://{}:{}", config.host, config.port);
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

        // 3. Enter main event loop
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

            // Update from daemon periodically
            self.update_state().await?;

            // Check if we should quit
            if self.should_quit {
                break;
            }
        }

        // 4. Cleanup on exit
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
        let stats_url = format!("{}/api/stats", self.client.base_url);
        let stats_response: Result<serde_json::Value, _> = self.client.get_with_retry(&stats_url).await;

        match (health, stats_response) {
            (Ok(health), Ok(stats)) => {
                // Parse cost by tier from stats response
                let cost_by_tier = self.parse_cost_by_tier(&stats);

                // Parse recent calls from activity events
                let recent_calls = self.parse_recent_calls(&stats);

                // Determine if system is active (has recent activity)
                let is_active = !recent_calls.is_empty();

                self.state = AppState::Connected {
                    cost_by_tier,
                    recent_calls,
                    health,
                    is_active,
                };
            }
            (Err(e), _) | (_, Err(e)) => {
                self.state = AppState::Error(format!("Failed to load data: {}", e));
            }
        }
        Ok(())
    }

    /// Parse cost breakdown by tier from stats JSON
    fn parse_cost_by_tier(&self, stats: &serde_json::Value) -> CostByTier {
        let mut cost_by_tier = CostByTier::default();

        // Try to extract model_distribution from chart_data
        if let Some(chart_data) = stats.get("chart_data") {
            if let Some(model_distribution) = chart_data.get("model_distribution").and_then(|d| d.as_array()) {
                let mut sonnet_cost = 0.0;
                let mut opus_cost = 0.0;
                let mut haiku_cost = 0.0;
                let mut total_cost = 0.0;

                // Get total cost from stats
                if let Some(total) = stats.get("project").and_then(|p| p.get("cost")).and_then(|c| c.as_f64()) {
                    total_cost = total;
                }

                // Sum costs per model name to calculate totals
                for model_item in model_distribution {
                    if let Some(model_name) = model_item.get("model").and_then(|m| m.as_str()) {
                        // Estimate cost based on percentage if available
                        if let Some(percentage) = model_item.get("percentage").and_then(|p| p.as_f64()) {
                            let cost = (total_cost * percentage) / 100.0;

                            if model_name.to_lowercase().contains("sonnet") {
                                sonnet_cost += cost;
                            } else if model_name.to_lowercase().contains("opus") {
                                opus_cost += cost;
                            } else if model_name.to_lowercase().contains("haiku") {
                                haiku_cost += cost;
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

                cost_by_tier = CostByTier {
                    sonnet_cost,
                    sonnet_pct,
                    sonnet_calls: 0, // TODO: extract from metrics_by_model
                    sonnet_tokens: TokenStats::default(),
                    opus_cost,
                    opus_pct,
                    opus_calls: 0,
                    opus_tokens: TokenStats::default(),
                    haiku_cost,
                    haiku_pct,
                    haiku_calls: 0,
                    haiku_tokens: TokenStats::default(),
                    total_cost: total_calculated,
                    total_calls: 0,
                    total_tokens: TokenStats::default(),
                };
            }
        }

        cost_by_tier
    }

    /// Parse recent API calls from activity events
    fn parse_recent_calls(&self, stats: &serde_json::Value) -> Vec<RecentCall> {
        let mut calls = Vec::new();

        if let Some(activity) = stats.get("activity").and_then(|a| a.as_array()) {
            for event in activity.iter().take(20) {
                if let (Some(model), Some(cost)) = (
                    event.get("model").and_then(|m| m.as_str()),
                    event.get("cost").and_then(|c| c.as_f64()),
                ) {
                    let tier = if model.contains("opus") {
                        "Opus"
                    } else if model.contains("sonnet") {
                        "Sonnet"
                    } else if model.contains("haiku") {
                        "Haiku"
                    } else {
                        "Unknown"
                    };

                    let file = event.get("file_source")
                        .and_then(|f| f.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    calls.push(RecentCall {
                        tier: tier.to_string(),
                        cost,
                        file,
                    });
                }
            }
        }

        calls
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
            _ => {}
        }
        Ok(false)
    }

    /// Render the current UI frame
    fn render(&mut self) -> Result<()> {
        let state = self.state.clone();
        let status_message = self.status_message.clone();

        self.terminal.draw(|f| {
            Self::ui_static(f, &state, &status_message);
        })?;
        Ok(())
    }

    /// Build the UI layout (static method to avoid borrowing issues)
    fn ui_static(f: &mut Frame, state: &AppState, status_message: &str) {
        match state {
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
            } => {
                Self::render_connected(f, cost_by_tier, recent_calls, health, *is_active, status_message);
            }
            AppState::Error(err) => {
                Self::render_error(f, err);
            }
            AppState::Shutting { message } => {
                Self::render_shutting(f, message);
            }
        }
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
        is_active: bool,
        status_message: &str,
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

        // Header
        Self::render_header(f, health, chunks[0]);

        // Main content area - single panel with cost summary and recent calls
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),  // Cost summary table
                Constraint::Length(3),  // Active/Idle status
                Constraint::Min(5),     // Recent calls list
            ].as_ref())
            .split(chunks[1]);

        // Cost summary by tier
        Self::render_cost_summary(f, cost_by_tier, content_chunks[0]);

        // Active/Idle indicator
        Self::render_status_indicator(f, is_active, content_chunks[1]);

        // Recent API calls
        Self::render_recent_calls(f, recent_calls, content_chunks[2]);

        // Footer
        Self::render_footer(f, chunks[2], status_message);
    }

    /// Render header with title and status
    fn render_header(f: &mut Frame, health: &HealthResponse, area: Rect) {
        let uptime = health.uptime_seconds;
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        let seconds = uptime % 60;

        let header_str = format!(
            "v{} | Port: {} | Uptime: {:02}:{:02}:{:02}",
            health.version, health.port, hours, minutes, seconds
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
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Gray)),
        );

        f.render_widget(header, area);
    }

    /// Render cost summary by tier
    fn render_cost_summary(f: &mut Frame, cost: &CostByTier, area: Rect) {
        let text = vec![
            Line::from(vec![
                Span::styled("Tier      ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("Cost       ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("%     ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("Calls  ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("Tokens (I/O/CW/CR)", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Sonnet    ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("${:>8.2} ", cost.sonnet_cost), Style::default().fg(Color::Green)),
                Span::styled(format!("{:>4.1}% ", cost.sonnet_pct), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:>6}  ", cost.sonnet_calls), Style::default().fg(Color::White)),
                Span::styled(format!("I:{} O:{} CW:{}",
                    Self::format_tokens(cost.sonnet_tokens.input),
                    Self::format_tokens(cost.sonnet_tokens.output),
                    Self::format_tokens(cost.sonnet_tokens.cache_write)
                ), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("           "),
                Span::raw("      "),
                Span::raw("        "),
                Span::styled(format!("CR:{}", Self::format_tokens(cost.sonnet_tokens.cache_read)), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled("Opus      ", Style::default().fg(Color::Magenta)),
                Span::styled(format!("${:>8.2} ", cost.opus_cost), Style::default().fg(Color::Green)),
                Span::styled(format!("{:>4.1}% ", cost.opus_pct), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:>6}  ", cost.opus_calls), Style::default().fg(Color::White)),
                Span::styled(format!("I:{} O:{} CW:{}",
                    Self::format_tokens(cost.opus_tokens.input),
                    Self::format_tokens(cost.opus_tokens.output),
                    Self::format_tokens(cost.opus_tokens.cache_write)
                ), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("           "),
                Span::raw("      "),
                Span::raw("        "),
                Span::styled(format!("CR:{}", Self::format_tokens(cost.opus_tokens.cache_read)), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled("Haiku     ", Style::default().fg(Color::Blue)),
                Span::styled(format!("${:>8.2} ", cost.haiku_cost), Style::default().fg(Color::Green)),
                Span::styled(format!("{:>4.1}% ", cost.haiku_pct), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:>6}  ", cost.haiku_calls), Style::default().fg(Color::White)),
                Span::styled(format!("I:{} O:{} CW:{}",
                    Self::format_tokens(cost.haiku_tokens.input),
                    Self::format_tokens(cost.haiku_tokens.output),
                    Self::format_tokens(cost.haiku_tokens.cache_write)
                ), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("           "),
                Span::raw("      "),
                Span::raw("        "),
                Span::styled(format!("CR:{}", Self::format_tokens(cost.haiku_tokens.cache_read)), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from("─".repeat(80)),
            Line::from(vec![
                Span::styled("TOTAL     ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:>8.2} ", cost.total_cost), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("100.0%", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>6}  ", cost.total_calls), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!("I:{} O:{} CW:{} CR:{}",
                    Self::format_tokens(cost.total_tokens.input),
                    Self::format_tokens(cost.total_tokens.output),
                    Self::format_tokens(cost.total_tokens.cache_write),
                    Self::format_tokens(cost.total_tokens.cache_read)
                ), Style::default().fg(Color::DarkGray)),
            ]),
        ];

        let para = Paragraph::new(text).block(
            Block::default()
                .title("Cost Summary by Tier")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
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

    /// Render active/idle status indicator
    fn render_status_indicator(f: &mut Frame, is_active: bool, area: Rect) {
        let (indicator, color) = if is_active {
            ("🟢 ACTIVE", Color::Green)
        } else {
            ("🔴 IDLE", Color::Red)
        };

        let text = vec![Line::from(Span::styled(
            format!("Status: {}", indicator),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))];

        let para = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        );

        f.render_widget(para, area);
    }

    /// Render recent API calls
    fn render_recent_calls(f: &mut Frame, calls: &[RecentCall], area: Rect) {
        let items: Vec<ListItem> = calls
            .iter()
            .map(|call| {
                let tier_color = match call.tier.as_str() {
                    "Opus" => Color::Magenta,
                    "Sonnet" => Color::Cyan,
                    "Haiku" => Color::Blue,
                    _ => Color::White,
                };

                let content = format!(
                    "{:<8} ${:>7.4}  {}",
                    call.tier,
                    call.cost,
                    call.file
                );

                ListItem::new(content).style(Style::default().fg(tier_color))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title("Recent API Calls (Last 20)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(list, area);
    }

    /// Render footer with controls
    fn render_footer(f: &mut Frame, area: Rect, status_message: &str) {
        let footer_text = if !status_message.is_empty() {
            format!("{} | q: Quit | r: Restart", status_message)
        } else {
            "q: Quit | r: Restart".to_string()
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

    /// Update state from daemon (periodic refresh)
    async fn update_state(&mut self) -> Result<()> {
        // Only update if connected
        if let AppState::Connected { .. } = self.state {
            // Refresh data every few cycles
            // For now, we'll do this on demand rather than every cycle
            // to avoid overwhelming the daemon
        }
        Ok(())
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
