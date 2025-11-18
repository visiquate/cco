# Auto-Update Administrator Guide

Complete guide to managing CCO updates in production environments.

## Overview

This guide covers deploying and managing CCO auto-updates across production systems. It includes strategies for controlled rollouts, monitoring update status, and handling failures.

## Deployment Strategies

### Strategy 1: Manual Updates (Recommended for Production)

**Best for:** High-availability production systems where downtime must be scheduled

**Setup:**

```bash
# On each instance, disable automatic updates
cco config set updates.enabled false
cco config set updates.check_interval never
```

**Update process:**
1. Check for updates in a development environment
2. Test the new version
3. Schedule maintenance window
4. Run updates on production servers:
```bash
cco update --yes
cco daemon restart
```

**Monitoring:**
```bash
# Before update
cco daemon status

# Check for available updates
cco update --check

# After update
cco daemon status
cco version
```

**Example script** (`/usr/local/bin/update-cco.sh`):
```bash
#!/bin/bash
set -e

echo "Checking for CCO updates..."
if cco update --check 2>&1 | grep -q "New version available"; then
    echo "Update available, installing..."
    cco update --yes

    echo "Restarting daemon..."
    cco daemon restart

    sleep 2

    if cco daemon status > /dev/null 2>&1; then
        echo "✅ Update successful"
        exit 0
    else
        echo "❌ Daemon restart failed"
        exit 1
    fi
else
    echo "No updates available"
    exit 0
fi
```

### Strategy 2: Staged Rollout

**Best for:** Large deployments where you want to test on a subset first

**Phase 1: Development/Testing**
```bash
# Enable beta channel
cco config set updates.channel beta
cco config set updates.auto_install false

# Manually test updates as they come in
cco update --check
```

**Phase 2: Staging Environment**
```bash
# Switch to stable, but don't auto-install
cco config set updates.channel stable
cco config set updates.auto_install false

# Verify in your staging environment before production rollout
cco update --check
```

**Phase 3: Production Rollout**
```bash
# Only after verification in staging
cco config set updates.channel stable
# Keep auto_install false for controlled updates
```

### Strategy 3: Automatic Updates with Monitoring

**Best for:** Smaller deployments or when you have good monitoring in place

**Setup:**

```bash
# Enable automatic updates
cco config set updates.enabled true
cco config set updates.check_interval daily
cco config set updates.auto_install true
cco config set updates.channel stable
```

**Monitoring requirements:**
- Health check API responding
- Service availability monitoring
- Log aggregation and alerts
- Version tracking

**Example health check script:**
```bash
#!/bin/bash

VERSION=$(cco version | head -1)
HEALTH=$(curl -s http://localhost:3000/health)

if echo "$HEALTH" | grep -q '"status":"ok"'; then
    echo "✅ CCO is healthy: $VERSION"
    exit 0
else
    echo "❌ CCO health check failed"
    exit 1
fi
```

## Multi-Instance Management

### Checking Updates Across Fleet

**Using SSH to check all servers:**

```bash
#!/bin/bash
# check-updates-fleet.sh

for server in prod-1 prod-2 prod-3; do
    echo "=== $server ==="
    ssh $server 'cco version && cco config show | grep -E "Last|channel"'
done
```

**Centralized status tracking:**

```bash
#!/bin/bash
# status-report.sh

echo "CCO Fleet Status Report"
echo "======================="

for server in prod-1 prod-2 prod-3; do
    VERSION=$(ssh $server cco version 2>/dev/null | head -1)
    CONFIG=$(ssh $server cco config show 2>/dev/null)
    CHANNEL=$(echo "$CONFIG" | grep "channel:" | awk '{print $2}')
    LAST_CHECK=$(echo "$CONFIG" | grep "Last check:" | sed 's/.*Last check: //')

    printf "%-10s Version: %-12s Channel: %-8s Last check: %s\n" \
        "$server" "$VERSION" "$CHANNEL" "$LAST_CHECK"
done
```

### Rolling Updates

**Update one server at a time with verification:**

```bash
#!/bin/bash
# rolling-update.sh

SERVERS=("prod-1" "prod-2" "prod-3")
HEALTH_ENDPOINT="http://localhost:3000/health"
WAIT_TIME=30

for server in "${SERVERS[@]}"; do
    echo "Updating $server..."

    # Run update
    ssh $server 'cco update --yes'

    # Restart
    ssh $server 'cco daemon restart'

    # Wait for service to start
    sleep $WAIT_TIME

    # Verify health
    if ssh $server "curl -s $HEALTH_ENDPOINT | grep -q ok"; then
        echo "✅ $server healthy"
    else
        echo "❌ $server failed health check - ROLLING BACK"
        ssh $server 'cp ~/.local/bin/cco.backup ~/.local/bin/cco && cco daemon restart'
        exit 1
    fi
done

echo "✅ All servers updated successfully"
```

## Monitoring Update Status

### Check Configuration Across Fleet

```bash
# View update settings on a server
cco config show

# Output:
# Update Configuration:
#   Enabled: true
#   Auto-install: false
#   Check interval: daily
#   Channel: stable
#   Last check: 2025-11-17 14:32:15 UTC
#   Last update: 2025-11-10 09:22:03 UTC
```

