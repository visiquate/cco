# CCO Installation Quick Start

**Version:** 2025.11.17
**Last Updated:** 2025-11-17

---

## Platform Support

| Platform | Status | System Requirements |
|----------|--------|-------------------|
| **macOS** | ✅ Fully Supported | macOS 10.13+ |
| **Linux** | ✅ Fully Supported | Any modern distribution |
| **Windows** | ✅ Fully Supported | Windows 10+ |

**🎉 No system dependencies required!** No macFUSE, no kernel extensions, no admin setup.

---

## macOS Installation

### One-Line Install
```bash
curl -fsSL https://cco.visiquate.com/install.sh | sh
```

### What Gets Installed
- Binary: `/usr/local/bin/cco`
- Config: `~/.config/cco/`
- Temp files: `/var/folders/xx/xxx/T/.cco-*`

### Verify Installation
```bash
# Check version
cco version

# Launch with orchestration
cco

# Monitor metrics
cco tui
```

### macOS-Specific Notes
- ✅ No macFUSE required (removed in v2025.11.17)
- ✅ No kernel extensions needed
- ✅ Works on Intel and Apple Silicon
- ✅ Gatekeeper quarantine automatically removed

---

## Linux Installation

### One-Line Install
```bash
curl -fsSL https://cco.visiquate.com/install.sh | sh
```

### What Gets Installed
- Binary: `/usr/local/bin/cco`
- Config: `~/.config/cco/`
- Temp files: `/tmp/.cco-*`

### Verify Installation
```bash
# Check version
cco version

# Launch with orchestration
cco

# Monitor metrics
cco tui
```

### Linux-Specific Notes
- ✅ No libfuse required (removed in v2025.11.17)
- ✅ Works on x86_64 and ARM64
- ✅ Compatible with all major distributions (Ubuntu, Debian, Fedora, Arch, etc.)

---

## Windows Installation

### PowerShell Install (Recommended)
```powershell
# Download installer
Invoke-WebRequest -Uri "https://cco.visiquate.com/install.ps1" -OutFile "install.ps1"

# Run installer
.\install.ps1
```

