# Orchestration Sidecar Troubleshooting Guide

**Version**: 1.0.0
**Date**: November 2025
**Audience**: Operators and developers troubleshooting issues

## Table of Contents

1. [Quick Diagnostics](#quick-diagnostics)
2. [Server Issues](#server-issues)
3. [Authentication Issues](#authentication-issues)
4. [Context Issues](#context-issues)
5. [Event Issues](#event-issues)
6. [Performance Issues](#performance-issues)
7. [Storage Issues](#storage-issues)
8. [Network Issues](#network-issues)
9. [Common Errors](#common-errors)
10. [Debug Mode](#debug-mode)

---

## Quick Diagnostics

### Health Check

```bash
# Check if sidecar is running
curl http://localhost:3001/health

# Expected: {"status": "healthy", ...}
# If fails: Sidecar is down or unreachable
```

### Status Check

```bash
# Get detailed status
curl -H "Authorization: Bearer $JWT_TOKEN" \
  http://localhost:3001/status

# Check:
# - agents.active > 0
# - storage.context_cache_entries > 0
# - events.queue_depth < 10000
```

### Log Check

```bash
# View sidecar logs
tail -f /tmp/cco-sidecar/logs/orchestration.log

# Or with journalctl (systemd)
journalctl -u cco-sidecar -f
```

### Process Check

```bash
# Check if process is running
ps aux | grep orchestration-server

# Check port binding
lsof -i :3001
netstat -tlnp | grep 3001
```

---

## Server Issues

### Issue: Sidecar won't start

**Symptoms:**
- `cco orchestration-server` exits immediately
- No response from port 3001
- Error: "Address already in use"

**Diagnosis:**

```bash
# Check if port is in use
lsof -i :3001

# Check for existing process
ps aux | grep orchestration-server

# Check logs
cat /tmp/cco-sidecar/logs/orchestration.log
```

**Solutions:**

```bash
# Kill existing process
pkill -f orchestration-server

# Or kill specific PID
kill <PID>

# Use different port
cco orchestration-server --port 4000

# Check file permissions
ls -la /tmp/cco-sidecar
chmod 755 /tmp/cco-sidecar

# Clear storage and restart
rm -rf /tmp/cco-sidecar/*
cco orchestration-server
```

### Issue: Sidecar crashes frequently

**Symptoms:**
- Process exits unexpectedly
- "Segmentation fault" errors
- Out of memory errors

**Diagnosis:**

```bash
# Check memory usage
curl http://localhost:3001/health | jq '.checks.memory_usage_mb'

# Check system resources
free -h
df -h

# Check for core dumps
ls -la /tmp/cco-sidecar/core.*
```

**Solutions:**

```bash
# Increase cache size limit
cco orchestration-server --cache-size-mb 512

# Clear cache frequently
curl -X DELETE http://localhost:3001/api/cache/all

# Monitor with restart on crash
while true; do
    cco orchestration-server
    echo "Crashed, restarting in 5s..."
    sleep 5
done

# Use systemd for auto-restart
# Create /etc/systemd/system/cco-sidecar.service
[Service]
Restart=always
RestartSec=5
```

### Issue: Graceful shutdown fails

**Symptoms:**
- `cco orchestration-server stop` hangs
- Process doesn't respond to SIGTERM
- Force kill required

**Diagnosis:**

```bash
# Check active agents
curl http://localhost:3001/status | jq '.agents.active'

# Check event queue
curl http://localhost:3001/status | jq '.events.queue_depth'
```

**Solutions:**

```bash
# Force shutdown
cco orchestration-server stop --force

# Or manual kill
kill -9 $(cat /var/run/cco-sidecar.pid)

# Increase shutdown timeout
cco orchestration-server stop --timeout 60
```

---

## Authentication Issues

### Issue: JWT token rejected

**Symptoms:**
- HTTP 401 Unauthorized
- "Invalid or expired JWT token"
- Agents can't access context

**Diagnosis:**

```bash
# Decode JWT token (using jwt.io or jwt-cli)
echo $JWT_TOKEN | jwt decode -

# Check expiration
echo $JWT_TOKEN | jwt decode - | jq '.exp'

# Test token
curl -H "Authorization: Bearer $JWT_TOKEN" \
  http://localhost:3001/api/context/test/python-specialist
```

**Solutions:**

```bash
# Refresh token (if agent)
# Token auto-refreshes 5 minutes before expiry

# Generate new token (if manual)
# Tokens are generated during agent spawn

# Check JWT secret matches
echo $CCO_SIDECAR_JWT_SECRET

# Restart sidecar with explicit secret
export CCO_SIDECAR_JWT_SECRET="your-secret-key"
cco orchestration-server
```

### Issue: Permission denied

**Symptoms:**
- HTTP 403 Forbidden
- "Agent lacks permission"
- Can read but not write

**Diagnosis:**

```bash
# Check token permissions
echo $JWT_TOKEN | jwt decode - | jq '.permissions'

# Expected permissions:
# - read_context
# - write_results
# - publish_events
# - spawn_agents (Chief Architect only)
```

**Solutions:**

```bash
# Verify agent type in token
echo $JWT_TOKEN | jwt decode - | jq '.agent_type'

# Respawn agent with correct type
cco agent spawn --type python-specialist --issue issue-123

# Check agent configuration
cat config/orchestra-config.json | jq '.codingAgents[] | select(.type=="python-specialist")'
```

---

## Context Issues

### Issue: Context not loading

**Symptoms:**
- HTTP 404 Not Found
- Empty context returned
- "No relevant files in context"

**Diagnosis:**

```bash
# Check context exists
cco context get issue-123 python-specialist

# Check cache status
curl http://localhost:3001/status | jq '.storage.context_cache_entries'

# Check file access
ls -la /tmp/cco-sidecar/context-cache/
```

**Solutions:**

```bash
# Clear and refresh context
cco context clear --issue issue-123
cco context refresh --issue issue-123 --force

# Check project directory
echo $PROJECT_ROOT
ls -la $PROJECT_ROOT

# Verify git repository
cd $PROJECT_ROOT
git status

# Manual context generation
cco context get issue-123 python-specialist --output context.json
cat context.json
```

### Issue: Context is truncated

**Symptoms:**
- `truncated: true` in response
- Missing files in context
- Incomplete git history

**Diagnosis:**

```bash
# Check context size
cco context get issue-123 python-specialist | jq '.total_size_bytes'

# Check truncation strategy
cco context get issue-123 python-specialist | jq '.truncation_strategy'
```

**Solutions:**

```bash
# Increase context size limit (not yet configurable)
# Future enhancement

# Request specific files (not yet implemented)
# Future enhancement

# Work with truncated context
# Prioritized files are always included
```

### Issue: Stale context

**Symptoms:**
- Context doesn't include recent changes
- Old file versions returned
- Missing new files

**Diagnosis:**

```bash
# Check cache timestamp
cco context get issue-123 python-specialist | jq '.timestamp'

# Compare to file modification times
ls -lt $PROJECT_ROOT/src/
```

**Solutions:**

```bash
# Force context refresh
cco context refresh --issue issue-123 --force

# Clear cache
cco context clear --issue issue-123

# Get fresh context
cco context get issue-123 python-specialist
```

---

## Event Issues

### Issue: Events not delivered

**Symptoms:**
- `wait_for_event` times out
- Subscribers not notified
- Event queue empty

**Diagnosis:**

```bash
# Check event queue
curl http://localhost:3001/status | jq '.events'

# List recent events
cco events list --limit 20

# Check subscriptions
curl http://localhost:3001/status | jq '.events.active_subscriptions'
```

**Solutions:**

```bash
# Verify event was published
cco events list --type agent_completed

# Check event topic
cco events list --topic implementation

# Verify filter syntax
# Correct: "issue_id:issue-123"
# Wrong:   "issue_id==issue-123"

# Test without filter
cco events subscribe agent_completed --timeout 60000

# Restart event bus (restart sidecar)
cco orchestration-server restart
```

### Issue: Event queue full

**Symptoms:**
- High queue depth (>9000)
- Event publishing fails
- Performance degradation

**Diagnosis:**

```bash
# Check queue depth
curl http://localhost:3001/status | jq '.events.queue_depth'

# Expected: < 1000
# Warning:  > 5000
# Critical: > 9000
```

**Solutions:**

```bash
# Events auto-expire after 24h
# Wait for old events to expire

# Or restart sidecar (clears queue)
cco orchestration-server restart

# Check for runaway event publishers
cco events list --limit 100 | jq '.[] | .publisher' | sort | uniq -c

# Reduce event TTL for new events
# (publish with shorter ttl_seconds)
```

### Issue: Long polling timeout

**Symptoms:**
- Subscribers always timing out
- No events received
- "timeout": true in response

**Diagnosis:**

```bash
# Check if events are being published
cco events list --topic implementation

# Check timeout setting
# Default: 30000ms (30 seconds)

# Check network connectivity
ping localhost
```

**Solutions:**

```bash
# Increase timeout
cco events subscribe agent_completed --timeout 60000

# Use continuous subscription
cco events subscribe agent_completed --continuous

# Check filter is not too restrictive
# Remove filter to test
cco events subscribe agent_completed
```

---

## Performance Issues

### Issue: Slow response times

**Symptoms:**
- High latency (>500ms)
- Timeouts
- Slow context retrieval

**Diagnosis:**

```bash
# Check performance metrics
curl http://localhost:3001/status | jq '.performance'

# Expected:
# - avg_response_time_ms: < 100
# - p99_response_time_ms: < 500

# Check system load
top
htop
```

**Solutions:**

```bash
# Increase cache size
cco orchestration-server --cache-size-mb 2048

# Clear cache to free memory
curl -X DELETE http://localhost:3001/api/cache/all

# Reduce concurrent agents
# (stop some agents)

# Check disk I/O
iostat -x 1

# Use SSD storage if possible
cco orchestration-server --storage /fast/ssd/path
```

### Issue: High memory usage

**Symptoms:**
- Memory usage >1GB
- System swapping
- OOM killer invoked

**Diagnosis:**

```bash
# Check memory usage
curl http://localhost:3001/health | jq '.checks.memory_usage_mb'

# Check cache size
curl http://localhost:3001/status | jq '.storage'

# System memory
free -h
```

**Solutions:**

```bash
# Reduce cache size
cco orchestration-server --cache-size-mb 512

# Clear cache
curl -X DELETE http://localhost:3001/api/cache/all

# Clear specific issue cache
curl -X DELETE http://localhost:3001/api/cache/context/issue-123

# Restart with lower limits
export CCO_SIDECAR_CACHE_SIZE_MB=512
cco orchestration-server
```

### Issue: High CPU usage

**Symptoms:**
- CPU usage >80%
- System lag
- Slow responses

**Diagnosis:**

```bash
# Check CPU usage
top | grep orchestration

# Check request rate
curl http://localhost:3001/status | jq '.performance.requests_per_second'
```

**Solutions:**

```bash
# Reduce concurrent agents
# (lower load)

# Check for infinite loops in agents
cco agent list --status active

# Rate limit aggressive agents
# (built into sidecar)

# Use multiple sidecar instances
# (not yet supported - future enhancement)
```

---

## Storage Issues

### Issue: Storage full

**Symptoms:**
- "No space left on device"
- Can't store results
- Context cache fails

**Diagnosis:**

```bash
# Check disk space
df -h /tmp/cco-sidecar

# Check storage size
du -sh /tmp/cco-sidecar/*

# Check storage status
curl http://localhost:3001/status | jq '.storage'
```

**Solutions:**

```bash
# Clear old results
rm -rf /tmp/cco-sidecar/results/old-project/*

# Clear cache
curl -X DELETE http://localhost:3001/api/cache/all

# Use different storage path
cco orchestration-server --storage /larger/disk/path

# Clean up event logs
rm /tmp/cco-sidecar/events/event-log.jsonl
```

### Issue: Storage corruption

**Symptoms:**
- Can't read results
- Invalid JSON errors
- Context cache errors

**Diagnosis:**

```bash
# Check file integrity
ls -la /tmp/cco-sidecar/results/
cat /tmp/cco-sidecar/results/project/issue-123/python-specialist.json

# Validate JSON
jq . /tmp/cco-sidecar/results/project/issue-123/python-specialist.json
```

**Solutions:**

```bash
# Remove corrupted file
rm /tmp/cco-sidecar/results/project/issue-123/python-specialist.json

# Clear all storage
rm -rf /tmp/cco-sidecar/*
cco orchestration-server

# Restore from backup (if available)
cp -r /backup/cco-sidecar/* /tmp/cco-sidecar/
```

---

## Network Issues

### Issue: Can't connect to sidecar

**Symptoms:**
- Connection refused
- Connection timeout
- Network unreachable

**Diagnosis:**

```bash
# Check sidecar is running
curl http://localhost:3001/health

# Check port binding
lsof -i :3001

# Check firewall
sudo ufw status
sudo iptables -L

# Check network connectivity
ping localhost
telnet localhost 3001
```

**Solutions:**

```bash
# Start sidecar if not running
cco orchestration-server

# Check bind address
cco orchestration-server --host 0.0.0.0

# Allow through firewall
sudo ufw allow 3001
sudo iptables -A INPUT -p tcp --dport 3001 -j ACCEPT

# Check for network namespace issues (Docker)
docker network inspect bridge
```

### Issue: Remote access fails

**Symptoms:**
- Works on localhost
- Fails from remote machine
- Connection timeout

**Diagnosis:**

```bash
# Check bind address
ps aux | grep orchestration-server | grep host

# Should be 0.0.0.0, not 127.0.0.1

# Test from remote
curl http://<server-ip>:3001/health
```

**Solutions:**

```bash
# Bind to all interfaces
cco orchestration-server --host 0.0.0.0

# Configure firewall
sudo ufw allow from <remote-ip> to any port 3001

# Use SSH tunnel (secure alternative)
ssh -L 3001:localhost:3001 user@server
curl http://localhost:3001/health
```

---

## Common Errors

### Error: "Invalid JWT token"

```
HTTP 401 Unauthorized
{"error": "unauthorized", "code": "AUTH_001"}
```

**Solution:**
```bash
# Check token expiration
echo $JWT_TOKEN | jwt decode - | jq '.exp'

# Respawn agent to get new token
cco agent spawn --type python-specialist --issue issue-123
```

### Error: "Context too large"

```
HTTP 500 Internal Server Error
{"error": "context_generation_failed", "code": "CTX_002"}
```

**Solution:**
```bash
# Context auto-truncates at 10MB
# This is a hard limit

# Request will return truncated context
# Check: context["truncated"] == true
```

### Error: "Event bus unavailable"

```
HTTP 500 Internal Server Error
{"error": "event_bus_unavailable", "code": "EVT_001"}
```

**Solution:**
```bash
# Restart sidecar
cco orchestration-server restart

# Check logs for root cause
tail -100 /tmp/cco-sidecar/logs/orchestration.log
```

### Error: "Rate limit exceeded"

```
HTTP 429 Too Many Requests
{"error": "rate_limit_exceeded", "code": "RATE_001"}
```

**Solution:**
```bash
# Wait for rate limit to reset
# Check Retry-After header
curl -I http://localhost:3001/api/...

# Reduce request rate
# Add delays between requests
time.sleep(1)
```

### Error: "Agent spawn failed"

```
HTTP 500 Internal Server Error
{"error": "spawn_failed", "code": "AGT_001"}
```

**Solution:**
```bash
# Check system resources
free -h
ps aux | wc -l

# Check agent configuration
cat config/orchestra-config.json | jq '.codingAgents[] | select(.type=="python-specialist")'

# Check logs
tail -100 /tmp/cco-sidecar/logs/orchestration.log
```

---

## Debug Mode

### Enable Debug Logging

```bash
# Start with debug logging
cco orchestration-server --log-level debug

# Or set environment variable
export CCO_SIDECAR_LOG_LEVEL=debug
cco orchestration-server
```

### Debug Output

```
[DEBUG] 2025-11-18T10:00:00Z Received GET /api/context/issue-123/python-specialist
[DEBUG] 2025-11-18T10:00:00Z JWT token validated: agent=python-specialist-uuid
[DEBUG] 2025-11-18T10:00:00Z Cache miss for context:issue-123:python-specialist
[DEBUG] 2025-11-18T10:00:01Z Gathering context: 42 files, 512KB
[DEBUG] 2025-11-18T10:00:01Z Context cached: key=context:issue-123:python-specialist
[DEBUG] 2025-11-18T10:00:01Z Response sent: 200 OK, 512KB, 1.2s
```

### Enable Request Tracing

```bash
# Trace all requests
export RUST_LOG=debug
cco orchestration-server

# Trace specific components
export RUST_LOG=orchestration::context=trace
cco orchestration-server
```

### Capture Network Traffic

```bash
# Capture HTTP traffic
sudo tcpdump -i lo -A -s 0 'tcp port 3001'

# Or use Wireshark
wireshark -i lo -f "tcp port 3001"
```

### Test with curl

```bash
# Verbose curl
curl -v -H "Authorization: Bearer $JWT_TOKEN" \
  http://localhost:3001/api/context/issue-123/python-specialist

# Save response headers
curl -D headers.txt -o response.json \
  http://localhost:3001/api/context/issue-123/python-specialist
```

---

## Getting Help

If none of these solutions work:

1. **Check logs**:
   ```bash
   tail -100 /tmp/cco-sidecar/logs/orchestration.log
   ```

2. **Collect debug info**:
   ```bash
   curl http://localhost:3001/health > health.json
   curl http://localhost:3001/status > status.json
   ps aux | grep orchestration > process.txt
   df -h > disk.txt
   free -h > memory.txt
   ```

3. **Create GitHub issue** with:
   - Error message
   - Steps to reproduce
   - Debug output
   - System information

4. **Check documentation**:
   - [FAQ](ORCHESTRATION_SIDECAR_FAQ.md)
   - [API Reference](ORCHESTRATION_SIDECAR_API_REFERENCE.md)
   - [Architecture](ORCHESTRATION_SIDECAR_ADVANCED.md)

---

## See Also

- [Quick Start Guide](ORCHESTRATION_SIDECAR_QUICKSTART.md)
- [FAQ](ORCHESTRATION_SIDECAR_FAQ.md)
- [API Reference](ORCHESTRATION_SIDECAR_API_REFERENCE.md)
- [Advanced Topics](ORCHESTRATION_SIDECAR_ADVANCED.md)