### Extract Specific Values

```bash
# Get just the last check time
cco config get updates.last_check

# Get just the channel
cco config get updates.channel

# Get auto-install status
cco config get updates.auto_install
```

### Monitoring Script

**Track updates in all your instances:**

```bash
#!/bin/bash
# monitor-updates.sh

# Create update tracking report
echo "update_status {" > /tmp/cco_updates.txt

for server in prod-1 prod-2 prod-3; do
    ssh $server bash -c '
        VERSION=$(cco version | head -1 | awk "{print \$3}")
        LAST=$(cco config get updates.last_check 2>/dev/null | awk "{print \$3, \$4}")
        CHANNEL=$(cco config get updates.channel 2>/dev/null | awk "{print \$3}")
        echo "  server=\"'$server'\":"
        echo "    version=\"$VERSION\""
        echo "    last_check=\"$LAST\""
        echo "    channel=\"$CHANNEL\""
    '
done >> /tmp/cco_updates.txt

echo "}" >> /tmp/cco_updates.txt

# Optionally send to monitoring system
# cat /tmp/cco_updates.txt | nc -w 1 metrics.example.com 9999
```

## Handling Failed Updates

### Automatic Rollback

The update system automatically handles failures:

1. **During download**: No files changed, old version continues
2. **During installation**: Backup is restored, old version continues
3. **After installation**: New binary is tested, automatic rollback on failure

**Evidence of rollback:**
```
⚠️  New binary verification failed, rolling back...
Update failed, rolled back to previous version
```

### Manual Rollback

If automatic rollback fails:

```bash
# Check if backup exists
ls -la ~/.local/bin/cco*

# Restore backup
cp ~/.local/bin/cco.backup ~/.local/bin/cco

# Verify
cco version

# Restart daemon
cco daemon restart
```

### Troubleshooting Failed Updates

**Check daemon logs for errors:**

```bash
# View latest logs
cco daemon logs

# Follow logs in real-time
cco daemon logs --follow

# Get specific number of lines
cco daemon logs -n 100
```

**Common failure scenarios:**

| Error | Cause | Solution |
|-------|-------|----------|
| "No release asset found for platform" | Unsupported OS/arch | Check `uname -m` and `uname -s` |
| "Checksum verification failed" | Network corruption | Retry update, check network |
| "Binary not found in archive" | Corrupted download | Retry update |
| "Failed to install new binary" | Permission issue | Fix `~/.local/bin` permissions |
| "New binary verification failed" | Binary incompatible | Check OS version compatibility |

## Network and Connectivity

### GitHub API Rate Limiting

The update system uses GitHub API (rate limit: 60 requests/hour unauthenticated)

**If you hit rate limits:**

```bash
# Wait before retrying
sleep 300  # Wait 5 minutes

# Disable frequent checks
cco config set updates.check_interval weekly
```

**Using authentication** (optional, increases limit to 5000/hour):

```bash
# Set GitHub token (requires public repo access)
export GITHUB_TOKEN="ghp_..."

# Then retry
cco update --check
```

### Network Requirements

CCO needs outbound HTTPS access to:

- `api.github.com` (version checking)
- `github.com` (downloading releases)

**Firewall rules needed:**

```
Allow outbound HTTPS (443) to:
- api.github.com
- github.com
- *.githubusercontent.com (release assets)
```

### Proxy Support

If behind a corporate proxy:

```bash
# Set proxy for HTTP requests
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"

# Then run update
cco update --yes
```

## Configuration Management

### Organizational Policies

**Standard configuration for all servers:**

```bash
#!/bin/bash
# setup-cco-updates.sh

# Disable manual updates in production
cco config set updates.enabled false
cco config set updates.check_interval never

# Only allow scheduled updates via automation
cco config set updates.auto_install false
cco config set updates.channel stable

# Verify
echo "Final configuration:"
cco config show
```

**Run on all servers during deployment:**

```bash
for server in $(cat servers.txt); do
    ssh $server bash < setup-cco-updates.sh
done
```

### Backup Configuration

**Backup all configurations before bulk changes:**

```bash
#!/bin/bash
# backup-cco-config.sh

BACKUP_DIR="backups/cco-config-$(date +%Y%m%d-%H%M%S)"
mkdir -p $BACKUP_DIR

for server in prod-1 prod-2 prod-3; do
    ssh $server cat ~/.config/cco/config.toml > $BACKUP_DIR/$server-config.toml
done

echo "Backed up to: $BACKUP_DIR"
```

**Restore if needed:**

```bash
scp $BACKUP_DIR/$server-config.toml $server:~/.config/cco/config.toml
ssh $server cco config show
```

## Audit and Compliance

### Update Log Tracking

Enable detailed logging:

```bash
# Enable debug logging
cco run --debug

# View logs
cco daemon logs --follow

# Filter for update events
cco daemon logs | grep -i update
```

**Example log output:**
```
[INFO] Starting update check
[INFO] Current version: 2025.11.1
[INFO] Latest version: 2025.11.2
[INFO] Downloading release...
[INFO] Verifying checksum...
[INFO] Installing update...
[INFO] Successfully updated to 2025.11.2
```

