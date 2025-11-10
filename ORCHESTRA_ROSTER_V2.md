# Claude Orchestra Roster v2.0

## 📊 Quick Stats
- **Total Agents**: 16 (was 14 in v1.0)
- **Claude API**: 6 agents (37.5%)
- **Ollama (coder.visiquate.com)**: 10 agents (62.5%)
- **Quality Gates**: UX Designer + Security Auditor can block deployment
- **Cost Savings**: ~62.5% through strategic routing

---

## 🎯 The Complete Army

### 1️⃣ LEADERSHIP (1 agent)

**Chief Architect** - 🟦 Claude (Opus 4.1)
- Strategic decision-making
- Architecture design and technology selection
- Agent coordination and compaction management
- Requirements discovery and specification
- Routes to Claude for strategic reasoning

---

### 2️⃣ CODING SPECIALISTS (5 agents)

All route to 🟧 Ollama (Qwen 2.5 Coder 32B)

**Python Specialist**
- FastAPI, Flask, Django
- Data processing and ML/AI integration
- Async/await patterns

**Swift/iOS Specialist**
- SwiftUI, UIKit, Core Data
- Combine framework
- iOS app architecture

**Go Specialist**
- Microservices and gRPC
- Concurrency patterns
- Cloud-native applications

**Rust Specialist**
- Systems programming
- Memory safety
- WebAssembly and async Rust

**Flutter Specialist**
- Cross-platform mobile apps
- State management (Bloc, Provider, Riverpod)
- Native integrations

---

### 3️⃣ INTEGRATION SPECIALISTS (3 agents)

**API Explorer** - 🟦 Claude (Sonnet 4.5)
- Third-party API exploration
- Endpoint testing and documentation
- Integration POCs and analysis

**Salesforce API Specialist** - 🟧 Ollama (Qwen 2.5 Coder 32B)
- Salesforce REST/SOAP/Bulk API
- SOQL query optimization
- OAuth 2.0 authentication
- Platform Events and Change Data Capture

**Authentik API Specialist** - 🟧 Ollama (Qwen 2.5 Coder 32B)
- OAuth2/OIDC flows
- SAML integration
- User provisioning and MFA
- Authentik webhooks

---

### 4️⃣ DESIGN & DOCUMENTATION (2 agents)

**Technical Writer** - 🟦 Claude (Sonnet 4.5) ✨ NEW
- Architecture documentation and system diagrams
- User guides and tutorials
- How-to guides and best practices
- Integration and deployment guides
- High-level conceptual documentation

**Documentation Lead** - 🟧 Ollama (Qwen 2.5 Coder 32B)
- Inline code comments and docstrings
- API reference documentation
- Code examples and snippets
- Function/method documentation (JSDoc, docstrings)
- Developer-focused code documentation

---

### 5️⃣ QUALITY ASSURANCE (3 agents)

**User Experience Designer** - 🟦 Claude (Sonnet 4.5) ✨ NEW
- UI/UX design and mockups
- User flow analysis and journey mapping
- Accessibility compliance (WCAG 2.1 AA)
- Usability testing and validation
- Mobile-first design review
- **⚠️ FINAL QUALITY VALIDATION - Can block deployment**

**QA Engineer** - 🟦 Claude (Sonnet 4.5)
- Integration and E2E test suites
- Performance testing
- CI/CD pipeline tests
- Autonomous test fixing
- Coordinates with UX Designer

**Security Auditor** - 🟦 Claude (Sonnet 4.5)
- Code security reviews
- Vulnerability scanning and OWASP compliance
- Authentication/authorization audits
- Dependency security audits
- **Can block deployment for security issues**

---

### 6️⃣ INFRASTRUCTURE & OPERATIONS (2 agents)

**DevOps Engineer** - 🟧 Ollama (Qwen 2.5 Coder 32B) ↔️ MOVED
- Docker and docker-compose configuration
- Kubernetes manifests and deployments
- CI/CD pipeline setup (GitHub Actions, GitLab CI)
- Infrastructure as Code (Terraform, CloudFormation)
- AWS infrastructure (ECS, ECR, CloudFormation)
- Monitoring and logging setup

**Credential Manager** - 🟧 Ollama (Qwen 2.5 Coder 32B)
- Credential storage implementation
- Secrets management and rotation
- Environment variable handling
- Secure retrieval mechanisms
- Coordinates with Security Auditor

---

## 🔄 Workflow Integration

### Standard Development Flow:

1. **Planning** - Chief Architect designs solution (Claude)
2. **Implementation** - Coding specialists build features (Ollama)
3. **Integration** - API specialists connect services (Mixed)
4. **Documentation** - Technical Writer writes guides (Claude)
5. **Code Docs** - Documentation Lead adds comments (Ollama)
6. **Infrastructure** - DevOps configures deployment (Ollama)
7. **Testing** - QA Engineer validates functionality (Claude)
8. **Security** - Security Auditor reviews code (Claude)
9. **UX Review** - UX Designer validates experience (Claude) ⚠️
10. **Deployment** - Only if all validations pass ✅

