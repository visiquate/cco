# Terminal System Architecture

## Overview

The Claude Orchestra PTY Terminal System provides a full terminal emulation experience within the web browser. It enables users to execute shell commands, interact with running processes, and manage file systems through a unified WebSocket-based interface.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Browser/Client Layer                          │
│  (xterm.js - Terminal Emulation + Input/Output Rendering)       │
│  - Character rendering                                           │
│  - Keyboard input capture                                        │
│  - ANSI escape code processing                                   │
│  - Window resize events                                          │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                    WebSocket Connection
                  (Binary & Text Messages)
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                    CCO Server Layer                              │
│  (Axum Web Framework + Tokio Async Runtime)                     │
│ - Terminal WebSocket Handler                                     │
│ - Message Serialization/Deserialization                          │
│ - PTY Session Management                                         │
│ - Bidirectional I/O Multiplexing                                 │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                      PTY Bridge
            (portable-pty Library Interface)
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                   PTY Layer (Kernel)                             │
│  (Pseudoterminal Master-Slave Pair)                             │
│ - Real Shell Process (bash/sh)                                   │
│ - Terminal Control Signals (SIGWINCH, SIGTERM)                  │
│ - Raw I/O with Canonical Processing                             │
│ - Process Group Management                                       │
└──────────────────────────────────────────────────────────────────┘
```

## Component Interaction

### 1. Client-Side Terminal (xterm.js)

The browser-based terminal emulator:
- Renders characters and ANSI escape codes
- Captures keyboard input and sends to server
- Displays server output in real-time
- Handles window resizing and sends to server

### 2. WebSocket Transport Layer

Bidirectional communication between client and server:
- **Protocol**: WebSocket (ws/wss)
- **Endpoint**: `/terminal`
- **Message Types**: Binary (raw shell output/input) and Text (control messages)

### 3. CCO Server Handler

Axum-based WebSocket handler in `server.rs`:
- Accepts WebSocket upgrade requests
- Creates and manages PTY sessions
- Multiplexes bidirectional I/O
- Handles client disconnect and cleanup

### 4. Terminal Session Manager

Rust implementation in `terminal.rs`:
- PTY lifecycle management
- Shell process spawning with proper environment
- Non-blocking I/O with async support
- Terminal control (resize, close, etc.)

### 5. Kernel PTY Interface

Operating system pseudoterminal:
- Real shell process execution
- Signal handling (SIGWINCH for resize, SIGTERM for termination)
- Raw terminal mode with proper line discipline
- Master-slave file descriptor pair

## Data Flow

### User Input Flow

```
User Types in Browser
         │
         ▼
xterm.js Captures Keystroke
         │
         ▼
WebSocket Message Sent (Binary: UTF-8 bytes)
         │
         ▼
CCO Server Receives Message
         │
         ▼
Terminal Session Handler
         │
         ▼
TerminalSession::write_input(bytes)
         │
         ▼
PTY Master Writer (writes to slave stdin)
         │
         ▼
Shell Process Receives Input
         │
         ▼
Shell Processes Command
```

### Output Flow

```
Shell Process Generates Output
         │
         ▼
PTY Slave stdout/stderr
         │
         ▼
PTY Master Reader (reads from master)
         │
         ▼
TerminalSession::read_output(buffer)
         │
         ▼
CCO Server Background Task (30ms polling)
         │
         ▼
WebSocket Message Sent (Binary: UTF-8 bytes)
         │
         ▼
Browser Receives Message
         │
         ▼
xterm.js Processes ANSI Codes
         │
         ▼
Terminal Rendered to Screen
```

### Terminal Resize Flow

```
Browser Window Resized / Terminal Zoom Changed
         │
         ▼
xterm.js Calculates New Dimensions
         │
         ▼
