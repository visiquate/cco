//! Test time-range filtering for Claude metrics
//!
//! Run with: cargo run --example test_time_filter

use std::time::{Duration, SystemTime};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Test 1: Load all metrics (no filter)
    println!("Test 1: Loading all metrics (no time filter)...");
    let start = std::time::Instant::now();
    let (metrics_all, oldest_all) = cco::claude_history::load_claude_metrics_with_time_filter(None).await?;
    let elapsed = start.elapsed();
    
    println!("  ✓ Loaded in {:.2}s", elapsed.as_secs_f64());
    println!("  ✓ Total cost: ${:.2}", metrics_all.total_cost);
    println!("  ✓ Messages: {}", metrics_all.messages_count);
    println!("  ✓ Conversations: {}", metrics_all.conversations_count);
    println!("  ✓ Projects: {}", metrics_all.project_breakdown.len());
    
    if let Some(oldest) = oldest_all {
        let age = oldest.elapsed().unwrap_or(Duration::from_secs(0));
        println!("  ✓ Oldest conversation: {} days ago", age.as_secs() / 86400);
    }
    
    println!();
    
    // Test 2: Load last 7 days
    println!("Test 2: Loading last 7 days...");
    let cutoff_7d = Some(SystemTime::now() - Duration::from_secs(7 * 86400));
    let start = std::time::Instant::now();
    let (metrics_7d, oldest_7d) = cco::claude_history::load_claude_metrics_with_time_filter(cutoff_7d).await?;
    let elapsed = start.elapsed();
    
    println!("  ✓ Loaded in {:.2}s", elapsed.as_secs_f64());
    println!("  ✓ Total cost: ${:.2}", metrics_7d.total_cost);
    println!("  ✓ Messages: {}", metrics_7d.messages_count);
    println!("  ✓ Conversations: {}", metrics_7d.conversations_count);
    println!("  ✓ Projects: {}", metrics_7d.project_breakdown.len());
    
    if let Some(oldest) = oldest_7d {
        let age = oldest.elapsed().unwrap_or(Duration::from_secs(0));
        println!("  ✓ Oldest conversation: {} days ago", age.as_secs() / 86400);
    }
    
    println!();
    
    // Test 3: Load today
    println!("Test 3: Loading today (last 24 hours)...");
    let cutoff_today = Some(SystemTime::now() - Duration::from_secs(86400));
    let start = std::time::Instant::now();
    let (metrics_today, oldest_today) = cco::claude_history::load_claude_metrics_with_time_filter(cutoff_today).await?;
    let elapsed = start.elapsed();
    
    println!("  ✓ Loaded in {:.2}s", elapsed.as_secs_f64());
    println!("  ✓ Total cost: ${:.2}", metrics_today.total_cost);
    println!("  ✓ Messages: {}", metrics_today.messages_count);
    println!("  ✓ Conversations: {}", metrics_today.conversations_count);
    println!("  ✓ Projects: {}", metrics_today.project_breakdown.len());
    
    if let Some(oldest) = oldest_today {
        let age = oldest.elapsed().unwrap_or(Duration::from_secs(0));
        println!("  ✓ Oldest conversation: {} hours ago", age.as_secs() / 3600);
    }
    
    println!();
    
    // Validation checks
    println!("Validation:");
    println!("  ✓ Today ≤ Week ≤ All: {} ≤ {} ≤ {} messages",
        metrics_today.messages_count,
        metrics_7d.messages_count,
        metrics_all.messages_count
    );
    
    assert!(metrics_today.messages_count <= metrics_7d.messages_count,
        "Today should have <= messages than 7 days");
    assert!(metrics_7d.messages_count <= metrics_all.messages_count,
        "7 days should have <= messages than all time");
    
    println!("  ✓ All validation checks passed!");
    
    Ok(())
}
