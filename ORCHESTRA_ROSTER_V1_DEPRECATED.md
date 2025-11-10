# Claude Orchestra Complete Roster

## 🎯 Total Force: 14 Specialized Agents

**🌐 Works from ANY directory!** The army automatically deploys when you describe complex tasks in Claude Code, regardless of which repository you're working in. Configuration lives here, agents operate in your current project directory.

---

## 🏗️ Command & Architecture (1 Agent)

### Chief Architect
- **Model:** Opus 4.1 (Advanced reasoning)
- **Type:** system-architect
- **Role:** Strategic decision-making and team coordination
- **Capabilities:**
  - System design and architecture
  - Technology stack selection
  - Agent task delegation
  - Requirements analysis
  - Final review and approval

---

## 💻 Coding Division (5 Agents)

### 1. Python Specialist
- **Model:** Sonnet
- **Type:** python-expert
- **Specialties:**
  - FastAPI / Flask / Django
  - Data processing
  - ML/AI integration
  - Async/await patterns
  - API development

### 2. Swift Specialist
- **Model:** Sonnet
- **Type:** ios-developer
- **Specialties:**
  - SwiftUI
  - UIKit
  - Core Data
  - Combine framework
  - iOS app architecture

### 3. Go Specialist
- **Model:** Sonnet
- **Type:** backend-dev
- **Specialties:**
  - Microservices
  - Concurrency patterns
  - gRPC
  - Cloud-native applications
  - Performance optimization

### 4. Rust Specialist
- **Model:** Sonnet
- **Type:** backend-dev
- **Specialties:**
  - Systems programming
  - Memory safety
  - Performance-critical code
  - WebAssembly
  - Async Rust

### 5. Flutter Specialist
- **Model:** Sonnet
- **Type:** mobile-developer
- **Specialties:**
  - Cross-platform mobile
  - State management (Provider, Riverpod, Bloc)
  - Native integrations
  - UI/UX implementation
  - Performance optimization

---

## 🔌 Integration Division (3 Agents)

### 1. API Explorer
- **Model:** Sonnet
- **Type:** researcher
- **Role:** General API exploration and integration analysis
- **Capabilities:**
  - REST and GraphQL API exploration
  - API authentication testing
  - OpenAPI/Swagger documentation
  - Rate limit analysis
  - Integration POC development
  - API client code generation
  - Change monitoring

### 2. Salesforce API Specialist
- **Model:** Sonnet
- **Type:** backend-dev
- **Role:** Salesforce integration expert
- **APIs Supported:**
  - REST API v59.0+
  - SOAP API
  - Bulk API 2.0
  - Streaming API
  - Metadata API
  - Tooling API
  - Analytics API
- **Capabilities:**
  - SOQL/SOSL query optimization
  - OAuth 2.0 with Salesforce
  - Bulk operations
  - Platform Events
  - Change Data Capture
  - Salesforce object mapping
  - Governor limit handling

### 3. Authentik API Specialist
- **Model:** Sonnet
- **Type:** backend-dev
- **Role:** Authentik authentication and API integration
- **APIs Supported:**
  - Core API (/api/v3/)
  - OAuth2 Provider API
  - SAML Provider API
  - LDAP Provider API
  - Flows API
  - Stages API
  - Events API
- **Capabilities:**
  - OAuth2/OIDC integration
  - SAML 2.0 configuration
  - User provisioning
  - Group management
  - MFA setup
  - Application providers
  - Policy engine integration
  - Event monitoring

---

## 🛠️ Support Division (5 Agents)

### 1. Documentation Lead
- **Model:** Haiku (Fast & efficient)
- **Type:** coder
- **Responsibilities:**
  - README files
  - API documentation
  - Code comments
  - Architecture diagrams
  - User guides
  - Setup instructions

### 2. QA Engineer
- **Model:** Sonnet
- **Type:** test-automator
- **Responsibilities:**
  - Integration test suites
  - End-to-end testing
  - Test automation
  - CI/CD pipeline tests
  - Performance testing
  - Test coverage reports

