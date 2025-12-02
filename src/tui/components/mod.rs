//! TUI components for rendering different dashboard sections

pub mod cost_analysis;
pub mod hooks_panel;
pub mod overview;
pub mod realtime;
pub mod session_info;

pub use cost_analysis::CostAnalysisComponent;
pub use hooks_panel::HooksPanel;
pub use overview::OverviewComponent;
pub use realtime::RealtimeComponent;
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
    #[test]
    fn test_components_module_compiles() {
        // Ensure all components are accessible
    }
}
