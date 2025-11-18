# CCO Migration Guide

**For Existing Users: Architecture Update**

This guide helps existing CCO users migrate to the simplified temp file approach.

---

## Architecture Change: FUSE → Temp Files

### What Changed

**Before:** Settings stored in FUSE virtual filesystem at `/var/run/cco/`

**After:** Settings stored in OS temp directory (`$TMPDIR/.cco-*`)

### User Impact

✅ **Benefits:**
- No macFUSE installation required
- Works on Windows (previously unsupported)
- Simpler setup (no system extensions)
- Same security (encryption unchanged)
- Faster startup (no VFS mount time)
- Automatic cleanup (temp files removed on daemon stop)

### New System Paths

| Platform | Temp Directory Path |
|----------|-------------------|
| macOS | `/var/folders/xx/xxx/T/.cco-*` |
| Windows | `C:\Users\[user]\AppData\Local\Temp\.cco-*` |
| Linux | `/tmp/.cco-*` |

All paths are temporary and auto-cleaned on daemon shutdown.

---

## What Changed?

### Before (Old Behavior)

```bash
$ cco                  # Installed server + launched TUI
$ cco run              # Ran server + launched TUI
$ cco dashboard        # Launched TUI
$ cco daemon start     # Started daemon in background
```

### After (New Behavior)

```bash
$ cco                  # Launches Claude Code with orchestration
$ cco tui              # Launches TUI monitoring dashboard (NEW)
$ cco daemon start     # Starts daemon (same as before)
$ cco [args...]        # Passes args to Claude Code
```

**Key Change:** `cco` (no arguments) now launches **Claude Code** instead of the TUI.

---

## Impact Assessment

### Breaking Changes

✅ **Backward Compatible:**
- `cco daemon start|stop|restart|status` - Still work exactly the same
- `cco server install|run|uninstall` - Still work exactly the same
- `cco version` - Still works
- `cco update` - Still works
- `cco config` - Still works

❌ **Breaking Change:**
- `cco` (no args) - Now launches **Claude Code** instead of TUI
  - **Migration:** Use `cco tui` to launch TUI

### New Features

✅ **Added:**
- `cco tui` - Launch TUI monitoring dashboard
- `cco [args...]` - Pass arguments directly to Claude Code
- Auto-start daemon when launching Claude Code
- Automatic VFS mounting and verification
- Automatic environment variable injection

---

## Migration Steps

### Step 1: Update CCO

```bash
# Update to latest version
cco update

# Or reinstall
curl -fsSL https://cco.visiquate.com/install.sh | sh

# Verify version
cco version
# Should show: 2025.11.17 or later
```

### Step 2: Update Your Workflow

#### Old TUI Launch Workflow

**Before:**
```bash
$ cco                  # Launched TUI
```

**After:**
```bash
$ cco tui              # Launch TUI (new command)
```

#### New Claude Code Launch Workflow

**Before:**
```bash
# Had to manually:
# 1. Start daemon
# 2. Set environment variables
# 3. Launch Claude Code

$ cco daemon start
$ export ORCHESTRATOR_ENABLED=true
$ export ORCHESTRATOR_VFS_MOUNT=/var/run/cco
$ export ORCHESTRATOR_API_URL=http://localhost:3000
$ claude-code
```

**After:**
```bash
# All automatic:
$ cd ~/my-project
$ cco

# CCO now:
# 1. Auto-starts daemon if needed
# 2. Verifies VFS mounted
# 3. Sets all environment variables
# 4. Launches Claude Code in current directory
```

### Step 3: Update Scripts and Aliases

If you have scripts or shell aliases that use `cco`:

**Before:**
```bash
# .zshrc or .bashrc
alias monitor='cco'                    # Launched TUI
alias start-cco='cco daemon start'    # Started daemon
```

**After:**
```bash
# .zshrc or .bashrc
alias monitor='cco tui'                # Launch TUI (updated)
alias dev='cco'                        # Launch Claude Code (new)
alias start-cco='cco daemon start'    # Same as before
```

### Step 4: Test New Workflow

