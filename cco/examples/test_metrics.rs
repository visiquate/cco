/// Quick test program to verify the new metrics parser
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Loading Claude metrics from home directory (parallel parser)...\n");

    // Load metrics using the high-performance parallel implementation
    let metrics = cco::claude_history::load_claude_metrics_from_home_dir_parallel().await?;

    // Display results
    println!("=== CLAUDE METRICS SUMMARY ===\n");

    println!("Global Totals:");
    println!("  Conversations: {}", metrics.conversations_count);
    println!("  Messages:      {}", metrics.messages_count);
    println!("  Total Cost:    ${:.2}", metrics.total_cost);
    println!("  Input Tokens:  {}", metrics.total_input_tokens);
    println!("  Output Tokens: {}", metrics.total_output_tokens);
    println!("  Cache Create:  {}", metrics.total_cache_creation_tokens);
    println!("  Cache Read:    {}", metrics.total_cache_read_tokens);

    println!(
        "\nPer-Model Breakdown ({} models):",
        metrics.model_breakdown.len()
    );
    let mut models: Vec<_> = metrics.model_breakdown.iter().collect();
    models.sort_by(|a, b| b.1.total_cost.partial_cmp(&a.1.total_cost).unwrap());

    for (model, breakdown) in models.iter().take(10) {
        println!(
            "  {:<25} ${:>8.2}  {} messages",
            model, breakdown.total_cost, breakdown.message_count
        );
    }

    println!(
        "\nPer-Project Breakdown ({} projects):",
        metrics.project_breakdown.len()
    );
    let mut projects: Vec<_> = metrics.project_breakdown.iter().collect();
    projects.sort_by(|a, b| b.1.total_cost.partial_cmp(&a.1.total_cost).unwrap());

    for (project, breakdown) in projects.iter().take(10) {
        let short_name = if project.len() > 40 {
            format!("{}...", &project[..37])
        } else {
            project.to_string()
        };
        println!(
            "  {:<40} ${:>8.2}  {} msgs  {} convos",
            short_name, breakdown.total_cost, breakdown.message_count, breakdown.conversation_count
        );
    }

    Ok(())
}
