# Terminal System Developer Guide

## Architecture Overview

The terminal system is built on a three-layer architecture:

```
┌─ Browser Layer ──────────────────────┐
│ xterm.js Terminal Emulator            │
│ - Rendering                           │
│ - Input Capture                       │
│ - ANSI Code Processing               │
└──────────┬──────────────────────────┘
           │ WebSocket
           │ (Binary/Text Messages)
           │
┌──────────▼──────────────────────────┐
│ CCO Server Layer (Async)             │
│ - WebSocket Handler                  │
│ - Message Router                     │
│ - Session Manager                    │
│ - I/O Multiplexer                    │
└──────────┬──────────────────────────┘
           │ PTY Interface
           │ (portable-pty)
           │
┌──────────▼──────────────────────────┐
│ Operating System Layer               │
│ - PTY Master-Slave Pair              │
│ - Shell Process (bash/sh)            │
│ - Signal Handling                    │
│ - Process Management                 │
└──────────────────────────────────────┘
```

## Code Structure

### Files

- **`cco/src/terminal.rs`**: Core PTY session management
  - `TerminalSession` struct
  - Shell spawning and process management
  - I/O operations
  - Size management

- **`cco/src/server.rs`**: WebSocket integration
  - `terminal_handler()`: HTTP upgrade
  - `handle_terminal_socket()`: Connection logic
  - Message routing
  - Background I/O tasks

## How to Extend Terminal Features

### Adding a New Command Handler

Example: Add a custom shell command handler

**Step 1: Define Handler Function**

In `terminal.rs`, add:
```rust
/// Handle custom terminal command
fn handle_custom_command(args: &[&str]) -> Result<String> {
    match args.get(0) {
        Some(&"status") => {
            Ok("Terminal online".to_string())
        }
        Some(&"info") => {
            Ok(format!("Columns: 80, Rows: 24"))
        }
        _ => Err(anyhow!("Unknown command"))
    }
}
```

**Step 2: Integrate with Write Handler**

In `server.rs`, modify `handle_terminal_socket()`:
```rust
Ok(Message::Text(text)) => {
    // Check for custom commands first
    if let Some(stripped) = text.strip_prefix("/terminal-") {
        let parts: Vec<&str> = stripped.split_whitespace().collect();
        match handle_custom_command(&parts) {
            Ok(response) => {
                let msg = Message::text(response);
                sender.send(msg).await?;
            }
            Err(e) => warn!("Command error: {}", e),
        }
    } else if text.starts_with("\x1b[RESIZE;") {
        // Existing resize logic...
    } else {
        // Normal shell input
        session.write_input(text.as_bytes())?;
    }
}
```

**Step 3: Test the Handler**

```bash
# Terminal client sends
/terminal-status

# Server responds
Terminal online
```

### Adding New Terminal Features

#### 1. Terminal Session History

Track command execution history:

```rust
pub struct HistoryEntry {
    command: String,
    timestamp: SystemTime,
    exit_code: i32,
}

pub struct TerminalSession {
    // ... existing fields ...
    history: Arc<Mutex<Vec<HistoryEntry>>>,
}

impl TerminalSession {
    fn record_command(&self, cmd: &str, exit_code: i32) -> Result<()> {
        let mut history = self.history.lock()?;
        history.push(HistoryEntry {
            command: cmd.to_string(),
            timestamp: SystemTime::now(),
            exit_code,
        });
        Ok(())
    }
}
```

#### 2. Output Recording

Capture terminal output to file:

```rust
pub struct TerminalSession {
    // ... existing fields ...
    output_log: Arc<Mutex<File>>,
}

pub fn read_output_with_log(&self, buffer: &mut [u8]) -> Result<usize> {
    let n = self.read_output(buffer)?;

    if n > 0 {
        let mut log = self.output_log.lock()?;
        log.write_all(&buffer[..n])?;
        log.flush()?;
    }

    Ok(n)
}
```

#### 3. Command Monitoring

Detect and monitor long-running commands:

```rust
pub struct CommandMonitor {
    command: String,
    start_time: Instant,
    warning_threshold: Duration,
}

impl CommandMonitor {
    pub fn check(&self) -> Option<String> {
        let elapsed = self.start_time.elapsed();
        if elapsed > self.warning_threshold {
            Some(format!(
                "Command running for {:?}",
                elapsed
            ))
        } else {
            None
        }
    }
}
```

#### 4. Multiple Terminal Support

Support multiple terminals in one session:

```rust
pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
}

impl TerminalManager {
    pub async fn new_session(&self) -> Result<String> {
        let session = TerminalSession::spawn_shell()?;
        let id = session.session_id().to_string();

        self.sessions.lock()?.insert(id.clone(), session);

        Ok(id)
    }

    pub fn get_session(&self, id: &str) -> Result<TerminalSession> {
        self.sessions
            .lock()?
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("Session not found"))
    }
}
```

## Debugging Tips

### Enable Detailed Logging

Set environment variable before running:

```bash
export RUST_LOG=cco::terminal=trace
cargo run --release
```

