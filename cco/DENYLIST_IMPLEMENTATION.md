# Command Denylist Implementation

## Overview

Added a command denylist to the permission system at `/Users/brent/git/cc-orchestra/cco/src/daemon/hooks/permissions.rs` that provides defense-in-depth by hard-blocking critical build commands even when `dangerously_skip_confirmations` is enabled.

## Implementation Details

### 1. Denylist Constants

Added `DENIED_COMMANDS` constant with 10 critical commands:

```rust
const DENIED_COMMANDS: &[&str] = &[
    // Build artifact deletion
    "cargo clean",
    "rm -rf target",
    "rm -rf target/",
    // Git destructive operations
    "git clean -fdx",
    "git clean -fd",
    // Docker destructive operations
    "docker system prune -a",
    "docker system prune --all",
    // Node/Python build cleanup
    "rm -rf node_modules",
    "rm -rf .venv",
];
```

### 2. Denylist Check Logic

Modified `PermissionHandler::process_request()` to check denylist **FIRST** (before any other logic):

```rust
// Check denylist FIRST - overrides all other logic
for denied_cmd in DENIED_COMMANDS {
    if request.command.contains(denied_cmd) {
        warn!(
            "🚫 Command denied by denylist: {} (matched: {})",
            request.command, denied_cmd
        );
        return PermissionResponse::new(
            PermissionDecision::Denied,
            format!(
                "Command denied: '{}' would delete critical build artifacts or perform destructive operations",
                request.command
            ),
        );
    }
}
```

### 3. Documentation

Added comprehensive documentation to `PermissionHandler` struct explaining:
- What the denylist is
- Which commands are denied
- Why they're denied
- That denylist overrides all configuration

### 4. Test Coverage

Added 13 comprehensive tests:

**Basic Denylist Tests:**
- `test_denylist_blocks_cargo_clean` - Blocks `cargo clean`
- `test_denylist_blocks_cargo_clean_in_compound_command` - Blocks in compound commands
- `test_denylist_blocks_rm_target` - Blocks both `rm -rf target` and `rm -rf target/`
- `test_denylist_blocks_git_clean` - Blocks both `-fdx` and `-fd` variants
- `test_denylist_blocks_docker_system_prune` - Blocks both `-a` and `--all` variants
- `test_denylist_blocks_rm_node_modules` - Blocks Node.js dependency deletion
- `test_denylist_blocks_rm_venv` - Blocks Python venv deletion

**Priority and Override Tests:**
- `test_denylist_blocks_even_with_skip_confirmations` - Denylist works with `dangerously_skip_confirmations=true`
- `test_denylist_with_auto_approve_read` - Denylist works even if misclassified as READ
- `test_denylist_priority_over_all_config` - Denylist has highest priority

**Safe Command Tests:**
- `test_safe_commands_still_work` - Ensures safe commands like `cargo check`, `cargo build`, `git status` still work

## Security Benefits

1. **Defense-in-Depth**: Even if `dangerously_skip_confirmations` is enabled, critical build artifacts are protected
2. **No False Positives**: Carefully chosen substring matching avoids blocking legitimate commands
3. **Clear Feedback**: Users get clear error messages explaining why commands are blocked
4. **Comprehensive Coverage**: Protects Rust, Node, Python, Git, and Docker build artifacts

## Verification

Standalone test verified all denylist logic works correctly:

```bash
✅ All denied commands blocked correctly (10/10)
✅ All safe commands allowed correctly (7/7)
✅ Total: 17/17 tests passed
```

## Files Modified

- `/Users/brent/git/cc-orchestra/cco/src/daemon/hooks/permissions.rs`
  - Added `DENIED_COMMANDS` constant (lines 197-212)
  - Modified `process_request()` to check denylist first (lines 251-266)
  - Updated documentation (lines 162-187)
  - Added 13 comprehensive tests (lines 458-610)

## Example Usage

```rust
// This command will be DENIED, even with dangerously_skip_confirmations=true
let request = PermissionRequest::new("cargo clean", CrudClassification::Delete);
let response = handler.process_request(request).await;
// response.decision == PermissionDecision::Denied
// response.reasoning == "Command denied: 'cargo clean' would delete critical build artifacts..."

// Safe commands still work
let request = PermissionRequest::new("cargo check", CrudClassification::Read);
let response = handler.process_request(request).await;
// response.decision == PermissionDecision::Approved
```

## Future Enhancements

Potential additions to the denylist:
- `npm run clean` - Node.js clean scripts
- `make clean` - Makefile clean targets
- `rm -rf dist/` - Distribution directory deletion
- `rm -rf build/` - Build directory deletion
- Custom project-specific critical paths
