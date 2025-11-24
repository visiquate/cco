# Auto-Update Troubleshooting Guide (Advanced)

## For Technical Users and Administrators

This guide provides advanced debugging techniques and solutions for complex auto-update issues.

---

## Diagnostic Tools and Techniques

### 1. Comprehensive System Information

Before reporting issues, gather complete system information:

```bash
#!/bin/bash
# gather-cco-diagnostics.sh

echo "=== CCO Diagnostics ==="
echo "Timestamp: $(date -u)"
echo ""

echo "=== Version Information ==="
cco --version 2>&1
echo ""

echo "=== System Information ==="
uname -a
echo "Kernel: $(uname -r)"
echo "CPU: $(uname -m)"
echo ""

echo "=== File Permissions ==="
ls -la ~/.local/bin/cco* 2>&1
ls -la ~/.config/cco/ 2>&1
ls -la ~/.cco/ 2>&1
echo ""

echo "=== Configuration ==="
cco config show 2>&1
echo ""

echo "=== Network Connectivity ==="
ping -c 1 github.com 2>&1
curl -I https://api.github.com 2>&1
echo ""

echo "=== Update Logs (Last 100 lines) ==="
tail -100 ~/.cco/logs/updates.log 2>&1
echo ""

echo "=== Disk Space ==="
df -h ~/.local/bin ~/
du -sh ~/.cco/ ~/.config/cco/ ~/.local/bin/ 2>&1
echo ""

echo "=== Process Information ==="
ps aux | grep cco
echo ""

echo "=== Environment Variables ==="
env | grep CCO
echo ""

echo "=== End of Diagnostics ==="
```

Run this:

```bash
bash gather-cco-diagnostics.sh > cco-diagnostics.txt
# Share cco-diagnostics.txt when reporting issues
```

### 2. Real-Time Update Monitoring

Monitor an update as it happens:

```bash
#!/bin/bash
# monitor-update.sh

echo "Starting update monitoring..."
echo "Log file: ~/.cco/logs/updates.log"
echo ""

# Terminal 1: Watch logs
tail -f ~/.cco/logs/updates.log &
TAIL_PID=$!

# Terminal 2: Start update
sleep 2
cco update --yes

# Stop watching
kill $TAIL_PID

echo "Update complete. Review logs above."
```

### 3. Detailed Logging

Enable verbose logging for updates:

```bash
# Run update with verbose output
cco update --verbose --yes

# Or with environment variables
RUST_LOG=debug cco update --yes

# This shows detailed internal operations
```

### 4. Network Debugging

Diagnose network issues:

```bash
#!/bin/bash
# test-network-connectivity.sh

echo "=== Testing GitHub Connectivity ==="

# Test 1: DNS Resolution
echo "1. DNS Resolution:"
nslookup github.com
nslookup api.github.com

# Test 2: Network Connectivity
echo ""
echo "2. Ping Test:"
ping -c 4 github.com

# Test 3: HTTPS Connection
echo ""
echo "3. HTTPS Connection:"
curl -v https://api.github.com

# Test 4: GitHub API
echo ""
echo "4. GitHub API Connectivity:"
curl -I https://api.github.com/repos/yourusername/cco/releases

# Test 5: Release Asset Download
echo ""
echo "5. Release Asset Test:"
curl -I https://github.com/yourusername/cco/releases/download/v2025.11.3/cco-v2025.11.3-linux-x86_64.tar.gz

# Test 6: Network Performance
echo ""
echo "6. Network Speed Test:"
time curl -o /dev/null https://github.com/yourusername/cco/releases/download/v2025.11.3/cco-v2025.11.3-linux-x86_64.tar.gz
```

---

## Common Issues and Solutions

### Issue 1: Checksum Verification Constantly Failing

**Symptoms:**
```
[ERROR] Checksum verification failed! Update aborted for security.
```

**Diagnosis:**

```bash
# Check network quality
for i in {1..10}; do
    echo "Attempt $i:"
    curl -I https://api.github.com
    echo ""
done

# Look for timeout patterns
ping -c 100 github.com | grep "time=*ms"

# Check for packet loss
mtr github.com -c 100
```

**Solutions:**

