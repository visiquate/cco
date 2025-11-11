# Claude Orchestra Documentation Index

## Overview

The Claude Orchestra is a comprehensive multi-agent development system with **117 specialized agents** organized across 3 model tiers, providing complete coverage for software development, operations, research, and business analysis.

**Current System**: All agents use direct Claude API integration with intelligent model selection:
- **1 agent** uses Claude Opus 4.1 (Chief Architect)
- **77 agents** use Claude Sonnet 4.5 (intelligent managers, reviewers, complex coding)
- **39 agents** use Claude Haiku 4.5 (basic coders, documentation, utilities)

**Future Enhancement**: ccproxy integration with local Ollama models is planned pending hardware availability.

## Quick Access

### 📊 Agent Information
- **[COMPREHENSIVE_ORCHESTRA_ROSTER.md](COMPREHENSIVE_ORCHESTRA_ROSTER.md)** - Complete list of all 117 agents with roles and specialties
- **[QUICK_AGENT_REFERENCE.md](QUICK_AGENT_REFERENCE.md)** - Quick reference guide for finding and using agents
- **[TDD_AWARE_PIPELINE.md](TDD_AWARE_PIPELINE.md)** - TDD methodology and pipeline coordination

### 🔧 Configuration & Architecture
- **[../config/orchestra-config.json](../config/orchestra-config.json)** - Main configuration file with all 117 agents
- **[ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md)** - System architecture and design diagrams
- **[TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md)** - Comprehensive technical overview

### 📚 Core Documentation
- **[README.md](README.md)** - Getting started guide with quick links
- **[QUICK_START.md](QUICK_START.md)** - 3-step quick start guide
- **[ORCHESTRA_USAGE_GUIDE.md](ORCHESTRA_USAGE_GUIDE.md)** - Comprehensive usage instructions
- **[API_INTEGRATION_GUIDE.md](API_INTEGRATION_GUIDE.md)** - Salesforce and Authentik integration

### 🚀 Advanced Features
- **[AUTONOMOUS_OPERATION_FRAMEWORK.md](AUTONOMOUS_OPERATION_FRAMEWORK.md)** - Extended autonomous operation (4-8 hours)
- **[AUTONOMOUS_WORKFLOW_GUIDE.md](AUTONOMOUS_WORKFLOW_GUIDE.md)** - Complete autonomous workflow example
- **[TDD_AWARE_PIPELINE.md](TDD_AWARE_PIPELINE.md)** - TDD methodology and pipeline

### 🔮 Future Enhancements
- **[future/README.md](future/README.md)** - Future enhancement documentation and roadmap
  - ccproxy integration planning
  - Local LLM routing architecture
  - Hardware requirements analysis
  - Implementation timeline
- **[future/ccproxy/](future/ccproxy/)** - Complete ccproxy deployment documentation
  - Native macOS deployment guide
  - LiteLLM configuration
  - Architecture decision records
  - Monitoring and troubleshooting

### 🛠️ Scripts
- **[../scripts/extract-agents.js](../scripts/extract-agents.js)** - Agent metadata extraction utility
- **[../scripts/build-comprehensive-config.js](../scripts/build-comprehensive-config.js)** - Config builder script

## Document Summaries

### COMPREHENSIVE_ORCHESTRA_ROSTER.md
**Purpose**: Complete reference of all 117 agents
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
**Purpose**: Documentation of configuration updates
**Contents**:
- Configuration evolution and updates
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

| Category | Count | Model Tier | Documentation |
|----------|-------|-----------|-----------------|
| Leadership | 1 | Opus 4.1 | Chief Architect (strategic decisions) |
| Intelligent Managers | 15 | Sonnet 4.5 | Code review, testing, security, DevOps, research |
| Intelligent Specialists | 62 | Sonnet 4.5 | Backend architects, performance, ML engineers, cloud specialists |
| Basic Coders | 25 | Haiku 4.5 | Language specialists, API documentation, basic research |
| Utilities & Support | 14 | Haiku 4.5 | DX optimization, git flow, monitoring, business analysis |
| **TOTAL** | **117** | **Mixed** | [COMPREHENSIVE_ORCHESTRA_ROSTER.md](COMPREHENSIVE_ORCHESTRA_ROSTER.md) |

**Distribution**:
- Opus 4.1: 1 agent (0.9%)
- Sonnet 4.5: 77 agents (65.8%)
- Haiku 4.5: 39 agents (33.3%)

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

### v3.0.0 (2025-11-11) - Documentation Organization with Future Enhancements
- Updated agent count to accurate 117 agents (1 Opus + 77 Sonnet + 39 Haiku)
- Created `/docs/future/` directory with future enhancement documentation
- Documented ccproxy integration planning with hardware requirements
- Added "Future Enhancements" section to INDEX.md
- Clarified current system uses direct Claude API (not yet using local LLMs)
- Added implementation timeline for ccproxy deployment phases
- Improved documentation organization and navigation

### v2.0.0 (2025-11-10) - Comprehensive Roster
- Current configuration: 117 agents (1 Opus + 77 Sonnet + 39 Haiku)
- Added agent categories and distribution analysis
- Integrated all agents from ~/.claude/agents/
- Created comprehensive documentation suite
- All agents assigned to specific model tiers

### v1.0.0 (Previous)
- 17 original agents
- 5 categories
- Basic coordination structure

## File Locations

