# Claude Orchestra Roster - TDD Edition

**Total Agents**: 15 (was 14, added TDD Coding Agent)
**Configuration**: TDD-aware two-phase pipeline with ccproxy routing
**Model Strategy**: Phase-based execution with memory-aware model loading
**Deployment**: https://coder.visiquate.com (Mac mini at 192.168.9.123)

---

## Agent Distribution by Model

### Claude API (1 Agent - Independent)
**Chief Architect** - Independent, runs before pipeline
- Primary Model: `claude-opus-4-1`
- Fallback Model: `claude-sonnet-4-5` (NOT 3.5)
- Not part of local model phases
- Direct Claude API access (not via ccproxy)

### Phase 1: qwen2.5-coder:32b-instruct (10 Agents)
**API Alias**: `claude-3-5-sonnet` via ccproxy
**Context**: 32k tokens | **Memory**: ~20GB
Specialized coding model for TDD and implementation

#### TDD Specialist (1) ⭐ NEW
1. **TDD Coding Agent**
   - Type: `coder`
   - Role: Write failing tests BEFORE any implementation
   - Responsibilities: Unit tests, fixtures, mocks, acceptance criteria

#### Coding Specialists (5)
2. **Python Expert** - `python-expert` type
3. **Swift Expert** - `ios-developer` type
4. **Go Expert** - `backend-dev` type
5. **Rust Expert** - `backend-dev` type
6. **Flutter Expert** - `mobile-developer` type

#### Integration Specialists (3)
7. **API Explorer** - `researcher` type
8. **Salesforce API Expert** - `backend-dev` type
9. **Authentik API Expert** - `backend-dev` type

#### DevOps (1)
10. **DevOps Engineer** - `deployment-engineer` type

### Phase 1: qwen-fast:latest (1 Agent)
**API Alias**: `claude-3-haiku` via ccproxy
**Context**: 32k tokens | **Memory**: ~5GB
Lightweight model for credential operations

11. **Credential Manager** - `coder` type
    - Runs in parallel with qwen2.5-coder agents
    - Total Phase 1 memory: ~25GB (both models loaded)

### Phase 2: qwen-quality-128k:latest (3 Agents)
**API Alias**: `gpt-4` via ccproxy
**Context**: 128k tokens | **Memory**: ~35GB
Deep reasoning model for quality and documentation

13. **QA Engineer**
    - Type: `test-automator`
    - Reviews TDD tests, adds edge cases, integration tests
    - Autonomous test fixing capability

14. **Security Auditor**
    - Type: `security-auditor`
    - Deep vulnerability analysis, threat modeling
    - Can block deployment for critical issues

15. **Documentation Lead**
    - Type: `coder`
    - Technical docs with architectural reasoning
    - API documentation with code examples

---

## Execution Timeline

```
Phase 0 (Independent):
  Chief Architect (Opus 4.1 → Sonnet 4.5 fallback) - 10 minutes
  Direct Claude API (not via ccproxy)

Phase 1 (Two models simultaneously):
  qwen2.5-coder (20GB) + qwen-fast (5GB) = 25GB total ✅
  - TDD + 9 coding/integration agents (qwen2.5-coder)
  - 1 credential agent (qwen-fast)
  Duration: 30 minutes

Model Swap:
  qwen2.5-coder unloads, qwen-quality-128k loads (~35GB)
  Duration: ~40 seconds (on-demand loading)

Phase 2 (Single model):
  qwen-quality-128k (35GB) ✅
  - QA, Security, Documentation (3 agents)
  Duration: 30 minutes

Total: ~70 minutes
Health checks: DISABLED (prevents model thrashing)
```

---

## Agent Responsibilities Matrix

