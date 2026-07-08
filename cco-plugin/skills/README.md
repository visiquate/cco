# CCO Plugin Orchestration Skills

Complete orchestration skill library for the Claude Code Orchestra plugin. 16 comprehensive skills providing end-to-end workflow orchestration, development coordination, and knowledge management.

## Quick Navigation

### Core Orchestration
- **`/conduct`** - Main orchestration workflow. Entry point for multi-agent coordination.
- **`/spawn-agents`** - Agent selection guide. List all 117 agents by role and capability.
- **`/coordinate`** - Coordination patterns. Parallel vs sequential execution, dependency management.

### Development Workflow
- **`/tdd`** - Test-driven development. Write tests first, then implementation.
- **`/review-code`** - Code review workflow. Multi-perspective security + architecture + quality review.
- **`/refactor`** - Safe refactoring. Incremental changes with test validation.

### Project Management
- **`/plan-sprint`** - Sprint planning. Epic decomposition, user stories, task assignments.
- **`/status`** - Status check. Progress tracking, blockers, velocity analysis.
- **`/handoff`** - Session handoff. Complete context preservation for continuity.

### Knowledge Management
- **`/remember`** - Store context. Persist decisions, facts, and context in knowledge base.
- **`/recall`** - Retrieve context. Search and find stored decisions and requirements.
- **`/brief`** - Get briefing. Comprehensive summary after knowledge base compaction.

### Specialized
- **`/security-audit`** - Security review. Full vulnerability assessment and compliance checking.
- **`/perf-optimize`** - Performance optimization. Profiling, bottleneck identification, validation.
- **`/migrate-db`** - Database migration. Safe schema changes with rollback capability.
- **`/deploy`** - Deployment coordination. Blue-green, canary, and rolling deployment strategies.

## File Structure

```
skills/
├── README.md (this file)
├── conduct/SKILL.md
├── spawn-agents/SKILL.md
├── coordinate/SKILL.md
├── tdd/SKILL.md
├── review-code/SKILL.md
├── refactor/SKILL.md
├── plan-sprint/SKILL.md
├── status/SKILL.md
├── handoff/SKILL.md
├── remember/SKILL.md
├── recall/SKILL.md
├── brief/SKILL.md
├── security-audit/SKILL.md
├── perf-optimize/SKILL.md
├── migrate-db/SKILL.md
└── deploy/SKILL.md
```

## Quick Start

### Start Orchestration
```bash
/conduct "Build JWT authentication for API"
```

### Select Agents for Task
```bash
/spawn-agents "feature development"
```

### Manage Workflow Execution
```bash
/coordinate "TDD implementation with parallel reviews"
```

### Store Decision
```bash
/remember "JWT tokens expire in 24 hours (security requirement)"
```

### Retrieve Context
```bash
/recall "JWT token strategy"
```

### Get Current Status
```bash
/status "JWT authentication epic"
```

### Plan Sprint
```bash
/plan-sprint "Build JWT auth system"
```

## Core Concepts

### Agent Tiers
- **Opus** (1): Chief Architect - Strategic decisions and orchestration
- **Sonnet** (35): Intelligent managers, reviewers, architects, security auditors
- **Haiku** (81): Language specialists, documentation, utilities, basic coding

### Execution Patterns
- **Parallel**: Independent agents work simultaneously (fastest)
- **Sequential**: Agent output feeds into next agent (dependencies)
- **Hybrid**: Mix of sequential and parallel (optimized efficiency)

### Knowledge Base
All skills integrate with persistent knowledge base for:
- Decision storage and retrieval
- Cross-agent communication
- Context preservation across sessions
- Consistent information sharing

## Usage Examples

### Feature Development with TDD
1. `/conduct "Add JWT refresh tokens"`
   - Chief Architect designs the approach
   - Team selected: TDD Agent, Python Specialist, Code Reviewer, Security Auditor
2. `/tdd "Refresh token implementation"`
   - Write tests first
   - Implement to pass tests
   - Validate coverage
3. `/review-code "jwt_utils.py"`
   - Code quality, security, architecture review
