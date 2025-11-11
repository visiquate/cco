# Comprehensive Orchestra Roster - 117 Specialized Agents

## Overview

The Claude Orchestra consists of **117 specialized agents** (1 Chief Architect + 116 specialists) organized into 12 categories, providing comprehensive coverage for all aspects of software development, operations, research, and business analysis.

**Model Distribution**:
- **1 agent** - Opus 4.1 (Chief Architect - strategic leadership)
- **77 agents** - Sonnet 4.5 (intelligent managers, reviewers, architects)
- **39 agents** - Haiku 4.5 (basic coders, documentation, utilities)

## Agent Categories

### 1. Architect (1 Agent)
**Chief Architect** - Strategic decision-making, system design, agent coordination, compaction management
- Model: Opus 4.1 with Sonnet 4.5 fallback
- Authority: High-level decisions with documentation

### 2. Coding Agents (6 Agents)
Core development specialists for TDD and language-specific implementation:
1. **TDD Coding Agent** - Test-driven development, Red-Green-Refactor cycle
2. **Python Specialist** - FastAPI, Django, ML/AI integration
3. **Swift Specialist** - SwiftUI, UIKit, iOS development
4. **Go Specialist** - Microservices, cloud-native applications
5. **Rust Specialist** - Systems programming, performance-critical code
6. **Flutter Specialist** - Cross-platform mobile development

### 3. Integration Agents (3 Agents)
API exploration and third-party integrations:
1. **API Explorer** - REST/GraphQL API analysis, authentication flows
2. **Salesforce API Specialist** - Salesforce REST/SOAP/Bulk API integration
3. **Authentik API Specialist** - OAuth2/OIDC, SAML integration

### 4. Development Agents (25 Agents)
Specialized development expertise across frameworks and technologies:
1. Frontend Developer
2. Backend Architect
3. Fullstack Developer
4. Code Reviewer
5. Debugger
6. Python Pro
7. Typescript Pro
8. Javascript Pro
9. Golang Pro
10. Rust Pro
11. Mobile Developer
12. iOS Developer
13. Next.js Architecture Expert
14. React Performance Optimization
15. React Performance Optimizer
16. GraphQL Architect
17. GraphQL Performance Optimizer
18. GraphQL Security Specialist
19. Shell Scripting Pro
20. Legacy Modernizer
21. Architecture Modernizer
22. DX Optimizer
23. Git Flow Manager
24. Dependency Manager
25. Error Detective

### 5. Data Agents (9 Agents)
Database design, optimization, and data engineering:
1. Database Architect
2. Database Admin
3. Database Optimization
4. Database Optimizer
5. Data Engineer
6. Data Scientist
7. Data Analyst
8. NoSQL Specialist
9. SQL Pro

### 6. Infrastructure Agents (10 Agents)
DevOps, cloud infrastructure, and deployment automation:
1. DevOps Engineer
2. Deployment Engineer
3. Cloud Architect
4. Cloud Migration Specialist
5. Terraform Specialist
6. Network Engineer
7. Monitoring Specialist
8. DevOps Troubleshooter
9. Incident Responder
10. Load Testing Specialist

### 7. Security Agents (8 Agents)
Security auditing, compliance, and penetration testing:
1. Security Auditor
2. Security Engineer
3. API Security Audit
4. Penetration Tester
5. Compliance Specialist
6. MCP Security Auditor
7. Web Accessibility Checker
8. Risk Manager

### 8. AI/ML Agents (6 Agents)
Machine learning, LLM integration, and MLOps:
1. AI Engineer
2. ML Engineer
3. MLOps Engineer
4. Model Evaluator
5. Prompt Engineer
6. LLMs Maintainer

### 9. MCP Agents (6 Agents)
Model Context Protocol specialists:
1. MCP Expert
2. MCP Server Architect
3. MCP Integration Engineer
4. MCP Deployment Orchestrator
5. MCP Protocol Specialist
6. MCP Testing Engineer

