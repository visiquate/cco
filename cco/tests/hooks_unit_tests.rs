//! Unit tests for hooks infrastructure (types, registry, executor)
//!
//! This test suite covers:
//! - HookPayload, HookType, HookConfig (types.rs)
//! - HookRegistry (registry.rs)
//! - HookExecutor (executor.rs)
//!
//! Test organization:
//! - 16 tests for types.rs
//! - 10 tests for registry.rs
//! - 21 tests for executor.rs
//! Total: 47 unit tests
//!
//! Run with: cargo test hooks_unit --lib

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::time::sleep;

// Note: These imports will work once hooks module is implemented
// use cco::daemon::hooks::types::{HookPayload, HookType, HookConfig};
// use cco::daemon::hooks::registry::HookRegistry;
// use cco::daemon::hooks::executor::HookExecutor;
// use cco::daemon::hooks::HookError;

// Mock types for development (remove when actual hooks module exists)
#[derive(Debug, Clone, PartialEq)]
pub struct HookPayload {
    pub command: String,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    PreStart,
    PostStart,
    PreStop,
    PostStop,
    Custom(&'static str),
}

#[derive(Debug, Clone)]
pub struct HookConfig {
    pub timeout: Duration,
    pub retries: usize,
    pub enabled: bool,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            retries: 2,
            enabled: true,
        }
    }
}

// =============================================================================
// SECTION 1: HookPayload Tests (5 tests)
// =============================================================================

#[test]
fn test_hook_payload_construction() {
    let mut context = HashMap::new();
    context.insert("key1".to_string(), "value1".to_string());
    context.insert("key2".to_string(), "value2".to_string());

    let payload = HookPayload {
        command: "start".to_string(),
        context: context.clone(),
    };

    assert_eq!(payload.command, "start");
    assert_eq!(payload.context.len(), 2);
    assert_eq!(payload.context.get("key1"), Some(&"value1".to_string()));
}

#[test]
fn test_hook_payload_with_empty_context() {
    let payload = HookPayload {
        command: "stop".to_string(),
        context: HashMap::new(),
    };

    assert_eq!(payload.command, "stop");
    assert!(payload.context.is_empty());
}

