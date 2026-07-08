# Conduct: Main Orchestration Workflow

## Trigger
`/conduct <requirement>`

Analyzes a user requirement and orchestrates the complete multi-agent workflow to deliver a solution.

## Purpose
The Conduct skill:
- Analyzes requirements to understand scope and complexity
- Designs optimal architecture with the Chief Architect
- Identifies and spawns appropriate agents in parallel
- Coordinates agents through the knowledge base
- Aggregates results into coherent deliverables

## Workflow

### Phase 1: Requirement Analysis
1. Parse the user requirement — extract scope, constraints, tech stack
2. Identify requirement type: Feature, Bug fix, Refactoring, Infrastructure, Security, Performance, Documentation

### Phase 2: Architect Decision
Spawn Chief Architect (Opus) to:
- Design system architecture
- Identify technical decisions
- List required agents by role

### Phase 3: Store Architecture & Create Tasks
```
knowledge_store(
  text="Architecture for [requirement]: Design decision...",
  knowledge_type="architecture",
  agent="chief-architect"
)

task_create(
  title="[Task name]",
  description="[Work to perform]",
  dependencies=[],
  agent="agent-name"
)
```

### Phase 4: Spawn Agent Teams by Requirement Type

**Feature Development:**
- TDD Agent (Sonnet) — tests first
- Language Specialists (Haiku) — implement
- Code Reviewer (Sonnet) — review
- Security Auditor (Sonnet) — security
- Technical Writer (Haiku) — docs

**Bug Fixes:**
- Error Detective (Sonnet) — root cause
- TDD Agent (Sonnet) — regression test
- Language Specialists (Haiku) — fix
- Test Engineer (Sonnet) — validate
- Code Reviewer (Sonnet) — review

**Refactoring:**
- Architect Review (Sonnet) — design
- TDD Agent (Sonnet) — test coverage
- Language Specialists (Haiku) — implement
- Performance Engineer (Sonnet) — optimize
- Code Reviewer (Sonnet) — review

**Infrastructure/DevOps:**
- DevOps Engineer (Sonnet)
- Cloud Architect (Sonnet)
- Terraform Specialist (Sonnet)
- Deployment Engineer (Sonnet)
- Monitoring Specialist (Sonnet)

**Security/Compliance:**
- Security Auditor (Sonnet)
- Penetration Tester (Sonnet)
- Compliance Specialist (Sonnet)
- Security Engineer (Sonnet)

### Phase 5: Parallel Execution
- Spawn all selected agents in ONE message
- Use Task tool to create parallel tasks
- Each agent has clear responsibilities
- Coordinate via knowledge base

### Phase 6: Monitor Coordination
```
knowledge_search(query="agent progress")
task_status(task_id="task-123")
knowledge_search(query="blocker")
```

### Phase 7: Aggregate Results
1. Collect results from all agents
2. Architect validates quality and consistency
3. Final review by Chief Architect
4. Document lessons learned

## Knowledge Manager Integration

**Before spawning agents:**
```
knowledge_recall(query="related decisions")
knowledge_search(query="technology constraints")
```

**After completion:**
```
knowledge_store(
  text="Conduct Complete: [requirement] - Results: [summary]",
  knowledge_type="completion",
  agent="conductor"
)
```

## Complexity Scoring

| Complexity | Team Size | Duration |
|-----------|-----------|----------|
| Simple (add feature) | 3-5 agents | 20-30 min |
| Moderate (refactor) | 5-8 agents | 45-60 min |
| Complex (new service) | 8-12 agents | 60-90 min |
| Very Complex | 12-20 agents | 2-3 hours |

## Best Practices

1. **Always start with Architect** — Get design before coding
2. **Spawn all agents at once** — Maximize parallelization
3. **Use knowledge manager** — Enable cross-agent communication
4. **Store decisions** — Support future hand-offs
5. **Security first** — Always include security review
6. **Test-driven** — Tests before implementation

## Output Format

Return comprehensive handoff with:
- Architecture overview
- Agent assignments and progress
- Code changes with tests
- Security & performance findings
- Documentation generated
- Next steps and deployment plan
- Knowledge base references
