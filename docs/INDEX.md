# Claude Orchestra Documentation Index

## Overview
The Claude Orchestra is a comprehensive multi-agent development system with **125 specialized agents** organized into 13 categories, providing complete coverage for software development, operations, research, and business analysis.

## Quick Access

### 📊 Agent Information
- **[COMPREHENSIVE_ORCHESTRA_ROSTER.md](COMPREHENSIVE_ORCHESTRA_ROSTER.md)** - Complete list of all 125 agents with roles and specialties
- **[QUICK_AGENT_REFERENCE.md](QUICK_AGENT_REFERENCE.md)** - Quick reference guide for finding and using agents
- **[CONFIG_UPDATE_SUMMARY.md](CONFIG_UPDATE_SUMMARY.md)** - Detailed summary of the config expansion from 17 to 125 agents
- **[DEPLOYMENT_COMPLETE.txt](DEPLOYMENT_COMPLETE.txt)** - Visual summary of the completed deployment

### 🔧 Configuration
- **[../config/orchestra-config.json](../config/orchestra-config.json)** - Main configuration file with all 125 agents

### 📚 Additional Documentation
- **[ARMY_USAGE_GUIDE.md](ARMY_USAGE_GUIDE.md)** - Comprehensive usage guide (if exists)
- **[TDD_AWARE_PIPELINE.md](TDD_AWARE_PIPELINE.md)** - TDD methodology guide (if exists)
- **[DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md)** - ccproxy deployment status (if exists)

### 🛠️ Scripts
- **[../scripts/extract-agents.js](../scripts/extract-agents.js)** - Agent metadata extraction utility
- **[../scripts/build-comprehensive-config.js](../scripts/build-comprehensive-config.js)** - Config builder script

## Document Summaries

### COMPREHENSIVE_ORCHESTRA_ROSTER.md
**Purpose**: Complete reference of all 125 agents
**Contents**:
- Full agent listing by category
- Role descriptions and specialties
- Model configuration details
- Coordination and integration information
- Usage examples

**When to use**: Need detailed information about specific agents or want to understand the full roster.

### QUICK_AGENT_REFERENCE.md
**Purpose**: Fast lookup guide for agent selection
**Contents**:
- Quick stats and category overview
- Most commonly used agents
- Agent selection by task type
- Agent types reference
- Spawning examples
- Tips for effective usage

**When to use**: Need to quickly find the right agent for a specific task.

### CONFIG_UPDATE_SUMMARY.md
**Purpose**: Documentation of the config expansion
**Contents**:
- What was done (17 → 125 agents)
- Changes made by category
- Agent configuration format
- Agent distribution breakdown
- Model configuration details
- Files created/modified
- Validation results

**When to use**: Need to understand how the config was expanded or troubleshoot configuration issues.

### DEPLOYMENT_COMPLETE.txt
**Purpose**: Visual summary of completed deployment
**Contents**:
- Agent distribution table
- Preserved and added agents breakdown
- Model configuration summary
- Files created/modified
- Validation checks
- Capabilities coverage checklist
- Usage example

**When to use**: Want a quick visual overview of the complete army deployment.

## Agent Categories Quick Reference

| Category | Count | Documentation Section |
|----------|-------|----------------------|
| Architect | 1 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 1 |
| Coding Agents | 6 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 2 |
| Integration Agents | 3 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 3 |
| Development Agents | 25 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 4 |
| Data Agents | 9 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 5 |
| Infrastructure Agents | 10 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 6 |
| Security Agents | 8 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 7 |
| AI/ML Agents | 6 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 8 |
| MCP Agents | 6 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 9 |
| Documentation Agents | 7 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 10 |
| Research Agents | 10 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 11 |
| Support Agents | 30 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 12 |
| Business Agents | 4 | COMPREHENSIVE_ORCHESTRA_ROSTER.md § 13 |

## Common Use Cases

### Starting a New Project
1. Read: QUICK_AGENT_REFERENCE.md
2. Spawn: Chief Architect + relevant specialists
3. Reference: COMPREHENSIVE_ORCHESTRA_ROSTER.md for agent details

### Finding the Right Agent
1. Check: QUICK_AGENT_REFERENCE.md § "Finding the Right Agent"
2. Verify: COMPREHENSIVE_ORCHESTRA_ROSTER.md for full role description
3. Spawn: Using correct agent type from reference

### Troubleshooting Configuration
1. Review: CONFIG_UPDATE_SUMMARY.md § "Validation"
2. Check: ../config/orchestra-config.json structure
3. Verify: Agent file references in ~/.claude/agents/

### Understanding Agent Capabilities
1. Browse: COMPREHENSIVE_ORCHESTRA_ROSTER.md by category
2. Quick lookup: QUICK_AGENT_REFERENCE.md tables
3. Details: Individual agent files in ~/.claude/agents/

## Version History

### v2.0.0 (2025-11-10) - Comprehensive Roster
- Expanded from 17 to 125 agents
- Added 10 new categories
- Integrated all agents from ~/.claude/agents/
- Created comprehensive documentation suite
- All agents use model: "sonnet-4.5" (except Architect)

### v1.0.0 (Previous)
- 17 original agents
- 5 categories
- Basic coordination structure

## File Locations

### Configuration
```
/Users/brent/git/cc-army/config/orchestra-config.json
```

### Documentation
```
/Users/brent/git/cc-army/docs/
├── INDEX.md (this file)
├── COMPREHENSIVE_ORCHESTRA_ROSTER.md
├── QUICK_AGENT_REFERENCE.md
├── CONFIG_UPDATE_SUMMARY.md
├── DEPLOYMENT_COMPLETE.txt
└── [other documentation files]
```

### Scripts
```
/Users/brent/git/cc-army/scripts/
├── extract-agents.js
└── build-comprehensive-config.js
```

### Agent Definitions
```
~/.claude/agents/
├── [107 agent definition files]
└── *.md
```

## Additional Resources

- **Main README**: [../README.md](../README.md)
- **Project CLAUDE.md**: [../CLAUDE.md](../CLAUDE.md)
- **Agent Roster (TDD)**: [ORCHESTRA_ROSTER_TDD.md](ORCHESTRA_ROSTER_TDD.md) (if exists)

## Quick Commands

### Validate Config
```bash
cat /Users/brent/git/cc-army/config/orchestra-config.json | jq '.' > /dev/null && echo "✅ Valid JSON"
```

### Count Agents by Category
```bash
cat /Users/brent/git/cc-army/config/orchestra-config.json | jq '{
  codingAgents: (.codingAgents | length),
  developmentAgents: (.developmentAgents | length),
  total: (1 + (.codingAgents | length) + (.developmentAgents | length) + ...)
}'
```

### Extract Agent Names
```bash
cat /Users/brent/git/cc-army/config/orchestra-config.json | jq -r '.developmentAgents[] | .name'
```

### Rebuild Config
```bash
node /Users/brent/git/cc-army/scripts/build-comprehensive-config.js
```

## Support

For questions or issues:
1. Check relevant documentation file
2. Review config structure in orchestra-config.json
3. Verify agent files in ~/.claude/agents/
4. Consult QUICK_AGENT_REFERENCE.md for usage patterns

---

**Documentation Version**: 2.0.0
**Last Updated**: 2025-11-10
**Total Agents**: 125
**Total Documents**: 4 core + scripts + config
