//! Integration tests for hooks execution
//!
//! Tests the hooks execution pipeline including:
//! - Pre-command hooks execution
//! - Post-command hooks execution
//! - Post-execution hooks
//! - Hook payload validation
//! - Multiple hooks execution order
//! - Hook failure handling
//!
//! Run with: cargo test hooks_execution

use cco::daemon::hooks::{
    ClassificationResult, CrudClassification, Hook, HookError, HookExecutor, HookPayload,
    HookRegistry, HookResult, HookType,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// =============================================================================
// Mock Hook Implementations
// =============================================================================

/// Simple counter hook that tracks execution count
#[derive(Clone)]
struct CounterHook {
    counter: Arc<AtomicUsize>,
}

impl CounterHook {
    fn new(counter: Arc<AtomicUsize>) -> Self {
        Self { counter }
    }
}

impl Hook for CounterHook {
    fn execute(&self, _payload: &HookPayload) -> HookResult<()> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// Hook that captures command in a mutex
#[derive(Clone)]
struct CommandCaptureHook {
    command: Arc<Mutex<String>>,
}

impl CommandCaptureHook {
    fn new(command: Arc<Mutex<String>>) -> Self {
        Self { command }
    }
}

impl Hook for CommandCaptureHook {
    fn execute(&self, payload: &HookPayload) -> HookResult<()> {
        *self.command.blocking_lock() = payload.command.clone();
        Ok(())
    }
}

/// Hook that captures classification
#[derive(Clone)]
struct ClassificationCaptureHook {
    classification: Arc<Mutex<Option<ClassificationResult>>>,
}

impl ClassificationCaptureHook {
    fn new(classification: Arc<Mutex<Option<ClassificationResult>>>) -> Self {
        Self { classification }
    }
}

impl Hook for ClassificationCaptureHook {
    fn execute(&self, payload: &HookPayload) -> HookResult<()> {
        *self.classification.blocking_lock() = payload.classification.clone();
        Ok(())
    }
}

/// Hook that captures execution result
#[derive(Clone)]
struct ExecutionResultCaptureHook {
    result: Arc<Mutex<Option<String>>>,
}

impl ExecutionResultCaptureHook {
    fn new(result: Arc<Mutex<Option<String>>>) -> Self {
        Self { result }
    }
}

impl Hook for ExecutionResultCaptureHook {
    fn execute(&self, payload: &HookPayload) -> HookResult<()> {
        *self.result.blocking_lock() = payload.execution_result.clone();
        Ok(())
    }
}

/// Hook that captures metadata
#[derive(Clone)]
struct MetadataCaptureHook {
    metadata: Arc<Mutex<HashMap<String, String>>>,
}

impl MetadataCaptureHook {
    fn new(metadata: Arc<Mutex<HashMap<String, String>>>) -> Self {
        Self { metadata }
    }
}

impl Hook for MetadataCaptureHook {
    fn execute(&self, payload: &HookPayload) -> HookResult<()> {
        *self.metadata.blocking_lock() = payload.metadata.clone();
        Ok(())
    }
}

/// Hook that tracks execution order
#[derive(Clone)]
struct OrderTrackingHook {
    order: Arc<Mutex<Vec<usize>>>,
    id: usize,
}

impl OrderTrackingHook {
    fn new(order: Arc<Mutex<Vec<usize>>>, id: usize) -> Self {
        Self { order, id }
    }
}

impl Hook for OrderTrackingHook {
    fn execute(&self, _payload: &HookPayload) -> HookResult<()> {
        self.order.blocking_lock().push(self.id);
        Ok(())
    }
}

/// Hook that logs hook type execution
#[derive(Clone)]
struct TypeLoggingHook {
    log: Arc<Mutex<Vec<String>>>,
    hook_type_name: String,
}

impl TypeLoggingHook {
    fn new(log: Arc<Mutex<Vec<String>>>, hook_type_name: String) -> Self {
        Self {
            log,
            hook_type_name,
        }
    }
}

impl Hook for TypeLoggingHook {
    fn execute(&self, _payload: &HookPayload) -> HookResult<()> {
        self.log.blocking_lock().push(self.hook_type_name.clone());
        Ok(())
    }
}

/// Hook that always fails
struct FailingHook {
    error_message: String,
}

impl FailingHook {
    fn new(error_message: impl Into<String>) -> Self {
        Self {
            error_message: error_message.into(),
        }
    }
}

impl Hook for FailingHook {
    fn execute(&self, _payload: &HookPayload) -> HookResult<()> {
        Err(HookError::execution_failed(
            "failing_hook",
            &self.error_message,
        ))
    }
}

/// Hook that sleeps for a specified duration
struct SleepingHook {
    duration: Duration,
}

impl SleepingHook {
    fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl Hook for SleepingHook {
    fn execute(&self, _payload: &HookPayload) -> HookResult<()> {
        std::thread::sleep(self.duration);
        Ok(())
    }
}

/// Hook that panics
struct PanickingHook;

impl Hook for PanickingHook {
    fn execute(&self, _payload: &HookPayload) -> HookResult<()> {
        panic!("Intentional panic for testing");
    }
}

/// Hook that tracks execution in shared data
#[derive(Clone)]
struct SharedDataHook {
    data: Arc<Mutex<Vec<String>>>,
}

impl SharedDataHook {
    fn new(data: Arc<Mutex<Vec<String>>>) -> Self {
        Self { data }
    }
}

impl Hook for SharedDataHook {
    fn execute(&self, payload: &HookPayload) -> HookResult<()> {
        self.data.blocking_lock().push(payload.command.clone());
        Ok(())
    }
}

// =============================================================================
// SECTION 1: Basic Hook Execution (3 tests)
// =============================================================================

#[tokio::test]
async fn test_pre_command_hook_executes() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    // Track if hook executed
    let executed = Arc::new(AtomicUsize::new(0));
    let hook = CounterHook::new(Arc::clone(&executed));

    // Register hook
    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    // Execute hook
    let payload = HookPayload::new("test command");
    executor
        .execute_hook(HookType::PreCommand, payload)
        .await
        .unwrap();

    assert_eq!(executed.load(Ordering::SeqCst), 1, "Hook should execute once");
}

#[tokio::test]
async fn test_post_command_hook_executes() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let executed = Arc::new(AtomicUsize::new(0));
    let hook = CounterHook::new(Arc::clone(&executed));

    registry
        .register(HookType::PostCommand, Box::new(hook))
        .unwrap();

    let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
    let payload = HookPayload::with_classification("ls -la", classification);

    executor
        .execute_hook(HookType::PostCommand, payload)
        .await
        .unwrap();

    assert_eq!(executed.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_post_execution_hook_executes() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let executed = Arc::new(AtomicUsize::new(0));
    let hook = CounterHook::new(Arc::clone(&executed));

    registry
        .register(HookType::PostExecution, Box::new(hook))
        .unwrap();

    let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
    let payload = HookPayload::with_execution("ls -la", classification, "success");

    executor
        .execute_hook(HookType::PostExecution, payload)
        .await
        .unwrap();

    assert_eq!(executed.load(Ordering::SeqCst), 1);
}

// =============================================================================
// SECTION 2: Hook Payload Validation (4 tests)
// =============================================================================

#[tokio::test]
async fn test_hook_payload_contains_command() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let received_command = Arc::new(Mutex::new(String::new()));
    let hook = CommandCaptureHook::new(Arc::clone(&received_command));

    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    let payload = HookPayload::new("test command 123");
    executor
        .execute_hook(HookType::PreCommand, payload)
        .await
        .unwrap();

    let received = received_command.lock().await;
    assert_eq!(*received, "test command 123");
}

#[tokio::test]
async fn test_hook_receives_classification() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let received_classification = Arc::new(Mutex::new(None));
    let hook = ClassificationCaptureHook::new(Arc::clone(&received_classification));

    registry
        .register(HookType::PostCommand, Box::new(hook))
        .unwrap();

    let classification = ClassificationResult::new(CrudClassification::Create, 0.88);
    let payload = HookPayload::with_classification("mkdir test", classification.clone());

    executor
        .execute_hook(HookType::PostCommand, payload)
        .await
        .unwrap();

    let received = received_classification.lock().await;
    assert!(received.is_some());
    assert_eq!(
        received.as_ref().unwrap().classification,
        CrudClassification::Create
    );
    assert_eq!(received.as_ref().unwrap().confidence, 0.88);
}

