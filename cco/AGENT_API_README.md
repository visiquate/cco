# Agent Definitions HTTP API - Quick Reference

## Overview

CCO now exposes HTTP endpoints to query agent definitions loaded from `~/.claude/agents/` directory. This enables:
- Dynamic agent discovery
- Dashboard agent listing
- Orchestration system integration
- Agent metadata queries without file system access

## API Endpoints

### GET /api/agents
Lists all available agents with complete configuration.

**Example:**
```bash
curl http://localhost:8000/api/agents | jq . | head -40
```

**Response:**
```json
{
  "agents": [
    {
      "name": "chief-architect",
      "model": "opus",
      "description": "Strategic architecture leadership and orchestra coordination...",
      "tools": ["Read", "Write", "Edit", "TodoWrite", "Bash"]
    },
    {
      "name": "python-specialist",
      "model": "haiku",
      "description": "Python development specialist for FastAPI/Flask...",
      "tools": ["Read", "Write", "Edit", "Bash"]
    },
    ...
  ]
}
```

**Status Codes:**
- `200 OK` - Always returns successfully (even if no agents loaded)

---

### GET /api/agents/{agent-name}
Get specific agent configuration by name.

**Example:**
```bash
curl http://localhost:8000/api/agents/python-specialist | jq .
```

**Success Response (200):**
```json
{
  "name": "python-specialist",
  "model": "haiku",
  "description": "Python development specialist for FastAPI/Flask, Django, data processing, ML/AI integration, and async patterns. Use PROACTIVELY for Python development tasks.",
  "tools": [
    "Read",
    "Write",
    "Edit",
    "Bash"
  ]
}
```

**Error Response (404):**
```json
{
  "error": "Agent not found: nonexistent"
}
```

**Status Codes:**
- `200 OK` - Agent found and returned
- `404 Not Found` - Agent doesn't exist

---

## Implementation Details

### Agent Definition Format

Agent definitions are stored as markdown files in `~/.claude/agents/` with YAML frontmatter:

```markdown
---
name: python-specialist
model: haiku
description: Python development specialist for FastAPI/Flask, Django, data processing, ML/AI integration, and async patterns. Use PROACTIVELY for Python development tasks.
tools: Read, Write, Edit, Bash
---

# Agent details below...
```

### Loading Process

1. **Startup**: CCO reads all `.md` files from `~/.claude/agents/`
2. **Parsing**: YAML frontmatter is extracted from each file
3. **Validation**: Required fields (name, model, description) are checked
4. **Storage**: All agents stored in HashMap for O(1) lookup
5. **Logging**: Agent count logged with any parse errors

**Startup Log Output:**
```
✓ Loaded 117 agents from "/Users/brent/.claude/agents" (0 errors)
→ Agent API: http://127.0.0.1:8000/api/agents
→ Agent Details: http://127.0.0.1:8000/api/agents/:name
```

### Data Structures

**Agent:**
```rust
pub struct Agent {
    pub name: String,           // Agent identifier
    pub model: String,          // LLM model assignment
    pub description: String,    // Purpose and capabilities
    pub tools: Vec<String>,     // Available tools
}
```

**AgentsConfig:**
```rust
pub struct AgentsConfig {
    pub agents: HashMap<String, Agent>,  // name -> Agent
}
```

### Performance

| Metric | Value |
|--------|-------|
| Load time | ~10ms for 117 agents |
| Memory footprint | ~500KB |
| API response time | <5ms (/api/agents), <1ms (/api/agents/:name) |
| Lookup complexity | O(1) HashMap |
| JSON serialization | <1ms |

## Integration Examples

### List All Agents
```bash
# Get agent count
curl -s http://localhost:8000/api/agents | jq '.agents | length'

# Get agent names only
curl -s http://localhost:8000/api/agents | jq '.agents[].name' -r

# Get agents by model
curl -s http://localhost:8000/api/agents | jq '.agents[] | select(.model == "opus")'

# Get agents by tools
curl -s http://localhost:8000/api/agents | jq '.agents[] | select(.tools | contains(["Edit"]))'
```

### Get Agent Details
```bash
# Full agent details
curl -s http://localhost:8000/api/agents/chief-architect | jq .

# Just description
curl -s http://localhost:8000/api/agents/chief-architect | jq '.description'

# Just tools
curl -s http://localhost:8000/api/agents/chief-architect | jq '.tools'

# Check if agent exists
curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/api/agents/fake-agent
# Output: 404
```

### Dashboard Integration
```javascript
// Fetch all agents
fetch('http://localhost:8000/api/agents')
  .then(r => r.json())
  .then(data => {
    console.log(`Loaded ${data.agents.length} agents`);
    data.agents.forEach(agent => {
      console.log(`- ${agent.name} (${agent.model})`);
    });
  });

// Get specific agent
fetch('http://localhost:8000/api/agents/python-specialist')
  .then(r => r.json())
  .then(agent => {
    if (agent.error) {
      console.error('Agent not found');
    } else {
      console.log(`Agent: ${agent.name}`);
      console.log(`Tools: ${agent.tools.join(', ')}`);
    }
  });
```

## Error Handling

### Missing Agents Directory
If `~/.claude/agents/` doesn't exist:
- Logs warning message
- Returns empty agent list
- Server starts normally (no fatal error)

### File Read Errors
If a file can't be read:
- Skipped with warning
- Other agents continue loading
- Logged but doesn't block server startup

### Parse Errors
If YAML frontmatter is invalid:
- Agent skipped with warning
- Other agents continue loading
- No JSON parsing error returned

### Agent Not Found
If querying non-existent agent:
- Returns HTTP 404
- Includes error message in JSON
- Logged for debugging

## Troubleshooting

### No agents returned
1. Check agents directory exists: `ls ~/.claude/agents/`
2. Verify file permissions: `ls -la ~/.claude/agents/*.md | head`
3. Check server logs: `RUST_LOG=info cco run --port 8000`
4. Verify YAML format in agent files

### 404 when querying valid agent
1. Verify agent name (case-sensitive)
2. Check agent file was parsed: Search startup logs for agent name
3. Verify YAML frontmatter format in agent file

### Slow API response
1. This shouldn't happen (typical: <5ms)
2. Check server load: `top` or `Activity Monitor`
3. Check network latency: `curl -w "@curl-format.txt" ...`

## Files

### Core Implementation
- `src/agents_config.rs` - Agent loading and parsing (291 lines)
- `src/server.rs` - HTTP endpoints and integration (967 lines)
- `src/lib.rs` - Module exports

### Test Results
- ✅ 3 unit tests (100% pass rate)
- ✅ 117 agents loaded successfully
- ✅ API endpoints responding correctly
- ✅ Error handling working as expected

## Related Documentation

- `AGENT_API_IMPLEMENTATION.md` - Full implementation details
- Agent definition examples: `~/.claude/agents/*.md`
- Server startup: `/Users/brent/git/cc-orchestra/cco/src/server.rs`
- Agent loading: `/Users/brent/git/cc-orchestra/cco/src/agents_config.rs`

## Version

- Implementation: November 15, 2025
- CCO Version: 2025.11.2
- Agents loaded: 117
- API stability: Production-ready
