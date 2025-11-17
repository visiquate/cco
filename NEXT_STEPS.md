# Next Steps - Agent Definitions System

**Project**: Claude Code Orchestrator (CCO)
**System**: Agent Definitions System
**Status**: ✅ Verified and Ready for Deployment
**Date**: November 15, 2025

---

## Immediate Actions (Ready Now)

### 1. Start Using the System

The system is fully operational and can be used immediately:

```bash
# Ensure CCO is running
$ cco run --port 3000

# Optional: Set environment variable for HTTP access
$ export CCO_API_URL=http://localhost:3000

# Test it works
$ curl http://localhost:3000/api/agents | jq '.agents | length'
117
```

Claude Code can now query agent configurations dynamically!

---

### 2. Update Documentation References

Update any project documentation that references hardcoded agent models to point to the new dynamic system:

**Before**:
```javascript
// Hardcoded model
Task("Rust Agent", "...", "rust-specialist", "haiku");
```

**After**:
```javascript
// Dynamic model from agent definition
const model = getAgentModel('rust-specialist');
Task("Rust Agent", "...", "rust-specialist", model);
```

---

## Short-Term Enhancements (Next 1-2 Weeks)

### 1. Enhanced HTTP Integration for agent-loader.js

Currently agent-loader.js only reads from the filesystem. Add HTTP support:

**File**: `/Users/brent/.claude/agent-loader.js`

```javascript
/**
 * Get the configured model for an agent via HTTP API (with fallback)
 */
async function getAgentModelHTTP(agentName) {
  const apiUrl = process.env.CCO_API_URL || 'http://localhost:3000';

  try {
    const response = await fetch(`${apiUrl}/api/agents/${agentName}`);
    if (response.ok) {
      const data = await response.json();
      console.log(`✅ Agent ${agentName} model via HTTP: ${data.model}`);
      return data.model;
    }
  } catch (error) {
    console.warn(`⚠️  HTTP API unavailable, falling back to filesystem`);
  }

  // Fallback to filesystem
  return getAgentModel(agentName);
}
```

**Benefits**:
- Faster queries (no file I/O)
- Centralized agent management
- Still works if CCO is down (fallback)

---

### 2. Add Agent Caching

Implement simple in-memory caching to reduce API calls:

**File**: `/Users/brent/.claude/agent-loader.js`

```javascript
const agentCache = new Map();
const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

function getCachedAgentModel(agentName) {
  const cached = agentCache.get(agentName);
  if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
    return cached.model;
  }

  // Fetch fresh data
  const model = getAgentModel(agentName);

  // Cache it
  agentCache.set(agentName, {
    model,
    timestamp: Date.now()
  });

  return model;
}
```

**Benefits**:
- Reduces file system access
- Improves performance
- Still refreshes periodically

---

### 3. Add Model Override Testing Support

Allow temporary model overrides for testing:

**File**: Add to `server.rs`

```rust
// POST /api/models/override
// Body: {"rust-specialist": "sonnet"}

async fn set_model_override(
    State(state): State<Arc<ServerState>>,
    Json(overrides): Json<HashMap<String, String>>,
) -> StatusCode {
    // Temporarily override models for testing
    // (in-memory only, not persisted)
    for (agent, model) in overrides {
        state.model_overrides.insert(agent, model);
    }
    StatusCode::OK
}
```

**Use Case**:
```bash
# Test a Haiku agent with Sonnet temporarily
curl -X POST http://localhost:3000/api/models/override \
  -d '{"rust-specialist": "sonnet"}'

# Now rust-specialist uses sonnet for this session
```

---

## Medium-Term Improvements (Next 1-2 Months)

### 1. Agent Versioning

Add version field to agent definitions:

**File**: `~/.claude/agents/rust-specialist.md`

```yaml
---
name: rust-specialist
model: haiku
version: 1.0.0
description: Rust development specialist...
tools: Read, Write, Edit, Bash
---
```

**Benefits**:
- Track agent definition changes
- Support multiple versions simultaneously
- Enable gradual rollouts

---

### 2. Dynamic Agent Registration

Allow runtime agent registration without restart:

**API Endpoint**: `POST /api/agents/register`

