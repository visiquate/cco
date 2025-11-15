# CCO Troubleshooting Guide

Common issues and solutions for CCO (Claude Code Orchestra).

## Table of Contents

- [Installation Issues](#installation-issues)
- [Proxy Issues](#proxy-issues)
- [Cache Issues](#cache-issues)
- [API and Connectivity Issues](#api-and-connectivity-issues)
- [Performance Issues](#performance-issues)
- [Configuration Issues](#configuration-issues)
- [Update Issues](#update-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Error Messages](#error-messages)
- [Debugging](#debugging)
- [Getting Help](#getting-help)

## Installation Issues

### Binary Not Found After Installation

**Symptoms**: `cco: command not found` after installation

**Solutions**:

```bash
# 1. Check if binary exists
ls -la ~/.local/bin/cco

# 2. Verify PATH includes ~/.local/bin
echo $PATH | grep ".local/bin"

# 3. If not in PATH, add it
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
source ~/.bashrc  # or source ~/.zshrc

# 4. Verify
which cco
cco --version
```

### Permission Denied Error

**Symptoms**: `Permission denied` when trying to run `cco`

**Solutions**:

```bash
# 1. Make binary executable
chmod +x ~/.local/bin/cco

# 2. Verify permissions
ls -l ~/.local/bin/cco
# Should show: -rwxr-xr-x

# 3. If directory permissions wrong
chmod 755 ~/.local/bin
```

### Installation Script Fails

**Symptoms**: Install script exits with error

**Solutions**:

```bash
# 1. Check curl/wget availability
which curl
which wget

# 2. Check internet connectivity
ping github.com

# 3. Try manual installation
curl -L -o cco.tar.gz https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-arm64.tar.gz
tar -xzf cco.tar.gz
mkdir -p ~/.local/bin
mv cco ~/.local/bin/
chmod +x ~/.local/bin/cco

# 4. If behind proxy
export HTTPS_PROXY="http://proxy.company.com:8080"
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash
```

### Checksum Verification Failed

**Symptoms**: Installation fails with "Checksum verification failed"

**Solutions**:

```bash
# 1. Clear download cache and retry
rm -rf ~/.cache/cco/downloads
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# 2. Download checksums manually
curl -L -o checksums.sha256 https://github.com/brentley/cco-releases/releases/download/v0.2.0/checksums.sha256

# 3. Verify manually
sha256sum -c checksums.sha256 --ignore-missing

# 4. If still fails, report issue (possible corrupted release)
```

## Proxy Issues

### Port Already in Use

**Symptoms**: `Address already in use` or `Failed to bind to port 8000`

**Solutions**:

```bash
# 1. Find process using port
lsof -i :8000  # macOS/Linux
netstat -ano | findstr :8000  # Windows

# 2. Kill the process
kill -9 <PID>  # macOS/Linux
taskkill /PID <PID> /F  # Windows

# 3. Or use a different port
cco proxy --port 9000
```

### Proxy Not Responding

**Symptoms**: Connection refused or timeout when accessing proxy

**Solutions**:

```bash
# 1. Check if proxy is running
ps aux | grep cco
curl http://localhost:8000/health

# 2. Check binding address
# If started with --host 127.0.0.1, only localhost can connect
# For network access, use --host 0.0.0.0
cco proxy --host 0.0.0.0 --port 8000

# 3. Check firewall
# macOS
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add $(which cco)

# Linux
sudo ufw allow 8000/tcp

# Windows
netsh advfirewall firewall add rule name="CCO" dir=in action=allow protocol=TCP localport=8000

# 4. Check logs
tail -f ~/.config/cco/cco.log
```

### Proxy Crashes or Exits Unexpectedly

**Symptoms**: Proxy stops running without clear error

**Solutions**:

```bash
# 1. Run in foreground to see errors
cco proxy --log-level debug

# 2. Check system resources
free -h  # Linux
vm_stat  # macOS
# Look for out-of-memory issues

# 3. Reduce cache size
cco config set cache.max_capacity 500

# 4. Check disk space
df -h

# 5. Review logs
tail -100 ~/.config/cco/cco.log
```

### 502 Bad Gateway Errors

**Symptoms**: Requests return 502 Bad Gateway

**Solutions**:

```bash
# 1. Check upstream API connectivity
curl -I https://api.anthropic.com/v1/messages

# 2. Verify API key is set
echo $ANTHROPIC_API_KEY

# 3. Check timeout settings
cco config set proxy.request_timeout 120

# 4. Check routing configuration
cco config show | grep routes

# 5. Test direct API call
curl -X POST https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-sonnet-3.5","messages":[{"role":"user","content":"test"}],"max_tokens":10}'
```

## Cache Issues

### Cache Not Working

**Symptoms**: Cache hit rate is 0% or very low

**Solutions**:

```bash
# 1. Verify cache is enabled
cco config get cache.enabled
cco config set cache.enabled true

# 2. Check cache stats
cco cache stats

# 3. Verify identical requests
# Cache keys include request body, model, and headers
# Slight variations prevent cache hits

# 4. Check TTL
cco config get cache.ttl_seconds
# If too short, entries expire quickly

# 5. Check cache size
cco cache stats | grep "Current Size"
# If full, entries are evicted

# 6. Increase capacity
cco config set cache.max_capacity 2000
```

### Cache Using Too Much Memory

**Symptoms**: High memory usage, system slowdown

**Solutions**:

```bash
# 1. Check cache memory usage
cco cache stats | grep "Memory Usage"

# 2. Reduce cache capacity
cco config set cache.max_capacity 500

# 3. Reduce TTL
cco config set cache.ttl_seconds 1800  # 30 minutes

# 4. Clear cache
cco cache clear

# 5. Disable caching temporarily
cco proxy --no-cache
```

### Cache Not Persisting

**Symptoms**: Cache resets after restart

**Solutions**:

```bash
# Cache is in-memory by default
# To persist cache:

# 1. Enable persistence
cco config set cache.persist_enabled true
cco config set cache.persist_path "~/.cache/cco/cache.db"

# 2. Verify settings
cco config get cache.persist_enabled
```

## API and Connectivity Issues

### API Key Not Working

**Symptoms**: 401 Unauthorized errors

**Solutions**:

```bash
# 1. Verify API key is set
echo $ANTHROPIC_API_KEY
# Should start with "sk-ant-"

# 2. Test API key directly
curl -X POST https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-sonnet-3.5","messages":[{"role":"user","content":"test"}],"max_tokens":10}'

# 3. Check for typos or extra whitespace
export ANTHROPIC_API_KEY="$(echo $ANTHROPIC_API_KEY | tr -d '[:space:]')"

# 4. Verify key hasn't expired or been revoked
# Check Anthropic console: https://console.anthropic.com/

# 5. Check provider configuration
cco config show | grep -A 5 "providers.anthropic"
```

### Connection Timeout

**Symptoms**: Requests timeout or take too long

**Solutions**:

```bash
# 1. Increase timeout
cco config set proxy.request_timeout 120

# 2. Check network latency
ping api.anthropic.com
traceroute api.anthropic.com  # macOS/Linux
tracert api.anthropic.com     # Windows

# 3. Check for proxy/firewall
curl -v https://api.anthropic.com/v1/messages

# 4. Use shorter timeout with retry
cco config set proxy.request_timeout 60
cco config set proxy.max_retries 3

# 5. Check if upstream API is down
# Visit: https://status.anthropic.com/
```

### SSL/TLS Certificate Errors

**Symptoms**: Certificate verification failed

**Solutions**:

```bash
# 1. Update CA certificates (Linux)
sudo apt-get update
sudo apt-get install ca-certificates

# 2. Update system (macOS)
softwareupdate --all --install --force

# 3. Check system time
date
# Incorrect system time causes certificate errors

# 4. Test SSL connection
openssl s_client -connect api.anthropic.com:443
```

### Rate Limiting Errors

**Symptoms**: 429 Too Many Requests

**Solutions**:

```bash
# 1. Check rate limit status
curl http://localhost:8000/api/stats/rate-limits

# 2. Configure client-side rate limiting
cco config set security.rate_limit_per_minute 60

# 3. Reduce request frequency
# Add delays between requests in your application

# 4. Contact Anthropic for rate limit increase
# https://console.anthropic.com/

# 5. Use cache more effectively
cco config set cache.ttl_seconds 7200  # Longer TTL
```

## Performance Issues

### High Latency

**Symptoms**: Requests are slow, high response times

**Solutions**:

```bash
# 1. Check cache hit rate
cco cache stats | grep "Hit Rate"
# Low hit rate means more API calls

# 2. Optimize cache
cco config set cache.strategy lfu  # Least Frequently Used
cco config set cache.max_capacity 5000

# 3. Check upstream latency
time curl -X POST https://api.anthropic.com/v1/messages ...

# 4. Reduce logging overhead
cco config set log_level warn

# 5. Check system resources
top  # CPU usage
free -h  # Memory usage
```

### Memory Leaks

**Symptoms**: Memory usage grows over time

**Solutions**:

```bash
# 1. Monitor memory usage
watch -n 5 'ps aux | grep cco | grep -v grep'

# 2. Reduce cache size
cco config set cache.max_capacity 500

# 3. Enable periodic cache pruning
cco config set cache.idle_timeout 1800  # 30 minutes

# 4. Restart proxy periodically (workaround)
crontab -e
# Add: 0 3 * * * pkill cco && sleep 5 && cco proxy --daemon

# 5. Report issue with reproduction steps
# https://github.com/brentley/cco-releases/issues
```

### High CPU Usage

**Symptoms**: CPU usage consistently high

**Solutions**:

```bash
# 1. Reduce worker threads
cco config set proxy.workers 2

# 2. Reduce log level
cco config set log_level error

# 3. Disable analytics temporarily
cco proxy --no-analytics

# 4. Check for infinite loops in logs
tail -f ~/.config/cco/cco.log

# 5. Profile the application
# Enable debug logging and look for hot spots
```

## Configuration Issues

### Configuration Not Loading

**Symptoms**: Changes to config.toml not taking effect

**Solutions**:

```bash
# 1. Verify config file location
ls -la ~/.config/cco/config.toml

# 2. Check syntax
cco config validate

# 3. Verify configuration is loaded
cco config show --effective

# 4. Restart proxy (config loaded on startup)
pkill cco
cco proxy

# 5. Check for syntax errors
# Common issues:
# - Missing quotes around strings
# - Wrong indentation
# - Typos in keys
```

### Environment Variables Not Working

**Symptoms**: Environment variables ignored

**Solutions**:

```bash
# 1. Verify variables are set
env | grep CCO
env | grep ANTHROPIC

# 2. Export variables (don't just set them)
export ANTHROPIC_API_KEY="sk-ant-..."  # ✓ Correct
ANTHROPIC_API_KEY="sk-ant-..."         # ✗ Wrong

# 3. Check precedence
# Command-line > Environment > Config file
# Env vars might be overridden

# 4. Use --env-file option
echo "ANTHROPIC_API_KEY=sk-ant-..." > .env
cco proxy --env-file .env

# 5. Verify in effective config
cco config show --effective | grep api_key
```

### Configuration Permissions

**Symptoms**: Can't read/write configuration

**Solutions**:

```bash
# 1. Check file permissions
ls -l ~/.config/cco/config.toml
# Should be: -rw------- (600)

# 2. Fix permissions
chmod 600 ~/.config/cco/config.toml

# 3. Check directory permissions
ls -ld ~/.config/cco
# Should be: drwx------ (700)

# 4. Fix directory permissions
chmod 700 ~/.config/cco

# 5. Recreate config if needed
mv ~/.config/cco/config.toml ~/.config/cco/config.toml.backup
cco config init
```

## Update Issues

See [UPDATING.md](UPDATING.md) for detailed update troubleshooting.

Quick fixes:

```bash
# Update check fails
cco update --check --verbose

# Download fails
rm -rf ~/.cache/cco/downloads && cco update --install

# Rollback if update breaks
cco update --rollback

# Force reinstall
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash
```

## Platform-Specific Issues

### macOS

#### "Damaged and can't be opened"

**Solution**:
```bash
xattr -d com.apple.quarantine ~/.local/bin/cco
```

#### "Developer cannot be verified"

**Solution**:
1. System Preferences → Security & Privacy
2. Click "Open Anyway" next to CCO message

#### Homebrew Installation Issues

**Solution**:
```bash
# Update Homebrew
brew update

# Reinstall
brew uninstall cco
brew install cco
```

### Linux

#### "error while loading shared libraries"

**Solution**:
```bash
# Install missing libraries
sudo apt-get install libc6 libssl3  # Debian/Ubuntu
sudo yum install glibc openssl       # RHEL/CentOS
```

#### systemd Service Issues

**Solution**:
```bash
# Check service status
sudo systemctl status cco

# View logs
sudo journalctl -u cco -f

# Restart service
sudo systemctl restart cco

# Enable on boot
sudo systemctl enable cco
```

### Windows

#### "Missing VCRUNTIME140.dll"

**Solution**:
Download and install: [Microsoft Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)

#### PowerShell Execution Policy

**Solution**:
```powershell
# Allow script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Or run specific script
powershell -ExecutionPolicy Bypass -File install.ps1
```

#### Windows Defender SmartScreen

**Solution**:
1. Click "More info"
2. Click "Run anyway"

Or add exclusion:
```powershell
Add-MpPreference -ExclusionPath "$env:USERPROFILE\.local\bin"
```

## Error Messages

### "Failed to bind address"

**Cause**: Port already in use or permission denied

**Solution**:
```bash
# Use different port
cco proxy --port 9000

# Or find and stop conflicting process
lsof -i :8000
kill -9 <PID>
```

### "Database is locked"

**Cause**: Another CCO instance accessing database

**Solution**:
```bash
# Stop all CCO instances
pkill cco

# Check for stale lock
ls -la ~/.config/cco/analytics.db-lock
rm ~/.config/cco/analytics.db-lock

# Restart
cco proxy
```

### "Connection refused"

**Cause**: Proxy not running or wrong host/port

**Solution**:
```bash
# Check if running
ps aux | grep cco

# Start proxy
cco proxy --port 8000

# Verify connectivity
curl http://localhost:8000/health
```

### "Invalid API key"

**Cause**: API key incorrect or not set

**Solution**:
```bash
# Set API key
export ANTHROPIC_API_KEY="sk-ant-your-key-here"

# Verify
echo $ANTHROPIC_API_KEY

# Test directly
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  ...
```

## Debugging

### Enable Debug Logging

```bash
# Start with debug logging
cco proxy --log-level debug

# Or set in config
cco config set log_level debug

# View logs in real-time
tail -f ~/.config/cco/cco.log
```

### Trace Requests

```bash
# Enable request tracing
cco proxy --trace-requests --log-level trace

# Shows full request/response details
```

### Health Check

```bash
# Check system health
cco health

# Check specific components
cco health --check-cache
cco health --check-database
cco health --check-upstream
```

### Collect Diagnostics

```bash
# Generate diagnostic report
cco diagnostics > cco-diagnostics.txt

# Include:
# - Version info
# - Configuration (redacted)
# - Recent logs
# - Cache stats
# - System info
```

## Getting Help

If you can't resolve the issue:

### 1. Search Existing Issues

https://github.com/brentley/cco-releases/issues

### 2. Check Documentation

- [README.md](README.md)
- [INSTALLATION.md](INSTALLATION.md)
- [CONFIGURATION.md](CONFIGURATION.md)
- [USAGE.md](USAGE.md)
- [UPDATING.md](UPDATING.md)

### 3. Create New Issue

Include:

```bash
# System information
cco --version
uname -a  # macOS/Linux
systeminfo  # Windows

# Configuration (redacted)
cco config show --effective

# Recent logs
tail -100 ~/.config/cco/cco.log

# Steps to reproduce
# Expected behavior
# Actual behavior
```

### 4. Contact Support

Email: support@visiquate.com

Include diagnostic information from step 3.

## Common Command Reference

```bash
# Installation
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Start proxy
cco proxy --port 8000

# Check status
cco health

# View logs
tail -f ~/.config/cco/cco.log

# Clear cache
cco cache clear

# Update
cco update --check
cco update --install

# Rollback
cco update --rollback

# Reset configuration
cco config reset

# Uninstall
rm ~/.local/bin/cco
rm -rf ~/.config/cco
```

---

Last updated: 2025-11-15

Still having issues? Email support@visiquate.com with diagnostic information.