WebSocket Message Sent (Text: \x1b[RESIZE;COLS;ROWS)
         │
         ▼
CCO Server Parses Resize Message
         │
         ▼
TerminalSession::set_terminal_size(cols, rows)
         │
         ▼
PTY Master Ioctl TIOCSWINSZ
         │
         ▼
Kernel Sends SIGWINCH to Shell
         │
         ▼
Shell Receives Signal (e.g., `stty -a` reflects new size)
         │
         ▼
Affected Programs Adapt to New Size
```

## Binary Protocol Specification

### Message Format

The terminal system uses a simple, efficient binary protocol for raw I/O:

#### Input Message (Client → Server)

```
Message Type: Binary
Content: Raw UTF-8 bytes
Example: "ls -la\n" → [0x6c, 0x73, 0x20, 0x2d, 0x6c, 0x61, 0x0a]
```

#### Output Message (Server → Client)

```
Message Type: Binary
Content: Raw UTF-8 bytes
Example: Shell output bytes
Note: May contain ANSI escape codes for colors, styles
```

#### Resize Message (Client → Server)

```
Message Type: Text
Format: \x1b[RESIZE;COLS;ROWS
Example: "\x1b[RESIZE;120;40" (120 columns, 40 rows)
Parsing: Split on ';', parse COLS and ROWS as u16
```

#### Keep-Alive Message (Server → Client)

```
Message Type: Binary (empty) or Text (empty)
Interval: 30 seconds
Purpose: Prevent connection timeout, verify session health
```

### Message Flow Sequence

```
1. Client → Server: Binary message with input (bytes)
   ├─ Server receives in handle_terminal_socket
   ├─ Calls session.write_input(&data)
   ├─ PTY master writer flushes to shell
   └─ Logger records "Received N bytes from client"

2. Server → Client: Binary message with output (bytes)
   ├─ Background task polls read_output() every 10ms
   ├─ When n > 0 bytes available
   ├─ Wraps in WebSocket binary message
   └─ Sends via sender.send(msg)

3. Client → Server: Text resize message
   ├─ xterm.js detects terminal size change
   ├─ Sends "\x1b[RESIZE;COLS;ROWS"
   ├─ Server parses format
   ├─ Calls session.set_terminal_size(cols, rows)
   └─ Kernel sends SIGWINCH to shell

4. Server → Client: Keep-alive empty message
   ├─ Every 30 seconds
   ├─ Verifies session.is_running()
   ├─ Detects closed sessions
   └─ Maintains connection health
```

## Security Architecture

### Process Isolation

- **Shell User**: Inherits from server process user (typically application user)
- **No Root Escalation**: Terminal operations run with current user privileges
- **Process Containment**: Each terminal session is independent child process
- **Graceful Shutdown**: Sessions killed on disconnect or timeout

### Input Validation

- **No Command Injection**: Raw bytes passed directly to shell (user responsibility)
- **UTF-8 Handling**: Client must send valid UTF-8 sequences
- **Buffer Boundaries**: 4KB input/output buffers prevent memory exhaustion
- **Message Limits**: WebSocket frame limits enforced by framework

### Output Security

- **ANSI Escape Codes**: Raw output includes terminal control sequences
- **No Sanitization**: Terminal assumes trusted local user
- **Sensitive Data**: Server logs disabled for terminal data by default
- **Network Transport**: Use WSS (WebSocket Secure) in production

### Authentication & Authorization

- **Session-based**: Terminal access tied to authenticated user session
- **No Built-in ACL**: Relies on server-level authentication middleware
- **User Context**: Shell inherits authenticated user's environment
- **Audit Logging**: Terminal session creation/closure logged with timestamp

### Resource Limits

- **Memory**: 4KB buffers per session, minimal memory overhead
- **CPU**: Non-blocking async I/O prevents CPU spinning
- **Process Limits**: Shell and children bound by system ulimits
- **Connection Limits**: WebSocket server enforces per-connection limits
- **Timeout**: Sessions kept alive by 30s keep-alive, detected dead sessions
- **No Output Buffering**: Real-time streaming prevents memory accumulation

## Performance Considerations

### Polling Strategy

```
Output Reading:
├─ Interval: 10ms (100 Hz sampling rate)
├─ Rationale: Balances responsiveness vs CPU usage
├─ Trade-off: ~100ms max latency, < 1% CPU per session
└─ Alternative: 50ms for interactive, 100ms for batch

Keep-Alive:
├─ Interval: 30 seconds
├─ Purpose: Detect stale connections, prevent proxy timeout
├─ Trade-off: Minimal overhead, reliable connection health
└─ Alternative: 60s for lower traffic servers
```

### Buffer Management

```
Per-Session Buffers:
├─ Input Buffer: 4KB (write_input parameter)
├─ Output Buffer: 4KB (read_output parameter)
├─ Rationale: Minimize memory per session
├─ Limitation: 4KB max per read (solution: loop until no data)
└─ Trade-off: Memory efficiency vs CPU for multiple reads

Total Memory Impact:
├─ Idle Session: ~100KB (PTY overhead + Rust structures)
├─ Active Session: ~100KB + buffer contents
├─ 1000 Sessions: ~100MB baseline
└─ Scaling: Linear with session count
```

### I/O Optimization

```
Async Execution:
├─ tokio::spawn for background task
├─ Non-blocking reads/writes
├─ tokio::select! for timeout handling
├─ Minimal context switching
└─ No thread overhead per session

Batching Opportunities:
├─ Read multiple times per polling interval if backlog
├─ Write accumulated input in single flush
├─ Send combined output frames
└─ Not currently implemented (premature optimization)
```

### Latency Profile

```
Keyboard Input → Display Output:
├─ xterm.js capture: < 1ms
├─ WebSocket transmission: 5-50ms (network dependent)
├─ Server processing: < 1ms
├─ PTY write: < 1ms
├─ Shell processing: variable (10-1000ms)
├─ PTY read: < 1ms
├─ WebSocket return: 5-50ms
├─ Browser rendering: < 1ms
└─ Total: 11-1100ms (shell dependent)

For responsive shells (local execution):
├─ Typical: 15-30ms round-trip
├─ 60 FPS rendering at 10ms polling: smooth
├─ Network latency dominates: 5-50ms per direction
```

## Environment Variables

The terminal system respects and propagates standard environment variables:

- **TERM**: Set to `xterm-256color` (color support)
- **LANG**: Set to `en_US.UTF-8` (UTF-8 encoding)
- **HOME**: Inherited from server process
- **USER**: Inherited from server process
- **PATH**: Inherited from server process
- **Shell-Specific**: Bash/sh specific variables preserved

## Configuration Options

### Runtime Configuration

- **Default Terminal Size**: 80 columns × 24 rows
- **Output Polling Interval**: 10ms (configurable)
- **Keep-Alive Interval**: 30 seconds (configurable)
- **Process Kill Timeout**: 5 seconds after kill signal
- **Shell Selection**: Auto-detect bash, fallback to sh

### Compile-Time Constants

Currently hardcoded, could be made configurable:
- **I/O Buffer Size**: 4096 bytes
- **Default Columns**: 80
- **Default Rows**: 24
- **Poll Timeout**: 10 milliseconds
- **Keep-alive Timeout**: 30 seconds
- **Process Termination Timeout**: 5 seconds

## Future Enhancements

1. **Terminal Multiplexing**: Multiple shells in single session
2. **History Recording**: Session playback capability
3. **File Transfer**: Native file upload/download
4. **Tab Support**: Multiple terminal tabs
5. **Search**: Terminal output searching
6. **Copy/Paste**: Enhanced clipboard operations
7. **Mouse Support**: xterm.js mouse event handling
8. **Configuration UI**: Runtime configuration panel
9. **Monitoring**: Resource usage per session
10. **Authentication**: Per-terminal access control

## Dependencies

- **portable-pty**: Cross-platform PTY abstraction
  - Version: Latest stable
  - License: MIT
  - Purpose: PTY lifecycle management

- **tokio**: Async runtime
  - Version: Latest stable
  - License: MIT
  - Purpose: Async I/O and task spawning

- **axum**: Web framework
  - Version: Latest stable
  - License: MIT
  - Purpose: WebSocket handling and HTTP routes

- **futures**: Async utilities
  - Version: Latest stable
  - License: MIT/Apache-2.0
  - Purpose: Stream composition and async iteration

## Debugging

### Enable Debug Logging

Set environment variable:
```bash
RUST_LOG=debug cargo run
# or
export RUST_LOG=cco::terminal=debug
```

### Log Output Examples

```
[2025-11-15T10:30:45.123Z DEBUG cco::terminal] Creating terminal session: a1b2c3d4-e5f6-g7h8-i9j0-k1l2m3n4o5p6
[2025-11-15T10:30:45.124Z DEBUG cco::terminal] Using shell: /bin/bash
[2025-11-15T10:30:45.126Z INFO cco::terminal] Shell process spawned for session: a1b2c3d4-e5f6-g7h8-i9j0-k1l2m3n4o5p6
[2025-11-15T10:30:46.000Z DEBUG cco::server] Read 47 bytes from shell
[2025-11-15T10:30:46.001Z DEBUG cco::server] Sending 47 bytes to client
[2025-11-15T10:30:46.500Z DEBUG cco::terminal] Resizing terminal to 120x30
```

### Common Issues

1. **No Output**: Check polling interval, PTY reader permissions
2. **Delayed Output**: Network latency, polling interval too long
3. **Process Hangs**: Shell waiting for input, timeout not triggered
4. **Memory Leak**: Check session cleanup in close_session()
5. **Broken Pipe**: PTY closed unexpectedly, check error logs

---

**Last Updated**: November 2025
**Status**: Production Ready
**Stability**: Stable
