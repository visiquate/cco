# Claude Orchestra Roster - Current Configuration

**⚠️ This file redirects to the current roster documentation**

## Current Roster: 125 Specialized Agents

The Claude Orchestra now consists of **125 specialized agents** organized into 13 categories, providing comprehensive coverage for all aspects of software development, operations, research, and business analysis.

**See the official roster documentation:**
- **Primary**: [docs/COMPREHENSIVE_ORCHESTRA_ROSTER.md](docs/COMPREHENSIVE_ORCHESTRA_ROSTER.md) - Complete 125-agent roster
- **Config**: [config/orchestra-config.json](config/orchestra-config.json) - Full agent configuration (2,556 lines)
- **Quick Reference**: [docs/QUICK_AGENT_REFERENCE.md](docs/QUICK_AGENT_REFERENCE.md) - Fast agent lookup
- **Update Summary**: [docs/CONFIG_UPDATE_SUMMARY.md](docs/CONFIG_UPDATE_SUMMARY.md) - Integration details
- **Usage**: [docs/ARMY_USAGE_GUIDE.md](docs/ARMY_USAGE_GUIDE.md) - How to use the army

## Quick Summary

### Agent Categories (125 Total)

1. **Architect** (1) - Chief Architect with Opus 4.1 → Sonnet 4.5 fallback
2. **Coding Agents** (6) - TDD, Python, Swift, Go, Rust, Flutter specialists
3. **Integration Agents** (3) - API Explorer, Salesforce, Authentik experts
4. **Development Agents** (25) - Frontend, backend, fullstack, language experts
5. **Data Agents** (9) - Database, data engineering, analytics specialists
6. **Infrastructure Agents** (10) - DevOps, cloud, deployment automation
7. **Security Agents** (8) - Security auditing, compliance, pen testing
8. **AI/ML Agents** (6) - ML engineering, LLM integration, MLOps
9. **MCP Agents** (6) - Model Context Protocol specialists
10. **Documentation Agents** (7) - Technical writing, API docs, changelogs
11. **Research Agents** (10) - Technical, academic, comprehensive research
12. **Support Agents** (30) - QA, testing, UX, performance, operations
13. **Business Agents** (4) - Product strategy, analysis, marketing

### Model Configuration

All agents (except Chief Architect) use **model: "sonnet-4.5"** per configuration:
- **Chief Architect**: Opus 4.1 with Sonnet 4.5 fallback
- **All Other Agents**: Sonnet 4.5 (claude-sonnet-4.5)

Optional ccproxy routing available for local Ollama models at https://coder.visiquate.com

## Deployment Status

- **Config**: `/Users/brent/git/cc-army/config/orchestra-config.json`
- **Backup**: `/Users/brent/git/cc-army/config/orchestra-config.json.backup`
- **Status**: ✅ Operational with 125 agents
- **Version**: 2.0.0 (Comprehensive Roster)
- **Last Updated**: 2025-11-10

## Integration Highlights

- ✅ Expanded from 17 to 125 agents
- ✅ All 107 agents from ~/.claude/agents/ integrated
- ✅ Organized into 13 logical categories
- ✅ Consistent model configuration (sonnet-4.5)
- ✅ Autonomous authority levels configured
- ✅ Knowledge Manager coordination enabled
- ✅ Validated JSON structure (2,556 lines)

---

**Previous versions:**
- `docs/ORCHESTRA_ROSTER_TDD.md` - 15-agent TDD pipeline (superseded)
- `ORCHESTRA_ROSTER_V1_DEPRECATED.md` - Original 14-agent roster (deprecated)
- `ORCHESTRA_ROSTER_V2.md` - Intermediate version (superseded)
