# Coordinate: Coordination Patterns

## Trigger
`/coordinate <agents or task>`

Establishes coordination patterns between agents, manages dependencies, and aggregates results for cohesive outcomes.

## Purpose
The Coordinate skill handles:
- Parallel vs sequential execution decisions
- Dependency management between agents
- Knowledge base coordination
- Result aggregation and synthesis
- Conflict resolution
- Progress tracking

## Coordination Models

### Parallel Execution (Default)
**When**: Tasks are independent and can run simultaneously
**Benefits**: 2.8-4.4x faster than sequential execution
**Setup**: All agents spawned in ONE message with Task tool

**Use when:**
- Agent outputs don't depend on each other
- Peer review type work (code review + security review)
- Parallel implementation (frontend + backend)
- Documentation runs alongside development
- Testing validates development in parallel

**Time savings:**
- Feature work: 20-30 minutes vs 60+ minutes sequential
- All agents run simultaneously without blocking

### Sequential Execution
**When**: Output from one agent is input to another
**Usage**: Chain agents with blockedBy dependencies in Task tool

**Pattern:**
1. Chief Architect designs system (output: architecture)
   ↓
2. TDD Agent writes tests (input: architecture, output: test suite)
   ↓
3. Language Specialists implement (input: tests + architecture)
   ↓
4. Test Engineer validates (input: code + tests)
   ↓
5. Code Reviewer reviews (input: all code)
   ↓
6. Security Auditor does final review

**Use when:**
- Design before implementation (always)
- Tests before code (TDD pattern)
- Integration after components done
- Final review after implementation
- Deployment after all testing

### Hybrid Execution (Sequential + Parallel)
**Most efficient** - Combines both patterns

**Pattern:**
Sequential backbone (1-2 steps) → Parallel branches (3-5 agents) → Sequential merge (final review)

**Example:**
1. Chief Architect designs
2. TDD Agent writes tests
3. Parallel: Python Specialist + Frontend Developer + Documentation Writer
4. Sequential: Integration, Security, Validation

**Time savings:** 30-45 minutes instead of 60+ minutes.

## Dependency Management

### Using Task Tool for Dependencies

**Create independent tasks (parallel):**
```
task_create(title="Code Review", agent="code-reviewer")
task_create(title="Security Review", agent="security-auditor")
task_create(title="Test Validation", agent="test-engineer")
(No blockedBy specified = run in parallel)
```

**Create dependent tasks (sequential):**
```
task_create(title="Design System", agent="chief-architect")
task_create(title="Write Tests", agent="tdd-agent", dependencies=["design_task_id"])
task_create(title="Implement", agent="language-specialist", dependencies=["test_task_id"])
task_create(title="Review", agent="code-reviewer", dependencies=["impl_task_id"])
```

**Complex dependency DAG:**
Architect → Tests → Backend + Frontend + Docs (parallel) → Integration → Security → Final

## Knowledge Base Coordination

### Central Coordination Hub

Store shared decisions in knowledge base:

```
knowledge_store(text="Architecture Decision: [design details]",
  knowledge_type="decision", agent="chief-architect")

knowledge_store(text="Test Suite: [test locations]",
  knowledge_type="status", agent="tdd-agent")

knowledge_store(text="Progress: [agent] completed [task]",
  knowledge_type="status", agent=[agent-name])
```

### Knowledge-Based Synchronization

Agents coordinate by querying knowledge base:

1. All agents query knowledge base for shared decisions
   ```
   knowledge_recall(query="architecture decisions, shared constraints")
   knowledge_search(query="prerequisite outputs")
   ```

2. Each agent stores its output in knowledge base
   ```
   knowledge_store(text="Implementation: [details]", knowledge_type="status")
   ```

3. Dependent agents query for prerequisite outputs
4. Final review agent aggregates all results

### Example Flow

```
Chief Architect stores: "Use JWT for auth, store in DB, expire in 24h"
    ↓
TDD Agent queries, writes tests for JWT validation
    ↓
Security Auditor queries, reviews JWT security
    ↓
Python Specialist queries, implements JWT with FastAPI
    ↓
Code Reviewer queries, checks implementation vs architecture
    ↓
All results stored in knowledge base for reference
```

