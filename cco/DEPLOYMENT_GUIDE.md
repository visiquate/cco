# CCO Deployment Guide

**Complete guide for deploying CCO as a production daemon**

## Table of Contents

1. [Installation Methods](#installation-methods)
2. [Configuration](#configuration)
3. [Daemon Setup](#daemon-setup)
4. [Monitoring](#monitoring)
5. [Updates](#updates)
6. [Troubleshooting](#troubleshooting)

---

## 1. Installation Methods

### 1.1 One-Line Install (Recommended)

**macOS/Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/USER/cc-orchestra/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/USER/cc-orchestra/main/install.ps1 | iex
```

### 1.2 Homebrew (macOS)

```bash
# Add tap
brew tap visiquate/cco

# Install CCO
brew install cco

# Start service (auto-start on boot)
brew services start cco

# Check status
brew services list | grep cco
```

### 1.3 Scoop (Windows)

```powershell
# Add bucket
scoop bucket add cco https://github.com/USER/scoop-cco

# Install CCO
scoop install cco

# Check installation
cco version
```

### 1.4 Manual Installation

**Download latest release:**
```bash
# Visit: https://github.com/USER/cc-orchestra/releases/latest

# macOS Intel
curl -L https://github.com/USER/cc-orchestra/releases/latest/download/cco-macos-intel.tar.gz -o cco.tar.gz

# macOS ARM
curl -L https://github.com/USER/cc-orchestra/releases/latest/download/cco-macos-arm.tar.gz -o cco.tar.gz

# Windows
# Download cco-windows-x64.zip from releases page
```

**Install:**
```bash
# macOS/Linux
tar xzf cco.tar.gz
sudo mv cco /usr/local/bin/
sudo chmod +x /usr/local/bin/cco

# Windows
# Extract cco.exe from zip
# Move to C:\Program Files\CCO\cco.exe
# Add C:\Program Files\CCO to PATH
```

---

## 2. Configuration

### 2.1 Environment Variables

**Required:**
```bash
# Anthropic API key
export ANTHROPIC_API_KEY="sk-ant-..."
```

**Optional:**
```bash
export CCO_PORT=3000                    # Server port (default: 3000)
export CCO_HOST=127.0.0.1              # Bind address (default: 127.0.0.1)
export CCO_DATABASE_URL=sqlite://...   # Database path
export CCO_CACHE_SIZE=1073741824       # Cache size in bytes (1GB)
export CCO_CACHE_TTL=3600              # Cache TTL in seconds
export CCO_LOG_LEVEL=info              # Log level (debug, info, warn, error)
export NO_BROWSER=1                    # Disable auto-opening browser
```

### 2.2 Configuration File

**Location:**
- macOS/Linux: `~/.config/cco/config.toml`
- Windows: `%APPDATA%\cco\config.toml`

**Example config.toml:**
```toml
[server]
host = "127.0.0.1"
port = 3000
auto_browser = true

[cache]
size = 1073741824  # 1 GB
ttl = 3600         # 1 hour

[database]
path = "~/.local/share/cco/analytics.db"

[logging]
level = "info"
file = "~/.local/share/cco/logs/cco.log"

[updates]
auto_check = true
channel = "stable"  # or "beta"

[rate_limit]
requests_per_minute = 100
max_connections_per_ip = 10
```

### 2.3 API Key Management

**Option 1: Environment Variable (Simple)**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

**Option 2: Encrypted Storage (Recommended)**
```bash
# Store encrypted
cco credentials store anthropic sk-ant-...

# List credentials
cco credentials list

# Retrieve
cco credentials retrieve anthropic
```

**Option 3: System Keychain (Most Secure)**
- macOS: Uses Keychain Access
- Windows: Uses Credential Manager
- Linux: Uses libsecret

---

## 3. Daemon Setup

### 3.1 macOS (launchd)

**Automatic Installation:**
```bash
# Install and start service
cco install

# Manual management:
launchctl load ~/Library/LaunchAgents/com.visiquate.cco.plist
launchctl unload ~/Library/LaunchAgents/com.visiquate.cco.plist
```

**Service File:** `~/Library/LaunchAgents/com.visiquate.cco.plist`
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.visiquate.cco</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/cco</string>
        <string>run</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>Crashed</key>
        <true/>
        <key>SuccessfulExit</key>
        <false/>
    </dict>

    <key>StandardOutPath</key>
    <string>/Users/USERNAME/.local/share/cco/logs/cco.log</string>

    <key>StandardErrorPath</key>
    <string>/Users/USERNAME/.local/share/cco/logs/cco.log</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>ANTHROPIC_API_KEY</key>
        <string>YOUR_API_KEY_HERE</string>
    </dict>

    <key>ProcessType</key>
    <string>Background</string>
</dict>
</plist>
```

**Management Commands:**
```bash
# Start
launchctl start com.visiquate.cco

# Stop
launchctl stop com.visiquate.cco

# Check status
launchctl list | grep cco

# View logs
tail -f ~/.local/share/cco/logs/cco.log
```

### 3.2 Linux (systemd)

**Service File:** `/etc/systemd/system/cco.service`
```ini
[Unit]
Description=CCO - Claude Code Orchestra API Cost Monitor
Documentation=https://github.com/USER/cc-orchestra
After=network.target

[Service]
Type=simple
User=cco
Group=cco
WorkingDirectory=/opt/cco

# Binary location
ExecStart=/usr/local/bin/cco run

# Environment
Environment="ANTHROPIC_API_KEY=sk-ant-..."
Environment="CCO_PORT=3000"
Environment="CCO_HOST=127.0.0.1"

# Restart behavior
Restart=on-failure
RestartSec=10
KillMode=mixed
TimeoutStopSec=10

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=cco

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/cco /var/log/cco

[Install]
WantedBy=multi-user.target
```

**Setup:**
```bash
# Create user
sudo useradd -r -s /bin/false cco

# Create directories
sudo mkdir -p /var/lib/cco /var/log/cco
sudo chown cco:cco /var/lib/cco /var/log/cco

# Install service file
sudo cp cco.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable and start
sudo systemctl enable cco
sudo systemctl start cco

# Check status
sudo systemctl status cco

# View logs
sudo journalctl -u cco -f
```

### 3.3 Windows (Windows Service)

**Method 1: sc command**
```powershell
# Run as Administrator

# Create service
sc create CCO binPath= "C:\Program Files\CCO\cco.exe run" start= auto

# Set description
sc description CCO "CCO - Claude Code Orchestra API Cost Monitor"

# Start service
sc start CCO

# Check status
sc query CCO

# Stop service
sc stop CCO

# Delete service
sc delete CCO
```

**Method 2: NSSM (Recommended)**
```powershell
# Download NSSM from https://nssm.cc/download
# Extract nssm.exe

# Install service
nssm install CCO "C:\Program Files\CCO\cco.exe" "run"

# Configure service
nssm set CCO AppEnvironmentExtra "ANTHROPIC_API_KEY=sk-ant-..."
nssm set CCO AppStdout "C:\ProgramData\CCO\logs\cco.log"
nssm set CCO AppStderr "C:\ProgramData\CCO\logs\cco.log"
nssm set CCO AppRotateFiles 1
nssm set CCO AppRotateOnline 1

# Start service
nssm start CCO

# Check status
nssm status CCO

# Stop service
nssm stop CCO

# Remove service
nssm remove CCO confirm
```

**Method 3: Task Scheduler (Alternative)**
```powershell
# Create task
$action = New-ScheduledTaskAction -Execute "C:\Program Files\CCO\cco.exe" -Argument "run"
$trigger = New-ScheduledTaskTrigger -AtStartup
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable

Register-ScheduledTask -TaskName "CCO" -Action $action -Trigger $trigger -Principal $principal -Settings $settings

# Start task
Start-ScheduledTask -TaskName "CCO"

# Check status
Get-ScheduledTask -TaskName "CCO"

# Stop task
Stop-ScheduledTask -TaskName "CCO"
```

---

## 4. Monitoring

### 4.1 Health Checks

**HTTP Endpoint:**
```bash
# Basic health check
curl http://localhost:3000/health

# Response:
{
  "status": "ok",
  "version": "2025.11.2",
  "uptime": 12345,
  "cache_stats": {
    "hit_rate": 0.75,
    "hits": 150,
    "misses": 50,
    "entries": 200,
    "total_savings": 25.50
  }
}
```

**Ready Check (for automation):**
```bash
curl http://localhost:3000/ready

# Response:
{
  "ready": true,
  "version": "2025.11.2",
  "timestamp": "2025-11-17T12:00:00Z"
}
```

### 4.2 Status Command

```bash
# Check running instances
cco status

# Output:
CCO Status:
  Port 3000: Running (PID 12345)
  Uptime: 2h 15m
  Version: 2025.11.2
  Health: OK
  Cache Hit Rate: 75%
  Total Savings: $25.50
```

### 4.3 Log Viewing

```bash
# View logs (cross-platform)
cco logs --follow

# View last 100 lines
cco logs --lines 100

# View logs for specific port
cco logs --port 3000 --follow
```

**Log Locations:**
- macOS/Linux: `~/.local/share/cco/logs/cco.log`
- Windows: `%LOCALAPPDATA%\cco\logs\cco.log`

### 4.4 Prometheus Metrics (Optional)

```bash
# Expose metrics
curl http://localhost:3000/metrics

# Example metrics:
cco_requests_total{model="claude-opus-4"} 150
cco_cache_hits_total 112
cco_cache_misses_total 38
cco_cost_savings_dollars 25.50
cco_uptime_seconds 8145
```

---

## 5. Updates

### 5.1 Automatic Update Checking

**On Startup:**
```bash
cco run
# Output:
# ℹ️  New version available: 2025.11.3 (current: 2025.11.2)
#    Run 'cco update' to upgrade
```

**Manual Check:**
```bash
cco update --check

# Output:
# Current version: 2025.11.2
# Latest version: 2025.11.3
# Release notes: https://github.com/USER/cc-orchestra/releases/tag/v2025.11.3
```

### 5.2 Update Process

**Interactive:**
```bash
cco update

# Output:
# Checking for updates...
# New version available: 2025.11.3 (current: 2025.11.2)
#
# Release Notes:
# - Bug fixes
# - Performance improvements
#
# Download and install? [y/N]: y
#
# Downloading... ████████████████████ 100%
# Verifying checksum... ✅
# Installing... ✅
# Restarting service... ✅
#
# Successfully updated to 2025.11.3
```

**Automatic (no confirmation):**
```bash
cco update --yes
```

**Choose Channel:**
```bash
# Stable (default)
cco update --channel stable

# Beta releases
cco update --channel beta
```

### 5.3 Rollback

```bash
# List installed versions
cco version --history

# Rollback to previous version
cco rollback

# Rollback to specific version
cco rollback 2025.11.1
```

---

## 6. Troubleshooting

### 6.1 Common Issues

**Port Already in Use:**
```bash
# Check what's using the port
lsof -i :3000  # macOS/Linux
netstat -ano | findstr :3000  # Windows

# Solution 1: Use different port
cco run --port 8888

# Solution 2: Shutdown existing instance
cco shutdown --all
```

**Permission Denied (macOS):**
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/cco

# Or allow in System Preferences:
# Security & Privacy → General → "Allow anyway"
```

**Windows SmartScreen:**
1. Click "More info"
2. Click "Run anyway"

**API Key Not Found:**
```bash
# Check environment variable
echo $ANTHROPIC_API_KEY  # Unix
echo %ANTHROPIC_API_KEY%  # Windows
$env:ANTHROPIC_API_KEY    # PowerShell

# Store in config
cco credentials store anthropic sk-ant-...

# Verify
cco credentials list
```

**Database Locked:**
```bash
# Stop all instances
cco shutdown --all

# Remove lock file
rm ~/.local/share/cco/analytics.db-wal
rm ~/.local/share/cco/analytics.db-shm

# Restart
cco run
```

### 6.2 Debug Mode

```bash
# Start with debug logging
cco run --debug

# Or set environment variable
export RUST_LOG=debug
cco run

# View detailed logs
cco logs --follow
```

### 6.3 Service Not Starting

**macOS (launchd):**
```bash
# Check service status
launchctl list | grep cco

# View errors
launchctl error 0

# Restart service
launchctl unload ~/Library/LaunchAgents/com.visiquate.cco.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.cco.plist
```

**Linux (systemd):**
```bash
# Check status
sudo systemctl status cco

# View logs
sudo journalctl -u cco -n 50

# Restart service
sudo systemctl restart cco

# Check configuration
sudo systemd-analyze verify cco.service
```

**Windows (Service):**
```powershell
# Check event logs
Get-EventLog -LogName Application -Source CCO -Newest 10

# Check service status
Get-Service CCO

# Restart service
Restart-Service CCO

# View logs
Get-Content $env:LOCALAPPDATA\cco\logs\cco.log -Tail 50
```

### 6.4 Performance Issues

**High Memory Usage:**
```bash
# Reduce cache size
export CCO_CACHE_SIZE=536870912  # 512 MB instead of 1 GB
cco run

# Or in config.toml:
[cache]
size = 536870912
```

**Slow Responses:**
```bash
# Check cache hit rate
curl http://localhost:3000/api/stats

# Clear cache if corrupted
curl -X POST http://localhost:3000/api/cache/clear

# Restart service
cco shutdown --all
cco run
```

### 6.5 Getting Help

**Command Help:**
```bash
cco --help
cco run --help
cco update --help
```

**System Info:**
```bash
cco version
cco health
cco status
```

**Community Support:**
- GitHub Issues: https://github.com/USER/cc-orchestra/issues
- Discussions: https://github.com/USER/cc-orchestra/discussions

---

## Appendix: Production Deployment Checklist

### Pre-Deployment

- [ ] Set `ANTHROPIC_API_KEY` securely
- [ ] Configure appropriate cache size
- [ ] Set up log rotation
- [ ] Review security settings (bind host, rate limits)
- [ ] Test health endpoints

### Deployment

- [ ] Install CCO binary
- [ ] Create configuration file
- [ ] Install daemon/service
- [ ] Verify service starts automatically
- [ ] Check health endpoint
- [ ] Monitor logs for errors

### Post-Deployment

- [ ] Enable automatic updates
- [ ] Set up monitoring alerts
- [ ] Document instance location and port
- [ ] Test failover/restart behavior
- [ ] Verify log rotation working

### Ongoing Maintenance

- [ ] Monitor cache hit rate
- [ ] Review cost savings weekly
- [ ] Check for updates monthly
- [ ] Review logs for errors
- [ ] Test backup/restore procedures
