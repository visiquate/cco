# Claude Orchestra Documentation Index

**Version**: 2.0.0
**Last Updated**: 2025-11-10
**Status**: Complete

---

## Overview

The Claude Orchestra documentation is organized in a 4-tier structure, progressing from executive summary to deep technical implementation. Each document targets a specific audience and depth of coverage.

**Total Documentation**: 4 primary documents + supporting reference materials
**Coverage**: Complete system architecture, agent capabilities, deployment, and autonomous operation

---

## Documentation Structure

### Tier 1: Executive Summary
**Document**: [EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md)
**Target Audience**: Executives, product managers, decision-makers
**Reading Time**: 10-15 minutes
**Depth**: High-level overview with business value

**When to Read**:
- First introduction to Claude Orchestra
- Making adoption decisions
- Understanding business impact
- Communicating value to stakeholders

**Key Topics**:
- What Claude Orchestra is and why it matters
- Real business impact (2.8-4.4x speed, 32% cost reduction)
- How the 15-agent team works together
- Practical use cases with concrete examples
- Success metrics and ROI

---

### Tier 2: Technical Overview
**Document**: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md)
**Target Audience**: Software engineers, technical leads, DevOps engineers
**Reading Time**: 30-45 minutes
**Depth**: Technical architecture with operational details

**When to Read**:
- Planning integration into development workflow
- Understanding technical capabilities
- Evaluating technology stack
- Learning coordination protocols
- Setting up projects for army usage

**Key Topics**:
- System architecture and component breakdown
- 15-agent roster with specialties
- Model routing via ccproxy (local Ollama)
- Knowledge Manager for persistent memory
- TDD-aware pipeline (Phase 1 & 2)
- Usage patterns and auto-detection triggers
- Performance characteristics and scalability
- Security and credential management
- Deployment architecture

---

### Tier 3: Deep Dive
**Document**: [DEEP_DIVE.md](DEEP_DIVE.md)
**Target Audience**: System architects, senior engineers, contributors
**Reading Time**: 1-2 hours
**Depth**: Complete implementation details with code examples

**When to Read**:
- Contributing to the orchestra codebase
- Extending agent capabilities
- Debugging issues or performance tuning
- Understanding compaction resilience
- Implementing custom integrations
- Advanced configuration and customization

**Key Topics**:
- Hierarchical coordination topology (Chief Architect leadership)
- Agent authority matrix (low/medium/high risk decisions)
- Autonomous operation framework (4-8 hour target)
- Knowledge Manager implementation (LanceDB, 384-dim embeddings)
- Coordination protocol (pre/during/post hooks)
- ccproxy deployment (LiteLLM, Traefik, Cloudflare tunnel)
- Autonomous features (model fallback, error recovery, checkpointing)
- Requirements discovery process (60-80 adaptive questions)
- File structure and configuration schema
- Credential management internals (AES-256-CBC encryption)
- API reference and CLI commands
- Troubleshooting guide

---

### Tier 4: Architecture Diagrams
**Document**: [ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md)
**Target Audience**: All technical roles
**Reading Time**: 20-30 minutes
**Depth**: Visual architecture with Mermaid diagrams

**When to Read**:
- Visual understanding of system architecture
- Presenting to technical teams
- Onboarding new engineers
- Documentation in pull requests
- System design discussions

**Key Diagrams**:
1. **High-Level System Architecture**: 15 agents, model routing, Knowledge Manager
2. **Agent Coordination Flow**: Sequence diagram showing TDD workflow
3. **Knowledge Manager Architecture**: LanceDB, per-repo isolation, compaction hooks
4. **ccproxy Model Routing**: API aliases, Ollama models, bearer token auth
5. **Autonomous Operation Workflow**: 8 phases, checkpoints, error recovery
6. **Cross-Repository Deployment**: Global config, auto-detection, per-repo context
7. **Agent Phase Architecture**: Phase 1 (11 agents, 25GB) → Phase 2 (3 agents, 35GB)
8. **Decision Authority Matrix**: Low/medium/high risk decision flowchart

---

