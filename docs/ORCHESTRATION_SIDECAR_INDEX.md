# Orchestration Sidecar Documentation Index

**Version**: 1.0.0
**Date**: November 2025

## Overview

Complete documentation for the Claude Orchestra orchestration sidecar system - the coordination layer that enables 119 agents to work together autonomously.

---

## Getting Started

### New Users Start Here

1. **[Quick Start Guide](ORCHESTRATION_SIDECAR_QUICKSTART.md)** (5 pages)
   - What is the orchestration sidecar
   - Why agents need it
   - How to launch it
   - First agent example
   - Troubleshooting basics

2. **[FAQ](ORCHESTRATION_SIDECAR_FAQ.md)** (10 pages)
   - Common questions answered
   - Quick solutions to common problems
   - Comparison with alternatives

### For Developers

3. **[Agent Integration Guide](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md)** (12 pages)
   - How agents use the sidecar
   - Getting context for your agent
   - Storing results
   - Publishing events
   - Subscribing to events
   - Error handling
   - Code examples in Python and Rust

4. **[API Reference](ORCHESTRATION_SIDECAR_API_REFERENCE.md)** (15 pages)
   - Complete HTTP API documentation
   - All 8 endpoints with examples
   - Request/response schemas
   - Authentication and authorization
   - Rate limiting
   - Error codes
   - Examples in cURL, Python, and Rust

### For Operators

5. **[CLI Reference](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md)** (8 pages)
   - Server commands (`cco orchestration-server`)
   - Context commands (`cco context`)
   - Results commands (`cco results`)
   - Event commands (`cco events`)
   - Agent commands (`cco agent`)
   - Configuration options
   - Environment variables

6. **[Troubleshooting Guide](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md)** (8 pages)
   - Quick diagnostics
   - Server issues
   - Authentication issues
   - Context issues
   - Event issues
   - Performance issues
   - Storage issues
   - Network issues
   - Common errors
   - Debug mode

### Advanced Topics

7. **[Event System Guide](ORCHESTRATION_SIDECAR_EVENTS.md)** (10 pages)
   - Event topics and taxonomy
   - Event types (architecture, implementation, testing, etc.)
   - Publishing events
   - Subscribing to events (long polling)
   - Event filtering
   - Multi-round workflows
   - Event patterns (request-response, broadcast, fan-out)
   - Best practices

8. **[Architecture Document](ORCHESTRATION_SIDECAR_ARCHITECTURE.md)** (20 pages)
   - System architecture overview
   - Component interactions
   - Data flow diagrams
   - Security model
   - Context injection strategy
   - Event coordination patterns
   - Storage layer design
   - Performance characteristics
   - Scaling considerations

---

## Documentation by Topic

### Architecture & Design

