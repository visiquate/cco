//! Comprehensive unit tests for LogWatcher
//!
//! Tests filesystem watching, debouncing, filtering, error handling,
//! and edge cases for Claude Code conversation log monitoring.

use cco::daemon::log_watcher::LogWatcher;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

#[tokio::test]
async fn test_log_watcher_detects_new_file() {
    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    // Create new .jsonl file
    let test_file = watch_path.join("conversation_123.jsonl");
    fs::write(&test_file, r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5"}}"#).unwrap();

    // Verify event received within 5 seconds
    let result = timeout(Duration::from_secs(5), rx.recv()).await;
    assert!(result.is_ok(), "Should receive file change event");

    let event_path = result.unwrap();
    assert!(event_path.is_some());
    assert_eq!(event_path.unwrap(), test_file);

    println!("✅ New file detection test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_ignores_non_jsonl() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    // Create non-jsonl files
    let txt_file = watch_path.join("notes.txt");
    let tmp_file = watch_path.join("temp.tmp");
    let swp_file = watch_path.join(".vim.swp");
    let hidden_file = watch_path.join(".hidden.jsonl");

    fs::write(&txt_file, "text content").unwrap();
    fs::write(&tmp_file, "temporary").unwrap();
    fs::write(&swp_file, "vim swap").unwrap();
    fs::write(&hidden_file, "hidden json").unwrap();

    // Wait 2 seconds - should NOT receive any events
    let result = timeout(Duration::from_secs(2), rx.recv()).await;
    assert!(result.is_err(), "Should NOT receive events for invalid files");

    println!("✅ Non-JSONL filtering test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_debounce() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    let test_file = watch_path.join("test.jsonl");

    // Modify same file 10 times rapidly
    for i in 0..10 {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&test_file)
            .unwrap();
        writeln!(file, r#"{{"line":{}}}"#, i).unwrap();
        file.flush().unwrap();
        drop(file);

        // Small delay between writes
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Collect events for 3 seconds
    let mut event_count = 0;
    let start = tokio::time::Instant::now();

    while start.elapsed() < Duration::from_secs(3) {
        match timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(_)) => event_count += 1,
            _ => break,
        }
    }

    // Should receive 1-3 events (debounced), not 10
    assert!(
        event_count >= 1 && event_count <= 3,
        "Expected 1-3 events due to debouncing, got {}",
        event_count
    );

    println!("✅ Debounce test passed: {} events received", event_count);

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    // Create multiple .jsonl files
    let file1 = watch_path.join("conv1.jsonl");
    let file2 = watch_path.join("conv2.jsonl");
    let file3 = watch_path.join("conv3.jsonl");

    fs::write(&file1, "content1").unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    fs::write(&file2, "content2").unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    fs::write(&file3, "content3").unwrap();

    // Collect events
    let mut received_files = Vec::new();
    for _ in 0..3 {
        if let Ok(Some(path)) = timeout(Duration::from_secs(5), rx.recv()).await {
            received_files.push(path);
        }
    }

    // Should receive events for all 3 files
    assert!(received_files.len() >= 3, "Should detect all 3 files");

    println!("✅ Multiple file detection test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    // Create nested directory structure
    let nested_dir = watch_path.join("project").join("subdir");
    fs::create_dir_all(&nested_dir).unwrap();

    // Create file in nested directory
    let nested_file = nested_dir.join("deep_conversation.jsonl");
    fs::write(&nested_file, "nested content").unwrap();

    // Should detect nested file (recursive watching)
    let result = timeout(Duration::from_secs(5), rx.recv()).await;
    assert!(result.is_ok(), "Should detect file in nested directory");

    println!("✅ Nested directory test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_handles_nonexistent_directory() {
    // Path that doesn't exist yet
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().join("nonexistent");

    // Should create directory and not panic
    let result = LogWatcher::new(watch_path.clone());
    assert!(result.is_ok(), "Should handle nonexistent directory gracefully");

    let (mut watcher, _rx) = result.unwrap();

    // Directory should now exist
    assert!(watch_path.exists(), "Directory should be created");

    // Should start watching successfully
    assert!(watcher.start().is_ok());

    println!("✅ Nonexistent directory handling test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_concurrent_operations() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    // Spawn multiple tasks creating files concurrently
    let mut handles = vec![];
    for i in 0..5 {
        let path = watch_path.clone();
        let handle = tokio::spawn(async move {
            let file = path.join(format!("concurrent_{}.jsonl", i));
            fs::write(&file, format!("content {}", i)).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Collect events (should get at least 5)
    let mut event_count = 0;
    for _ in 0..5 {
        match timeout(Duration::from_secs(5), rx.recv()).await {
            Ok(Some(_)) => event_count += 1,
            _ => break,
        }
    }

    assert!(event_count >= 5, "Should detect all concurrent file creations");

    println!("✅ Concurrent operations test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_file_modification() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    let test_file = watch_path.join("modify_test.jsonl");

    // Create file
    fs::write(&test_file, "initial content").unwrap();

    // Wait for initial create event
    let _ = timeout(Duration::from_secs(3), rx.recv()).await;

    // Clear debounce
    watcher.clear_debounce().await;

    // Modify file after debounce period
    tokio::time::sleep(Duration::from_millis(1500)).await;
    fs::write(&test_file, "modified content").unwrap();

    // Should receive modification event
    let result = timeout(Duration::from_secs(3), rx.recv()).await;
    assert!(result.is_ok(), "Should detect file modification");

    println!("✅ File modification test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    // Create large file (10MB)
    let test_file = watch_path.join("large.jsonl");
    let mut file = fs::File::create(&test_file).unwrap();
    for i in 0..100_000 {
        writeln!(file, r#"{{"line":{},"data":"some data here"}}"#, i).unwrap();
    }
    file.flush().unwrap();
    drop(file);

    // Should still detect large file
    let result = timeout(Duration::from_secs(10), rx.recv()).await;
    assert!(result.is_ok(), "Should detect large file creation");

    println!("✅ Large file test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_rapid_create_delete() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    // Rapidly create and delete files
    for i in 0..10 {
        let test_file = watch_path.join(format!("temp_{}.jsonl", i));
        fs::write(&test_file, "temporary").unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        let _ = fs::remove_file(&test_file);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Should receive some events (creates), but may miss some (deletes not tracked)
    let mut event_count = 0;
    loop {
        match timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Some(_)) => event_count += 1,
            _ => break,
        }
    }

    // Should have received at least a few events
    assert!(event_count > 0, "Should detect some file operations");

    println!("✅ Rapid create/delete test passed: {} events", event_count);

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_stop_and_restart() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();

    // Start watching
    watcher.start().unwrap();

    // Create file while watching
    let file1 = watch_path.join("before_stop.jsonl");
    fs::write(&file1, "content1").unwrap();

    // Should receive event
    let result1 = timeout(Duration::from_secs(3), rx.recv()).await;
    assert!(result1.is_ok(), "Should detect file before stop");

    // Stop watching
    watcher.stop().unwrap();

    // Create file while NOT watching
    let file2 = watch_path.join("during_stop.jsonl");
    fs::write(&file2, "content2").unwrap();

    // Should NOT receive event (timeout expected)
    let result2 = timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(result2.is_err(), "Should NOT detect file while stopped");

    // Restart watching
    watcher.start().unwrap();

    // Clear debounce to allow immediate events
    watcher.clear_debounce().await;

    // Create file after restart
    let file3 = watch_path.join("after_restart.jsonl");
    fs::write(&file3, "content3").unwrap();

    // Should receive event again
    let result3 = timeout(Duration::from_secs(3), rx.recv()).await;
    assert!(result3.is_ok(), "Should detect file after restart");

    println!("✅ Stop and restart test passed");

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_channel_buffer_overflow() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    // Create 150 files rapidly (exceeds channel buffer of 100)
    for i in 0..150 {
        let file = watch_path.join(format!("overflow_{}.jsonl", i));
        fs::write(&file, format!("content {}", i)).unwrap();
    }

    // Drain channel - should get at least some events
    let mut event_count = 0;
    loop {
        match timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(_)) => {
                event_count += 1;
                if event_count >= 100 {
                    break; // Stop after collecting 100
                }
            }
            _ => break,
        }
    }

    // Should have received many events (may lose some due to buffer overflow)
    assert!(event_count >= 50, "Should receive many events even with overflow");

    println!("✅ Channel buffer overflow test passed: {} events", event_count);

    // Cleanup
    let _ = watcher.stop();
}

#[tokio::test]
async fn test_log_watcher_clear_debounce() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
    watcher.start().unwrap();

    let test_file = watch_path.join("debounce_test.jsonl");

    // Create file
    fs::write(&test_file, "content1").unwrap();
    let _ = timeout(Duration::from_secs(2), rx.recv()).await;

    // Try to modify immediately (should be debounced)
    fs::write(&test_file, "content2").unwrap();
    let result1 = timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(result1.is_err(), "Should be debounced");

    // Clear debounce
    watcher.clear_debounce().await;

    // Now modification should trigger event
    fs::write(&test_file, "content3").unwrap();
    let result2 = timeout(Duration::from_secs(3), rx.recv()).await;
    assert!(result2.is_ok(), "Should detect after clearing debounce");

    println!("✅ Clear debounce test passed");

    // Cleanup
    let _ = watcher.stop();
}
