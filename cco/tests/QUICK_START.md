# Quick Start - E2E Agent Verification Tests

## Prerequisites

```bash
# Install jq (if needed)
brew install jq

# Verify installations
node --version  # Should be v14+
jq --version    # Should be jq-1.6+
curl --version  # Should be available
```

## Run Tests

```bash
# Navigate to CCO directory
cd /Users/brent/git/cc-orchestra/cco

# Build CCO server (if not already built)
cargo build --release

# Run tests
./tests/e2e-agent-verification.sh
```

## View Results

```bash
# View summary
cat /tmp/e2e-test-results-*.json | jq '.summary'

# View all results
cat /tmp/e2e-test-results-*.json | jq .

# Check specific test
cat /tmp/e2e-test-results-*.json | jq '.test_results.API_GET_SPECIFIC_AGENTS'
```

## Expected Output

```
==========================================
  E2E Agent Verification Test Summary
==========================================

Total Tests:    12
Passed:         12
Failed:         0
Warnings:       0

✓ ALL TESTS PASSED
```

## Troubleshooting

```bash
# Port in use?
lsof -i :3210
kill $(lsof -t -i :3210)

# Kill stuck processes
pkill -f "cargo run.*cco"

# Use different port
CCO_PORT=3211 ./tests/e2e-agent-verification.sh
```

## Documentation

- **E2E_TEST_README.md** - Complete usage guide
- **TEST_VERIFICATION_CHECKLIST.md** - Manual verification steps
- **TEST_RESULTS.md** - Results template

## What Gets Tested

1. CCO server startup
2. HTTP API endpoints (GET /api/agents, /api/agents/{name})
3. agent-loader.js integration (API + CLI)
4. Fallback to local files
5. Network timeout handling
6. E2E agent spawning workflow
7. Model distribution (1 Opus, ~37 Sonnet, ~81 Haiku)

## Success Criteria

- All 12 tests pass
- API response times < 100ms
- Correct model assignments verified
- Fallback works within 5s
- E2E workflow validated

---

**Quick Command:**
```bash
./tests/e2e-agent-verification.sh
```
