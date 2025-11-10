# Claude Orchestra Documentation

Complete documentation for the Claude Orchestra multi-agent development system with autonomous operation capabilities.

---

## Quick Links

### Getting Started
- [Main README](../README.md) - Project overview and quick start
- [Quick Start Guide](QUICK_START.md) - Get up and running quickly
- [Army Roster](../ORCHESTRA_ROSTER.md) - Complete agent specifications

### Usage Guides
- [Army Usage Guide](ARMY_USAGE_GUIDE.md) - Comprehensive usage instructions
- [Cross-Repository Usage](CROSS_REPO_USAGE.md) - Use army from any directory
- [Cross-Repo Implementation](CROSS_REPO_IMPLEMENTATION.md) - Implementation details

### Autonomous Operation (NEW)
- [**Autonomous Operation Analysis**](AUTONOMOUS_OPERATION_ANALYSIS.md) - Obstacles and solutions for hours-long autonomous operation
- [**Autonomous Operation Framework**](AUTONOMOUS_OPERATION_FRAMEWORK.md) - Complete framework for autonomous work
- [**Autonomous Workflow Guide**](AUTONOMOUS_WORKFLOW_GUIDE.md) - End-to-end workflow for 4-8 hour autonomous sessions

### Specialized Guides
- [API Integration Guide](API_INTEGRATION_GUIDE.md) - Salesforce and Authentik integration
- [DevOps Agent Guide](DEVOPS_AGENT_GUIDE.md) - Infrastructure and deployment
- [Example Workflow](EXAMPLE_WORKFLOW.md) - Full example workflow

### Configuration
- [Army Configuration](../config/orchestra-config.json) - Agent definitions and settings
- [Project Template](PROJECT_CLAUDE_TEMPLATE.md) - Template for project-specific customization

---

## What's New in v2.0.0

### 🚀 Autonomous Operation Capabilities

The Claude Orchestra now supports **extended autonomous operation** for 4-8 hours without user intervention:

**Key Features:**
- ✅ **Automatic Model Fallback**: Opus → Sonnet 4.5 when tokens exhausted
- ✅ **Compaction Resilience**: Zero data loss across conversation compactions
- ✅ **Autonomous Error Recovery**: 90%+ errors handled without user
- ✅ **Smart Decision Making**: Clear authority matrix for low/medium/high risk decisions
- ✅ **Progress Checkpointing**: Automatic checkpoints every 30-60 minutes
- ✅ **Heartbeat Monitoring**: Agent health tracking and automatic recovery
- ✅ **Autonomous Test Fixing**: QA agent fixes common test failures
- ✅ **Continuous Quality**: Security and quality checks throughout

**Updated Configuration:**
- All agents now use **Sonnet 4.5** (not generic "sonnet")
- Architect has **automatic fallback** from Opus to Sonnet 4.5
- **Decision authority matrix** defines autonomous decision boundaries
- **Compaction management** enabled with pre/post scripts

**New Documentation:**
1. [Autonomous Operation Analysis](AUTONOMOUS_OPERATION_ANALYSIS.md) - Deep dive into obstacles and solutions
2. [Autonomous Operation Framework](AUTONOMOUS_OPERATION_FRAMEWORK.md) - Framework components and implementation
3. [Autonomous Workflow Guide](AUTONOMOUS_WORKFLOW_GUIDE.md) - Complete 8-hour workflow example

**New Scripts:**
- `scripts/pre-compaction.sh` - Export state before compaction
- `scripts/post-compaction.sh` - Restore state after compaction

---

## Documentation Structure

### Core Documentation

**[Main README](../README.md)**
- Project overview
- Quick start (3 steps)
- Cross-repository usage
- Agent roles
- Usage examples
- Architecture

**[Army Roster](../ORCHESTRA_ROSTER.md)**
- Complete agent specifications (14 agents)
- Capabilities matrix
- Coordination protocol
- Common deployment scenarios
- Performance characteristics

### Usage Documentation

**[Army Usage Guide](ARMY_USAGE_GUIDE.md)**
- Detailed usage instructions
- Workflow phases
- Coordination protocols
- Best practices
- Common scenarios

**[Cross-Repository Usage](CROSS_REPO_USAGE.md)**
- Three-tier configuration architecture
- Trigger patterns (activate/bypass)
- Project customization
- Agent selection logic
- Troubleshooting

**[Cross-Repo Implementation](CROSS_REPO_IMPLEMENTATION.md)**
- Implementation details
- Architecture diagrams
- Files modified/created
- Testing recommendations
- Benefits breakdown

### Autonomous Operation Documentation

**[Autonomous Operation Analysis](AUTONOMOUS_OPERATION_ANALYSIS.md)** ⭐ NEW
- 7 critical obstacles to autonomous operation
- Detailed solutions for each obstacle
- Implementation priorities
- Success metrics
- Risk mitigation

**[Autonomous Operation Framework](AUTONOMOUS_OPERATION_FRAMEWORK.md)** ⭐ NEW
- Model fallback system
- Compaction management
- Error recovery system
- Decision authority matrix
- Progress checkpointing
- Heartbeat coordination
- Autonomous testing

**[Autonomous Workflow Guide](AUTONOMOUS_WORKFLOW_GUIDE.md)** ⭐ NEW
- Complete 8-hour workflow example
- Phase-by-phase breakdown
- Real-world timeline
- Monitoring dashboard
- Emergency procedures

### Integration Documentation

