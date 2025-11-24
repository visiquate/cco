# FUSE VFS CLI Enhancements - Update Summary

**Date:** 2025-11-17
**Status:** Specification Complete

---

## What Was Updated

The FUSE VFS implementation plan has been enhanced with new CLI behaviors that transform `cco` into a seamless Claude Code launcher with orchestration support.

### New Documents Created

1. **[cco/docs/FUSE_VFS_CLI_ENHANCEMENTS.md](/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_CLI_ENHANCEMENTS.md)**
   - Complete implementation specification
   - Command routing logic
   - Error handling scenarios
   - Integration with FUSE VFS
   - Testing strategy
   - Documentation requirements

2. **[cco/docs/FUSE_VFS_CLI_QUICK_SUMMARY.md](/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_CLI_QUICK_SUMMARY.md)**
   - Quick reference guide
   - Before/after comparison
   - Command routing table
   - Migration guide

### Existing Documents Updated

1. **[cco/docs/FUSE_VFS_IMPLEMENTATION_PLAN.md](/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_IMPLEMENTATION_PLAN.md)**
   - Added "CLI Enhancements" section (section 14)
   - Added cross-references to new documents
   - Updated Table of Contents

---

## Key Changes

### 1. Default `cco` Behavior

**Before:**
```bash
$ cco
# Installs server + launches TUI dashboard
```

**After:**
```bash
$ cco
# 1. Auto-starts daemon if not running
# 2. Verifies VFS mounted at /var/run/cco/
# 3. Sets ORCHESTRATOR_* environment variables
# 4. Launches Claude Code in current directory
```

### 2. New `cco tui` Subcommand

```bash
$ cco tui
# Launches TUI monitoring dashboard
# Allows running both cco and cco tui simultaneously
```

### 3. Pass-Through Arguments

```bash
$ cco --help                # Shows Claude Code help
$ cco analyze code.py       # Runs Claude Code analysis
$ cco [any-args]           # Passed to Claude Code
```

---

## Implementation Requirements

### New Files to Create

```
cco/src/commands/
├── launcher.rs            # NEW: Claude Code launcher
│   ├── launch_claude_code()
│   ├── ensure_daemon_running()
│   ├── verify_vfs_mounted()
│   ├── set_orchestrator_env_vars()
│   └── find_claude_code_executable()
│
└── tui.rs                 # NEW: TUI dashboard launcher
    └── launch_tui()
```

### Files to Modify

```
cco/src/
├── main.rs                # MODIFY: Add CLI routing
│   ├── Add trailing_var_arg for Claude Code args
│   ├── Add Tui subcommand
│   └── Route to launcher for no subcommand
│
└── commands/
    └── mod.rs             # MODIFY: Export launcher and tui modules
```

### Dependencies to Add

```toml
[dependencies]
which = "5.0"              # Find executables in PATH
```

---

## Environment Variables Set by CCO

When `cco` launches Claude Code, these are automatically set:

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

## VFS Integration

### Daemon Startup with VFS

```
User: cco daemon start
    │
    ├─> DaemonManager::start()
    ├─> Load DaemonConfig
    ├─> Mount VFS at /var/run/cco/
    │   ├─> Generate sealed files in memory
    │   │   ├─> agents.sealed (encrypted)
    │   │   ├─> orchestrator.sealed (encrypted)
    │   │   ├─> hooks.sealed (encrypted)
    │   │   ├─> .manifest (plaintext)
    │   │   └─> health (plaintext: "OK")
    │   │
    │   └─> Mount FUSE filesystem
    │
    └─> Start HTTP server on port 3000
```

### Claude Code Launch with VFS

```
User: cco
    │
    ├─> Check daemon status
    │   └─> If not running: auto-start (3s)
    │
    ├─> Verify VFS mounted
    │   ├─> Check /var/run/cco/health exists
    │   └─> Verify health == "OK"
    │
    ├─> Set ORCHESTRATOR_* environment vars
    │
    └─> Launch Claude Code
        ├─> Preserve current working directory
        └─> Inherit environment with ORCHESTRATOR_* vars
```

---

## Error Handling

| Scenario | Detection | Recovery |
|----------|-----------|----------|
| **Daemon not running** | `manager.get_status()` fails | Auto-start daemon |
| **VFS not mounted** | `/var/run/cco/health` missing | Suggest `cco daemon restart` |
| **VFS unhealthy** | Health != "OK" | Suggest `cco daemon restart` |
| **Claude Code not found** | `which::which()` fails | Show install instructions |
| **Daemon start fails** | `manager.start()` fails | Show error and exit |

---

## Testing Strategy

### Unit Tests

- [ ] `test_ensure_daemon_running_starts_if_not_running()`
- [ ] `test_verify_vfs_mounted_succeeds_if_healthy()`
- [ ] `test_verify_vfs_mounted_fails_if_not_mounted()`
- [ ] `test_set_orchestrator_env_vars()`
- [ ] `test_find_claude_code_executable_finds_in_path()`
- [ ] `test_find_claude_code_executable_fails_if_not_found()`

### Integration Tests

- [ ] Full workflow: `cco` → daemon start → VFS mount → Claude Code launch
- [ ] TUI launch with daemon auto-start
- [ ] Pass-through arguments to Claude Code
- [ ] Multiple simultaneous sessions (cco + cco tui)

### E2E Tests