### Audit Trail

**Track updates across your fleet:**

```bash
#!/bin/bash
# audit-updates.sh

# Log all updates
for server in prod-*; do
    LAST_UPDATE=$(ssh $server cco config get updates.last_update 2>/dev/null | awk '{print $3, $4}')
    VERSION=$(ssh $server cco version 2>/dev/null | head -1 | awk '{print $3}')
    echo "$(date): $server updated to $VERSION on $LAST_UPDATE" >> update-audit.log
done
```

### Compliance Requirements

**Document your update policy:**

1. **Frequency**: How often updates are checked
2. **Approval**: Who approves production updates
3. **Testing**: What testing is performed before rollout
4. **Rollback**: How to handle failed updates
5. **Monitoring**: How updates are verified

**Example policy template:**

```
CCO Update Policy
=================

Frequency: Weekly manual updates on Tuesday 2 AM UTC
Channels: Stable only (no beta in production)
Testing: 7-day soak in staging before production
Approval: SRE team reviews release notes
Rollback: Automatic rollback on health check failure
Monitoring: Health checks every 30 seconds post-update
Notification: Slack alert on update completion
```

## Performance Considerations

### Update Timing

**Avoid peak hours:**

```bash
# Schedule updates during low-traffic windows
# Example: Tuesday 2-3 AM UTC
0 2 * * 2 /usr/local/bin/cco update --yes
```

**Stagger updates across instances:**

```bash
# prod-1: Tuesday 2 AM
# prod-2: Tuesday 2:10 AM
# prod-3: Tuesday 2:20 AM
# Prevents fleet-wide disruption if update fails
```

### Storage Requirements

**Disk space for updates:**

- Binary size: ~30-50 MB
- Temporary files during update: ~100 MB
- Backup of previous version: ~50 MB

**Recommended free space:** 500 MB minimum

```bash
# Check available space
df -h ~/.local/bin

# Check CCO binary size
ls -lh ~/.local/bin/cco
```

## Automation Examples

### Cron-Based Updates

**Update daily at 3 AM:**

```bash
0 3 * * * /usr/local/bin/cco update --yes 2>&1 | logger -t cco-update
```

**Update weekly on Saturday:**

```bash
0 3 * * 6 /usr/local/bin/cco update --yes 2>&1 | logger -t cco-update
```

### Systemd Timer (Alternative to Cron)

**Create timer file** (`/etc/systemd/system/cco-update.timer`):

```ini
[Unit]
Description=CCO Auto-Update Timer
After=network.target

[Timer]
OnCalendar=Tue *-*-* 02:00:00 UTC
Persistent=true

[Install]
WantedBy=timers.target
```

**Create service file** (`/etc/systemd/system/cco-update.service`):

```ini
[Unit]
Description=CCO Auto-Update Service

[Service]
Type=oneshot
User=root
ExecStart=/usr/local/bin/cco update --yes
StandardOutput=journal
StandardError=journal
```

**Enable and start:**

```bash
sudo systemctl daemon-reload
sudo systemctl enable cco-update.timer
sudo systemctl start cco-update.timer

# Check status
sudo systemctl status cco-update.timer
sudo systemctl list-timers cco-update.timer
```

### Kubernetes Deployment

**ConfigMap for update configuration:**

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cco-config
  namespace: default
data:
  config.toml: |
    [updates]
    enabled = false          # Updates handled by deployment
    auto_install = false
    check_interval = "never"
    channel = "stable"
```

**Pod image update strategy:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cco-daemon
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  template:
    spec:
      containers:
      - name: cco
        image: cco:2025.11.2  # Update version here
        imagePullPolicy: Always
        volumeMounts:
        - name: config
          mountPath: /root/.config/cco
      volumes:
      - name: config
        configMap:
          name: cco-config
```

## Troubleshooting for Administrators

### Fleet-Wide Update Failure

**If all servers fail to update:**

1. Check GitHub status: https://www.githubstatus.com/
2. Check network connectivity: `ping github.com`
3. Check firewall rules allow access to api.github.com
4. Retry after GitHub issue resolves

### Inconsistent Versions

**If different servers have different versions:**

```bash
# List all versions
for server in prod-*; do
    VERSION=$(ssh $server cco version | head -1)
    echo "$server: $VERSION"
done

# Update all to latest
./rolling-update.sh
```

### Stuck Update Check

**If update checks hang on all servers:**

```bash
# Temporarily disable updates
for server in prod-*; do
    ssh $server cco config set updates.enabled false
done

# Investigate GitHub connectivity
ssh prod-1 curl -v https://api.github.com/repos/brentley/cco-releases/releases/latest

# Re-enable after resolving
for server in prod-*; do
    ssh $server cco config set updates.enabled true
done
```

## See Also

- [User Guide](./AUTO_UPDATE_USER_GUIDE.md) - For end users
- [Architecture Overview](./AUTO_UPDATE_ARCHITECTURE.md) - How updates work internally
- [Production Readiness](./PRODUCTION_CHECKLIST.md) - Full deployment checklist