```bash
curl -X POST http://localhost:3000/api/agents/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-agent",
    "model": "haiku",
    "description": "A new agent",
    "tools": ["Read", "Write"]
  }'
```

**Benefits**:
- Add agents without restarting CCO
- Faster development iteration
- Dynamic agent discovery

---

### 3. Hot Reload on File Changes

Watch `~/.claude/agents/` for changes and reload automatically:

**Implementation**: Use `notify` crate in Rust

```rust
use notify::{Watcher, RecursiveMode, watcher};

fn watch_agents_dir() {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    watcher.watch("~/.claude/agents/", RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(event) => {
                info!("Agent file changed, reloading...");
                reload_agents();
            }
            Err(e) => error!("Watch error: {}", e),
        }
    }
}
```

**Benefits**:
- Changes take effect immediately
- No manual restarts needed
- Better developer experience

---

### 4. Metrics and Monitoring

Add Prometheus metrics for agent usage:

**Metrics to Track**:
- `agent_requests_total{agent="rust-specialist"}`
- `agent_model_usage{model="haiku"}`
- `agent_api_response_time`
- `agent_cache_hit_rate`

**Implementation**:
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref AGENT_REQUESTS: Counter = register_counter!(
        "agent_requests_total",
        "Total number of agent requests"
    ).unwrap();
}
```

**Benefits**:
- Monitor agent usage patterns
- Identify performance bottlenecks
- Track cost optimization success

---

## Long-Term Enhancements (3+ Months)

### 1. Agent Discovery Service

Create a centralized agent registry that multiple CCO instances can share:

**Architecture**:
```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ CCO Instance│────▶│   Agent     │◀────│ CCO Instance│
│     #1      │     │  Registry   │     │     #2      │
└─────────────┘     │  (Redis/DB) │     └─────────────┘
                    └─────────────┘
```

**Benefits**:
- Share agents across teams
- Centralized management
- Version control integration

---

### 2. Agent Analytics Dashboard

Build a web dashboard to visualize agent usage:

**Features**:
- Real-time agent usage graphs
- Model distribution pie chart
- Cost tracking per agent
- Performance metrics
- Popular agents ranking

**Tech Stack**: React + Chart.js + WebSocket for real-time updates

---

### 3. Agent Templates and Generation

Create a tool to generate new agent definitions:

```bash
$ cco agent new \
  --name database-specialist \
  --model haiku \
  --tools "Read,Write,Bash" \
  --description "Database optimization specialist"

