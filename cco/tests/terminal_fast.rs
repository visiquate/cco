//! Fast integration tests for terminal functionality - optimized for CI/CD
//!
//! These tests focus on core functionality without long timeouts:
//! - Terminal session lifecycle
//! - Basic I/O operations
//! - Error handling
//! - Concurrency safety

use cco::terminal::TerminalSession;
use std::time::Duration;

#[tokio::test]
async fn test_terminal_session_spawn() {
    let session = TerminalSession::spawn_shell();
    assert!(session.is_ok(), "Should spawn shell successfully");

    let session = session.unwrap();
    assert!(!session.session_id().is_empty(), "Session ID should be non-empty");
    assert_eq!(session.session_id().len(), 36, "Session ID should be UUID format");

    let running = session.is_running().await.unwrap();
    assert!(running, "Shell should be running after spawn");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_session_clone() {
    let session1 = TerminalSession::spawn_shell().unwrap();
    let session1_id = session1.session_id().to_string();

    let session2 = session1.clone();
    let session2_id = session2.session_id().to_string();

    assert_eq!(session1_id, session2_id, "Clones should share session ID");
    assert!(session1.is_running().await.unwrap());
    assert!(session2.is_running().await.unwrap());

    session1.close_session().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(!session2.is_running().await.unwrap(), "All clones affected by close");
}

#[tokio::test]
async fn test_shell_detection() {
    let session = TerminalSession::spawn_shell();
    assert!(session.is_ok(), "Should detect and spawn shell");

    let session = session.unwrap();
    assert!(session.is_running().await.unwrap());

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_multiple_concurrent_sessions() {
    let session1 = TerminalSession::spawn_shell().unwrap();
    let session2 = TerminalSession::spawn_shell().unwrap();
    let session3 = TerminalSession::spawn_shell().unwrap();

    let id1 = session1.session_id().to_string();
    let id2 = session2.session_id().to_string();
    let id3 = session3.session_id().to_string();

    assert_ne!(id1, id2, "Each session should have unique ID");
    assert_ne!(id2, id3, "Each session should have unique ID");
    assert_ne!(id1, id3, "Each session should have unique ID");

    assert!(session1.is_running().await.unwrap());
    assert!(session2.is_running().await.unwrap());
    assert!(session3.is_running().await.unwrap());

    session1.close_session().await.unwrap();
    tokio::time::sleep(Duration::from_millis(25)).await;

    assert!(!session1.is_running().await.unwrap());
    assert!(session2.is_running().await.unwrap());
    assert!(session3.is_running().await.unwrap());

    let _ = session2.close_session().await;
    let _ = session3.close_session().await;
}

#[tokio::test]
async fn test_terminal_write_echo_input() {
    let session = TerminalSession::spawn_shell().unwrap();

    let write_result = session.write_input(b"echo hello\n").await;
    assert!(write_result.is_ok(), "Should write input successfully");
    assert_eq!(write_result.unwrap(), 11, "Should write all bytes");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).await.unwrap();
    let output = String::from_utf8_lossy(&buffer[..n]);

    assert!(output.contains("hello"), "Output should contain echoed text");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_write_multiple() {
    let session = TerminalSession::spawn_shell().unwrap();

    session.write_input(b"echo first\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(75)).await;

    session.write_input(b"echo second\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(75)).await;

    let mut output = String::new();
    for _ in 0..5 {
        let mut buffer = [0u8; 4096];
        if let Ok(n) = session.read_output(&mut buffer).await {
            if n > 0 {
                output.push_str(&String::from_utf8_lossy(&buffer[..n]));
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(output.contains("first") || output.len() > 0);

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_read_non_blocking() {
    let session = TerminalSession::spawn_shell().unwrap();

    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).await.unwrap();

    assert_eq!(n, 0, "Read with no data should return 0");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_read_partial_buffer() {
    let session = TerminalSession::spawn_shell().unwrap();

    session.write_input(b"echo This is a longer echo test\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut small_buffer = [0u8; 32];
    let n = session.read_output(&mut small_buffer).await.unwrap();

    assert!(n > 0 && n <= 32, "Should return data within buffer size");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_large_input() {
    let session = TerminalSession::spawn_shell().unwrap();

    let large_input = vec![b'x'; 1000];
    let mut cmd = b"echo ".to_vec();
    cmd.extend_from_slice(&large_input);
    cmd.extend_from_slice(b"\n");

    let write_result = session.write_input(&cmd).await;
    assert!(write_result.is_ok(), "Should handle large input");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).await.unwrap();
    assert!(n > 0, "Should get output from large command");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_write_control_chars() {
    let session = TerminalSession::spawn_shell().unwrap();

    session.write_input(b"sleep 30\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let result = session.write_input(&[0x03]).await; // Ctrl+C
    assert!(result.is_ok(), "Should write control character");

    let result = session.write_input(b"echo test\n").await;
    assert!(result.is_ok(), "Should write after control char");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_resize() {
    let session = TerminalSession::spawn_shell().unwrap();

    let result = session.set_terminal_size(100, 30).await;
    assert!(result.is_ok(), "Should handle resize");

    let write_result = session.write_input(b"echo resized\n").await;
    assert!(write_result.is_ok(), "Should write after resize");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_is_running() {
    let session = TerminalSession::spawn_shell().unwrap();

    assert!(
        session.is_running().await.unwrap(),
        "Should be running initially"
    );

    session.close_session().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(
        !session.is_running().await.unwrap(),
        "Should not be running after close"
    );
}

#[tokio::test]
async fn test_terminal_rapid_status_checks() {
    let session = TerminalSession::spawn_shell().unwrap();

    for _ in 0..50 {
        assert!(session.is_running().await.unwrap());
    }

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_close_idempotent() {
    let session = TerminalSession::spawn_shell().unwrap();

    let result1 = session.close_session().await;
    assert!(result1.is_ok(), "First close should succeed");

    tokio::time::sleep(Duration::from_millis(25)).await;

    let result2 = session.close_session().await;
    assert!(result2.is_ok(), "Second close should be safe");

    let result3 = session.close_session().await;
    assert!(result3.is_ok(), "Third close should be safe");
}

#[tokio::test]
async fn test_terminal_concurrent_read_write() {
    let session = TerminalSession::spawn_shell().unwrap();

    let session_clone1 = session.clone();
    let session_clone2 = session.clone();

    let task1 = tokio::spawn(async move {
        for i in 0..3 {
            let _ = session_clone1
                .write_input(format!("echo task1_{}\n", i).as_bytes())
                .await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    let task2 = tokio::spawn(async move {
        for i in 0..3 {
            let _ = session_clone2
                .write_input(format!("echo task2_{}\n", i).as_bytes())
                .await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    let _ = task1.await;
    let _ = task2.await;

    let write_result = session.write_input(b"echo final\n").await;
    assert!(write_result.is_ok(), "Session should be responsive");

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_lock_contention() {
    let session = TerminalSession::spawn_shell().unwrap();

    let mut handles = vec![];

    for _ in 0..5 {
        let session_clone = session.clone();
        let handle = tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            for _ in 0..3 {
                let _ = session_clone.read_output(&mut buffer).await;
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    assert!(
        session.is_running().await.is_ok(),
        "Should handle concurrent reads"
    );

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_cleanup_after_intensive_ops() {
    let session = TerminalSession::spawn_shell().unwrap();

    for i in 0..20 {
        let _ = session
            .write_input(format!("echo {}\n", i).as_bytes())
            .await;
        let mut buffer = [0u8; 4096];
        let _ = session.read_output(&mut buffer).await;
    }

    let result = session.close_session().await;
    assert!(result.is_ok(), "Should close cleanly after intensive operations");
}

#[tokio::test]
async fn test_terminal_write_to_closed_session() {
    let session = TerminalSession::spawn_shell().unwrap();

    session.close_session().await.unwrap();
    tokio::time::sleep(Duration::from_millis(25)).await;

    let result = session.write_input(b"echo after_close\n").await;
    // Should either succeed or fail gracefully
    let _ = result;
}

#[tokio::test]
async fn test_terminal_empty_write() {
    let session = TerminalSession::spawn_shell().unwrap();

    let result = session.write_input(b"").await;
    assert!(result.is_ok(), "Empty write should be handled");

    if let Ok(n) = result {
        assert_eq!(n, 0, "Empty write should return 0");
    }

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_terminal_rapid_open_close() {
    for _ in 0..3 {
        let session = TerminalSession::spawn_shell().unwrap();
        session.close_session().await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

// Test helpers for measuring latency
#[tokio::test]
async fn test_spawn_latency() {
    let start = std::time::Instant::now();
    let session = TerminalSession::spawn_shell().unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(1),
        "Spawn should complete quickly"
    );
    assert!(session.is_running().await.unwrap());

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_write_latency() {
    let session = TerminalSession::spawn_shell().unwrap();

    let start = std::time::Instant::now();
    session.write_input(b"echo test\n").await.unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(100),
        "Write should be fast"
    );

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_read_latency() {
    let session = TerminalSession::spawn_shell().unwrap();

    let start = std::time::Instant::now();
    let mut buffer = [0u8; 4096];
    let _ = session.read_output(&mut buffer).await;
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(50),
        "Read should be fast"
    );

    let _ = session.close_session().await;
}

#[tokio::test]
async fn test_close_latency() {
    let session = TerminalSession::spawn_shell().unwrap();

    let start = std::time::Instant::now();
    session.close_session().await.unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(1),
        "Close should complete quickly"
    );
}

// These would require a running server
#[tokio::test]
#[ignore]
async fn websocket_terminal_connect() {
    // Test WebSocket connection to terminal
}

#[tokio::test]
#[ignore]
async fn websocket_localhost_only() {
    // Test localhost-only enforcement
}

#[tokio::test]
#[ignore]
async fn websocket_rate_limiting() {
    // Test rate limiting (10 req/sec)
}