- [ ] Clean slate installation
- [ ] Daemon crash recovery
- [ ] VFS health check failure
- [ ] Claude Code executable not found

---

## Documentation Updates Required

### Files to Update

- [ ] `README.md` - Add new CLI usage examples
- [ ] `INSTALLATION.md` - Update installation workflow
- [ ] `USER_GUIDE.md` - Add `cco` and `cco tui` documentation
- [ ] `CLAUDE.md` - Update orchestration setup instructions

### New Sections Needed

**README.md:**
```markdown
## Quick Start

```bash
# Install CCO
curl -fsSL https://cco.visiquate.com/install.sh | sh

# Launch Claude Code with orchestration
cd ~/my-project
cco

# Monitor in another terminal
cco tui
```
```

**INSTALLATION.md:**
```markdown
## Verify Installation

```bash
# Step 1: Start daemon
cco daemon start

# Step 2: Verify VFS mounted
cat /var/run/cco/health  # Should output "OK"

# Step 3: Launch Claude Code
cd ~/test-project
cco
```
```

---

## Migration Path

### For Current Users

**Old Workflow:**
```bash
$ cco daemon start
$ cco  # Launches TUI
```

**New Workflow:**
```bash
$ cco         # Launches Claude Code (daemon auto-starts)
$ cco tui     # Launches TUI (in separate terminal)
```

### Backward Compatibility

✅ All existing subcommands still work:
- `cco daemon start|stop|restart|status`
- `cco server install|run|uninstall`
- `cco version`
- `cco update`
- `cco config [action]`

❌ Breaking change:
- `cco` (no args) now launches Claude Code instead of TUI
- Users must use `cco tui` for TUI dashboard

---

## Implementation Checklist

### Phase 1: Core Launcher
- [ ] Create `cco/src/commands/launcher.rs`
- [ ] Implement `ensure_daemon_running()`
- [ ] Implement `verify_vfs_mounted()`
- [ ] Implement `set_orchestrator_env_vars()`
- [ ] Implement `launch_claude_code_process()`
- [ ] Implement `find_claude_code_executable()`

### Phase 2: TUI Subcommand
- [ ] Create `cco/src/commands/tui.rs`
- [ ] Implement `launch_tui()`
- [ ] Add daemon auto-start prompt

### Phase 3: CLI Routing
- [ ] Update `cco/src/main.rs`
- [ ] Add `trailing_var_arg` to Cli struct
- [ ] Add `Tui` subcommand
- [ ] Route no subcommand to launcher
- [ ] Route `tui` to TUI handler

### Phase 4: Testing
- [ ] Write unit tests for launcher
- [ ] Write integration tests for full workflow
- [ ] Write E2E tests with mock Claude Code
- [ ] Test error recovery scenarios

### Phase 5: Documentation
- [ ] Update README.md
- [ ] Update INSTALLATION.md
- [ ] Update USER_GUIDE.md
- [ ] Update CLAUDE.md
- [ ] Create migration guide

---

## Success Metrics

- ✅ `cco` launches Claude Code in < 3 seconds (including daemon auto-start)
- ✅ VFS health check completes in < 100ms
- ✅ Environment variables correctly set in 100% of launches
- ✅ Zero errors when daemon already running
- ✅ Graceful error messages for all failure scenarios
- ✅ Pass-through arguments work correctly with Claude Code
- ✅ Simultaneous `cco` + `cco tui` sessions supported

---

## Related Files

### Documentation
- `/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_IMPLEMENTATION_PLAN.md`
- `/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_CLI_ENHANCEMENTS.md`
- `/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_CLI_QUICK_SUMMARY.md`
- `/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_ARCHITECTURE.md`
- `/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_SECURITY_ANALYSIS.md`

### Source Code (Existing)
- `/Users/brent/git/cc-orchestra/cco/src/main.rs`
- `/Users/brent/git/cc-orchestra/cco/src/commands/mod.rs`
- `/Users/brent/git/cc-orchestra/cco/src/daemon/mod.rs`

### Source Code (To Be Created)
- `/Users/brent/git/cc-orchestra/cco/src/commands/launcher.rs`
- `/Users/brent/git/cc-orchestra/cco/src/commands/tui.rs`
- `/Users/brent/git/cc-orchestra/cco/src/daemon/vfs/mod.rs`
- `/Users/brent/git/cc-orchestra/cco/src/daemon/vfs/sealed.rs`
- `/Users/brent/git/cc-orchestra/cco/src/daemon/vfs/mount.rs`

---

## Next Steps

1. **Review Documents**
   - Read FUSE_VFS_CLI_ENHANCEMENTS.md for complete implementation details
   - Review command routing logic and error handling scenarios

2. **Start Implementation**
   - Create `cco/src/commands/launcher.rs` module
   - Create `cco/src/commands/tui.rs` module
   - Update `cco/src/main.rs` with CLI routing

3. **Test Integration**
   - Write unit tests for launcher logic
   - Write integration tests for VFS + launcher
   - Test with mock Claude Code executable

4. **Update Documentation**
   - Update README.md with new CLI examples
   - Update installation guide
   - Create migration guide for existing users

5. **Deploy and Iterate**
   - Deploy to test environment
   - Gather user feedback
   - Iterate based on real-world usage

---

**Status:** Specification Complete, Ready for Implementation
**Review Date:** 2025-12-01 (2 weeks after implementation start)
