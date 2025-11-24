# Version-Aware Service Update

## Overview

The CCO daemon now includes automatic version-aware service updates. When the binary version changes (e.g., after a package update), the next `cco server run` command will automatically detect the version mismatch and gracefully restart the daemon with the new version.

## Key Features

1. **Automatic Detection**: Compares running daemon version with current binary version
2. **Graceful Restart**: Stops old daemon cleanly before starting new version
3. **Idempotent**: No-op when versions match (safe to run multiple times)
4. **Transparent**: User doesn't need manual intervention

## Implementation Details

### Version Comparison Logic

```rust
// Get current binary version
let current_version = env!("CCO_VERSION");

// Compare versions (strip git hash suffix if present)
let running_version = health.version.split('+').next().unwrap_or(&health.version);
let binary_version = current_version.split('+').next().unwrap_or(current_version);

if running_version != binary_version {
    // Version mismatch - restart daemon
    manager.stop().await?;
    // Fall through to start logic
}
```

### Version Format

CCO uses date-based versioning with optional git hash:

- **Format**: `YYYY.MM.N+<git-hash>`
- **Example**: `2025.11.3+a5a0f13`

The comparison ignores the git hash suffix, only comparing the semantic version.

### Files Modified

- **`/Users/brent/git/cc-orchestra/cco/src/commands/server.rs`**
  - Added version comparison logic in `run()` function (lines 79-112)
  - Detects mismatch between running daemon and current binary
  - Triggers graceful restart when versions differ

## Usage

### Normal Usage

Just run `cco server run` after updating the package:

```bash
# Update package (via auto-update or manual build)
cco update

# Start/restart server (automatically detects version change)
cco server run
```

### Example Output

**When version mismatch is detected:**

```
🔄 Version mismatch detected:
   Running: 2025.11.3+a5a0f13
   Binary:  2025.11.4+c8e73cc
   Restarting daemon with new version...
Shutting down daemon (PID 12345)...
✅ Daemon shut down gracefully
🔌 Starting server on 127.0.0.1:3000...
✅ Daemon started successfully (PID: 12346)
⏳ Waiting for server to become ready...
✅ Server ready
   Dashboard: http://127.0.0.1:3000
```

**When versions match (idempotent):**

```
✅ Server already running
   PID: 12346
   Port: 3000
   Version: 2025.11.4+c8e73cc
   Dashboard: http://127.0.0.1:3000
```

## Testing

### Manual Testing

1. Build and start current version:
   ```bash
   cargo build --release
   ./target/release/cco server run
   ```

2. Verify version:
   ```bash
   curl http://127.0.0.1:3000/health | jq -r '.version'
   ```

3. Simulate version change:
   ```bash
   # Modify version in build.rs
   sed -i 's/"2025.11.3"/"2025.11.4"/' build.rs

   # Rebuild
   cargo build --release
   ```

4. Test automatic restart:
   ```bash
   ./target/release/cco server run
   # Should detect version mismatch and restart
   ```

5. Test idempotency:
   ```bash
   ./target/release/cco server run
   # Should report "already running" without restart
   ```

### Automated Testing

A comprehensive test script is available at `/Users/brent/git/cc-orchestra/cco/test-version-aware-update.sh`:

```bash
chmod +x test-version-aware-update.sh
./test-version-aware-update.sh
```

The test script:
- Builds current version
- Starts daemon
- Simulates version change by modifying build.rs
- Rebuilds with new version
- Verifies automatic restart on version mismatch
- Verifies idempotency (no restart when versions match)
- Restores original version

## Edge Cases Handled

1. **Daemon not responding**: If health check fails, attempts restart regardless of version
2. **Stale PID file**: Cleans up and starts fresh if process is not running
3. **Graceful shutdown failure**: Falls back to SIGKILL if graceful shutdown times out
4. **Version string variations**: Handles versions with or without git hash suffix

## Integration with Auto-Update

This feature works seamlessly with the auto-update system:

1. Auto-update downloads new binary
2. Auto-update installs new version
3. User runs `cco` or `cco server run`
4. Version mismatch detected automatically
5. Daemon restarted with new version

No manual intervention required!

## Benefits

1. **Zero Downtime Updates**: Graceful restart ensures smooth transitions
2. **User Convenience**: No need to manually stop/restart after updates
3. **Safe**: Idempotent behavior prevents accidental restarts
4. **Transparent**: Clear messaging about what's happening
5. **Reliable**: Handles edge cases and failures gracefully

## Version History

- **2025.11.5**: Initial implementation of version-aware updates
- **Future**: Could extend to config file changes, dependency updates, etc.

## Related Files

- `/Users/brent/git/cc-orchestra/cco/src/commands/server.rs` - Main implementation
- `/Users/brent/git/cc-orchestra/cco/src/daemon/lifecycle.rs` - Daemon lifecycle management
- `/Users/brent/git/cc-orchestra/cco/src/api_client.rs` - Health check client
- `/Users/brent/git/cc-orchestra/cco/build.rs` - Version generation
- `/Users/brent/git/cc-orchestra/cco/test-version-aware-update.sh` - Test script
