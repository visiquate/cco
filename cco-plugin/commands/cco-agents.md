# CCO Agents

## Trigger
`/cco-agents`

List and inspect all available agents by tier and specialty.

## Purpose
This command displays the roster of 117 specialized agents organized by tier:
- Agent names and roles
- Model tier distribution (Opus, Sonnet, Haiku)
- Agent specialties and use cases
- How to spawn specific agents

## Usage

```
You: /cco-agents

🎭 Agent Roster
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total: 117 agents across 3 tiers

🟦 OPUS (Strategic Leadership) [1 agent]
  • Chief Architect (agent-id: chief-architect)
    Role: Orchestration, architecture decisions, team coordination

🟨 SONNET (Management & Review) [35 agents]
  • Backend Architect (backend-architect)
    Role: Architecture decisions, complex system design
  • Code Reviewer (code-reviewer)
    Role: Comprehensive code review, quality assurance
  • Security Auditor (security-auditor)
    Role: Security assessment, threat analysis
  • Python Specialist (python-specialist)
    Role: Python development, framework expertise
  • TypeScript Architect (typescript-architect)
    Role: TypeScript/Next.js architecture
  • Database Architect (database-architect)
    Role: Database design, optimization
  ... (29 more Sonnet agents)

🟩 HAIKU (Implementation Specialists) [81 agents]
  • Rust Specialist (rust-specialist)
    Role: Rust systems programming, performance
  • React Performance Expert (react-perf-expert)
    Role: React optimization, performance tuning
  • Documentation Writer (docs-writer)
    Role: Technical documentation, guides
  ... (78 more Haiku agents)

Agent Filter Examples:
  • cco agents list --model opus
  • cco agents list --model sonnet
  • cco agents list --model haiku
  • cco agents info backend-architect
```

## When to Use

Use `/cco-agents` when you want to:
- See all available agents
- Find an agent for a specific task
- Understand the agent tier distribution
- Check an agent's capabilities
- Spawn a specific agent by ID

## Implementation

To use this command, ask Claude to:
1. Run `cco agents list` to get the complete agent roster
2. Organize agents by tier (Opus, Sonnet, Haiku)
3. Show agent IDs and brief descriptions of their specialties
4. Optionally explain the cost implications (Opus >> Sonnet >> Haiku)

## Notes

- **Opus** (1 agent): Most expensive, strategic decisions only
- **Sonnet** (35 agents): Mid-tier, complex tasks and reviews
- **Haiku** (81 agents): Cost-effective, task-specific specialists
- Agent IDs are used with `cco agents spawn <agent-id>`
- Always specify the correct agent model to optimize cost