#[tokio::test]
async fn test_hook_receives_execution_result() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let received_result = Arc::new(Mutex::new(None));
    let hook = ExecutionResultCaptureHook::new(Arc::clone(&received_result));

    registry
        .register(HookType::PostExecution, Box::new(hook))
        .unwrap();

    let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
    let payload = HookPayload::with_execution("git status", classification, "exit code 0");

    executor
        .execute_hook(HookType::PostExecution, payload)
        .await
        .unwrap();

    let received = received_result.lock().await;
    assert_eq!(received.as_ref(), Some(&"exit code 0".to_string()));
}

#[tokio::test]
async fn test_hook_receives_metadata() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let received_metadata = Arc::new(Mutex::new(HashMap::new()));
    let hook = MetadataCaptureHook::new(Arc::clone(&received_metadata));

    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    let payload = HookPayload::new("test")
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

    executor
        .execute_hook(HookType::PreCommand, payload)
        .await
        .unwrap();

    let received = received_metadata.lock().await;
    assert_eq!(received.get("key1"), Some(&"value1".to_string()));
    assert_eq!(received.get("key2"), Some(&"value2".to_string()));
}

// =============================================================================
// SECTION 3: Multiple Hooks Execution (3 tests)
// =============================================================================

#[tokio::test]
async fn test_multiple_hooks_execute_in_order() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let execution_order = Arc::new(Mutex::new(Vec::new()));

    // Register 3 hooks
    for i in 1..=3 {
        let hook = OrderTrackingHook::new(Arc::clone(&execution_order), i);
        registry
            .register(HookType::PreCommand, Box::new(hook))
            .unwrap();
    }

    let payload = HookPayload::new("test");
    executor
        .execute_hook(HookType::PreCommand, payload)
        .await
        .unwrap();

    let order = execution_order.lock().await;
    assert_eq!(
        *order,
        vec![1, 2, 3],
        "Hooks should execute in registration order"
    );
}

