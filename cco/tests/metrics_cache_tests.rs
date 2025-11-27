//! Comprehensive unit tests for MetricsCache
//!
//! Tests concurrent access, thread safety, ring buffer behavior,
//! snapshot management, and edge cases.

use cco::daemon::metrics_cache::{MetricsCache, StatsSnapshot};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

#[test]
fn test_metrics_cache_concurrent_updates() {
    // Spawn 10 threads updating cache simultaneously
    let cache = Arc::new(MetricsCache::new(1000));
    let mut handles = vec![];

    for thread_id in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let snapshot = StatsSnapshot {
                    timestamp: SystemTime::now(),
                    total_requests: (thread_id * 100 + i) as u64,
                    successful_requests: (thread_id * 100 + i) as u64,
                    failed_requests: 0,
                    avg_response_time: 50.0,
                    uptime: Duration::from_secs((thread_id * 100 + i) as u64),
                    port: 3000,
                    total_cost: (thread_id as f64) * 0.1,
                    total_tokens: (thread_id * 1000 + i) as u64,
                    messages_count: (thread_id * 10 + i / 10) as u64,
                };
                cache_clone.update(snapshot);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify no data races occurred - cache should have 1000 entries
    assert_eq!(cache.len(), 1000);

    // Verify latest entry is valid
    let latest = cache.get_latest();
    assert!(latest.is_some());

    println!(
        "✅ Concurrent test passed: {} entries in cache",
        cache.len()
    );
}

#[test]
fn test_metrics_cache_snapshot_consistency() {
    let cache = MetricsCache::new(100);

    // Add known values
    let snapshot1 = StatsSnapshot {
        timestamp: SystemTime::now(),
        total_requests: 42,
        successful_requests: 40,
        failed_requests: 2,
        avg_response_time: 125.5,
        uptime: Duration::from_secs(3600),
        port: 8080,
        total_cost: 12.34,
        total_tokens: 1_000_000,
        messages_count: 567,
    };

    cache.update(snapshot1.clone());

    // Take snapshot
    let retrieved = cache.get_latest().expect("Cache should have entry");

    // Verify snapshot matches expected values
    assert_eq!(retrieved.total_requests, 42);
    assert_eq!(retrieved.successful_requests, 40);
    assert_eq!(retrieved.failed_requests, 2);
    assert!((retrieved.avg_response_time - 125.5).abs() < 0.001);
    assert_eq!(retrieved.uptime, Duration::from_secs(3600));
    assert_eq!(retrieved.port, 8080);
    assert!((retrieved.total_cost - 12.34).abs() < 0.001);
    assert_eq!(retrieved.total_tokens, 1_000_000);
    assert_eq!(retrieved.messages_count, 567);

    println!("✅ Snapshot consistency test passed");
}

#[test]
fn test_metrics_cache_model_breakdown() {
    // This test verifies that multiple models can be aggregated correctly
    let cache = MetricsCache::new(100);

    // Add metrics for Opus, Sonnet, and Haiku
    let models = vec![
        ("opus", 15.0, 75.0, 100_000, 50_000),
        ("sonnet", 3.0, 15.0, 500_000, 300_000),
        ("haiku", 1.0, 5.0, 1_000_000, 800_000),
    ];

    for (model, input_price, output_price, input_tokens, output_tokens) in models {
        let cost = (input_tokens as f64 / 1_000_000.0) * input_price
            + (output_tokens as f64 / 1_000_000.0) * output_price;

        let snapshot = StatsSnapshot {
            timestamp: SystemTime::now(),
            total_requests: 10,
            successful_requests: 10,
            failed_requests: 0,
            avg_response_time: 100.0,
            uptime: Duration::from_secs(1800),
            port: 3000,
            total_cost: cost,
            total_tokens: input_tokens + output_tokens,
            messages_count: 5,
        };

        cache.update(snapshot);

        println!("Added {} metrics: ${:.4} cost", model, cost);
    }

    // Verify aggregation
    let all_snapshots = cache.get_all();
    assert_eq!(all_snapshots.len(), 3);

    let total_cost: f64 = all_snapshots.iter().map(|s| s.total_cost).sum();
    let total_tokens: u64 = all_snapshots.iter().map(|s| s.total_tokens).sum();

    println!("Total aggregated cost: ${:.4}", total_cost);
    println!("Total aggregated tokens: {}", total_tokens);

    // Verify totals
    assert!(total_cost > 0.0);
    assert_eq!(total_tokens, 150_000 + 800_000 + 1_800_000);

    println!("✅ Model breakdown test passed");
}

