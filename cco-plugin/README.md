# CCO Plugin - Claude Code Orchestra

Multi-agent orchestration plugin for Claude Code with 117 specialized agents.

## Overview

CCO (Claude Code Orchestra) is a plugin that enables sophisticated multi-agent workflows within Claude Code. It provides a TDD-aware pipeline with three model tiers optimized for cost and capability.

## Installation

Copy or symlink this plugin directory to your Claude Code plugins location:

```bash
# Symlink for development
ln -s /path/to/cco-plugin ~/.claude/plugins/cco

# Or copy for production
cp -r /path/to/cco-plugin ~/.claude/plugins/cco
```

## Directory Structure

```
cco-plugin/
├── .claude-plugin/
│   └── plugin.json          # Plugin manifest
├── agents/
│   ├── leadership/          # Opus agents (strategic)
│   ├── managers/            # Sonnet agents (coordination)
│   └── specialists/         # Haiku agents (implementation)
├── skills/
│   ├── conduct/             # Orchestrate full workflows
│   ├── spawn-agents/        # Spawn agents in parallel
│   ├── coordinate/          # Agent coordination
│   ├── tdd/                 # Test-driven development
│   ├── review-code/         # Code review workflows
│   ├── refactor/            # Refactoring operations
│   ├── plan-sprint/         # Sprint planning
│   ├── status/              # Status reporting
│   ├── handoff/             # Agent handoffs
│   ├── remember/            # Knowledge storage
│   ├── recall/              # Knowledge retrieval
│   ├── brief/               # Agent briefing
│   ├── security-audit/      # Security auditing
│   ├── perf-optimize/       # Performance optimization
│   ├── migrate-db/          # Database migrations
│   └── deploy/              # Deployment workflows
├── hooks/                   # Lifecycle hooks
└── scripts/                 # Utility scripts
```

## Model Distribution

| Model | Agent Count | Use Cases |
|-------|-------------|-----------|
| **Opus** | 1 | Chief Architect - strategic decisions |
| **Sonnet** | 35 | Managers, reviewers, complex implementations |
| **Haiku** | 81 | Basic coders, documentation, utilities |

## Skills

### conduct
Orchestrate a complete multi-agent workflow for a requirement.

### spawn-agents
Spawn multiple agents in parallel for concurrent execution.

### coordinate
Enable cross-agent communication and synchronization.

### tdd
Test-driven development workflow - tests written BEFORE implementation.

### review-code
Comprehensive code review with multiple specialized reviewers.

### refactor
Intelligent refactoring with preservation of functionality.

### plan-sprint
Sprint planning with task breakdown and agent assignment.

### status
Report current workflow status across all active agents.

### handoff
Coordinate handoffs between agents for sequential tasks.

### remember
Store knowledge in the persistent knowledge base.

### recall
Retrieve relevant knowledge from the knowledge base.

### brief
Brief agents with context before starting work.

### security-audit
Run security audit with the Security Auditor agent.

### perf-optimize
Performance optimization workflow.

### migrate-db
Database migration with the Database Architect.

### deploy
Deployment workflow with DevOps agents.

## Usage Examples

```bash
# Conduct a full orchestration
/cco conduct "Build a REST API with JWT authentication"

# Spawn specific agents
/cco spawn-agents tdd-agent security-auditor python-specialist

# Run TDD workflow
/cco tdd "Add user registration feature"

# Code review
/cco review-code src/auth/

# Security audit
/cco security-audit

# Check status
/cco status
```

## License

MIT
