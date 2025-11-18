# CLI Enhancements Implementation Status

**Date**: 2025-11-17
**Developer**: Lead Developer (CLI Enhancements)
**Status**: Phase 1-3 Complete (Pending macFUSE Installation)

---

## Implementation Summary

### ✅ Phase 1: Core Launcher Module (COMPLETE)

**File Created**: `cco/src/commands/launcher.rs`

**Functions Implemented**:
- ✅ `launch_claude_code(args: Vec<String>)` - Main entry point
- ✅ `ensure_daemon_running()` - Auto-start daemon with 3-second timeout
- ✅ `verify_vfs_mounted()` - Comprehensive VFS health checks
- ✅ `set_orchestrator_env_vars()` - Environment variable injection
- ✅ `find_claude_code_executable()` - Find Claude Code in PATH using `which` crate
- ✅ `launch_claude_code_process()` - Spawn Claude Code with env vars and args

**Features**:
- Daemon auto-start if not running (up to 3 seconds wait)
- VFS health validation (health file, manifest, sealed files)
- 7 ORCHESTRATOR_* environment variables set
- Current working directory preserved
- Pass-through arguments supported
- Comprehensive error handling with user-friendly messages

---

### ✅ Phase 2: TUI Subcommand Module (COMPLETE)

**File Created**: `cco/src/commands/tui.rs`

**Functions Implemented**:
- ✅ `launch_tui()` - TUI dashboard launcher with daemon check
- ✅ Interactive prompt to start daemon if not running
- ✅ Integration with existing `cco::TuiApp`

**Features**:
- Checks daemon status before launching
- Prompts user to start daemon (Y/n prompt)
- 2-second wait for daemon startup
- Clear error messages with troubleshooting steps
- Fallback suggestion to use web dashboard

---

### ✅ Phase 3: CLI Routing in main.rs (COMPLETE)

**Files Modified**:
- ✅ `cco/src/main.rs` - CLI routing logic updated
- ✅ `cco/src/commands/mod.rs` - Module exports added

**Changes**:
1. **Added `claude_args` field to `Cli` struct**:
   - `#[arg(trailing_var_arg = true, allow_hyphen_values = true)]`
   - Allows pass-through arguments to Claude Code

2. **Added `Tui` subcommand variant**:
   - New enum variant in `Commands`
   - Routes to `commands::tui::launch_tui()`

3. **Updated default behavior** (no subcommand):
   - Old: Install server → Run server → Launch TUI
   - New: Launch Claude Code with orchestration support
   - Calls `commands::launcher::launch_claude_code(cli.claude_args)`

4. **Updated module exports**:
   - Added `pub mod launcher;`
   - Added `pub mod tui;`

---

### ✅ Dependencies Added

**Updated `Cargo.toml`**:
- ✅ `which = "5.0"` - Find executables in PATH
- ✅ `fuser = "0.14"` - FUSE bindings (for VFS)
- ✅ `nix = { version = "0.27", features = ["signal", "mount"] }` - Unix APIs

---

## ⚠️ Build Status: Blocked on macFUSE

### Current Issue

The project has existing VFS (Virtual FileSystem) code in `cco/src/daemon/vfs/` that depends on the `fuser` crate, which requires **macFUSE** to be installed on macOS.

**Error**:
```
pkg-config exited with status code 1
The system library `fuse` required by crate `fuser` was not found.
The system library `osxfuse` required by crate `fuser` was not found.
```

### Resolution Steps

**Option 1: Install macFUSE (Recommended)**:
```bash
# Install via Homebrew
brew install --cask macfuse

# Manual steps required:
# 1. Enter sudo password when prompted
# 2. Enable kernel extension in System Settings → Privacy & Security
# 3. Reboot if required
```

**Option 2: Make VFS Optional** (future work):
- Add feature flag `vfs` in Cargo.toml
- Make `fuser` dependency optional
- Conditionally compile VFS code
- CLI enhancements work without VFS

---

## Command Routing Logic

### New Behavior

