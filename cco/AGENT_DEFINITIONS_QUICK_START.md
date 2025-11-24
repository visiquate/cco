# Agent Definitions System - Quick Start Guide

**Version**: 2.0.0
**Last Updated**: November 15, 2025
**For**: Developers and operators

## One-Minute Overview

The Agent Definitions System allows Claude Code to dynamically discover agent configurations from CCO via HTTP APIs, instead of hardcoding them.

```
Agent YAML Files (~/.claude/agents/*.md)
    ↓
orchestration Config (config/orchestra-config.json)
    ↓
Build Time: build.rs embeds definitions in binary
    ↓
CCO Binary (all 119 agents included)
    ↓
Runtime: HTTP API serves definitions
    ↓
Claude Code: Fetches from /api/agents endpoint
```

## For End Users

### Start CCO Server

```bash
# On your development machine
./target/release/cco-proxy --port 8000
```

### Set Environment Variable

```bash
export CCO_API_URL="http://localhost:8000"
```

### Verify It Works

```bash
# Check health
curl http://localhost:8000/health

# Get all agents
curl http://localhost:8000/api/agents | jq '.agents | length'
# Should return: 119

# Get specific agent
curl http://localhost:8000/api/agents/python-specialist | jq '.agent.name'
# Should return: "Python Specialist"
```

## For Developers

### Understanding the Flow

1. **Discovery Phase**: Claude Code reads `CCO_API_URL` environment variable
2. **Query Phase**: Sends HTTP GET to `/api/agents`
3. **Cache Phase**: Caches response locally
4. **Fallback Phase**: Uses cache if CCO unreachable

### Using the API

All endpoints return JSON:

```bash
# List all agents
curl http://localhost:8000/api/agents

# Get one agent
curl http://localhost:8000/api/agents/python-specialist

# Filter by model tier
curl "http://localhost:8000/api/agents?model=sonnet"

# Get agents in category
curl http://localhost:8000/api/agents/category/development
```

### Override Agent Models (Testing)

```bash
# Check current overrides
curl http://localhost:8000/api/models/override

# Apply override (use Sonnet instead of Haiku)
curl -X POST http://localhost:8000/api/models/override \
  -H "Content-Type: application/json" \
  -d '{"python-specialist": "sonnet"}'

# Verify override
curl "http://localhost:8000/api/agents/python-specialist" | jq '.agent.model'
# Should return: "sonnet"

# Clear overrides
curl -X POST http://localhost:8000/api/models/override \
  -H "Content-Type: application/json" \
  -d '{}'
```

## For System Administrators

### Deployment Checklist

- [ ] Install CCO binary: `./cco/target/release/cco-proxy`
- [ ] Choose port (default: 8000)
- [ ] Configure firewall to allow connections
- [ ] Set environment variable: `export CCO_API_URL="http://your-cco-host:8000"`
- [ ] Test health: `curl http://your-cco-host:8000/health`
- [ ] Monitor logs: `tail -f ~/.local/share/cco/logs/cco-8000.log`

### High Availability

Run multiple CCO instances:

```bash
# Instance 1
./cco-proxy --port 8000

# Instance 2
./cco-proxy --port 8001

# Instance 3
./cco-proxy --port 8002
```

Configure load balancer to round-robin across instances.

### Docker Deployment

```bash
# Build image
docker build -t cco:latest ./cco

# Run container
docker run -d \
  -p 8000:8000 \
  --name cco-primary \
  --restart unless-stopped \
  cco:latest
```

## Troubleshooting

### Agent Not Found

```bash
# Check if agent exists
curl -s http://localhost:8000/api/agents | jq '.agents[] | select(.type == "python-specialist")'

# Get specific error
curl http://localhost:8000/api/agents/unknown-agent
# Returns 404 with error details
```

### CCO Not Responding

```bash
# Check if CCO is running
lsof -i :8000

# Check health
curl -v http://localhost:8000/health

# View logs
tail -f ~/.local/share/cco/logs/cco-8000.log
```

