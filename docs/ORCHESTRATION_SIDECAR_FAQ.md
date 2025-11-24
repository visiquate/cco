# Orchestration Sidecar FAQ

**Version**: 1.0.0
**Date**: November 2025
**Audience**: All users

## General Questions

### What is the orchestration sidecar?

The orchestration sidecar is an HTTP server that coordinates the 119 agents in the Claude Orchestra system. It runs on port 3001 and provides:
- **Context injection** - Delivers relevant project information to agents
- **Event coordination** - Enables agent-to-agent communication
- **Result storage** - Persists agent outputs

Think of it as the nervous system that connects all agents.

### Why do I need it?

Without the sidecar, agents would:
- Work in isolation without knowing what others have done
- Duplicate effort by redoing work
- Require manual coordination from you
- Lack relevant context about the project

With the sidecar, agents work autonomously for hours without your intervention.

### How is it different from the CCO daemon?

| Feature | CCO Daemon | Orchestration Sidecar |
|---------|-----------|----------------------|
| Port | 3000 | 3001 |
| Purpose | API cost monitoring, TUI | Agent coordination |
| Users | Humans | Agents |
| API | Session/agent management | Context/events/results |

They work together: the daemon manages sessions, the sidecar coordinates agents within those sessions.

### Is it required?

For autonomous multi-agent operation, yes. Without it:
- You can still use Claude Code manually
- You can't spawn multiple coordinating agents
- No context sharing between agents
- No event-driven workflows

### How much does it cost?

The sidecar itself is free and open-source. It only uses local resources (CPU, memory, disk). Agent API calls to Claude (via ccproxy) are the only cost.

## Installation & Setup

### How do I install it?

The sidecar is included in the CCO binary:

```bash
# Install CCO (includes sidecar)
curl -fsSL https://raw.githubusercontent.com/visiquate/cco/main/install.sh | sh

# Verify
cco orchestration-server --version
```

### How do I start it?

Two ways:

**Standalone:**
```bash
cco orchestration-server
```

**With daemon (recommended):**
```bash
cco daemon start  # Starts both daemon and sidecar
```

### What port does it use?

Default: **3001**

Change it:
```bash
cco orchestration-server --port 4000
```

### Can I run it on a remote server?

Yes, but you'll need to:

1. Bind to all interfaces:
   ```bash
   cco orchestration-server --host 0.0.0.0
   ```

2. Configure firewall:
   ```bash
   sudo ufw allow 3001
   ```

3. Use HTTPS (recommended for production):
   ```bash
   # Not yet implemented - use reverse proxy:
   nginx -> https://domain.com -> http://localhost:3001
   ```

### Does it work with Docker?

Yes:

```bash
docker run -p 3001:3001 \
  -v $(pwd):/workspace \
  ghcr.io/visiquate/cco:latest \
  cco orchestration-server
```

## Usage

### How do agents connect to it?

Agents automatically connect when spawned:

```bash
# Spawn agent
cco agent spawn --type python-specialist --issue issue-123

# Agent receives:
# - SIDECAR_URL=http://localhost:3001/api
# - JWT_TOKEN=<token>
# - Agent connects automatically
```

### Do I need to manage JWT tokens?

No. The sidecar:
- Generates tokens when spawning agents
- Injects tokens into agent environment
- Auto-refreshes tokens before expiry
- Validates tokens on every request

You only need tokens if manually calling the API (not recommended).

### How long do agents run?

Agents run until:
- Task complete (normal exit)
- Error encountered (failure exit)
- Manually killed
- Token expires (1 hour, auto-refreshed)

The sidecar supports **4-8 hour** autonomous operation through token refresh and compaction resilience.

### Can I monitor agent progress?

Yes:

```bash
# List active agents
cco agent list

# View results
cco results list --issue issue-123

# Subscribe to events
cco events subscribe agent_completed --continuous
```

### How do I stop an agent?

