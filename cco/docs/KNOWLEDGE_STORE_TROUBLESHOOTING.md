# Knowledge Store Troubleshooting Guide

**Common issues, diagnostic procedures, and solutions for the Knowledge Store**

**Version:** 1.0.0
**Last Updated:** November 28, 2025
**Status:** Active

---

## Table of Contents

1. [Quick Diagnostics](#quick-diagnostics)
2. [Connection Issues](#connection-issues)
3. [Authentication Issues](#authentication-issues)
4. [Storage Issues](#storage-issues)
5. [Search Issues](#search-issues)
6. [Performance Issues](#performance-issues)
7. [Data Issues](#data-issues)
8. [API Issues](#api-issues)
9. [Logging & Debugging](#logging--debugging)
10. [Getting Help](#getting-help)

---

## Quick Diagnostics

### Health Check

**Step 1: Verify daemon is running**
```bash
cco daemon status
# Output: Daemon running on port 8303, PID 12345
```

**Step 2: Verify API token exists**
```bash
[ -f ~/.cco/api_token ] && echo "Token exists" || echo "Token missing"
```

**Step 3: Test health endpoint**
```bash
curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .total_records
```

**Step 4: Check file permissions**
```bash
ls -ld ~/.cco/knowledge
# Should show: drwx------
```

### Diagnostic Script

**Create `check-knowledge-store.sh`:**
```bash
#!/bin/bash
set -e

echo "=== Knowledge Store Diagnostics ==="

# Check daemon
echo -n "1. Daemon status: "
if cco daemon status &>/dev/null; then
    echo "OK"
else
    echo "FAILED - Daemon not running"
    exit 1
fi

# Check token
echo -n "2. API token: "
if [ -f ~/.cco/api_token ] && [ -s ~/.cco/api_token ]; then
    echo "OK"
else
    echo "FAILED - Token missing or empty"
    exit 1
fi

# Check health
echo -n "3. Health check: "
if curl -s http://localhost:8303/api/knowledge/stats \
    -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
    | jq .repository &>/dev/null; then
    echo "OK"
else
    echo "FAILED - Health check failed"
    exit 1
fi

# Check permissions
echo -n "4. File permissions: "
PERMS=$(stat -c %a ~/.cco/knowledge 2>/dev/null || stat -f %A ~/.cco/knowledge)
if [ "$PERMS" = "700" ]; then
    echo "OK"
else
    echo "FAILED - Permissions are $PERMS (should be 700)"
    exit 1
fi

echo ""
echo "All checks passed!"
```

**Run it:**
```bash
chmod +x check-knowledge-store.sh
./check-knowledge-store.sh
```

---

## Connection Issues

### Issue 1: "Connection refused"

**Symptom:**
```
error: Connection refused (os error 111)
Cannot connect to http://localhost:8303
```

**Diagnosis:**
```bash
# Check if daemon is running
cco daemon status

# Check if port 8303 is in use
lsof -i :8303

# Try restarting
cco daemon stop
cco daemon start
```

**Solution:**

**Step 1: Verify daemon is running**
```bash
ps aux | grep cco.*daemon
# Should show daemon process
```

**Step 2: Check port availability**
```bash
# On macOS
lsof -i :8303

# On Linux
netstat -tlnp | grep 8303
```

**Step 3: Start daemon if not running**
```bash
cco daemon start
sleep 2
cco daemon status
```

**Step 4: Check firewall**
```bash
# macOS firewall check
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
```

**Step 5: Restart if still not working**
```bash
cco daemon restart
sleep 3
curl http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

---

### Issue 2: "Connection timed out"

**Symptom:**
```
error: Connection timed out
timeout waiting for connection
```

**Diagnosis:**
```bash
# Check if daemon is responsive
timeout 5 curl http://localhost:8303/api/knowledge/stats

# Check system resources
top -bn1 | grep Mem
# Check if RAM exhausted

ps aux | grep cco
# Check if process still running
```

**Solution:**

**Step 1: Check if daemon is overloaded**
```bash
# Check memory usage
ps aux | grep cco | grep -v grep | awk '{print $6}'
# Should be < 1GB typical

# Check CPU
top -b -n1 | grep cco
```

**Step 2: Reduce concurrent requests**
```bash
# Limit concurrent requests if high volume
# Implement request throttling in client
sleep 0.1  # Add delay between requests
```

**Step 3: Increase timeout in client**
```python
# Python
response = requests.post(
    url,
    timeout=30  # Increase from default 5s
)

# Bash
curl --max-time 30 url
```

**Step 4: Restart daemon if needed**
```bash
cco daemon restart
```

---

## Authentication Issues

### Issue 3: "401 Unauthorized"

**Symptom:**
```json
{
  "error": "Unauthorized",
  "code": 401
}
```

**Diagnosis:**
```bash
# Check token exists
cat ~/.cco/api_token

# Check token is not empty
[ -s ~/.cco/api_token ] && echo "Has content" || echo "Empty"

# Check token in header
TOKEN=$(cat ~/.cco/api_token)
curl -v http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $TOKEN" 2>&1 | grep -i authorization
```

**Solution:**

**Step 1: Verify token file exists**
```bash
ls -la ~/.cco/api_token
# Should show -rw-r--r--
```

**Step 2: Check token content**
```bash
TOKEN=$(cat ~/.cco/api_token)
echo "Token length: ${#TOKEN}"
echo "Token: $TOKEN"
# Should be 30+ characters
```

**Step 3: Verify header format**
```bash
# Correct format
curl -H "Authorization: Bearer $(cat ~/.cco/api_token)" http://localhost:8303/api/knowledge/stats

# Wrong formats that won't work
# -H "Authorization: $(cat ~/.cco/api_token)"  # Missing "Bearer"
# -H "Token: $(cat ~/.cco/api_token)"  # Wrong header name
```

**Step 4: Regenerate token**
```bash
# Restart daemon to generate new token
cco daemon restart

# Verify new token works
curl http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

---

### Issue 4: Token file location wrong

**Symptom:**
```
FileNotFoundError: [Errno 2] No such file or directory: '/wrong/path/api_token'
```

**Solution:**

**Step 1: Find token location**
```bash
# Standard location
ls ~/.cco/api_token

# Search for it
find ~ -name "api_token" 2>/dev/null
```

**Step 2: Use correct path in code**
```python
# Python
import os

token_path = os.path.expanduser("~/.cco/api_token")
with open(token_path) as f:
    token = f.read().strip()

# Don't use relative paths
# token_path = ".cco/api_token"  # WRONG
```

**Step 3: Verify path**
```bash
TOKEN_PATH=~/.cco/api_token
[ -f "$TOKEN_PATH" ] && echo "Found" || echo "Not found"
```

---

## Storage Issues

### Issue 5: "No storage directory"

**Symptom:**
```
Error: Failed to create knowledge directory
Directory not found: ~/.cco/knowledge/cc-orchestra
```

**Diagnosis:**
```bash
# Check if directory exists
ls -ld ~/.cco/knowledge/

# Check if parent exists
ls -ld ~/.cco/

# Check if home exists
ls -d ~/
```

**Solution:**

**Step 1: Verify directory exists**
```bash
# Check ~/.cco/knowledge exists
ls -ld ~/.cco/knowledge/ || echo "Missing"

# Check project directory
ls -ld ~/.cco/knowledge/cc-orchestra/ || echo "Missing"
```

**Step 2: Create if missing**
```bash
# Create knowledge directory
mkdir -p ~/.cco/knowledge

# Set permissions
chmod 0o700 ~/.cco/knowledge

# Verify
ls -ld ~/.cco/knowledge/
```

**Step 3: Restart daemon**
```bash
cco daemon restart
```

---

### Issue 6: "Permission denied"

**Symptom:**
```
Error: Permission denied (os error 13)
Operation not permitted: ~/.cco/knowledge
```

**Diagnosis:**
```bash
# Check current permissions
ls -ld ~/.cco/knowledge/

# Check if you own it
ls -ld ~/.cco/knowledge | awk '{print $3}'

# Try to access
cd ~/.cco/knowledge && pwd
```

**Solution:**

**Step 1: Fix permissions**
```bash
# Fix directory permissions
chmod 0o700 ~/.cco/knowledge

# Fix file permissions
find ~/.cco/knowledge -type f -exec chmod 0o600 {} \;
find ~/.cco/knowledge -type d -exec chmod 0o700 {} \;
```

**Step 2: Verify ownership**
```bash
# Check owner
ls -l ~/.cco/ | grep knowledge

# Change if needed (if you own ~/.cco)
chown -R $(whoami) ~/.cco/knowledge
```

**Step 3: Test access**
```bash
# Try to list
ls ~/.cco/knowledge/

# Try to create file
touch ~/.cco/knowledge/test.txt && rm ~/.cco/knowledge/test.txt
```

---

## Search Issues

### Issue 7: "No results found"

**Symptom:**
```
Search returned 0 results
Expected results but got empty array
```

**Diagnosis:**
```bash
# Check if items stored
curl http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .total_records

# Check project_id matches
curl http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .by_project

# Check search query
curl -X POST http://localhost:8303/api/knowledge/search \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"query":"test","limit":10}' | jq .
```

**Solution:**

**Step 1: Verify items are stored**
```bash
# Get stats
STATS=$(curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)")

echo "Total records: $(echo $STATS | jq .total_records)"
echo "By project: $(echo $STATS | jq .by_project)"
```

**Step 2: Check project_id**
```bash
# Current project
PROJECT=$(basename $(pwd))
echo "Current project: $PROJECT"

# Stored projects
curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .by_project
```

**Step 3: Search without project filter**
```bash
# Try search without filtering
curl -X POST http://localhost:8303/api/knowledge/search \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"query":"architecture","limit":10}' | jq '.[].knowledge_type'
```

**Step 4: Check knowledge types**
```bash
# Get all unique types
curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .by_type
```

---

### Issue 8: "Low similarity scores"

**Symptom:**
```
Scores are very low (< 0.2) even for similar items
Query should match but score < threshold
```

**Diagnosis:**
```bash
# Check embedding consistency
# Same text should always produce same vector
# Different text should produce different vectors

# Test with exact match
curl -X POST http://localhost:8303/api/knowledge/search \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"query":"exact stored text","limit":1}' | jq .[0].score
# Should be close to 1.0
```

**Solution:**

**Step 1: Understand embedding approach**
```
Current approach: SHA256-based (deterministic, not semantic)
- Same text → same embedding (score 1.0)
- Very different text → different embedding (score ~0.0)
- Semantically similar but different text → may have low score

This is expected! Not using ML embeddings.
```

**Step 2: Lower threshold if needed**
```bash
# Try with lower threshold
curl -X POST http://localhost:8303/api/knowledge/search \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"query":"test","threshold":0.0,"limit":10}' | jq '.[] | {score, text}'
```

**Step 3: Use exact text when possible**
```bash
# For better results, search with exact phrases stored
# Instead of: "FastAPI framework"
# Search for: "We decided to use FastAPI" (exact stored text)
```

---

## Performance Issues

### Issue 9: "Slow search performance"

**Symptom:**
```
Search taking >100ms
curl timing shows high latency
```

**Diagnosis:**
```bash
# Measure latency
time curl -X POST http://localhost:8303/api/knowledge/search \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"query":"test","limit":10}'

# Check item count
curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .total_records

# Check system load
top -bn1 | head -3
```

**Solution:**

**Step 1: Check item count (scales linearly)**
```bash
# Get count
COUNT=$(curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .total_records)

echo "Items: $COUNT"
# Expected latency = COUNT * 0.01ms
# 1000 items = ~10ms
# 10000 items = ~100ms
# 100000 items = ~1000ms (slow!)
```

**Step 2: Reduce limit**
```bash
# Smaller limit = faster search
curl -X POST http://localhost:8303/api/knowledge/search \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"query":"test","limit":5}' # Was 100

# Time difference should be minimal (limit only affects sorting)
```

**Step 3: Reduce item count**
```bash
# Clean up old items
curl -X POST http://localhost:8303/api/knowledge/cleanup \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"older_than_days":30}'

# Check new count
curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .total_records
```

**Step 4: Profile operation**
```bash
# Enable debug logging
RUST_LOG=debug cco daemon restart

# Run search and check logs
curl -X POST http://localhost:8303/api/knowledge/search ...

# View logs
tail -50 ~/.cco/logs/daemon.log | grep -i "search\|duration"
```

---

### Issue 10: "High memory usage"

**Symptom:**
```
Daemon using > 500MB RAM
Memory keeps growing
```

**Diagnosis:**
```bash
# Check memory usage
ps aux | grep cco | grep daemon | awk '{print "PID:", $2, "Memory:", $6 " KB"}'

# Calculate expected usage
ITEMS=$(curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" | jq .total_records)
echo "Expected: $((ITEMS * 2)) KB (2KB per item)"

# Check for memory leak
ps aux | grep cco | grep daemon | awk '{print $6}' &
sleep 60
ps aux | grep cco | grep daemon | awk '{print $6}'
```

**Solution:**

**Step 1: Calculate expected memory**
```bash
# Each item ≈ 2 KB
# 10K items = ~20 MB
# 100K items = ~200 MB
# 500K items = ~1 GB

# Check if memory is reasonable
STATS=$(curl -s http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)")

ITEMS=$(echo $STATS | jq .total_records)
EXPECTED=$((ITEMS * 2 / 1024))  # MB

MEM_ACTUAL=$(ps aux | grep '[c]co.*daemon' | awk '{print $6 / 1024}')

echo "Items: $ITEMS, Expected: ${EXPECTED}MB, Actual: ${MEM_ACTUAL}MB"
```

**Step 2: Reduce item count if excessive**
```bash
# Cleanup old items
curl -X POST http://localhost:8303/api/knowledge/cleanup \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"older_than_days":7}'
```

**Step 3: Restart daemon periodically**
```bash
# Add to crontab for daily restart
# 2 3 * * * cco daemon restart > /dev/null 2>&1
```

---

## Data Issues

### Issue 11: "Data lost after restart"

**Symptom:**
```
Items disappeared after daemon restart
stats shows total_records=0
```

**Explanation:**
```
Current implementation: In-memory storage only
Data is lost when daemon restarts (expected behavior)

This is temporary until LanceDB integration.
```

**Workaround:**

```bash
# Store items before compaction
curl -X POST http://localhost:8303/api/knowledge/pre-compaction \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{
    "conversation": "full conversation text",
    "project_id": "my-project",
    "session_id": "session-123"
  }'

# Items will be re-stored on next daemon startup
# (Future: with disk persistence)
```

---

### Issue 12: "Duplicate items stored"

**Symptom:**
```
Same item appears multiple times
Search returns duplicates
```

**Solution:**

**Step 1: Check for duplicates**
```python
import requests

client = requests.Session()
TOKEN = open(os.path.expanduser("~/.cco/api_token")).read().strip()
headers = {"Authorization": f"Bearer {TOKEN}"}

# Search for duplicates
results = requests.post(
    "http://localhost:8303/api/knowledge/search",
    headers=headers,
    json={"query": "known text", "limit": 100}
).json()

texts = [r["text"] for r in results]
print(f"Total: {len(texts)}, Unique: {len(set(texts))}")

# Show duplicates
from collections import Counter
duplicates = [t for t, c in Counter(texts).items() if c > 1]
print("Duplicates:", duplicates)
```

**Step 2: Prevent duplicates in client**
```python
# Track stored items
stored_texts = set()

def store_if_new(text, type, agent):
    if text not in stored_texts:
        client.store(text, type, agent)
        stored_texts.add(text)
```

---

## API Issues

### Issue 13: "400 Bad Request"

**Symptom:**
```json
{
  "error": "Invalid request: ...",
  "code": 400
}
```

**Diagnosis:**
```bash
# Check request format
curl -X POST http://localhost:8303/api/knowledge/store \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"text":"test"}'  # Minimal valid request

# Check JSON validity
echo '{"invalid json}' | jq .  # Will error
```

**Solution:**

**Step 1: Validate JSON**
```bash
# Test JSON
PAYLOAD='{"text":"test knowledge","type":"decision","agent":"architect"}'
echo "$PAYLOAD" | jq .  # Should output formatted JSON

# If error: fix JSON syntax
```

**Step 2: Check required fields**
```bash
# Minimum required: text
curl -X POST http://localhost:8303/api/knowledge/store \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{"text":"hello world"}'
```

**Step 3: Check field types**
```bash
# text must be string (not number)
# limit must be number (not string)
# threshold must be number between 0-1

# Correct:
{"text":"string","limit":10,"threshold":0.5}

# Wrong:
{"text":123,"limit":"10","threshold":"0.5"}
```

---

### Issue 14: "413 Payload Too Large"

**Symptom:**
```json
{
  "error": "Text field exceeds 10 MB limit (got 15728640 bytes)",
  "code": 413
}
```

**Solution:**

**Step 1: Check text size**
```bash
# Get text size in Python
import sys
text = "your knowledge text here"
print(f"Size: {len(text)} bytes ({len(text) / 1024 / 1024:.1f} MB)")

# Limits:
# Single text: 10 MB
# Batch total: 50 MB
# Query: 100 KB
```

**Step 2: Split large knowledge**
```python
# Don't store entire conversation
# Break into meaningful chunks
chunks = conversation.split("\n\n")  # By paragraph
chunks = [c for c in chunks if len(c) > 50]  # Filter small

for chunk in chunks:
    if len(chunk) <= 10 * 1024 * 1024:
        client.store(chunk, "general", "agent")
```

---

## Logging & Debugging

### Enable Debug Logging

**Start daemon with debug logs:**
```bash
# Set log level
RUST_LOG=debug cco daemon start

# Or for specific module
RUST_LOG=cco::daemon::knowledge=trace cco daemon start
```

**View logs:**
```bash
# Tail logs
tail -f ~/.cco/logs/daemon.log

# Search for errors
grep ERROR ~/.cco/logs/daemon.log

# Search for specific operation
grep -i "store\|search" ~/.cco/logs/daemon.log | tail -20
```

### Performance Tracing

**Measure operation timing:**
```bash
# Use curl with timing
curl -w "\nTotal time: %{time_total}s\n" \
  -X POST http://localhost:8303/api/knowledge/search \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"query":"test","limit":10}'
```

**Create timing script:**
```bash
#!/bin/bash
# measure-search.sh

TOKEN=$(cat ~/.cco/api_token)

for i in {1..5}; do
    echo "Run $i:"
    curl -w "  Query: %{time_starttransfer}s total: %{time_total}s\n" \
      -X POST http://localhost:8303/api/knowledge/search \
      -H "Authorization: Bearer $TOKEN" \
      -d '{"query":"test","limit":10}' \
      -o /dev/null -s
done
```

---

## Getting Help

### Before Filing Issue

1. **Run diagnostics**
   ```bash
   ./check-knowledge-store.sh
   ```

2. **Collect logs**
   ```bash
   RUST_LOG=debug cco daemon restart
   sleep 5
   # Reproduce issue
   tail -100 ~/.cco/logs/daemon.log > issue-logs.txt
   ```

3. **Check related docs**
   - API Reference: KNOWLEDGE_STORE_API.md
   - Architecture: KNOWLEDGE_STORE_ARCHITECTURE.md
   - Security: KNOWLEDGE_STORE_SECURITY.md

### File GitHub Issue

**Include:**
- OS and version
- Diagnostic output
- Error message (full stack trace if available)
- Steps to reproduce
- Expected vs actual behavior
- Recent logs (last 100 lines)

**Template:**
```markdown
## Issue: [Brief description]

### Environment
- OS: macOS 14.1
- CCO version: 2025.11.28
- Rust version: 1.72

### Reproduction Steps
1. Run daemon with: `cco daemon start`
2. Execute: `curl ...`
3. Observe error: ...

### Expected
Should return 10 results

### Actual
Returns 0 results

### Logs
[Last 50 lines of daemon.log]
```

---

**Last Updated:** November 28, 2025
**Version:** 1.0.0
**Maintained by:** CCO Support Team
