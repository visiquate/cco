# End-to-End Agent Verification Tests

## Overview

This test suite provides comprehensive end-to-end verification of the agent definition system, validating the complete flow from CCO server startup through Claude Code agent spawning.

## What Gets Tested

### 1. CCO Server & HTTP API
- Server startup and health checks
- `/api/agents` endpoint (list all agents)
- `/api/agents/{agent-name}` endpoint (get specific agent)
- Response structure and data correctness
- Error handling (404 for non-existent agents)
- Response times and performance

### 2. Agent Definition Loading
- All 117-119 agents loaded correctly
- Correct model assignments:
  - 1 agent with `opus` (chief-architect)
  - ~37 agents with `sonnet` (managers, reviewers, architects)
  - ~81 agents with `haiku` (basic coders, documentation, utilities)
- Agent data structure validation

### 3. agent-loader.js Integration
- API connection and data retrieval
- Correct model extraction from API responses
- CLI usage functionality
- Logging and error messages

### 4. Fallback Mechanisms
- Fallback to local `~/.claude/agents/*.md` files when API unavailable
- Graceful degradation on network issues
- Timeout handling
- Connection refused handling

### 5. End-to-End Flow Simulation
- Complete agent spawning workflow
- Model retrieval → Task spawning flow
- Verification that Claude Code would spawn agents with correct models

## Prerequisites

### System Requirements
- macOS, Linux, or WSL (bash required)
- Node.js (v14 or higher)
- Rust toolchain (for building CCO)
- curl (HTTP client)
- jq (JSON processor)

### Installation

```bash
# Install jq (if not already installed)
# macOS:
brew install jq

# Ubuntu/Debian:
sudo apt-get install jq

# Verify installations
node --version  # Should show v14+
jq --version    # Should show jq-1.6 or higher
curl --version  # Should be available
```

### File Structure
```
cco/
├── tests/
│   ├── e2e-agent-verification.sh       # Main test script
│   ├── TEST_VERIFICATION_CHECKLIST.md  # Manual verification checklist
│   ├── TEST_RESULTS.md                 # Test results template
│   └── E2E_TEST_README.md              # This file
├── src/
│   └── server.rs                       # CCO server implementation
└── Cargo.toml
```

## Running the Tests

### Quick Start

```bash
# 1. Navigate to CCO directory
cd /Users/brent/git/cc-orchestra/cco

# 2. Build CCO server (if not already built)
cargo build --release

# 3. Make test script executable
chmod +x tests/e2e-agent-verification.sh

# 4. Run all tests
./tests/e2e-agent-verification.sh
```

### Test Output

The test script provides real-time output with color-coded results:

```
[INFO] Starting CCO server...
[PASS] CCO server is running
[INFO] Test 1: GET /api/agents - List all agents
[PASS] API_LIST_ALL_AGENTS
  → Found 119 agents
[INFO] Test 2: GET /api/agents/{agent-name} - Verify specific agent models
  Testing: chief-architect (expect: opus)
[PASS]     chief-architect: opus ✓
  Testing: rust-specialist (expect: haiku)
[PASS]     rust-specialist: haiku ✓
...
```

### Test Results Location

Results are saved to a timestamped JSON file:
```
/tmp/e2e-test-results-<timestamp>.json
```

Example:
```bash
# View results
cat /tmp/e2e-test-results-*.json | jq .

# Extract summary
cat /tmp/e2e-test-results-*.json | jq '.summary'
```

## Test Suite Breakdown

### Test 1: List All Agents
**Purpose:** Verify all agents are loaded and accessible via HTTP API

**What it tests:**
- GET /api/agents returns valid JSON
- Response contains "agents" array
- Agent count is 117-119
- Response time is acceptable

**Success criteria:**
- Valid JSON response
- Agent count in expected range
- Response < 100ms

### Test 2: Get Specific Agents
**Purpose:** Verify individual agent retrieval with correct models

**Agents tested:**
- chief-architect → opus
- rust-specialist → haiku
- test-engineer → haiku
- security-auditor → sonnet
- api-explorer → sonnet
- python-specialist → haiku
- tdd-coding-agent → haiku
- devops-engineer → haiku
- documentation-expert → haiku
- backend-architect → sonnet

**Success criteria:**
- Each agent returns correct model
- Response structure valid
- Response < 50ms per agent

### Test 3: 404 Error Handling
**Purpose:** Verify proper error responses