### Configuration
```
/Users/brent/git/cc-orchestra/config/orchestra-config.json
```

### Documentation
```
/Users/brent/git/cc-orchestra/docs/
├── INDEX.md (this file - documentation index)
├── README.md (getting started)
├── COMPREHENSIVE_ORCHESTRA_ROSTER.md (117 agent details)
├── QUICK_AGENT_REFERENCE.md (quick lookup)
├── ORCHESTRA_USAGE_GUIDE.md (usage instructions)
├── QUICK_START.md (3-step guide)
├── AUTONOMOUS_OPERATION_FRAMEWORK.md (4-8 hour operation)
├── future/ (Future enhancements)
│   ├── README.md (Future enhancement roadmap)
│   └── ccproxy/ (ccproxy deployment documentation)
│       ├── NATIVE_MACOS_DEPLOYMENT.md
│       ├── DEPLOYMENT_STEPS.md
│       ├── ARCHITECTURE_DECISIONS.md
│       ├── ARCHITECTURE.md
│       ├── config.yaml
│       └── com.visiquate.ccproxy.plist
└── [other documentation files]
```

### Scripts
```
/Users/brent/git/cc-orchestra/scripts/
├── extract-agents.js
└── build-comprehensive-config.js
```

### Agent Definitions
```
~/.claude/agents/
├── [117 agent definition files]
└── *.md
```

## Additional Resources

- **Main README**: [../README.md](../README.md) - Project overview and architecture
- **Project CLAUDE.md**: [../CLAUDE.md](../CLAUDE.md) - Agent configuration and model assignments
- **Agent Roster (TDD)**: [ORCHESTRA_ROSTER_TDD.md](ORCHESTRA_ROSTER_TDD.md) - TDD pipeline details
- **Future Enhancements**: [future/README.md](future/README.md) - ccproxy deployment roadmap

## Quick Commands

### Validate Config
```bash
cat /Users/brent/git/cc-orchestra/config/orchestra-config.json | jq '.' > /dev/null && echo "✅ Valid JSON"
```

### Count Total Agents
```bash
cat /Users/brent/git/cc-orchestra/config/orchestra-config.json | jq '.agents | length'
```

### List Agent Names and Models
```bash
cat /Users/brent/git/cc-orchestra/config/orchestra-config.json | jq -r '.agents[] | "\(.name) - \(.model)"'
```

### Extract Agents by Model Tier
```bash
# Opus agents
cat /Users/brent/git/cc-orchestra/config/orchestra-config.json | jq -r '.agents[] | select(.model == "opus") | .name'

# Sonnet agents
cat /Users/brent/git/cc-orchestra/config/orchestra-config.json | jq -r '.agents[] | select(.model == "sonnet-4.5") | .name'

# Haiku agents
cat /Users/brent/git/cc-orchestra/config/orchestra-config.json | jq -r '.agents[] | select(.model == "haiku") | .name'
```

### Validate ccproxy Config
```bash
python3 -c "import yaml; yaml.safe_load(open('/Users/brent/git/cc-orchestra/docs/future/ccproxy/config.yaml'))" && echo "✅ Valid YAML"
```

## Navigation Guide

### For New Users
1. Start: [README.md](README.md)
2. Quick start: [QUICK_START.md](QUICK_START.md)
3. Agent selection: [QUICK_AGENT_REFERENCE.md](QUICK_AGENT_REFERENCE.md)

### For Advanced Users
1. Architecture: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md)
2. TDD pipeline: [TDD_AWARE_PIPELINE.md](TDD_AWARE_PIPELINE.md)
3. Autonomous operation: [AUTONOMOUS_OPERATION_FRAMEWORK.md](AUTONOMOUS_OPERATION_FRAMEWORK.md)

### For Deployment Planning
1. Current system: [ORCHESTRA_USAGE_GUIDE.md](ORCHESTRA_USAGE_GUIDE.md)
2. Future enhancements: [future/README.md](future/README.md)
3. ccproxy deployment: [future/ccproxy/NATIVE_MACOS_DEPLOYMENT.md](future/ccproxy/NATIVE_MACOS_DEPLOYMENT.md)

## Support

For questions or issues:

**Using the current system**:
1. Check relevant documentation in [README.md](README.md)
2. Review agent selection in [QUICK_AGENT_REFERENCE.md](QUICK_AGENT_REFERENCE.md)
3. Verify agent configuration in orchestra-config.json
4. Consult [ORCHESTRA_USAGE_GUIDE.md](ORCHESTRA_USAGE_GUIDE.md)

**Planning future ccproxy deployment**:
1. Review [future/README.md](future/README.md)
2. Check hardware requirements in [future/ccproxy/NATIVE_MACOS_DEPLOYMENT.md](future/ccproxy/NATIVE_MACOS_DEPLOYMENT.md)
3. Review deployment steps in [future/ccproxy/DEPLOYMENT_STEPS.md](future/ccproxy/DEPLOYMENT_STEPS.md)
4. Check architecture decisions in [future/ccproxy/ARCHITECTURE_DECISIONS.md](future/ccproxy/ARCHITECTURE_DECISIONS.md)

---

**Documentation Version**: 3.0.0
**Last Updated**: 2025-11-11
**Total Agents**: 117 (1 Opus + 77 Sonnet + 39 Haiku)
**Current Status**: Direct Claude API (ccproxy planned, hardware pending)
**Total Documentation Files**: 50+ files with organized hierarchy