### 10. Documentation Agents (7 Agents)
Technical writing and documentation generation:
1. Documentation Lead
2. Technical Writer
3. Documentation Expert
4. API Documenter
5. Changelog Generator
6. Markdown Syntax Formatter
7. Report Generator

### 11. Research Agents (10 Agents)
Research, analysis, and information synthesis:
1. Technical Researcher
2. Academic Researcher
3. Research Orchestrator
4. Research Coordinator
5. Research Synthesizer
6. Research Brief Generator
7. Comprehensive Researcher
8. Fact Checker
9. Query Clarifier
10. Search Specialist

### 12. Support Agents (30 Agents)
Quality assurance, testing, UX, and operational support:
1. Documentation Lead (existing)
2. Technical Writer (existing)
3. User Experience Designer (existing)
4. QA Engineer (existing)
5. Security Auditor (existing)
6. Credential Manager (existing)
7. DevOps Engineer (existing)
8. Test Engineer
9. Test Automator
10. UI/UX Designer
11. CLI UI Designer
12. Performance Engineer
13. Performance Profiler
14. Context Manager
15. Task Decomposition Expert
16. Architect Review
17. Command Expert
18. Connection Agent
19. Metadata Agent
20. Tag Agent
21. Document Structure Analyzer
22. URL Link Extractor
23. Project Supervisor Orchestrator
24. Agent Overview
25. Supabase Schema Architect
26. Supabase Realtime Optimizer
27. Review Agent
28. Flutter Go Reviewer
29. Markdown Syntax Formatter
30. Document Structure Analyzer

### 13. Business Agents (4 Agents)
Business strategy, analysis, and marketing:
1. Product Strategist
2. Business Analyst
3. Content Marketer
4. Quant Analyst

## Model Configuration

All agents (except Chief Architect) use **model: "sonnet-4.5"**, which routes through ccproxy to local Ollama models:

- **Phase 1 Coding**: qwen2.5-coder:32b-instruct (32k context)
- **Phase 1 Lightweight**: qwen-fast:latest (32k context)
- **Phase 2 Reasoning**: qwen-quality-128k:latest (128k context)

## Autonomous Authority

Agents are configured with appropriate autonomous authority levels:

- **Low Risk**: All agents can make low-risk decisions autonomously
- **Medium Risk**: Most agents except security and infrastructure
- **High Risk**: None (requires user approval)
- **Architect Approval**: Required for medium-risk decisions

## Agent File Locations

All new agents reference their source files in `~/.claude/agents/`:
```json
{
  "agentFile": "~/.claude/agents/agent-name.md"
}
```

## Coordination & Integration

All agents integrate with:
- **Knowledge Manager**: Persistent cross-agent memory
- **MCP Servers**: claude-flow@alpha, ruv-swarm
- **Workflow Hooks**: Pre/post compaction, progress tracking
- **Decision Authority Matrix**: Clear escalation paths

## Usage

When spawning agents via Claude Code's Task tool:

```javascript
// Single message with parallel agent spawning
Task("Frontend Developer", "Build React UI...", "coder")
Task("Backend Architect", "Design API...", "system-architect")
Task("Database Architect", "Design schema...", "system-architect")
Task("QA Engineer", "Create test suite...", "test-automator")
Task("Security Auditor", "Review security...", "security-auditor")
```

## Total Agent Count

**117 specialized agents** covering:
- ✅ All programming languages and frameworks
- ✅ Complete DevOps and infrastructure lifecycle
- ✅ Comprehensive security and compliance
- ✅ Full research and documentation capabilities
- ✅ End-to-end quality assurance
- ✅ Business strategy and analysis
- ✅ AI/ML development and operations
- ✅ MCP protocol ecosystem

## Future Enhancements

Potential additions:
- Blockchain/Web3 specialists
- Game development experts
- Embedded systems engineers
- IoT platform specialists
- AR/VR development agents
- Quantum computing researchers

---

**Last Updated**: 2025-11-11
**Version**: 2.0.0 (Model-Optimized Configuration)
**Config File**: `/Users/brent/git/cc-orchestra/config/orchestra-config.json`
