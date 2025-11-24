# Troubleshooting Guide

Solutions for common problems and how to debug them.

## Quick Diagnosis

**CCO won't start?** → Jump to [Startup Issues](#startup-issues)

**Daemon won't start?** → Jump to [Daemon Issues](#daemon-issues)

**Claude Code not launching?** → Jump to [Claude Code Integration](#claude-code-integration)

**Requests not being cached?** → Jump to [Cache Issues](#cache-issues)

**API key errors?** → Jump to [Authentication Issues](#authentication-issues)

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

**Solution:**
```bash
# Start daemon explicitly
cco daemon start

# Expected output:
# ✅ Daemon started successfully
#    PID: 12345
#    Dashboard: http://localhost:3000
```

---

## Startup Issues

### Error: "Port already in use"

**Problem:** CCO can't bind to the port

**Solution 1: Use a different port**
```bash
cco daemon start --port 9000
```

**Solution 2: Find and stop conflicting process**
```bash
# Find what's using the port
lsof -i :3000

# Kill it
kill <PID>

# Start CCO
cco daemon start
```

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
```

**Solution 2: Add to PATH**
```bash
# Find where Claude Code is installed
find /usr /opt ~/Applications -name "claude-code" 2>/dev/null

# Add to shell profile
echo 'export PATH="$PATH:/path/to/claude-code"' >> ~/.zshrc

# Reload shell
source ~/.zshrc

# Verify
which claude-code
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
echo $ANTHROPIC_API_KEY
```

**Solution 2: Verify key is valid**
```bash
# Test the key directly
curl -H "x-api-key: $ANTHROPIC_API_KEY" \
  https://api.anthropic.com/v1/models

# Should return 200 OK with list of models
```

**Solution 3: Use environment file**
```bash
# Create .env file
cat > .env << EOF
ANTHROPIC_API_KEY=sk-ant-...
EOF

# Load before starting
source .env
cco daemon start
```

---

## Cache Issues

### Cache not working (all requests show misses)

**Problem:** Requests aren't being cached despite identical prompts

**Solution 1: Verify cache is enabled**
```bash
# Check cache status
curl http://localhost:3000/api/cache/stats

# Response should show stats:
# {
#   "hits": 0,
#   "misses": 10,
#   "hitRate": 0,
#   "size": 0
# }
```

**Solution 2: Ensure identical requests**
```bash
# Cache key includes model, prompt, and parameters
# Any difference = cache miss

# These will cache:
request1 = {"model": "claude-opus", "prompt": "Hello", "temperature": 1.0}
request2 = {"model": "claude-opus", "prompt": "Hello", "temperature": 1.0}

# These won't cache (different temperature):
request1 = {"model": "claude-opus", "prompt": "Hello", "temperature": 1.0}
request2 = {"model": "claude-opus", "prompt": "Hello", "temperature": 0.5}
```

**Solution 3: Check cache size**
```bash
# See if cache is full
curl http://localhost:3000/api/cache/stats | jq '.size, .maxSize'

# If full, increase size
cco daemon restart --cache-size 2000
```

### Cache growing too large

**Problem:** Cache size keeps increasing

**Solution 1: Reduce cache size**
```bash
# Set smaller limit
cco daemon restart --cache-size 200

# LRU eviction will clean up automatically
```

**Solution 2: Reduce TTL**
```bash
# Cache entries expire faster
cco daemon restart --cache-ttl 1800  # 30 min
```

**Solution 3: Manually clear cache**
```bash
# Emergency clear
curl -X POST http://localhost:3000/api/cache/clear

# Response:
# {
#   "status": "ok",
#   "cleared": 1234,
#   "freedMemory": "256MB"
# }
```

---

## Claude Code Integration

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

---

## Dashboard Issues

### Dashboard shows "Disconnected"

**Problem:** Dashboard can't connect to CCO backend

**Solution 1: Verify CCO is running**
```bash
# Check if service is up
curl http://localhost:3000/health

# Should return 200 OK
```

**Solution 2: Check connection**
```bash
# From browser console (F12 > Console):
fetch('http://localhost:3000/api/project/stats')
    .then(r => r.json())
    .then(d => console.log(d))

# If fails, see error message
```

**Solution 3: Try direct URL**
```bash
# Open in browser:
http://localhost:3000

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
curl http://localhost:3000/api/machine/stats | jq '.chartData'

# Should have data, not empty arrays
```

**Solution 2: Check browser console for errors**
```bash
# Open DevTools (F12)
# Console tab shows JavaScript errors
# Look for D3.js or xterm.js loading issues
```

---

## Performance Issues

### Slow response times

**Problem:** Requests taking >2 seconds

**Solution 1: Check if cache is working**
```bash
# Compare cached vs uncached
curl http://localhost:3000/api/cache/stats

# Hits should be <100ms, misses 100-500ms
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
cco daemon restart --workers 16

# Check optimal setting
nproc  # Shows CPU count, use 2x that
```

### High memory usage

**Problem:** CCO using >1GB RAM

**Solution 1: Reduce cache size**
```bash
# Cache is the main memory consumer
cco daemon restart --cache-size 100
```

**Solution 2: Monitor memory over time**
```bash
# See if memory grows unbounded (memory leak)
cco daemon start &
PID=$!

for i in {1..60}; do
    ps aux | grep $PID | grep -v grep
    sleep 10
done

# Look for VSZ (virtual size) growing
```

---

## Getting Help

### Enable Debug Logging

```bash
# Verbose output to see what's happening
cco daemon restart --log-level debug

# Then check logs while making requests
cco daemon logs --follow
```

### Collect Diagnostic Info

```bash
# Check daemon is running
cco daemon status

# View recent logs
cco daemon logs | tail -50

# Check environment
env | grep -E "ORCHESTRATOR|CCO_|API_KEY" | head -20
```

### Common Patterns to Check

```bash
# 1. API key not set
echo "API Key: $ANTHROPIC_API_KEY" | grep -c "sk-ant"

# 2. Port in use
lsof -i :3000

# 3. Network can't reach API
curl -I https://api.anthropic.com

# 4. Temp directory writable
touch $TMPDIR/test && rm $TMPDIR/test

# 5. Claude Code in PATH
which claude-code
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

**Q: How do I update CCO?**
A: Download new binary and restart.

**Q: Can I use CCO with multiple API providers?**
A: Yes, configure routes in model-routing.json.

---

## Still Having Issues?

1. Check above solutions for your exact error
2. Enable debug logging: `cco daemon restart --log-level debug`
3. Check daemon status: `cco daemon status`
4. View logs: `cco daemon logs --follow`
5. Open a GitHub issue with logs and config (remove API keys!)
