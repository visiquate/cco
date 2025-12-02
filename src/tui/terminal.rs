//! Terminal management and rendering for the TUI dashboard

use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::layout::Alignment;
use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io::Stdout;
use std::time::Duration;

use crate::tui::app::{App, AppState};
use crate::tui::components::*;
use crate::tui::event::{is_next_tab_key, is_prev_tab_key, is_quit_key, Event, EventHandler};

/// Terminal wrapper for Ratatui
pub struct Terminal {
    terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl Terminal {
    /// Create a new terminal instance
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Main draw loop for the dashboard
    pub async fn draw_loop(
        mut self,
        mut app: App,
        event_handler: EventHandler,
        refresh_rate_ms: u64,
    ) -> Result<()> {
        let tick_duration = Duration::from_millis(refresh_rate_ms);
        let mut last_update = std::time::Instant::now();

        loop {
            // Update metrics periodically
            if last_update.elapsed() >= tick_duration {
                let _ = app.update_metrics().await;
                last_update = std::time::Instant::now();
            }

            // Draw current frame
            {
                let app_ref = &app;
                self.terminal.draw(|f| {
                    Self::ui_static(f, app_ref);
                })?;
            }

            // Handle events
            if let Ok(Some(event)) = event_handler.next() {
                match event {
                    Event::Key(key) => {
                        if is_quit_key(key) {
                            app.exit();
                        } else if is_next_tab_key(key) {
                            app.next_tab();
                        } else if is_prev_tab_key(key) {
                            app.prev_tab();
                        }
                    }
                    Event::Resize(_, _) => {
                        // Handle window resize
                    }
                    _ => {}
                }
            }

            // Check if we should exit
            if app.should_exit {
                break;
            }

            // Small sleep to prevent busy-waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Cleanup
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(())
    }

    /// Render the UI
    fn ui_static(f: &mut Frame, app: &App) {
        let size = f.size();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(size);

        // Header
        Self::render_header(f, app, chunks[0]);

        // Main content
        Self::render_content(f, app, chunks[1]);

        // Footer
        Self::render_footer(f, app, chunks[2]);
    }

    /// Render header with title and tabs
    fn render_header(f: &mut Frame, app: &App, area: Rect) {
        let tabs = vec!["Overview", "Real-time", "Cost Analysis", "Session Info"];
        let mut header_content = vec![];

        // Title
        header_content.push(Line::from(vec![
            Span::styled(
                "CCO Metrics Dashboard",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Press Tab/← → to navigate, Ctrl+C to exit"),
        ]));

        // Tab bar
        let mut tab_spans = vec![];
        for (idx, tab) in tabs.iter().enumerate() {
            let is_active = idx == app.current_tab.index();
            let style = if is_active {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            if idx > 0 {
                tab_spans.push(Span::raw(" │ "));
            }
            tab_spans.push(Span::styled(format!(" {} ", tab), style));
        }

        header_content.push(Line::from(tab_spans));

        let header = Paragraph::new(header_content)
            .block(Block::default().borders(Borders::BOTTOM))
            .style(Style::default().fg(Color::White));

        f.render_widget(header, area);
    }

    /// Render main content area based on current tab
    fn render_content(f: &mut Frame, app: &App, area: Rect) {
        match app.current_tab {
            AppState::Overview => {
                let component = OverviewComponent;
                component.render(f, app, area);
            }
            AppState::RealTime => {
                let component = RealtimeComponent;
                component.render(f, app, area);
            }
            AppState::CostAnalysis => {
                let component = CostAnalysisComponent;
                component.render(f, app, area);
            }
            AppState::SessionInfo => {
                let component = SessionInfoComponent;
                component.render(f, app, area);
            }
        }
    }

    /// Render footer with status information
    fn render_footer(f: &mut Frame, _app: &App, area: Rect) {
        let footer = Paragraph::new(
            "Updated every 1s | Use Tab or ← → arrows to switch tabs | Ctrl+C to quit",
        )
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

        f.render_widget(footer, area);
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Cleanup terminal on drop
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_terminal_creation() {
        // Terminal creation is tested during TUI runtime
        // This test verifies the module compiles correctly
    }
}
