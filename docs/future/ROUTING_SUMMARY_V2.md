# Claude Orchestra - Complete Routing Summary v2.0

**STATUS: PLANNED - FUTURE HYBRID ROUTING (NOT CURRENTLY IMPLEMENTED)**

## Overview
The Claude Orchestra **would include 16 specialized agents** with intelligent LLM routing between Claude API and Ollama (coder.visiquate.com) **when ccproxy is deployed**. Currently, all agents use Claude API exclusively.

## ✨ New in v2.0

### Added Agents:
1. **Technical Writer** - Architecture documentation and user guides (Claude)
2. **User Experience Designer** - UI/UX design and final validation (Claude)

### Routing Changes:
- **DevOps Engineer** now routes to Ollama (was Claude)
- **Documentation Lead** refocused on code-level docs (Ollama)
- **Technical Writer** handles high-level architecture docs (Claude)

---

## Complete Routing Table (16 Agents)

### 🟦 CLAUDE API (6 agents - 37.5%)

| # | Agent | Type | Model | Responsibilities |
|---|-------|------|-------|------------------|
| 1 | **Chief Architect** | system-architect | Opus 4.1 | Strategic decisions, architecture design, team coordination |
| 7 | **API Explorer** | researcher | Sonnet 4.5 | API exploration, integration analysis, endpoint testing |
| 11 | **Technical Writer** | technical-writer | Sonnet 4.5 | Architecture docs, user guides, system diagrams, how-tos |
| 12 | **User Experience Designer** | ux-designer | Sonnet 4.5 | UI/UX design, accessibility, usability testing, **final validation** |
| 13 | **QA Engineer** | test-automator | Sonnet 4.5 | Integration testing, E2E testing, quality assurance |
| 14 | **Security Auditor** | security-auditor | Sonnet 4.5 | Security reviews, vulnerability scanning, OWASP compliance |

### 🟧 OLLAMA (10 agents - 62.5%)

| # | Agent | Type | Model | Responsibilities |
|---|-------|------|-------|------------------|
| 2 | **Python Specialist** | python-expert | Qwen 2.5 Coder 32B | Python implementation (FastAPI, Django, ML/AI) |
| 3 | **Swift/iOS Specialist** | ios-developer | Qwen 2.5 Coder 32B | iOS development (SwiftUI, UIKit, Core Data) |
| 4 | **Go Specialist** | backend-dev | Qwen 2.5 Coder 32B | Go microservices, gRPC, concurrency |
| 5 | **Rust Specialist** | backend-dev | Qwen 2.5 Coder 32B | Systems programming, memory safety, WebAssembly |
| 6 | **Flutter Specialist** | mobile-developer | Qwen 2.5 Coder 32B | Cross-platform mobile apps, state management |
| 8 | **Salesforce API Specialist** | backend-dev | Qwen 2.5 Coder 32B | Salesforce REST/SOAP API, SOQL, OAuth 2.0 |
| 9 | **Authentik API Specialist** | backend-dev | Qwen 2.5 Coder 32B | OAuth2/OIDC, SAML, user provisioning, MFA |
| 10 | **Documentation Lead** | coder | Qwen 2.5 Coder 32B | Code comments, API docs, code examples, docstrings |
| 15 | **Credential Manager** | coder | Qwen 2.5 Coder 32B | Credential management implementation |
| 16 | **DevOps Engineer** | deployment-engineer | Qwen 2.5 Coder 32B | Docker, Kubernetes, CI/CD, Infrastructure as Code |

---

## Key Responsibilities by Agent

### Leadership & Strategy (Claude)

**Chief Architect** (Opus 4.1):
- Strategic decision-making
- Architecture design
- Technology stack selection
- Agent coordination
- Compaction management

### Design & Validation (Claude)

**User Experience Designer** (Sonnet 4.5):
- UI/UX design and mockups
- User flow analysis
- Accessibility compliance (WCAG 2.1 AA)
- Usability testing
- Mobile-first design review
- **Final quality validation before completion** ⚠️ Can block deployment
- Coordinates with QA on usability

**Technical Writer** (Sonnet 4.5):
- Architecture documentation
- System design diagrams
- User guides and tutorials
- How-to guides
- Conceptual documentation
- Integration guides
- Deployment guides

### Quality & Security (Claude)

**QA Engineer** (Sonnet 4.5):
- Integration test suites
- End-to-end testing
- Performance testing
- CI/CD pipeline tests
- Autonomous test fixing

**Security Auditor** (Sonnet 4.5):
- Code security review
- Vulnerability scanning
- OWASP compliance
- Authentication/authorization review
- Can block deployment

**API Explorer** (Sonnet 4.5):
- Explore third-party APIs
- Test endpoints and authentication
- Create integration POCs
- Analyze rate limits

### Implementation (Ollama)

**All 5 Language Specialists**:
- Python, Swift, Go, Rust, Flutter
- Full implementation of features
- Clean, well-documented code
- Follow architecture decisions

**API Integration Specialists**:
- Salesforce API integration
- Authentik authentication integration
- OAuth 2.0, SAML, OIDC flows

**Documentation Lead** (Ollama):
- Inline code comments
- API reference docs
- Code examples and snippets
- Function/method documentation
- Docstrings, JSDoc

**DevOps Engineer** (Ollama):
- Docker and docker-compose config
- Kubernetes manifests
- CI/CD pipeline setup
- Infrastructure as Code
- AWS infrastructure (ECS, ECR)
- Monitoring and logging