Log levels:
- `trace`: Very detailed function entry/exit
- `debug`: Important information
- `info`: Major events
- `warn`: Warnings
- `error`: Errors only

### Log Output Examples

**Session Creation**:
```
[DEBUG] Creating terminal session: a1b2c3d4-e5f6-g7h8
[DEBUG] Using shell: /bin/bash
[INFO] Shell process spawned for session: a1b2c3d4-e5f6-g7h8
```

**I/O Operations**:
```
[DEBUG] Received 47 bytes from client
[DEBUG] Read 256 bytes from shell
[DEBUG] Sending 256 bytes to client
```

**Session Cleanup**:
```
[INFO] Closing terminal session: a1b2c3d4-e5f6-g7h8
[DEBUG] Process termination timeout reached
[INFO] Terminal session closed
```

### Debugging in Code

Add debug prints:

```rust
pub fn read_output(&self, buffer: &mut [u8]) -> Result<usize> {
    let mut reader = self.reader.lock()?;

    // Debug: Print buffer state
    eprintln!("DEBUG: Buffer capacity = {}", buffer.len());

    match reader.read(buffer) {
        Ok(n) => {
            eprintln!("DEBUG: Read {} bytes: {:?}", n, &buffer[..n.min(50)]);
            Ok(n)
        }
        Err(e) => {
            eprintln!("DEBUG: Read error: {}", e);
            Err(anyhow!("Read failed: {}", e))
        }
    }
}
```

### Testing During Development

**Run Tests**:
```bash
cargo test terminal::tests --release -- --nocapture
```

**Run Specific Test**:
```bash
cargo test test_spawn_shell -- --nocapture
```

**Test with Debug Logging**:
```bash
RUST_LOG=debug cargo test test_spawn_shell -- --nocapture
```

### Interactive Testing

Create a debug binary:

```rust
// src/bin/test_terminal.rs
use cco::terminal::TerminalSession;
use std::io::{self, Read};

#[tokio::main]
async fn main() {
    env_logger::init();

    let session = TerminalSession::spawn_shell().unwrap();
    println!("Session: {}", session.session_id());

    // Write test command
    session.write_input(b"echo hello\n").unwrap();

    // Wait and read output
    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut buffer = [0u8; 4096];
    let n = session.read_output(&mut buffer).unwrap();

    println!("Output: {}", String::from_utf8_lossy(&buffer[..n]));

    session.close_session().unwrap();
}
```

Run with:
```bash
cargo run --bin test_terminal
```

## Performance Testing

### Measure Throughput

Test input/output rate:

```rust
#[test]
fn benchmark_input_throughput() {
    let session = TerminalSession::spawn_shell().unwrap();
    let start = Instant::now();

    let data = b"x".repeat(10000);
    let iterations = 100;

    for _ in 0..iterations {
        session.write_input(&data).unwrap();
    }

    let elapsed = start.elapsed();
    let bytes_written = data.len() * iterations;
    let throughput = bytes_written as f64 / elapsed.as_secs_f64();

    println!("Throughput: {:.2} MB/s", throughput / 1_000_000.0);

    session.close_session().unwrap();
}
```

### Profile CPU Usage

```bash
# Run with perf profiler
perf record -g cargo run --release

# View results
perf report
```

### Memory Profiling

```bash
# Run with Valgrind
valgrind --leak-check=full cargo run
```

## Security Considerations

### Input Validation

Currently, no input validation is performed. Consider:

```rust
pub fn validate_input(input: &[u8]) -> Result<()> {
    // Check for NUL bytes that might break shell parsing
    if input.contains(&0) {
        return Err(anyhow!("NUL byte in input"));
    }

    // Validate UTF-8
    std::str::from_utf8(input)?;

    Ok(())
}
```

### Resource Limits

Implement resource controls:

```rust
const MAX_SESSION_MEMORY: u64 = 100 * 1024 * 1024; // 100MB
const MAX_OUTPUT_BUFFER: usize = 1024 * 1024;      // 1MB per read
const PROCESS_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour

pub struct TerminalSession {
    // ... existing fields ...
    created_at: Instant,
    bytes_read: Arc<AtomicU64>,
}

pub fn read_output(&self, buffer: &mut [u8]) -> Result<usize> {
    // Check timeout
    if self.created_at.elapsed() > PROCESS_TIMEOUT {
        return Err(anyhow!("Session timeout"));
    }

    // Check resource limits
    let current_bytes = self.bytes_read.load(Ordering::Relaxed);
    if current_bytes > MAX_SESSION_MEMORY {
        return Err(anyhow!("Memory limit exceeded"));
    }

    // ... normal read logic ...
}
```

### Secure Cleanup

Ensure process cleanup:

```rust
impl Drop for TerminalSession {
    fn drop(&mut self) {
        // Automatically cleanup on drop
        if let Err(e) = self.close_session() {
            warn!("Error during cleanup: {}", e);
        }
    }
}
```

## Code Style and Patterns

### Naming Conventions

