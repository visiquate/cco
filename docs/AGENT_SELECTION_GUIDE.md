# Agent Selection Guide - Smart Agent Composition

## Overview

This guide helps you select the right agents for your project from the Claude Orchestra's **125 specialized agents**. Smart agent selection is key to efficient, high-quality development.

## Quick Selection Process

```
1. Identify project type (web app, mobile, API, ML, etc.)
   ↓
2. Determine complexity level (simple, moderate, complex, enterprise)
   ↓
3. Select core agents (always needed)
   ↓
4. Add specialized agents (based on requirements)
   ↓
5. Include support agents (QA, security, docs)
   ↓
6. Spawn all agents in parallel
```

## Core Agents (Always Include)

These agents should be part of almost every project:

| Agent | Type | Why Always Include |
|-------|------|-------------------|
| **Chief Architect** | backend-architect | Strategic decisions, coordinates agents |
| **TDD Coding Agent** | fullstack-developer | Ensures tests written before implementation |
| **QA Engineer** | test-automator | Integration testing, E2E tests, autonomous fixing |
| **Security Auditor** | security-auditor | Security review, vulnerability scanning |
| **Documentation Lead** | fullstack-developer | Technical documentation, keeps docs updated |

**Minimum viable team: 5 agents** (above list)

## Selection by Project Type

### 1. Web Application (Full Stack)

**Simple (6-8 agents):**
```javascript
Task("Chief Architect", "Design architecture", "backend-architect")
Task("TDD Coding Agent", "Write tests first", "fullstack-developer")
Task("Frontend Developer", "Build React UI", "fullstack-developer")
Task("Backend Architect", "Design REST API", "backend-architect")
Task("Database Architect", "Design schema", "backend-architect")
Task("QA Engineer", "Integration tests", "test-automator")
Task("Security Auditor", "Security review", "security-auditor")
Task("Documentation Lead", "API docs", "fullstack-developer")
```

**Production-Ready (12-15 agents):**
Add to simple:
- DevOps Engineer (CI/CD)
- Cloud Architect (AWS/GCP infrastructure)
- Monitoring Specialist (observability)
- Performance Engineer (optimization)
- API Documenter (OpenAPI/Swagger)
- Credential Manager (secrets management)
- Load Testing Specialist (performance testing)

**Enterprise (20+ agents):**
Add to production:
- Security Engineer (infrastructure security)
- Compliance Specialist (GDPR, SOC2)
- Network Engineer (VPC, routing)
- Incident Responder (on-call support)
- Business Analyst (KPI tracking)
- Technical Writer (user documentation)
- Content Marketer (launch materials)

### 2. Mobile Application

**iOS (7-9 agents):**
```javascript
Task("Chief Architect", "Design app architecture", "backend-architect")
Task("TDD Coding Agent", "Write tests first", "fullstack-developer")
Task("Swift Specialist", "Build SwiftUI app", "ios-developer")
Task("Backend Architect", "Design API", "backend-architect")
Task("API Explorer", "Explore third-party APIs", "technical-researcher")
Task("QA Engineer", "UI/integration tests", "test-automator")
Task("Security Auditor", "App security review", "security-auditor")
Task("UI/UX Designer", "Design user experience", "ui-ux-designer")
Task("Documentation Lead", "User guides", "fullstack-developer")
```

**Cross-Platform Flutter (8-10 agents):**
Replace Swift Specialist with:
- Flutter Specialist
- Mobile Developer (platform-specific features)

**With Backend (12-15 agents):**
Add backend specialists:
- Python/Go/Rust Specialist (API implementation)
- Database Architect
- DevOps Engineer
- Monitoring Specialist

### 3. API / Microservices

**Single API (6-8 agents):**
```javascript
Task("Chief Architect", "Design API architecture", "backend-architect")
Task("TDD Coding Agent", "Write tests first", "fullstack-developer")
Task("Backend Architect", "Design REST/GraphQL API", "backend-architect")
Task("Python Specialist", "FastAPI implementation", "python-pro")
Task("Database Architect", "Design data model", "backend-architect")
Task("QA Engineer", "API testing", "test-automator")
Task("Security Auditor", "API security", "security-auditor")
Task("API Documenter", "OpenAPI spec", "technical-writer")
```