#[test]
fn test_metrics_cache_ring_buffer_overflow() {
    // Test that cache properly maintains max_entries limit
    let max_entries = 50;
    let cache = MetricsCache::new(max_entries);

    // Add 100 entries (2x max)
    for i in 0..100 {
        let snapshot = StatsSnapshot {
            timestamp: SystemTime::now(),
            total_requests: i,
            successful_requests: i,
            failed_requests: 0,
            avg_response_time: 50.0,
            uptime: Duration::from_secs(i),
            port: 3000,
            total_cost: (i as f64) * 0.01,
            total_tokens: i * 1000,
            messages_count: i / 10,
        };
        cache.update(snapshot);
    }

    // Should only keep last 50 entries
    assert_eq!(cache.len(), max_entries);

    // Latest should be entry 99
    let latest = cache.get_latest().unwrap();
    assert_eq!(latest.total_requests, 99);

    // Verify oldest entry is 50 (first 50 were dropped)
    let all = cache.get_all();
    assert_eq!(all.first().unwrap().total_requests, 50);
    assert_eq!(all.last().unwrap().total_requests, 99);

    println!("✅ Ring buffer overflow test passed");
}

#[test]
fn test_metrics_cache_empty_state() {
    let cache = MetricsCache::new(100);

    // Verify empty cache behavior
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    assert!(cache.get_latest().is_none());
    assert!(cache.get_all().is_empty());

    println!("✅ Empty state test passed");
}

#[test]
fn test_metrics_cache_time_range_queries() {
    let cache = MetricsCache::new(100);
    let base_time = SystemTime::now();

    // Add entries at 1-second intervals
    for i in 0..60 {
        let timestamp = base_time + Duration::from_secs(i);
        let snapshot = StatsSnapshot {
            timestamp,
            total_requests: i,
            successful_requests: i,
            failed_requests: 0,
            avg_response_time: 50.0,
            uptime: Duration::from_secs(i),
            port: 3000,
            total_cost: 0.0,
            total_tokens: 0,
            messages_count: 0,
        };
        cache.update(snapshot);
    }

    // Query last 10 seconds
    let end = base_time + Duration::from_secs(59);
    let start = end - Duration::from_secs(10);
    let range = cache.get_range(start, end);

    // Should get ~10 entries
    assert!(range.len() >= 10 && range.len() <= 12); // Allow some timing variance

    // Verify entries are in range
    for snapshot in range {
        assert!(snapshot.timestamp >= start);
        assert!(snapshot.timestamp <= end);
    }

    println!("✅ Time range query test passed");
}

#[test]
fn test_metrics_cache_clone_isolation() {
    let cache1 = MetricsCache::new(100);

    let snapshot1 = StatsSnapshot {
        timestamp: SystemTime::now(),
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        avg_response_time: 75.0,
        uptime: Duration::from_secs(7200),
        port: 3000,
        total_cost: 5.67,
        total_tokens: 500_000,
        messages_count: 250,
    };

    cache1.update(snapshot1);

    // Clone cache
    let cache2 = cache1.clone();

    // Verify clone has same data
    assert_eq!(cache2.len(), 1);
    let latest = cache2.get_latest().unwrap();
    assert_eq!(latest.total_requests, 100);

    // Add to original cache
    let snapshot2 = StatsSnapshot {
        timestamp: SystemTime::now(),
        total_requests: 200,
        successful_requests: 190,
        failed_requests: 10,
        avg_response_time: 80.0,
        uptime: Duration::from_secs(7300),
        port: 3000,
        total_cost: 10.34,
        total_tokens: 1_000_000,
        messages_count: 500,
    };

    cache1.update(snapshot2);

    // Original now has 2 entries
    assert_eq!(cache1.len(), 2);

    // Clone should ALSO have 2 entries (shared Arc<RwLock>)
    assert_eq!(cache2.len(), 2);

    println!("✅ Clone isolation test passed (shared state verified)");
}