**[API Integration Guide](API_INTEGRATION_GUIDE.md)**
- Salesforce API integration
- Authentik API integration
- General API integration patterns
- OAuth2 workflows
- Rate limit handling

**[DevOps Agent Guide](DEVOPS_AGENT_GUIDE.md)**
- Docker and Kubernetes
- AWS infrastructure
- CI/CD pipelines
- Monitoring and logging
- Deployment strategies

**[Example Workflow](EXAMPLE_WORKFLOW.md)**
- Full workflow from start to finish
- Real commands and outputs
- Agent coordination examples
- Best practices in action

---

## Quick Navigation by Use Case

### "I want to use the army in my project"
1. Read [Cross-Repository Usage](CROSS_REPO_USAGE.md)
2. Copy [Project Template](PROJECT_CLAUDE_TEMPLATE.md) to your project
3. Follow [Quick Start Guide](QUICK_START.md)

### "I want autonomous operation for hours"
1. Read [Autonomous Workflow Guide](AUTONOMOUS_WORKFLOW_GUIDE.md)
2. Review [Autonomous Operation Framework](AUTONOMOUS_OPERATION_FRAMEWORK.md)
3. Understand [Autonomous Operation Analysis](AUTONOMOUS_OPERATION_ANALYSIS.md)

### "I need to integrate with Salesforce"
1. Read [API Integration Guide](API_INTEGRATION_GUIDE.md) - Salesforce section
2. Check [Army Roster](../ORCHESTRA_ROSTER.md) - Salesforce API Specialist
3. See [Example Workflow](EXAMPLE_WORKFLOW.md) - Salesforce example

### "I need to deploy infrastructure"
1. Read [DevOps Agent Guide](DEVOPS_AGENT_GUIDE.md)
2. Check [Army Roster](../ORCHESTRA_ROSTER.md) - DevOps Engineer
3. Review deployment examples

### "I want to customize army behavior"
1. Read [Cross-Repository Usage](CROSS_REPO_USAGE.md) - Project Customization
2. Copy [Project Template](PROJECT_CLAUDE_TEMPLATE.md)
3. Edit for your project needs

---

## Configuration Files

### Army Configuration
**File:** [`config/orchestra-config.json`](../config/orchestra-config.json)

**Contains:**
- 14 agent definitions (1 Architect + 5 Coding + 3 Integration + 5 Support)
- Model specifications (Opus for Architect with Sonnet 4.5 fallback, Sonnet 4.5 for most others)
- Agent capabilities and specialties
- Coordination topology (hierarchical)
- Autonomous operation settings
- Decision authority matrix
- Compaction management settings

**Updated in v2.0.0:**
- ✅ All agents use specific model versions ("sonnet-4.5" not "sonnet")
- ✅ Architect has automatic fallback configuration
- ✅ Autonomous authority levels defined per agent
- ✅ Autonomous operation enabled with settings

### Project Template
**File:** [`docs/PROJECT_CLAUDE_TEMPLATE.md`](PROJECT_CLAUDE_TEMPLATE.md)

**Use:** Copy to your project root as `CLAUDE.md` to customize army behavior

**Contains:**
- Agent preference checkboxes
- Custom trigger patterns
- Technology stack documentation
- Project-specific rules
- Security requirements
- Testing requirements
- Deployment requirements

---

## Scripts

### Compaction Management

**Pre-Compaction:**
```bash
./scripts/pre-compaction.sh
```
- Exports ALL critical state to MCP memory
- Preserves architecture, credentials, agent states
- Runs automatically before compaction

**Post-Compaction:**
```bash
./scripts/post-compaction.sh <SESSION_ID>
```
- Restores ALL state from MCP memory
- Broadcasts restoration to all agents
- Runs automatically after compaction

---

## Skills

### Project Discovery
**File:** [`skills/project-discovery.md`](../skills/project-discovery.md)

**Purpose:** Comprehensive requirements discovery before implementation

**Features:**
- 60-80 adaptive questions across 7 phases
- Interactive with clarification rounds
- Generates complete specification
- Stores in persistent memory
- Mandatory for complex projects

**Phases:**
0. Initial Assessment (determines scope)
1. Project Foundation
2. Technology Stack
3. Integration Requirements
4. Security & Compliance
5. Quality Requirements
6. Deployment & Operations
7. Definition of Done (MANDATORY)

---

## Version History

### v2.0.0 (Current) - Autonomous Operation
- ✅ Autonomous operation for 4-8 hours
- ✅ Automatic model fallback (Opus → Sonnet 4.5)
- ✅ All agents use Sonnet 4.5
- ✅ Compaction resilience with pre/post scripts
- ✅ Decision authority matrix
- ✅ Error recovery system
- ✅ Progress checkpointing
- ✅ Heartbeat monitoring
- ✅ Comprehensive documentation (3 new guides)

### v1.0.0 - Cross-Repository Usage
- ✅ Cross-repository operation
- ✅ Three-tier configuration architecture
- ✅ Auto-detection triggers
- ✅ Project customization template
- ✅ 14 specialized agents
- ✅ MCP coordination

---

## Contributing

This is a personal project demonstrating multi-agent development patterns. Feel free to fork and adapt for your needs.

---

## Support

- **Issues**: Report bugs or request features in GitHub issues
- **Documentation**: All guides in `docs/` directory
- **Examples**: See `docs/EXAMPLE_WORKFLOW.md` for complete examples

---

## License

MIT

---

**Built with Claude Code** - Demonstrating the power of autonomous multi-agent development.

Last updated: 2025-01-15