## Result Aggregation

### Two-Level Aggregation

**Level 1: Agent-Specific Storage**
- Code changes: File paths
- Test results: Coverage %
- Security findings: Severity levels
- Documentation: File locations

**Level 2: Conductor Aggregation**
Query knowledge base and merge:
1. Code changes into coherent changeset
2. Test results into single report
3. Security findings by severity
4. Documentation into complete docs
5. Create deployment plan

### Aggregation Workflow

```
Query Knowledge Base
    ↓
├─ Code Changes from Language Specialists
├─ Test Results from Test Engineer
├─ Security Findings from Security Auditor
├─ Performance Findings from Performance Engineer
├─ Documentation from Technical Writer
├─ Architecture Decisions from Chief Architect
└─ Review Comments from Code Reviewer
    ↓
Consolidate Results
    ↓
Generate Final Deliverables
```

## Conflict Resolution

### Architectural Conflicts
1. Detect conflict: Code Reviewer notices deviation from architecture
2. Escalate: Query knowledge base for architectural decision
3. Mediate: Architect Review agent (Sonnet) mediates
4. Resolve: Store authoritative decision in knowledge base
5. Implement: Language Specialist implements per decision
6. Verify: Code Reviewer confirms compliance

### Security Conflicts
1. Log: Security Auditor stores findings in knowledge base
2. Review: Chief Architect reviews severity and impact
3. Decide: Architect decides on remediation approach
4. Implement: Responsible agent implements fix
5. Verify: Security Auditor re-validates fix

### Performance Conflicts
1. Identify: Performance Engineer measures and reports gap
2. Analyze: Performance Profiler identifies bottlenecks
3. Plan: Chief Architect decides on optimization strategy
4. Implement: Language Specialist implements optimization
5. Validate: Performance Engineer confirms improvement

## Parallel vs Sequential Decision Matrix

| Scenario | Pattern | Reason |
|----------|---------|--------|
| Feature development with TDD | Hybrid | Tests sequential, implementation parallel |
| Code review + security review | Parallel | Peer reviews don't depend on each other |
| Bug fix workflow | Sequential | Diagnosis before fix, regression test before fix |
| New API design | Sequential | Architect first, then implementation |
| Performance optimization | Sequential | Profile first, then optimize, then validate |
| Documentation | Parallel | Can write docs while code development happens |
| Refactoring | Hybrid | Design sequentially, implementation parallel |
| Infrastructure setup | Sequential | Plan → implementation → validation → monitoring |
| Data pipeline | Sequential | Design → tests → implementation → validation |
| Mobile + backend | Hybrid | Architecture sequential, then parallel implementation |

## Monitoring Coordination

### Health Checks During Execution

```
knowledge_search(query="Progress")         # Check progress
knowledge_search(query="Blocker")          # Identify blockers
knowledge_search(query="In Progress")      # Monitor timeline
```

### Detecting Coordination Issues

**Issue: Agent blocked waiting for input**
→ Check knowledge base for prerequisite output
→ If missing, escalate to chief architect

**Issue: Conflicting implementations**
→ Query knowledge base for shared decision
→ If none exists, escalate to architect

**Issue: Agent failure**
→ Error Detective analyzes failure
→ Re-spawn agent or adjust team

## Best Practices

1. **Maximize parallelism** - Spawn independent agents together
2. **Minimize sequencing** - Only block when truly necessary
3. **Use knowledge base** - Central coordination hub for async sync
4. **Clear dependencies** - Document blockedBy relationships
5. **Early conflict detection** - Use code review to catch deviations early
6. **Progress tracking** - Monitor via knowledge base queries
7. **Escalate thoughtfully** - Use architect for conflicts
8. **Document decisions** - Store in knowledge base for reference

## Output Format

Return coordination plan with:
- Execution pattern (parallel/sequential/hybrid)
- Agent dependency graph
- Knowledge base coordination points
- Timeline with milestones
- Risk/blocker identification
- Escalation criteria
- Aggregation strategy
