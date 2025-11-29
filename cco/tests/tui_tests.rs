//! Comprehensive tests for the TUI dashboard module

use cco::metrics::{ApiCallEvent, MetricsEngine, ModelTier, TokenBreakdown};
use cco::persistence::{models::ApiMetricRecord, PersistenceLayer};
use cco::tui::app::{ApiCallDisplay, App, AppState};
use std::collections::VecDeque;
use tempfile::TempDir;

#[tokio::test]
async fn test_app_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let app = App::new(persistence, metrics_engine);

    assert_eq!(app.current_tab, AppState::Overview);
    assert_eq!(app.summary.call_count, 0);
    assert_eq!(app.metrics_count, 0);
}

#[tokio::test]
async fn test_app_tab_navigation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let mut app = App::new(persistence, metrics_engine);

    assert_eq!(app.current_tab, AppState::Overview);

    app.next_tab();
    assert_eq!(app.current_tab, AppState::RealTime);

    app.next_tab();
    assert_eq!(app.current_tab, AppState::CostAnalysis);

    app.next_tab();
    assert_eq!(app.current_tab, AppState::SessionInfo);

    app.next_tab();
    assert_eq!(app.current_tab, AppState::Overview); // Cycles back
}

#[tokio::test]
async fn test_app_reverse_navigation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let mut app = App::new(persistence, metrics_engine);

    app.prev_tab();
    assert_eq!(app.current_tab, AppState::SessionInfo);

    app.prev_tab();
    assert_eq!(app.current_tab, AppState::CostAnalysis);

    app.prev_tab();
    assert_eq!(app.current_tab, AppState::RealTime);

    app.prev_tab();
    assert_eq!(app.current_tab, AppState::Overview);
}

#[tokio::test]
async fn test_metrics_update_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let mut app = App::new(persistence, metrics_engine);

    // Update with no data
    let result = app.update_metrics().await;
    assert!(result.is_ok());
    assert_eq!(app.summary.call_count, 0);
    assert_eq!(app.recent_calls.len(), 0);
}

#[tokio::test]
async fn test_metrics_aggregation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    // Record some events
    for i in 0..5 {
        let tokens = TokenBreakdown {
            input_tokens: 100 * i,
            output_tokens: 50 * i,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        let event = ApiCallEvent::new("claude-opus-4".to_string(), tokens, None, None);

        if let Some(event) = event {
            metrics_engine.record_event(event).await;
        }
    }

    let mut app = App::new(persistence, metrics_engine);
    let result = app.update_metrics().await;

    assert!(result.is_ok());
    assert_eq!(app.summary.call_count, 5);
    assert!(app.summary.total_cost_usd > 0.0);
}

#[tokio::test]
async fn test_uptime_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let app = App::new(persistence, metrics_engine);

    // Uptime should be >= 0
    let uptime = app.uptime_seconds();
    assert!(uptime >= 0);

    // Formatted uptime should be HH:MM:SS
    let formatted = app.uptime_formatted();
    let parts: Vec<&str> = formatted.split(':').collect();
    assert_eq!(parts.len(), 3);
}

#[tokio::test]
async fn test_cache_hit_rate_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let app = App::new(persistence, metrics_engine);

    // No calls = 0% cache hit rate
    assert_eq!(app.cache_hit_rate(), 0.0);
}

#[tokio::test]
async fn test_cache_hit_rate_with_data() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    // Record event with cache hits
    let tokens = TokenBreakdown {
        input_tokens: 100,
        output_tokens: 50,
        cache_write_tokens: 0,
        cache_read_tokens: 50,
    };

    let event = ApiCallEvent::new("claude-opus-4".to_string(), tokens, None, None).unwrap();

    metrics_engine.record_event(event).await;

    let mut app = App::new(persistence, metrics_engine);
    let _ = app.update_metrics().await;

    let rate = app.cache_hit_rate();
    assert!(rate > 0.0 && rate <= 100.0);
}

#[tokio::test]
async fn test_avg_cost_per_call_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let app = App::new(persistence, metrics_engine);

    // No calls = $0 average
    assert_eq!(app.avg_cost_per_call(), 0.0);
}