### Wrong Model Being Used

```bash
# Check current assignment
curl "http://localhost:8000/api/agents/python-specialist" | jq '.agent.model'

# Check if overridden
curl http://localhost:8000/api/models/override | jq '.overrides."python-specialist"'

# Override if needed
curl -X POST http://localhost:8000/api/models/override \
  -H "Content-Type: application/json" \
  -d '{"python-specialist": "sonnet"}'
```

## Common Curl Commands

```bash
# Get all agents (pretty printed)
curl http://localhost:8000/api/agents | jq .

# Count agents
curl -s http://localhost:8000/api/agents | jq '.agents | length'

# List agent names
curl -s http://localhost:8000/api/agents | jq '.agents[].name'

# Get agent with specific model
curl -s http://localhost:8000/api/agents | jq '.agents[] | select(.model == "opus")'

# Check health with timestamp
curl -s http://localhost:8000/health | jq '{status, version, uptime: .cache_stats}'

# Get agent capabilities
curl -s http://localhost:8000/api/agents/python-specialist | jq '.agent.capabilities'

# List all models used
curl -s http://localhost:8000/api/agents | jq '[.agents[].model] | unique'

# Count agents by model
curl -s http://localhost:8000/api/agents | jq 'group_by(.model) | map({model: .[0].model, count: length})'
```

## API Endpoint Quick Reference

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/health` | Server health check |
| GET | `/api/agents` | List all agents |
| GET | `/api/agents/{type}` | Get specific agent |
| GET | `/api/agents/category/{name}` | Get agents by category |
| GET | `/api/agents/model/{name}` | Get agents by model |
| GET | `/api/models/override` | Get current overrides |
| POST | `/api/models/override` | Set model overrides |

## Configuration

### Environment Variables

```bash
# Tell Claude Code where to find CCO (required)
export CCO_API_URL="http://localhost:8000"

# Or for remote CCO
export CCO_API_URL="http://cc-orchestrator.example.com:8000"
```

### CCO Command-Line

```bash
# Default port 8000
./cco-proxy

# Custom port
./cco-proxy --port 9000

# Verbose logging
RUST_LOG=debug ./cco-proxy
```

## Performance Expectations

| Operation | Response Time |
|-----------|---------------|
| Health check | <1ms |
| List all agents | <10ms |
| Get single agent | <2ms |
| Model override | <5ms |

## Security Notes

### Current Implementation

- No authentication (open to localhost)
- No rate limiting
- All connections in plain HTTP
- Definitions are read-only (except overrides)

### Recommendations

For production:
- Use HTTPS/TLS
- Implement API key authentication
- Add rate limiting
- Restrict to internal network
- Monitor all API access

## Next Steps

1. **Read**: `/Users/brent/.claude/AGENT_DEFINITIONS_ARCHITECTURE.md` for system overview
2. **Reference**: `/Users/brent/git/cc-orchestra/cco/AGENT_DEFINITIONS_API.md` for API details
3. **Implement**: `/Users/brent/git/cc-orchestra/cco/IMPLEMENTATION_CHECKLIST.md` for build tasks

## Getting Help

- **Architecture questions**: See AGENT_DEFINITIONS_ARCHITECTURE.md
- **API questions**: See AGENT_DEFINITIONS_API.md
- **Implementation questions**: See IMPLEMENTATION_CHECKLIST.md
- **Troubleshooting**: See Architecture doc "Troubleshooting" section

## Key Takeaways

1. CCO binary contains all 119 agent definitions (embedded at build time)
2. HTTP API serves definitions to Claude Code at runtime
3. Environment variable `CCO_API_URL` configures the endpoint
4. Falls back to cache/defaults if CCO unavailable
5. Model overrides allow testing with different model tiers

---

**Ready to start?**

1. Start CCO: `./cco-proxy --port 8000`
2. Check health: `curl http://localhost:8000/health`
3. Explore: `curl http://localhost:8000/api/agents | jq .`
