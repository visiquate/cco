//! Terminal User Interface (TUI) for CCO metrics dashboard
//!
//! Provides real-time metrics visualization using Ratatui with support for:
//! - Live metrics display with 1-second updates
//! - Multi-tab navigation (Overview, Real-time, Cost Analysis, Session Info)
//! - Cost breakdown by model tier
//! - Cache hit rate visualization
//! - Interactive controls (arrow keys, vim keys, Ctrl+C to exit)

pub mod app;
pub mod components;
pub mod event;
pub mod terminal;

pub use app::{App, AppState};
pub use event::{Event, EventHandler};
pub use terminal::Terminal;

use crate::metrics::MetricsEngine;
use crate::persistence::PersistenceLayer;
use anyhow::Result;
use std::path::Path;

/// Run the TUI dashboard
pub async fn run_dashboard(db_path: impl AsRef<Path>, refresh_rate_ms: u64) -> Result<()> {
    // Initialize persistence layer and metrics engine
    let persistence = PersistenceLayer::new(db_path).await?;
    let metrics_engine = MetricsEngine::new();

    // Create app state
    let app = App::new(persistence, metrics_engine);

    // Create terminal and event handler
    let terminal = Terminal::new()?;
    let event_handler = EventHandler::new(250); // 250ms event polling

    // Run the dashboard
    terminal
        .draw_loop(app, event_handler, refresh_rate_ms)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_module_imports() {
        // Ensure all modules compile correctly
        let _app_state = AppState::Overview;
    }
}