**Microservices (15-20 agents):**
Add to single API:
- Go Specialist (high-performance services)
- Rust Specialist (performance-critical services)
- Cloud Architect (service mesh, K8s)
- DevOps Engineer (CI/CD pipeline)
- Monitoring Specialist (distributed tracing)
- API Security Audit (comprehensive security)
- Network Engineer (service routing)
- Database Optimizer (query performance)

**GraphQL API (add 2 agents):**
- GraphQL Architect
- GraphQL Performance Optimizer

### 4. Data Pipeline / ETL

**Basic Pipeline (8-10 agents):**
```javascript
Task("Chief Architect", "Design pipeline architecture", "backend-architect")
Task("TDD Coding Agent", "Write pipeline tests", "fullstack-developer")
Task("Data Engineer", "Build ETL pipeline", "fullstack-developer")
Task("Python Specialist", "Implement data processing", "python-pro")
Task("Database Architect", "Design data warehouse", "backend-architect")
Task("Cloud Architect", "AWS/GCP infrastructure", "backend-architect")
Task("QA Engineer", "Data quality tests", "test-automator")
Task("Monitoring Specialist", "Pipeline monitoring", "fullstack-developer")
Task("Security Auditor", "Data security review", "security-auditor")
Task("Documentation Lead", "Pipeline docs", "fullstack-developer")
```

**ML Pipeline (add 4-6 agents):**
- ML Engineer
- MLOps Engineer
- Data Scientist
- Model Evaluator
- AI Engineer (if LLM integration)

**Real-Time Streaming (add 3-4 agents):**
- Performance Engineer
- Database Optimizer
- Load Testing Specialist

### 5. AI/ML Project

**Model Training (10-12 agents):**
```javascript
Task("Chief Architect", "Design ML architecture", "backend-architect")
Task("TDD Coding Agent", "Write model tests", "fullstack-developer")
Task("Data Scientist", "Exploratory analysis", "technical-researcher")
Task("ML Engineer", "Model implementation", "fullstack-developer")
Task("MLOps Engineer", "Training pipeline", "deployment-engineer")
Task("Data Engineer", "Feature engineering", "fullstack-developer")
Task("Python Specialist", "Python implementation", "python-pro")
Task("Model Evaluator", "Model evaluation", "fullstack-developer")
Task("QA Engineer", "ML testing", "test-automator")
Task("Security Auditor", "Model security", "security-auditor")
Task("Documentation Lead", "Model documentation", "fullstack-developer")
Task("Technical Writer", "Model cards", "technical-writer")
```

**LLM Integration (add 3-5 agents):**
- AI Engineer
- Prompt Engineer
- API Explorer (LLM API integration)
- Backend Architect (API wrapper)

**Production ML (add 5-7 agents):**
- DevOps Engineer
- Cloud Architect
- Monitoring Specialist
- Performance Profiler
- Database Architect

### 6. API Integration Project

**Third-Party API (6-8 agents):**
```javascript
Task("Chief Architect", "Design integration", "backend-architect")
Task("TDD Coding Agent", "Write integration tests", "fullstack-developer")
Task("API Explorer", "Explore API endpoints", "technical-researcher")
Task("Backend Architect", "Design wrapper API", "backend-architect")
Task("Python Specialist", "Implement integration", "python-pro")
Task("QA Engineer", "Integration tests", "test-automator")
Task("Security Auditor", "API security", "security-auditor")
Task("Documentation Lead", "Integration docs", "fullstack-developer")
```

**Salesforce Integration (add 1 agent):**
- Salesforce API Specialist

**Authentik/OAuth Integration (add 1 agent):**
- Authentik API Specialist

**Multi-System Integration (add 3-5 agents):**
- Data Engineer (data transformation)
- Database Architect (sync strategy)
- Monitoring Specialist (integration health)

### 7. Research Project