## Quick Navigation by Use Case

### "I'm new to Claude Orchestra"
**Start here**: [EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md)
**Next**: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → Usage Patterns section
**Then**: [ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md) → High-Level System Architecture

---

### "I want to use the orchestra in my project"
**Start here**: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → Usage Patterns
**Next**: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → Agent Selection Examples
**Then**: [README.md](../README.md) → Quick Start

**Key Resources**:
- [QUICK_START.md](QUICK_START.md) - Getting started guide
- [ARMY_USAGE_GUIDE.md](ARMY_USAGE_GUIDE.md) - Comprehensive usage patterns
- [ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md) → Cross-Repository Deployment

---

### "I'm integrating with Salesforce or Authentik"
**Start here**: [API_INTEGRATION_GUIDE.md](API_INTEGRATION_GUIDE.md)
**Next**: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → Integration Specialists section
**Then**: [DEEP_DIVE.md](DEEP_DIVE.md) → Implementation Details → Integration Patterns

**Key Resources**:
- [ORCHESTRA_ROSTER_TDD.md](ORCHESTRA_ROSTER_TDD.md) → Salesforce API Specialist
- [ORCHESTRA_ROSTER_TDD.md](ORCHESTRA_ROSTER_TDD.md) → Authentik API Specialist
- Credential Manager for secure token management

---

### "I'm setting up infrastructure or deploying"
**Start here**: [DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md)
**Next**: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → Deployment Architecture
**Then**: [DEVOPS_AGENT_GUIDE.md](DEVOPS_AGENT_GUIDE.md)

**Key Resources**:
- [DEEP_DIVE.md](DEEP_DIVE.md) → ccproxy Deployment
- [ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md) → ccproxy Model Routing
- [config/ccproxy/ccproxy-config-tdd-pipeline.yaml](../config/ccproxy/ccproxy-config-tdd-pipeline.yaml)

---

### "I'm contributing to the codebase"
**Start here**: [DEEP_DIVE.md](DEEP_DIVE.md) → System Design Deep Dive
**Next**: [DEEP_DIVE.md](DEEP_DIVE.md) → Implementation Details
**Then**: [DEEP_DIVE.md](DEEP_DIVE.md) → API Reference

**Key Resources**:
- [config/orchestra-config.json](../config/orchestra-config.json) - Agent definitions
- [src/orchestra-conductor.js](../src/orchestra-conductor.js) - Main orchestrator
- [src/knowledge-manager.js](../src/knowledge-manager.js) - Knowledge Manager
- [DEEP_DIVE.md](DEEP_DIVE.md) → Advanced Topics → Custom Agent Creation

---

### "I'm troubleshooting an issue"
**Start here**: [DEEP_DIVE.md](DEEP_DIVE.md) → Troubleshooting Guide
**Next**: [DEEP_DIVE.md](DEEP_DIVE.md) → Common Issues
**Then**: [DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md) → Verification Commands

**Key Resources**:
- [DEEP_DIVE.md](DEEP_DIVE.md) → Health Checks
- [DEEP_DIVE.md](DEEP_DIVE.md) → Recovery Procedures
- [KNOWLEDGE_MANAGER_GUIDE.md](KNOWLEDGE_MANAGER_GUIDE.md) → Operations

---

### "I need to understand the Knowledge Manager"
**Start here**: [KNOWLEDGE_MANAGER_GUIDE.md](KNOWLEDGE_MANAGER_GUIDE.md)
**Next**: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → Coordination & Memory
**Then**: [DEEP_DIVE.md](DEEP_DIVE.md) → Knowledge Manager Implementation

**Key Resources**:
- [ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md) → Knowledge Manager Architecture
- [src/knowledge-manager.js](../src/knowledge-manager.js) - Implementation
- [DEEP_DIVE.md](DEEP_DIVE.md) → Pre/Post-Compaction Hooks

---