**Credential Manager** (Ollama):
- Credential storage implementation
- Secrets management code
- Environment variable handling
- Rotation tracking

---

## Workflow: How User Experience Designer Works

The **User Experience Designer** is integrated into the development workflow to ensure high standards:

### Before Implementation:
1. Reviews architecture decisions
2. Creates UI/UX mockups and wireframes
3. Defines user flows and journeys
4. Sets accessibility requirements

### During Implementation:
1. Monitors coding agent progress
2. Reviews implementations for UX compliance
3. Provides feedback on mobile-first design
4. Coordinates with QA on usability testing

### Before Completion (CRITICAL):
1. **Final validation checkpoint** - Reviews entire implementation
2. Checks accessibility compliance (WCAG 2.1 AA)
3. Validates user flows and interactions
4. Ensures mobile-first design standards
5. **Can block deployment** if standards not met
6. Provides final approval or requests changes

### The UX Designer ensures:
- ✅ High-quality user experience
- ✅ Accessibility compliance
- ✅ Mobile-first design
- ✅ Consistent UI/UX patterns
- ✅ Nothing goes out without final validation

---

## Documentation Split: Technical Writer vs Documentation Lead

### Technical Writer (Claude) handles:
- 📋 Architecture documentation
- 📊 System design diagrams
- 📖 User guides and tutorials
- 📝 How-to guides
- 🎓 Conceptual documentation
- 🔗 Integration guides
- 🚀 Deployment guides

**Why Claude?** High-level technical communication requires strategic thinking, clear explanations, and understanding of system architecture.

### Documentation Lead (Ollama) handles:
- 💻 Inline code comments
- 📚 API reference documentation
- 🔧 Code examples and snippets
- 📑 Function/method documentation
- 📖 Docstrings and JSDoc
- 📄 README code sections

**Why Ollama?** Code-level documentation benefits from understanding code syntax, patterns, and can be generated alongside implementation.

---

## Routing Statistics

### By Destination:
- **Claude API**: 6 agents (37.5%)
  - 1 Architect (Opus)
  - 5 Support/Design/QA agents (Sonnet)

- **Ollama**: 10 agents (62.5%)
  - 5 Language specialists
  - 2 API integration specialists
  - 3 Implementation support agents

### By Category:
| Category | Claude | Ollama | Total |
|----------|--------|--------|-------|
| **Leadership** | 1 | 0 | 1 |
| **Coding Specialists** | 0 | 5 | 5 |
| **Integration Specialists** | 1 | 2 | 3 |
| **Support Agents** | 4 | 3 | 7 |
| **TOTAL** | **6** | **10** | **16** |

### Cost Analysis:

**Before routing (all Claude)**:
- 16 agents × average tokens per task
- Estimated cost: $X per project

**After routing (62.5% on Ollama)**:
- 10 agents on self-hosted Ollama: $0
- 6 agents on Claude: Strategic/high-value only
- **Estimated savings: ~62.5% on coding tasks**
- **No quality compromise** - Qwen 2.5 Coder excels at implementation

---

## Testing Routing

### Test Individual Agent:
```bash
node src/llm-router.js route <agent-type> <task-type>

# Examples:
node src/llm-router.js route ux-designer "design interface"
node src/llm-router.js route technical-writer "write guide"
node src/llm-router.js route deployment-engineer "deploy"
```

### Test All Agents:
```bash
/tmp/test-all-16-agents.sh
```

### Make Live Request:
```bash
# Test Ollama endpoint
node src/llm-router.js call-coding-llm "Write a Python function..."

# View routing stats
node src/llm-router.js stats
```

---

## Version History

### v2.0 (2025-01-16):
- ➕ Added User Experience Designer (Claude)
- ➕ Added Technical Writer (Claude)
- ↔️ Moved DevOps Engineer to Ollama
- 📝 Split documentation responsibilities
- ✅ Final UX validation checkpoint
- 📊 16 total agents (was 14)

### v1.0 (2025-01-16):
- Initial routing implementation
- 14 agents (6 Claude, 8 Ollama)
- Qwen 2.5 Coder 32B model

---

## Recommendations

### Current Configuration Optimized For:
✅ **Strategic Work (Claude)**: Architecture, UX design, security, QA
✅ **Implementation (Ollama)**: All coding, DevOps config, code docs
✅ **Quality Gates**: UX Designer and Security Auditor can block deployment
✅ **Cost Efficiency**: 62.5% of agents on self-hosted Ollama

### UX Designer Integration:
- Always include UX Designer in projects with user interfaces
- Final validation happens before completion
- Ensures high standards and accessibility
- Can block deployment if UX requirements not met

### When to Adjust:
- If UX Designer blocks too frequently → Review design requirements earlier
- If DevOps work needs strategic planning → Consider hybrid approach
- If code docs need improvement → May move Documentation Lead to Claude

---

## Summary

The Claude Orchestra v2.0 provides:
- ✅ **16 specialized agents** covering all aspects of development
- ✅ **Intelligent routing** optimized for task type
- ✅ **Quality gates** with UX and Security validators
- ✅ **Cost savings** through strategic Ollama usage
- ✅ **High standards** with final UX validation
- ✅ **Complete documentation** with specialized writers

**Nothing goes out without meeting our high standards.** 🎯
