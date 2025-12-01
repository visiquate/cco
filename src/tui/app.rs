//! Application state and logic for the TUI dashboard

use crate::metrics::{MetricsEngine, MetricsSummary};
use crate::persistence::{PersistenceLayer, PersistenceResult};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// Application state for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Overview tab - summary metrics
    Overview,
    /// Real-time metrics tab - live API calls
    RealTime,
    /// Cost analysis tab - breakdown by model tier
    CostAnalysis,
    /// Session information tab - uptime, metrics count
    SessionInfo,
}

impl AppState {
    /// Get the index of this state (for tab navigation)
    pub fn index(self) -> usize {
        match self {
            AppState::Overview => 0,
            AppState::RealTime => 1,
            AppState::CostAnalysis => 2,
            AppState::SessionInfo => 3,
        }
    }

    /// Get the next state (cycle through tabs)
    pub fn next(self) -> Self {
        match self {
            AppState::Overview => AppState::RealTime,
            AppState::RealTime => AppState::CostAnalysis,
            AppState::CostAnalysis => AppState::SessionInfo,
            AppState::SessionInfo => AppState::Overview,
        }
    }

    /// Get the previous state (cycle through tabs)
    pub fn prev(self) -> Self {
        match self {
            AppState::Overview => AppState::SessionInfo,
            AppState::RealTime => AppState::Overview,
            AppState::CostAnalysis => AppState::RealTime,
            AppState::SessionInfo => AppState::CostAnalysis,
        }
    }
}

/// Display data for recent API call
#[derive(Debug, Clone)]
pub struct ApiCallDisplay {
    pub model_name: String,
    pub timestamp: DateTime<Utc>,
    pub tokens: u64,
    pub cost_usd: f64,
}

/// Application state container
pub struct App {
    /// Current active tab
    pub current_tab: AppState,

    /// Persistence layer for database access
    pub persistence: PersistenceLayer,

    /// Metrics engine for real-time aggregation
    pub metrics_engine: MetricsEngine,

    /// Current metrics summary
    pub summary: MetricsSummary,

    /// Recent API calls for display
    pub recent_calls: VecDeque<ApiCallDisplay>,

    /// Session start time
    pub session_start: DateTime<Utc>,

    /// Last update time
    pub last_update: DateTime<Utc>,

    /// Total metrics recorded this session
    pub metrics_count: u64,

    /// Should exit the application
    pub should_exit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(persistence: PersistenceLayer, metrics_engine: MetricsEngine) -> Self {
        let now = Utc::now();

        Self {
            current_tab: AppState::Overview,
            persistence,
            metrics_engine,
            summary: MetricsSummary {
                total_cost_usd: 0.0,
                call_count: 0,
                tokens_by_type: Default::default(),
                by_model_tier: Default::default(),
                total_tokens: 0,
            },
            recent_calls: VecDeque::with_capacity(10),
            session_start: now,
            last_update: now,
            metrics_count: 0,
            should_exit: false,
        }
    }

    /// Switch to the next tab
    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    /// Switch to the previous tab
    pub fn prev_tab(&mut self) {
        self.current_tab = self.current_tab.prev();
    }

    /// Update metrics from the engine and persistence layer
    pub async fn update_metrics(&mut self) -> PersistenceResult<()> {
        // Get summary from metrics engine
        self.summary = self.metrics_engine.get_summary().await;

        // Get recent calls from metrics engine
        let recent = self.metrics_engine.get_recent_calls(10).await;
        self.recent_calls.clear();
        for event in recent.iter().rev() {
            self.recent_calls.push_back(ApiCallDisplay {
                model_name: event.model_name.clone(),
                timestamp: event.timestamp,
                tokens: event.tokens.total_tokens(),
                cost_usd: event.cost_usd,
            });
        }

        // Update metadata
        self.last_update = Utc::now();
        self.metrics_count = self.summary.call_count;

        Ok(())
    }

    /// Mark application for exit
    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    /// Get session uptime in seconds
    pub fn uptime_seconds(&self) -> i64 {
        (Utc::now() - self.session_start).num_seconds()
    }

    /// Get uptime formatted as HH:MM:SS
    pub fn uptime_formatted(&self) -> String {
        let seconds = self.uptime_seconds();
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }

    /// Get cache hit rate as a percentage
    pub fn cache_hit_rate(&self) -> f64 {
        if self.summary.call_count == 0 {
            return 0.0;
        }

        let cache_hits = self
            .summary
            .by_model_tier
            .values()
            .map(|tier| tier.cache_read_tokens)
            .sum::<u64>();

        if cache_hits == 0 {
            return 0.0;
        }

        (cache_hits as f64 / self.summary.total_tokens as f64) * 100.0
    }

    /// Get average cost per call
    pub fn avg_cost_per_call(&self) -> f64 {
        if self.summary.call_count == 0 {
            return 0.0;
        }
        self.summary.total_cost_usd / self.summary.call_count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_navigation() {
        let mut state = AppState::Overview;
        assert_eq!(state.index(), 0);

        state = state.next();
        assert_eq!(state, AppState::RealTime);
        assert_eq!(state.index(), 1);

        state = state.next();
        assert_eq!(state, AppState::CostAnalysis);

        state = state.prev();
        assert_eq!(state, AppState::RealTime);
    }

    #[test]
    fn test_app_state_cycle() {
        let mut state = AppState::SessionInfo;
        state = state.next();
        assert_eq!(state, AppState::Overview); // Cycles back

        state = AppState::Overview;
        state = state.prev();
        assert_eq!(state, AppState::SessionInfo); // Cycles back
    }
}
