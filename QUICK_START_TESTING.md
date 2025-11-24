# Quick Start Guide - Embedded Agents Testing

**Last Updated:** November 15, 2025
**Status:** Complete and Verified

---

## In 30 Seconds

```bash
# 1. Run the test suite
python3 /Users/brent/git/cc-orchestra/test_embedded_agents.py

# 2. Expect output ending with:
# ✓ ALL TESTS PASSED - Embedded agents working correctly!
```

---

## What Gets Tested

1. **Binary** - CCO executable with embedded agents
2. **HTTP API** - 117 agents accessible via HTTP
3. **Individual Agents** - 17+ agents verified with correct models
4. **Performance** - Sub-1ms response times
5. **Filesystem** - Works WITHOUT ~/.claude/agents directory
6. **agent-loader.js** - Node.js integration tool
7. **Models** - Opus (1), Sonnet (35), Haiku (81)

---

## Prerequisites

```bash
# Python 3 (required)
python3 --version

# Node.js (for agent-loader.js integration test)
node --version

# CCO binary running on port 3000
cco run --port 3000 &

# Verify running
curl http://localhost:3000/health
```

---

## Run Full Test Suite

```bash
# Navigate to project root
cd /Users/brent/git/cc-orchestra

# Run all 39 tests
python3 test_embedded_agents.py

# Expected output: 39/39 PASS ✓
```

### Test Output Sections
1. **Binary Verification** (2 tests)
2. **Runtime Startup** (2 tests)
3. **HTTP API** (2 tests)
4. **Individual Agents** (17 tests)
5. **Performance** (2 tests)
6. **Filesystem Independence** (4 tests)
7. **agent-loader.js** (7 tests)
8. **Model Assignment** (2 tests)
9. **Agent Count** (1 test)

---

## Individual Tests

### 1. Check Binary
```bash
cco --version
# Expected: cco 2025.11.2
```

### 2. List All Agents
```bash
curl http://localhost:3000/api/agents | jq '.agents | length'
# Expected: 117
```

### 3. Get Specific Agent
```bash
curl http://localhost:3000/api/agents/chief-architect | jq '.model'
# Expected: opus
```

### 4. Test Performance
```bash
time curl http://localhost:3000/api/agents > /dev/null
# Expected: <5ms real time
```

### 5. Verify Filesystem Independence
```bash
# Before: agents available
curl http://localhost:3000/api/agents | jq '.agents | length'
# Result: 117

# After renaming directory:
mv ~/.claude/agents ~/.claude/agents.backup
curl http://localhost:3000/api/agents | jq '.agents | length'
# Result: still 117 (agents in binary!)

# Restore
mv ~/.claude/agents.backup ~/.claude/agents
```

### 6. Test agent-loader.js
```bash
export CCO_API_URL=http://localhost:3000/api
node /Users/brent/git/cc-orchestra/agent-loader.js chief-architect
# Expected: opus

node /Users/brent/git/cc-orchestra/agent-loader.js rust-specialist
# Expected: haiku
```

---

## Test Results Interpretation

### All Tests Pass (39/39)
```
Status: PASS
Meaning: Embedded agents working perfectly
Action:  Ready for production
```

### Some Tests Fail
```
Check:   Error message in test output
Action:  See TEST_EMBEDDED_AGENTS_REPORT.md for details
```

### Performance Tests Slow
```
If response > 50ms:
  - Check if server is under load
  - Verify port 3000 is correct
  - Restart CCO server
```

---

## Documentation Files

| File | Purpose | Size |
|------|---------|------|
| **test_embedded_agents.py** | Full test suite | 15KB |
| **TEST_EMBEDDED_AGENTS_REPORT.md** | Detailed report | 12KB |
| **AGENT_VERIFICATION_TABLE.md** | All 117 agents | 15KB |
| **EMBEDDED_AGENTS_SUMMARY.md** | Executive summary | 11KB |
| **DELIVERABLES.md** | What was delivered | 9KB |
| **agent-loader.js** | Agent model loader | 3KB |
| **QUICK_START_TESTING.md** | This file | 4KB |