**Literature Review (6-8 agents):**
```javascript
Task("Research Coordinator", "Coordinate research", "fullstack-developer")
Task("Academic Researcher", "Scholarly sources", "technical-researcher")
Task("Technical Researcher", "Technical papers", "technical-researcher")
Task("Fact Checker", "Verify claims", "fullstack-developer")
Task("Research Synthesizer", "Synthesize findings", "fullstack-developer")
Task("Report Generator", "Generate report", "fullstack-developer")
Task("Documentation Lead", "Format documentation", "fullstack-developer")
Task("Technical Writer", "Final editing", "technical-writer")
```

**Data Analysis Research (add 2-3 agents):**
- Data Analyst
- Data Scientist
- Business Analyst

**Technical Feasibility Study (add 3-5 agents):**
- Chief Architect
- Backend Architect
- Cloud Architect

### 8. Security Audit / Compliance

**Security Audit (8-10 agents):**
```javascript
Task("Chief Architect", "Review architecture", "backend-architect")
Task("Security Auditor", "Security assessment", "security-auditor")
Task("Security Engineer", "Infrastructure security", "fullstack-developer")
Task("Penetration Tester", "Penetration testing", "fullstack-developer")
Task("API Security Audit", "API security", "fullstack-developer")
Task("Compliance Specialist", "Compliance review", "fullstack-developer")
Task("Code Reviewer", "Code security review", "code-reviewer")
Task("QA Engineer", "Security testing", "test-automator")
Task("Documentation Lead", "Security docs", "fullstack-developer")
Task("Technical Writer", "Compliance reports", "technical-writer")
```

**Web Application Security (add 1-2 agents):**
- Web Accessibility Checker
- Frontend Developer (for XSS/CSRF fixes)

**GraphQL Security (add 1 agent):**
- GraphQL Security Specialist

### 9. DevOps / Infrastructure

**CI/CD Pipeline (8-10 agents):**
```javascript
Task("Chief Architect", "Design CI/CD architecture", "backend-architect")
Task("DevOps Engineer", "Implement CI/CD", "deployment-engineer")
Task("Cloud Architect", "Cloud infrastructure", "backend-architect")
Task("Terraform Specialist", "Infrastructure as code", "fullstack-developer")
Task("Security Engineer", "Pipeline security", "fullstack-developer")
Task("Monitoring Specialist", "Pipeline monitoring", "fullstack-developer")
Task("QA Engineer", "Pipeline tests", "test-automator")
Task("Network Engineer", "Network configuration", "fullstack-developer")
Task("Documentation Lead", "DevOps docs", "fullstack-developer")
Task("Deployment Engineer", "Deployment automation", "deployment-engineer")
```

**Kubernetes (add 2-3 agents):**
- Cloud Migration Specialist
- Load Testing Specialist

**Monitoring/Observability (add 2 agents):**
- Performance Profiler
- Incident Responder

### 10. MCP Server Development

**Basic MCP Server (8-10 agents):**
```javascript
Task("Chief Architect", "Design MCP architecture", "backend-architect")
Task("TDD Coding Agent", "Write protocol tests", "fullstack-developer")
Task("MCP Server Architect", "Design server", "fullstack-developer")
Task("MCP Protocol Specialist", "Protocol compliance", "fullstack-developer")
Task("Backend Architect", "Server architecture", "backend-architect")
Task("TypeScript Pro", "TypeScript implementation", "fullstack-developer")
Task("QA Engineer", "Protocol testing", "test-automator")
Task("Security Auditor", "Protocol security", "security-auditor")
Task("API Documenter", "Protocol docs", "technical-writer")
Task("Documentation Lead", "Server docs", "fullstack-developer")
```

**MCP Integration (add 2-3 agents):**
- MCP Integration Engineer
- API Explorer

**Production MCP (add 3-5 agents):**
- MCP Deployment Orchestrator
- MCP Security Auditor
- MCP Testing Engineer

## Complexity-Based Selection

### Simple Projects (5-8 agents)
**Characteristics:**
- Single technology stack
- Well-defined requirements
- Short timeline (days)
- No integrations