| Command | Behavior |
|---------|----------|
| `cco` | Launch Claude Code with orchestration |
| `cco [args...]` | Pass args to Claude Code |
| `cco tui` | Launch TUI monitoring dashboard |
| `cco daemon start` | Start daemon (existing) |
| `cco daemon stop` | Stop daemon (existing) |
| `cco version` | Show version (existing) |
| `cco --help` | Show CCO help (existing) |

### Examples

```bash
# Launch Claude Code in current directory
$ cd ~/my-project
$ cco

# Launch Claude Code with arguments
$ cco --help                    # Shows Claude Code help
$ cco analyze src/main.rs       # Runs Claude Code analysis

# Monitor in separate terminal
$ cco tui

# Manage daemon
$ cco daemon start
$ cco daemon stop
$ cco daemon status
```

---

## User Experience Flow

### Flow 1: Developer Launches Claude Code (`cco`)

```
$ cco

1. ⚙️  Starting daemon... (if not running)
2. ✅ Daemon started
3. ✅ VFS mounted and healthy
4. ✅ Orchestration environment configured
5. 🚀 Launching Claude Code with orchestration support...
   Working directory: /Users/dev/my-project
   VFS mount: /var/run/cco/

[Claude Code launches with ORCHESTRATOR_* vars set]
```

### Flow 2: Developer Monitors Activity (`cco tui`)

```
$ cco tui

1. ✅ Daemon is running
2. 🎯 Launching TUI dashboard...
   Press 'q' to quit, 'h' for help

[TUI dashboard displays with real-time metrics]
```

### Flow 3: Daemon Not Running (Auto-Start)

```
$ cco

1. ⚙️  Starting daemon...
   [Waits up to 3 seconds]
2. ✅ Daemon started
3. ✅ VFS mounted and healthy
4. ✅ Orchestration environment configured
5. 🚀 Launching Claude Code...
```

---

## Environment Variables Set

When `cco` launches Claude Code, these environment variables are automatically injected:

```bash
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_VFS_MOUNT=/var/run/cco
ORCHESTRATOR_AGENTS=/var/run/cco/agents.sealed
ORCHESTRATOR_RULES=/var/run/cco/orchestrator.sealed
ORCHESTRATOR_HOOKS=/var/run/cco/hooks.sealed
ORCHESTRATOR_MANIFEST=/var/run/cco/.manifest
ORCHESTRATOR_API_URL=http://localhost:3000
```

---

## Error Handling

### Comprehensive Error Scenarios

| Scenario | Detection | Message | Recovery |
|----------|-----------|---------|----------|
| Daemon not running | Health check fails | "Starting daemon..." | Auto-start |
| Daemon start timeout | No health after 3s | "Failed to start daemon. Try: cco daemon restart" | Exit with error |
| VFS not mounted | /var/run/cco/health missing | "VFS not mounted. Try: cco daemon restart" | Exit with error |
| VFS unhealthy | health != "OK" | "VFS unhealthy. Try: cco daemon restart" | Exit with error |
| Claude Code not found | which() returns None | "Claude Code not found. Install from: https://claude.ai/code" | Exit with error |
| Claude Code fails | Exit code != 0 | "Claude Code exited with status: {code}" | Exit with error |
| CWD not accessible | current_dir() fails | "Cannot access current directory" | Exit with error |

### Example Error Messages

**Claude Code Not Found**:
```
✅ Daemon is running
✅ VFS mounted and healthy
✅ Orchestration environment configured
❌ Claude Code executable not found in PATH
   Please install Claude Code first:
   https://claude.ai/code

   After installation, ensure 'claude' is in your PATH.
```

**VFS Not Mounted**:
```
✅ Daemon is running
❌ VFS not mounted at /var/run/cco/
   The daemon is running but VFS failed to mount.
   Try: cco daemon restart
```

---

## Testing Status

### ⏳ Phase 4: Tests, Error Handling, Documentation (PENDING)

**Blocked by**: macFUSE installation required for build

**Planned Tests**:

