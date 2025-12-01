//! Session information component - uptime and metrics tracking

use super::Component;
use crate::tui::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct SessionInfoComponent;

impl Component for SessionInfoComponent {
    fn render(&self, f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(area);

        // Session Start Time
        let start_time = app.session_start.format("%Y-%m-%d %H:%M:%S UTC");
        let start_widget = Paragraph::new(format!("{}", start_time))
            .block(
                Block::default()
                    .title("Session Start")
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(start_widget, chunks[0]);

        // Uptime
        let uptime = app.uptime_formatted();
        let uptime_widget = Paragraph::new(uptime)
            .block(Block::default().title("Uptime").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(uptime_widget, chunks[1]);

        // Metrics Recorded
        let metrics_text = format!("{}", app.metrics_count);
        let metrics_widget = Paragraph::new(metrics_text)
            .block(
                Block::default()
                    .title("Metrics Recorded")
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(metrics_widget, chunks[2]);

        // Metrics Per Minute
        let uptime_secs = app.uptime_seconds();
        let metrics_per_min = if uptime_secs > 0 {
            (app.metrics_count as f64 * 60.0) / uptime_secs as f64
        } else {
            0.0
        };
        let rate_text = format!("{:.2}", metrics_per_min);
        let rate_widget = Paragraph::new(rate_text)
            .block(Block::default().title("Metrics/Min").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Magenta));
        f.render_widget(rate_widget, chunks[3]);

        // Session Summary
        if !chunks[4].is_empty() {
            let mut lines = vec![Line::from(Span::styled(
                "Session Summary",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))];

            lines.push(Line::from(""));
            lines.push(Line::from(format!(
                "Started: {}",
                app.session_start.format("%Y-%m-%d %H:%M:%S UTC")
            )));
            lines.push(Line::from(format!("Duration: {}", app.uptime_formatted())));
            lines.push(Line::from(format!("Total Metrics: {}", app.metrics_count)));
            lines.push(Line::from(format!(
                "Average Rate: {:.2} metrics/min",
                metrics_per_min
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(format!(
                "Total Cost: ${:.4}",
                app.summary.total_cost_usd
            )));
            lines.push(Line::from(format!(
                "Total Tokens: {}",
                app.summary.total_tokens
            )));

            let summary = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White));
            f.render_widget(summary, chunks[4]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info_component_creation() {
        let _component = SessionInfoComponent;
        // Component is created successfully
    }
}
