//! TUI components for rendering different dashboard sections

pub mod overview;
pub mod realtime;
pub mod cost_analysis;
pub mod session_info;

pub use overview::OverviewComponent;
pub use realtime::RealtimeComponent;
pub use cost_analysis::CostAnalysisComponent;
pub use session_info::SessionInfoComponent;

use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Trait for UI components
pub trait Component {
    /// Render the component
    fn render(&self, f: &mut Frame, app: &App, area: Rect);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_components_module_compiles() {
        // Ensure all components are accessible
    }
}
