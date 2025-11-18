# TUI/Daemon Startup Failure - Root Cause Analysis & Fix

## Problem Statement

When running `cco` without arguments (expecting TUI/daemon mode), the application crashed with:
```
Failed to start TUI: Failed to enable raw mode
Falling back to daemon mode...
🚀 Starting Claude Code Orchestra 2025.11.3+472efbd...
🌐 Server running at http://127.0.0.1:3000
   Open this URL in your browser to access the dashboard

thread 'main' (13181068) panicked at src/server.rs:1819:20:
Registering a blocking socket with the tokio runtime is unsupported.
If you wish to do anyways, please add `--cfg tokio_allow_from_blocking_fd` to your RUSTFLAGS.
```

The TUI tried to start but failed (expected in non-terminal environments), then fell back to daemon mode, but the daemon crashed immediately during startup.

## Root Cause

Located in `/Users/brent/git/cc-orchestra/cco/src/server.rs:1819`:

The code was creating a standard library `TcpListener` (blocking socket) and then attempting to convert it to a tokio async `TcpListener` without setting it to non-blocking mode first:

```rust
let std_listener = StdTcpListener::bind(&addr)?;

// ... set SO_REUSEADDR socket option ...

// BUG: std_listener is still in BLOCKING mode at this point
let listener = TcpListener::from_std(std_listener)?;
```

**Why this fails:**
- `StdTcpListener::bind()` creates a blocking socket by default
- Tokio's `TcpListener::from_std()` requires the socket to be in **non-blocking** mode
- Attempting to register a blocking socket with the tokio runtime causes an immediate panic
- This is a safety mechanism in tokio to prevent unexpected blocking calls in async code

## The Fix

Added a single line to set the socket to non-blocking mode before converting it to tokio:

**File:** `/Users/brent/git/cc-orchestra/cco/src/server.rs`

**Change at line 1819-1823:**
```rust
// Set non-blocking mode BEFORE converting to tokio TcpListener
// This is required because tokio's from_std() requires the socket to be non-blocking
std_listener.set_nonblocking(true)?;

let listener = TcpListener::from_std(std_listener)?;
```

This ensures:
1. Socket is created with SO_REUSEADDR option (for port reuse)
2. Socket is set to non-blocking mode (required by tokio)
3. Socket is converted to async tokio TcpListener (now valid operation)
4. Server starts successfully

## Verification

**Before the fix:**
```
Process exited immediately with panic about blocking socket
Port 3000 not bound
No API endpoints accessible
```

**After the fix:**
```
✅ Process stays running
✅ Port 3000 is bound and listening
✅ /health endpoint returns: {"status": "ok", "version": "2025.11.3+472efbd", ...}
✅ /ready endpoint returns: {"ready": true, "timestamp": "...", ...}
✅ /api/agents endpoint returns: 117 agents as JSON array
✅ Graceful shutdown on Ctrl+C
```

## Testing Performed

1. **Daemon Direct Launch**: `./target/debug/cco run`
   - Result: Server starts successfully
   - Port 3000 binds without error
   - All API endpoints respond correctly

2. **No-Argument Launch (Fallback to Daemon)**:
   - TUI fails (expected - no terminal)
   - Falls back to daemon mode
   - Daemon now starts successfully (previously crashed)

3. **Health Endpoint Testing**:
   ```bash
   curl http://127.0.0.1:3000/health
   # Returns valid JSON with status "ok"
   ```

4. **Ready Endpoint Testing**:
   ```bash
   curl http://127.0.0.1:3000/ready
   # Returns valid JSON with ready=true
   ```

5. **Agent List Endpoint Testing**:
   ```bash
   curl http://127.0.0.1:3000/api/agents | jq '. | length'
   # Returns: 117
   ```

## Impact

- **Severity**: Critical - Blocks any daemon startup
- **Scope**: Affects all daemon launches (TUI fallback, direct run, systemd service)
- **Platform**: macOS, Linux, Unix systems (Windows may handle this differently)
- **Risk of Fix**: Zero - standard socket configuration practice

## Code Review Notes

The root cause is a common mistake when bridging std library blocking I/O with tokio async I/O. The fix follows tokio best practices:

1. Create socket with desired options (SO_REUSEADDR)
2. Set to non-blocking mode
3. Convert to tokio async wrapper

This is the standard pattern documented in tokio's from_std() function.

## Related Files Modified

- `/Users/brent/git/cc-orchestra/cco/src/server.rs` - Line 1819-1823

## Recommendation for Prevention

1. Add integration tests that verify daemon startup without TUI
2. Add CI check for blocking socket registration errors
3. Document the non-blocking mode requirement in code comments (already added)
4. Consider using tokio's TcpListener::bind() directly instead of converting from std (future optimization)
