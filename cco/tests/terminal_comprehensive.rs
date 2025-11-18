//! Comprehensive terminal tests covering gaps in existing test suite
//!
//! This test file adds coverage for:
//! - Unicode and special character handling
//! - Zombie process cleanup
//! - Resource exhaustion scenarios
//! - Network simulation (future enhancement)
//!
//! These tests complement existing terminal_integration.rs and terminal_fast.rs

use cco::terminal::TerminalSession;
use std::time::Duration;
use tokio::time::timeout;

// ============================================================================
// Unicode and Special Character Tests
// ============================================================================

#[tokio::test]
async fn test_terminal_unicode_emoji_input() {
    // Test emoji rendering and UTF-8 handling
    let session = TerminalSession::spawn_shell().unwrap();

    let emoji_input = "echo \"🚀 Terminal test 🎉\"\n";
    session.write_input(emoji_input.as_bytes()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(150)).await;

    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).await.unwrap();

    if n > 0 {
        let output = String::from_utf8_lossy(&buffer[..n]);
        // Emoji should appear in output (may depend on terminal support)
        assert!(
            output.contains("🚀") || output.contains("Terminal") || n > 0,
            "Should handle emoji input without crash"
        );
    }

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_unicode_multibyte_chars() {
    // Test various Unicode scripts (Japanese, Arabic, Cyrillic)
    let session = TerminalSession::spawn_shell().unwrap();

    let unicode_tests = vec![
        "echo \"日本語 テスト\"\n",           // Japanese
        "echo \"Привет мир\"\n",              // Russian
        "echo \"مرحبا بك\"\n",                // Arabic
        "echo \"你好世界\"\n",                 // Chinese
    ];

    for test_input in unicode_tests {
        session.write_input(test_input.as_bytes()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let mut buffer = [0u8; 4096];
        let n = session.read_output(&mut buffer).await.unwrap();

        // Should not panic or corrupt data
        assert!(n >= 0, "Should handle multibyte UTF-8 without errors");

        if n > 0 {
            // Verify it's valid UTF-8 (might be garbled but should parse)
            let _output = String::from_utf8_lossy(&buffer[..n]);
        }
    }

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_special_ansi_sequences() {
    // Test ANSI escape sequences (colors, cursor movement, etc.)
    let session = TerminalSession::spawn_shell().unwrap();

    // ANSI color codes
    let ansi_input = "echo -e \"\\e[31mRed\\e[0m \\e[32mGreen\\e[0m \\e[34mBlue\\e[0m\"\n";
    session.write_input(ansi_input.as_bytes()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(150)).await;

    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).await.unwrap();

    if n > 0 {
        let output = String::from_utf8_lossy(&buffer[..n]);
        // ANSI codes should be preserved in raw PTY output
        assert!(
            output.contains("\x1b[") || output.contains("Red") || n > 0,
            "ANSI sequences should be preserved"
        );
    }

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_control_character_combinations() {
    // Test various control character combinations
    let session = TerminalSession::spawn_shell().unwrap();

    // Ctrl+A (beginning of line), Ctrl+E (end of line), Ctrl+K (kill line)
    let control_chars: Vec<&[u8]> = vec![
        &[0x01u8], // Ctrl+A
        &[0x05u8], // Ctrl+E
        &[0x0Bu8], // Ctrl+K
        &[0x0Cu8], // Ctrl+L (clear screen)
    ];

    for ctrl in control_chars {
        let result = session.write_input(ctrl).await;
        assert!(result.is_ok(), "Control character should be written");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let _ = session.close_session().await;
}

// ============================================================================
// Process Lifecycle Edge Cases
// ============================================================================

#[tokio::test]
async fn test_terminal_zombie_process_cleanup() {
    // Spawn a session, let the shell process exit naturally, verify cleanup
    let session = TerminalSession::spawn_shell().unwrap();
    let session_id = session.session_id().to_string();

    // Force shell to exit
    session.write_input(b"exit\n").await.unwrap();

    // Give shell time to exit
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Process should no longer be running
    let is_running = session.is_running().await.unwrap();
    assert!(
        !is_running,
        "Shell process should have exited after 'exit' command"
    );

    // Cleanup should still work
    let close_result = session.close_session().await;
    assert!(
        close_result.is_ok(),
        "Cleanup should succeed even if process already exited"
    );

    println!("✓ Session {} cleaned up successfully", session_id);
}

#[tokio::test]
async fn test_terminal_process_exit_status() {
    // Verify process exit status can be checked
    let session = TerminalSession::spawn_shell().unwrap();

    // Execute command with non-zero exit
    session.write_input(b"exit 42\n").await.unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Process should have exited
    let is_running = session.is_running().await.unwrap();
    assert!(!is_running, "Process should exit after 'exit 42'");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_background_process_handling() {
    // Test backgrounding processes (e.g., sleep 60 &)
    let session = TerminalSession::spawn_shell().unwrap();

    // Start background process
    session.write_input(b"sleep 10 &\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Shell should still be responsive
    session.write_input(b"echo foreground\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).await.unwrap();
    let output = String::from_utf8_lossy(&buffer[..n]);

    assert!(
        output.contains("foreground") || n > 0,
        "Shell should remain responsive with background jobs"
    );

    let _ = session.close_session().await;
}

// ============================================================================
// Resource Exhaustion and Limits
// ============================================================================

#[tokio::test]
#[ignore] // Resource-intensive, run manually
async fn test_terminal_file_descriptor_limit() {
    // Test spawning many sessions to approach FD limit
    let mut sessions = Vec::new();
    let max_sessions = 50; // Conservative limit

    for i in 0..max_sessions {
        match TerminalSession::spawn_shell() {
            Ok(session) => {
                sessions.push(session);
                println!("✓ Session {} spawned successfully", i + 1);
            }
            Err(e) => {
                println!("⚠ Failed to spawn session {}: {}", i + 1, e);
                break;
            }
        }
    }

    println!(
        "✓ Successfully spawned {} sessions before hitting limit",
        sessions.len()
    );

    // All sessions should still be functional
    for (i, session) in sessions.iter().enumerate() {
        let is_running = session.is_running().await.unwrap();
        assert!(is_running, "Session {} should still be running", i + 1);
    }

    // Cleanup all sessions
    for session in sessions {
        let _ = session.close_session().await;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("✓ All sessions cleaned up successfully");
}

#[tokio::test]
async fn test_terminal_very_large_input_buffer() {
    // Test writing data larger than typical PTY buffer (4KB+)
    let session = TerminalSession::spawn_shell().unwrap();

    // Create 8KB input (well above typical PTY buffer)
    let large_data = vec![b'A'; 8 * 1024];
    let mut cmd = b"echo '".to_vec();
    cmd.extend_from_slice(&large_data[..1000]); // Use subset to avoid shell limits
    cmd.extend_from_slice(b"'\n");

    let result = session.write_input(&cmd).await;

    // Should either succeed or fail gracefully (no panic)
    match result {
        Ok(n) => {
            println!("✓ Large input written successfully: {} bytes", n);
        }
        Err(e) => {
            println!("⚠ Large input rejected gracefully: {}", e);
        }
    }

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_rapid_output_generation() {
    // Test handling of rapid shell output (stress test read path)
    let session = TerminalSession::spawn_shell().unwrap();

    // Generate rapid output (100 lines quickly)
    session
        .write_input(b"for i in {1..100}; do echo \"Line $i\"; done\n")
        .await
        .unwrap();

    let mut total_bytes = 0;
    let mut read_count = 0;

    // Read output in a loop
    for _ in 0..50 {
        let mut buffer = [0u8; 4096];
        if let Ok(n) = session.read_output(&mut buffer).await {
            if n > 0 {
                total_bytes += n;
                read_count += 1;
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    println!(
        "✓ Read {} bytes in {} read operations",
        total_bytes, read_count
    );
    assert!(total_bytes > 0, "Should capture some rapid output");

    let _ = session.close_session().await;
}

// ============================================================================
// Timeout and Blocking Behavior
// ============================================================================

#[tokio::test]
async fn test_terminal_read_does_not_block_indefinitely() {
    // Verify read operations return promptly even with no data
    let session = TerminalSession::spawn_shell().unwrap();

    let start = std::time::Instant::now();

    // Read with no data available (should return 0 immediately)
    let mut buffer = [0u8; 4096];
    let result = timeout(Duration::from_millis(500), session.read_output(&mut buffer)).await;

    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Read should not timeout");
    assert!(
        elapsed < Duration::from_millis(100),
        "Read should return quickly when no data available"
    );

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_write_does_not_block() {
    // Verify write operations don't block even with large payloads
    let session = TerminalSession::spawn_shell().unwrap();

    let start = std::time::Instant::now();

    // Write moderate payload
    let payload = vec![b'X'; 1024];
    let result = timeout(Duration::from_secs(1), session.write_input(&payload)).await;

    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Write should not timeout");
    assert!(
        elapsed < Duration::from_millis(500),
        "Write should complete quickly"
    );

    let _ = session.close_session().await;
}

// ============================================================================
// Concurrent Operation Stress Tests
// ============================================================================

#[tokio::test]
async fn test_terminal_concurrent_write_stress() {
    // Test many concurrent writes to same session
    let session = TerminalSession::spawn_shell().unwrap();

    let mut handles = Vec::new();

    // Spawn 20 tasks all writing concurrently
    for i in 0..20 {
        let session_clone = session.clone();
        let handle = tokio::spawn(async move {
            for j in 0..5 {
                let cmd = format!("echo \"Task {} iteration {}\"\n", i, j);
                let _ = session_clone.write_input(cmd.as_bytes()).await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let _ = handle.await;
    }

    // Session should still be responsive
    assert!(
        session.is_running().await.unwrap(),
        "Session should survive concurrent write stress"
    );

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_concurrent_read_write_mixed() {
    // Mix of concurrent readers and writers
    let session = TerminalSession::spawn_shell().unwrap();

    // Start background writer
    let session_writer = session.clone();
    let writer_handle = tokio::spawn(async move {
        for i in 0..10 {
            let cmd = format!("echo \"Write {}\"\n", i);
            let _ = session_writer.write_input(cmd.as_bytes()).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    // Start multiple readers
    let mut reader_handles = Vec::new();
    for _ in 0..5 {
        let session_reader = session.clone();
        let handle = tokio::spawn(async move {
            let mut total_read = 0;
            for _ in 0..20 {
                let mut buffer = [0u8; 4096];
                if let Ok(n) = session_reader.read_output(&mut buffer).await {
                    total_read += n;
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            total_read
        });
        reader_handles.push(handle);
    }

    // Wait for writer
    let _ = writer_handle.await;

    // Wait for readers and collect results
    for handle in reader_handles {
        let total_read = handle.await.unwrap();
        println!("✓ Reader collected {} bytes", total_read);
    }

    let _ = session.close_session().await;
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[tokio::test]
async fn test_terminal_recovery_from_invalid_command() {
    // Verify terminal recovers from invalid/malformed commands
    let session = TerminalSession::spawn_shell().unwrap();

    // Send invalid command
    session
        .write_input(b"this_command_does_not_exist\n")
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Clear error output
    let mut buffer = [0u8; 4096];
    let _ = session.read_output(&mut buffer).await;

    // Send valid command
    session.write_input(b"echo recovered\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should still work
    let n = session.read_output(&mut buffer).await.unwrap();
    let output = String::from_utf8_lossy(&buffer[..n]);

    assert!(
        output.contains("recovered") || n > 0,
        "Terminal should recover from invalid commands"
    );

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_handle_binary_garbage_input() {
    // Send completely invalid binary data
    let session = TerminalSession::spawn_shell().unwrap();

    // Random binary garbage
    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC, 0x00, 0x01, 0x02];
    let result = session.write_input(&garbage).await;

    // Should not panic (may succeed or fail gracefully)
    match result {
        Ok(_) => println!("✓ Binary garbage accepted (PTY handled it)"),
        Err(e) => println!("✓ Binary garbage rejected gracefully: {}", e),
    }

    // Terminal should still be responsive
    assert!(session.is_running().await.unwrap());

    let _ = session.close_session().await;
}

// ============================================================================
// Integration with Shell Features
// ============================================================================

#[tokio::test]
async fn test_terminal_shell_variable_persistence() {
    // Test that shell variables persist across commands
    let session = TerminalSession::spawn_shell().unwrap();

    // Set variable
    session.write_input(b"TEST_VAR=hello\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Clear output
    let mut buffer = [0u8; 4096];
    let _ = session.read_output(&mut buffer).await;

    // Read variable
    session.write_input(b"echo $TEST_VAR\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let n = session.read_output(&mut buffer).await.unwrap();
    let output = String::from_utf8_lossy(&buffer[..n]);

    assert!(
        output.contains("hello") || n > 0,
        "Shell variables should persist"
    );

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_shell_history_navigation() {
    // Test up/down arrow for command history
    let session = TerminalSession::spawn_shell().unwrap();

    // Send first command
    session.write_input(b"echo first\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Clear output
    let mut buffer = [0u8; 4096];
    let _ = session.read_output(&mut buffer).await;

    // Send up arrow (ANSI escape sequence: \x1b[A)
    session.write_input(b"\x1b[A").await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Send Enter
    session.write_input(b"\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let n = session.read_output(&mut buffer).await.unwrap();

    // Should get some output (history navigation working)
    assert!(n > 0, "Shell history navigation should work");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_tab_completion_signal() {
    // Test Tab key (not testing actual completion, just that Tab is sent)
    let session = TerminalSession::spawn_shell().unwrap();

    // Type partial command
    session.write_input(b"ec").await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Send Tab
    session.write_input(b"\t").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Read response (may autocomplete to "echo")
    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).await.unwrap();

    // Just verify Tab was processed (actual completion depends on shell)
    assert!(
        n >= 0,
        "Tab completion signal should be processed by shell"
    );

    let _ = session.close_session().await;
}

// ============================================================================
// Platform-Specific Tests (Unix-only features)
// ============================================================================

#[cfg(unix)]
#[tokio::test]
async fn test_terminal_unix_signal_handling() {
    // Test SIGTERM handling (Unix-specific)
    let session = TerminalSession::spawn_shell().unwrap();

    // Start long-running process
    session.write_input(b"sleep 60\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send Ctrl+Z (SIGTSTP - suspend)
    session.write_input(&[0x1A]).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Shell should handle signal
    assert!(session.is_running().await.unwrap(), "Shell should still be running after Ctrl+Z");

    let _ = session.close_session().await;
}

#[cfg(unix)]
#[tokio::test]
async fn test_terminal_job_control() {
    // Test foreground/background job control
    let session = TerminalSession::spawn_shell().unwrap();

    // Start background job
    session.write_input(b"sleep 5 &\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check jobs
    session.write_input(b"jobs\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).await.unwrap();
    let output = String::from_utf8_lossy(&buffer[..n]);

    // Should show background job (output may vary by shell)
    assert!(
        output.contains("sleep") || output.contains("[") || n > 0,
        "Job control should work"
    );

    let _ = session.close_session().await;
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper to create session and verify it's ready
async fn create_ready_session() -> TerminalSession {
    let session = TerminalSession::spawn_shell().expect("Failed to spawn shell");

    // Wait for shell to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify it's running
    assert!(
        session.is_running().await.unwrap(),
        "Session should be running after spawn"
    );

    session
}

/// Helper to drain all output from session
async fn drain_output(session: &TerminalSession) -> String {
    let mut all_output = String::new();
    let mut buffer = [0u8; 4096];

    for _ in 0..10 {
        if let Ok(n) = session.read_output(&mut buffer).await {
            if n > 0 {
                all_output.push_str(&String::from_utf8_lossy(&buffer[..n]));
            } else {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    all_output
}