```bash
# 1. Check DNS resolution
nslookup github.com
# If fails, try different DNS:
# 8.8.8.8 (Google)
# 1.1.1.1 (Cloudflare)
# 208.67.222.222 (OpenDNS)

# 2. Check for proxy interference
echo $HTTP_PROXY $HTTPS_PROXY

# 3. Test without proxy
unset HTTP_PROXY HTTPS_PROXY
cco update --check

# 4. Test specific release
VERSION=2025.11.3
URL="https://github.com/yourusername/cco/releases/download/v${VERSION}"

# Download with retry
wget --tries=5 --timeout=10 "${URL}/cco-v${VERSION}-linux-x86_64.tar.gz"

# 5. Check firewall rules
iptables -L -n | grep 443
ufw status | grep 443  # If using UFW

# 6. Check for man-in-the-middle
openssl s_client -connect api.github.com:443
# Look for certificate details
```

### Issue 2: Permission Denied During Update

**Symptoms:**
```
[ERROR] Permission denied during installation
```

**Diagnosis:**

```bash
# Check binary permissions
ls -la ~/.local/bin/cco
ls -la ~/.local/bin/
ls -la ~/.local/

# Check owner
stat ~/.local/bin/cco

# Check for immutable flag (Linux)
lsattr ~/.local/bin/cco
```

**Solutions:**

```bash
# 1. Fix directory permissions
chmod 755 ~/.local/bin/
chmod 755 ~/.local/

# 2. Fix file permissions
chmod 755 ~/.local/bin/cco

# 3. Remove immutable flag (if set)
chattr -i ~/.local/bin/cco

# 4. Change ownership (if wrong user)
chown $USER:$GROUP ~/.local/bin/cco

# 5. Reinstall to different location
mkdir -p ~/bin
# Download and extract to ~/bin/cco
# Add ~/bin to PATH:
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# 6. Install system-wide (if allowed)
sudo mv ~/.local/bin/cco /usr/local/bin/cco
sudo chmod 755 /usr/local/bin/cco
```

### Issue 3: Binary Verification Fails After Installation

**Symptoms:**
```
[ERROR] Verification of new binary failed
[INFO] Rolling back to backup
```

**Diagnosis:**

```bash
# Check if binary is executable
file ~/.local/bin/cco
# Should show: ELF 64-bit LSB executable

# Test binary directly
~/.local/bin/cco --version

# Check platform compatibility
uname -m  # Your architecture
file ~/.local/bin/cco | grep "x86-64\|aarch64\|ARM"  # Binary architecture

# Check for missing dependencies
ldd ~/.local/bin/cco 2>&1

# Run under strace to see errors
strace -e trace=file ~/.local/bin/cco --version 2>&1 | head -50
```

**Solutions:**

```bash
# 1. Verify correct platform binary was downloaded
# Expected: matches output of: uname -m
# x86_64 → cco-*-linux-x86_64
# aarch64 → cco-*-linux-aarch64
# arm64 → cco-*-darwin-arm64

# 2. Check for corrupted download
wget -c <URL>  # Resume download if interrupted

# 3. Manually verify platform
echo "Your architecture:"
uname -m
echo "Your OS:"
uname -s

# 4. Download correct binary manually
VERSION=2025.11.3
PLATFORM="linux-x86_64"  # Adjust for your system
URL="https://github.com/yourusername/cco/releases/download/v${VERSION}/cco-v${VERSION}-${PLATFORM}.tar.gz"

wget "$URL"
tar xzf "cco-v${VERSION}-${PLATFORM}.tar.gz"
chmod +x cco
mv cco ~/.local/bin/cco

# 5. Test after manual installation
cco --version
```

### Issue 4: Update Never Completes

**Symptoms:**
- Update process hangs indefinitely
- No progress for 10+ minutes
- Process doesn't respond to Ctrl+C

**Diagnosis:**

```bash
# Check if process is stuck
ps aux | grep cco

# Monitor system resources
top -p <PID>

# Check for I/O wait
iostat -x 1 10

# Check network activity
netstat -tnp | grep cco

# Check for zombie processes
ps aux | grep Z
```

**Solutions:**

```bash
# 1. Kill hung process
pkill -9 cco

# 2. Check disk space
df -h ~/.local/
du -sh /tmp/cco-*

# 3. Clear temporary files
rm -rf /tmp/cco-*

# 4. Check system load
uptime

# 5. Retry with timeout
timeout 300 cco update --yes  # 5 minute timeout

# 6. Disable and enable auto-update
cco config set updates.enabled false
cco config set updates.enabled true
cco update --check
```

