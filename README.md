# Claude Orchestra

A sophisticated multi-agent development system powered by Claude Code, featuring **119 specialized agents** organized across **13 functional sections** in a TDD-aware pipeline.

## Overview

The Claude Orchestra is a complete development team on demand:

- **1 Chief Architect** (Claude Opus 4.1) - Strategic decisions and coordination
- **37 Intelligent Managers** (Sonnet 4.5) - Complex reasoning, code review, security, testing, architecture
- **81 Basic Specialists** (Haiku 4.5) - Language coders, documentation, utilities, lightweight tasks (68% cost savings)
- **13 Functional Sections** - Developers, architects, security auditors, testers, researchers, writers, and more

**Key Statistics:**
- **Total Agents**: 119 (1 Chief Architect + 118 specialized agents)
- **Model Distribution**: Optimized by role complexity (Opus for leadership, Sonnet for intelligence, Haiku for basic tasks)
- **Cost Efficiency**: 68% of agents use cost-effective Haiku 4.5
- **Type Specialization**: Agents assigned to specific types based on role requirements
- **ccproxy**: Future enhancement (hardware pending) - currently using direct Claude API

All agents work in parallel using Claude Code's Task tool with Knowledge Manager coordination for efficient, high-quality development.

## Features

✨ **Hierarchical Coordination** - Architect-led team structure with intelligent task delegation
⚡ **Parallel Execution** - All agents work concurrently for 2.8-4.4x speed improvement
🧠 **Shared Knowledge** - Cross-agent communication via Knowledge Manager with LanceDB vector search
🔒 **Security First** - Built-in security auditing and secure credential management
🎯 **Specialized Experts** - Language-specific agents with deep domain knowledge
📊 **Quality Assurance** - Automated integration testing and code review
📚 **Auto Documentation** - Parallel documentation generation as code is written
🚀 **DevOps Ready** - Automated builds, deployments, and infrastructure as code
💾 **Persistent Memory** - Knowledge base survives conversation compactions
💰 **Model Override** - Transparent cost optimization (73% savings via Sonnet→Haiku routing)
📊 **Cost Monitoring** - Background daemon with TUI dashboard for real-time API cost tracking

## Quick Start

### 1. Prerequisites

The Claude Orchestra uses the built-in Knowledge Manager for coordination. No additional installation needed!

### 2. Deploy Your Orchestra

In Claude Code, simply describe what you want to build:

```
"Build a REST API with user authentication in Python"
```

Claude Code will automatically:
1. Initialize MCP coordination topology
2. Spawn all relevant agents in parallel
3. Architect analyzes and designs the system
4. Coding agents implement features
5. QA creates and runs tests
6. Security audits the code
7. Documentation is generated
8. Credentials are managed securely

## 🌐 Cross-Repository Usage

**The Claude Orchestra works from ANY directory!** You don't need to be in the cc-orchestra repository to use it.

### How It Works

1. **Orchestra configuration** lives in `/Users/brent/git/cc-orchestra/`
2. **Orchestra operates** in your current working directory (wherever you invoke Claude Code)
3. **Auto-detection** triggers orchestra based on task complexity (configured in global `~/.claude/CLAUDE.md`)

### Usage from Any Project

```bash
# Navigate to ANY project directory
cd ~/git/my-awesome-project

# Open Claude Code and describe what you want
# The orchestra automatically deploys if task matches trigger patterns
```

**Example:**
```
You: "Build a Python API with JWT authentication and deploy with Docker"

Claude Code:
✓ Detects complex multi-technology task
✓ Loads orchestra config from cc-orchestra directory
✓ Spawns agents in YOUR project directory
✓ Agents create files in YOUR project
✓ Coordination via Knowledge Manager
```

### Trigger Patterns (Auto-Activation)

The orchestra **automatically activates** for:
- Full-stack applications ("Build a mobile app with backend")
- Multi-technology projects ("Create with Python and Go")
- Complex features ("API with Salesforce integration")
- DevOps tasks ("Deploy to AWS with Kubernetes")
- Enterprise integrations ("Set up Authentik authentication")
- Production systems ("Build with tests, security, and monitoring")

The orchestra **bypasses** for simple tasks:
- Single-file changes ("Fix typo in README")
- Simple queries ("What does this function do?")
- Basic operations ("Run tests", "Check git status")
- Small additions ("Add a comment", "Update .gitignore")

### Explicit Invocation

You can always explicitly request the orchestra:
```
"Use the Claude Orchestra to build this feature"
"Deploy the full orchestra for this task"
```

### Project-Specific Customization

Create a `CLAUDE.md` in your project root to customize orchestra behavior:

```bash
# Copy template to your project
cp /Users/brent/git/cc-orchestra/docs/PROJECT_CLAUDE_TEMPLATE.md ~/git/your-project/CLAUDE.md

# Edit to customize agent selection, trigger patterns, tech stack
```

**Example project CLAUDE.md:**
```markdown
## Claude Orchestra Configuration

### Agent Preferences
- [x] Python Expert - All backend work
- [x] Flutter Expert - Mobile app
- [x] Authentik API Expert - OAuth2 integration
- [x] All support agents (Docs, QA, Security, DevOps, Credentials)

### Technology Stack
- Backend: Python 3.11 + FastAPI
- Frontend: Flutter 3.x
- Auth: Authentik OAuth2
- Deployment: AWS ECS
```

### Benefits

✅ **No context switching** - Orchestra works wherever you are
✅ **Automatic detection** - Smart activation based on task complexity
✅ **Project-specific** - Customize per project with local CLAUDE.md
✅ **Consistent quality** - Same high standards across all projects
✅ **Parallel efficiency** - 2.8-4.4x faster than sequential development

### Reference

See [docs/PROJECT_CLAUDE_TEMPLATE.md](docs/PROJECT_CLAUDE_TEMPLATE.md) for project customization guide.

## Agent Roles

### Chief Architect (Opus 4.1)
- Analyzes user requirements
- Makes architecture decisions
- Coordinates all agents
- Reviews final outputs
- Ensures quality standards

### Coding Specialists (Sonnet)
- **Python Expert**: FastAPI, Django, ML/AI, async patterns
- **Swift Expert**: SwiftUI, UIKit, iOS development
- **Go Expert**: Microservices, concurrency, cloud-native
- **Rust Expert**: Systems programming, performance, WebAssembly
- **Flutter Expert**: Cross-platform mobile, state management

### Integration Specialists
- **API Explorer** (Sonnet): REST/GraphQL API exploration, documentation, testing
- **Salesforce API Expert** (Sonnet): Salesforce REST/SOAP API, SOQL, OAuth, Bulk operations
- **Authentik API Expert** (Sonnet): OAuth2/OIDC, user provisioning, SAML, MFA

### Support Agents
- **Documentation Lead** (Haiku): Technical docs, API documentation
- **QA Engineer** (Sonnet): Integration tests, E2E testing
- **Security Auditor** (Sonnet): Vulnerability scanning, OWASP compliance
- **Credential Manager** (Haiku): Secure secrets management
- **DevOps Engineer** (Sonnet): Docker, Kubernetes, AWS, CI/CD, deployments

## Usage Examples

### Simple Feature
```
User: "Add JWT authentication to the Python API"

Orchestra Response:
- Architect designs auth flow
- Python expert implements JWT
- Security audits implementation
- QA writes integration tests
- Docs updates API documentation
- Credentials manages JWT secret
```

### Complex Multi-Language Project
```
User: "Build a mobile app (Flutter) with Go backend and Python ML service"

Orchestra Response:
- Architect designs 3-tier architecture
- Flutter expert builds mobile UI
- Go expert creates REST API
- Python expert implements ML inference
- QA tests full system integration
- Security reviews all components
- Docs creates system documentation
- Credentials manages all API keys
```

## Credential Management

Secure credential storage and retrieval via OS keyring integration:

```bash
# Store credentials with metadata
cco credentials store db_password "secret123" --credential-type database --description "Primary database password"

# Retrieve credentials
cco credentials retrieve db_password

# List all credentials
cco credentials list

# Check rotation status
cco credentials check-rotation

# Delete a credential
cco credentials delete db_password
```

Security Features:
- **OS Keyring Integration**: Credentials stored in macOS Keychain, Linux Secret Service, or Windows DPAPI
- **FIPS 140-2 Compliant Encryption**: AES-256-GCM for superior security
- **Secure Memory Management**: SecretString zeroization prevents memory attacks
- **Comprehensive Audit Logging**: All operations logged with full traceability
- **Rate Limiting**: 10 attempts per 60 seconds prevents brute-force attacks
- **Automatic Rotation Tracking**: Configurable per-credential rotation policies
- **Zero File-Based Vulnerabilities**: No temporary files or exposed credentials
- **Never stored in git**: Credentials managed through daemon only

## Architecture

```
┌─────────────────────────────────────────────────────┐
│           Chief Architect (Opus 4.1)                │
│       Strategic Decisions & Coordination             │
└───────────────────┬─────────────────────────────────┘
                    │
        ┌───────────┴────────────┐
        │                        │
┌───────▼───────┐      ┌────────▼────────┐
│ Coding Agents │      │  Support Agents  │
├───────────────┤      ├─────────────────┤
│ • Python      │      │ • Documentation │
│ • Swift       │      │ • QA/Testing    │
│ • Go          │      │ • Security      │
│ • Rust        │      │ • Credentials   │
│ • Flutter     │      │                 │
└───────────────┘      └─────────────────┘
```

