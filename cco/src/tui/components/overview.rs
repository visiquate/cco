//! Overview component - summary metrics display

use super::Component;
use crate::tui::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

pub struct OverviewComponent;

impl Component for OverviewComponent {
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

        // Total Cost
        let total_cost_text = format!("${:.4}", app.summary.total_cost_usd);
        let total_cost = Paragraph::new(total_cost_text)
            .block(Block::default().title("Total Cost").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(total_cost, chunks[0]);

        // API Calls
        let call_count_text = format!("{}", app.summary.call_count);
        let call_count = Paragraph::new(call_count_text)
            .block(Block::default().title("API Calls").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(call_count, chunks[1]);

        // Total Tokens
        let total_tokens_text = format!("{}", app.summary.total_tokens);
        let total_tokens = Paragraph::new(total_tokens_text)
            .block(Block::default().title("Total Tokens").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(total_tokens, chunks[2]);

        // Cache Hit Rate Gauge
        let cache_rate = app.cache_hit_rate();
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title("Cache Hit Rate")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Blue))
            .ratio(cache_rate / 100.0)
            .label(format!("{:.1}%", cache_rate));
        f.render_widget(gauge, chunks[3]);

        // Model Tier Breakdown
        if !chunks[4].is_empty() {
            let mut tier_lines = vec![Line::from(Span::styled(
                "Model Tier Breakdown",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))];

            for (tier, metrics) in &app.summary.by_model_tier {
                let _color = match tier.as_str() {
                    "Opus" => Color::Magenta,
                    "Sonnet" => Color::Blue,
                    "Haiku" => Color::Cyan,
                    _ => Color::White,
                };

                tier_lines.push(Line::from(format!(
                    "{}: {} calls, {} tokens, ${:.4}",
                    tier, metrics.call_count, metrics.total_tokens, metrics.total_cost_usd
                )));
            }

            let tier_breakdown = Paragraph::new(tier_lines)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White));
            f.render_widget(tier_breakdown, chunks[4]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overview_component_creation() {
        let _component = OverviewComponent;
        // Component is created successfully
    }
}