#[tokio::test]
async fn test_hooks_different_types_dont_interfere() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let pre_count = Arc::new(AtomicUsize::new(0));
    let post_count = Arc::new(AtomicUsize::new(0));

    let pre_hook = CounterHook::new(Arc::clone(&pre_count));
    let post_hook = CounterHook::new(Arc::clone(&post_count));

    registry
        .register(HookType::PreCommand, Box::new(pre_hook))
        .unwrap();

    registry
        .register(HookType::PostCommand, Box::new(post_hook))
        .unwrap();

    // Execute PreCommand hook
    executor
        .execute_hook(HookType::PreCommand, HookPayload::new("test"))
        .await
        .unwrap();

    assert_eq!(pre_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        post_count.load(Ordering::SeqCst),
        0,
        "PostCommand hook should not execute"
    );
}

#[tokio::test]
async fn test_execute_all_hook_types_in_sequence() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let execution_log = Arc::new(Mutex::new(Vec::new()));

    // Register hooks for each type
    for hook_type in &[
        HookType::PreCommand,
        HookType::PostCommand,
        HookType::PostExecution,
    ] {
        let type_name = format!("{:?}", hook_type);
        let hook = TypeLoggingHook::new(Arc::clone(&execution_log), type_name);
        registry.register(*hook_type, Box::new(hook)).unwrap();
    }

    // Execute in lifecycle order
    executor
        .execute_hook(HookType::PreCommand, HookPayload::new("cmd"))
        .await
        .unwrap();

    let classification = ClassificationResult::new(CrudClassification::Read, 0.9);
    executor
        .execute_hook(
            HookType::PostCommand,
            HookPayload::with_classification("cmd", classification.clone()),
        )
        .await
        .unwrap();

    executor
        .execute_hook(
            HookType::PostExecution,
            HookPayload::with_execution("cmd", classification, "success"),
        )
        .await
        .unwrap();

    let log = execution_log.lock().await;
    assert_eq!(log.len(), 3);
    assert_eq!(log[0], "PreCommand");
    assert_eq!(log[1], "PostCommand");
    assert_eq!(log[2], "PostExecution");
}

// =============================================================================
// SECTION 4: Hook Failure Handling (5 tests)
// =============================================================================