## Coordination Protocol

Every agent follows this workflow:

**Before Work:**
```bash
# Review knowledge base for relevant context
cco knowledge search "task keywords"
cco knowledge search "architect decisions"

# Check credential inventory
cco credentials list
```

**During Work:**
```bash
# Update knowledge base with progress
cco knowledge store "Edit: [filename] - [changes]" --type edit --agent [agent-name]
cco knowledge store "Progress: [status]" --type status --agent [agent-name]

# Manage credentials securely
cco credentials store [key] [value] --credential-type [type] --service [service]
```

**After Work:**
```bash
# Document task completion
cco knowledge store "Task complete: [task]" --type completion --agent [agent-name]

# Verify credential rotation status
cco credentials check-rotation
```

## Performance

- **Agent Spawn**: Instant (parallel execution)
- **Speed**: 2.8-4.4x faster than sequential
- **Token Reduction**: ~32% with Knowledge Manager
- **Coordination Overhead**: Minimal with built-in coordination

## Model Override Feature

Save 73% on LLM costs with transparent model rewriting:

```bash
# Enable cost optimization (Sonnet → Haiku routing)
export ANTHROPIC_API_BASE_URL=http://localhost:3000/v1
./cco run --port 3000

# Claude Code continues to work normally, but pays Haiku prices!
```

**Cost Savings Example:**
- Small team (5 agents, 50 runs/month): Save **$26/month** ($318/year)
- Medium team (10 agents, 300 runs/month): Save **$693/month** ($8,316/year)
- Large deployment (50 agents, 1000 runs/month): Save **$6,476/month** ($77,712/year)

**Documentation:**
- **[User Guide](docs/MODEL_OVERRIDE_USER_GUIDE.md)** - How to use model overrides
- **[Operator Guide](docs/MODEL_OVERRIDE_OPERATOR_GUIDE.md)** - Deploy and manage in production
- **[Configuration Reference](docs/MODEL_OVERRIDE_CONFIG_REFERENCE.md)** - All configuration options
- **[Cost Analysis](docs/COST_ANALYSIS.md)** - Detailed cost calculations and ROI
- **[Migration Guide](docs/MIGRATE_TO_MODEL_OVERRIDE.md)** - Enable overrides in existing deployments
- **[API Documentation](docs/API.md)** - Integration and monitoring endpoints

## Cost Monitoring System

Architected as a lightweight background daemon with a Terminal User Interface (TUI) dashboard for real-time API cost tracking.

### Future Features

- **Real-Time Cost Monitor**: Live tracking of API costs as requests are made
- **Cost Summary**: Aggregate costs, token counts, and per-model tier breakdowns
- **Historical Tracking**: SQLite-based persistence for long-term metrics analysis
- **Cross-Platform**: Native executables for macOS and Windows
- **Self-Contained**: No external dependencies—single binary deployment

## Documentation

- **[Orchestra Roster](ORCHESTRA_ROSTER.md)** - Complete agent specifications and capabilities
- **[Quick Start](docs/QUICK_START.md)** - Get started quickly with examples
- **[Usage Guide](docs/ORCHESTRA_USAGE_GUIDE.md)** - Comprehensive usage instructions
- **[API Integration Guide](docs/API_INTEGRATION_GUIDE.md)** - Salesforce and Authentik integration
- **[DevOps Guide](docs/DEVOPS_AGENT_GUIDE.md)** - Infrastructure and deployment
- **[Example Workflow](docs/EXAMPLE_WORKFLOW.md)** - Full example workflow
- **[Configuration](config/orchestra-config.json)** - Agent configuration and roles
- **[CLAUDE.md](CLAUDE.md)** - Instructions for Claude Code

## Best Practices

1. ✅ Always spawn agents in parallel (one message)
2. ✅ Let the Architect lead the design
3. ✅ Use Knowledge Manager for coordination
4. ✅ Store decisions in Knowledge Manager for persistence
5. ✅ Include Security Auditor in all projects
6. ✅ Never hardcode credentials
7. ✅ Document as you build
8. ✅ Test everything with QA agent

## Requirements

- Node.js 16+
- Claude Code CLI
- Built-in Knowledge Manager (LanceDB)

## Contributing

This is a personal project demonstrating multi-agent development patterns. Feel free to fork and adapt for your needs.

## License

MIT

---

**Built with Claude Code** - Demonstrating the power of coordinated multi-agent development.