### 3. Security Auditor
- **Model:** Sonnet
- **Type:** security-auditor
- **Responsibilities:**
  - Code security review
  - Vulnerability scanning
  - OWASP compliance
  - Authentication/authorization review
  - Dependency audits
  - API security analysis
  - Security best practices
  - runs wiz code scans using wizcli, then remediates the findings: 'wizcli dir scan --no-publish --path .'

### 4. Credential Manager
- **Model:** Haiku (Fast & efficient)
- **Type:** coder
- **Responsibilities:**
  - Credential storage strategy
  - Secrets management (AWS Secrets Manager, etc.)
  - Environment variable handling
  - Credential rotation tracking
  - Security best practices
  - Credential inventory

### 5. DevOps Engineer
- **Model:** Sonnet
- **Type:** deployment-engineer
- **Specialties:**
  - Docker & Docker Compose
  - Kubernetes (EKS, GKE, AKS)
  - AWS (ECS, CloudFormation, Lambda, EC2)
  - CI/CD (GitHub Actions, GitLab CI)
  - Terraform & Infrastructure as Code
  - Monitoring (Prometheus, Grafana, CloudWatch)
  - Blue-green & canary deployments
  - Container security
  - Cost optimization

---

## 🎖️ Army Capabilities Matrix

| Capability | Agents Involved | Model Used |
|-----------|----------------|------------|
| System Architecture | Chief Architect | Opus 4.1 |
| Python Development | Python Specialist | Sonnet |
| iOS Development | Swift Specialist | Sonnet |
| Backend Services (Go) | Go Specialist | Sonnet |
| Systems Programming | Rust Specialist | Sonnet |
| Mobile Apps | Flutter Specialist | Sonnet |
| API Exploration | API Explorer | Sonnet |
| Salesforce Integration | Salesforce API Specialist | Sonnet |
| Authentication (Authentik) | Authentik API Specialist | Sonnet |
| Documentation | Documentation Lead | Haiku |
| Quality Assurance | QA Engineer | Sonnet |
| Security Auditing | Security Auditor | Sonnet |
| Credentials Management | Credential Manager | Haiku |
| DevOps & Deployment | DevOps Engineer | Sonnet |

---

## 🔄 Coordination & Communication

### MCP Servers
- **claude-flow@alpha** (Required): Core coordination, shared memory, hooks
- **ruv-swarm** (Recommended): Advanced Byzantine fault-tolerant coordination

### Coordination Topology
- **Type:** Hierarchical
- **Leader:** Chief Architect (Opus 4.1)
- **Memory Sharing:** Enabled
- **Consensus Required For:**
  - Architecture decisions
  - Technology selection
  - Security policies
  - API integration strategies

### Communication Protocol
All agents use hooks and shared memory:
1. **Pre-task**: Initialize with architect's decisions
2. **During**: Share progress via memory and hooks
3. **Post-task**: Report completion and store results

---

## 📊 Performance Characteristics

- **Total Agents:** 14
- **Parallel Execution:** All agents work concurrently
- **Speed Improvement:** 2.8-4.4x faster than sequential
- **Token Efficiency:** ~32% reduction via shared memory
- **Coordination Overhead:** Minimal with hooks
- **Model Distribution:**
  - 1 Opus 4.1 (Architect)
  - 11 Sonnet (Coding, Integration, Support - high quality)
  - 2 Haiku (Documentation, Credentials - fast & efficient)

---

## 🎯 Common Deployment Scenarios

### Scenario 1: Full-Stack App with Auth
**Request:** "Build a task app with Flutter frontend, Go backend, and Authentik auth"

**Agents Deployed:**
- Chief Architect
- Flutter Specialist
- Go Specialist
- Authentik API Specialist
- QA Engineer
- Security Auditor
- DevOps Engineer
- Documentation Lead
- Credential Manager