### Issue 5: Logs Growing Too Large

**Symptoms:**
- `~/.cco/logs/updates.log` is very large (>100 MB)
- Disk space issues

**Solution:**

```bash
# Check current size
du -h ~/.cco/logs/updates.log

# Rotate/archive old logs
# Option 1: Keep last 1000 lines
tail -1000 ~/.cco/logs/updates.log > ~/.cco/logs/updates.log.tmp
mv ~/.cco/logs/updates.log.tmp ~/.cco/logs/updates.log

# Option 2: Archive and compress
gzip -c ~/.cco/logs/updates.log > ~/.cco/logs/updates.log.gz
tail -1000 ~/.cco/logs/updates.log.gz > ~/.cco/logs/updates.log

# Option 3: Clear entirely (not recommended)
# rm ~/.cco/logs/updates.log
# New log will be created on next update
```

### Issue 6: Configuration File Corrupted

**Symptoms:**
```
[ERROR] Invalid configuration
```

**Diagnosis:**

```bash
# Check configuration format
cat ~/.config/cco/config.toml

# Validate TOML syntax
python3 -m pip install tomli
python3 << 'EOF'
import tomli
try:
    with open(os.path.expanduser('~/.config/cco/config.toml'), 'rb') as f:
        tomli.load(f)
    print("✓ Configuration is valid")
except Exception as e:
    print("✗ Configuration error:", e)
EOF
```

**Solutions:**

```bash
# 1. Reset to defaults
cco config reset

# 2. Manually fix if you know TOML
nano ~/.config/cco/config.toml

# 3. Restore from backup
cp ~/.config/cco/config.toml.bak ~/.config/cco/config.toml

# 4. Reconfigure from scratch
rm ~/.config/cco/config.toml
cco config show  # This will create new config
```

---

## Advanced Debugging Scenarios

### Scenario 1: Updates Fail Only on Weekends

**Possible causes:**
- ISP routing changes
- Network maintenance windows
- Firewall rule changes

**Debug:**

```bash
# Test connectivity at specific times
for hour in {0..23}; do
    at "02:00 tomorrow + $hour hours" << 'EOF'
        curl -I https://api.github.com >> /tmp/cco-network-test.log 2>&1
EOF
done

# Review results
cat /tmp/cco-network-test.log
```

### Scenario 2: Checksum Works Manually But Fails in Auto-Update

**Possible causes:**
- Timing-dependent issue
- Environment differences
- File descriptor limits

**Debug:**

```bash
# Manually verify
VERSION=2025.11.3
wget "https://github.com/yourusername/cco/releases/download/v${VERSION}/cco-v${VERSION}-linux-x86_64.tar.gz"
wget "https://github.com/yourusername/cco/releases/download/v${VERSION}/checksums.sha256"
sha256sum -c checksums.sha256

# Compare with auto-update process
cco update --verbose --yes

# Check for environment differences
env > /tmp/manual-env.txt
(cco update --yes >/dev/null 2>&1) & env > /tmp/auto-env.txt
diff /tmp/manual-env.txt /tmp/auto-env.txt
```

### Scenario 3: Different Behavior Across Machines

**Possible causes:**
- Different OS versions
- Different system configurations
- Network connectivity differences

**Debug on each machine:**

```bash
#!/bin/bash
# compare-systems.sh

for machine in $MACHINES; do
    echo "=== $machine ==="
    ssh user@$machine "
        echo 'Version:'
        uname -a
        echo 'Network:'
        ping -c 1 github.com
        echo 'Update logs:'
        tail -5 ~/.cco/logs/updates.log
        echo ''
    "
done > comparison.txt
```

---

## Log Analysis Techniques

### Parse Update Log for Metrics

```bash
#!/bin/bash
# analyze-update-logs.sh

echo "=== Update Log Analysis ==="

# Total updates
echo "Total updates: $(grep 'successfully installed' ~/.cco/logs/updates.log | wc -l)"

# Total failures
echo "Total failures: $(grep 'ERROR' ~/.cco/logs/updates.log | wc -l)"

# Failed checksums
echo "Checksum failures: $(grep 'Checksum verification failed' ~/.cco/logs/updates.log | wc -l)"

# Average update time
echo "Average update duration:"
grep -E '\[.*\] Check started' ~/.cco/logs/updates.log | while read line; do
    timestamp=$(echo "$line" | grep -oP '\[\K[^\]]+')
    # Calculate time from start to success
done

# Most common errors
echo ""
echo "Top errors:"
grep 'ERROR' ~/.cco/logs/updates.log | sed 's/.*ERROR: //' | sort | uniq -c | sort -rn | head -5

# Update history
echo ""
echo "Recent updates:"
grep 'successfully installed' ~/.cco/logs/updates.log | tail -5
```

