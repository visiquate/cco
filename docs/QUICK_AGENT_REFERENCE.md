# Quick Agent Reference - 125 Specialized Agents

## Quick Stats
- **Total Agents**: 125
- **Categories**: 13
- **Model**: All use `sonnet-4.5` (except Chief Architect: `opus`)
- **Config**: `/Users/brent/git/cc-army/config/orchestra-config.json`

## Agent Categories at a Glance

| Category | Count | Use When |
|----------|-------|----------|
| **Architect** | 1 | Strategic decisions, system design, coordination |
| **Coding** | 6 | TDD, language-specific implementation |
| **Integration** | 3 | API exploration, Salesforce, Authentik |
| **Development** | 25 | Specialized development tasks |
| **Data** | 9 | Database design, data engineering |
| **Infrastructure** | 10 | DevOps, cloud, deployment |
| **Security** | 8 | Security audits, compliance, pentesting |
| **AI/ML** | 6 | Machine learning, LLM integration |
| **MCP** | 6 | MCP server development |
| **Documentation** | 7 | Technical writing, API docs |
| **Research** | 10 | Research, fact-checking, analysis |
| **Support** | 30 | QA, testing, UX, operations |
| **Business** | 4 | Strategy, analysis, marketing |

## Most Commonly Used Agents

### Core Development (Every Project)
```javascript
Task("TDD Coding Agent", "Write tests first", "coder")
Task("Python Specialist", "Implement FastAPI", "python-expert")
Task("Frontend Developer", "Build React UI", "coder")
Task("Backend Architect", "Design API", "system-architect")
```

### Quality Assurance (Every Project)
```javascript
Task("QA Engineer", "Integration tests", "test-automator")
Task("Security Auditor", "Security review", "security-auditor")
Task("Code Reviewer", "Code review", "reviewer")
```

### Infrastructure (Production Projects)
```javascript
Task("DevOps Engineer", "CI/CD setup", "deployment-engineer")
Task("Cloud Architect", "AWS infrastructure", "system-architect")
Task("Monitoring Specialist", "Observability", "coder")
```

## Agent Selection by Task Type

### Web Application
- Frontend Developer
- Backend Architect
- Database Architect
- QA Engineer
- Security Auditor
- DevOps Engineer

### Mobile App
- Swift Specialist / Flutter Specialist
- Backend Architect
- API Explorer
- Mobile Developer
- QA Engineer

### Data Pipeline
- Data Engineer
- Database Architect
- Cloud Architect
- Monitoring Specialist

### API Integration
- API Explorer
- Salesforce API Specialist (if Salesforce)
- Authentik API Specialist (if auth)
- Backend Architect
- Security Auditor

### AI/ML Project
- AI Engineer
- ML Engineer
- MLOps Engineer
- Data Scientist
- Model Evaluator

### Research Project
- Technical Researcher
- Academic Researcher
- Research Synthesizer
- Fact Checker
- Report Generator

## Agent Types Reference

| Type | Purpose | Examples |
|------|---------|----------|
| `system-architect` | Architecture & design | Chief Architect, Backend Architect, Cloud Architect |
| `coder` | Implementation | TDD Agent, Frontend Dev, most specialists |
| `python-expert` | Python development | Python Specialist |
| `ios-developer` | iOS/Swift | Swift Specialist |
| `mobile-developer` | Mobile apps | Flutter Specialist, Mobile Developer |
| `backend-dev` | Backend services | Go Specialist, Rust Specialist |
| `deployment-engineer` | DevOps/Infrastructure | DevOps Engineer, MLOps Engineer |
| `security-auditor` | Security review | Security Auditor, Penetration Tester |
| `test-automator` | Testing & QA | QA Engineer, Test Engineer |
| `technical-writer` | Documentation | Technical Writer, API Documenter |
| `researcher` | Research & analysis | Researchers, Data Scientist |
| `reviewer` | Code review | Code Reviewer, Architect Review |
| `ux-designer` | UX/UI design | User Experience Designer |

## Model Routing (via ccproxy)

All agents use `model: "sonnet-4.5"` which routes to:

| Phase | API Alias | Ollama Model | Context | Agents |
|-------|-----------|--------------|---------|--------|
| Phase 1 | `claude-3-5-sonnet` | qwen2.5-coder:32b | 32k | 1-10 (coding) |
| Phase 1 | `claude-3-haiku` | qwen-fast:latest | 32k | 11 (credentials) |
| Phase 2 | `gpt-4` | qwen-quality-128k | 128k | 13-15 (QA, security, docs) |

**Exception**: Chief Architect uses real Claude API (Opus → Sonnet 4.5 fallback)

## Spawning Agents

### Single Agent
```javascript
Task("Agent Name", "Task description", "agent-type")
```

### Multiple Agents (Parallel)
```javascript
// ONE message with ALL agents
Task("Frontend Developer", "Build React UI...", "coder")
Task("Backend Architect", "Design API...", "system-architect")
Task("QA Engineer", "Create tests...", "test-automator")
Task("Security Auditor", "Review security...", "security-auditor")

TodoWrite({ todos: [
  {content: "Build UI", status: "in_progress", activeForm: "Building UI"},
  {content: "Design API", status: "in_progress", activeForm: "Designing API"},
  {content: "Create tests", status: "pending", activeForm: "Creating tests"},
  {content: "Security review", status: "pending", activeForm: "Reviewing security"}
]})
```

## Agent Authority Levels

### Low Risk (All agents can decide)
- Code formatting
- Minor version updates
- Test strategies
- File organization

### Medium Risk (Requires Architect approval)
- Technology choices within stack
- API design decisions
- Database schema changes
- Security approaches

### High Risk (Requires user approval)
- New external services
- Major architecture changes
- Breaking API changes
- Production deployments

## Tips for Effective Agent Usage

1. **Spawn in Parallel**: Always spawn related agents in ONE message
2. **Use TodoWrite**: Track progress with comprehensive todos
3. **Knowledge Manager**: Agents coordinate via persistent memory
4. **Right Tool**: Choose specialized agent for the task
5. **Architect First**: Complex projects should start with Chief Architect
6. **Security Always**: Include Security Auditor for production code
7. **Test Coverage**: Always include QA Engineer or Test Automator

## Finding the Right Agent

**Need help with...**
- **Languages**: Python/TypeScript/JavaScript/Go/Rust Pro
- **Frameworks**: Next.js/React/GraphQL specialists
- **Databases**: Database Architect, Optimizer, NoSQL Specialist
- **Cloud**: Cloud Architect, DevOps Engineer, Terraform Specialist
- **Security**: Security Auditor, Penetration Tester, API Security
- **AI/ML**: AI Engineer, ML Engineer, MLOps Engineer
- **Documentation**: Technical Writer, API Documenter
- **Research**: Technical/Academic Researcher, Fact Checker
- **Testing**: QA Engineer, Test Automator, Performance Profiler
- **MCP**: MCP Server Architect, Integration Engineer

## Documentation

- Full roster: `/Users/brent/git/cc-army/docs/COMPREHENSIVE_ORCHESTRA_ROSTER.md`
- Update summary: `/Users/brent/git/cc-army/docs/CONFIG_UPDATE_SUMMARY.md`
- Config file: `/Users/brent/git/cc-army/config/orchestra-config.json`

---

**Quick Reference** | **125 Agents** | **13 Categories** | **Complete Coverage**