**What it tests:**
- Request non-existent agent
- Verify 404 HTTP status code
- Check error message format

**Success criteria:**
- Returns HTTP 404
- Error message in JSON format

### Test 4: Response Structure
**Purpose:** Validate API response schema

**Required fields:**
- `name` - Agent identifier
- `model` - Assigned model (opus/sonnet/haiku)
- `description` - Agent description
- `tools` - Available tools array

**Success criteria:**
- All required fields present
- Data types correct
- No unexpected fields

### Test 5: Response Time
**Purpose:** Verify API performance

**Metrics:**
- List all agents: < 100ms
- Individual agent: < 50ms

**Success criteria:**
- Response times within targets
- Consistent performance across runs

### Test 6: agent-loader with API
**Purpose:** Verify JavaScript loader integration

**What it tests:**
- CCO_API_URL environment variable usage
- HTTP request to CCO server
- Model extraction from API response
- Logging shows "from API" source

**Success criteria:**
- Correct models returned
- API used as primary source
- Errors handled gracefully

### Test 7: CLI Usage
**Purpose:** Verify command-line interface

**Command tested:**
```bash
node ~/.claude/agent-loader.js rust-specialist
```

**Success criteria:**
- Exits with code 0
- Outputs correct model
- No errors on stderr

### Test 8: Fallback Mechanism
**Purpose:** Verify graceful degradation

**What it tests:**
- Stop CCO server
- agent-loader falls back to local files
- Correct model still returned
- Fallback logged appropriately

**Success criteria:**
- Fallback triggered automatically
- Log shows fallback message
- Correct model from local files
- CCO server restarts successfully

### Test 9: Network Timeout
**Purpose:** Verify timeout handling

**What it tests:**
- Use unreachable IP (10.255.255.1)
- Verify quick fallback
- No hanging or indefinite wait

**Success criteria:**
- Fallback within 5 seconds
- Timeout error logged
- Continues to function

### Test 10: E2E Agent Spawning
**Purpose:** Simulate complete Claude Code workflow

**Simulated flow:**
```javascript
// What Claude Code does:
model = getAgentModel('rust-specialist')  // Step 1
Task("Rust Specialist", "...", "rust-specialist", model)  // Step 2
```

**Success criteria:**
- Correct model returned
- Flow completes successfully
- Agent would spawn with correct model

### Test 11: Agent Count
**Purpose:** Verify complete agent loading

**What it tests:**
- Total agent count from API
- Count matches expected range (117-119)

**Success criteria:**
- Count in expected range
- No missing agents
- No duplicate entries

### Test 12: Model Distribution
**Purpose:** Validate model assignments

**Expected distribution:**
- 1 opus (chief-architect)
- ~37 sonnet (managers, reviewers, architects)
- ~81 haiku (basic coders, docs, utilities)

**Success criteria:**
- Opus count exactly 1
- Sonnet count 30-45
- Haiku count 70-90

## Understanding Test Results

### Success Output
```
==========================================
  E2E Agent Verification Test Summary
==========================================

Total Tests:    12
Passed:         12
Failed:         0
Warnings:       0

✓ ALL TESTS PASSED

Full results: /tmp/e2e-test-results-1731709200.json
```

### Failure Output
```
==========================================
  E2E Agent Verification Test Summary
==========================================

Total Tests:    12
Passed:         10
Failed:         2
Warnings:       1

✗ SOME TESTS FAILED

Failed Tests:
  - API_GET_SPECIFIC_AGENTS: rust-specialist returned 'sonnet', expected 'haiku'
  - FALLBACK_MECHANISM: Fallback took 8s, expected < 5s

Warnings:
  - List endpoint slow: 150ms > 100ms
```

### JSON Results Format
```json
{
  "timestamp": "2025-11-15T20:00:00Z",
  "summary": {
    "total": 12,
    "passed": 12,
    "failed": 0,
    "warnings": 0,
    "success_rate": 100.00
  },
  "environment": {
    "cco_api_url": "http://127.0.0.1:3210",
    "cco_port": 3210,
    "agent_loader_path": "/Users/brent/.claude/agent-loader.js"
  },
  "test_results": {
    "API_LIST_ALL_AGENTS": "PASS",
    "API_GET_SPECIFIC_AGENTS": "PASS",
    ...
  },
  "failures": [],
  "warnings": []
}
```

