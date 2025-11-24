# End-to-End Agent Verification Checklist

## Pre-Test Requirements

- [ ] CCO server compiled successfully (`cargo build --release`)
- [ ] Agent definition files exist in `~/.claude/agents/` directory
- [ ] `agent-loader.js` exists at `~/.claude/agent-loader.js`
- [ ] Node.js installed and available in PATH
- [ ] `jq` command-line JSON processor installed
- [ ] `curl` available for HTTP requests

## Test Execution Checklist

### 1. Test Setup & Infrastructure

- [ ] CCO server starts successfully on configured port
- [ ] `/api/agents` endpoint is accessible
- [ ] All 117-119 agents loaded and available via HTTP
- [ ] Server responds to health check endpoint

### 2. HTTP API Verification

#### List All Agents (GET /api/agents)
- [ ] Returns valid JSON response
- [ ] Contains "agents" array
- [ ] Agent count is 117-119
- [ ] Response time < 100ms

#### Get Individual Agent (GET /api/agents/{agent-name})
- [ ] `chief-architect` returns model: `opus`
- [ ] `rust-specialist` returns model: `haiku`
- [ ] `test-engineer` returns model: `haiku`
- [ ] `security-auditor` returns model: `sonnet`
- [ ] `api-explorer` returns model: `sonnet`
- [ ] `python-specialist` returns model: `haiku`
- [ ] `tdd-coding-agent` returns model: `haiku`
- [ ] `devops-engineer` returns model: `haiku`
- [ ] `documentation-expert` returns model: `haiku`
- [ ] `backend-architect` returns model: `sonnet`

#### Response Structure
- [ ] Contains required field: `name`
- [ ] Contains required field: `model`
- [ ] Contains required field: `description`
- [ ] Contains required field: `tools`
- [ ] Response time < 50ms for individual agent

#### Error Handling
- [ ] Returns 404 for non-existent agent
- [ ] Returns appropriate error message in JSON format

### 3. agent-loader.js Integration

#### API Integration (CCO_API_URL set)
- [ ] Successfully loads `rust-specialist` from API
- [ ] Successfully loads `chief-architect` from API
- [ ] Successfully loads `security-auditor` from API
- [ ] Returns correct model for each agent
- [ ] Logs indicate "from API" source
- [ ] CLI usage works correctly (`node agent-loader.js <agent-name>`)

#### Fallback Mechanism (API unavailable)
- [ ] Falls back to local files when CCO server stopped
- [ ] Falls back to local files when API unreachable
- [ ] Logs indicate "falling back to local files"
- [ ] Still returns correct model from local files
- [ ] All agents accessible via fallback

### 4. Error Cases

#### Network Timeout
- [ ] Handles connection timeout gracefully
- [ ] Falls back within reasonable time (< 5 seconds)
- [ ] Does not hang indefinitely

#### Malformed Responses
- [ ] Handles invalid JSON gracefully
- [ ] Falls back to local files on parse error
- [ ] Logs error appropriately

#### Connection Refused
- [ ] Falls back to local files immediately
- [ ] Provides helpful error message
- [ ] Continues to function

### 5. End-to-End Flow Verification

#### Simulated Claude Agent Spawning
- [ ] `getAgentModel('rust-specialist')` returns `haiku`
- [ ] `getAgentModel('chief-architect')` returns `opus`
- [ ] `getAgentModel('security-auditor')` returns `sonnet`
- [ ] Simulated Task() call would use correct model
- [ ] Flow completes successfully from start to finish

### 6. Model Distribution Validation

- [ ] Exactly 1 agent uses `opus` model (chief-architect)
- [ ] Approximately 37 agents use `sonnet` model
- [ ] Approximately 81 agents use `haiku` model
- [ ] Total agent count matches expected (117-119)

### 7. Performance Verification

- [ ] List all agents: < 100ms response time
- [ ] Get individual agent: < 50ms response time
- [ ] agent-loader execution: < 200ms total
- [ ] Fallback mechanism: < 2s to detect and switch

### 8. Documentation & Reporting

- [ ] Test results JSON file generated
- [ ] Summary includes pass/fail counts
- [ ] Failed tests clearly documented
- [ ] Warnings captured and reported
- [ ] Performance metrics recorded

## Post-Test Verification

- [ ] All tests passed (0 failures)
- [ ] CCO server shut down cleanly
- [ ] No resource leaks (check with `lsof`)
- [ ] Test results file is valid JSON
- [ ] All temporary files cleaned up

## Known Issues / Expected Failures

_Document any known issues or expected failures here_

## Test Execution Instructions

```bash
# 1. Navigate to CCO directory
cd /Users/brent/git/cc-orchestra/cco

# 2. Build CCO server (if not already built)
cargo build --release

# 3. Make test script executable
chmod +x tests/e2e-agent-verification.sh

# 4. Run tests
./tests/e2e-agent-verification.sh

# 5. View results
cat /tmp/e2e-test-results-*.json | jq .
```

## Troubleshooting

### CCO Server Won't Start
```bash
# Check if port is already in use
lsof -i :3210

# Kill existing process if needed
kill $(lsof -t -i :3210)

# Check build errors
cargo build --release 2>&1 | tee build.log
```

### Agent Loader Not Found
```bash
# Verify agent-loader.js exists
ls -la ~/.claude/agent-loader.js

# If missing, check project source
ls -la /Users/brent/git/cc-orchestra/src/agent-loader.js
```

### Missing Dependencies
```bash
# Install jq (macOS)
brew install jq

# Verify Node.js
node --version

# Verify curl
curl --version
```

## Success Criteria

- **All 12 core tests pass** (0 failures)
- **API response times acceptable** (< 100ms list, < 50ms individual)
- **Fallback mechanism works** (falls back within 5s)
- **Correct model distribution** (1 Opus, ~37 Sonnet, ~81 Haiku)
- **End-to-end flow validated** (agent spawning simulation works)

## Sign-off

- [ ] All tests passed
- [ ] Test results reviewed and approved
- [ ] Documentation updated
- [ ] Ready for production use

**Tester:** _______________
**Date:** _______________
**Signature:** _______________
