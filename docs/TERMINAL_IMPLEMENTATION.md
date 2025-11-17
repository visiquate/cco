# Terminal System Implementation Guide

## Overview

This document explains the internal implementation of the PTY terminal system, including code structure, module organization, key functions, and design decisions.

## File Structure

```
cco/src/
├── terminal.rs          # Core PTY session management
├── server.rs            # WebSocket handler integration
├── main.rs              # Application entry point
└── ...
```

## Core Module: `terminal.rs`

### TerminalSession Structure

```rust
pub struct TerminalSession {
    session_id: String,                                    // Unique session UUID
    child: Arc<Mutex<Option<Box<dyn portable_pty::Child>>>>, // Shell process handle
    reader: Arc<Mutex<Box<dyn Read + Send>>>,             // PTY master reader
    writer: Arc<Mutex<Box<dyn Write + Send>>>,            // PTY master writer
}
```

**Design Rationale**:
- `Arc<Mutex<>>`: Thread-safe shared ownership for async handlers
- `Option`: Tracks session state (Some = running, None = closed)
- `dyn Read/Write`: Trait objects for cross-platform PTY abstraction
- `Box`: Heap allocation for trait objects

### Key Functions

#### `TerminalSession::spawn_shell() -> Result<Self>`

Creates a new terminal session with a real shell process.

**Implementation Steps**:

1. **Generate Session ID**
   ```rust
   let session_id = Uuid::new_v4().to_string();
   ```
   - Unique identifier for tracking
   - Used in logging and debugging

2. **Get PTY System**
   ```rust
   let pty_system = native_pty_system();
   ```
   - Platform-specific implementation (Unix/Windows)
   - Handles OS differences transparently

3. **Create PTY Pair**
   ```rust
   let pair = pty_system.openpty(
       portable_pty::PtySize {
           rows: 24,
           cols: 80,
           pixel_width: 0,
           pixel_height: 0,
       }
   )?;
   ```
   - Master: Server-side (reads output, writes input)
   - Slave: Child process (executes shell)
   - Size: 80×24 standard terminal size

4. **Detect Shell**
   ```rust
   let shell = detect_shell()?;
   ```
   - Prefer bash for features and compatibility
   - Fallback to sh for portability
   - Check SHELL environment variable

5. **Build Shell Command**
   ```rust
   let mut cmd = CommandBuilder::new(&shell);
   cmd.env("TERM", "xterm-256color");
   cmd.env("LANG", "en_US.UTF-8");
   // ... copy environment variables
   ```
   - Configure terminal type for color support
   - Preserve HOME, USER, PATH from parent
   - Set UTF-8 encoding

6. **Spawn Child Process**
   ```rust
   let child = pair.slave.spawn_command(cmd)?;
   ```
   - PTY slave becomes shell stdin/stdout/stderr
   - Returns child process handle
   - Shell inherits parent's environment

7. **Create Master I/O**
   ```rust
   let reader = pair.master.try_clone_reader()?;
   let writer = pair.master.try_clone_writer()?;
   ```
   - Independent read and write handles
   - Enables concurrent I/O operations
   - Both operate on same PTY master FD

8. **Return Session**
   - Wrap in Arc<Mutex<>> for thread safety
   - Store reader/writer separately
   - Ready for async operations

**Error Handling**:
- PTY creation fails: System limits or permission issues
- Shell detection fails: No shell found on system
- Process spawn fails: Command not found or permission denied
- Clone fails: PTY doesn't support multiple readers/writers

#### `TerminalSession::write_input(&self, input: &[u8]) -> Result<usize>`

Send keyboard input to the shell process.

**Implementation**:
```rust
pub fn write_input(&self, input: &[u8]) -> Result<usize> {
    let mut writer = self.writer.lock()
        .map_err(|e| anyhow!("Lock error: {}", e))?;

    writer.write_all(input)
        .map_err(|e| anyhow!("Failed to write to shell: {}", e))?;

    writer.flush()
        .map_err(|e| anyhow!("Failed to flush shell input: {}", e))?;

    Ok(input.len())
}
```

**Steps**:
1. Acquire mutex lock on writer
2. Write all bytes to PTY master
3. Flush to ensure immediate delivery
4. Return byte count