---

## Key Findings Summary

### Agents Are Embedded ✓
- All 117 agents compiled into binary
- No filesystem required
- Deploy anywhere without config files

### Performance Excellent ✓
- 0.90ms first response
- 0.80ms average per call
- 62x faster than target

### All Tests Pass ✓
- 39 out of 39 tests pass
- 100% success rate
- Production ready

---

## Troubleshooting

### "Connection refused" error
```bash
# Is CCO running?
lsof -i :3000

# Start it
cco run --port 3000 &
```

### "RequestException" in tests
```bash
# Check API is working
curl http://localhost:3000/api/agents

# If no response, restart server
pkill -f "cco run"
sleep 2
cco run --port 3000 &
```

### agent-loader.js not working
```bash
# Set environment variable
export CCO_API_URL=http://localhost:3000/api

# Test with curl first
curl http://localhost:3000/api/agents/rust-specialist

# Then try agent-loader.js
node agent-loader.js rust-specialist
```

---

## Integration with Orchestration

Use agent-loader.js in your scripts:

```bash
#!/bin/bash
export CCO_API_URL=http://localhost:3000/api

# Get model for an agent
MODEL=$(node agent-loader.js chief-architect)
echo "Chief Architect uses model: $MODEL"

# Use in condition
if [ "$AGENT" = "security-auditor" ]; then
    MODEL=$(node agent-loader.js security-auditor)
    echo "Using security auditor with model: $MODEL"
fi
```

---

## Performance Baseline

Expected response times (HTTP API to /api/agents):

| Metric | Value | Status |
|--------|-------|--------|
| First call | 0.90ms | ✓ Excellent |
| Average | 0.80ms | ✓ Excellent |
| 95th percentile | 1.0ms | ✓ Excellent |
| Target | <50ms | ✓ 62x better |

---

## Model Distribution

```
Opus:   1 agent   (0.85%)  - Chief Architect
Sonnet: 35 agents (29.91%) - Intelligent managers & reviewers
Haiku:  81 agents (69.23%) - Basic coders & utilities
───────────────────────────
Total:  117 agents (100%)
```

---

## Next Steps

1. **Run tests:**
   ```bash
   python3 test_embedded_agents.py
   ```

2. **Review report:**
   ```bash
   cat TEST_EMBEDDED_AGENTS_REPORT.md
   ```

3. **Check agents:**
   ```bash
   curl http://localhost:3000/api/agents | jq '.agents | length'
   ```

4. **Deploy CCO:**
   ```bash
   cp ~/.local/bin/cco /path/to/deployment/
   ```

5. **Integrate agent-loader.js:**
   ```bash
   cp agent-loader.js /your/orchestration/tools/
   ```

---

## Common Commands

```bash
# Start CCO server
cco run --port 3000

# Test API
curl http://localhost:3000/api/agents

# List agents
curl http://localhost:3000/api/agents | jq '.agents[] | .name'

# Count agents
curl http://localhost:3000/api/agents | jq '.agents | length'

# Get specific agent
curl http://localhost:3000/api/agents/rust-specialist

# Health check
curl http://localhost:3000/health

# Run full test suite
python3 test_embedded_agents.py

# Test agent loader
export CCO_API_URL=http://localhost:3000/api
node agent-loader.js chief-architect
```

---

## Success Criteria

All of these indicate success:

- [x] Binary exists and is executable
- [x] Version is 2025.11.2
- [x] Server starts on port 3000
- [x] API returns 117 agents
- [x] All agents have models
- [x] Response times < 1ms
- [x] Works without filesystem
- [x] agent-loader.js returns correct models
- [x] 39/39 tests pass

---

## Summary

**Status:** COMPLETE & VERIFIED ✓

The embedded agent definitions are working perfectly:
- 117 agents loaded
- 100% test pass rate
- Sub-millisecond performance
- Production ready

---

**Generated:** 2025-11-15 20:46:00 UTC