#[tokio::test]
async fn test_avg_cost_per_call_with_data() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    // Record multiple events
    for i in 1..=3 {
        let tokens = TokenBreakdown {
            input_tokens: 1000 * i,
            output_tokens: 500 * i,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        let event = ApiCallEvent::new("claude-haiku".to_string(), tokens, None, None).unwrap();

        metrics_engine.record_event(event).await;
    }

    let mut app = App::new(persistence, metrics_engine);
    let _ = app.update_metrics().await;

    let avg_cost = app.avg_cost_per_call();
    assert!(avg_cost > 0.0);

    // Verify average calculation
    let total = app.summary.total_cost_usd;
    let count = app.summary.call_count as f64;
    assert!((avg_cost - (total / count)).abs() < 0.0001);
}

#[tokio::test]
async fn test_multiple_model_tiers() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    // Record events for different model tiers
    let models = vec!["claude-opus-4", "claude-sonnet-3.5", "claude-3-haiku"];

    for (idx, model) in models.iter().enumerate() {
        let tokens = TokenBreakdown {
            input_tokens: 100 * (idx as u64 + 1),
            output_tokens: 50 * (idx as u64 + 1),
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        let event = ApiCallEvent::new(model.to_string(), tokens, None, None).unwrap();

        metrics_engine.record_event(event).await;
    }

    let mut app = App::new(persistence, metrics_engine);
    let _ = app.update_metrics().await;

    // Should have data for all 3 tiers
    assert_eq!(app.summary.by_model_tier.len(), 3);
    assert!(app.summary.by_model_tier.contains_key("Opus"));
    assert!(app.summary.by_model_tier.contains_key("Sonnet"));
    assert!(app.summary.by_model_tier.contains_key("Haiku"));
}

#[tokio::test]
async fn test_app_state_indices() {
    assert_eq!(AppState::Overview.index(), 0);
    assert_eq!(AppState::RealTime.index(), 1);
    assert_eq!(AppState::CostAnalysis.index(), 2);
    assert_eq!(AppState::SessionInfo.index(), 3);
}

#[tokio::test]
async fn test_app_exit_flag() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let mut app = App::new(persistence, metrics_engine);

    assert!(!app.should_exit);
    app.exit();
    assert!(app.should_exit);
}

#[test]
fn test_api_call_display_creation() {
    use chrono::Utc;

    let display = ApiCallDisplay {
        model_name: "claude-opus-4".to_string(),
        timestamp: Utc::now(),
        tokens: 1000,
        cost_usd: 0.05,
    };

    assert_eq!(display.model_name, "claude-opus-4");
    assert_eq!(display.tokens, 1000);
    assert_eq!(display.cost_usd, 0.05);
}

#[tokio::test]
async fn test_persistence_integration() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();

    // Record event in persistence layer
    let record = ApiMetricRecord::new(
        1000,
        "claude-opus-4".to_string(),
        1000,
        500,
        0,
        0,
        0.05,
        Some("req-123".to_string()),
    );

    let result = persistence.record_event(record).await;
    assert!(result.is_ok());

    // Retrieve and verify
    let metrics = persistence.get_metrics(900, 1100).await.unwrap();
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].model_name, "claude-opus-4");
}

#[tokio::test]
async fn test_token_breakdown_totals() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    let tokens = TokenBreakdown {
        input_tokens: 100,
        output_tokens: 50,
        cache_write_tokens: 20,
        cache_read_tokens: 30,
    };

    let event = ApiCallEvent::new("claude-opus-4".to_string(), tokens, None, None).unwrap();

    metrics_engine.record_event(event).await;

    let mut app = App::new(persistence, metrics_engine);
    let _ = app.update_metrics().await;

    assert_eq!(app.summary.tokens_by_type.input, 100);
    assert_eq!(app.summary.tokens_by_type.output, 50);
    assert_eq!(app.summary.tokens_by_type.cache_write, 20);
    assert_eq!(app.summary.tokens_by_type.cache_read, 30);
    assert_eq!(app.summary.total_tokens, 200);
}

#[tokio::test]
async fn test_recent_calls_deque() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let persistence = PersistenceLayer::new(&db_path).await.unwrap();
    let metrics_engine = MetricsEngine::new();

    // Record 15 events (more than capacity of 10)
    for i in 0..15 {
        let tokens = TokenBreakdown {
            input_tokens: 100 * i,
            output_tokens: 50 * i,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        let event = ApiCallEvent::new("claude-opus-4".to_string(), tokens, None, None).unwrap();

        metrics_engine.record_event(event).await;
    }

    let mut app = App::new(persistence, metrics_engine);
    let _ = app.update_metrics().await;

    // Should only keep last 10
    assert!(app.recent_calls.len() <= 10);
}