**Error Cases**:
- Lock poisoned: Previous thread panicked while holding lock
- Write fails: PTY closed or permission denied
- Flush fails: I/O error on PTY master

**Performance Notes**:
- Mutex lock is brief (< 1μs typically)
- Write/flush calls may block if PTY buffer full
- No data batching (single message = single write call)

#### `TerminalSession::read_output(&self, buffer: &mut [u8]) -> Result<usize>`

Read shell output for sending to client.

**Implementation**:
```rust
pub fn read_output(&self, buffer: &mut [u8]) -> Result<usize> {
    let mut reader = self.reader.lock()
        .map_err(|e| anyhow!("Lock error: {}", e))?;

    match reader.read(buffer) {
        Ok(n) => {
            if n > 0 {
                debug!("Read {} bytes from shell", n);
            }
            Ok(n)
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            Ok(0)  // No data available - normal
        }
        Err(e) => {
            error!("Failed to read from shell: {}", e);
            Err(anyhow!("Failed to read from shell: {}", e))
        }
    }
}
```

**Steps**:
1. Acquire mutex lock on reader
2. Attempt non-blocking read
3. Handle three cases:
   - `Ok(n > 0)`: Data available, return count
   - `Ok(0)`: EOF (shell closed)
   - `WouldBlock`: No data (non-blocking), return 0
   - `Err`: Real I/O error

**Key Design Decision**:
- Non-blocking reads allow polling without blocking
- Polling task calls repeatedly to drain output
- 10ms interval balances responsiveness vs CPU

**Return Values**:
- `n > 0`: Bytes available (1 to buffer.len())
- `n = 0`: No data or EOF
- `Err`: Real I/O error occurred

#### `TerminalSession::set_terminal_size(&self, cols: u16, rows: u16) -> Result<()>`

Notify shell of terminal size change (for responsive apps).

**Current Implementation**:
```rust
pub fn set_terminal_size(&self, cols: u16, rows: u16) -> Result<()> {
    debug!("Resizing terminal to {}x{}", cols, rows);

    let child = self.child.lock()
        .map_err(|e| anyhow!("Lock error: {}", e))?;

    if let Some(ref _child) = *child {
        debug!("Terminal resize requested (cols={}, rows={})", cols, rows);
        Ok(())
    } else {
        Err(anyhow!("Child process not running"))
    }
}
```

**Limitation**:
- Currently logs but doesn't actually resize
- portable-pty doesn't expose PTY resize directly
- Need raw PTY file descriptor for TIOCSWINSZ ioctl
- Shell processes SIGWINCH signal on resize

**Future Implementation**:
```rust
// Would require:
// 1. Access to PTY master file descriptor
// 2. Call libc::ioctl with TIOCSWINSZ
// 3. Send SIGWINCH to child process

use libc::{ioctl, TIOCSWINSZ};

pub fn set_terminal_size(&self, cols: u16, rows: u16) -> Result<()> {
    // Get PTY fd from portable_pty
    let winsize = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        libc::ioctl(pty_fd, TIOCSWINSZ, &winsize);
    }

    // Shell receives SIGWINCH
    // Responsive apps (less, vim, etc.) redraw with new size
    Ok(())
}
```

#### `TerminalSession::close_session(&self) -> Result<()>`

Gracefully terminate the terminal session.

**Implementation**:
```rust
pub fn close_session(&self) -> Result<()> {
    info!("Closing terminal session: {}", self.session_id);

    let mut child = self.child.lock()
        .map_err(|e| anyhow!("Lock error: {}", e))?;

    if let Some(mut child) = child.take() {
        // Kill the process
        child.kill()
            .map_err(|e| anyhow!("Failed to kill shell process: {}", e))?;

        // Wait for process to exit with timeout
        match child.wait_with_deadline(
            std::time::Instant::now() + Duration::from_secs(5)
        ) {
            Ok(_) => {
                info!("Shell process terminated gracefully");
                Ok(())
            }
            Err(e) => {
                warn!("Process termination error (may already be dead): {}", e);
                Ok(())  // Accept termination error
            }
        }
    } else {
        Ok(())  // Already closed
    }
}
```