**Agent Selection:**
- Chief Architect
- TDD Coding Agent
- 1-2 language specialists
- QA Engineer
- Security Auditor
- Documentation Lead

### Moderate Projects (10-15 agents)
**Characteristics:**
- 2-3 technologies
- Some requirements ambiguity
- Medium timeline (weeks)
- 1-2 integrations

**Agent Selection:**
- Core agents (5)
- Additional language specialists (2-3)
- Integration specialists (1-2)
- DevOps Engineer
- Additional support (UI/UX, Performance)

### Complex Projects (15-25 agents)
**Characteristics:**
- Multiple technologies
- Evolving requirements
- Long timeline (months)
- Multiple integrations
- Production deployment

**Agent Selection:**
- Core agents (5)
- Full tech stack specialists (4-6)
- Integration specialists (2-4)
- Full DevOps team (3-4)
- Comprehensive support (QA, Security, Docs, UX)
- Monitoring and operations

### Enterprise Projects (25+ agents)
**Characteristics:**
- Enterprise-scale system
- Complex requirements
- Long-term project
- Multiple systems integration
- Compliance requirements
- High availability needs

**Agent Selection:**
- All relevant categories
- Multiple specialists per area
- Full security and compliance team
- Operations and monitoring team
- Business intelligence team
- Documentation and training team

## Decision Trees

### Web Application Decision Tree

```
Is it a web application?
├─ Yes → Frontend + Backend needed?
│  ├─ Yes (Full Stack)
│  │  ├─ Simple → 6-8 agents (Frontend, Backend, DB, QA, Security, Docs)
│  │  ├─ Production → Add DevOps, Cloud, Monitoring
│  │  └─ Enterprise → Add Security team, Compliance, Business
│  │
│  └─ No (API only)
│     ├─ Single API → 6-8 agents
│     └─ Microservices → 15-20 agents
│
└─ No → What type of project? (See other decision trees)
```

### Mobile Application Decision Tree

```
Is it a mobile application?
├─ Yes → Platform?
│  ├─ iOS → Swift Specialist + core team (7-9 agents)
│  ├─ Android → Mobile Developer + core team (7-9 agents)
│  ├─ Cross-platform → Flutter Specialist + core team (8-10 agents)
│  └─ Multiple platforms → Platform specialists for each (12-15 agents)
│
│  Backend needed?
│  ├─ Yes → Add Backend team (4-6 more agents)
│  └─ No → Continue with mobile-only
│
└─ No → What type of project?
```

### Data/ML Decision Tree

```
Is it a data or ML project?
├─ Yes → Type?
│  ├─ Data Pipeline
│  │  ├─ Basic ETL → 8-10 agents
│  │  ├─ Real-time → Add streaming specialists (3-4)
│  │  └─ ML Pipeline → Add ML team (4-6)
│  │
│  ├─ ML Model Development
│  │  ├─ Training → 10-12 agents
│  │  ├─ LLM Integration → Add AI team (3-5)
│  │  └─ Production → Add DevOps/Ops team (5-7)
│  │
│  └─ Data Analysis
│     └─ Research team (6-8 agents)
│
└─ No → What type of project?
```

### Integration Decision Tree

```
Is it an integration project?
├─ Yes → Integration type?
│  ├─ Third-party API → API Explorer + core (6-8 agents)
│  ├─ Salesforce → Add Salesforce Specialist
│  ├─ Authentik/OAuth → Add Authentik Specialist
│  └─ Multi-system → Add Data Engineer, DB Architect (3-5 more)
│
│  Data transformation needed?
│  ├─ Yes → Add Data Engineering team
│  └─ No → Continue with integration-only
│
└─ No → What type of project?
```

## Common Agent Combinations