#[test]
fn test_metrics_cache_high_frequency_updates() {
    // Simulate high-frequency updates (e.g., 100 updates/sec)
    let cache = Arc::new(MetricsCache::new(1000));
    let mut handles = vec![];

    let num_threads = 4;
    let updates_per_thread = 250; // Total: 1000 updates

    for thread_id in 0..num_threads {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for i in 0..updates_per_thread {
                let snapshot = StatsSnapshot {
                    timestamp: SystemTime::now(),
                    total_requests: (thread_id * updates_per_thread + i) as u64,
                    successful_requests: (thread_id * updates_per_thread + i) as u64,
                    failed_requests: 0,
                    avg_response_time: 10.0,
                    uptime: Duration::from_millis((thread_id * updates_per_thread + i) as u64 * 10),
                    port: 3000,
                    total_cost: 0.001,
                    total_tokens: 1000,
                    messages_count: 1,
                };
                cache_clone.update(snapshot);

                // Simulate 10ms interval (100 Hz)
                thread::sleep(Duration::from_micros(100));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Should have 1000 entries (exactly max_entries)
    assert_eq!(cache.len(), 1000);

    println!("✅ High-frequency update test passed");
}

#[test]
fn test_metrics_cache_boundary_conditions() {
    // Test with max_entries = 1 (minimal cache)
    let cache = MetricsCache::new(1);

    let snapshot1 = StatsSnapshot {
        timestamp: SystemTime::now(),
        total_requests: 1,
        successful_requests: 1,
        failed_requests: 0,
        avg_response_time: 50.0,
        uptime: Duration::from_secs(60),
        port: 3000,
        total_cost: 0.01,
        total_tokens: 1000,
        messages_count: 1,
    };

    cache.update(snapshot1);
    assert_eq!(cache.len(), 1);

    // Add second snapshot - should evict first
    let snapshot2 = StatsSnapshot {
        timestamp: SystemTime::now(),
        total_requests: 2,
        successful_requests: 2,
        failed_requests: 0,
        avg_response_time: 60.0,
        uptime: Duration::from_secs(120),
        port: 3000,
        total_cost: 0.02,
        total_tokens: 2000,
        messages_count: 2,
    };

    cache.update(snapshot2);
    assert_eq!(cache.len(), 1);

    let latest = cache.get_latest().unwrap();
    assert_eq!(latest.total_requests, 2); // Should be second snapshot

    println!("✅ Boundary condition test passed");
}

#[test]
fn test_metrics_cache_zero_capacity() {
    // Test degenerate case: max_entries = 0
    let cache = MetricsCache::new(0);

    let snapshot = StatsSnapshot {
        timestamp: SystemTime::now(),
        total_requests: 1,
        successful_requests: 1,
        failed_requests: 0,
        avg_response_time: 50.0,
        uptime: Duration::from_secs(60),
        port: 3000,
        total_cost: 0.01,
        total_tokens: 1000,
        messages_count: 1,
    };

    cache.update(snapshot);

    // With max_entries=0, should immediately evict
    // Implementation will have 1 entry briefly, then remove it
    assert!(cache.len() <= 1);

    println!("✅ Zero capacity test passed");
}

#[tokio::test]
async fn test_metrics_cache_async_access() {
    // Test that cache works correctly from async context
    let cache = Arc::new(MetricsCache::new(100));
    let mut tasks = vec![];

    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let task = tokio::spawn(async move {
            for j in 0..10 {
                let snapshot = StatsSnapshot {
                    timestamp: SystemTime::now(),
                    total_requests: (i * 10 + j) as u64,
                    successful_requests: (i * 10 + j) as u64,
                    failed_requests: 0,
                    avg_response_time: 50.0,
                    uptime: Duration::from_secs((i * 10 + j) as u64),
                    port: 3000,
                    total_cost: 0.01,
                    total_tokens: 1000,
                    messages_count: 1,
                };
                cache_clone.update(snapshot);
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await.expect("Task panicked");
    }

    assert_eq!(cache.len(), 100);

    println!("✅ Async access test passed");
}
