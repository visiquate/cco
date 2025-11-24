# Test Verification Report - Debug Mode & Shutdown Functionality
**Date**: November 16, 2025
**CCO Version**: 2025.11.2
**Status**: PARTIALLY COMPLETE

---

## TEST 1: Debug Mode Implementation

### Finding: INCOMPLETE - Debug flag not yet implemented

#### Check 1: Help Output
- **Command**: `/Users/brent/.cargo/bin/cco run --help`
- **Result**: ❌ FAILED - `--debug` flag NOT found in help
- **Current flags available**:
  - `-p, --port <PORT>` (default: 3000)
  - `--host <HOST>` (default: 127.0.0.1)
  - `--database-url <DATABASE_URL>` (default: sqlite://analytics.db)
  - `--cache-size <CACHE_SIZE>` (default: 1073741824)
  - `--cache-ttl <CACHE_TTL>` (default: 3600)

#### Check 2: Source Code Verification
- **File**: `/Users/brent/git/cc-orchestra/cco/src/main.rs`
- **Finding**: No `debug` or `Debug` field in the `Run` command struct (lines 28-48)
- **Current `Run` command struct**:
  ```rust
  Run {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Database URL
    #[arg(long, default_value = "sqlite://analytics.db")]
    database_url: String,

    /// Cache size in bytes
    #[arg(long, default_value = "1073741824")]
    cache_size: u64,

    /// Cache TTL in seconds
    #[arg(long, default_value = "3600")]
    cache_ttl: u64,
  }
  ```

#### Check 3: Normal Mode Output
- **Command**: `NO_BROWSER=1 /Users/brent/.cargo/bin/cco run --port 3001`
- **Output**:
  ```
  🚀 Starting Claude Code Orchestra 2025.11.2...
  🌐 Server running at http://127.0.0.1:3001
  ```
- **Logging Level**: Minimal (only startup messages)
- **Observation**: Logging is configured via `tracing_subscriber::fmt::init()` in main.rs (line 167)

#### Check 4: Debug Mode (Attempted)
- **Command**: `NO_BROWSER=1 /Users/brent/.cargo/bin/cco run --debug --port 3001`
- **Result**: ❌ FAILED - Unrecognized argument `--debug`
- **Error Output**: Command line parsing rejected the flag

### Verdict: DEBUG MODE NOT YET IMPLEMENTED ❌

---

## TEST 2: Shutdown Button Functionality

### Finding: WORKING - Shutdown endpoint is functional

#### Check 1: Endpoint Path Discovery
- **Initial attempt**: `/shutdown` → ❌ 404 Not Found
- **Correct endpoint**: `/api/shutdown` (POST)
- **Source**: `/Users/brent/git/cc-orchestra/cco/src/server.rs:1617`
  ```rust
  .route("/api/shutdown", post(shutdown_handler))
  ```

#### Check 2: Shutdown Endpoint Test
- **Command**: `curl -X POST http://127.0.0.1:3003/api/shutdown`
- **Response**: ✅ HTTP 200 OK
- **Response Body**:
  ```json
  {"status": "shutdown_initiated", "timestamp": "...", ...}
  ```
- **Response Headers**:
  - Content-Type: application/json
  - HTTP Status: 200 OK

#### Check 3: Process Termination
- **Test**: Started server with PID tracking, called `/api/shutdown`, checked if process still running
- **Result**: ✅ SUCCESS - Process terminated cleanly within 2 seconds
- **Verification**: `ps aux | grep "[c]co run"` - no process found after shutdown

#### Check 4: Graceful Shutdown
- **Source**: `/Users/brent/git/cc-orchestra/cco/src/server.rs:1662`
- **Implementation**: `shutdown_signal()` function enables graceful shutdown
- **Status**: ✅ CONFIRMED - Server shuts down cleanly without errors

### Verdict: SHUTDOWN ENDPOINT WORKING PERFECTLY ✅

---

## Summary

| Feature | Status | Notes |
|---------|--------|-------|
| Debug Mode Flag (`--debug`) | ❌ NOT IMPLEMENTED | Needs to be added to `Run` command struct in main.rs |
| Debug Mode Logging | ❌ NOT IMPLEMENTED | Tracing level filtering not yet configured |
| Shutdown Endpoint (`/api/shutdown`) | ✅ WORKING | HTTP 200 response, graceful process termination |
| Shutdown Process Cleanup | ✅ WORKING | No zombie processes, clean exit |
| Dashboard Shutdown Button | ⚠️ UNKNOWN | Endpoint works; button UI functionality not tested |

---

## Recommendations

### For Debug Mode (BLOCKING)
1. **Add debug flag to Run command** in `cco/src/main.rs`:
   ```rust
   Run {
       /// Enable debug logging
       #[arg(long)]
       debug: bool,

       // ... existing fields ...
   }
   ```

2. **Configure tracing level** based on debug flag:
   ```rust
   if debug {
       // Use DEBUG level
       tracing_subscriber::fmt()
           .with_max_level(tracing::Level::DEBUG)
           .init();
   } else {
       // Use INFO level (default)
       tracing_subscriber::fmt()
           .with_max_level(tracing::Level::INFO)
           .init();
   }
   ```

3. **Add debug logging** throughout server startup:
   - Agent loading initialization
   - Connection tracker creation
   - Cache initialization
   - Router setup

4. **Pass debug flag to run_server()**:
   ```rust
   run_server(&host, port, cache_size, cache_ttl, debug).await
   ```

### For Shutdown (COMPLETE)
- ✅ No changes needed - feature works as expected
- Update documentation to reference `/api/shutdown` endpoint (not `/shutdown`)
- Ensure dashboard UI properly displays shutdown button

---

## Test Commands Reference

### Verify Current Status
```bash
# Check available flags
/Users/brent/.cargo/bin/cco run --help

# Start server (normal mode)
NO_BROWSER=1 /Users/brent/.cargo/bin/cco run --port 3001

# Test shutdown endpoint
curl -X POST http://127.0.0.1:3000/api/shutdown
```

### Once Debug Mode is Implemented
```bash
# Test debug mode (after implementation)
NO_BROWSER=1 /Users/brent/.cargo/bin/cco run --debug --port 3001

# Compare logging output (normal vs debug)
# Normal should show minimal logs
# Debug should show component initialization details
```

---

**Next Step**: Implement debug mode flag and logging configuration in Rust Pro agent task
