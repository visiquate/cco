# Compile-Time Agent Embedding Documentation Index

Complete documentation for CCO's compile-time agent definition embedding system.

## Documentation Overview

This directory contains comprehensive documentation for understanding, developing, deploying, and troubleshooting CCO's compile-time embedding of agent definitions.

### Quick Navigation

| Document | Purpose | Audience | Length |
|----------|---------|----------|--------|
| [EMBEDDING_ARCHITECTURE.md](#embedding_architecture) | System design and architecture | Everyone | 12 KB |
| [BUILD_PROCESS.md](#build_process) | Build system and development | Developers | 10 KB |
| [config/agents/README.md](#agents_readme) | Agent development guide | Developers | 7.5 KB |
| [DEPLOYMENT_EMBEDDING.md](#deployment) | Deployment and operations | DevOps/Users | 13 KB |
| [EMBEDDING_TROUBLESHOOTING.md](#troubleshooting) | Common issues and solutions | Everyone | 12 KB |
| [EMBEDDING_IMPLEMENTATION.md](#implementation) | Code implementation details | Maintainers | 21 KB |

**Total Documentation:** ~75 KB, ~700+ lines of technical documentation

## Document Details

### EMBEDDING_ARCHITECTURE.md {#embedding_architecture}

**What it covers:**

- System architecture diagram (agents → build.rs → binary → HTTP API)
- Build-time process overview
- Runtime process and initialization
- Distribution model (single binary approach)
- Offline operation capabilities
- Custom agent extensions
- Benefits comparison table

**Use this document if you want to:**

- Understand how the system works
- Learn about agent definition compilation
- See how data flows through the system
- Understand distribution benefits
- Know why this approach was chosen

**Key sections:**

```
Overview
├─ System Architecture
├─ Build-Time Process
│  ├─ Configuration Files
│  ├─ build.rs Responsibilities
│  └─ Runtime Constants
├─ Runtime Process
│  ├─ Startup Sequence
│  ├─ Agent Loading Flow
│  └─ Data Structures
├─ Distribution Model
│  ├─ Single Binary Distribution
│  ├─ User Experience
│  ├─ Cross-Platform Support
│  └─ Offline Operation
├─ Benefits
├─ Custom Agent Extensions
└─ See Also
```

### BUILD_PROCESS.md {#build_process}

**What it covers:**

- Detailed build.rs script explanation
- File watch setup and triggers
- Version information extraction (git hash, timestamp)
- Configuration validation during build
- Compiler configuration settings
- Development workflow examples
- Build triggers and incremental compilation
- Environment variables (CCO_VERSION, RUST_LOG)
- CI/CD integration examples
- Performance characteristics
- Troubleshooting build issues

**Use this document if you want to:**

- Understand how the build system works
- Add or modify agent definitions
- Set version for releases
- Integrate CCO into CI/CD pipeline
- Debug build failures
- Optimize build performance

**Key sections:**

```
Overview
├─ Build Phases
│  ├─ Phase 1: Initialization
│  ├─ Phase 2: Build Script Execution
│  │  ├─ File Watch Setup
│  │  ├─ Version Information
│  │  ├─ Configuration Validation
│  │  └─ Compiler Configuration
│  ├─ Phase 3: Rust Compilation
│  └─ Phase 4: Artifact Generation
├─ Build Triggers and Incremental Compilation
├─ Development Workflow
├─ Build Configuration
├─ Troubleshooting Build Issues
├─ Environment Variables
├─ Build Performance
└─ CI/CD Integration
```

### config/agents/README.md {#agents_readme}

**What it covers:**

- Purpose of agents directory
- Agent file format (YAML frontmatter)
- Required fields (name, model, description, tools)
- Step-by-step guide for adding new agents
- Instructions for modifying existing agents
- File and naming conventions
- Model selection guide (opus, sonnet, haiku)
- Tools definition and assignment
- Validation procedures
- Best practices and common pitfalls
- Directory structure and organization

**Use this document if you want to:**

- Add a new agent definition
- Modify an existing agent
- Understand agent file format
- Learn naming conventions
- Assign correct model to agents
- Validate your agent definitions

**Key sections:**

```
Quick Start
├─ Purpose
├─ File Format
└─ Components
├─ Adding a New Agent
├─ Modifying Existing Agents
├─ File Naming Conventions
├─ Agent Naming Conventions
├─ Model Assignment
├─ Tools Definition
├─ Validation
├─ Best Practices
├─ Directory Structure
├─ Troubleshooting
└─ Related Documentation
```

### DEPLOYMENT_EMBEDDING.md {#deployment}

**What it covers:**

- Distribution methods (GitHub, upload, package manager)
- Installation instructions (macOS, Linux, Windows)
- Running CCO with various options
- Accessing agent definitions via HTTP API
- Version management and deployment
- Docker deployment (Dockerfile, docker-compose example)
- Kubernetes deployment (manifests, deployment commands)
- Production considerations
- Resource requirements (minimum and recommended)
- Monitoring endpoints and health checks
- Database and logging
- Updates and rollback procedures
- Performance optimization
- Security considerations
- Multi-instance setup for high availability

**Use this document if you want to:**

- Deploy CCO to production
- Set up Docker/Kubernetes deployment
- Install CCO on end-user systems
- Manage versions and releases
- Monitor CCO in production
- Update or rollback deployments
- Optimize performance for your environment

**Key sections:**

```
Overview
├─ Key Deployment Benefits
├─ Distribution
│  ├─ Before Distribution
│  ├─ Distribution Methods
│  └─ File Distribution Checklist
├─ Installation
│  ├─ macOS Installation
│  ├─ Linux Installation
│  └─ Windows Installation
├─ Running CCO
├─ Accessing Agent Definitions
├─ Version Management
├─ Docker Deployment
├─ Kubernetes Deployment
├─ Production Considerations
├─ Monitoring
├─ Updates and Rollback
├─ Performance Optimization
├─ Security
└─ Summary
```

### EMBEDDING_TROUBLESHOOTING.md {#troubleshooting}

**What it covers:**

- Build failures and solutions
  - Config file not found
  - Invalid JSON
  - Failed to read config
  - Slow builds
- Agent loading issues
  - No agents loaded
  - Agent not found after build
  - Wrong agent information
- Runtime issues
  - Server won't start
  - Server crashes on startup
  - API returns empty list
- Version issues
  - Wrong version displayed
  - Version not in binary
- Performance issues
  - Slow agent loading
  - High memory usage
- Path and directory issues
- Diagnostic commands
- Common issues summary table

**Use this document if you want to:**

- Fix build failures
- Debug agent loading issues
- Troubleshoot runtime errors
- Diagnose version problems
- Solve performance issues
- Get diagnostic information
- Find solutions to common problems

**Key sections:**

```
Build Issues
├─ Config file not found
├─ Invalid JSON
├─ Failed to read config
└─ Build takes too long
├─ Agent Loading Issues
├─ Runtime Issues
├─ Version Issues
├─ Performance Issues
├─ Path and Directory Issues
├─ Diagnostic Commands
└─ Getting Help
```

### EMBEDDING_IMPLEMENTATION.md {#implementation}

**What it covers:**

- Architecture components (build.rs, agents_config.rs, server.rs)
- Detailed code implementation
  - build.rs functions and responsibilities
  - Agent structures and data types
  - YAML frontmatter parser implementation
  - Agent loading logic
  - HTTP API endpoints
- Data flow diagrams (build-time and runtime)
- Memory layout and constants
- Design decisions with trade-offs
  - Why load at runtime vs compile-time
  - Why HashMap for agents
  - Why YAML frontmatter
- Cargo build system integration
- Performance characteristics (timing and complexity)
- Error handling strategies
- Testing (unit and integration tests)

**Use this document if you want to:**

- Understand the code implementation
- Modify the agent system
- Debug complex issues
- Learn design decisions
- Contribute to the project
- Understand performance trade-offs
- See code examples and patterns

**Key sections:**

```
Architecture Components
├─ build.rs - Build Script
├─ agents_config.rs - Configuration
└─ server.rs - HTTP API
├─ Data Flow (Build Time and Runtime)
├─ Memory Layout
├─ Design Decisions
├─ Cargo Build System Integration
├─ Performance Characteristics
├─ Error Handling
└─ Testing
```

## Using This Documentation

### For Different Roles

#### Developers

Start with:
1. [EMBEDDING_ARCHITECTURE.md](#embedding_architecture) - Understand the overall system
2. [BUILD_PROCESS.md](#build_process) - Learn the build workflow
3. [config/agents/README.md](#agents_readme) - Learn how to add/modify agents
4. [EMBEDDING_IMPLEMENTATION.md](#implementation) - Understand the code

#### DevOps/Operations

Start with:
1. [EMBEDDING_ARCHITECTURE.md](#embedding_architecture) - Understand how it works
2. [DEPLOYMENT_EMBEDDING.md](#deployment) - Learn deployment options
3. [EMBEDDING_TROUBLESHOOTING.md](#troubleshooting) - Fix common issues
4. [BUILD_PROCESS.md](#build_process) - Understand CI/CD integration

#### End Users

Start with:
1. [DEPLOYMENT_EMBEDDING.md](#deployment) - Installation instructions
2. [EMBEDDING_TROUBLESHOOTING.md](#troubleshooting) - Fix any issues

#### Maintainers

Start with:
1. [EMBEDDING_IMPLEMENTATION.md](#implementation) - Understand the code
2. [EMBEDDING_ARCHITECTURE.md](#embedding_architecture) - Design decisions
3. [EMBEDDING_TROUBLESHOOTING.md](#troubleshooting) - Common issues
4. [BUILD_PROCESS.md](#build_process) - Build system details

### By Task

| Task | Documents |
|------|-----------|
| **Add new agent** | agents/README.md, BUILD_PROCESS.md |
| **Deploy to production** | DEPLOYMENT_EMBEDDING.md, EMBEDDING_TROUBLESHOOTING.md |
| **Create Docker image** | DEPLOYMENT_EMBEDDING.md |
| **Deploy to Kubernetes** | DEPLOYMENT_EMBEDDING.md |
| **Fix build failure** | EMBEDDING_TROUBLESHOOTING.md, BUILD_PROCESS.md |
| **Fix agent loading** | EMBEDDING_TROUBLESHOOTING.md, agents/README.md |
| **Understand system** | EMBEDDING_ARCHITECTURE.md, EMBEDDING_IMPLEMENTATION.md |
| **Optimize performance** | DEPLOYMENT_EMBEDDING.md, EMBEDDING_IMPLEMENTATION.md |
| **Debug issue** | EMBEDDING_TROUBLESHOOTING.md, EMBEDDING_IMPLEMENTATION.md |
| **Release new version** | BUILD_PROCESS.md, DEPLOYMENT_EMBEDDING.md |

## Key Concepts

### Agent Definition

An agent definition is a markdown file with YAML frontmatter in `cco/config/agents/`:

```markdown
---
name: agent-identifier
model: opus|sonnet|haiku
description: Brief description
tools: Tool1, Tool2, Tool3
---

# Agent Documentation
Additional markdown content...
```

### Compile-Time Embedding

Definitions are validated during build (`cargo build --release`) but loaded at runtime from `~/.claude/agents/`.

This approach:
- Validates syntax at build time (fail fast)
- Allows custom agent extensions
- Keeps binaries smaller
- Enables incremental updates without rebuilding

### HTTP API

Agent definitions are served via HTTP:

```bash
# List all agents
curl http://localhost:3000/api/agents

# Get specific agent
curl http://localhost:3000/api/agents/chief-architect
```

### Distribution

Single executable binary contains:
- All compiled code
- Version information
- Build metadata
- Embedded constants

Users receive:
- One file: `cco` binary
- No configuration files needed
- Works offline
- Cross-platform compatible

## Cross-References

All documents reference each other at the bottom with "See Also" sections for easy navigation.

### Quick Links

- **Config Files**: `/Users/brent/git/cc-orchestra/cco/config/`
- **Build Script**: `/Users/brent/git/cc-orchestra/cco/build.rs`
- **Source Code**: `/Users/brent/git/cc-orchestra/cco/src/`
  - `agents_config.rs` - Agent loading logic
  - `server.rs` - HTTP API endpoints

## Documentation Statistics

```
Total Documents: 6
Total Lines: ~1,200
Total Size: ~75 KB
Code Examples: 30+
Diagrams: 5+
Tables: 15+
Troubleshooting Issues: 20+
```

## Maintenance

These documents are kept in sync with the codebase:

- **agents_config.rs changes** → Update agents/README.md and EMBEDDING_IMPLEMENTATION.md
- **build.rs changes** → Update BUILD_PROCESS.md and EMBEDDING_IMPLEMENTATION.md
- **server.rs changes** → Update EMBEDDING_ARCHITECTURE.md and DEPLOYMENT_EMBEDDING.md
- **New agent added** → Reference in BUILD_PROCESS.md development workflow section

## Related Files

**Configuration:**
- `cco/config/agents/` - 119 agent definitions
- `cco/config/orchestra-config.json` - Orchestration configuration

**Source Code:**
- `cco/build.rs` - Build script
- `cco/src/agents_config.rs` - Agent loading and parsing
- `cco/src/server.rs` - HTTP API
- `cco/Cargo.toml` - Package configuration

**Tests:**
- `cco/tests/` - Integration tests

## Questions?

For specific questions, refer to:

1. **"How does build work?"** → BUILD_PROCESS.md
2. **"How do I add an agent?"** → config/agents/README.md
3. **"How do I deploy?"** → DEPLOYMENT_EMBEDDING.md
4. **"Why did this fail?"** → EMBEDDING_TROUBLESHOOTING.md
5. **"How is it implemented?"** → EMBEDDING_IMPLEMENTATION.md
6. **"What is the system design?"** → EMBEDDING_ARCHITECTURE.md

## Summary

This documentation provides:

✓ **Architecture understanding** - How the system is designed
✓ **Developer guide** - How to work with agents
✓ **Build documentation** - How compilation works
✓ **Deployment guide** - How to get to production
✓ **Troubleshooting** - How to fix common issues
✓ **Implementation details** - How code implements design
✓ **Best practices** - How to use the system correctly
✓ **Examples** - Code snippets and procedures
✓ **Quick reference** - Tables and summaries
✓ **Cross-references** - Links between documents

All documentation is clear, comprehensive, and suitable for developers, operations, users, and maintainers.