#[tokio::test]
async fn test_hook_failure_doesnt_block_command() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    // Register a failing hook
    let hook = FailingHook::new("intentional failure");
    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    let payload = HookPayload::new("test command");
    let result = executor.execute_hook(HookType::PreCommand, payload).await;

    // Hook execution should report error but not panic
    assert!(result.is_err());
}

#[tokio::test]
async fn test_hook_failure_logged() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let hook = FailingHook::new("test error");
    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    let payload = HookPayload::new("test");
    let result = executor.execute_hook(HookType::PreCommand, payload).await;

    assert!(result.is_err());
    // In production, verify error is logged
    // This would require a tracing subscriber to capture logs
}

#[tokio::test]
async fn test_one_hook_fails_others_execute() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let success_count = Arc::new(AtomicUsize::new(0));

    // Register 3 hooks: success, fail, success
    let hook1 = CounterHook::new(Arc::clone(&success_count));
    registry
        .register(HookType::PreCommand, Box::new(hook1))
        .unwrap();

    let hook2 = FailingHook::new("fail");
    registry
        .register(HookType::PreCommand, Box::new(hook2))
        .unwrap();

    let hook3 = CounterHook::new(Arc::clone(&success_count));
    registry
        .register(HookType::PreCommand, Box::new(hook3))
        .unwrap();

    let payload = HookPayload::new("test");
    let _ = executor.execute_hook(HookType::PreCommand, payload).await;

    // Both successful hooks should execute
    assert_eq!(success_count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_hook_timeout_enforcement() {
    let registry = Arc::new(HookRegistry::new());
    // Use custom config with 0 retries to test pure timeout behavior
    let executor =
        HookExecutor::with_config(registry.clone(), Duration::from_secs(2), 0);

    // Register a hook that takes too long
    let hook = SleepingHook::new(Duration::from_secs(10));
    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    let payload = HookPayload::new("test");
    let start = std::time::Instant::now();
    let result = executor.execute_hook(HookType::PreCommand, payload).await;
    let elapsed = start.elapsed();

    // Should timeout at ~2 seconds (configured timeout)
    assert!(
        elapsed < Duration::from_secs(4),
        "Hook should timeout within 4 seconds, but took {:?}",
        elapsed
    );
    // Timeout should result in error
    assert!(result.is_err(), "Timeout should result in error");
}

#[tokio::test]
async fn test_hook_panic_recovery() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let executed_after_panic = Arc::new(AtomicUsize::new(0));

    // Register hook that panics
    registry
        .register(HookType::PreCommand, Box::new(PanickingHook))
        .unwrap();

    // Register another hook after the panicking one
    let hook = CounterHook::new(Arc::clone(&executed_after_panic));
    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    let payload = HookPayload::new("test");
    let _result = executor.execute_hook(HookType::PreCommand, payload).await;

    // Execution should continue despite panic
    // Note: Actual panic handling depends on implementation
    // The second hook may or may not execute depending on panic recovery strategy
}

// =============================================================================
// SECTION 5: Concurrent Hook Execution (2 tests)
// =============================================================================

#[tokio::test]
async fn test_concurrent_hook_executions() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let counter = Arc::new(AtomicUsize::new(0));
    let hook = CounterHook::new(Arc::clone(&counter));

    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    // Execute 10 hooks concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let exec = executor.clone();
        handles.push(tokio::spawn(async move {
            let payload = HookPayload::new(format!("command_{}", i));
            exec.execute_hook(HookType::PreCommand, payload).await
        }));
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[tokio::test]
async fn test_hook_execution_thread_safety() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let shared_data = Arc::new(Mutex::new(Vec::new()));
    let hook = SharedDataHook::new(Arc::clone(&shared_data));

    registry
        .register(HookType::PreCommand, Box::new(hook))
        .unwrap();

    // Execute 100 hooks concurrently
    let mut handles = vec![];
    for i in 0..100 {
        let exec = executor.clone();
        handles.push(tokio::spawn(async move {
            let payload = HookPayload::new(format!("cmd_{}", i));
            exec.execute_hook(HookType::PreCommand, payload).await
        }));
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let data = shared_data.lock().await;
    assert_eq!(
        data.len(),
        100,
        "All hooks should execute without data loss"
    );
}
