# Agent Definitions Documentation Index

**Project**: Claude Code Orchestrator (CCO)
**Subsystem**: Agent Definition System
**Version**: 2.0.0
**Last Updated**: November 15, 2025
**Status**: Complete

## Documentation Overview

Comprehensive documentation for the Agent Definitions System - a framework for dynamically discovering and managing 119 specialized agents through HTTP APIs.

## Quick Navigation

### Start Here
- **New to the system?** → Read [Quick Start Guide](#quick-start-guide)
- **Need architecture details?** → Read [Architecture Document](#architecture-document)
- **Building the system?** → Read [Implementation Checklist](#implementation-checklist)
- **Using the API?** → Read [API Reference](#api-reference)

### Documents by Audience

**For Architects & Tech Leads**
- Architecture Document (complete system design)
- Implementation Checklist (phases and timeline)

**For Developers**
- API Reference (all endpoints and examples)
- Quick Start Guide (getting started fast)

**For DevOps & Operations**
- Quick Start Guide (deployment guide)
- Architecture Document (system overview)

**For Project Managers**
- Implementation Checklist (timeline and phases)
- Quick Start Guide (high-level overview)

## Document Index

### 1. Quick Start Guide

**Location**: `/Users/brent/git/cc-orchestra/cco/AGENT_DEFINITIONS_QUICK_START.md`
**Size**: 7.2 KB
**Time to Read**: 10 minutes

**Purpose**: Get up and running quickly with practical examples

**Contents**:
- One-minute system overview
- Common curl commands
- Environment variable setup
- Basic troubleshooting
- Performance expectations
- Security notes
- Quick reference table

**Best For**:
- First-time users
- Quick command lookup
- Basic troubleshooting
- Operator runbooks

**Key Sections**:
- Start CCO server
- Set environment variables
- Verify it works
- Common curl commands
- Deployment checklist

---

### 2. Architecture Document

**Location**: `/Users/brent/.claude/AGENT_DEFINITIONS_ARCHITECTURE.md`
**Size**: 23 KB
**Time to Read**: 30-40 minutes

**Purpose**: Comprehensive system architecture and design

**Contents**:
- System architecture with diagrams
- Data flow (build time → runtime)
- Configuration file specifications
- Agent definition schema
- Integration guide for developers
- Troubleshooting reference
- Performance considerations
- Security guidelines
- Future enhancements

**Best For**:
- Understanding the system design
- Integrating agent definitions
- Troubleshooting complex issues
- Planning enhancements

**Key Sections**:
- System Architecture Diagram
- Data Flow Overview
- Configuration Files
- API Reference Overview
- Integration Guide
- Troubleshooting

---

### 3. API Reference

**Location**: `/Users/brent/git/cc-orchestra/cco/AGENT_DEFINITIONS_API.md`
**Size**: 16 KB
**Time to Read**: 20-30 minutes

**Purpose**: Complete HTTP API endpoint reference

**Contents**:
- 7 endpoints fully documented
- Request/response examples
- Query parameters and filtering
- Pagination support
- Error codes and handling
- Status codes reference
- Best practices for consuming API
- 20+ curl examples
- JavaScript integration patterns

**Best For**:
- API consumers and developers
- Integration with Claude Code
- Building clients
- Testing the system

**Endpoints Documented**:
1. GET /health - Health check
2. GET /api/agents - List all agents
3. GET /api/agents/{name} - Get specific agent
4. GET /api/agents/category/{category} - Get by category
5. GET /api/agents/model/{model} - Get by model tier
6. POST /api/models/override - Set model overrides
7. GET /api/models/override - Get current overrides

**Key Sections**:
- Base URL and common response format
- Each endpoint with examples
- Error response format
- Request/response examples
- Rate limiting information

---

### 4. Implementation Checklist

**Location**: `/Users/brent/git/cc-orchestra/cco/IMPLEMENTATION_CHECKLIST.md`
**Size**: 15 KB
**Time to Read**: 25-35 minutes

**Purpose**: Detailed implementation roadmap and tracking

**Contents**:
- 10 implementation phases
- 50+ detailed checklist items
- Acceptance criteria for each
- 28-day implementation timeline
- Sign-off requirements
- Progress tracking template
- Phase dependencies
- Task prioritization

**Best For**:
- Project managers
- Development teams
- Implementation tracking
- Timeline planning

**Implementation Phases**:
1. API Foundation (5 days)
2. Model Override System (3 days)
3. Error Handling & Validation (2 days)
4. Agent Definition Loading (2 days)
5. Integration & Testing (5 days)
6. Documentation (3 days)
7. Deployment & Operations (2 days)
8. Security (2 days)
9. Maintenance & Support (2 days)
10. Future Enhancements (Backlog)

**Key Sections**:
- Each phase with tasks
- Acceptance criteria
- Testing requirements
- Timeline and dependencies

---

## System Overview

### What is the Agent Definitions System?

The Agent Definitions System enables Claude Code to dynamically discover 119 specialized agents from CCO (Claude Code Orchestrator) via HTTP APIs, instead of hardcoding them.

### Key Flow

```
Agent YAML Files + JSON Config
    ↓
Build Time: Compiled into CCO binary
    ↓
Runtime: HTTP API serves definitions
    ↓
Claude Code: Fetches via /api/agents endpoint
```

### Key Features

- **119 Agents**: Organized by category, model tier
- **HTTP API**: 7 endpoints for discovery and management
- **Dynamic Loading**: No hardcoded agent definitions
- **Model Overrides**: Test with different model tiers
- **Fallback Support**: Works even if CCO is unavailable

### Model Distribution

| Model | Count | Cost | Use Case |
|-------|-------|------|----------|
| Opus | 1 | High | Strategic decisions, architecture |
| Sonnet | 37 | Medium | Code review, complex coding |
| Haiku | 81 | Low | Basic coding, documentation |

## Getting Started

### For New Users

1. **Read**: Quick Start Guide (10 min)
   - Get system overview
   - Learn common commands

2. **Try**: Run CCO server
   ```bash
   ./cco-proxy --port 8000
   ```

3. **Test**: Basic API calls
   ```bash
   curl http://localhost:8000/health
   curl http://localhost:8000/api/agents
   ```

4. **Explore**: Check out specific agents
   ```bash
   curl http://localhost:8000/api/agents/python-specialist
   ```

### For Developers

1. **Read**: Architecture Document (30 min)
   - Understand system design
   - Learn integration points

2. **Read**: API Reference (20 min)
   - All endpoints documented
   - Examples for each endpoint

3. **Integrate**: Start using the API
   ```javascript
   const agents = await fetch(
     `${process.env.CCO_API_URL}/api/agents`
   );
   ```

4. **Deploy**: Follow deployment guide in Quick Start

### For Operations

1. **Read**: Quick Start Guide (10 min)
   - Deployment checklist
   - Common commands

2. **Deploy**: Start CCO server
   - Choose port
   - Configure firewall
   - Set environment variables

3. **Monitor**: Check health
   ```bash
   curl http://your-cco-host:8000/health
   ```

4. **Troubleshoot**: Use troubleshooting guide
   - Agent not found
   - CCO not responding
   - Wrong model being used

## Documentation Quality

All documents include:

- Clear section headings
- Table of contents
- Diagrams and examples
- Cross-references
- Troubleshooting sections
- Best practices
- Real-world examples

## Common Tasks

### Check System Status
```bash
curl http://localhost:8000/health | jq .
```

### List All Agents
```bash
curl http://localhost:8000/api/agents | jq '.agents | length'
```

### Get Specific Agent
```bash
curl http://localhost:8000/api/agents/python-specialist | jq .
```

### Override Agent Model
```bash
curl -X POST http://localhost:8000/api/models/override \
  -d '{"python-specialist": "sonnet"}'
```

### Get Current Overrides
```bash
curl http://localhost:8000/api/models/override | jq .
```

## Support & Help

### For Questions About...

| Topic | See Document |
|-------|--------------|
| System design | Architecture Document |
| API usage | API Reference |
| Getting started | Quick Start Guide |
| Implementation | Implementation Checklist |
| Quick answers | Quick Start Guide |

### Troubleshooting Guide

Each document includes a troubleshooting section:

- **Quick Start**: Basic troubleshooting
- **Architecture**: Detailed troubleshooting
- **API Reference**: Error codes and handling

### Common Issues

| Issue | Solution |
|-------|----------|
| Agent not found | See Architecture troubleshooting |
| CCO not responding | See Quick Start troubleshooting |
| Wrong model | Check model assignment |
| API error | See API Reference error codes |

## Document Relationships

```
QUICK_START (Entry point)
    ↓
    ├─→ ARCHITECTURE (Deep dive)
    ├─→ API_REFERENCE (Details)
    └─→ IMPLEMENTATION (Project tracking)
```

## Project Metadata

| Aspect | Details |
|--------|---------|
| System Name | Agent Definitions System |
| Version | 2.0.0 |
| Created | November 15, 2025 |
| Status | Complete |
| Total Lines | 2,100+ |
| Total Size | 61 KB |
| Endpoints | 7 |
| Agents | 119 |
| Implementation Time | 28 days |

## File Locations

```
~/.claude/
  └─ AGENT_DEFINITIONS_ARCHITECTURE.md (23 KB)

/Users/brent/git/cc-orchestra/cco/
  ├─ AGENT_DEFINITIONS_API.md (16 KB)
  ├─ AGENT_DEFINITIONS_QUICK_START.md (7.2 KB)
  └─ IMPLEMENTATION_CHECKLIST.md (15 KB)

/Users/brent/git/cc-orchestra/
  └─ AGENT_DEFINITIONS_DOCUMENTATION.md (THIS FILE)
```

## Related Project Documents

- `/Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md` - Agent delegation rules
- `/Users/brent/git/cc-orchestra/config/orchestra-config.json` - Agent definitions
- `/Users/brent/git/cc-orchestra/cco/README.md` - CCO overview
- `/Users/brent/.claude/CLAUDE.md` - Global Claude instructions

## Versions

| Version | Date | Changes |
|---------|------|---------|
| 2.0.0 | Nov 15, 2025 | Initial documentation |
| 2.0.1 | Planned | Feedback integration |
| 3.0.0 | Planned | Dynamic registration, versioning |

## License and Attribution

These documents are part of the Claude Code Orchestrator project.

All documentation is provided as-is for system documentation purposes.

---

## Next Steps

1. **Choose your starting point** based on your role
2. **Read the appropriate document(s)**
3. **Follow the guidance in each document**
4. **Refer back as needed for reference**

**Questions?** See the appropriate document's support section.

**Ready to start?** Begin with [Quick Start Guide](./cco/AGENT_DEFINITIONS_QUICK_START.md)
