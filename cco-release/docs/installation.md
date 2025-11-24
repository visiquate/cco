# Installation Guide

Complete installation instructions for Claude Code Orchestrator (CCO).

## System Requirements

### Cross-Platform (macOS + Windows + Linux)

- **No special system dependencies**
- **Works out of the box**
- **Runs as standard user (no admin needed)**
- **Operating System**: macOS, Linux, or Windows (WSL2 for Windows)

### Required

- **Claude Code**: Download from https://claude.ai/code
- **Rust** (if building from source): 1.75 or later

### Optional

- **Ollama**: For local LLM support (https://ollama.ai)
- **Docker**: For containerized deployment

---

## Quick Install

```bash
# Download and install CCO
curl -fsSL https://cco.visiquate.com/install.sh | sh

# Verify installation
cco --version

# Start daemon
cco daemon start

# Verify temp files created
ls -la $TMPDIR/.cco-*

# Launch Claude Code
cd ~/my-project
cco
```

---

## Installation Methods

### Method 1: Official Installation Script (Recommended)

```bash
# Download and install CCO
curl -fsSL https://cco.visiquate.com/install.sh | sh

# The script will:
# - Download the latest CCO binary
# - Install to ~/.local/bin/cco
# - Add to PATH if needed
# - Verify installation
```

### Method 2: Build from Source

```bash
# Clone repository
git clone https://github.com/visiquate/cco.git
cd cco

# Build release binary
cargo build --release

# Install to system
sudo ln -sf $(pwd)/target/release/cco /usr/local/bin/cco

# Verify installation
cco version
```

### Method 3: Docker

```bash
# Pull image
docker pull cco:latest

# Run container
docker run -d -p 3000:3000 \
  -e ANTHROPIC_API_KEY="sk-ant-..." \
  --name cco \
  cco:latest
```

---

## Verify Installation

Follow these steps to verify CCO is installed and working correctly.

### Step 1: Check Installation

```bash
# Verify CCO is in PATH
which cco
# Output: /usr/local/bin/cco (or ~/.local/bin/cco)

# Check version
cco version
# Output: CCO version 2025.11.17+abc123
```

### Step 2: Configure API Keys

```bash
# Set Anthropic API key (required)
export ANTHROPIC_API_KEY="sk-ant-..."

# Optional: Add to shell profile for persistence
echo 'export ANTHROPIC_API_KEY="sk-ant-..."' >> ~/.zshrc  # or ~/.bashrc

# Verify key is set
echo $ANTHROPIC_API_KEY
# Should show your key
```

### Step 3: Start Daemon

```bash
# Start daemon (first time may take a few seconds)
cco daemon start

# Expected output:
# ✅ Daemon started successfully
#    PID: 12345
#    Temp files: $TMPDIR/.cco-*
#    Dashboard: http://localhost:3000

# Check daemon status
cco daemon status
# Output: Daemon is running (PID: 12345)
```

### Step 4: Verify Temp Files Created

```bash
# List temp files
ls -la $TMPDIR/.cco-*

# Expected output:
# -rw------- 1 user user 123456 Nov 17 10:00 .cco-orchestrator-settings
# -rw------- 1 user user   1234 Nov 17 10:00 .cco-agents-sealed
# -rw------- 1 user user    567 Nov 17 10:00 .cco-rules-sealed
# -rw------- 1 user user    890 Nov 17 10:00 .cco-hooks-sealed

# All files should be encrypted (binary data)
```

### Step 5: Launch Claude Code

```bash
# Navigate to test project
cd ~/test-project

# Launch Claude Code with orchestration
cco

# Expected output:
# ✅ Daemon is running
# ✅ Settings configured
# ✅ Orchestration environment configured
# 🚀 Launching Claude Code with orchestration support...
#    Working directory: /Users/you/test-project
#    Settings: $TMPDIR/.cco-orchestrator-settings

# Claude Code should launch with ORCHESTRATOR_* variables set
```

### Step 6: Monitor (Optional)

In another terminal:

```bash
# Launch TUI dashboard
cco tui

# Expected output:
# ✅ Daemon is running
# 🎯 Launching TUI dashboard...

# Should show real-time metrics dashboard
```

---

## Installation as System Service (Optional)

To have CCO daemon start automatically on boot:

### macOS (launchd)

```bash
# Install daemon as system service
cco daemon install

# Enable auto-start on boot
cco daemon enable

# Start service
cco daemon start

# Check status
cco daemon status
```

### Linux (systemd)

```bash
# Install systemd service
cco daemon install

# Enable auto-start on boot
sudo systemctl enable cco

# Start service
sudo systemctl start cco

# Check status
sudo systemctl status cco
```

### Windows (WSL2)

```bash
# WSL2 doesn't have systemd by default
# Use Windows Task Scheduler or manual start

# Add to WSL startup script
echo 'cco daemon start' >> ~/.profile
```

---

## Troubleshooting Installation

### Issue: Permission denied starting daemon

**Cause:** First-time daemon start may need proper permissions

**Solution:**
```bash
# Create temp directory if needed (usually automatic)
mkdir -p $TMPDIR

# Verify temp directory is writable
touch $TMPDIR/test && rm $TMPDIR/test

# If fails, check permissions
ls -la $TMPDIR

# Start daemon
cco daemon start
```

### Issue: Temp files not found

**Cause:** Daemon didn't start successfully or temp files were cleaned up

**Solution:**
```bash
# Restart daemon to recreate temp files
cco daemon restart

# Verify files created
ls -la $TMPDIR/.cco-*

# If still missing, check daemon logs
cco daemon logs | grep -i "temp\|file"
```

### Issue: Claude Code not found

**Cause:** Claude Code not installed or not in PATH

**Solution:**
```bash
# Install Claude Code from official site
# https://claude.ai/code

# Verify installation
which claude-code
# or
which claude

# If not in PATH, add to shell profile
export PATH="$PATH:/path/to/claude-code"
```

### Issue: Port 3000 already in use

**Cause:** Another service is using port 3000

**Solution:**
```bash
# Find what's using the port
lsof -i :3000

# Kill the process (if safe to do so)
kill <PID>

# Or start CCO on different port
cco daemon start --port 3001

# Update environment variable
export ORCHESTRATOR_API_URL=http://localhost:3001
```

### Issue: Daemon won't start - "Address already in use"

**Cause:** Previous daemon instance still running

**Solution:**
```bash
# Stop existing daemon
cco daemon stop

# If that doesn't work, find and kill process
ps aux | grep cco
kill <PID>

# Clean up lock file (if exists)
rm -f /tmp/cco.lock

# Start fresh
cco daemon start
```

### Issue: Settings file verification failed

**Cause:** Temp files corrupted or encryption key mismatch

**Solution:**
```bash
# Remove corrupted files
rm -f $TMPDIR/.cco-*

# Restart daemon to recreate
cco daemon restart

# Verify files are recreated
ls -la $TMPDIR/.cco-*
```

---

## Uninstallation

### Remove CCO Binary

```bash
# If installed via script
rm ~/.local/bin/cco

# If installed to /usr/local/bin
sudo rm /usr/local/bin/cco
```

### Remove System Service

```bash
# Disable and uninstall service
cco daemon disable
cco daemon uninstall

# Or manually (macOS)
launchctl unload ~/Library/LaunchAgents/com.visiquate.cco.plist
rm ~/Library/LaunchAgents/com.visiquate.cco.plist

# Or manually (Linux)
sudo systemctl stop cco
sudo systemctl disable cco
sudo rm /etc/systemd/system/cco.service
```

### Remove Data and Configuration

```bash
# Remove temp files (automatic on daemon stop)
cco daemon stop

# Remove database and logs
rm -f ~/.cco/analytics.db
rm -rf ~/.cco/logs

# Remove configuration
rm -rf ~/.config/cco
```

---

## Post-Installation

### Recommended Next Steps

1. **Configure auto-updates:**
   ```bash
   cco config set auto_update enabled
   ```

2. **Set up monitoring:**
   ```bash
   # Enable metrics export
   cco config set metrics_enabled true
   ```

3. **Customize daemon settings:**
   ```bash
   # Set cache size (in MB)
   cco config set cache_size 1000

   # Set cache TTL (in seconds)
   cco config set cache_ttl 7200
   ```

4. **Read documentation:**
   - [Command Reference](./commands.md) - Complete command reference
   - [Troubleshooting](./troubleshooting.md) - Common issues
   - [README](../README.md) - Project overview

---

## Upgrading CCO

### Check for Updates

```bash
# Check if update available
cco update --check

# Output:
# Current version: 2025.11.17
# Latest version:  2025.11.18
# Update available!
```

### Install Update

```bash
# Update to latest version
cco update

# With auto-confirmation
cco update --yes

# Expected output:
# Downloading CCO 2025.11.18...
# Installing update...
# ✅ Update complete
# Please restart daemon: cco daemon restart
```

### Restart After Update

```bash
# Restart daemon to load new version
cco daemon restart

# Verify new version
cco version
# Output: CCO version 2025.11.18+def456
```

---

## Platform-Specific Notes

### macOS

- **No special requirements** - Works out of the box
- Uses standard macOS temp directory (`$TMPDIR`)
- No kernel extensions needed
- Runs as standard user

### Linux

- **No special requirements** - Works out of the box
- Uses `/tmp/` as temp directory
- No FUSE module required
- SELinux: May need policy adjustments (rare)

### Windows (WSL2)

- **WSL2 required** (not WSL1)
- Uses Windows temp directory via WSL
- No special setup needed
- May need to update WSL: `wsl --update`

---

## Getting Help

If you encounter issues during installation:

1. Check [Troubleshooting](#troubleshooting-installation) section above
2. Enable debug logging: `cco daemon logs --follow`
3. Check daemon status: `cco daemon status`
4. Verify temp files: `ls -la $TMPDIR/.cco-*`
5. Open issue on GitHub with logs (remove sensitive data)

**Common Issues:**
- Port conflicts → Use different port
- Temp files missing → Restart daemon
- Claude Code not found → Add to PATH
- Permission errors → Check temp directory permissions