### Manual Install
1. Download Windows binary from [releases page](https://github.com/anthropics/claude-code/releases)
2. Extract `cco.exe` to `C:\Program Files\CCO\`
3. Add to PATH:
   ```powershell
   $env:Path += ";C:\Program Files\CCO"
   [Environment]::SetEnvironmentVariable("Path", $env:Path, [System.EnvironmentVariableTarget]::User)
   ```

### What Gets Installed
- Binary: `C:\Program Files\CCO\cco.exe`
- Config: `%APPDATA%\cco\`
- Temp files: `%TEMP%\.cco-*`

### Verify Installation
```powershell
# Check version
cco version

# Launch with orchestration
cco

# Monitor metrics
cco tui
```

### Windows-Specific Notes
- ✅ NEW in v2025.11.17!
- ✅ No system dependencies required
- ✅ Works on Windows 10 and 11
- ✅ PowerShell and CMD supported

---

## Configuration

### Set API Key
```bash
# macOS/Linux
export ANTHROPIC_API_KEY='sk-ant-...'

# Windows (PowerShell)
$env:ANTHROPIC_API_KEY = 'sk-ant-...'
```

### Optional: Make API Key Persistent
```bash
# macOS/Linux: Add to ~/.bashrc or ~/.zshrc
echo 'export ANTHROPIC_API_KEY="sk-ant-..."' >> ~/.bashrc

# Windows (PowerShell): Add to profile
echo '$env:ANTHROPIC_API_KEY = "sk-ant-..."' >> $PROFILE
```

---

## First Run

### Start CCO
```bash
# Launch Claude Code with orchestration
cco
```

This will:
1. Auto-start daemon (first run takes ~3 seconds)
2. Create temp files in OS temp directory
3. Set environment variables
4. Launch Claude Code in current directory

### Monitor Metrics
```bash
# In another terminal
cco tui
```

This shows:
- Real-time API cost tracking
- Agent activity
- Token usage
- Performance metrics

---

## Common Commands

### Daemon Management
```bash
# Start daemon
cco daemon start

# Stop daemon
cco daemon stop

# Restart daemon
cco daemon restart

# Check status
cco daemon status

# View logs
cco daemon logs
```

### Orchestration
```bash
# Launch with orchestration (default)
cco

# Pass arguments to Claude Code
cco --help
cco analyze code.py
cco [any-claude-code-args]

# Launch TUI dashboard
cco tui
```

### Configuration
```bash
# View config
cco config show

# Edit config
cco config edit

# Reset to defaults
cco config reset
```

---

## Troubleshooting

### Daemon Won't Start
```bash
# Check if port 3000 is in use
lsof -ti:3000  # macOS/Linux
netstat -ano | findstr :3000  # Windows

# Use different port
cco daemon start --port 3001
```

### Files Not Found
```bash
# Restart daemon to recreate files
cco daemon restart

# Verify files created (macOS/Linux)
ls -la $TMPDIR/.cco-*

# Verify files created (Windows)
Get-ChildItem $env:TEMP\.cco-*
```

### Claude Code Can't Find Settings
```bash
# Always launch via cco (sets environment variables)
cco

# NOT directly:
claude-code  # Won't have orchestration
```

### Permission Denied
```bash
# macOS/Linux: Check file permissions
ls -la /usr/local/bin/cco

# Should be executable (755)
chmod +x /usr/local/bin/cco

# Windows: Run as Administrator
Right-click → Run as administrator
```

---

## Upgrading

### Automatic Upgrade
```bash
# macOS/Linux
curl -fsSL https://cco.visiquate.com/install.sh | sh

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://cco.visiquate.com/install.ps1" | Invoke-Expression
```

### Manual Upgrade
1. Download latest release
2. Replace existing binary
3. Restart daemon: `cco daemon restart`

---

## Uninstalling

### macOS/Linux
```bash
# Stop daemon
cco daemon stop

# Remove binary
sudo rm /usr/local/bin/cco

# Remove config (optional)
rm -rf ~/.config/cco

# Temp files auto-cleaned on reboot
```

### Windows
```powershell
# Stop daemon
cco daemon stop

# Remove binary
Remove-Item "C:\Program Files\CCO\cco.exe"

# Remove config (optional)
Remove-Item -Recurse "$env:APPDATA\cco"

# Temp files auto-cleaned on reboot
```

---

## What Changed in v2025.11.17

### 🎉 No More System Dependencies!

**Before:**
- macOS: Required macFUSE installation
- Linux: Required libfuse
- Windows: Not supported

**After:**
- macOS: Zero dependencies ✅
- Linux: Zero dependencies ✅
- Windows: Fully supported ✅

### Migration Guide

If upgrading from older version:

1. **No action required** - All commands work the same
2. **Optional**: Uninstall macFUSE (if desired)
   ```bash
   # macOS
   brew uninstall --cask macfuse
   ```
3. **Benefit**: Simpler installation, faster startup

---

## Getting Help

- 📖 [Full Installation Guide](./cco/INSTALLATION.md)
- 🔧 [Troubleshooting Guide](./cco/TROUBLESHOOTING.md)
- 📚 [Usage Guide](./cco/USAGE.md)
- 🏗️ [Architecture Overview](./cco/docs/ARCHITECTURE_SIMPLIFIED.md)
- 🐛 [Report Issues](https://github.com/anthropics/claude-code/issues)

---

## Platform-Specific Resources

### macOS
- [macOS Installation Guide](./cco/INSTALLATION.md#macos)
- [macOS Troubleshooting](./cco/TROUBLESHOOTING.md#macos)

### Linux
- [Linux Installation Guide](./cco/INSTALLATION.md#linux)
- [Linux Troubleshooting](./cco/TROUBLESHOOTING.md#linux)

### Windows
- [Windows Installation Guide](./cco/INSTALLATION.md#windows)
- [Windows Troubleshooting](./cco/TROUBLESHOOTING.md#windows)

---

**Version:** 2025.11.17
**Status:** Production Ready
**Support:** All major platforms (macOS, Linux, Windows)