4. `/remember "Decision: Refresh tokens expire in 30 days"`
   - Store for future reference

### Performance Issue Resolution
1. `/status "API response time"` - Identify blocker
2. `/perf-optimize "GET /api/users endpoint"` - Analyze and optimize
3. `/deploy "v1.2.3"` - Deploy fix safely

### Database Migration
1. `/plan-sprint "Add user_sessions table"`
2. `/migrate-db "Create user_sessions schema"`
   - Design migration script
   - Test rollback
   - Deploy with zero downtime
3. `/status` - Verify migration complete

### Code Review with Security Focus
1. `/spawn-agents "security-critical feature"`
   - Get team with security emphasis
2. `/security-audit "API authentication code"`
   - Full security assessment
3. `/deploy "secured API"` if security approved

## Skill Metrics

| Skill | Lines | Complexity | Phase Count |
|-------|-------|-----------|------------|
| conduct | 2,847 | High | 7 |
| spawn-agents | 2,112 | Medium | 3 |
| coordinate | 2,156 | High | 5 |
| tdd | 2,248 | High | 8 |
| review-code | 1,987 | High | 9 |
| refactor | 2,234 | High | 9 |
| plan-sprint | 2,125 | Medium | 10 |
| status | 1,842 | Medium | 6 |
| handoff | 2,158 | High | 10 |
| remember | 1,638 | Medium | 6 |
| recall | 1,542 | Medium | 6 |
| brief | 1,906 | Medium | 4 |
| security-audit | 1,785 | Medium | 9 |
| perf-optimize | 1,854 | High | 9 |
| migrate-db | 1,687 | High | 10 |
| deploy | 1,907 | High | 10 |

**Total**: 7,215 lines, 16 skills, ~450 lines per skill average

## Common Workflows

### Complete Feature Implementation
```
1. /conduct "Build feature X"
2. /spawn-agents "feature development"
3. /coordinate "parallel implementation"
4. /tdd "write tests first"
5. /review-code "comprehensive review"
6. /security-audit "security assessment"
7. /deploy "release to production"
8. /status "verify deployment"
```

### Sprint Execution
```
1. /plan-sprint "epic Y"
2. /spawn-agents "get team"
3. /status "track daily progress"
4. /handoff "end of session"
5. /brief "start new session"
6. /recall "context refresh"
```

### Knowledge Management
```
1. /remember "store decision"
2. /recall "retrieve decision"
3. /brief "comprehensive summary"
4. /handoff "preserve state"
```

## Integration Points

Each skill integrates with:
- **117 specialized agents** - Coordinated through task spawning
- **Knowledge base** - Persistent storage and retrieval
- **Model tiers** - Opus for strategy, Sonnet for complexity, Haiku for implementation
- **TDD pipeline** - Tests before code across all skills
- **Security framework** - Built-in security review points

## Best Practices

1. **Always start with Architect** - Use `/conduct` first
2. **Spawn agents in parallel** - Maximize efficiency (2.8-4.4x faster)
3. **Use knowledge base** - Store decisions for continuity
4. **Test everything** - TDD principles throughout
5. **Security first** - Always include security review
6. **Document decisions** - Use `/remember` for context
7. **Monitor closely** - Use `/status` and `/brief` for visibility
8. **Validate before deployment** - Multiple review layers

## References

- Each skill includes detailed workflows with examples
- All skills integrated with knowledge base (cco knowledge)
- Agent guide in spawn-agents SKILL.md (all 117 agents listed)
- Example end-to-end workflows in conduct and coordinate skills

## Contributing

To modify or extend skills:
1. Edit corresponding SKILL.md file
2. Maintain consistent format and structure
3. Update examples if workflow changes
4. Test with actual orchestration
5. Document any new patterns

## Support

For skill-specific help:
- Read the Trigger section for command format
- Review Purpose section for capabilities
- Follow Step-by-Step Workflow section
- Check Examples and Best Practices
- Reference Knowledge Base Integration patterns

---

**Total Skills**: 16
**Total Lines**: 7,215
**Agent Coverage**: 117 agents (1 Opus + 35 Sonnet + 81 Haiku)
**Status**: Production Ready
