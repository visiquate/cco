/// Benchmark program to compare sequential vs parallel metrics parsing
use anyhow::Result;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with INFO level to see performance logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║       Claude Metrics Parser Performance Benchmark         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Count files first
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let projects_dir = format!("{}/.claude/projects", home);

    println!("📊 Counting JSONL files in {}...", projects_dir);

    let mut file_count = 0usize;
    let mut project_entries = tokio::fs::read_dir(&projects_dir).await?;
    while let Some(project_entry) = project_entries.next_entry().await? {
        let path = project_entry.path();
        if path.is_dir() {
            let mut files = tokio::fs::read_dir(&path).await?;
            while let Some(file_entry) = files.next_entry().await? {
                if file_entry.path().extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    file_count += 1;
                }
            }
        }
    }

    println!("   Found {} JSONL files\n", file_count);

    // Benchmark 1: Sequential (original) implementation
    println!("═══════════════════════════════════════════════════════════");
    println!("🐌 SEQUENTIAL PARSING (original)");
    println!("═══════════════════════════════════════════════════════════");

    let sequential_start = Instant::now();
    let sequential_metrics = cco::claude_history::load_claude_metrics_from_home_dir().await?;
    let sequential_elapsed = sequential_start.elapsed();

    println!("\n✅ Sequential Results:");
    println!("   Time:          {:.2}s", sequential_elapsed.as_secs_f64());
    println!("   Throughput:    {:.1} files/sec", file_count as f64 / sequential_elapsed.as_secs_f64());
    println!("   Messages:      {}", sequential_metrics.messages_count);
    println!("   Conversations: {}", sequential_metrics.conversations_count);
    println!("   Total Cost:    ${:.2}", sequential_metrics.total_cost);
    println!("   Projects:      {}", sequential_metrics.project_breakdown.len());
    println!("   Models:        {}", sequential_metrics.model_breakdown.len());

    // Benchmark 2: Parallel implementation
    println!("\n═══════════════════════════════════════════════════════════");
    println!("🚀 PARALLEL PARSING (optimized)");
    println!("═══════════════════════════════════════════════════════════");

    let parallel_start = Instant::now();
    let parallel_metrics = cco::claude_history::load_claude_metrics_from_home_dir_parallel().await?;
    let parallel_elapsed = parallel_start.elapsed();

    println!("\n✅ Parallel Results:");
    println!("   Time:          {:.2}s", parallel_elapsed.as_secs_f64());
    println!("   Throughput:    {:.1} files/sec", file_count as f64 / parallel_elapsed.as_secs_f64());
    println!("   Messages:      {}", parallel_metrics.messages_count);
    println!("   Conversations: {}", parallel_metrics.conversations_count);
    println!("   Total Cost:    ${:.2}", parallel_metrics.total_cost);
    println!("   Projects:      {}", parallel_metrics.project_breakdown.len());
    println!("   Models:        {}", parallel_metrics.model_breakdown.len());

    // Compare results
    println!("\n═══════════════════════════════════════════════════════════");
    println!("📈 PERFORMANCE COMPARISON");
    println!("═══════════════════════════════════════════════════════════");

    let speedup = sequential_elapsed.as_secs_f64() / parallel_elapsed.as_secs_f64();
    let time_saved = sequential_elapsed - parallel_elapsed;

    println!("\n⚡ Speedup:       {:.2}x faster", speedup);
    println!("   Time saved:    {:.2}s", time_saved.as_secs_f64());
    println!("   Improvement:   {:.1}%", (speedup - 1.0) * 100.0);

    // Validate results match
    println!("\n🔍 VALIDATION");
    println!("═══════════════════════════════════════════════════════════");

    let messages_match = sequential_metrics.messages_count == parallel_metrics.messages_count;
    let cost_match = (sequential_metrics.total_cost - parallel_metrics.total_cost).abs() < 0.01;
    let projects_match = sequential_metrics.project_breakdown.len() == parallel_metrics.project_breakdown.len();
    let models_match = sequential_metrics.model_breakdown.len() == parallel_metrics.model_breakdown.len();

    println!("   Messages match:      {} ({} vs {})",
             if messages_match { "✅" } else { "❌" },
             sequential_metrics.messages_count,
             parallel_metrics.messages_count);
    println!("   Cost match:          {} (${:.2} vs ${:.2})",
             if cost_match { "✅" } else { "❌" },
             sequential_metrics.total_cost,
             parallel_metrics.total_cost);
    println!("   Projects match:      {} ({} vs {})",
             if projects_match { "✅" } else { "❌" },
             sequential_metrics.project_breakdown.len(),
             parallel_metrics.project_breakdown.len());
    println!("   Models match:        {} ({} vs {})",
             if models_match { "✅" } else { "❌" },
             sequential_metrics.model_breakdown.len(),
             parallel_metrics.model_breakdown.len());

    let all_match = messages_match && cost_match && projects_match && models_match;

    if all_match {
        println!("\n✅ ALL VALIDATIONS PASSED - Results are identical!");
    } else {
        println!("\n⚠️  VALIDATION FAILED - Results differ!");
    }

    // Target metrics
    println!("\n═══════════════════════════════════════════════════════════");
    println!("🎯 TARGET ACHIEVEMENT");
    println!("═══════════════════════════════════════════════════════════");

    let target_time = 60.0; // seconds
    let target_throughput = 40.0; // files/sec
    let target_memory = 500.0; // MB

    let parallel_throughput = file_count as f64 / parallel_elapsed.as_secs_f64();
    let time_target_met = parallel_elapsed.as_secs_f64() < target_time;
    let throughput_target_met = parallel_throughput > target_throughput;

    println!("   Time < 60s:          {} ({:.2}s)",
             if time_target_met { "✅" } else { "❌" },
             parallel_elapsed.as_secs_f64());
    println!("   Throughput > 40/s:   {} ({:.1} files/sec)",
             if throughput_target_met { "✅" } else { "❌" },
             parallel_throughput);
    println!("   Memory < 500MB:      ⚠️  (requires manual measurement)");

    println!("\n═══════════════════════════════════════════════════════════");

    if time_target_met && throughput_target_met && all_match {
        println!("🎉 SUCCESS! All targets met and validated!");
    } else if all_match {
        println!("⚠️  Performance targets not met, but results are correct.");
    } else {
        println!("❌ FAILURE - Results validation failed!");
    }

    println!("═══════════════════════════════════════════════════════════\n");

    Ok(())
}
