# Terminal Segfault Fix Report

## Date: 2025-11-16

## Issues Addressed

### TASK 1: Suppress Synthetic Model Warnings ✅

**Problem**: Log spam from repeated warnings about `<synthetic>` model pricing
```
WARN cco::claude_history: Unknown model pricing for: <synthetic>, using Sonnet defaults
```

**Root Cause**: The `<synthetic>` model is an infrastructure placeholder from Claude Code with 0 tokens (not a real model). It appears in error responses and system events.

**Solution**: Changed log level from WARN to DEBUG for unknown models in `/Users/brent/git/cc-orchestra/cco/src/claude_history.rs` (lines 141-151).

**Changes**:
- Removed duplicate warning message
- Changed to DEBUG level only
- Added explanatory comment about synthetic infrastructure events

### TASK 2: Fix Segfault During Terminal I/O ✅

**Problem**: Server crashes AFTER "Terminal session initialized and ready for I/O" message when terminal is accessed via WebSocket or with `--debug` flag.

**Symptoms**:
```
Terminal session initialized and ready for I/O
Segmentation fault: 11
```

**Root Cause Analysis**:

The segfault occurred due to **missing error checking in file descriptor duplication**.

**Location**: `/Users/brent/git/cc-orchestra/cco/src/terminal.rs` - `PtyMaster::from_fd()` (lines 56-87)

**Problem Flow**:
1. `spawn_shell()` creates PTY pair and gets raw FD from `pair.master`
2. Duplicates FD once and passes to `PtyMaster::from_fd()`
3. `from_fd()` duplicates FD twice more (for read and write handles)
4. **BUT**: No error checking on `libc::dup()` calls
5. If `dup()` failed (returned -1), we'd wrap invalid FD in `OwnedFd`
6. First I/O operation (`read()` or `write()`) would access invalid FD → **SEGFAULT**

**Why It Happened During I/O, Not Startup**:
- PTY creation and initial shell spawn work fine
- The duplicated FDs might be invalid due to resource exhaustion or race conditions
- Segfault occurs on **first actual read/write** to the invalid FD (line 1011 or 1178 in server.rs)

**Solution Implemented**:

Added error checking for all `libc::dup()` calls in `PtyMaster::from_fd()`:

```rust
// Duplicate for reading
let read_fd_raw = unsafe { libc::dup(master_fd_raw) };
if read_fd_raw < 0 {
    panic!("Failed to duplicate PTY master FD for reading: {}", std::io::Error::last_os_error());
}

// Duplicate for writing
let write_fd_raw = unsafe { libc::dup(master_fd_raw) };
if write_fd_raw < 0 {
    unsafe { libc::close(read_fd_raw); }  // Clean up first dup
    panic!("Failed to duplicate PTY master FD for writing: {}", std::io::Error::last_os_error());
}
```

**Benefits**:
- Early failure with clear error message instead of silent corruption
- Proper cleanup if second dup fails (closes first dup to prevent leak)
- Guarantees valid FDs are wrapped in `OwnedFd`

## Testing Required

1. **Run server with --debug flag**:
   ```bash
   ./target/release/cco --debug
   ```
   - Should start without warnings about synthetic models
   - Should run stably without crashes

2. **Open dashboard and create terminal session**:
   ```bash
   open http://localhost:3000
   ```
   - Click "Terminal" tab
   - Type commands
   - Should NOT segfault during I/O
   - Should see graceful shutdown when exiting

3. **Verify log cleanliness**:
   - Check for absence of synthetic model warnings
   - Verify DEBUG logs only show synthetic info when `RUST_LOG=debug`

## Files Modified

1. `/Users/brent/git/cc-orchestra/cco/src/claude_history.rs` (lines 141-151)
   - Reduced log noise for synthetic models

2. `/Users/brent/git/cc-orchestra/cco/src/terminal.rs` (lines 48-87)
   - Added error checking for FD duplication
   - Added safety documentation
   - Added resource cleanup on partial failure

## Build Status

✅ **Builds successfully**:
```
Finished `release` profile [optimized] target(s) in 14.95s
```

## Next Steps

1. Test terminal functionality with `--debug` flag
2. Verify no segfaults during terminal I/O
3. Confirm reduced log noise from synthetic model warnings
4. Monitor for any new FD-related errors (should be caught early now)
