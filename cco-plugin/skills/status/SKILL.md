# Status: Status Check

## Trigger
`/status [epic or task]`

Queries progress tracking to show current state, blockers, completed items, and next steps.

## Purpose
The Status skill provides:
- Real-time progress visibility
- Blocker identification and impact assessment
- Agent workload and capacity tracking
- Timeline and velocity analysis
- Risk assessment and mitigations

## Workflow

### 1. Gather Status Information

**Using MCP tools:**
```
task_list(limit=100)                    # Get task progress
agent_performance_stats(agent_name=null) # Agent workload
knowledge_search(query="status")        # Status entries
knowledge_lifecycle_status()            # Knowledge health stats
```

### 2. Analyze Progress

**Calculate metrics:**
- Completion percentage (completed/total points)
- Velocity analysis (points/day actual vs planned)
- Timeline tracking (on schedule, behind, ahead)
- Agent utilization (capacity used)
- Blocker impact (points blocked, timeline impact)

### 3. Identify Blockers

**Search and categorize:**
```
knowledge_search(query="blocker")
knowledge_search(query="blocked")
knowledge_search(query="waiting")
```

For each blocker: status, severity, impact, mitigation, owner

### 4. Report Key Sections

**Executive Summary:**
- Epic/task name and progress %
- Overall status (ON_TRACK, BEHIND, AHEAD)
- Open blockers count
- Team size and velocity

**Progress by Work Item:**
- User story/task status
- Points completed/total
- Owner and expected completion
- Blockers if any

**Agent Status:**
- Agent name and model tier
- Tasks assigned/completed/in-progress
- Capacity utilization
- Concerns or risks

**Blockers & Risks:**
- Open blockers with impact
- Identified risks and mitigations
- Timeline variance analysis

**Recommendations:**
- Immediate actions needed
- Resource adjustments
- Risk mitigation steps

## Example Report Structure

```
PROJECT STATUS: JWT Authentication System
==========================================
Progress: 35% complete (35/100 points)
Status: BEHIND SCHEDULE (+0.6 days)
Open Blockers: 2 (Database schema, Security review)
Velocity: 10 pts/day (planned 13/day)

WORK BY STORY:
- Story 1: 8/8 points COMPLETE ✓
- Story 2: 5/8 points IN PROGRESS
- Story 3: 0/13 points BLOCKED (awaiting DB schema)

AGENT WORKLOAD:
- TDD Agent: 40% utilized ✓
- Python Specialist: 80% utilized ⚠️ (at capacity)
- Code Reviewer: 50% utilized ✓
- Security Auditor: 30% prioritizing review

CRITICAL ACTIONS:
1. Confirm database schema completion
2. Expedite security review of JWT logic
3. Consider backup Python specialist if velocity doesn't improve
```

## Best Practices

1. Update daily — capture current state
2. Be specific — point to exact blockers
3. Quantify progress — use actual metrics
4. Identify risks early — don't wait for crisis
5. Track all blockers — prevent gaps

## Output Format

Return comprehensive status report with:
- Executive summary (progress %, status, blockers)
- Progress by work item
- Agent workload and utilization
- All open blockers and impacts
- Risk assessment
- Velocity and timeline analysis
- Recommendations for next actions
