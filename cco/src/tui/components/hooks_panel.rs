//! Hooks Status Panel for TUI
//!
//! Displays real-time hooks classification decisions with:
//! - Status line with hooks enabled/disabled state
//! - Recent classification decisions (last 5)
//! - Classification statistics (READ/CREATE/UPDATE/DELETE percentages)
//! - Model status and performance metrics

use crate::api_client::ApiClient;
use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::Instant;

/// Classification type for CRUD operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Classification {
    Read,
    Create,
    Update,
    Delete,
    Unknown,
}

impl Classification {
    /// Get color for this classification type
    pub fn color(&self) -> Color {
        match self {
            Classification::Read => Color::Green,
            Classification::Create | Classification::Update | Classification::Delete => {
                Color::Yellow
            }
            Classification::Unknown => Color::Red,
        }
    }

    /// Get display string
    pub fn as_str(&self) -> &str {
        match self {
            Classification::Read => "READ",
            Classification::Create => "CREATE",
            Classification::Update => "UPDATE",
            Classification::Delete => "DELETE",
            Classification::Unknown => "UNKNOWN",
        }
    }
}

/// Decision made for a command classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub command: String,
    pub classification: Classification,
    pub timestamp: DateTime<Utc>,
    pub decision: String, // "APPROVED", "PENDING", "DENIED"
    pub confidence_score: f32,
}

/// Statistics for classification decisions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionStats {
    pub read_count: u64,
    pub create_count: u64,
    pub update_count: u64,
    pub delete_count: u64,
    pub total_requests: u64,
}