### Extract Specific Information

```bash
# Find when last update happened
grep 'successfully installed' ~/.cco/logs/updates.log | tail -1

# Find when last check happened
grep 'Check started' ~/.cco/logs/updates.log | tail -1

# Find all network errors
grep 'Network\|Connection\|timeout' ~/.cco/logs/updates.log

# Find all permission errors
grep 'Permission denied\|permission' ~/.cco/logs/updates.log

# Find version history
grep 'successfully installed' ~/.cco/logs/updates.log | grep -oP 'CCO \K[0-9.]+' | sort -u
```

---

## Performance Analysis

### Identify Slow Updates

```bash
#!/bin/bash
# analyze-update-performance.sh

echo "=== Update Performance Analysis ==="

# Extract timestamps for each update
grep -E '\[.*\] (Check started|Successfully installed)' ~/.cco/logs/updates.log |
while read line; do
    timestamp=$(echo "$line" | grep -oP '\[\K[^\]]+')
    status=$(echo "$line" | grep -oP '\] \K[^[]+')
    echo "$timestamp $status"
done |
awk '
  /Check started/ { start=$1; next }
  /successfully installed/ {
    cmd = "date -d '"'"'"$1"'"'"' +%s"
    cmd | getline current_time
    cmd = "date -d '"'"'"start"'"'"' +%s"
    cmd | getline start_time
    duration = current_time - start_time
    print "Update duration: " duration " seconds"
    close(cmd)
  }
'
```

### Download Speed Analysis

```bash
# Extract download information
grep 'Downloaded' ~/.cco/logs/updates.log |
while read line; do
    # Parse size from log
    size=$(echo "$line" | grep -oP '[0-9.]+ MB' | head -1)
    echo "Downloaded: $size"
done

# Calculate average download size
grep 'Downloaded' ~/.cco/logs/updates.log |
grep -oP '[0-9.]+' |
awk '{sum+=$1; count++} END {print "Average:", sum/count, "MB"}'
```

---

## Recovery Procedures

### Complete Update Reset

```bash
#!/bin/bash
# complete-reset.sh

echo "Warning: This will reset all update configuration"
read -p "Continue? (y/n)" -n 1

# 1. Stop any running updates
pkill -f cco

# 2. Remove configuration
rm -f ~/.config/cco/config.toml

# 3. Clear logs
rm -f ~/.cco/logs/updates.log

# 4. Remove temp files
rm -rf /tmp/cco-*

# 5. Verify binary still works
cco --version

# 6. Reconfigure
cco config set updates.enabled true
cco config set updates.auto_install true

echo "Reset complete. CCO will reconfigure on next run."
```

### Backup and Restore Procedure

```bash
# Before making changes:
tar czf ~/cco-backup-$(date +%Y%m%d).tar.gz \
    ~/.config/cco \
    ~/.cco \
    ~/.local/bin/cco*

# If needed, restore:
tar xzf ~/cco-backup-20251117.tar.gz -C ~/
```

---

## When to Contact Support

**Contact support with:**

1. **Diagnostic output**: Run gather-cco-diagnostics.sh
2. **Specific error messages**: From logs
3. **Steps to reproduce**: What you did
4. **System information**: OS, architecture
5. **Network information**: ISP, firewall, proxy info
6. **Expected vs actual behavior**: What should happen vs what did

**Include but DO NOT share:**
- API keys or credentials
- Personal file paths (sanitize)
- Sensitive configuration details

## Related Documentation

- [Auto-Update User Guide](AUTO_UPDATE_USER_GUIDE.md)
- [Auto-Update Security](AUTO_UPDATE_SECURITY.md)
- [Auto-Update Administrator Guide](AUTO_UPDATE_ADMIN_GUIDE.md)
- [Auto-Update Command Reference](AUTO_UPDATE_COMMAND_REFERENCE.md)