```bash
# Graceful shutdown
cco agent kill <agent-id>

# Force kill
cco agent kill <agent-id> --force
```

## Context System

### What is context?

Context is the project information given to each agent:
- Project structure (files/directories)
- Relevant source files
- Git history
- Previous agent results
- Project metadata

### How is context gathered?

Automatically when agents request it:

1. Agent calls: `GET /api/context/issue-123/python-specialist`
2. Sidecar gathers:
   - All `.py` files (for Python specialist)
   - Test files
   - Dependencies (`requirements.txt`)
   - Previous architectural decisions
   - Git commits
3. Sidecar caches context
4. Sidecar returns context to agent

### Why is my context truncated?

Context has a **10MB limit** to avoid overwhelming agents. When exceeded, the sidecar:
- Keeps highest priority files
- Truncates or summarizes large files
- Sets `truncated: true` in response

This is automatic and intelligent.

### How do I refresh context?

```bash
# Force refresh
cco context refresh --issue issue-123 --force

# Clear cache
cco context clear --issue issue-123
```

### Can I see the context?

Yes:

```bash
# View context for an agent
cco context get issue-123 python-specialist

# Save to file
cco context get issue-123 python-specialist --output context.json
```

## Event System

### What are events?

Events are messages agents send to coordinate:
- "Architecture defined" → Coding agents start
- "Implementation complete" → Test engineer starts
- "Tests passing" → Security auditor starts

### How do agents receive events?

Via **long polling**:

```python
# Agent subscribes
events = wait_for_event("implementation_complete", timeout=30000)

# Waits up to 30 seconds for event
# Returns immediately when event arrives
```

### What happens if an event is missed?

Events are retained for **24 hours** in a circular buffer (10,000 events). Agents can:
- Retrieve missed events
- Filter by timestamp
- Use correlation IDs to find related events

### Can I see event history?

Yes:

```bash
# List recent events
cco events list --limit 50

# Filter by topic
cco events list --topic implementation

# Filter by type
cco events list --type agent_completed
```

### How do I publish custom events?

```bash
# Via CLI
cco events publish \
  --type custom_event \
  --topic coordination \
  --data '{"message": "checkpoint reached"}'

# Via API (from agent code)
publish_event("custom_event", "coordination", {"message": "checkpoint"})
```

## Storage & Data

### Where is data stored?

Default: `/tmp/cco-sidecar/`

```
/tmp/cco-sidecar/
├── results/           # Agent results (JSON files)
├── context-cache/     # Context cache (in-memory + disk)
└── events/            # Event log (circular buffer)
```

Change it:
```bash
cco orchestration-server --storage /custom/path
```

### Is data persistent?

- **Results**: Yes (JSON files on disk)
- **Context cache**: No (in-memory, lost on restart)
- **Events**: Partial (circular buffer, 24h retention)

For production, use persistent storage:
```bash
cco orchestration-server --storage /var/lib/cco-sidecar
```

### How much disk space is needed?

Typical usage:
- **Small project** (<10 agents): ~50-100 MB
- **Medium project** (10-50 agents): ~100-500 MB
- **Large project** (50+ agents): ~500-2000 MB

Context cache is limited to 1GB by default.

### Can I backup data?

Yes:

```bash
# Backup
tar -czf sidecar-backup.tar.gz /tmp/cco-sidecar/

# Restore
tar -xzf sidecar-backup.tar.gz -C /tmp/
```

## Performance

### How many agents can it handle?

**Target: 119 concurrent agents** (the full orchestra)

Tested configurations:
- 10 agents: Smooth performance
- 50 agents: Good performance
- 119 agents: Designed for, not yet stress-tested

### What are the resource requirements?

**Minimum:**
- CPU: 2 cores
- RAM: 2 GB
- Disk: 5 GB free

**Recommended:**
- CPU: 4+ cores
- RAM: 4+ GB
- Disk: 20+ GB free
- SSD storage

### Why is it slow?

