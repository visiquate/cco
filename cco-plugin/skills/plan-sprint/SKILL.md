# Plan Sprint: Sprint Planning

## Trigger
`/plan-sprint <epic or requirement>`

Breaks down an epic into tasks, estimates complexity, and assigns to appropriate agents.

## Purpose
The Plan Sprint skill provides:
- Epic decomposition into user stories
- User story breakdown into tasks
- Complexity estimation
- Agent assignment matching skills to tasks
- Sprint timeline planning
- Dependency identification

## Workflow

### Phase 1: Analyze Epic

1. Parse epic description and acceptance criteria
2. Assess complexity, technology stack, team skills, risks
3. Query knowledge base for context:
   ```
   knowledge_recall(query="epic requirements, architecture decisions")
   knowledge_search(query="technology stack, related features")
   ```

### Phase 2: Spawn Planning Team

Spawn **Task Decomposition Expert** (Haiku) + **Business Analyst** (Haiku) to:
- Break epic into user stories
- Break stories into tasks
- Identify dependencies
- Estimate effort and complexity

### Phase 3: Generate Sprint Plan

Create comprehensive breakdown:

**User Stories:**
- Title and acceptance criteria
- Effort points (3-21)
- Dependencies
- Assigned agent(s)

**Tasks Per Story:**
- Type (testing, implementation, review, docs)
- Owner agent
- Effort points
- Deliverable

**Store plan:**
```
knowledge_store(
  text="Sprint Plan: [epic]
User Stories: [count]
Total Points: [points]
Timeline: [hours]
By Phase: [day-by-day tasks]",
  knowledge_type="decision",
  agent="task-decomposition-expert"
)
```

### Phase 4: Create Task DAG

For each task:
```
task_create(
  title="[task name]",
  description="[requirements and acceptance criteria]",
  parent_id="[epic or parent story]",
  dependencies=[related_task_ids],
  agent="[assigned agent name]"
)
```

### Phase 5: Identify Dependencies

Visualize sequential and parallel opportunities:
- Design before implementation
- Tests before code (TDD)
- Documentation can start early
- Integration after components ready

## Complexity Scoring

| Points | Effort | Duration | Risk | Typical Task |
|--------|--------|----------|------|--------------|
| **3** | Trivial | 30 min - 1 hour | Very Low | Documentation, small fix |
| **5** | Small | 1-2 hours | Low | Simple feature, utility |
| **8** | Medium | 2-4 hours | Medium | Feature with tests |
| **13** | Large | 4-8 hours | High | Complex feature, integration |
| **21** | Very Large | 8+ hours | Very High | Break down more |

## Agent Assignment Guide

### For Implementation Tasks
- **Small (3-5 points)**: Language Specialist (Haiku)
- **Medium (8 points)**: Language Specialist (Haiku) with Code Reviewer (Sonnet)
- **Large (13 points)**: Backend Architect (Sonnet) + Language Specialist (Haiku)

### For Testing Tasks
- **Write tests**: TDD Agent (Sonnet)
- **Run tests**: Test Engineer (Sonnet)
- **Coverage analysis**: Test Engineer (Sonnet)

### For Review Tasks
- **Code review**: Code Reviewer (Sonnet)
- **Security review**: Security Auditor (Sonnet)
- **Architecture review**: Architect Review (Sonnet)

### For Documentation Tasks
- **API docs**: API Documenter (Haiku)
- **System docs**: Technical Writer (Haiku)
- **Complex docs**: Documentation Expert (Sonnet)

## Planning Patterns

**Simple Feature (1 story, 8 points):**
TDD Agent (2) → Language Specialist (5) → Code Reviewer (2) = 1 day

**Medium Feature (2-3 stories, 16-24 points):**
TDD (6) + Language Specialists (12) + Code Reviewer (4) + Security (2) = 1.5-2 days

**Large Epic (4-5 stories, 40+ points):**
Parallel teams by component type with sequential integration. 2-3 days with full team.

## Best Practices

1. Break epics down - No story > 13 points
2. Identify dependencies early
3. Assign to appropriate skill levels
4. Include testing - TDD from start
5. Reserve review time
6. Plan for security review
7. Store decisions in knowledge base
8. Track velocity for estimation
9. Leave 10-15% buffer for unknowns

## Output Format

Return comprehensive sprint plan with:
- Epic overview and user stories
- Acceptance criteria and effort estimates
- Tasks broken by story with agent assignments
- Dependency diagram and parallel opportunities
- Day-by-day or week-by-week timeline
- Risk assessment and mitigations
- Velocity baseline
- Knowledge base references