### Combo 1: "Basic Web App"
```javascript
// 8 agents, covers 80% of web projects
Task("Chief Architect", "Design app", "backend-architect")
Task("TDD Coding Agent", "Tests first", "fullstack-developer")
Task("Frontend Developer", "React UI", "fullstack-developer")
Task("Backend Architect", "REST API", "backend-architect")
Task("Python Specialist", "FastAPI", "python-pro")
Task("Database Architect", "PostgreSQL schema", "backend-architect")
Task("QA Engineer", "Integration tests", "test-automator")
Task("Security Auditor", "Security review", "security-auditor")
```

### Combo 2: "Production Web App"
```javascript
// Add to Basic Web App (5 more = 13 total)
Task("DevOps Engineer", "CI/CD", "deployment-engineer")
Task("Cloud Architect", "AWS setup", "backend-architect")
Task("Monitoring Specialist", "Observability", "fullstack-developer")
Task("Documentation Lead", "API docs", "fullstack-developer")
Task("Credential Manager", "Secrets", "fullstack-developer")
```

### Combo 3: "Mobile + Backend"
```javascript
// 12 agents
Task("Chief Architect", "Design system", "backend-architect")
Task("TDD Coding Agent", "Tests first", "fullstack-developer")
Task("Flutter Specialist", "Cross-platform app", "mobile-developer")
Task("Go Specialist", "Backend API", "backend-architect")
Task("Database Architect", "Data model", "backend-architect")
Task("API Explorer", "Third-party APIs", "technical-researcher")
Task("QA Engineer", "Testing", "test-automator")
Task("Security Auditor", "Security", "security-auditor")
Task("DevOps Engineer", "Deployment", "deployment-engineer")
Task("UI/UX Designer", "User experience", "ui-ux-designer")
Task("Documentation Lead", "Documentation", "fullstack-developer")
Task("Credential Manager", "API keys", "fullstack-developer")
```

### Combo 4: "ML Pipeline"
```javascript
// 14 agents
Task("Chief Architect", "ML architecture", "backend-architect")
Task("TDD Coding Agent", "Tests", "fullstack-developer")
Task("Data Scientist", "Analysis", "technical-researcher")
Task("ML Engineer", "Model training", "fullstack-developer")
Task("MLOps Engineer", "ML pipeline", "deployment-engineer")
Task("Data Engineer", "Feature engineering", "fullstack-developer")
Task("Python Specialist", "Implementation", "python-pro")
Task("Model Evaluator", "Evaluation", "fullstack-developer")
Task("Cloud Architect", "ML infrastructure", "backend-architect")
Task("Database Architect", "Feature store", "backend-architect")
Task("QA Engineer", "ML testing", "test-automator")
Task("Security Auditor", "Model security", "security-auditor")
Task("Monitoring Specialist", "Model monitoring", "fullstack-developer")
Task("Documentation Lead", "ML docs", "fullstack-developer")
```

### Combo 5: "API Integration"
```javascript
// 10 agents (with Salesforce)
Task("Chief Architect", "Integration design", "backend-architect")
Task("TDD Coding Agent", "Tests", "fullstack-developer")
Task("API Explorer", "API exploration", "technical-researcher")
Task("Salesforce API Specialist", "Salesforce integration", "backend-architect")
Task("Backend Architect", "Wrapper API", "backend-architect")
Task("Python Specialist", "Implementation", "python-pro")
Task("Database Architect", "Sync strategy", "backend-architect")
Task("QA Engineer", "Integration tests", "test-automator")
Task("Security Auditor", "API security", "security-auditor")
Task("Documentation Lead", "Integration docs", "fullstack-developer")
```

## Anti-Patterns (What NOT to Do)

### ❌ Anti-Pattern 1: Too Few Agents
**Problem:** Using only 1-2 agents for complex projects
```javascript
// BAD: Only 2 agents for full-stack production app
Task("Python Specialist", "Build everything", "python-pro")
Task("QA Engineer", "Test everything", "test-automator")
```

**Solution:** Use appropriate team size (8-15 agents for production)

### ❌ Anti-Pattern 2: Too Many Agents
**Problem:** Using 20+ agents for simple task
```javascript
// BAD: 20 agents to fix a typo in documentation
Task("Chief Architect", "Review typo", "backend-architect")
Task("Documentation Lead", "Fix typo", "fullstack-developer")
// ... 18 more agents ...
```