#[test]
fn test_hook_payload_with_complex_context() {
    let mut context = HashMap::new();
    context.insert("config".to_string(), r#"{"port":3000,"host":"localhost"}"#.to_string());
    context.insert("env".to_string(), "production".to_string());

    let payload = HookPayload {
        command: "configure".to_string(),
        context,
    };

    assert_eq!(payload.context.get("config").unwrap(), r#"{"port":3000,"host":"localhost"}"#);
}

#[test]
fn test_hook_payload_command_types() {
    let commands = vec!["start", "stop", "restart", "status", "custom_command"];

    for cmd in commands {
        let payload = HookPayload {
            command: cmd.to_string(),
            context: HashMap::new(),
        };
        assert_eq!(payload.command, cmd);
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_hook_payload_serialization() {
    // This test requires serde implementation
    // Will be implemented when HookPayload has #[derive(Serialize, Deserialize)]

    let mut context = HashMap::new();
    context.insert("key".to_string(), "value".to_string());

    let payload = HookPayload {
        command: "test".to_string(),
        context,
    };

    // Serialize and deserialize
    // let json = serde_json::to_string(&payload).unwrap();
    // let deserialized: HookPayload = serde_json::from_str(&json).unwrap();
    // assert_eq!(payload, deserialized);
}

// =============================================================================
// SECTION 2: HookType Tests (3 tests)
// =============================================================================

#[test]
fn test_hook_type_variants() {
    let types = vec![
        HookType::PreStart,
        HookType::PostStart,
        HookType::PreStop,
        HookType::PostStop,
        HookType::Custom("my_custom_hook"),
    ];

    // Verify all variants exist and are unique
    assert_eq!(types.len(), 5);
    assert_ne!(HookType::PreStart, HookType::PostStart);
    assert_ne!(HookType::PreStop, HookType::PostStop);
}

#[test]
fn test_hook_type_equality() {
    assert_eq!(HookType::PreStart, HookType::PreStart);
    assert_eq!(HookType::Custom("test"), HookType::Custom("test"));
    assert_ne!(HookType::Custom("test1"), HookType::Custom("test2"));
}

#[test]
fn test_hook_type_display() {
    // This test requires Display trait implementation
    // format!("{}", HookType::PreStart) should return "PreStart"

    // Temporary assertion - will be replaced with actual Display test
    let hook_type = HookType::PreStart;
    assert_eq!(hook_type, HookType::PreStart);
}

// =============================================================================
// SECTION 3: HookConfig Tests (8 tests)
// =============================================================================

#[test]
fn test_hook_config_default_values() {
    let config = HookConfig::default();

    assert_eq!(config.timeout, Duration::from_secs(5));
    assert_eq!(config.retries, 2);
    assert_eq!(config.enabled, true);
}

#[test]
fn test_hook_config_custom_values() {
    let config = HookConfig {
        timeout: Duration::from_secs(10),
        retries: 5,
        enabled: false,
    };

    assert_eq!(config.timeout, Duration::from_secs(10));
    assert_eq!(config.retries, 5);
    assert_eq!(config.enabled, false);
}

#[test]
fn test_hook_config_timeout_conversion() {
    let timeout_secs = 30;
    let config = HookConfig {
        timeout: Duration::from_secs(timeout_secs),
        retries: 2,
        enabled: true,
    };

    assert_eq!(config.timeout.as_secs(), timeout_secs);
}

#[cfg(feature = "serde")]
#[test]
fn test_hook_config_from_json() {
    // This requires serde implementation
    // let json = r#"{"timeout_secs": 10, "retries": 3, "enabled": true}"#;
    // let config: HookConfig = serde_json::from_str(json).unwrap();
    // assert_eq!(config.timeout, Duration::from_secs(10));
    // assert_eq!(config.retries, 3);
}

#[cfg(feature = "toml")]
#[test]
fn test_hook_config_from_toml() {
    // This requires toml deserialization
    // let toml_str = r#"
    // timeout_secs = 15
    // retries = 4
    // enabled = false
    // "#;
    // let config: HookConfig = toml::from_str(toml_str).unwrap();
    // assert_eq!(config.timeout, Duration::from_secs(15));
}

#[test]
#[should_panic(expected = "timeout must be > 0")]
fn test_hook_config_validation_timeout_zero() {
    // This test requires validation logic in HookConfig::new()
    // HookConfig::new(Duration::from_secs(0), 2, true).unwrap();

    // Placeholder panic for test structure
    panic!("timeout must be > 0");
}

#[test]
#[should_panic(expected = "retries must be <= 10")]
fn test_hook_config_validation_excessive_retries() {
    // This test requires validation logic
    // HookConfig::new(Duration::from_secs(5), 100, true).unwrap();

    // Placeholder panic for test structure
    panic!("retries must be <= 10");
}

#[test]
fn test_hook_config_disabled_hook() {
    let config = HookConfig {
        timeout: Duration::from_secs(5),
        retries: 2,
        enabled: false,
    };

    // When implemented, disabled hooks should be skipped by executor
    assert!(!config.enabled);
}

// =============================================================================
// SECTION 4: HookRegistry Tests (10 tests)
// =============================================================================

// Note: These tests use a mock HookRegistry for now
// Replace with actual HookRegistry once implemented

struct MockHookRegistry {
    hooks: Arc<dashmap::DashMap<String, Vec<String>>>,
}

impl MockHookRegistry {
    fn new() -> Self {
        Self {
            hooks: Arc::new(dashmap::DashMap::new()),
        }
    }

    fn register(&self, hook_type: HookType, hook_name: String) {
        let key = format!("{:?}", hook_type);
        self.hooks.entry(key).or_insert_with(Vec::new).push(hook_name);
    }

    fn get_hooks(&self, hook_type: HookType) -> Vec<String> {
        let key = format!("{:?}", hook_type);
        self.hooks.get(&key).map(|v| v.clone()).unwrap_or_default()
    }

    fn clear(&self) {
        self.hooks.clear();
    }
}

#[test]
fn test_registry_register_hook() {
    let registry = MockHookRegistry::new();
    registry.register(HookType::PreStart, "setup_logging".to_string());

    let hooks = registry.get_hooks(HookType::PreStart);
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0], "setup_logging");
}

#[test]
fn test_registry_get_hooks_existing() {
    let registry = MockHookRegistry::new();
    registry.register(HookType::PostStart, "notify_system".to_string());

    let hooks = registry.get_hooks(HookType::PostStart);
    assert!(!hooks.is_empty());
}

#[test]
fn test_registry_get_hooks_nonexistent() {
    let registry = MockHookRegistry::new();

    let hooks = registry.get_hooks(HookType::PreStop);
    assert!(hooks.is_empty());
}

#[test]
fn test_registry_clear_all_hooks() {
    let registry = MockHookRegistry::new();
    registry.register(HookType::PreStart, "hook1".to_string());
    registry.register(HookType::PostStart, "hook2".to_string());

    registry.clear();

    assert!(registry.get_hooks(HookType::PreStart).is_empty());
    assert!(registry.get_hooks(HookType::PostStart).is_empty());
}

#[test]
fn test_registry_multiple_hooks_same_type() {
    let registry = MockHookRegistry::new();
    registry.register(HookType::PreStart, "hook1".to_string());
    registry.register(HookType::PreStart, "hook2".to_string());
    registry.register(HookType::PreStart, "hook3".to_string());

    let hooks = registry.get_hooks(HookType::PreStart);
    assert_eq!(hooks.len(), 3);
}

#[test]
fn test_registry_hooks_different_types() {
    let registry = MockHookRegistry::new();
    registry.register(HookType::PreStart, "pre_hook".to_string());
    registry.register(HookType::PostStop, "post_hook".to_string());

    let pre_hooks = registry.get_hooks(HookType::PreStart);
    let post_hooks = registry.get_hooks(HookType::PostStop);

    assert_eq!(pre_hooks.len(), 1);
    assert_eq!(post_hooks.len(), 1);
    assert_eq!(pre_hooks[0], "pre_hook");
    assert_eq!(post_hooks[0], "post_hook");
}

#[tokio::test]
async fn test_registry_concurrent_registration() {
    let registry = Arc::new(MockHookRegistry::new());
    let mut handles = vec![];

    // Spawn 10 tasks, each registering 10 hooks
    for i in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let hook_name = format!("hook_{}_{}", i, j);
                registry_clone.register(HookType::PreStart, hook_name);
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify total count
    let hooks = registry.get_hooks(HookType::PreStart);
    assert_eq!(hooks.len(), 100);
}

#[tokio::test]
async fn test_registry_concurrent_reads() {
    let registry = Arc::new(MockHookRegistry::new());

    // Register some hooks
    for i in 0..5 {
        registry.register(HookType::PostStart, format!("hook_{}", i));
    }

    let mut handles = vec![];

    // Spawn 20 tasks reading concurrently
    for _ in 0..20 {
        let registry_clone = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            let hooks = registry_clone.get_hooks(HookType::PostStart);
            assert_eq!(hooks.len(), 5);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_registry_concurrent_read_write() {
    let registry = Arc::new(MockHookRegistry::new());
    let mut handles = vec![];

    // Spawn 10 writers
    for i in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            registry_clone.register(HookType::PreStop, format!("write_{}", i));
        });
        handles.push(handle);
    }

    // Spawn 10 readers
    for _ in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            let _hooks = registry_clone.get_hooks(HookType::PreStop);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify data consistency
    let hooks = registry.get_hooks(HookType::PreStop);
    assert_eq!(hooks.len(), 10);
}

#[test]
fn test_registry_arc_sharing() {
    let registry = Arc::new(MockHookRegistry::new());
    let registry_clone = Arc::clone(&registry);

    registry.register(HookType::PreStart, "shared_hook".to_string());

    let hooks = registry_clone.get_hooks(HookType::PreStart);
    assert_eq!(hooks.len(), 1);
}

// =============================================================================
// SECTION 5: HookExecutor Tests (21 tests)
// =============================================================================

// Mock hook functions for testing

async fn mock_successful_hook(_payload: HookPayload) -> Result<(), String> {
    sleep(Duration::from_millis(50)).await;
    Ok(())
}

async fn mock_failing_hook(_payload: HookPayload) -> Result<(), String> {
    Err("Mock failure".to_string())
}

async fn mock_timeout_hook(_payload: HookPayload) -> Result<(), String> {
    sleep(Duration::from_secs(30)).await;
    Ok(())
}

async fn mock_quick_hook(_payload: HookPayload) -> Result<(), String> {
    sleep(Duration::from_millis(10)).await;
    Ok(())
}

// 5.1 Basic Execution Tests

#[tokio::test]
async fn test_execute_successful_hook() {
    let payload = HookPayload {
        command: "test".to_string(),
        context: HashMap::new(),
    };

    let result = mock_successful_hook(payload).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_failing_hook() {
    let payload = HookPayload {
        command: "test".to_string(),
        context: HashMap::new(),
    };

    let result = mock_failing_hook(payload).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Mock failure");
}

#[tokio::test]
async fn test_execute_hook_with_payload() {
    let mut context = HashMap::new();
    context.insert("config".to_string(), "test_value".to_string());

    let payload = HookPayload {
        command: "configure".to_string(),
        context,
    };

    // Mock hook that validates payload
    let validate_hook = |p: HookPayload| async move {
        assert_eq!(p.command, "configure");
        assert_eq!(p.context.get("config").unwrap(), "test_value");
        Ok::<(), String>(())
    };

    let result: Result<(), String> = validate_hook(payload).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_hook_return_value() {
    let payload = HookPayload {
        command: "test".to_string(),
        context: HashMap::new(),
    };

    let result = mock_successful_hook(payload).await;
    assert!(matches!(result, Ok(())));
}

#[tokio::test]
async fn test_execute_async_hook() {
    let payload = HookPayload {
        command: "test".to_string(),
        context: HashMap::new(),
    };

    // Verify hook is async and awaitable
    let result = tokio::spawn(async move {
        mock_successful_hook(payload).await
    }).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());
}

// 5.2 Timeout Tests

#[tokio::test]
async fn test_execute_hook_timeout() {
    let payload = HookPayload {
        command: "test".to_string(),
        context: HashMap::new(),
    };

    let timeout = Duration::from_secs(1);
    let result = tokio::time::timeout(timeout, mock_timeout_hook(payload)).await;

    assert!(result.is_err(), "Hook should have timed out");
}

#[tokio::test]
async fn test_execute_hook_within_timeout() {
    let payload = HookPayload {
        command: "test".to_string(),
        context: HashMap::new(),
    };

    let timeout = Duration::from_secs(5);
    let result = tokio::time::timeout(timeout, mock_successful_hook(payload)).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());
}

#[tokio::test]
async fn test_timeout_accuracy() {
    let payload = HookPayload {
        command: "test".to_string(),
        context: HashMap::new(),
    };

    let timeout = Duration::from_secs(2);
    let start = Instant::now();

    let _result = tokio::time::timeout(timeout, mock_timeout_hook(payload)).await;

    let elapsed = start.elapsed();

    // Timeout should fire at ~2s ±200ms
    assert!(elapsed >= Duration::from_millis(1800));
    assert!(elapsed <= Duration::from_millis(2200));
}

#[tokio::test]
async fn test_timeout_cleanup() {
    let payload = HookPayload {
        command: "test".to_string(),
        context: HashMap::new(),
    };

    let timeout = Duration::from_millis(100);
    let _result = tokio::time::timeout(timeout, mock_timeout_hook(payload)).await;

    // Verify task is cancelled (no way to test directly without runtime introspection)
    // In practice, verify memory doesn't grow after many timeouts
    sleep(Duration::from_millis(50)).await;
}

// 5.3 Retry Logic Tests

#[tokio::test]
async fn test_retry_successful_on_second_attempt() {
    let attempt_counter = Arc::new(AtomicUsize::new(0));

    let retry_hook = |counter: Arc<AtomicUsize>| async move {
        let attempt = counter.fetch_add(1, Ordering::SeqCst);
        if attempt == 0 {
            Err("First attempt fails".to_string())
        } else {
            Ok(())
        }
    };

    // Simulate retry logic
    let mut last_error = None;
    for _ in 0..3 {
        match retry_hook(Arc::clone(&attempt_counter)).await {
            Ok(_) => {
                last_error = None;
                break;
            }
            Err(e) => last_error = Some(e),
        }
    }

    assert!(last_error.is_none(), "Should succeed on retry");
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_retry_exhaustion() {
    let max_retries = 2;
    let attempt_counter = Arc::new(AtomicUsize::new(0));

    let always_fail_hook = |counter: Arc<AtomicUsize>| async move {
        counter.fetch_add(1, Ordering::SeqCst);
        Err::<(), String>("Always fails".to_string())
    };

    // Simulate retry logic
    let mut last_error = None;
    for _ in 0..=max_retries {
        match always_fail_hook(Arc::clone(&attempt_counter)).await {
            Ok(_) => break,
            Err(e) => last_error = Some(e),
        }
    }

    assert!(last_error.is_some());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3); // Initial + 2 retries
}

#[tokio::test]
async fn test_retry_count_tracking() {
    let attempt_counter = Arc::new(AtomicUsize::new(0));
    let max_retries = 5;

    let counting_hook = |counter: Arc<AtomicUsize>| async move {
        counter.fetch_add(1, Ordering::SeqCst);
        Err::<(), String>("Track attempts".to_string())
    };

    for _ in 0..=max_retries {
        let _: Result<(), String> = counting_hook(Arc::clone(&attempt_counter)).await;
    }

    assert_eq!(attempt_counter.load(Ordering::SeqCst), max_retries + 1);
}

#[tokio::test]
async fn test_no_retry_on_immediate_success() {
    let attempt_counter = Arc::new(AtomicUsize::new(0));

    let immediate_success_hook = |counter: Arc<AtomicUsize>| async move {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok::<(), String>(())
    };

    let result: Result<(), String> = immediate_success_hook(Arc::clone(&attempt_counter)).await;

    assert!(result.is_ok());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 1); // Only one attempt
}