Common causes:
1. **Large context** - Projects with 1000+ files
2. **High agent count** - 50+ concurrent agents
3. **Slow disk** - Use SSD instead of HDD
4. **Low memory** - Increase cache size
5. **High event volume** - Queue is full (>9000)

Solutions:
```bash
# Increase cache
cco orchestration-server --cache-size-mb 2048

# Clear cache
curl -X DELETE http://localhost:3001/api/cache/all

# Check performance
curl http://localhost:3001/status | jq '.performance'
```

### How do I optimize performance?

1. **Use SSD storage**
2. **Increase cache size** (if you have RAM)
3. **Clear old results** regularly
4. **Limit concurrent agents** to system capacity
5. **Use filters** when subscribing to events
6. **Monitor** performance metrics

## Security

### Is the sidecar secure?

Security features:
- **JWT authentication** - All API requests require valid tokens
- **Project isolation** - Agents can't access other projects
- **Rate limiting** - Prevents DoS attacks
- **Input validation** - Sanitizes all inputs
- **Secure defaults** - Binds to localhost only

### Should I expose it to the internet?

**Not directly.** For remote access:
- Use SSH tunnel
- Use reverse proxy with TLS
- Use VPN
- Implement additional authentication

### How are JWT tokens secured?

- **RSA-256 signing** - Cryptographically secure
- **Short expiry** - 1 hour (auto-refreshed)
- **Unique per agent** - Can't share tokens
- **Permission-based** - Different permissions per agent type

### Can agents access each other's data?

No. Each agent can only:
- Read context for their assigned issue
- Write results for their assigned issue
- Publish events (visible to all)
- Subscribe to events (visible to all)

Project-level isolation prevents cross-project access.

### What about secrets in context?

**Warning:** Context includes file contents. Don't store secrets in:
- Source code
- Configuration files
- Environment files (`.env`)

Use the **credential manager** instead:
```bash
cco credentials store API_KEY "secret-value"
```

## Troubleshooting

### Sidecar won't start

```bash
# Check if port is in use
lsof -i :3001

# Kill existing process
pkill -f orchestration-server

# Use different port
cco orchestration-server --port 4000
```

### Agents can't connect

```bash
# Check sidecar is running
curl http://localhost:3001/health

# Check firewall
sudo ufw status

# Check agent environment
echo $SIDECAR_URL
echo $JWT_TOKEN
```

### Context is empty

```bash
# Refresh context
cco context refresh --issue issue-123 --force

# Check project directory
echo $PROJECT_ROOT
ls -la $PROJECT_ROOT
```

### Events not delivered

```bash
# Verify event was published
cco events list --type agent_completed

# Check filter syntax
# Correct: "issue_id:issue-123"
# Wrong:   "issue_id==issue-123"

# Test without filter
cco events subscribe agent_completed
```

### High memory usage

```bash
# Check memory
curl http://localhost:3001/health | jq '.checks.memory_usage_mb'

# Clear cache
curl -X DELETE http://localhost:3001/api/cache/all

# Reduce cache size
cco orchestration-server --cache-size-mb 512
```

### More troubleshooting

See: [Troubleshooting Guide](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md)

## Advanced Topics

### Can I run multiple sidecars?

Not yet supported. Future enhancement for:
- Load balancing
- High availability
- Horizontal scaling

### Can I use a different storage backend?

Currently only filesystem is supported. Future enhancements:
- Redis for cache
- PostgreSQL for results
- Kafka for events

### Can I integrate with external systems?

Yes, via:
- **HTTP API** - Call from any language
- **Webhooks** - Not yet implemented
- **Event streaming** - Subscribe to events

### Can I extend the agent types?

Yes! Edit `config/orchestra-config.json`:

```json
{
  "codingAgents": [
    {
      "name": "My Custom Agent",
      "type": "my-custom-agent",
      "model": "sonnet",
      "role": "Does custom work",
      "capabilities": [...]
    }
  ]
}
```