- **[System Architecture](ORCHESTRATION_SIDECAR_ARCHITECTURE.md#system-architecture)**
  - High-level component diagram
  - Component details (Server, Broker, Event Bus, Storage, Context Injector)
  - TDD-aware coordination flow

- **[Security Model](ORCHESTRATION_SIDECAR_ARCHITECTURE.md#security-model)**
  - Agent authentication (JWT)
  - Authorization rules
  - Project isolation
  - Threat model and mitigations

- **[Storage Design](ORCHESTRATION_SIDECAR_ARCHITECTURE.md#database-schema)**
  - Events table (in-memory)
  - Results storage (JSON files)
  - Context cache schema

### API Documentation

- **[Context API](ORCHESTRATION_SIDECAR_API_REFERENCE.md#get-context)**
  - `GET /api/context/:issue_id/:agent_type`
  - Request/response schemas
  - Context structure
  - Examples

- **[Results API](ORCHESTRATION_SIDECAR_API_REFERENCE.md#post-results)**
  - `POST /api/results`
  - Result schema
  - Next agent suggestions
  - Examples

- **[Events API](ORCHESTRATION_SIDECAR_API_REFERENCE.md#post-events)**
  - `POST /api/events/:event_type`
  - `GET /api/events/wait/:event_type`
  - Event schema
  - Long polling
  - Examples

- **[Health & Status](ORCHESTRATION_SIDECAR_API_REFERENCE.md#get-health)**
  - `GET /health`
  - `GET /status`
  - System metrics

### Agent Development

- **[Agent Lifecycle](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md#agent-lifecycle)**
  - Agent spawn
  - Initialization
  - Get context
  - Execute work
  - Store results
  - Publish completion

- **[Context Usage](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md#getting-context)**
  - What context contains
  - Context by agent type
  - Requesting specific context
  - Handling large context

- **[Event Coordination](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md#publishing-events)**
  - Publishing events
  - Subscribing to events
  - Event filtering
  - Background subscriptions

- **[Error Handling](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md#error-handling)**
  - Authentication errors
  - Rate limiting
  - Network errors
  - Sidecar unavailable

- **[Code Examples](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md#code-examples)**
  - Complete Python agent
  - Complete Rust agent

### Event System

- **[Event Topics](ORCHESTRATION_SIDECAR_EVENTS.md#event-topics)**
  - Standard topics (architecture, implementation, testing, etc.)
  - Topic subscription rules
  - Custom topics

- **[Event Types](ORCHESTRATION_SIDECAR_EVENTS.md#event-types)**
  - Architecture events
  - Implementation events
  - Testing events
  - Security events
  - Deployment events
  - Error events
  - Coordination events

- **[Event Patterns](ORCHESTRATION_SIDECAR_EVENTS.md#event-patterns)**
  - Request-response pattern
  - Broadcast pattern
  - Fan-out pattern
  - Circuit breaker pattern

- **[Multi-Round Workflows](ORCHESTRATION_SIDECAR_EVENTS.md#multi-round-workflows)**
  - Feedback loop pattern
  - Phased workflow pattern
  - Dependency chain pattern

### Operations

- **[Server Management](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md#server-commands)**
  - Starting the server
  - Stopping the server
  - Restarting the server
  - Checking status

- **[Context Management](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md#context-commands)**
  - Getting context
  - Refreshing context
  - Clearing context cache

- **[Results Management](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md#results-commands)**
  - Storing results
  - Listing results
  - Getting specific results

- **[Event Management](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md#event-commands)**
  - Publishing events
  - Subscribing to events
  - Listing event history

- **[Agent Management](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md#agent-commands)**
  - Spawning agents
  - Listing active agents
  - Killing agents

### Troubleshooting

- **[Quick Diagnostics](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md#quick-diagnostics)**
  - Health check
  - Status check
  - Log check
  - Process check

- **[Server Issues](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md#server-issues)**
  - Sidecar won't start
  - Sidecar crashes frequently
  - Graceful shutdown fails

- **[Authentication Issues](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md#authentication-issues)**
  - JWT token rejected
  - Permission denied

- **[Context Issues](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md#context-issues)**
  - Context not loading
  - Context is truncated
  - Stale context

- **[Event Issues](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md#event-issues)**
  - Events not delivered
  - Event queue full
  - Long polling timeout

- **[Performance Issues](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md#performance-issues)**
  - Slow response times
  - High memory usage
  - High CPU usage

- **[Storage Issues](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md#storage-issues)**
  - Storage full
  - Storage corruption

- **[Network Issues](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md#network-issues)**
  - Can't connect to sidecar
  - Remote access fails

---

## Quick Reference

### Common Commands

```bash
# Start sidecar
cco orchestration-server

# Check health
curl http://localhost:3001/health

# Get context
cco context get issue-123 python-specialist

# List results
cco results list --issue issue-123

# Subscribe to events
cco events subscribe agent_completed --continuous

# Spawn agent
cco agent spawn --type python-specialist --issue issue-123
```

### Key Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check (no auth) |
| `/status` | GET | System status (auth required) |
| `/api/context/:issue/:type` | GET | Get context for agent |
| `/api/results` | POST | Store agent results |
| `/api/events/:type` | POST | Publish event |
| `/api/events/wait/:type` | GET | Subscribe to events (long polling) |
| `/api/agents/spawn` | POST | Spawn new agent |
| `/api/cache/context/:issue` | DELETE | Clear context cache |

### Configuration

```bash
# Environment variables
export CCO_SIDECAR_PORT=3001
export CCO_SIDECAR_HOST=127.0.0.1
export CCO_SIDECAR_STORAGE=/tmp/cco-sidecar
export CCO_SIDECAR_CACHE_SIZE_MB=1024
export CCO_SIDECAR_LOG_LEVEL=info

# Config file: ~/.cco/config.json
{
  "sidecar": {
    "port": 3001,
    "host": "127.0.0.1",
    "storage_path": "/tmp/cco-sidecar",
    "cache_size_mb": 1024
  }
}
```

### Default Ports

| Service | Port | Purpose |
|---------|------|---------|
| CCO Daemon | 3000 | API cost monitoring, TUI |
| Orchestration Sidecar | 3001 | Agent coordination |

### Storage Locations

```
/tmp/cco-sidecar/
├── results/                  # Agent results (JSON)
│   └── <project>/
│       └── <issue>/
│           └── <agent>.json
├── context-cache/            # Context cache (LRU)
└── events/                   # Event log (circular)
    └── event-log.jsonl
```

### Authentication

```bash
# JWT token structure
{
  "sub": "agent-uuid",
  "agent_type": "python-specialist",
  "project_id": "project-abc",
  "permissions": ["read_context", "write_results", "publish_events"],
  "exp": 1700000000,
  "iat": 1699990000
}

# Usage
curl -H "Authorization: Bearer $JWT_TOKEN" \
  http://localhost:3001/api/context/issue-123/python-specialist
```

---

## Navigation Map

```
┌────────────────────────────────────────────────────────────┐
│                    DOCUMENTATION MAP                       │
└────────────────────────────────────────────────────────────┘

Getting Started
├── Quick Start Guide         → Launch & first example
├── FAQ                       → Common questions
└── Troubleshooting          → Fix common issues

Developer Documentation
├── Agent Integration Guide   → Build agents
├── API Reference            → HTTP API docs
├── Event System Guide       → Coordinate with events
└── Code Examples            → Python & Rust

Operator Documentation
├── CLI Reference            → Command-line tools
├── Configuration           → Setup & tuning
├── Troubleshooting         → Diagnose & fix
└── Performance            → Optimize

Architecture Documentation
├── System Architecture      → High-level design
├── Component Details       → Deep dives
├── Security Model          → Authentication & isolation
└── Scaling               → Performance & limits
```

---

## Documentation Status

### Completed

- ✅ Quick Start Guide (5 pages)
- ✅ API Reference (15 pages)
- ✅ Agent Integration Guide (12 pages)
- ✅ CLI Reference (8 pages)
- ✅ Event System Guide (10 pages)
- ✅ Troubleshooting Guide (8 pages)
- ✅ FAQ (10 pages)
- ✅ Architecture Document (20 pages)
- ✅ Documentation Index (this document)

**Total: 88 pages of comprehensive documentation**

### Future Additions

- ⏳ Advanced Deployment Guide
- ⏳ Performance Tuning Guide
- ⏳ Security Hardening Guide
- ⏳ Migration Guide
- ⏳ Video Tutorials
- ⏳ Interactive Examples

---

## Contributing to Documentation

### How to Contribute

1. **Find an issue**:
   - Typos
   - Unclear explanations
   - Missing examples
   - Outdated information

2. **Propose improvement**:
   - Create GitHub issue
   - Describe the problem
   - Suggest solution

3. **Submit changes**:
   - Fork repository
   - Make changes
   - Submit pull request

### Documentation Style Guide

- **Clear**: Use simple language
- **Concise**: No unnecessary words
- **Complete**: Cover all aspects
- **Correct**: Test all examples
- **Consistent**: Follow existing style
- **Code examples**: Always test them
- **Diagrams**: Use ASCII art for compatibility

### Testing Documentation

```bash
# Test all code examples
./scripts/test-docs.sh

# Check for broken links
./scripts/check-links.sh

# Validate JSON examples
./scripts/validate-json.sh
```

---

## Version History

### v1.0.0 (November 2025)

- Initial documentation release
- 9 comprehensive guides (88 pages)
- Complete API reference
- Code examples in Python and Rust
- Troubleshooting guide
- FAQ with 50+ questions

### Planned Updates

- v1.1.0 (Q1 2026): Performance tuning guide
- v1.2.0 (Q2 2026): Advanced deployment guide
- v1.3.0 (Q3 2026): Video tutorials
- v2.0.0 (Q4 2026): Multi-sidecar architecture

---

## Getting Help

### Documentation Issues

If you find issues with the documentation:

1. **Typos/errors**: Create GitHub issue
2. **Unclear sections**: Ask on Discord
3. **Missing content**: Request on GitHub
4. **Code examples**: Report bugs on GitHub

### Technical Support

For technical issues with the sidecar:

1. **Check**: [Troubleshooting Guide](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md)
2. **Search**: GitHub issues
3. **Ask**: Discord community
4. **Report**: GitHub issue (with logs)

### Contact

- **GitHub**: [Repository Issues](https://github.com/visiquate/cco/issues)
- **Discord**: [Community Server](https://discord.gg/...) (coming soon)
- **Email**: support@visiquate.com

---

## License

This documentation is licensed under MIT License.

Copyright (c) 2025 VisiQuate

Permission is hereby granted, free of charge, to any person obtaining a copy
of this documentation and associated documentation files, to deal in the
documentation without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the documentation, and to permit persons to whom the documentation is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the documentation.

---

**Last Updated**: November 18, 2025
**Documentation Version**: 1.0.0
**Sidecar Version**: 1.0.0