**Steps**:
1. Acquire exclusive lock on child process
2. Extract child process (Option::take)
3. Send SIGTERM (kill signal)
4. Wait with 5-second timeout
5. Accept timeout or error (process dead anyway)

**Cleanup Behavior**:
- Closes PTY master file descriptor
- Shell inherits process termination
- Any child processes of shell become orphaned (reparented to init)
- Remaining input/output handles drop and close

#### `TerminalSession::is_running(&self) -> Result<bool>`

Check if shell process is still alive.

**Implementation**:
```rust
pub fn is_running(&self) -> Result<bool> {
    let mut child = self.child.lock()
        .map_err(|e| anyhow!("Lock error: {}", e))?;

    if let Some(ref mut child) = *child {
        match child.try_wait() {
            Ok(Some(_)) => Ok(false),  // Exited
            Ok(None) => Ok(true),      // Still running
            Err(_) => Ok(false),       // Error = dead
        }
    } else {
        Ok(false)  // Already closed
    }
}
```

**Return Values**:
- `true`: Shell process actively running
- `false`: Process exited or closed
- `Err`: Unexpected error (rare)

**Non-blocking Check**:
- try_wait() doesn't block on process exit
- Safe to call frequently (keep-alive task)
- Detects dead sessions quickly

### Helper Function: `detect_shell() -> Result<String>`

Determine which shell to use.

**Search Order**:
1. `/bin/bash` - Preferred (features, POSIX compliance)
2. `/bin/sh` - Fallback (maximum compatibility)
3. `SHELL` environment variable - User's configured shell
4. Error - No shell found

**Rationale**:
- Bash preferred for interactive features
- sh fallback for minimal systems
- Environment variable respects user preference
- Explicit paths avoid PATH traversal security issues

## Server Integration: `server.rs`

### WebSocket Endpoint

```rust
async fn terminal_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<ServerState>>
) -> Response {
    ws.on_upgrade(|socket| handle_terminal_socket(socket))
}
```

**Purpose**: HTTP upgrade handler
- Accepts WebSocket upgrade requests
- Delegates to connection handler

### Connection Handler

```rust
async fn handle_terminal_socket(socket: WebSocket) {
    // 1. Spawn shell
    // 2. Initialize I/O
    // 3. Start background output task
    // 4. Handle client input loop
    // 5. Cleanup on disconnect
}
```

**Flow**:
```
1. Spawn shell process
   ├─ Creates TerminalSession
   └─ Returns on error

2. Send initial output
   ├─ Read any greeting output
   └─ Send to client

3. Start output task
   ├─ tokio::spawn background task
   ├─ Polls read_output() every 10ms
   ├─ Sends via WebSocket
   └─ Checks keep-alive every 30s

4. Main input loop
   ├─ while let Some(msg) = receiver.next().await
   ├─ Match message type (Binary/Text)
   ├─ Call session.write_input() or set_terminal_size()
   └─ Break on error or close

5. Cleanup
   ├─ Abort background task
   ├─ Call session.close_session()
   ├─ Log closure
   └─ Drop resources
```

### Message Handling

**Binary Messages** (Input):
```rust
Ok(Message::Binary(data)) => {
    match session.write_input(&data) {
        Ok(n) => debug!("Received {} bytes from client", n),
        Err(e) => {
            error!("Failed to write to shell: {}", e);
            break;
        }
    }
}
```

**Text Messages** (Control):
```rust
Ok(Message::Text(text)) => {
    if text.starts_with("\x1b[RESIZE;") {
        // Parse and handle resize
        if let Some(rest) = text.strip_prefix("\x1b[RESIZE;") {
            let parts: Vec<&str> = rest.split(';').collect();
            if parts.len() >= 2 {
                if let (Ok(cols), Ok(rows)) = (
                    parts[0].parse::<u16>(),
                    parts[1].trim().parse::<u16>(),
                ) {
                    session.set_terminal_size(cols, rows)?;
                }
            }
        }
    } else {
        // Treat as text input
        session.write_input(text.as_bytes())?;
    }
}
```

**Close Messages**:
```rust
Ok(Message::Close(_)) => {
    debug!("Client closed WebSocket connection");
    break;  // Exit input loop
}
```

