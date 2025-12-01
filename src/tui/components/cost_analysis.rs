//! Cost analysis component - breakdown by model tier and token types

use super::Component;
use crate::tui::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

pub struct CostAnalysisComponent;

impl Component for CostAnalysisComponent {
    fn render(&self, f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(area);

        // Cost by Token Type Header
        let total_tokens = app.summary.total_tokens;
        let tokens = &app.summary.tokens_by_type;

        // Input tokens gauge
        let input_ratio = if total_tokens > 0 {
            tokens.input as f64 / total_tokens as f64
        } else {
            0.0
        };
        let input_gauge = Gauge::default()
            .block(Block::default().title("Input Tokens").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(input_ratio)
            .label(format!("{} ({:.1}%)", tokens.input, input_ratio * 100.0));
        f.render_widget(input_gauge, chunks[0]);

        // Output tokens gauge
        let output_ratio = if total_tokens > 0 {
            tokens.output as f64 / total_tokens as f64
        } else {
            0.0
        };
        let output_gauge = Gauge::default()
            .block(
                Block::default()
                    .title("Output Tokens")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio(output_ratio)
            .label(format!("{} ({:.1}%)", tokens.output, output_ratio * 100.0));
        f.render_widget(output_gauge, chunks[1]);

        // Cache tokens gauge
        let cache_tokens = tokens.cache_read + tokens.cache_write;
        let cache_ratio = if total_tokens > 0 {
            cache_tokens as f64 / total_tokens as f64
        } else {
            0.0
        };
        let cache_gauge = Gauge::default()
            .block(Block::default().title("Cache Tokens").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Blue))
            .ratio(cache_ratio)
            .label(format!("{} ({:.1}%)", cache_tokens, cache_ratio * 100.0));
        f.render_widget(cache_gauge, chunks[2]);

        // Model tier cost breakdown
        if !chunks[3].is_empty() {
            let mut lines = vec![Line::from(Span::styled(
                "Cost by Model Tier",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))];

            if app.summary.by_model_tier.is_empty() {
                lines.push(Line::from("No model tier data yet..."));
            } else {
                // Calculate percentages
                let total_cost = app.summary.total_cost_usd;

                for (tier, metrics) in &app.summary.by_model_tier {
                    let color = match tier.as_str() {
                        "Opus" => Color::Magenta,
                        "Sonnet" => Color::Blue,
                        "Haiku" => Color::Cyan,
                        _ => Color::White,
                    };

                    let percentage = if total_cost > 0.0 {
                        (metrics.total_cost_usd / total_cost) * 100.0
                    } else {
                        0.0
                    };

                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("{:<10}", tier),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!(
                            ": ${:>8.4} ({:>5.1}%) | {} calls | {} tokens",
                            metrics.total_cost_usd,
                            percentage,
                            metrics.call_count,
                            metrics.total_tokens
                        )),
                    ]));
                }

                // Average cost per call
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("Avg Cost/Call: ${:.4}", app.avg_cost_per_call()),
                    Style::default().fg(Color::Cyan),
                )));
            }

            let breakdown = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White));
            f.render_widget(breakdown, chunks[3]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_analysis_component_creation() {
        let _component = CostAnalysisComponent;
        // Component is created successfully
    }
}