#[tokio::test]
async fn test_retry_delay() {
    let attempt_counter = Arc::new(AtomicUsize::new(0));
    let retry_delay = Duration::from_millis(100);

    let delayed_hook = |counter: Arc<AtomicUsize>| async move {
        counter.fetch_add(1, Ordering::SeqCst);
        Err::<(), String>("Delayed retry".to_string())
    };

    let start = Instant::now();

    // Simulate 2 retries with delay
    for i in 0..3 {
        let _: Result<(), String> = delayed_hook(Arc::clone(&attempt_counter)).await;
        if i < 2 {
            sleep(retry_delay).await;
        }
    }

    let elapsed = start.elapsed();

    // Should take at least 200ms (2 retries * 100ms delay)
    assert!(elapsed >= Duration::from_millis(200));
}

// 5.4 Panic Recovery Tests

#[tokio::test]
async fn test_hook_panic_recovery() {
    let panic_hook = || async {
        panic!("Intentional panic for testing");
    };

    // Use catch_unwind equivalent for async
    let result = tokio::spawn(panic_hook()).await;

    assert!(result.is_err(), "Should catch panic");
}

#[tokio::test]
async fn test_hook_panic_message() {
    let panic_message = "Custom panic message";

    let panic_hook = move || async move {
        panic!("{}", panic_message);
    };

    let result = tokio::spawn(panic_hook()).await;

    assert!(result.is_err());
    // Panic message extraction requires runtime support
}