## Dependencies

### portable-pty

**Purpose**: Cross-platform PTY abstraction
**Key Types**:
- `PtySystem`: Factory for creating PTY pairs
- `PtyPair`: Master + Slave file descriptors
- `PtySize`: Terminal dimensions
- `CommandBuilder`: Shell command configuration
- `Child`: Running process handle

**Key Methods**:
- `native_pty_system()`: Get platform-specific implementation
- `openpty(size)`: Create master-slave pair
- `spawn_command(cmd)`: Execute command in slave
- `try_clone_reader()`: Get independent reader
- `try_clone_writer()`: Get independent writer
- `try_wait()`: Non-blocking process status check
- `wait_with_deadline()`: Blocking wait with timeout

### tokio

**Components Used**:
- `tokio::spawn()`: Background task creation
- `tokio::select!()`: Multi-condition await
- `tokio::time::interval()`: Periodic task scheduler
- `Duration`: Timeout specification

### axum

**Components Used**:
- `WebSocket`: Protocol handling
- `Message::Binary/Text/Close`: Message types
- `WebSocketUpgrade`: HTTP upgrade handler
- `Stream/Sink` traits: Async message I/O

## Error Handling Strategy

### Lock Errors

Occur when mutex is poisoned (previous holder panicked).

```rust
let mut child = self.child.lock()
    .map_err(|e| anyhow!("Lock error: {}", e))?;
```

**Recovery**:
- Log error
- Return to caller
- Caller typically aborts session

### I/O Errors

PTY read/write operations fail.

```rust
Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
    Ok(0)  // Expected in non-blocking mode
}
Err(e) => {
    error!("Failed to read from shell: {}", e);
    Err(anyhow!("..."))
}
```

**Common Cases**:
- `WouldBlock`: Normal, no data available
- `BrokenPipe`: Shell exited
- `PermissionDenied`: Permission issues
- `Interrupted`: System signal (retry)

### Process Errors

Child process operations fail.

```rust
child.kill().map_err(|e| anyhow!("Failed to kill: {}", e))?;
match child.wait_with_deadline(...) {
    Ok(_) => { /* success */ },
    Err(e) => {
        warn!("Termination error (may already be dead): {}", e);
        Ok(())  // Accept - process is dead anyway
    }
}
```

## Testing

### Unit Tests

Located at bottom of `terminal.rs`:

```rust
#[tokio::test]
async fn test_spawn_shell() { }

#[tokio::test]
async fn test_write_input() { }

#[tokio::test]
async fn test_read_output() { }

#[tokio::test]
async fn test_terminal_size() { }

#[tokio::test]
async fn test_close_session() { }
```

**Test Patterns**:
- Each test is async (tokio::test)
- Spawns shell, performs operation, cleanup
- Assertions on success/failure

## Performance Optimizations

### Current

1. **Non-blocking I/O**: No thread blocking
2. **Async Runtime**: Tokio handles concurrency
3. **Minimal Buffering**: 4KB read at a time
4. **Efficient Polling**: 10ms interval, tokio::select!

### Potential

1. **Output Batching**: Combine multiple reads before sending
2. **Input Coalescing**: Batch client input messages
3. **Lazy Session Creation**: Create PTY only when needed
4. **Connection Pooling**: Reuse shell sessions
5. **Memory Pools**: Preallocate buffers

## Platform Considerations

### Unix/Linux

- Standard POSIX PTY interface
- `portable-pty` wraps ioctl calls
- Signal handling via kernel

### macOS

- Similar to Linux
- BSD PTY API slightly different
- `portable-pty` abstracts differences

### Windows

- Windows Pseudoconsole API
- Different process/pipe model
- `portable-pty` provides compatibility layer

## Security Notes

### Input

- No validation or sanitization
- Raw bytes passed to shell
- User responsible for safe commands

### Output

- Raw terminal output including control codes
- No filtering or escaping
- Assume trusted local user

### Process

- Runs as current user
- Inherits environment
- No privilege escalation

### Network

- Use WSS (secure WebSocket) in production
- Authenticate at HTTP layer
- Encryption in transit

---

**Last Updated**: November 2025
**Version**: 1.0
**Status**: Documentation Complete