### Quality Gates:

**Before Deployment**, these agents must approve:
- ✅ QA Engineer - All tests passing
- ✅ Security Auditor - No critical vulnerabilities
- ✅ UX Designer - Meets accessibility and UX standards

**Either Security Auditor or UX Designer can block deployment.**

---

## 💰 Cost & Performance Analysis

### Ollama Endpoint (coder.visiquate.com):
- **Model**: Qwen 2.5 Coder 32B (Q4_K_M quantization)
- **Cost**: $0 (self-hosted)
- **Performance**: 14-75 seconds per response
- **Agents**: 10 (all implementation work)

### Claude API:
- **Models**: Opus 4.1, Sonnet 4.5
- **Cost**: Pay per token
- **Performance**: 2-10 seconds per response
- **Agents**: 6 (strategic and analytical work)

### Estimated Savings:
- **v1.0**: All 14 agents on Claude = 100% Claude costs
- **v2.0**: 10/16 agents on Ollama = **~62.5% cost reduction**
- **Quality**: No compromise - Qwen 2.5 excels at implementation

---

## 🚀 Using the Army

### Spawn All Agents (Claude Code):
```javascript
// Single message - spawn all 16 agents in parallel
Task("Chief Architect", "Analyze requirements and design...", "backend-architect", "opus")
Task("Python Specialist", "Implement features...", "python-pro", "sonnet")
Task("Swift Specialist", "Build iOS app...", "ios-developer", "sonnet")
Task("Go Specialist", "Create microservices...", "backend-architect", "sonnet")
Task("Rust Specialist", "Build systems code...", "backend-architect", "sonnet")
Task("Flutter Specialist", "Develop mobile app...", "mobile-developer", "sonnet")
Task("API Explorer", "Explore third-party APIs...", "technical-researcher", "sonnet")
Task("Salesforce Specialist", "Integrate Salesforce...", "backend-architect", "sonnet")
Task("Authentik Specialist", "Setup authentication...", "backend-architect", "sonnet")
Task("Documentation Lead", "Add code comments...", "fullstack-developer", "haiku")
Task("Technical Writer", "Write architecture docs...", "technical-writer", "sonnet")
Task("UX Designer", "Design and validate UX...", "ui-ux-designer", "sonnet")
Task("QA Engineer", "Create test suites...", "test-automator", "sonnet")
Task("Security Auditor", "Review security...", "security-auditor", "sonnet")
Task("Credential Manager", "Manage secrets...", "fullstack-developer", "haiku")
Task("DevOps Engineer", "Setup infrastructure...", "deployment-engineer", "sonnet")

TodoWrite({ todos: [10-20 tasks covering all phases] })
```

### Test Routing:
```bash
# View routing configuration
node src/llm-router.js stats

# Test specific agent routing
node src/llm-router.js route ux-designer "design interface"
node src/llm-router.js route deployment-engineer "deploy"
node src/llm-router.js route technical-writer "write guide"

# Test Ollama endpoint
node src/llm-router.js call-coding-llm "Write a Python function..."

# View all 16 agents
/tmp/test-all-16-agents.sh
```

---

## ⚙️ Configuration Files

- **Army Config**: `/Users/brent/git/cc-army/config/orchestra-config.json`
- **LLM Router**: `/Users/brent/git/cc-army/src/llm-router.js`
- **Orchestrator**: `/Users/brent/git/cc-army/src/orchestra-conductor.js`

---

## 📚 Documentation

- [LLM Routing Guide](docs/LLM_ROUTING_GUIDE.md) - How routing works
- [Routing Summary v2](docs/ROUTING_SUMMARY_V2.md) - Complete breakdown
- [Army Usage Guide](docs/ARMY_USAGE_GUIDE.md) - How to use the army
- [Quick Start](docs/QUICK_START.md) - Get started quickly

---

## ✨ What's New in v2.0

### Added:
- ✅ **User Experience Designer** - Final quality validation checkpoint
- ✅ **Technical Writer** - High-level architecture documentation

### Changed:
- ↔️ **DevOps Engineer** moved to Ollama (was Claude)
- 📝 **Documentation Lead** refocused on code-level docs
- 📊 **Total agents**: 16 (was 14)

### Improved:
- 🎯 **Quality gates** with UX and Security validators
- 📚 **Documentation split** between technical and code docs
- 💰 **Cost savings** increased to 62.5%
- ⚠️ **Final validation** ensures high standards

---

## 🎯 Key Principles

1. **Strategic work uses Claude** - Architecture, design, security, QA
2. **Implementation uses Ollama** - All coding, DevOps config, code docs
3. **Quality gates matter** - UX and Security can block deployment
4. **High standards enforced** - UX Designer ensures quality before completion
5. **Cost-effective scaling** - Self-hosted Ollama for bulk implementation
6. **No compromise on quality** - Qwen 2.5 Coder excels at implementation

---

**The Claude Orchestra v2.0: Nothing ships without meeting our high standards.** 🚀