```bash
# Ensure daemon is running
cco daemon start

# Test TUI launch (new command)
cco tui
# Should show monitoring dashboard

# Test Claude Code launch
cd ~/test-project
cco
# Should launch Claude Code with orchestration

# Verify environment variables are set
# In Claude Code terminal, check:
env | grep ORCHESTRATOR
# Should show:
# ORCHESTRATOR_ENABLED=true
# ORCHESTRATOR_VFS_MOUNT=/var/run/cco
# ORCHESTRATOR_API_URL=http://localhost:3000
# ... (and more)
```

---

## Feature Comparison

| Feature | Old Behavior | New Behavior |
|---------|-------------|--------------|
| `cco` (no args) | Launch TUI | Launch Claude Code |
| `cco tui` | N/A (didn't exist) | Launch TUI |
| `cco daemon start` | Start daemon | Start daemon (same) |
| `cco daemon stop` | Stop daemon | Stop daemon (same) |
| `cco version` | Show version | Show version (same) |
| `cco update` | Check/install updates | Check/install updates (same) |
| `cco [args]` | N/A | Pass to Claude Code |
| VFS mounting | Manual | Automatic |
| Environment vars | Manual | Automatic |
| Daemon auto-start | No | Yes |

---

## Common Migration Scenarios

### Scenario 1: Daily Development Workflow

**Before:**
```bash
# Terminal 1: Start daemon
$ cco daemon start

# Terminal 2: Monitor
$ cco  # Launched TUI

# Terminal 3: Development
$ cd ~/project
$ export ORCHESTRATOR_ENABLED=true
$ claude-code
```

**After:**
```bash
# Terminal 1: Development
$ cd ~/project
$ cco  # Auto-starts daemon, launches Claude Code

# Terminal 2: Monitor (optional)
$ cco tui  # Monitor in separate terminal
```

**Benefits:**
- 3 terminals → 1-2 terminals
- 3 manual steps → 1 command
- Automatic daemon management

### Scenario 2: Team Development Server

**Before:**
```bash
# Server setup
ssh dev-server
cco daemon start --host 0.0.0.0 --port 3000

# Each developer
export ORCHESTRATOR_API_URL=http://dev-server:3000
claude-code
```

**After:**
```bash
# Server setup (same)
ssh dev-server
cco daemon start --host 0.0.0.0 --port 3000

# Each developer (simplified)
export ORCHESTRATOR_API_URL=http://dev-server:3000
cco  # Launches with orchestration
```

**Benefits:**
- Cleaner developer onboarding
- Consistent environment setup

### Scenario 3: Multiple Projects

**Before:**
```bash
# Switch between projects manually
cd ~/project-1
export ORCHESTRATOR_API_URL=http://localhost:3000
claude-code

# Later...
cd ~/project-2
export ORCHESTRATOR_API_URL=http://localhost:3000
claude-code
```

**After:**
```bash
# Switch seamlessly
cd ~/project-1
cco  # Launches Claude Code in project-1

# Later...
cd ~/project-2
cco  # Launches Claude Code in project-2
# Both share same daemon automatically
```

**Benefits:**
- No environment variable management
- Automatic context switching

---

## Troubleshooting Migration

### Issue: `cco` launches Claude Code instead of TUI

**This is expected behavior.**

**Solution:** Use `cco tui` to launch TUI.

```bash
# Old command (no longer launches TUI)
cco

# New command (launches TUI)
cco tui
```

### Issue: Scripts broken after update

**Cause:** Scripts use `cco` to launch TUI

**Solution:** Update scripts to use `cco tui`:

```bash
# Find affected scripts
grep -r "^cco$" ~/scripts/

# Update each occurrence
# Before: cco
# After:  cco tui
```

### Issue: Daemon not auto-starting

**Cause:** Permissions or VFS mount issue

**Solution:**
```bash
# Check daemon status
cco daemon status

# If not running, start manually
cco daemon start

# Check VFS health
cat /var/run/cco/health
# Should output: OK

# If VFS not mounted, restart daemon
cco daemon restart
```

### Issue: Environment variables not set

**Cause:** Running Claude Code directly instead of via `cco`

**Solution:**
```bash
# Don't do this:
claude-code

# Do this instead:
cco  # Automatically sets all variables
```

### Issue: Multiple `cco` instances

**Cause:** Running `cco` and `cco tui` simultaneously

**Solution:** This is **normal and expected**.

```bash
# Terminal 1: Development
cco  # Claude Code

# Terminal 2: Monitoring
cco tui  # TUI dashboard

# Both share the same daemon - this is correct
```

---

## Rollback (If Needed)

If you need to revert to the old behavior:

### Option 1: Use Previous Version

```bash
# Uninstall current version
sudo rm /usr/local/bin/cco

# Download previous version
curl -L https://cco.visiquate.com/releases/2025.11.16/cco -o cco
chmod +x cco
sudo mv cco /usr/local/bin/

# Verify version
cco version
```

### Option 2: Create Alias for Old Behavior

```bash
# Add to ~/.zshrc or ~/.bashrc
alias cco-old='cco tui'  # Old behavior (launch TUI)
alias cco-new='cco'      # New behavior (launch Claude Code)

# Use old command
cco-old  # Launches TUI (like old `cco`)
```

---

## FAQ

### Q: Why did the behavior change?

**A:** The new behavior makes CCO the **primary launcher** for Claude Code with orchestration support. This streamlines the developer workflow from 3+ manual steps to a single command.

### Q: Can I still use the TUI?

**A:** Yes! Use `cco tui` to launch the TUI monitoring dashboard.

### Q: What if I forget and type just `cco`?

**A:** It will launch Claude Code with orchestration. If you wanted the TUI, just run `cco tui` instead.

### Q: Do I need to update my team's workflow?

**A:** Yes, if your team has scripts or documentation that use `cco` to launch the TUI, update them to use `cco tui`.

### Q: Will this break my CI/CD pipelines?

**A:** No, if your pipelines use specific commands like `cco daemon start` or `cco version`, they remain unchanged.

### Q: Can I use both `cco` and `cco tui` at the same time?

**A:** Yes! You can run `cco` (Claude Code) in one terminal and `cco tui` (monitoring) in another. Both share the same daemon.

### Q: What about `cco run` or `cco dashboard`?

**A:** These commands still exist for backward compatibility but are deprecated. Use `cco` (Claude Code) or `cco tui` (dashboard) instead.

### Q: How do I pass arguments to Claude Code?

**A:** Just add them after `cco`:
```bash
cco --help              # Shows Claude Code help
cco analyze src/main.rs  # Analyzes file
```

---

## Getting Help

If you encounter issues during migration:

1. **Check documentation:**
   - [INSTALLATION.md](./INSTALLATION.md) - Installation and verification
   - [USAGE.md](./USAGE.md) - Complete command reference
   - [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) - Common issues

2. **Enable debug logging:**
   ```bash
   cco daemon logs --follow
   ```

3. **Verify installation:**
   ```bash
   cco version          # Check version
   cco daemon status    # Check daemon
   cat /var/run/cco/health  # Check VFS
   ```

4. **Open issue on GitHub:**
   - Include version: `cco version`
   - Include logs: `cco daemon logs`
   - Remove sensitive data (API keys, etc.)

---

## Summary

**What you need to know:**

✅ `cco` now launches **Claude Code** with orchestration
✅ Use `cco tui` to launch **TUI dashboard**
✅ All daemon commands work the same
✅ Daemon auto-starts when needed
✅ Environment variables set automatically
✅ VFS mounts automatically
✅ You can run `cco` and `cco tui` simultaneously

**Migration checklist:**

- [ ] Update CCO to latest version
- [ ] Update scripts/aliases to use `cco tui`
- [ ] Test `cco` launches Claude Code
- [ ] Test `cco tui` launches dashboard
- [ ] Verify daemon auto-starts
- [ ] Verify environment variables set
- [ ] Update team documentation

**Timeline:**

- Old behavior deprecated but still accessible via `cco tui`
- New behavior is the recommended workflow
- No forced breaking changes to daemon commands