✅ Created: ~/.claude/agents/database-specialist.md
✅ Registered with CCO
✅ Ready to use!
```

**Benefits**:
- Faster agent creation
- Consistent structure
- Reduced errors

---

### 4. Integration with CI/CD

Add GitHub Actions to validate agent definitions:

**File**: `.github/workflows/validate-agents.yml`

```yaml
name: Validate Agent Definitions

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Validate YAML frontmatter
        run: |
          for file in ~/.claude/agents/*.md; do
            # Validate YAML structure
            # Check required fields
            # Verify model is valid
          done

      - name: Check for duplicates
        run: |
          # Ensure no duplicate agent names
          # Check for conflicting definitions

      - name: Test API endpoints
        run: |
          cco run --port 3000 &
          sleep 5
          ./test-agent-system.sh
```

---

## Recommended Priority Order

Based on impact and effort, here's the recommended implementation order:

### High Priority (Do First)
1. ✅ **Use the system** (Ready now!)
2. Enhanced HTTP integration for agent-loader.js
3. Add model override testing support
4. Agent caching

### Medium Priority (Next Quarter)
5. Agent versioning
6. Dynamic agent registration
7. Hot reload on file changes
8. Basic metrics

### Lower Priority (Future)
9. Agent discovery service
10. Analytics dashboard
11. Agent templates
12. CI/CD integration

---

## Migration Guide for Existing Code

If you have existing code with hardcoded models, here's how to migrate:

### Step 1: Identify Hardcoded Models

```bash
# Find all Task() calls with hardcoded models
grep -r 'Task.*".*".*"haiku"' .
grep -r 'Task.*".*".*"sonnet"' .
grep -r 'Task.*".*".*"opus"' .
```

### Step 2: Replace with Dynamic Lookup

**Before**:
```javascript
Task("Python Specialist", "Implement feature X", "python-specialist", "haiku");
```

**After**:
```javascript
const { getAgentModel } = require('~/.claude/agent-loader.js');
const model = getAgentModel('python-specialist');
Task("Python Specialist", "Implement feature X", "python-specialist", model);
```

### Step 3: Test

```bash
# Run your code and verify agents still work
# Check that models are correctly assigned
```

---

## Testing Strategy

### Unit Tests
- Test YAML frontmatter parsing
- Test HTTP endpoints
- Test fallback mechanism
- Test model override logic

### Integration Tests
- Test complete flow (CCO → API → agent-loader → Claude Code)
- Test with CCO unavailable (fallback)
- Test with invalid agent names
- Test performance under load

### End-to-End Tests
- Use the existing `test-agent-system.sh` script
- Run before each deployment
- Include in CI/CD pipeline

---

## Monitoring Recommendations

### What to Monitor

1. **Agent Request Rate**
   - Requests per agent
   - Overall request rate
   - Peak usage times

2. **Model Distribution**
   - Percentage per model tier
   - Cost tracking
   - Model switching frequency

3. **Performance Metrics**
   - API response time (should stay < 10ms)
   - Cache hit rate
   - Error rate

4. **System Health**
   - CCO uptime
   - Agent file count
   - HTTP endpoint availability

### Alerting Thresholds

- API response time > 50ms (warn)
- API response time > 100ms (critical)
- Error rate > 1% (warn)
- CCO downtime (critical)

---

## Support and Troubleshooting

### Common Issues and Solutions

**Issue 1**: "Agent not found" error
```bash
# Solution: Check agent file exists
ls ~/.claude/agents/agent-name.md

# Verify YAML frontmatter is valid
head -20 ~/.claude/agents/agent-name.md
```

**Issue 2**: Wrong model being used
```bash
# Solution: Check agent definition
curl http://localhost:3000/api/agents/agent-name | jq '.model'

# Compare with file
grep "model:" ~/.claude/agents/agent-name.md
```

**Issue 3**: CCO not responding
```bash
# Solution: Restart CCO
killall cco
cco run --port 3000

# Verify it started
curl http://localhost:3000/health
```

**Issue 4**: Fallback not working
```bash
# Solution: Check agent-loader.js
cd ~/.claude
node -e "const { getAgentModel } = require('./agent-loader.js'); console.log(getAgentModel('test-agent'));"
```

---

## Documentation Updates Needed

1. Update main README.md to mention dynamic agent system
2. Add section to ORCHESTRATOR_RULES.md about agent models
3. Update any existing docs that mention hardcoded models
4. Create quick reference guide for common agent tasks

---

## Questions to Consider

1. **Should we add authentication to the API?**
   - Currently open, which is fine for local use
   - Production deployment might need API keys

2. **Should we support custom agent directories?**
   - Currently hardcoded to `~/.claude/agents/`
   - Could add `--agents-dir` flag

3. **Should we version the API?**
   - Currently no versioning
   - Could add `/v1/api/agents` for future compatibility

4. **Should we add rate limiting?**
   - Not needed for local use
   - Production deployment might need it

---

## Conclusion

The Agent Definitions System is **production-ready** and can be deployed immediately. The recommended next steps are:

1. ✅ **Start using it now** (zero changes needed!)
2. Add HTTP support to agent-loader.js (1-2 days)
3. Add model override for testing (1 day)
4. Add basic caching (1 day)
5. Monitor usage and iterate based on feedback

**Total estimated effort for short-term enhancements**: 1 week

The system is designed to be extended incrementally, so you can add features as needed without disrupting existing functionality.

---

**Ready to deploy? The system is waiting for you!** ✅

For questions or issues, refer to:
- `/Users/brent/git/cc-orchestra/VERIFICATION_REPORT.md` (detailed test results)
- `/Users/brent/git/cc-orchestra/cco/test-agent-system.sh` (verification script)
- `/Users/brent/git/cc-orchestra/AGENT_DEFINITIONS_DOCUMENTATION.md` (architecture docs)
