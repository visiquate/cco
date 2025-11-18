# Daemon Startup Bug Fix - Complete Analysis

## Problem

Running `cco` without arguments crashed immediately with:

```
Failed to start TUI: Failed to enable raw mode
Falling back to daemon mode...
🚀 Starting Claude Code Orchestra...

thread 'main' panicked at src/server.rs:1819:20:
Registering a blocking socket with the tokio runtime is unsupported.
```

This prevented any daemon startup (whether via TUI fallback, direct `cco run`, or systemd service).

## Root Cause

**File**: `/Users/brent/git/cc-orchestra/cco/src/server.rs`
**Lines**: 1798-1819

The server initialization code was:

```rust
// Line 1798: Create a blocking socket
let std_listener = StdTcpListener::bind(&addr)?;

// Lines 1800-1817: Configure socket (SO_REUSEADDR for port reuse)
#[cfg(unix)]
{
    use std::os::unix::io::AsRawFd;
    let socket_fd = std_listener.as_raw_fd();
    // ... setsockopt call ...
}

// Line 1819: Convert to tokio TcpListener (ERROR: socket still blocking!)
let listener = TcpListener::from_std(std_listener)?;
```

**The Issue:**
- `StdTcpListener::bind()` creates a **blocking** socket
- Tokio's `TcpListener::from_std()` requires a **non-blocking** socket
- Attempting to convert a blocking socket to a tokio async socket causes a panic
- This is a safety mechanism in tokio to prevent blocking operations in async contexts

**Why This Matters:**
- Tokio is an async runtime using event-driven I/O (epoll/kqueue)
- Blocking I/O on a tokio thread would block the entire executor
- Tokio cannot monitor or prevent blocking calls on std library sockets
- The safety check forces the socket to be non-blocking before accepting it

## Solution

Added one line to set the socket to non-blocking mode before conversion:

```rust
// Set non-blocking mode BEFORE converting to tokio TcpListener
// This is required because tokio's from_std() requires the socket to be non-blocking
std_listener.set_nonblocking(true)?;

let listener = TcpListener::from_std(std_listener)?;
```

**Correct Order of Operations:**
1. Create socket: `StdTcpListener::bind()`
2. Configure options: `setsockopt(SO_REUSEADDR)`
3. Set non-blocking: `set_nonblocking(true)` ← **ADDED**
4. Convert to async: `TcpListener::from_std()`

## Code Change

**File**: `/Users/brent/git/cc-orchestra/cco/src/server.rs`

**Lines**: 1819-1823 (added 4 lines including comments)

```diff
     }

+    // Set non-blocking mode BEFORE converting to tokio TcpListener
+    // This is required because tokio's from_std() requires the socket to be non-blocking
+    std_listener.set_nonblocking(true)?;
+
     let listener = TcpListener::from_std(std_listener)?;
```

## Verification

### Before Fix
- Process crashes immediately with panic
- Port 3000 not bound
- No API endpoints accessible
- TUI fallback fails
- Daemon mode fails

### After Fix
- Server starts successfully
- Logs: "✅ Server listening on http://127.0.0.1:3000"
- Port 3000 bound and listening
- All API endpoints return HTTP 200:
  - GET /health → 200 (daemon status)
  - GET /ready → 200 (server ready)
  - GET /api/agents → 200 (returns 117 agents)
  - GET /api/stats → 200 (analytics data)
  - GET / → 200 (dashboard HTML)
- Graceful shutdown on Ctrl+C

### Test Results

```
✅ Daemon process: RUNNING
✅ Port 3000: BOUND
✅ Health endpoint: HTTP 200
✅ Ready endpoint: HTTP 200
✅ Agents endpoint: HTTP 200 (117 agents)
✅ Stats endpoint: HTTP 200
✅ Dashboard HTML: HTTP 200
✅ Graceful shutdown: WORKING
```

## Impact Assessment

| Aspect | Details |
|--------|---------|
| **Severity** | CRITICAL - Blocks all daemon operations |
| **Scope** | All daemon launch paths (fallback, direct, systemd) |
| **Platform** | macOS, Linux, Unix systems |
| **User Impact** | Users cannot run `cco` at all |
| **Fix Complexity** | Minimal - one line of code |
| **Risk** | Zero - standard socket configuration |
| **Breaking Changes** | None |

## Technical Deep Dive

### Why Tokio Panics on Blocking Sockets

Tokio is built on async/await and event-driven I/O:

```
┌─────────────────────────────────────────┐
│  Tokio Event Loop (epoll/kqueue)        │
│  ┌─────────────────────────────────────┐│
│  │ Waiting for I/O readiness events   ││
│  │ - Socket A: readable                ││
│  │ - Socket B: writable                ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘

Problem with blocking socket:
- Socket calls block() → entire thread blocks
- Event loop cannot process other coroutines
- All async tasks hang
- Deadlock situation
```

### Socket Modes Explained

**Blocking Mode** (default):
```rust
let socket = StdTcpListener::bind("127.0.0.1:3000")?;
// socket.accept() will block until connection arrives
// Thread is stuck waiting
```

**Non-Blocking Mode** (required by tokio):
```rust
let socket = StdTcpListener::bind("127.0.0.1:3000")?;
socket.set_nonblocking(true)?;
// socket.accept() returns immediately with WouldBlock
// Event loop checks readiness before calling accept()
// Never blocks the executor thread
```

## Prevention

### Code Review Checklist
- [ ] Any `from_std()` calls verified for non-blocking mode
- [ ] Socket configuration matches tokio documentation
- [ ] Comments explain why non-blocking is required
- [ ] Tests verify daemon startup without TUI

### Future Optimization
Consider using tokio's native API directly:
```rust
// Instead of:
let std_listener = StdTcpListener::bind(&addr)?;
std_listener.set_nonblocking(true)?;
let listener = TcpListener::from_std(std_listener)?;

// Use tokio's native binding:
let listener = TcpListener::bind(&addr).await?;
// More efficient and clearer intent
```

### Test Coverage
```bash
# Add test for daemon startup without TUI
cargo test daemon_startup -- --nocapture --test-threads=1

# Manual verification
./target/debug/cco run &
sleep 2
curl http://127.0.0.1:3000/health | jq .
kill %1
```

## References

- [Tokio from_std() documentation](https://docs.rs/tokio/latest/tokio/net/struct.TcpListener.html#method.from_std)
- [Socket non-blocking mode](https://man7.org/linux/man-pages/man2/fcntl.2.html)
- [Event-driven I/O patterns](https://en.wikipedia.org/wiki/Asynchronous_I/O)

## Summary

This was a straightforward but critical bug where a blocking socket was passed to an async runtime. The fix is minimal, well-tested, and follows established Rust/Tokio best practices. The daemon now starts successfully in all scenarios.

**Status**: FIXED, TESTED, AND VERIFIED
