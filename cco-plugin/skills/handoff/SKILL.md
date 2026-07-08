# Handoff: Session Handoff

## Trigger
`/handoff [epic or current work]`

Generates comprehensive handoff documentation to transition work between sessions or agents.

## Purpose
The Handoff skill provides:
- Complete context preservation
- Session state summarization
- Next steps documentation
- Critical decisions captured
- Open items and blockers
- Knowledge base persistence

## Workflow

### Phase 1: Gather Current State

Use MCP tools to collect context:

```
knowledge_recall(query="sprint plan, epic status, progress")
knowledge_search(query="completed items, blockers, risks")
task_list(limit=100)
agent_performance_stats()
```

Collect:
- Sprint information and progress
- Completed work and approvals
- In-progress items and blockers
- All decisions and architecture
- Team activity and metrics

### Phase 2: Analyze Session

Determine:
- Duration, scope, and completion %
- Team agents involved
- Quality metrics (tests, reviews, security)
- Bottlenecks and collaboration patterns

### Phase 3: Generate Handoff Document

**Executive Summary:**
- Epic, session date, duration
- Status and completion %
- Quick status (points by state)
- Next session recommendation

**Completed Items:**
- User stories with approval status
- Test coverage and code review
- Security clearance
- Files changed and integration status

**In-Progress Items:**
- Current state and % complete
- Blockers with mitigation
- Next steps
- Files being modified

**Not Started Items:**
- Prerequisites and dependencies
- Owners and schedules

**Blockers & Risks:**
- Severity, impact, mitigation
- Owner and resolution timeline

**Key Decisions:**
- Architecture decisions made
- Rationale and constraints
- Technology choices

**Code & Configuration:**
- Files ready for review
- Files in development
- Test coverage metrics
- Environment setup needed

**Next Steps:**
- Immediate actions (next 2-4 hours)
- Next session kickoff plan
- Critical handoff points
- Owner transition details

### Phase 4: Store in Knowledge Base

```
knowledge_store(
  text="Handoff: [epic] - Session [date]
Status: [% complete]
Completed: [items]
In Progress: [items]
Blockers: [list]
Next Owner: [name]",
  knowledge_type="decision",
  agent="handoff-agent"
)
```

### Phase 5: Generate Report

Output comprehensive handoff document with sections above plus references to knowledge base entries.

## Handoff Checklist

**Before Handing Off:**

- [ ] All in-progress work saved and committed
- [ ] All decisions stored in knowledge base
- [ ] All blockers documented with mitigations
- [ ] Code review comments addressed
- [ ] Test suite runs and all tests passing
- [ ] Coverage metrics documented
- [ ] Security review status recorded
- [ ] Architecture decisions captured
- [ ] Timeline and estimates updated
- [ ] Risks and issues documented

## Output Format

Return comprehensive handoff document with:
- Executive summary and metrics
- Completed work with approval status
- In-progress work with next steps
- Not started work with prerequisites
- All blockers with impact and mitigation
- Key architectural decisions made
- Code locations and test coverage
- Next session schedule and owner transition plan
- Knowledge base references
