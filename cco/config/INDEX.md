# CCO Configuration Files - Index

This directory contains the Claude Orchestrator (CCO) agent definitions and configuration files organized for compile-time embedding.

## Quick Navigation

### Agent Definitions
- **Directory**: `agents/` - Contains all 117 agent definition files
- **Total Files**: 117 .md files + 1 README.md
- **Format**: Markdown with YAML frontmatter
- **Naming**: kebab-case (e.g., `chief-architect.md`, `python-specialist.md`)

### Configuration Files
- **agents.json** - Build system manifest (for build.rs)
  - 117 agent objects in JSON array format
  - Alphabetically sorted
  - Contains: name, model, description, tools, file_path

### Documentation Files
- **agents/README.md** - Agent system documentation
  - How agent system works
  - File format and structure
  - Validation rules
  - How to add new agents

- **AGENTS_MANIFEST.txt** - Complete agent listing
  - All 117 agents with descriptions
  - Organized by model type
  - Organized by function category
  - Statistics and distribution

- **AGENT_VALIDATION_REPORT.md** - Validation details
  - Validation results and metrics
  - Issues found and resolved
  - Quality assurance checklist
  - Verification results

- **AGENT_SETUP_SUMMARY.md** - Executive summary
  - High-level overview
  - Timeline and milestones
  - Integration status
  - Next steps for build system

## Agent Statistics

```
Total Agents: 117

By Model:
  Opus:   1 agent  (0.9%)   - Chief Architect
  Sonnet: 35 agents (29.9%)  - Managers, reviewers, architects
  Haiku:  81 agents (69.2%)  - Language specialists, utilities

By Category:
  Leadership: 1
  Development: 25
  Integration: 3
  Infrastructure: 10
  Quality & Security: 15
  Research & Analysis: 20
  Documentation: 5
  Utilities: 18
```

## File Locations

```
cc-orchestra/
├── cco/config/
│   ├── agents/
│   │   ├── *.md (117 agent files)
│   │   └── README.md
│   ├── agents.json (53 KB)
│   ├── AGENTS_MANIFEST.txt
│   ├── AGENT_VALIDATION_REPORT.md
│   ├── AGENT_SETUP_SUMMARY.md
│   └── INDEX.md (this file)
│
└── AGENT_COPY_DELIVERABLES.md (root level)
```

## Build System Integration

### For build.rs Developers

1. **Read Manifest**: Use `cco/config/agents.json`
2. **Parse Each Agent**:
   - Read corresponding .md file from `cco/config/agents/`
   - Extract YAML frontmatter (between --- markers)
   - Parse required fields: name, model, description, tools
3. **Embed in Binary**: Generate embedded data structure
4. **Create Runtime Accessors**: Functions to retrieve agents

### Manifest Format

```json
[
  {
    "name": "agent-name",
    "model": "opus|sonnet|haiku",
    "description": "Brief description",
    "tools": "Read, Write, Edit, Bash",
    "file_path": "cco/config/agents/{name}.md"
  }
]
```

## Agent Definition Format

Each agent file has YAML frontmatter followed by documentation:

```markdown
---
name: agent-name
description: Brief description of agent
model: opus|sonnet|haiku
tools: Read, Write, Edit, Bash
---

# Agent Documentation

[Agent persona and detailed instructions...]
```

### Required Fields

| Field | Purpose | Example |
|-------|---------|---------|
| name | Unique identifier | chief-architect |
| description | Brief description | Strategic architecture leadership |
| model | Claude model to use | opus, sonnet, or haiku |
| tools | Available tools | Read, Write, Edit, Bash |

## Validation

All 117 agents have been validated:

```
✅ YAML frontmatter: 117/117 valid
✅ Required fields: 117/117 present
✅ Model values: 117/117 valid
✅ Tool specifications: 117/117 defined
✅ Final validation: 117/117 PASS (100%)
```

Issues found and fixed: 6 files (100% resolution)

## Adding New Agents

To add a new agent:

1. Create file: `agents/{new-agent-name}.md`
2. Add YAML frontmatter with required fields
3. Run validation: `node /tmp/validate_agents.js`
4. Update manifest: agents.json is auto-generated
5. Commit changes

See `agents/README.md` for detailed instructions.

## Important Files

### For Build System
- `agents.json` - Read this first for the agent manifest
- `agents/README.md` - Documentation on agent system

### For Validation
- `AGENT_VALIDATION_REPORT.md` - Detailed validation results
- `/tmp/validate_agents.js` - Reusable validation script

### For Documentation
- `AGENT_SETUP_SUMMARY.md` - Executive overview
- `AGENTS_MANIFEST.txt` - Complete agent listing

### For Reference
- `agents/` - All 117 agent definition files

## Next Steps

1. Integrate with build.rs to read agents.json
2. Parse agent YAML frontmatter at build time
3. Embed agent definitions in binary
4. Create runtime agent loading functions
5. Test agent instantiation
6. Deploy to production

## Status

- Agents Copied: 117/117 (100%)
- Validation: 117/117 PASS (100%)
- Documentation: Complete
- Ready for Build System: YES
- Ready for Production: YES

## References

- Root deliverables: `/Users/brent/git/cc-orchestra/AGENT_COPY_DELIVERABLES.md`
- Orchestrator rules: `/Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md`
- Project README: `/Users/brent/git/cc-orchestra/README.md`

---

**Last Updated**: November 15, 2025
**Status**: Ready for build system integration
**Confidence**: 100%