## Troubleshooting

### CCO Server Won't Start

**Problem:** Server fails to bind to port

**Solution:**
```bash
# Check if port is in use
lsof -i :3210

# Kill process if needed
kill $(lsof -t -i :3210)

# Or use different port
CCO_PORT=3211 ./tests/e2e-agent-verification.sh
```

### Agent Loader Not Found

**Problem:** `~/.claude/agent-loader.js` doesn't exist

**Solution:**
```bash
# Check if file exists
ls -la ~/.claude/agent-loader.js

# If missing, ensure it's in the correct location
# The file should have been created from the project source
```

### Missing Dependencies

**Problem:** `jq: command not found` or `curl: command not found`

**Solution:**
```bash
# Install jq
brew install jq  # macOS
sudo apt-get install jq  # Ubuntu

# curl should be pre-installed on most systems
# If missing:
brew install curl  # macOS
sudo apt-get install curl  # Ubuntu
```

### Tests Hang or Timeout

**Problem:** Tests don't complete

**Solution:**
```bash
# Increase timeout (edit script)
# Look for timeout values in e2e-agent-verification.sh

# Or kill and restart
pkill -f "cargo run.*cco"
./tests/e2e-agent-verification.sh
```

### Agent Files Missing

**Problem:** Fallback tests fail because `~/.claude/agents/` is empty

**Solution:**
```bash
# Verify agent files exist
ls -la ~/.claude/agents/

# Should show 117-119 .md files
# If missing, agents may not have been deployed yet
```

## Advanced Usage

### Running Specific Tests

Edit the script to comment out tests you don't want to run:

```bash
# In e2e-agent-verification.sh main() function:
# test_list_all_agents          # ← Comment out to skip
test_get_specific_agents
test_404_handling
# test_response_structure       # ← Comment out to skip
...
```

### Custom Configuration

```bash
# Use custom port
CCO_PORT=4000 ./tests/e2e-agent-verification.sh

# Use custom host
CCO_HOST=localhost ./tests/e2e-agent-verification.sh

# Both
CCO_PORT=4000 CCO_HOST=0.0.0.0 ./tests/e2e-agent-verification.sh
```

### Debugging

```bash
# Run with verbose output
bash -x ./tests/e2e-agent-verification.sh

# Check CCO server logs
tail -f /tmp/cco-e2e-test.log

# Manual API testing
curl -s http://127.0.0.1:3210/api/agents | jq .
curl -s http://127.0.0.1:3210/api/agents/rust-specialist | jq .
```

## Documentation Files

### TEST_VERIFICATION_CHECKLIST.md
Complete checklist for manual verification including:
- Pre-test requirements
- Test execution steps
- Success criteria
- Post-test validation
- Sign-off section

### TEST_RESULTS.md
Comprehensive test results template with:
- Executive summary
- Detailed test results
- Performance metrics
- Issues found
- Recommendations

### E2E_TEST_README.md
This file - complete guide to running and understanding the E2E tests.

## Success Criteria

All tests must pass with:
- **0 failures**
- **API response times acceptable** (< 100ms list, < 50ms individual)
- **Fallback mechanism works** (falls back within 5s)
- **Correct model distribution** (1 Opus, ~37 Sonnet, ~81 Haiku)
- **End-to-end flow validated** (agent spawning simulation works)

## Support

For issues or questions:

1. Check troubleshooting section above
2. Review test logs: `/tmp/cco-e2e-test.log`
3. Review test results: `/tmp/e2e-test-results-*.json`
4. Check CCO server status: `lsof -i :3210`
5. Verify agent files: `ls ~/.claude/agents/`

## Related Files

```
cco/tests/
├── e2e-agent-verification.sh          # Main test script (executable)
├── TEST_VERIFICATION_CHECKLIST.md     # Manual verification checklist
├── TEST_RESULTS.md                    # Results template/documentation
└── E2E_TEST_README.md                 # This file
```

## Version

**Test Suite Version:** 1.0
**Last Updated:** 2025-11-15
**Compatible with:** CCO v2025.11.2+

---

**Quick Command Reference:**

```bash
# Run all tests
./tests/e2e-agent-verification.sh

# View results
cat /tmp/e2e-test-results-*.json | jq .

# Check server
lsof -i :3210

# Manual API test
curl -s http://127.0.0.1:3210/api/agents | jq '.agents | length'
```