### "I want to understand TDD methodology"
**Start here**: [TDD_AWARE_PIPELINE.md](TDD_AWARE_PIPELINE.md)
**Next**: [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → How It Works → TDD Workflow
**Then**: [ORCHESTRA_ROSTER_TDD.md](ORCHESTRA_ROSTER_TDD.md) → TDD Coding Agent

**Key Resources**:
- [ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md) → Agent Coordination Flow
- [DEEP_DIVE.md](DEEP_DIVE.md) → Workflow Phases → Implementation

---

### "I need autonomous operation details"
**Start here**: [AUTONOMOUS_OPERATION_FRAMEWORK.md](AUTONOMOUS_OPERATION_FRAMEWORK.md)
**Next**: [DEEP_DIVE.md](DEEP_DIVE.md) → Autonomous Operation (v2.0.0)
**Then**: [ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md) → Autonomous Operation Workflow

**Key Resources**:
- [config/orchestra-config.json](../config/orchestra-config.json) → autonomousOperation section
- [DEEP_DIVE.md](DEEP_DIVE.md) → Automatic Model Fallback
- [DEEP_DIVE.md](DEEP_DIVE.md) → Compaction Resilience
- [DEEP_DIVE.md](DEEP_DIVE.md) → Error Recovery Strategies

---

## Supporting Documentation

### Configuration Files
- **[config/orchestra-config.json](../config/orchestra-config.json)** - Complete agent definitions, model mappings, autonomous settings
- **[config/ccproxy/ccproxy-config-tdd-pipeline.yaml](../config/ccproxy/ccproxy-config-tdd-pipeline.yaml)** - LiteLLM proxy configuration
- **[CLAUDE.md](../CLAUDE.md)** - Project-specific instructions for cc-orchestra repo
- **[ORCHESTRATOR_RULES.md](../ORCHESTRATOR_RULES.md)** - Critical delegation and coordination rules

### Implementation Files
- **[src/orchestra-conductor.js](../src/orchestra-conductor.js)** - Main orchestration logic (510 lines)
- **[src/knowledge-manager.js](../src/knowledge-manager.js)** - LanceDB integration (637 lines)
- **[src/credential-manager.js](../src/credential-manager.js)** - Secure credential management (305 lines)

### Additional Guides
- **[QUICK_START.md](QUICK_START.md)** - Getting started guide
- **[ARMY_USAGE_GUIDE.md](ARMY_USAGE_GUIDE.md)** - Comprehensive usage patterns
- **[ORCHESTRA_ROSTER_TDD.md](ORCHESTRA_ROSTER_TDD.md)** - Complete 15-agent roster
- **[TDD_AWARE_PIPELINE.md](TDD_AWARE_PIPELINE.md)** - TDD methodology (956 lines)
- **[AUTONOMOUS_OPERATION_FRAMEWORK.md](AUTONOMOUS_OPERATION_FRAMEWORK.md)** - Autonomous features
- **[API_INTEGRATION_GUIDE.md](API_INTEGRATION_GUIDE.md)** - Salesforce/Authentik integration
- **[DEVOPS_AGENT_GUIDE.md](DEVOPS_AGENT_GUIDE.md)** - Infrastructure and deployment
- **[DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md)** - ccproxy deployment status
- **[KNOWLEDGE_MANAGER_GUIDE.md](KNOWLEDGE_MANAGER_GUIDE.md)** - Knowledge Manager operations

---

## Documentation Principles

### 1. Progressive Disclosure
Documentation progresses from high-level overview (Tier 1) to deep implementation details (Tier 3), allowing readers to stop at the appropriate depth for their needs.

### 2. Audience-Specific
Each tier targets a specific audience with appropriate technical depth:
- **Executives**: Business value and ROI
- **Engineers**: Technical architecture and usage
- **Architects**: Implementation and extensibility
- **All**: Visual architecture

### 3. Cross-Referenced
All documents link to related content, enabling easy navigation between topics and depth levels.

### 4. Example-Driven
Every concept includes concrete examples:
- Use cases with realistic scenarios
- Code snippets for implementation
- Command-line examples for operations
- Diagrams for visual understanding

### 5. Living Documentation
Documentation is updated with every major system change and versioned alongside the codebase.

---

## Documentation Coverage

### Complete Coverage
✅ System architecture and agent roster
✅ Model routing and deployment infrastructure
✅ Knowledge Manager implementation
✅ Coordination protocols and workflows
✅ TDD-aware pipeline methodology
✅ Autonomous operation framework
✅ Security and credential management
✅ API integration patterns (Salesforce, Authentik)
✅ Cross-repository deployment
✅ Troubleshooting and recovery procedures
✅ Visual architecture diagrams

### Well-Documented Components
✅ 15-agent roster with specialties
✅ Chief Architect authority and fallback
✅ ccproxy deployment (LiteLLM, Traefik, Cloudflare)
✅ Knowledge Manager (LanceDB, vector search)
✅ Credential Manager (AES-256-CBC encryption)
✅ Decision authority matrix (low/medium/high risk)
✅ Compaction resilience (pre/post hooks)
✅ Error recovery strategies (90%+ autonomous)
✅ Requirements discovery process (60-80 questions)

---

## Future Documentation Enhancements

### Planned Additions
- **Video Tutorials**: Walkthrough of common workflows
- **API Examples**: More integration code samples
- **Performance Tuning**: Optimization guide
- **Case Studies**: Real-world project outcomes
- **Contributor Guide**: How to extend the orchestra
- **Runbooks**: Operational procedures for production

### Requested Topics
- Advanced Knowledge Manager queries
- Custom agent creation tutorial
- Multi-repo coordination strategies
- Performance benchmarking methodology
- Integration with CI/CD pipelines

---

## Documentation Maintenance

### Update Schedule
- **Major Releases**: Complete documentation review
- **Feature Additions**: Update relevant sections within 1 week
- **Bug Fixes**: Update troubleshooting guide
- **Configuration Changes**: Update immediately

### Versioning
Documentation version matches system version (currently 2.0.0).

### Feedback
Documentation improvements are tracked in GitHub issues with the `documentation` label.

---

## Quick Reference

### Most Common Questions

**Q: How do I get started with Claude Orchestra?**
A: Read [EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md), then [QUICK_START.md](QUICK_START.md)

**Q: What agents are available?**
A: See [ORCHESTRA_ROSTER_TDD.md](ORCHESTRA_ROSTER_TDD.md) for the complete 15-agent roster

**Q: How does the TDD pipeline work?**
A: Read [TDD_AWARE_PIPELINE.md](TDD_AWARE_PIPELINE.md) and [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → TDD Workflow

**Q: How do I integrate with Salesforce?**
A: See [API_INTEGRATION_GUIDE.md](API_INTEGRATION_GUIDE.md) → Salesforce Integration

**Q: What is the Knowledge Manager?**
A: Read [KNOWLEDGE_MANAGER_GUIDE.md](KNOWLEDGE_MANAGER_GUIDE.md) and [ARCHITECTURE_DIAGRAMS.md](ARCHITECTURE_DIAGRAMS.md) → Knowledge Manager Architecture

**Q: How does autonomous operation work?**
A: Read [AUTONOMOUS_OPERATION_FRAMEWORK.md](AUTONOMOUS_OPERATION_FRAMEWORK.md) and [DEEP_DIVE.md](DEEP_DIVE.md) → Autonomous Operation

**Q: How is the system deployed?**
A: See [DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md) and [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) → Deployment Architecture

**Q: I'm getting an error, how do I troubleshoot?**
A: Check [DEEP_DIVE.md](DEEP_DIVE.md) → Troubleshooting Guide

---

## Summary

The Claude Orchestra documentation provides comprehensive coverage across 4 tiers:
1. **Executive Summary**: Business value and high-level overview
2. **Technical Overview**: Architecture and operational details
3. **Deep Dive**: Complete implementation with code examples
4. **Architecture Diagrams**: Visual system architecture

All documentation is current as of version 2.0.0 and covers the complete 15-agent TDD-aware pipeline with autonomous operation capabilities.

**Start your journey**: [EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md)

---

**Last Updated**: 2025-11-10
**Status**: Complete and current
**Maintainers**: Documentation Team
