# Brief: Get Briefing

## Trigger
`/brief [epic or context]`

Generates a comprehensive briefing summarizing current project state, decisions, and context after knowledge base compaction.

## Purpose
The Brief skill provides:
- Quick context retrieval after session compaction
- Consolidated project state snapshot
- Decision summary for new session startup
- Onboarding material for new agents
- Progress overview and critical items highlight

## Workflow

### Phase 1: Gather Current State
```
knowledge_recall(query="project status")        # Comprehensive context
task_list(limit=100)                           # Task progress
agent_performance_stats(agent_name=null)       # Team status
knowledge_lifecycle_status()                   # Knowledge health
knowledge_search(query="blocker")              # Identify blockers
```

### Phase 2: Compile Briefing Sections

**Section 1: Executive Summary**
- Project name and epic
- Overall progress % and status (ON_TRACK, BEHIND, AHEAD)
- Timeline variance and velocity
- Open blockers count
- Team size and active agents
- Critical path items

**Section 2: Architectural Decisions**
- Key decisions made (approval status)
- Technology choices
- Architecture patterns
- Pending decisions

**Section 3: Work Progress**
- Completed tasks/stories (points)
- In-progress items (% complete)
- Not started (blocked or waiting)
- Total points and % complete

**Section 4: Blockers & Risks**
- Open blockers (status, severity, impact, mitigation)
- Identified risks (likelihood, impact, mitigation)
- Timeline impact assessment

**Section 5: Team & Workload**
- Active agents by role
- Utilization % and capacity
- Current assignments
- Concerns or bottlenecks

**Section 6: Quality Metrics**
- Test coverage (target vs actual)
- Performance (target vs actual)
- Code quality metrics
- Security issues count

**Section 7: Tech Stack**
- Languages and frameworks
- Database and infrastructure
- Testing tools
- CI/CD approach

**Section 8: Next 24-48 Hours**
- Priority actions (critical, high, medium)
- Owners and expected duration
- Success criteria
- Prerequisite work

### Phase 3: Store Briefing
```
knowledge_store(
  text="Briefing: [Epic] - [Date]\n" +
       "Progress: X%, Status: [STATUS], Blockers: N\n" +
       "Critical Path: [steps]\n" +
       "Next Actions: [list]",
  knowledge_type="status",
  agent="brief-agent"
)
```

### Phase 4: Format for Onboarding
Create concise onboarding summary:
- 5-minute read
- Key decisions summary
- Your specific task assignment
- Prerequisites and blockers
- Questions and contacts

## Briefing Contents

**For Continuing Work:**
- Current progress and metrics
- Open blockers and workarounds
- Team capacity
- Next immediate actions
- Timeline status

**For Onboarding New Agents:**
- Project overview
- Key decisions with rationale
- Current state and progress
- Specific task assignment
- Prerequisite work
- Key constraints

## Best Practices

1. **Be concise** — 80-100 lines max per briefing
2. **Include metrics** — Progress %, velocity, timeline
3. **Highlight risks** — Early warning system
4. **Provide context** — Why decisions were made
5. **Link to KB** — Reference knowledge base entries
6. **Action-oriented** — Clear next steps

## Output Format

Return comprehensive briefing with:
- Executive summary (status, progress, timeline)
- All key architectural decisions
- Current work progress by story
- All blockers and mitigations
- Active agents and workload
- Quality metrics and baselines
- Technology stack overview
- Next 24-48 hour action items
- Recommendations and optimizations
- Knowledge base references
- Success criteria for next session