```rust
// Constants: SCREAMING_SNAKE_CASE
const DEFAULT_ROWS: u16 = 24;
const POLLING_INTERVAL_MS: u64 = 10;

// Functions: snake_case
pub fn write_input(&self, data: &[u8]) -> Result<usize>

// Types: PascalCase
pub struct TerminalSession

// Variables: snake_case
let session_id = Uuid::new_v4();
```

### Error Handling Pattern

```rust
// Prefer anyhow::Result for library code
pub fn operation() -> Result<String> {
    let value = some_operation()
        .map_err(|e| anyhow!("Operation failed: {}", e))?;
    Ok(value)
}

// Log errors but continue
match session.read_output(&mut buf) {
    Ok(n) => { /* process */ }
    Err(e) => warn!("Read error: {}", e),
}
```

### Async Patterns

```rust
// Use tokio::spawn for background tasks
tokio::spawn(async move {
    loop {
        match some_async_op().await {
            Ok(val) => { /* handle */ }
            Err(e) => {
                error!("Error: {}", e);
                break;
            }
        }
    }
});

// Use tokio::select! for multiple conditions
tokio::select! {
    msg = receiver.next() => { /* handle message */ }
    _ = timeout.tick() => { /* handle timeout */ }
}
```

## Testing Strategy

### Unit Tests

Test individual functions in isolation:

```rust
#[test]
fn test_shell_detection() {
    let shell = detect_shell();
    assert!(shell.is_ok());

    let shell_path = shell.unwrap();
    assert!(std::path::Path::new(&shell_path).exists());
}
```

### Integration Tests

Test component interaction:

```rust
#[tokio::test]
async fn test_shell_interaction() {
    let session = TerminalSession::spawn_shell().unwrap();

    // Send command
    session.write_input(b"echo hello\n").unwrap();

    // Read response
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut buf = [0u8; 1024];
    let n = session.read_output(&mut buf).unwrap();

    assert!(n > 0);
    let output = String::from_utf8_lossy(&buf[..n]);
    assert!(output.contains("hello"));
}
```

### End-to-End Tests

Test full system with WebSocket:

```bash
# Use curl with WebSocket support or dedicated client
cargo build --example ws_client

# Run test suite
./target/debug/examples/ws_client --test-sequence
```

## Adding Tests

### Create Test File

```bash
# Add to cargo test config
# tests/terminal_integration.rs
```

### Write Tests

```rust
#[tokio::test]
async fn test_new_feature() {
    // Setup
    let session = TerminalSession::spawn_shell().unwrap();

    // Test
    let result = session.new_feature();

    // Assert
    assert!(result.is_ok());

    // Cleanup
    let _ = session.close_session();
}
```

### Run Tests

```bash
cargo test terminal       # Run all terminal tests
cargo test -- --nocapture # Show output
cargo test -- --test-threads=1 # Sequential tests
```

## Common Tasks

### Update Buffer Size

In `terminal.rs` and `server.rs`:

```rust
// Before
let mut buffer = [0u8; 4096];

// After (8KB buffer)
let mut buffer = [0u8; 8192];
```

Consider impact on memory and latency.

### Change Polling Interval

In `server.rs` background task:

```rust
// Before (10ms)
let mut read_interval = interval(Duration::from_millis(10));

// After (20ms - less responsive but less CPU)
let mut read_interval = interval(Duration::from_millis(20));
```

### Modify Default Terminal Size

In `terminal.rs`:

```rust
// Before (80x24)
portable_pty::PtySize {
    rows: 24,
    cols: 80,
    // ...
}

// After (100x30)
portable_pty::PtySize {
    rows: 30,
    cols: 100,
    // ...
}
```

## Deployment Considerations

### Environment Variables

Set before running:

```bash
RUST_LOG=cco::terminal=info   # Enable logging
RUST_BACKTRACE=1              # Full backtraces on panic
```

### Resource Planning

Per-session overhead:
- Memory: ~100KB baseline
- File descriptors: 2 (PTY master)
- Threads: 0 (fully async)

For 1000 concurrent sessions:
- ~100MB RAM
- 2000 file descriptors (check system limit with `ulimit -n`)
- No thread overhead

### Monitoring

Monitor in production:

```bash
# Check connections
netstat -an | grep -c ESTABLISHED

# Monitor processes
top -p $(pgrep cco)

# Check file descriptors
lsof -p $(pgrep cco) | wc -l
```

## Future Improvements

1. **Full PTY Resize**: Implement TIOCSWINSZ ioctl
2. **Output Recording**: Session replay capability
3. **Tab Support**: Multiple terminals in one session
4. **Search**: Terminal output search
5. **Mouse Support**: xterm.js mouse handling
6. **Configuration**: Runtime configuration options
7. **Metrics**: Performance monitoring
8. **Authentication**: Per-terminal access control
9. **Rate Limiting**: DoS protection
10. **Clustering**: Multi-server terminal sessions

---

**Last Updated**: November 2025
**Version**: 1.0
**Status**: Documentation Complete
