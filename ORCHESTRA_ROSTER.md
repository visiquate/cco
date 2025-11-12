# Claude Orchestra Roster - Current Configuration

## Current Roster: 119 Specialized Agents

The Claude Orchestra consists of **119 specialized agents** organized into 12 categories, providing comprehensive coverage for all aspects of software development, operations, research, and business analysis.

**Official Documentation:**
- **Config**: [config/orchestra-config.json](config/orchestra-config.json) - Full agent configuration
- **Usage Guide**: [docs/ORCHESTRA_USAGE_GUIDE.md](docs/ORCHESTRA_USAGE_GUIDE.md) - How to use the orchestra
- **Quick Reference**: [docs/QUICK_AGENT_REFERENCE.md](docs/QUICK_AGENT_REFERENCE.md) - Fast agent lookup

## Quick Summary

### Agent Categories (119 Total)

1. **Architect** (1) - Chief Architect with Opus 4.1
2. **Coding Agents** (6) - TDD, Python, Swift, Go, Rust, Flutter specialists
3. **Integration Agents** (3) - API Explorer, Salesforce, Authentik experts
4. **Development Agents** (29) - Frontend, backend, fullstack, language experts
5. **Data Agents** (11) - Database, data engineering, analytics specialists
6. **Infrastructure Agents** (10) - DevOps, cloud, deployment automation
7. **Security Agents** (8) - Security auditing, compliance, pen testing
8. **AI/ML Agents** (6) - ML engineering, LLM integration, MLOps
9. **MCP Agents** (6) - Model Context Protocol specialists
10. **Documentation Agents** (7) - Technical writing, API docs, changelogs
11. **Research Agents** (10) - Technical, academic, comprehensive research
12. **Support Agents** (18) - QA, testing, UX, performance, operations
13. **Business Agents** (4) - Product strategy, analysis, marketing

### Model Configuration

Agents are assigned models based on role complexity:
- **Chief Architect**: Opus 4.1 (strategic leadership)
- **Intelligent Managers** (37 agents, 31.4%): Sonnet 4.5 (complex reasoning, code review, security, architecture)
- **Basic Specialists** (81 agents, 68.6%): Haiku 4.5 (language coding, documentation, utilities)

**ccproxy**: Future enhancement for local LLM routing (hardware pending) - currently using direct Claude API

## Deployment Status

- **Config**: `/Users/brent/git/cc-orchestra/config/orchestra-config.json`
- **Status**: ✅ Operational with 119 agents
- **Version**: 2.0.0 (Model-Optimized Configuration)
- **Last Updated**: 2025-11-11

## Configuration Highlights

- ✅ **119 Total Agents**: 1 Chief Architect + 118 specialized agents
- ✅ **Model Optimization**: Agents assigned by role complexity
  - 1 agent using Opus 4.1 (strategic leadership)
  - 37 agents using Sonnet 4.5 (intelligent management, 31.4%)
  - 81 agents using Haiku 4.5 (basic coding & utilities, 68.6%)
- ✅ **Organized into 12 categories** for logical grouping
- ✅ **Autonomous authority levels** configured per agent
- ✅ **Knowledge Manager coordination** enabled
- ✅ **ccproxy references removed** (future enhancement, hardware pending)

---

**Previous versions:**
- `docs/ORCHESTRA_ROSTER_TDD.md` - 15-agent TDD pipeline (superseded)
- `ORCHESTRA_ROSTER_V1_DEPRECATED.md` - Original 14-agent roster (deprecated)
- `ORCHESTRA_ROSTER_V2.md` - Intermediate version (superseded)