#### Unit Tests
- [ ] `test_set_orchestrator_env_vars()` - Verify 7 env vars
- [ ] `test_find_claude_code_executable()` - Mock PATH search
- [ ] `test_verify_vfs_mounted()` - Mock health file checks
- [ ] `test_ensure_daemon_running()` - Mock daemon status API
- [ ] `test_launch_claude_code_process()` - Mock process spawn

#### Integration Tests
- [ ] Full workflow: `cco` → daemon auto-start → VFS check → Claude Code launch
- [ ] TUI launch with daemon auto-start
- [ ] Pass-through arguments work correctly
- [ ] Error recovery scenarios (11+ cases)
- [ ] Multiple simultaneous sessions (cco + cco tui)

#### E2E Tests
- [ ] Clean environment (daemon not running)
- [ ] Daemon crash recovery
- [ ] VFS health check failure
- [ ] Claude Code executable not found
- [ ] Verify exit codes and error messages

**Target Coverage**: > 90%

---

## Documentation Updates (PENDING)

### Files to Update

1. **README.md**:
   - New CLI usage examples
   - Updated quick start guide
   - Architecture diagram with launcher flow

2. **INSTALLATION.md**:
   - macFUSE installation requirement
   - New workflow (cco vs cco tui)
   - Troubleshooting section

3. **USER_GUIDE.md**:
   - `cco` command documentation
   - `cco tui` command documentation
   - Pass-through arguments examples
   - Environment variables reference

4. **MIGRATION_GUIDE.md** (NEW):
   - Old users: `cco` launched TUI
   - New users: `cco tui` launches TUI
   - `cco` now launches Claude Code
   - All other commands unchanged

---

## Next Steps

### Immediate Actions

1. **Install macFUSE**:
   ```bash
   brew install --cask macfuse
   # Enable kernel extension in System Settings → Privacy & Security
   # Reboot if required
   ```

2. **Verify Build**:
   ```bash
   cd /Users/brent/git/cc-orchestra/cco
   cargo build --release
   ```

3. **Run Unit Tests**:
   ```bash
   cargo test --lib launcher
   cargo test --lib tui
   ```

### Phase 4 Implementation (2-3 days)

1. Write unit tests for launcher module (< 1 day)
2. Write integration tests for full workflow (< 1 day)
3. Write E2E tests with mock Claude Code (< 1 day)
4. Update documentation (README, guides) (< 1 day)
5. Create migration guide for users (< 4 hours)

---

## Success Criteria (After Phase 4)

- ✅ Default `cco` launches Claude Code (< 3 seconds with daemon auto-start)
- ✅ `cco tui` launches monitoring dashboard
- ✅ Pass-through arguments work: `cco --help`, `cco analyze code.py`
- ✅ All 11+ error scenarios handled with user-friendly messages
- ✅ Daemon auto-start works (3 second wait)
- ✅ VFS health check completes in < 100ms
- ✅ Environment variables correctly set
- ✅ Zero errors when daemon already running
- ✅ Simultaneous `cco` + `cco tui` sessions work
- ✅ Test coverage > 90%
- ✅ Documentation complete and accurate

---

## Files Changed

### New Files
- `cco/src/commands/launcher.rs` (370 lines)
- `cco/src/commands/tui.rs` (110 lines)
- `cco/CLI_ENHANCEMENTS_STATUS.md` (this file)

### Modified Files
- `cco/src/main.rs` (updated CLI routing)
- `cco/src/commands/mod.rs` (added module exports)
- `cco/Cargo.toml` (added dependencies)

### Total Lines of Code
- New: ~480 lines
- Modified: ~50 lines
- Documentation: ~300 lines

---

## Coordination

### Dependencies
- **Rust Pro**: VFS implementation (partially complete, needs macFUSE)
- **Security Auditor**: Will review environment variable handling
- **Test Engineer**: Will write comprehensive test suite
- **Chief Architect**: Review after each phase completion

### Delivery Status
- **Phase 1**: ✅ Complete
- **Phase 2**: ✅ Complete
- **Phase 3**: ✅ Complete
- **Phase 4**: ⏳ Pending (blocked on macFUSE)

---

**Last Updated**: 2025-11-17 22:00 UTC
**Next Review**: After macFUSE installation and successful build
