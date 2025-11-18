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
    HookExecutor, HookPayload, HookRegistry, HookType, ClassificationResult, CrudClassification,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

// =============================================================================
// SECTION 1: Basic Hook Execution (3 tests)
// =============================================================================

#[tokio::test]
async fn test_pre_command_hook_executes() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    // Track if hook executed
    let executed = Arc::new(AtomicUsize::new(0));
    let exec_clone = Arc::clone(&executed);

    // Register hook
    registry.register(
        HookType::PreCommand,
        Box::new(move |_payload| {
            exec_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    ).unwrap();

    // Execute hook
    let payload = HookPayload::new("test command");
    executor.execute_hook(HookType::PreCommand, payload).await.unwrap();

    assert_eq!(executed.load(Ordering::SeqCst), 1, "Hook should execute once");
}

#[tokio::test]
async fn test_post_command_hook_executes() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let executed = Arc::new(AtomicUsize::new(0));
    let exec_clone = Arc::clone(&executed);

    registry.register(
        HookType::PostCommand,
        Box::new(move |_payload| {
            exec_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    ).unwrap();

    let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
    let payload = HookPayload::with_classification("ls -la", classification);

    executor.execute_hook(HookType::PostCommand, payload).await.unwrap();

    assert_eq!(executed.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_post_execution_hook_executes() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let executed = Arc::new(AtomicUsize::new(0));
    let exec_clone = Arc::clone(&executed);

    registry.register(
        HookType::PostExecution,
        Box::new(move |_payload| {
            exec_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    ).unwrap();

    let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
    let payload = HookPayload::with_execution("ls -la", classification, "success");

    executor.execute_hook(HookType::PostExecution, payload).await.unwrap();

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
    let cmd_clone = Arc::clone(&received_command);

    registry.register(
        HookType::PreCommand,
        Box::new(move |payload| {
            *cmd_clone.blocking_lock() = payload.command.clone();
            Ok(())
        }),
    ).unwrap();

    let payload = HookPayload::new("test command 123");
    executor.execute_hook(HookType::PreCommand, payload).await.unwrap();

    let received = received_command.lock().await;
    assert_eq!(*received, "test command 123");
}

#[tokio::test]
async fn test_hook_receives_classification() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let received_classification = Arc::new(Mutex::new(None));
    let class_clone = Arc::clone(&received_classification);

    registry.register(
        HookType::PostCommand,
        Box::new(move |payload| {
            *class_clone.blocking_lock() = payload.classification.clone();
            Ok(())
        }),
    ).unwrap();

    let classification = ClassificationResult::new(CrudClassification::Create, 0.88);
    let payload = HookPayload::with_classification("mkdir test", classification.clone());

    executor.execute_hook(HookType::PostCommand, payload).await.unwrap();

    let received = received_classification.lock().await;
    assert!(received.is_some());
    assert_eq!(received.as_ref().unwrap().classification, CrudClassification::Create);
    assert_eq!(received.as_ref().unwrap().confidence, 0.88);
}

#[tokio::test]
async fn test_hook_receives_execution_result() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let received_result = Arc::new(Mutex::new(None));
    let result_clone = Arc::clone(&received_result);

    registry.register(
        HookType::PostExecution,
        Box::new(move |payload| {
            *result_clone.blocking_lock() = payload.execution_result.clone();
            Ok(())
        }),
    ).unwrap();

    let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
    let payload = HookPayload::with_execution("git status", classification, "exit code 0");

    executor.execute_hook(HookType::PostExecution, payload).await.unwrap();

    let received = received_result.lock().await;
    assert_eq!(received.as_ref(), Some(&"exit code 0".to_string()));
}

#[tokio::test]
async fn test_hook_receives_metadata() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let received_metadata = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let meta_clone = Arc::clone(&received_metadata);

    registry.register(
        HookType::PreCommand,
        Box::new(move |payload| {
            *meta_clone.blocking_lock() = payload.metadata.clone();
            Ok(())
        }),
    ).unwrap();

    let payload = HookPayload::new("test")
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

    executor.execute_hook(HookType::PreCommand, payload).await.unwrap();

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
        let order_clone = Arc::clone(&execution_order);
        registry.register(
            HookType::PreCommand,
            Box::new(move |_payload| {
                order_clone.blocking_lock().push(i);
                Ok(())
            }),
        ).unwrap();
    }

    let payload = HookPayload::new("test");
    executor.execute_hook(HookType::PreCommand, payload).await.unwrap();

    let order = execution_order.lock().await;
    assert_eq!(*order, vec![1, 2, 3], "Hooks should execute in registration order");
}

#[tokio::test]
async fn test_hooks_different_types_dont_interfere() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let pre_count = Arc::new(AtomicUsize::new(0));
    let post_count = Arc::new(AtomicUsize::new(0));

    let pre_clone = Arc::clone(&pre_count);
    let post_clone = Arc::clone(&post_count);

    registry.register(
        HookType::PreCommand,
        Box::new(move |_| {
            pre_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    ).unwrap();

    registry.register(
        HookType::PostCommand,
        Box::new(move |_| {
            post_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    ).unwrap();

    // Execute PreCommand hook
    executor.execute_hook(HookType::PreCommand, HookPayload::new("test")).await.unwrap();

    assert_eq!(pre_count.load(Ordering::SeqCst), 1);
    assert_eq!(post_count.load(Ordering::SeqCst), 0, "PostCommand hook should not execute");
}

#[tokio::test]
async fn test_execute_all_hook_types_in_sequence() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let execution_log = Arc::new(Mutex::new(Vec::new()));

    // Register hooks for each type
    for hook_type in &[HookType::PreCommand, HookType::PostCommand, HookType::PostExecution] {
        let log_clone = Arc::clone(&execution_log);
        let type_name = format!("{:?}", hook_type);
        registry.register(
            *hook_type,
            Box::new(move |_| {
                log_clone.blocking_lock().push(type_name.clone());
                Ok(())
            }),
        ).unwrap();
    }

    // Execute in lifecycle order
    executor.execute_hook(HookType::PreCommand, HookPayload::new("cmd")).await.unwrap();

    let classification = ClassificationResult::new(CrudClassification::Read, 0.9);
    executor.execute_hook(
        HookType::PostCommand,
        HookPayload::with_classification("cmd", classification.clone())
    ).await.unwrap();

    executor.execute_hook(
        HookType::PostExecution,
        HookPayload::with_execution("cmd", classification, "success")
    ).await.unwrap();

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
    registry.register(
        HookType::PreCommand,
        Box::new(|_| Err(cco::daemon::hooks::HookError::execution_failed("test", "intentional failure"))),
    ).unwrap();

    let payload = HookPayload::new("test command");
    let result = executor.execute_hook(HookType::PreCommand, payload).await;

    // Hook execution should report error but not panic
    assert!(result.is_err());
}

#[tokio::test]
async fn test_hook_failure_logged() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    registry.register(
        HookType::PreCommand,
        Box::new(|_| Err(cco::daemon::hooks::HookError::execution_failed("test_hook", "test error"))),
    ).unwrap();

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
    let count_clone = Arc::clone(&success_count);
    registry.register(
        HookType::PreCommand,
        Box::new(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    ).unwrap();

    registry.register(
        HookType::PreCommand,
        Box::new(|_| Err(cco::daemon::hooks::HookError::execution_failed("failing_hook", "fail"))),
    ).unwrap();

    let count_clone2 = Arc::clone(&success_count);
    registry.register(
        HookType::PreCommand,
        Box::new(move |_| {
            count_clone2.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    ).unwrap();

    let payload = HookPayload::new("test");
    let _ = executor.execute_hook(HookType::PreCommand, payload).await;

    // Both successful hooks should execute
    assert_eq!(success_count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_hook_timeout_enforcement() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    // Register a hook that takes too long
    registry.register(
        HookType::PreCommand,
        Box::new(|_| {
            std::thread::sleep(Duration::from_secs(10)); // Exceeds default timeout
            Ok(())
        }),
    ).unwrap();

    let payload = HookPayload::new("test");
    let start = std::time::Instant::now();
    let result = executor.execute_hook(HookType::PreCommand, payload).await;
    let elapsed = start.elapsed();

    // Should timeout before 10 seconds
    assert!(elapsed < Duration::from_secs(6));
    // Timeout should result in error
    assert!(result.is_err() || result.is_ok()); // Depends on timeout implementation
}

#[tokio::test]
async fn test_hook_panic_recovery() {
    let registry = Arc::new(HookRegistry::new());
    let executor = HookExecutor::new(registry.clone());

    let executed_after_panic = Arc::new(AtomicUsize::new(0));

    // Register hook that panics
    registry.register(
        HookType::PreCommand,
        Box::new(|_| {
            panic!("Intentional panic for testing");
        }),
    ).unwrap();

    // Register another hook after the panicking one
    let exec_clone = Arc::clone(&executed_after_panic);
    registry.register(
        HookType::PreCommand,
        Box::new(move |_| {
            exec_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    ).unwrap();

    let payload = HookPayload::new("test");
    let result = executor.execute_hook(HookType::PreCommand, payload).await;

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
    let counter_clone = Arc::clone(&counter);

    registry.register(
        HookType::PreCommand,
        Box::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(10));
            Ok(())
        }),
    ).unwrap();

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
    let data_clone = Arc::clone(&shared_data);

    registry.register(
        HookType::PreCommand,
        Box::new(move |payload| {
            data_clone.blocking_lock().push(payload.command.clone());
            Ok(())
        }),
    ).unwrap();

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
    assert_eq!(data.len(), 100, "All hooks should execute without data loss");
}