**Deliverables:** Complete app with auth, tests, docs, deployment

---

### Scenario 2: Salesforce Integration
**Request:** "Build a dashboard that displays live Salesforce Opportunity data"

**Agents Deployed:**
- Chief Architect
- Salesforce API Specialist
- Python/Go Specialist (backend service)
- Flutter Specialist (if mobile dashboard)
- API Explorer (explore needed endpoints)
- QA Engineer
- Security Auditor
- DevOps Engineer
- Documentation Lead
- Credential Manager

**Deliverables:** Dashboard app, Salesforce integration, real-time updates, deployment, docs

---

### Scenario 2b: Authentik Authentication
**Request:** "Add Authentik OAuth2 authentication to our web and mobile apps"

**Agents Deployed:**
- Chief Architect
- Authentik API Specialist
- Flutter Specialist (mobile OAuth2)
- Python/Go Specialist (web backend)
- QA Engineer
- Security Auditor
- DevOps Engineer
- Documentation Lead
- Credential Manager

**Deliverables:** OAuth2 integration for web and mobile, MFA support, deployment, docs

---

### Scenario 3: API Integration Project
**Request:** "Explore and integrate with [New Third-Party API]"

**Agents Deployed:**
- Chief Architect
- API Explorer (exploration and POC)
- Python/Go Specialist (implementation)
- QA Engineer
- Security Auditor
- Documentation Lead
- Credential Manager

**Deliverables:** API client, integration, tests, security review, docs

---

### Scenario 4: Microservices Deployment
**Request:** "Build microservices in Python and Go, deploy to Kubernetes on AWS"

**Agents Deployed:**
- Chief Architect
- Python Specialist
- Go Specialist
- DevOps Engineer (K8s, AWS)
- QA Engineer
- Security Auditor
- Documentation Lead
- Credential Manager

**Deliverables:** Services, K8s manifests, CI/CD, monitoring, docs

---

## 🚀 Quick Deployment

In Claude Code, simply describe what you want:

**Simple:**
```
"Add Authentik authentication to my Python API"
```

**Medium:**
```
"Sync users from Authentik to Salesforce Contacts with group mapping"
```

**Complex:**
```
"Build a full-stack application with Flutter mobile app, Go microservices,
Authentik authentication, Salesforce integration, deployed to AWS ECS
with monitoring and documentation"
```

Claude Code will:
1. Initialize MCP coordination (hierarchical, 14 agents max)
2. Spawn ALL relevant agents in parallel (one message)
3. Architect designs the system
4. Coding agents implement
5. Integration agents handle APIs
6. Support agents ensure quality
7. Everything coordinated via shared memory

---

## 📚 Documentation

- **Main README**: [README.md](README.md)
- **Usage Guide**: [docs/ARMY_USAGE_GUIDE.md](docs/ARMY_USAGE_GUIDE.md)
- **Quick Start**: [docs/QUICK_START.md](docs/QUICK_START.md)
- **API Integration**: [docs/API_INTEGRATION_GUIDE.md](docs/API_INTEGRATION_GUIDE.md)
- **DevOps Guide**: [docs/DEVOPS_AGENT_GUIDE.md](docs/DEVOPS_AGENT_GUIDE.md)
- **Example Workflow**: [docs/EXAMPLE_WORKFLOW.md](docs/EXAMPLE_WORKFLOW.md)
- **Configuration**: [config/orchestra-config.json](config/orchestra-config.json)

---

## 🎖️ Army Philosophy

**"Specialized Experts, Unified Coordination"**

Each agent is a specialist in their domain, but all work together through:
- Shared memory for context
- Hooks for coordination
- Hierarchical leadership
- Parallel execution
- Continuous communication

The result: **Production-ready code with built-in quality, security, testing, documentation, and deployment** - all developed 3-4x faster than traditional sequential development.

---

**Your Claude Orchestra is ready to deploy!** 🤖⚔️