**Solution:** Match agent count to complexity (5-8 for simple tasks)

### ❌ Anti-Pattern 3: Wrong Specialists
**Problem:** Using wrong language/framework specialists
```javascript
// BAD: Python specialist for Go project
Task("Python Specialist", "Build Go microservice", "python-pro")
```

**Solution:** Use Go Specialist or Backend Architect for Go projects

### ❌ Anti-Pattern 4: Missing Core Agents
**Problem:** Skipping essential agents (security, QA, docs)
```javascript
// BAD: No security or QA for production API
Task("Backend Architect", "Build API", "backend-architect")
Task("DevOps Engineer", "Deploy", "deployment-engineer")
```

**Solution:** Always include core agents (especially QA, Security)

### ❌ Anti-Pattern 5: Sequential Spawning
**Problem:** Spawning agents one at a time across multiple messages
```javascript
// BAD: Multiple messages
Message 1: Task("Agent 1", ...)
Message 2: Task("Agent 2", ...)
Message 3: Task("Agent 3", ...)
```

**Solution:** Spawn ALL agents in ONE message (parallel execution)

### ❌ Anti-Pattern 6: Duplicate Roles
**Problem:** Multiple agents doing the same thing
```javascript
// BAD: 3 agents all doing backend work
Task("Backend Architect", "Design API", "backend-architect")
Task("Python Specialist", "Design API", "python-pro")
Task("Go Specialist", "Design API", "backend-architect")
```

**Solution:** Clear role separation, one agent per responsibility

### ❌ Anti-Pattern 7: Missing Architect
**Problem:** Complex project without architectural oversight
```javascript
// BAD: 15 agents, no architect coordination
Task("Python Specialist", "Build backend", "python-pro")
Task("Frontend Developer", "Build UI", "fullstack-developer")
// ... 13 more agents, no coordination
```

**Solution:** Always include Chief Architect for 10+ agent teams

## Scaling Agent Teams

### Small Team (5-8 agents)
**When:** Simple projects, clear requirements, short timeline
**Composition:** Core + 1-3 specialists
**Coordination:** Minimal, direct communication

### Medium Team (10-15 agents)
**When:** Moderate complexity, some ambiguity, weeks timeline
**Composition:** Core + specialists + support
**Coordination:** Architect coordinates, Knowledge Manager

### Large Team (15-25 agents)
**When:** Complex project, multiple systems, months timeline
**Composition:** Full stack, multiple specialists, ops team
**Coordination:** Architect leads, structured phases

### Enterprise Team (25+ agents)
**When:** Enterprise scale, compliance, long-term
**Composition:** Multiple teams, full support, operations
**Coordination:** Hierarchical, detailed planning, checkpoints

## Agent Authority and Risk Management

### Low-Risk Decisions (All agents)
- Code formatting and style
- Variable/function naming
- Test strategies
- Documentation structure
- Minor version updates

### Medium-Risk Decisions (Architect approval)
- Technology choices within approved stack
- API endpoint design
- Database schema changes
- Security implementation approaches
- Performance optimization strategies

### High-Risk Decisions (User approval)
- Major architecture changes
- New external services/dependencies
- Breaking API changes
- Production deployments
- Cost-impacting infrastructure changes

## Troubleshooting Agent Selection

### Issue: "Project moving too slowly"
**Diagnosis:** Probably too few agents
**Solution:** Add more specialists in parallel

### Issue: "Too much coordination overhead"
**Diagnosis:** Too many agents for task complexity
**Solution:** Reduce to core team + key specialists

### Issue: "Agents stepping on each other"
**Diagnosis:** Unclear role boundaries
**Solution:** Better task descriptions, clear file ownership

### Issue: "Quality issues in production"
**Diagnosis:** Missing QA or Security Auditor
**Solution:** Always include both for production code

### Issue: "Inconsistent architecture"
**Diagnosis:** Missing Chief Architect
**Solution:** Add architect for 10+ agent teams