| Agent | Ollama Model | API Alias | Phase | Primary Responsibility | Depends On |
|-------|--------------|-----------|-------|----------------------|------------|
| Chief Architect | (Claude API) | opus/sonnet-4.5 | 0 | System design & architecture | User requirements |
| **TDD Coding Agent** ⭐ | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Write failing tests first | Architecture |
| Python Expert | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Implement to pass tests | TDD tests |
| Swift Expert | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Implement to pass tests | TDD tests |
| Go Expert | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Implement to pass tests | TDD tests |
| Rust Expert | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Implement to pass tests | TDD tests |
| Flutter Expert | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Implement to pass tests | TDD tests |
| API Explorer | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Research integrations | Architecture |
| Salesforce API | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | CRM integration | TDD tests |
| Authentik API | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Auth implementation | TDD tests |
| DevOps Engineer | qwen2.5-coder:32b | claude-3-5-sonnet | 1 | Infrastructure setup | Architecture |
| Credential Manager | qwen-fast:latest | claude-3-haiku | 1 | Secrets management | All agents |
| QA Engineer | qwen-quality-128k | gpt-4 | 2 | Review & enhance tests | Phase 1 complete |
| Security Auditor | qwen-quality-128k | gpt-4 | 2 | Security analysis | Phase 1 complete |
| Documentation Lead | qwen-quality-128k | gpt-4 | 2 | Technical documentation | Phase 1 complete |

---

## Memory Keys by Agent

```yaml
chief-architect:
  writes: architect/decisions
  reads: user/requirements

tdd-coding-agent:
  writes: tdd/failing-tests/*
  reads: architect/decisions

python-expert:
  writes: coder/python/*
  reads:
    - tdd/failing-tests/python
    - architect/decisions

qa-engineer:
  writes: qa/review/*
  reads:
    - tdd/failing-tests/*
    - coder/implementation/*

security-auditor:
  writes: security/findings/*
  reads:
    - coder/implementation/*
    - architect/decisions

documentation-lead:
  writes: docs/*
  reads:
    - architect/decisions
    - coder/implementation/*
    - qa/review/*
```

---

## Agent Selection Guidelines

### Minimal API Project
- Chief Architect
- TDD Coding Agent
- Python Expert
- QA Engineer
- Security Auditor
- Credential Manager
**Total**: 6 agents

### Full-Stack Application
- Chief Architect
- TDD Coding Agent
- Python Expert (backend)
- Flutter Expert (mobile)
- Swift Expert (iOS native)
- Authentik API (auth)
- QA Engineer
- Security Auditor
- Documentation Lead
- DevOps Engineer
- Credential Manager
**Total**: 11 agents

### Enterprise Integration
- All 15 agents

---

## Coordination Protocols

### TDD Agent Protocol
```bash
# Must run FIRST in Phase 1 (logically)
# Retrieve architecture decisions from Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js search "architect decisions"

# Write comprehensive tests
# Store tests in Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js store \
  "Tests written: Comprehensive test suite for all features" \
  --type implementation --agent tdd-agent

# Notify other agents
node ~/git/cc-army/src/knowledge-manager.js store \
  "Status: Tests ready for implementation" \
  --type status --agent tdd-agent
```

### Coding Specialist Protocol
```bash
# Depends on TDD tests being available
# Retrieve test specifications from Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js search "tdd agent tests"
node ~/git/cc-army/src/knowledge-manager.js search "test specifications"

# Implement to make tests pass
pytest tests/  # Verify green

# Store implementation in Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js store \
  "Implementation complete: All tests passing" \
  --type implementation --agent coding-specialist
```

### QA Engineer Protocol (Phase 2)
```bash
# Reviews and enhances after implementation
# Retrieve tests and implementation from Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js search "tdd agent tests"
node ~/git/cc-army/src/knowledge-manager.js search "implementation"

# Add edge cases and integration tests
# Store QA enhancements in Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js store \
  "QA enhancements: Added edge cases and integration tests" \
  --type implementation --agent qa-engineer
```

---

## Key Differences from Original Army

1. **Added TDD Coding Agent** - 15th agent for test-first development
2. **Two-phase execution** - Phase 1 (qwen-fast) → Phase 2 (qwen-quality)
3. **Strict TDD workflow** - Tests MUST be written before code
4. **Model-aware scheduling** - Optimized for memory constraints
5. **Enhanced QA role** - Reviews tests rather than writing them

---

## Success Metrics

- **TDD Compliance**: 100% (all features have tests first)
- **Test Coverage**: ≥90% for critical components
- **Pipeline Speed**: <2 hours for full-stack projects
- **Quality Gates**: All tests green before Phase 2
- **Model Efficiency**: Only 1 model swap needed