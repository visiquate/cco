# CCO Troubleshooting Guide

Solutions for common problems and how to debug them.

## Quick Diagnosis

**CCO won't start?** → Jump to [Startup Issues](#startup-issues)

**Daemon won't start?** → Jump to [Daemon Issues](#daemon-issues)

**Temp files not found?** → Jump to [Temp File Issues](#temp-file-issues)

**Claude Code not launching?** → Jump to [Claude Code Integration](#claude-code-integration)

**Requests not being cached?** → Jump to [Cache Issues](#cache-issues)

**API key errors?** → Jump to [Authentication Issues](#authentication-issues)

**Performance problems?** → Jump to [Performance Issues](#performance-issues)

**Dashboard not working?** → Jump to [Dashboard Issues](#dashboard-issues)

---

## Daemon Issues

### Error: "Daemon failed to start"

**Problem:** Daemon won't start or crashes immediately

**Solution 1: Check port conflicts**
```bash
# Check if port 3000 is in use
lsof -i :3000

# If in use, kill the process
kill <PID>

# Or use a different port
cco daemon start --port 3001
```

**Solution 2: Check daemon logs**
```bash
# View daemon logs for errors
cco daemon logs

# Look for specific error messages
# Common issues:
# - "Address already in use" → Port conflict
# - "Permission denied" → Fix temp directory permissions
# - "Failed to write temp file" → Temp directory not writable
```

**Solution 3: Check permissions**
```bash
# Temp directory may need permissions
mkdir -p $TMPDIR
chmod 755 $TMPDIR

# Restart daemon
cco daemon restart
```

### Error: "Daemon is not running"

**Problem:** Daemon status shows not running

**Solution 1: Start daemon**
```bash
# Start daemon explicitly
cco daemon start

# Expected output:
# ✅ Daemon started successfully
#    PID: 12345
#    VFS: /var/run/cco
#    Dashboard: http://localhost:3000
```

**Solution 2: Check if daemon process exists**
```bash
# Find daemon process
ps aux | grep cco

# If process exists but status says "not running":
# Kill zombie process
kill <PID>

# Start fresh
cco daemon start
```

**Solution 3: Remove lock file**
```bash
# Stale lock file may prevent start
rm -f /var/run/cco.lock

# Try starting again
cco daemon start
```

---

## Temp File Issues

### Error: "Temp files not found"

**Problem:** Settings files missing from temp directory

**Solution 1: Restart daemon**
```bash
# Restart to recreate temp files
cco daemon restart

# Verify files created
ls -la $TMPDIR/.cco-*
# Should show encrypted files
```

**Solution 2: Check temp directory**
```bash
# Verify temp directory exists and is writable
echo $TMPDIR
# Should show path like /var/folders/xx/xxx/T/

# Test write permissions
touch $TMPDIR/test && rm $TMPDIR/test
# Should succeed without errors

# If fails, check permissions
ls -la $TMPDIR
```

**Solution 3: Check permissions**
```bash
# Verify directory permissions
ls -la $TMPDIR

# If permission denied:
chmod 755 $TMPDIR

# Restart daemon
cco daemon restart
```

**Solution 4: Manual verification**
```bash
# List temp files
ls -la $TMPDIR/.cco-*
# Should show:
# .cco-orchestrator-settings
# .cco-agents-sealed
# .cco-rules-sealed
# .cco-hooks-sealed

# All files should be encrypted (binary data)
# File permissions should be -rw------- (600)
```

### Error: "Settings file verification failed"

**Problem:** Temp files corrupted or encryption key mismatch

**Solution:**
```bash
# Remove corrupted files
rm -f $TMPDIR/.cco-*

# Restart daemon to recreate
cco daemon restart

# Verify files recreated
ls -la $TMPDIR/.cco-*
# Should show fresh encrypted files
```

### Error: "Temp file missing: .cco-agents-sealed"

**Problem:** Expected file not in temp directory

**Solution:**
```bash
# Restart daemon to regenerate files
cco daemon restart

# Verify all files present
ls -la $TMPDIR/.cco-*
# Should show all files:
# .cco-orchestrator-settings
# .cco-agents-sealed
# .cco-rules-sealed
# .cco-hooks-sealed

# If still missing, check daemon logs
cco daemon logs | grep -i "temp\|file\|sealed"
```

---

## Claude Code Integration

### Error: "Claude Code executable not found"

**Problem:** CCO can't find Claude Code in PATH

**Solution 1: Install Claude Code**
```bash
# Install from official site
# https://claude.ai/code

# Verify installation
which claude-code
# or
which claude

# Should show path like:
# /usr/local/bin/claude-code
```

**Solution 2: Add to PATH**
```bash
# Find where Claude Code is installed
find /usr /opt ~/Applications -name "claude-code" 2>/dev/null

# Add to PATH in shell profile
echo 'export PATH="$PATH:/path/to/claude-code"' >> ~/.zshrc  # or ~/.bashrc

# Reload shell
source ~/.zshrc  # or ~/.bashrc

# Verify
which claude-code
```

**Solution 3: Create symlink**
```bash
# If Claude Code installed but not in PATH
sudo ln -s /path/to/claude-code /usr/local/bin/claude-code

# Verify
which claude-code
# Output: /usr/local/bin/claude-code
```

### Error: "Launch takes 3+ seconds"

**Problem:** First `cco` invocation is slow

**Cause:** This is **normal behavior** - daemon auto-starts on first run

**Solution:**
```bash
# First run (3-4 seconds - daemon starting)
time cco
# real    0m3.542s

# Subsequent runs (<1 second - daemon already running)
time cco
# real    0m0.234s

# To avoid delay, pre-start daemon:
cco daemon start
# Then `cco` will be instant
```

### Error: "Environment variables not set"

**Problem:** ORCHESTRATOR_* variables not in Claude Code environment

**Cause:** Running Claude Code directly instead of via `cco`

**Solution:**
```bash
# Don't do this:
claude-code  # Variables NOT set

# Do this instead:
cco  # Variables automatically set

# Verify variables are set (from within Claude Code):
env | grep ORCHESTRATOR
# Should show:
# ORCHESTRATOR_ENABLED=true
# ORCHESTRATOR_VFS_MOUNT=/var/run/cco
# ORCHESTRATOR_API_URL=http://localhost:3000
# ... (and more)
```

### Error: "Multiple cco processes running"

**Problem:** `cco` and `cco tui` running simultaneously

**This is normal and expected behavior!**

```bash
# Terminal 1: Development
cco  # Claude Code process

# Terminal 2: Monitoring
cco tui  # TUI dashboard process

# Both processes:
# - Share the same daemon
# - Have separate UIs
# - Don't conflict

# Verify both are running
ps aux | grep cco
# Should show multiple cco processes - this is correct
```

---

## Startup Issues

### Error: "Port already in use"

**Problem:** CCO can't bind to the port (another service is using it)

**Solution 1: Use a different port**
```bash
./cco-proxy --port 9000
```

**Solution 2: Find and stop the conflicting process**
```bash
# Find what's using port 8000
lsof -i :8000

# Output:
# COMMAND   PID    USER  FD  TYPE  DEVICE SIZE/OFF NODE NAME
# cco-proxy 1234   user  3u  IPv4  12345      0t0  TCP *:8000

# Kill it
kill 1234

# Now start CCO
./cco-proxy
```

**Solution 3: Use port 0 (let OS choose)**
```bash
./cco-proxy --port 0
# Will show: "Listening on http://127.0.0.1:54321"
```

### Error: "Failed to open database"

**Problem:** Can't create or write to the SQLite database

**Solution 1: Check permissions**
```bash
# Check if you can write to current directory
touch test.db && rm test.db

# If fails, use a different directory
mkdir -p /tmp/cco
./cco-proxy --db-path /tmp/cco/analytics.db
```

**Solution 2: Specify absolute path**
```bash
# Always use full paths, not relative
./cco-proxy --db-path /var/lib/cco/analytics.db

# Create directory if needed
sudo mkdir -p /var/lib/cco
sudo chown $USER /var/lib/cco
```

**Solution 3: Check disk space**
```bash
# Verify you have space
df -h

# If low on space, archive old data
curl -X POST http://localhost:8000/api/maintenance/archive-old-analytics
```

### Error: "Unable to parse configuration file"

**Problem:** Config JSON is malformed

**Solution: Validate JSON syntax**
```bash
# Check for syntax errors
python -m json.tool config/model-routing.json

# Or use jq
jq . config/model-routing.json

# Common issues:
# 1. Trailing commas (not allowed in JSON)
# 2. Single quotes (must be double quotes)
# 3. Missing colons in key-value pairs
```

**Example error:**
```json
// Wrong: trailing comma
{
  "routes": [
    { "pattern": "claude-*", "provider": "anthropic" },  // ← Remove comma
  ]
}

// Wrong: single quotes
{ 'pattern': 'claude-*' }  // ← Use double quotes

// Correct:
{ "pattern": "claude-*", "provider": "anthropic" }
```

### Error: "Workers configuration invalid"

**Problem:** Number of workers is too high or invalid

**Solution:**
```bash
# Use CPU count
./cco-proxy --workers $(nproc)

# Or specify reasonable number
./cco-proxy --workers 4

# Too high causes memory issues:
# Max recommended: 2x CPU cores
nproc  # Shows CPU count
```

---

## Authentication Issues

### Error: "Invalid API key" or 401 Unauthorized

**Problem:** API key not set or invalid

**Solution 1: Set API key**
```bash
# Check if key is set
echo $ANTHROPIC_API_KEY

# If empty, set it
export ANTHROPIC_API_KEY="sk-ant-..."

# Verify it's set
echo $ANTHROPIC_API_KEY  # Should show key, not blank
```

**Solution 2: Verify key is valid**
```bash
# Test the key directly
curl -H "x-api-key: $ANTHROPIC_API_KEY" \
  https://api.anthropic.com/v1/models

# Should return 200 OK with list of models
# If fails, key is invalid or revoked
```

**Solution 3: Use environment file**
```bash
# Create .env file
cat > .env << EOF
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
EOF

# Load it before starting
source .env
./cco-proxy
```

**Solution 4: Docker with secrets**
```bash
# Dockerfile
FROM cco-proxy:latest
RUN --mount=type=secret,id=anthropic_key \
    export ANTHROPIC_API_KEY=$(cat /run/secrets/anthropic_key)

# Run with secret
docker run --secret anthropic_key=sk-ant-... cco-proxy
```

### Error: "Multiple API keys not working"

**Problem:** OpenAI key works but Anthropic doesn't, or vice versa

**Solution: Set both keys**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."

# Verify both are set
env | grep _API_KEY
```

**Solution: Use per-provider environment variables**
```bash
# In config/model-routing.json
{
  "routes": [
    {
      "pattern": "^claude-",
      "api_key_env": "ANTHROPIC_API_KEY"
    },
    {
      "pattern": "^gpt-",
      "api_key_env": "OPENAI_API_KEY"
    }
  ]
}
```

---

## Cache Issues

### Cache not working (all requests show misses)

**Problem:** Requests aren't being cached even with identical prompts

**Solution 1: Verify cache is enabled**
```bash
# Check cache status
curl http://localhost:8000/api/cache/stats

# Response should show stats, not error
{
  "hits": 0,
  "misses": 10,
  "hitRate": 0,
  "size": 0
}
```

**Solution 2: Ensure identical requests**
```bash
# Cache key includes model, prompt, and parameters
# Any difference = cache miss

# Same (will cache):
request1 = {"model": "claude-opus", "prompt": "Hello", "temp": 1.0}
request2 = {"model": "claude-opus", "prompt": "Hello", "temp": 1.0}

# Different (won't cache):
request1 = {"model": "claude-opus", "prompt": "Hello", "temp": 1.0}
request2 = {"model": "claude-opus", "prompt": "Hello", "temp": 0.5}  # Different temp

# Check what's different
curl -X POST http://localhost:8000/api/cache/debug \
  -d '{"request": {...}}'
```

**Solution 3: Check cache size**
```bash
# See if cache is full
curl http://localhost:8000/api/cache/stats | jq '.size, .maxSize'

# If full, increase size
./cco-proxy --cache-size 2000  # 2GB instead of 500MB
```

**Solution 4: Verify TTL not too short**
```bash
# Check TTL
./cco-proxy --cache-ttl 3600

# If set to 0, cache is disabled
# Default 3600 (1 hour) is usually good
```

### Cache growing too large

**Problem:** Cache size keeps increasing

**Solution 1: Reduce cache size**
```bash
# Set smaller limit
./cco-proxy --cache-size 200  # 200MB instead of 500MB

# LRU eviction will clean up automatically
```

**Solution 2: Reduce TTL**
```bash
# Cache entries expire faster
./cco-proxy --cache-ttl 1800  # 30 min instead of 1 hour
```

**Solution 3: Manually clear cache**
```bash
# Emergency clear
curl -X POST http://localhost:8000/api/cache/clear

# Response:
{
  "status": "ok",
  "cleared": 1234,
  "freedMemory": "256MB"
}
```

**Solution 4: Check what's using space**
```bash
# Find top cached items
curl http://localhost:8000/api/cache/top-items?limit=20 | jq '.[] | {size, hits}'

# Look for items with few hits - they're wasting space
```

### Cache hit rate too low

**Problem:** Cache hits only 20% even with repeated requests

**Solution 1: Check for parameter variations**
```bash
# Log actual requests to find variations
curl -H "Debug: true" http://localhost:8000/api/debug/requests

# Look for:
# - Different temperatures
# - Different max_tokens
# - Different system prompts
# - Different capitalization

# All cause cache misses
```

**Solution 2: Standardize prompts**
```python
# Bad: Different prompts each time
for i in range(10):
    response = client.messages.create(
        model="claude-opus",
        messages=[{"role": "user", "content": f"Task #{i}: {task}"}]
    )

# Good: Use template (high cache hit on template)
template = "Task: {task}"
for i in range(10):
    prompt = template.format(task=task)
    response = client.messages.create(
        model="claude-opus",
        messages=[{"role": "user", "content": prompt}]
    )
```

**Solution 3: Batch similar requests**
```python
# Requests are more likely to repeat if batched
all_tasks = [...]  # Load all tasks first

# Process them (lots of repetition)
for task in all_tasks:
    response = client.messages.create(...)

# vs processing them randomly:
import random
random.shuffle(all_tasks)  # Randomize order (less cache hits)
```

---

## API Issues

### Error: "Connection refused" or "Can't reach Claude API"

**Problem:** CCO can't connect to upstream API (Claude, OpenAI, etc.)

**Solution 1: Check network**
```bash
# Test connection to Claude API
curl -I https://api.anthropic.com

# Should return HTTP 200 or 401 (not connection error)
```

**Solution 2: Check firewall**
```bash
# If behind corporate firewall, may need proxy
./cco-proxy --http-proxy http://proxy.company.com:8080

# Or use environment variable
export HTTP_PROXY=http://proxy:8080
export HTTPS_PROXY=http://proxy:8080
./cco-proxy
```

**Solution 3: Check DNS**
```bash
# Verify DNS resolution
nslookup api.anthropic.com

# Should return IP address
# If fails, DNS is broken
```

**Solution 4: Test with verbose logging**
```bash
# See detailed connection logs
./cco-proxy --log-level debug 2>&1 | grep -i "connection\|api"
```

### Error: "Request timeout"

**Problem:** Requests to Claude API are taking too long

**Solution 1: Increase timeout**
```bash
# In model-routing.json, increase timeout_ms
{
  "pattern": "^claude-",
  "timeout_ms": 120000  # 2 minutes instead of 60 seconds
}
```

**Solution 2: Check network latency**
```bash
# Measure round-trip time
ping api.anthropic.com

# If >200ms, you have high latency
# Try different CDN region
```

**Solution 3: Check CCO load**
```bash
# See if CCO is overloaded
curl http://localhost:8000/metrics | grep "queue_depth\|processing_time"

# High values mean slow processing
# Increase --workers
```

### Error: "Rate limit exceeded" (429)

**Problem:** Too many requests to upstream API

**Solution 1: Add backoff/retry**
```python
# CCO will retry automatically, but check:
# - Is retries set in config?
# - Are retries actually happening?

curl http://localhost:8000/api/logs | grep "retry"
```

**Solution 2: Implement client-side throttling**
```python
import time
import anthropic

client = anthropic.Anthropic(base_url="http://localhost:8000")

def throttled_request(prompt, max_concurrent=5):
    # Limit concurrent requests
    time.sleep(1.0 / max_concurrent)

    return client.messages.create(
        model="claude-opus",
        messages=[{"role": "user", "content": prompt}]
    )
```

**Solution 3: Use request queuing**
```bash
# CCO has built-in queue, check status
curl http://localhost:8000/metrics | grep queue

# If queue is growing, reduce request rate
```

---

## Performance Issues

### Slow response times

**Problem:** Requests taking >2 seconds

**Solution 1: Check if cache is working**
```bash
# Compare cached vs uncached
curl http://localhost:8000/api/cache/stats

# Hits should be 0ms, misses should be 100-500ms
```

**Solution 2: Check network**
```bash
# Measure latency to Claude API
time curl -w "@curl-format.txt" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  https://api.anthropic.com/v1/models

# If >500ms, it's network issue not CCO
```

**Solution 3: Increase workers**
```bash
# More workers = more parallel processing
./cco-proxy --workers 16

# Check optimal setting
nproc  # Shows CPU count, use 2x that
```

**Solution 4: Check database performance**
```bash
# Run database analysis
curl -X POST http://localhost:8000/api/database/analyze

# Output shows slow queries
```

### High CPU usage

**Problem:** CCO using 100% CPU

**Solution 1: Reduce workers**
```bash
# Too many workers can cause context switching
./cco-proxy --workers 4  # Start lower

# Monitor CPU and find sweet spot
watch -n 1 "top -bn1 | grep cco-proxy"
```

**Solution 2: Check for runaway requests**
```bash
# See what's being processed
curl http://localhost:8000/api/debug/active-requests

# Kill slow requests if needed
curl -X POST http://localhost:8000/api/requests/kill?older_than=30000
```

**Solution 3: Reduce cache size**
```bash
# Large cache can use CPU for cleanup
./cco-proxy --cache-size 200  # Smaller cache
```

### High memory usage

**Problem:** CCO using >1GB RAM

**Solution 1: Reduce cache size**
```bash
# Cache is the main memory consumer
./cco-proxy --cache-size 100  # 100MB instead of 500MB
```

**Solution 2: Reduce database cache**
```bash
# SQLite can be configured for lower memory
export CCO_DB_CACHE_SIZE=5000  # pages, not bytes
./cco-proxy
```

**Solution 3: Monitor memory over time**
```bash
# See if memory grows unbounded (memory leak)
./cco-proxy &
PID=$!

for i in {1..60}; do
    ps aux | grep $PID | grep -v grep
    sleep 10
done

# Look for VSZ (virtual size) growing
```

---

## Dashboard Issues

### Dashboard shows "Disconnected"

**Problem:** Dashboard can't connect to CCO backend

**Solution 1: Verify CCO is running**
```bash
# Check if service is up
curl http://localhost:8000/health

# Should return 200 OK
```

**Solution 2: Check connection**
```bash
# From browser console (F12 > Console):
fetch('http://localhost:8000/api/project/stats')
    .then(r => r.json())
    .then(d => console.log(d))

# If fails, see error message
```

**Solution 3: Check CORS**
```bash
# If dashboard is on different domain/port, CORS may block it
# Check response headers:
curl -I http://localhost:8000/api/project/stats

# Should include:
# Access-Control-Allow-Origin: *
```

**Solution 4: Try direct URL**
```bash
# Open in browser:
http://localhost:8000

# If this fails:
# 1. Check port is correct
# 2. Check firewall isn't blocking
# 3. Check binding address (127.0.0.1 vs 0.0.0.0)
```

### Charts not rendering

**Problem:** Dashboard shows empty charts

**Solution 1: Check data is available**
```bash
# Query the data endpoint
curl http://localhost:8000/api/machine/stats | jq '.chartData'

# Should have data, not empty arrays
```

**Solution 2: Check browser console for errors**
```bash
# Open DevTools (F12)
# Console tab shows JavaScript errors
# Look for D3.js or xterm.js loading issues
```

**Solution 3: Verify D3 CDN is accessible**
```bash
# Check if external libraries load
curl -I https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js

# Should return 200 OK
```

### Terminal not working

**Problem:** Terminal tab shows "Connecting..." forever

**Solution 1: Check WebSocket support**
```bash
# From browser console:
console.log(WebSocket ? "WebSocket available" : "WebSocket unavailable")

# Must be available (modern browsers have it)
```

**Solution 2: Verify WebSocket endpoint**
```bash
# Test connection
wscat -c ws://localhost:8000/terminal

# Should connect and show prompt
# Type commands and see output
```

**Solution 3: Check firewall**
```bash
# WebSocket may be blocked
# Try different port
./cco-proxy --port 9000

# Then connect to ws://localhost:9000/terminal
```

---

## Database Issues

### Error: "Database locked" or "disk I/O error"

**Problem:** SQLite database is locked or corrupted

**Solution 1: Check file permissions**
```bash
# Make sure you can read/write
ls -la cco.db

# Should be readable/writable by CCO user
chmod 600 cco.db
```

**Solution 2: Check disk space**
```bash
# Database needs space to write
df -h

# If <10% free, clean up files
# Or use different disk:
./cco-proxy --db-path /mnt/data/cco.db
```

**Solution 3: Backup and recover**
```bash
# Backup current database
cp cco.db cco.db.backup

# Try to recover
sqlite3 cco.db "PRAGMA integrity_check;"

# If corrupted, restore from backup
cp cco.db.backup cco.db
```

**Solution 4: Reset database**
```bash
# Delete database and start fresh
rm cco.db

# Next start will recreate it
./cco-proxy

# Note: This loses all historical analytics!
```

### Query is too slow

**Problem:** Analytics queries taking >5 seconds

**Solution 1: Analyze slow queries**
```bash
# Enable query profiling
curl -X POST http://localhost:8000/api/database/profile

# Run your queries, then check:
curl http://localhost:8000/api/database/profile-results
```

**Solution 2: Create indexes**
```bash
# CCO creates indexes automatically, but if slow:
curl -X POST http://localhost:8000/api/database/rebuild-indexes

# This may take a while on large databases
```

**Solution 3: Archive old data**
```bash
# Large databases are slow
curl -X POST http://localhost:8000/api/maintenance/archive-old-analytics \
  -d '{"older_than_days": 90}'

# Keep only recent data in main database
```

---

## Getting Help

### Enable Debug Logging

```bash
# Verbose output to see what's happening
./cco-proxy --log-level debug > cco-debug.log 2>&1

# Then check log while making requests
tail -f cco-debug.log
```

### Collect Diagnostic Info

```bash
# Create diagnostics bundle
curl http://localhost:8000/api/diagnostics > cco-diag.json

# Share this when asking for help (remove sensitive data first)
cat cco-diag.json | jq 'del(.. | .api_key?, .token?)'
```

### Common Patterns to Check

```bash
# These cause most issues:

# 1. API key not set
echo "API Key: $ANTHROPIC_API_KEY" | grep -c "sk-ant"

# 2. Port in use
lsof -i :8000

# 3. Database file not writable
touch cco.db && rm cco.db

# 4. Network can't reach API
curl -I https://api.anthropic.com

# 5. Config has syntax errors
jq . config/*.json
```

---

## FAQ

**Q: Is CCO safe to use in production?**
A: Yes, but start with non-critical workloads first.

**Q: Will CCO cache my sensitive data?**
A: Yes, locally. Use HTTPS and network security.

**Q: Can I run multiple CCO instances?**
A: Yes, each gets their own cache and database.

**Q: What happens if CCO crashes?**
A: Restart it. Cache is in-memory and will be lost, but database persists.

**Q: Can I migrate cache to another CCO instance?**
A: Cache can't migrate (in-memory), but database can be copied.

**Q: How do I update CCO?**
A: Download new binary and restart (or use rolling deployment if multiple instances).

## Still Having Issues?

1. Check above solutions for your exact error
2. Enable debug logging: `./cco-proxy --log-level debug`
3. Collect diagnostics: `curl http://localhost:8000/api/diagnostics`
4. Open a GitHub issue with logs and config (remove API keys!)