impl DecisionStats {
    /// Calculate percentage for READ operations
    pub fn read_pct(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.read_count as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate percentage for CREATE operations
    pub fn create_pct(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.create_count as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate percentage for UPDATE operations
    pub fn update_pct(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.update_count as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate percentage for DELETE operations
    pub fn delete_pct(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.delete_count as f64 / self.total_requests as f64) * 100.0
    }
}

/// Response from /api/hooks/decisions endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionsResponse {
    pub recent: Vec<Decision>,
    pub stats: DecisionStats,
    pub enabled: bool,
    pub model_loaded: bool,
    pub model_name: String,
    pub last_classification_ms: Option<u32>,
}

/// Hooks panel state
pub enum HooksPanelState {
    /// Hooks system is disabled
    Disabled,
    /// Hooks enabled but model not loaded
    Loading,
    /// API unavailable or error occurred
    Unavailable(String),
    /// No classifications yet
    NoData,
    /// Active with data
    Active(DecisionsResponse),
}

/// Hooks status panel widget
pub struct HooksPanel {
    api_client: ApiClient,
    state: HooksPanelState,
    last_update: Option<Instant>,
    update_interval: Duration,
}

impl HooksPanel {
    /// Create new hooks panel
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            api_client,
            state: HooksPanelState::NoData,
            last_update: None,
            update_interval: Duration::from_secs(5),
        }
    }

    /// Update panel data from API
    pub async fn update(&mut self) {
        // Check if we need to update (throttle to avoid spamming API)
        if let Some(last) = self.last_update {
            if last.elapsed() < self.update_interval {
                return;
            }
        }

        // Fetch decisions from API
        let url = format!("{}/api/hooks/decisions", self.api_client.base_url);
        match self.api_client.get_with_retry::<DecisionsResponse>(&url).await {
            Ok(response) => {
                if !response.enabled {
                    self.state = HooksPanelState::Disabled;
                } else if !response.model_loaded {
                    self.state = HooksPanelState::Loading;
                } else if response.recent.is_empty() && response.stats.total_requests == 0 {
                    self.state = HooksPanelState::NoData;
                } else {
                    self.state = HooksPanelState::Active(response);
                }
                self.last_update = Some(Instant::now());
            }
            Err(e) => {
                self.state = HooksPanelState::Unavailable(e.to_string());
                self.last_update = Some(Instant::now());
            }
        }
    }

    /// Render the hooks panel
    pub fn render(&self, f: &mut Frame, area: Rect) {
        match &self.state {
            HooksPanelState::Disabled => self.render_disabled(f, area),
            HooksPanelState::Loading => self.render_loading(f, area),
            HooksPanelState::Unavailable(err) => self.render_unavailable(f, area, err),
            HooksPanelState::NoData => self.render_no_data(f, area),
            HooksPanelState::Active(data) => self.render_active(f, area, data),
        }
    }

    /// Render disabled state
    fn render_disabled(&self, f: &mut Frame, area: Rect) {
        let text = vec![Line::from(Span::styled(
            "Hooks: DISABLED",
            Style::default().fg(Color::DarkGray),
        ))];

        let para = Paragraph::new(text).block(
            Block::default()
                .title("Hooks Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(para, area);
    }

    /// Render loading state
    fn render_loading(&self, f: &mut Frame, area: Rect) {
        let text = vec![Line::from(Span::styled(
            "Hooks: ENABLED | Model: LOADING...",
            Style::default().fg(Color::Yellow),
        ))];

        let para = Paragraph::new(text).block(
            Block::default()
                .title("Hooks Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(para, area);
    }

    /// Render unavailable state
    fn render_unavailable(&self, f: &mut Frame, area: Rect, _err: &str) {
        let text = vec![Line::from(Span::styled(
            "Hooks: UNAVAILABLE",
            Style::default().fg(Color::Red),
        ))];

        let para = Paragraph::new(text).block(
            Block::default()
                .title("Hooks Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

        f.render_widget(para, area);
    }

    /// Render no data state
    fn render_no_data(&self, f: &mut Frame, area: Rect) {
        let text = vec![Line::from(Span::styled(
            "Hooks: ENABLED | Model: loaded | No classifications yet",
            Style::default().fg(Color::Cyan),
        ))];

        let para = Paragraph::new(text).block(
            Block::default()
                .title("Hooks Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(para, area);
    }

    /// Render active state with data
    fn render_active(&self, f: &mut Frame, area: Rect, data: &DecisionsResponse) {
        // Split area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Status line
                Constraint::Length(7),  // Recent decisions (5 + 2 for borders)
                Constraint::Length(3),  // Statistics
                Constraint::Min(0),     // Spacer
            ])
            .split(area);

        // Render status line
        self.render_status_line(f, chunks[0], data);

        // Render recent decisions
        self.render_recent_decisions(f, chunks[1], data);

        // Render statistics
        self.render_statistics(f, chunks[2], data);
    }

    /// Render status line
    fn render_status_line(&self, f: &mut Frame, area: Rect, data: &DecisionsResponse) {
        let latency_str = if let Some(ms) = data.last_classification_ms {
            format!("{}ms", ms)
        } else {
            "N/A".to_string()
        };

        let status_line = format!(
            "Hooks: ENABLED | Model: {} | Last check: {}",
            data.model_name, latency_str
        );

        let text = vec![Line::from(Span::styled(
            status_line,
            Style::default().fg(Color::Green),
        ))];

        let para = Paragraph::new(text).block(
            Block::default()
                .title("Hooks Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

        f.render_widget(para, area);
    }

    /// Render recent decisions
    fn render_recent_decisions(&self, f: &mut Frame, area: Rect, data: &DecisionsResponse) {
        let mut lines = vec![];

        // Take last 5 decisions
        for decision in data.recent.iter().take(5) {
            // Truncate command to 50 chars
            let command = if decision.command.len() > 50 {
                format!("{}...", &decision.command[..47])
            } else {
                decision.command.clone()
            };

            // Format relative time
            let elapsed = Utc::now().signed_duration_since(decision.timestamp);
            let time_str = if elapsed.num_seconds() < 60 {
                format!("{}s ago", elapsed.num_seconds())
            } else if elapsed.num_minutes() < 60 {
                format!("{}m ago", elapsed.num_minutes())
            } else {
                format!("{}h ago", elapsed.num_hours())
            };

            // Create line with colored classification
            let line = Line::from(vec![
                Span::raw(format!("{:<50} ", command)),
                Span::styled(
                    format!("{:<8} ", decision.classification.as_str()),
                    Style::default()
                        .fg(decision.classification.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<10} ", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    &decision.decision,
                    Style::default().fg(if decision.decision == "APPROVED" {
                        Color::Green
                    } else if decision.decision == "DENIED" {
                        Color::Red
                    } else {
                        Color::Yellow
                    }),
                ),
            ]);

            lines.push(line);
        }

        // Fill remaining lines if fewer than 5 decisions
        while lines.len() < 5 {
            lines.push(Line::from(Span::styled(
                "---",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let para = Paragraph::new(lines).block(
            Block::default()
                .title("Recent Classifications")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(para, area);
    }

    /// Render statistics
    fn render_statistics(&self, f: &mut Frame, area: Rect, data: &DecisionsResponse) {
        let stats_line = format!(
            "READ: {:.0}% | CREATE: {:.0}% | UPDATE: {:.0}% | DELETE: {:.0}% | Total: {}",
            data.stats.read_pct(),
            data.stats.create_pct(),
            data.stats.update_pct(),
            data.stats.delete_pct(),
            data.stats.total_requests
        );

        let text = vec![Line::from(vec![
            Span::styled(
                format!("READ: {:.0}%", data.stats.read_pct()),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("CREATE: {:.0}%", data.stats.create_pct()),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("UPDATE: {:.0}%", data.stats.update_pct()),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("DELETE: {:.0}%", data.stats.delete_pct()),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("Total: {}", data.stats.total_requests),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ])];

        let para = Paragraph::new(text).block(
            Block::default()
                .title("Classification Statistics")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );

        f.render_widget(para, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_colors() {
        assert_eq!(Classification::Read.color(), Color::Green);
        assert_eq!(Classification::Create.color(), Color::Yellow);
        assert_eq!(Classification::Update.color(), Color::Yellow);
        assert_eq!(Classification::Delete.color(), Color::Yellow);
        assert_eq!(Classification::Unknown.color(), Color::Red);
    }

    #[test]
    fn test_decision_stats_percentages() {
        let stats = DecisionStats {
            read_count: 60,
            create_count: 25,
            update_count: 10,
            delete_count: 5,
            total_requests: 100,
        };

        assert_eq!(stats.read_pct(), 60.0);
        assert_eq!(stats.create_pct(), 25.0);
        assert_eq!(stats.update_pct(), 10.0);
        assert_eq!(stats.delete_pct(), 5.0);
    }

    #[test]
    fn test_decision_stats_zero_requests() {
        let stats = DecisionStats::default();

        assert_eq!(stats.read_pct(), 0.0);
        assert_eq!(stats.create_pct(), 0.0);
        assert_eq!(stats.update_pct(), 0.0);
        assert_eq!(stats.delete_pct(), 0.0);
    }
}