### Can I customize context gathering?

Not yet. Future enhancement for:
- Custom file filters
- Additional context sources
- Context transformation rules

## Comparison with Alternatives

### vs. Manual Coordination

| Feature | Manual | Sidecar |
|---------|--------|---------|
| Agent coordination | You do it | Automatic |
| Context sharing | Copy/paste | Automatic |
| Event handling | Manual messages | Pub-sub |
| Scalability | 1-2 agents | 119 agents |
| Time saved | 0% | 70-80% |

### vs. Other Agent Frameworks

| Feature | Sidecar | AutoGPT | LangChain Agents |
|---------|---------|---------|------------------|
| Agent count | 119 | 1 | Few |
| Coordination | Event-driven | Sequential | Chain-based |
| Context injection | Automatic | Manual | Manual |
| Claude integration | Native | Via API | Via API |
| TDD-aware | Yes | No | No |

### vs. Knowledge Manager

The Knowledge Manager and Sidecar work together:

| System | Purpose | Persistence |
|--------|---------|-------------|
| Knowledge Manager | Long-term memory across compactions | Permanent |
| Sidecar | Short-term coordination within session | 24 hours |

Use both for optimal results.

## Getting Help

### Where can I get support?

1. **Documentation**:
   - [Quick Start](ORCHESTRATION_SIDECAR_QUICKSTART.md)
   - [Troubleshooting](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md)
   - [API Reference](ORCHESTRATION_SIDECAR_API_REFERENCE.md)

2. **GitHub Issues**:
   - Report bugs
   - Request features
   - Ask questions

3. **Discord** (future):
   - Community support
   - Real-time help

### How do I report bugs?

Create a GitHub issue with:
1. **Description** - What went wrong?
2. **Steps to reproduce** - How to trigger it?
3. **Expected behavior** - What should happen?
4. **Actual behavior** - What actually happened?
5. **Logs** - Include relevant log output
6. **Environment** - OS, CCO version, etc.

### How do I request features?

Create a GitHub issue with:
1. **Use case** - Why do you need it?
2. **Proposed solution** - How should it work?
3. **Alternatives considered** - Other approaches?
4. **Additional context** - Examples, mockups, etc.

### Where is the source code?

GitHub: `https://github.com/visiquate/cco` (replace with actual URL)

The sidecar code is in:
- `cco/src/orchestration_server.rs` - HTTP server
- `cco/src/orchestration/` - Core components

## Future Enhancements

### What's planned?

**Short term (Q1 2026):**
- Webhook support for events
- Custom context filters
- Enhanced event filtering (OR, patterns)
- Metrics export (Prometheus)

**Medium term (Q2-Q3 2026):**
- Multiple sidecar instances (HA)
- Alternative storage backends (Redis, PostgreSQL)
- Event replay and debugging
- Web UI for monitoring

**Long term (Q4 2026+):**
- Distributed tracing
- Agent marketplace
- ML-based agent selection
- Cloud-hosted option

### How can I contribute?

1. **Code contributions**:
   - Fork the repository
   - Create a feature branch
   - Submit a pull request

2. **Documentation**:
   - Fix typos
   - Add examples
   - Improve clarity

3. **Testing**:
   - Report bugs
   - Test new features
   - Provide feedback

4. **Spread the word**:
   - Write blog posts
   - Create tutorials
   - Share on social media

---

## See Also

- [Quick Start Guide](ORCHESTRATION_SIDECAR_QUICKSTART.md)
- [API Reference](ORCHESTRATION_SIDECAR_API_REFERENCE.md)
- [Agent Integration Guide](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md)
- [CLI Reference](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md)
- [Event System](ORCHESTRATION_SIDECAR_EVENTS.md)
- [Troubleshooting](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md)
- [Advanced Topics](ORCHESTRATION_SIDECAR_ADVANCED.md)
- [Documentation Index](ORCHESTRATION_SIDECAR_INDEX.md)
