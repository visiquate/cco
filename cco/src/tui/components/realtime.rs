//! Real-time metrics component - live API call display

use super::Component;
use crate::tui::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub struct RealtimeComponent;

impl Component for RealtimeComponent {
    fn render(&self, f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        // Header with current timestamp
        let now = chrono::Utc::now();
        let header = Paragraph::new(format!(
            "Last Updated: {} UTC | Total Calls: {}",
            now.format("%H:%M:%S"),
            app.summary.call_count
        ))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
        f.render_widget(header, chunks[0]);

        // List of recent API calls
        let items: Vec<ListItem> = app
            .recent_calls
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, call)| {
                let color = match call.model_name.to_lowercase() {
                    _ if call.model_name.contains("opus") => Color::Magenta,
                    _ if call.model_name.contains("sonnet") => Color::Blue,
                    _ if call.model_name.contains("haiku") => Color::Cyan,
                    _ => Color::White,
                };

                let content = format!(
                    "[{}] {} | {} tokens | ${:.4}",
                    idx + 1,
                    call.model_name,
                    call.tokens,
                    call.cost_usd
                );

                ListItem::new(content).style(Style::default().fg(color))
            })
            .collect();

        let list = if items.is_empty() {
            List::new([ListItem::new("No API calls recorded yet...")].to_vec())
                .block(
                    Block::default()
                        .title("Recent API Calls")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::DarkGray))
        } else {
            List::new(items)
                .block(
                    Block::default()
                        .title("Recent API Calls")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::White))
        };

        f.render_widget(list, chunks[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_component_creation() {
        let _component = RealtimeComponent;
        // Component is created successfully
    }
}
