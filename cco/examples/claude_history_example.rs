//! Example demonstrating how to use the claude_history module
//!
//! This example shows how to load and analyze Claude conversation history
//! from a project directory.
//!
//! Usage:
//!   cargo run --example claude_history_example

use cco::claude_history::load_claude_project_metrics;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt::init();

    // Example project path (adjust to your actual project)
    // Claude encodes paths with dashes, e.g., /Users/brent/git/cc-orchestra becomes
    // -Users-brent-git-cc-orchestra
    let project_path = dirs::home_dir()
        .unwrap()
        .join(".claude/projects/-Users-brent-git-cc-orchestra");

    println!("Loading Claude metrics from: {:?}", project_path);
    println!();

    match load_claude_project_metrics(project_path.to_str().unwrap()).await {
        Ok(metrics) => {
            println!("=== Claude Project Metrics ===");
            println!();
            println!("Total Conversations: {}", metrics.conversations_count);
            println!("Total Messages:      {}", metrics.messages_count);
            println!();
            println!("Token Usage:");
            println!("  Input tokens:         {:>12}", metrics.total_input_tokens);
            println!("  Output tokens:        {:>12}", metrics.total_output_tokens);
            println!("  Cache creation:       {:>12}", metrics.total_cache_creation_tokens);
            println!("  Cache reads:          {:>12}", metrics.total_cache_read_tokens);
            println!();
            println!("Total Cost: ${:.4}", metrics.total_cost);
            println!();

            if !metrics.model_breakdown.is_empty() {
                println!("=== Breakdown by Model ===");
                println!();

                let mut models: Vec<_> = metrics.model_breakdown.iter().collect();
                models.sort_by(|a, b| b.1.total_cost.partial_cmp(&a.1.total_cost).unwrap());

                for (model, breakdown) in models {
                    println!("{}:", model);
                    println!("  Messages:       {:>8}", breakdown.message_count);
                    println!("  Input tokens:   {:>12}", breakdown.input_tokens);
                    println!("  Output tokens:  {:>12}", breakdown.output_tokens);
                    if breakdown.cache_creation_tokens > 0 {
                        println!("  Cache creation: {:>12}", breakdown.cache_creation_tokens);
                    }
                    if breakdown.cache_read_tokens > 0 {
                        println!("  Cache reads:    {:>12}", breakdown.cache_read_tokens);
                    }
                    println!("  Cost:           ${:>11.4}", breakdown.total_cost);
                    println!();
                }
            }

            println!("Last updated: {}", metrics.last_updated);
        }
        Err(e) => {
            eprintln!("Error loading metrics: {}", e);
            eprintln!();
            eprintln!("Make sure the project directory exists and contains .jsonl files");
        }
    }

    Ok(())
}