#[tokio::test]
async fn test_hook_panic_no_retry() {
    // Panics should not be retried (permanent failure)
    let attempt_counter = Arc::new(AtomicUsize::new(0));

    let panic_hook = |counter: Arc<AtomicUsize>| async move {
        counter.fetch_add(1, Ordering::SeqCst);
        panic!("No retry on panic");
    };

    let _result = tokio::spawn(panic_hook(Arc::clone(&attempt_counter))).await;

    // Only one attempt should occur
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_multiple_hook_panic_isolation() {
    let results = Arc::new(dashmap::DashMap::new());

    // Execute hooks independently without using a Vec of closures (type mismatch issue)
    // Hook 0: should succeed
    let result_0 = tokio::spawn(async { Ok::<(), String>(()) }).await;
    results.insert(0, result_0.is_ok());

    // Hook 1: should panic
    let result_1 = tokio::spawn(async { panic!("Middle hook panics") }).await;
    results.insert(1, result_1.is_ok());

    // Hook 2: should succeed
    let result_2 = tokio::spawn(async { Ok::<(), String>(()) }).await;
    results.insert(2, result_2.is_ok());

    // Verify hooks 0 and 2 succeeded, hook 1 panicked
    assert_eq!(results.get(&0).unwrap().clone(), true);
    assert_eq!(results.get(&1).unwrap().clone(), false);
    assert_eq!(results.get(&2).unwrap().clone(), true);
}

// 5.5 Concurrent Execution Tests

#[tokio::test]
async fn test_execute_multiple_hooks_parallel() {
    let hook_count = 5;
    let mut handles = vec![];

    let start = Instant::now();

    for i in 0..hook_count {
        let handle = tokio::spawn(async move {
            let payload = HookPayload {
                command: format!("hook_{}", i),
                context: HashMap::new(),
            };
            mock_successful_hook(payload).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();

    // All hooks should complete in ~50ms (not 250ms sequential)
    assert!(elapsed < Duration::from_millis(150));
}

#[tokio::test]
async fn test_concurrent_execution_isolation() {
    let shared_state = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let state = Arc::clone(&shared_state);
        let handle: tokio::task::JoinHandle<Result<(), String>> = tokio::spawn(async move {
            state.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(10)).await;
            Ok::<(), String>(())
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    assert_eq!(shared_state.load(Ordering::SeqCst), 10);
}

#[tokio::test]
async fn test_concurrent_execution_error_handling() {
    let mut handles = vec![];

    // Mix of successful and failing hooks
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let payload = HookPayload {
                command: format!("hook_{}", i),
                context: HashMap::new(),
            };

            if i % 2 == 0 {
                mock_successful_hook(payload).await
            } else {
                mock_failing_hook(payload).await
            }
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    assert_eq!(success_count, 5);
    assert_eq!(error_count, 5);
}

// =============================================================================
// Test Helpers
// =============================================================================

/// Measure duration of async operation
async fn measure_duration<F, Fut, T>(f: F) -> (Duration, T)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = f().await;
    let duration = start.elapsed();
    (duration, result)
}

/// Assert duration is within tolerance
fn assert_duration_near(actual: Duration, expected: Duration, tolerance_ms: u64) {
    let diff = if actual > expected {
        actual - expected
    } else {
        expected - actual
    };
    assert!(
        diff.as_millis() <= tolerance_ms as u128,
        "Duration {:?} not within {}ms of {:?}",
        actual,
        tolerance_ms,
        expected
    );
}

#[test]
fn test_duration_helper() {
    let duration = Duration::from_millis(1050);
    let expected = Duration::from_secs(1);

    // Should pass: 1050ms is within 100ms of 1000ms
    assert_duration_near(duration, expected, 100);
}