### Issue: "Poor documentation"
**Diagnosis:** Missing Documentation Lead or Technical Writer
**Solution:** Include documentation agents from start

## Quick Reference Table

| Project Type | Minimum Agents | Optimal Agents | Must-Have Specialists |
|-------------|----------------|----------------|----------------------|
| Web App (Simple) | 5 | 8 | Frontend, Backend, DB |
| Web App (Production) | 8 | 13 | + DevOps, Cloud, Monitoring |
| Mobile (iOS) | 5 | 9 | Swift, Backend, UI/UX |
| Mobile (Cross-platform) | 5 | 10 | Flutter, Backend, UI/UX |
| API/Microservices | 6 | 12 | Backend, DB, API Docs |
| Data Pipeline | 8 | 12 | Data Engineer, DB, Cloud |
| ML Project | 10 | 16 | Data Scientist, ML Eng, MLOps |
| Integration | 6 | 10 | API Explorer, relevant specialist |
| Security Audit | 8 | 12 | Security team, Compliance |
| DevOps/Infrastructure | 8 | 12 | DevOps, Cloud, Monitoring |
| MCP Server | 8 | 12 | MCP specialists, TypeScript |

## Templates for Common Scenarios

### Template 1: "Startup MVP"
**Goal:** Launch fast, production-ready
**Agents:** 10-12
- Chief Architect
- TDD Coding Agent
- Frontend Developer
- Backend Architect
- Python/Go Specialist
- Database Architect
- DevOps Engineer
- QA Engineer
- Security Auditor
- Documentation Lead
- UI/UX Designer (optional)
- Credential Manager

### Template 2: "Enterprise Feature Addition"
**Goal:** Add feature to existing system, full compliance
**Agents:** 15-20
- All Template 1 agents
- Code Reviewer
- Security Engineer
- Compliance Specialist
- Performance Engineer
- Monitoring Specialist
- Technical Writer
- Business Analyst
- Load Testing Specialist

### Template 3: "Open Source Library"
**Goal:** Public library with great docs and tests
**Agents:** 8-10
- Chief Architect
- TDD Coding Agent
- Language Specialist (relevant to library)
- QA Engineer
- Code Reviewer
- Security Auditor
- API Documenter
- Technical Writer
- Test Automator
- Documentation Lead

### Template 4: "Data Science Project"
**Goal:** ML model with production deployment
**Agents:** 14-16
- Chief Architect
- TDD Coding Agent
- Data Scientist
- ML Engineer
- MLOps Engineer
- Data Engineer
- Python Specialist
- Model Evaluator
- Cloud Architect
- Database Architect
- DevOps Engineer
- QA Engineer
- Security Auditor
- Monitoring Specialist
- Documentation Lead
- Technical Writer

## Best Practices Summary

1. **Always start with core agents** (Architect, TDD, QA, Security, Docs)
2. **Match team size to complexity** (5-8 simple, 10-15 moderate, 15-25 complex)
3. **Spawn all agents in parallel** (one message)
4. **Include specialists for your tech stack** (Python, Swift, Go, etc.)
5. **Add support agents** (DevOps, Monitoring, UI/UX as needed)
6. **Scale vertically** (more agents) or horizontally (more phases)
7. **Use Knowledge Manager** for coordination
8. **Include TodoWrite** for tracking (10-20 todos)
9. **Document authority levels** (who approves what)
10. **Review and adjust** after each project phase

## Further Reading

- **Full Roster**: [COMPREHENSIVE_ORCHESTRA_ROSTER.md](COMPREHENSIVE_ORCHESTRA_ROSTER.md)
- **Quick Reference**: [QUICK_AGENT_REFERENCE.md](QUICK_AGENT_REFERENCE.md)
- **Config File**: `/Users/brent/git/cc-orchestra/config/orchestra-config.json`
- **Usage Guide**: [ORCHESTRA_USAGE_GUIDE.md](ORCHESTRA_USAGE_GUIDE.md)

---

**Smart Selection = Efficient Development** | **125 Agents** | **Right Tool for Every Job**
