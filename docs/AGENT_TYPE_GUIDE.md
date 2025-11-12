# Agent Type Guide - Claude Orchestra

**Version**: 2.1.0
**Date**: 2025-11-11
**Status**: Comprehensive guide to all 15 agent types in the reconciled orchestra

---

## Overview

The Claude Orchestra uses **15 specialized agent types** instead of generic roles. This guide explains when to use each type, their capabilities, and how they contribute to the orchestration system.

**Key Achievement**: Reduced generic "coder" usage from 52.7% to 3.4% through this type specialization framework.

---

## Agent Types Reference

### 1. test-automator (6 agents)

**Purpose**: Test creation, enhancement, and quality assurance

**When to Use**:
- Write failing tests before implementation (TDD workflow)
- Enhance existing test coverage
- Create edge case and integration tests
- Autonomous test fixing and repair
- Performance benchmarking
- Load testing and stress testing

**Capabilities**:
- Red-Green-Refactor cycle management
- Unit test design with mocks and fixtures
- Integration and E2E test creation
- Test coverage analysis (90%+ target)
- Test failure diagnosis and fixing
- Performance testing and benchmarking

**Agents in This Type**:
- TDD Coding Agent (Phase 1 Lead)
- Test Engineer
- Test Automator
- Web Accessibility Checker
- Load Testing Specialist
- MCP Testing Engineer

**Model**: Primarily Sonnet 4.5 (complex reasoning) | Some Haiku (specific domains)

---

### 2. backend-dev (30 agents)

**Purpose**: Backend development, database design, and infrastructure code

**When to Use**:
- Implement REST/GraphQL APIs
- Database schema design and optimization
- Data pipeline and ETL development
- Performance optimization
- Language-specific backend work (TypeScript, JavaScript, Node.js)
- Database administration and optimization
- Data engineering tasks

**Capabilities**:
- API design and implementation
- Query optimization (SQL, NoSQL)
- Database architecture
- Performance profiling
- Async/await patterns
- Microservices development
- Data structure design

**Agents in This Type** (30 total):
- Python Expert
- Go Expert
- Rust Expert
- Typescript Pro
- Javascript Pro
- Database Admin
- Database Optimization
- Database Optimizer
- Data Engineer
- SQL Pro
- NoSQL Specialist
- AI Engineer
- ML Engineer
- Graphql Performance Optimizer
- Performance Engineer
- Performance Profiler
- Command Expert
- Connection Agent
- Metadata Agent
- Tag Agent
- Fullstack Developer
- Frontend Developer
- React Performance Optimization
- React Performance Optimizer
- MCP Expert
- MCP Integration Engineer
- Web Vitals Optimizer
- Supabase Realtime Optimizer
- And 2 more specialized backend developers

**Model**: Sonnet 4.5 (complexity) | Haiku optimization for specific subtasks

---

### 3. security-auditor (7 agents)

**Purpose**: Security review, vulnerability analysis, and compliance

**When to Use**:
- Review authentication implementations
- Assess vulnerability risks
- Ensure OWASP compliance
- Design threat models
- Plan penetration tests
- Check API security
- Validate credential handling
- Review compliance requirements

**Capabilities**:
- Vulnerability assessment
- Threat modeling
- OWASP Top 10 compliance
- Penetration test planning
- Security best practices
- Compliance review (SOC 2, GDPR, HIPAA)
- Code security analysis
- API security auditing

**Agents in This Type**:
- Security Auditor (Phase 2)
- Security Engineer
- API Security Audit
- Penetration Tester
- Compliance Specialist
- MCP Security Auditor
- Graphql Security Specialist

**Model**: Sonnet 4.5 (complex threat analysis)

---

### 4. researcher (17 agents)

**Purpose**: Analysis, research, exploration, and fact-checking

**When to Use**:
- Explore third-party APIs
- Research technologies and patterns
- Analyze business requirements
- Verify facts and data
- Synthesize information
- Generate research briefs
- Model evaluation
- Quantitative analysis

**Capabilities**:
- API exploration and documentation
- Technology research
- Pattern analysis
- Data fact-checking
- Business analysis
- Model evaluation
- Prompt optimization
- Research synthesis
- Search and information gathering

**Agents in This Type**:
- API Explorer
- Research Synthesizer
- Research Brief Generator
- Comprehensive Researcher
- Fact Checker
- Search Specialist
- Model Evaluator
- Business Analyst
- Quant Analyst
- Document Structure Analyzer
- URL Link Extractor
- Mcp Protocol Specialist
- And 5 more research specialists

**Model**: Sonnet 4.5 for complex analysis | Haiku for lightweight tasks

---

### 5. deployment-engineer (15 agents)

**Purpose**: DevOps, infrastructure, and deployment automation

**When to Use**:
- Docker and container orchestration
- CI/CD pipeline setup
- Infrastructure as Code
- Cloud migrations
- Monitoring and observability setup
- Incident response and troubleshooting
- Package and dependency management
- Network configuration
- DevOps automation

**Capabilities**:
- Docker and Kubernetes management
- CI/CD pipeline creation
- Terraform and IaC
- AWS/Azure/GCP deployment
- Monitoring setup (Prometheus, ELK)
- Incident response
- Git workflow automation
- Deployment scripting
- Network infrastructure

**Agents in This Type**:
- DevOps Engineer
- Deployment Engineer
- Cloud Migration Specialist
- Terraform Specialist
- Network Engineer
- Monitoring Specialist
- Devops Troubleshooter
- Incident Responder
- Shell Scripting Pro
- Git Flow Manager
- Dependency Manager
- Dx Optimizer
- LLMS Maintainer
- And 2 more DevOps specialists

**Model**: Sonnet 4.5 | Haiku for automation and scripting

---

### 6. system-architect (10 agents)

**Purpose**: System design, architecture decisions, and modernization

**When to Use**:
- Design system architecture
- Make technology stack decisions
- Create API contracts
- Design database schemas
- Modernize legacy systems
- Review architecture patterns
- Plan scalability
- Design system integrations

**Capabilities**:
- System architecture design
- Technology selection
- API design and contracts
- Database architecture
- Scalability planning
- Architecture patterns
- Legacy system modernization
- Integration architecture

**Agents in This Type**:
- Chief Architect
- Backend Architect
- Cloud Architect
- Database Architect
- Context Manager
- Nextjs Architecture Expert
- Graphql Architect
- Legacy Modernizer
- Architecture Modernizer
- Supabase Schema Architect

**Model**: Opus 4.1 (Chief Architect) | Sonnet 4.5 for specialists

---

### 7. technical-writer (7 agents)

**Purpose**: Documentation, API reference, and technical writing

**When to Use**:
- Write API documentation
- Create architectural documentation
- Generate change logs
- Format documentation
- Write technical guides
- Create deployment runbooks
- Document system design
- Format markdown and syntax

**Capabilities**:
- API documentation (OpenAPI/Swagger)
- Technical writing
- Architecture documentation
- Changelog creation
- Markdown formatting
- Code documentation
- Design documentation
- Release notes

**Agents in This Type**:
- Technical Writer
- Documentation Expert
- API Documenter
- Documentation Lead (Phase 2)
- Changelog Generator
- Markdown Syntax Formatter
- Report Generator

**Model**: Haiku (documentation is structured) | Sonnet 4.5 for complex technical writing

---

### 8. planner (7 agents)

**Purpose**: Planning, coordination, and project organization

**When to Use**:
- Plan project timelines
- Decompose complex tasks
- Coordinate team workflows
- Clarify requirements
- Manage risk
- Plan research efforts
- Coordinate product strategy
- Manage project supervision

**Capabilities**:
- Task decomposition
- Project planning
- Requirement clarification
- Timeline estimation
- Risk assessment
- Workflow coordination
- Product strategy
- Research planning

**Agents in This Type**:
- Task Decomposition Expert
- Research Orchestrator
- Research Coordinator
- Query Clarifier
- Product Strategist
- Project Supervisor Orchestrator
- Risk Manager

**Model**: Sonnet 4.5 | Haiku for structured planning tasks

---

### 9. reviewer (3 agents)

**Purpose**: Code review and quality assessment

**When to Use**:
- Review code quality
- Assess architecture patterns
- Review implementation against specs
- Analyze code structure
- Cross-stack review

**Capabilities**:
- Code quality review
- Architecture pattern review
- Implementation verification
- Code analysis
- Best practices assessment

**Agents in This Type**:
- Code Reviewer
- Architect Review
- Flutter Go Reviewer

**Model**: Sonnet 4.5

---

### 10. debugger (2 agents)

**Purpose**: Error diagnosis, debugging, and troubleshooting

**When to Use**:
- Diagnose bugs from logs
- Analyze error patterns
- Troubleshoot production issues
- Debug complex systems
- Analyze stack traces

**Capabilities**:
- Error pattern analysis
- Log analysis
- Stack trace interpretation
- Bug diagnosis
- Troubleshooting methodology

**Agents in This Type**:
- Debugger
- Error Detective

**Model**: Sonnet 4.5

---

### 11. ux-designer (2 agents)

**Purpose**: UI/UX design and user interface development

**When to Use**:
- Design user interfaces
- Create UX specifications
- Design CLI interfaces
- Plan user flows
- Accessibility review

**Capabilities**:
- UI design
- UX specification
- User flow design
- Accessibility design
- Interaction design

**Agents in This Type**:
- UI/UX Designer
- CLI UI Designer

**Model**: Haiku for interface generation | Sonnet 4.5 for complex UX

---

### 12. python-expert (2 agents)

**Purpose**: Python language specialization

**When to Use**:
- Python backend development
- FastAPI/Django implementation
- Data science tasks
- Machine learning development
- Async Python patterns

**Capabilities**:
- FastAPI/Django expertise
- Async/await patterns
- Data science libraries
- ML frameworks
- Python best practices

**Agents in This Type**:
- Python Expert
- Python Pro

**Model**: Sonnet 4.5

---

### 13. ios-developer (2 agents)

**Purpose**: iOS development specialization

**When to Use**:
- SwiftUI development
- UIKit implementation
- iOS app architecture
- Native integrations
- Core Data/Combine

**Capabilities**:
- SwiftUI development
- UIKit patterns
- iOS architecture
- Native frameworks
- Performance optimization

**Agents in This Type**:
- Swift Expert
- iOS Developer

**Model**: Sonnet 4.5

---

### 14. mobile-developer (2 agents)

**Purpose**: Cross-platform mobile development

**When to Use**:
- Flutter app development
- State management (Bloc, Riverpod)
- Cross-platform UI
- Native integrations
- Mobile performance

**Capabilities**:
- Flutter development
- State management
- Cross-platform code
- Native integration
- Mobile optimization

**Agents in This Type**:
- Flutter Expert
- Mobile Developer

**Model**: Sonnet 4.5

---

### 15. coder (4 agents)

**Purpose**: General-purpose coding (rarely used, highly specific)

**When to Use**:
- Only when task doesn't fit other types
- Multi-language scripting
- Utility development
- Specialized coding tasks

**Capabilities**:
- General coding
- Multi-language support
- Utility scripting
- General implementation

**Agents in This Type** (Minimal - Post-Reconciliation):
- Unused Code Cleaner
- And 3 other highly specific coders

**Model**: Haiku (lightweight tasks) | Sonnet 4.5 for complex work

**Note**: This type is now minimal (4 agents) compared to pre-reconciliation (68 agents). Almost all work should use specialized types above.

---

## Agent Selection Guide

### How to Choose the Right Agent Type

**1. Identify the Task Category**
- Security review → `security-auditor`
- Database work → `backend-dev`
- Documentation → `technical-writer`
- Testing → `test-automator`
- Research/exploration → `researcher`
- Deployment → `deployment-engineer`
- Architecture → `system-architect`

**2. Match Specialization Level**
- Specialized type available? Use it (99% of cases)
- No specific type? Check sub-categories within type
- Still no match? Use `coder` type (rare, <3% of work)

**3. Consider Model Efficiency**
- Complex reasoning needed? Sonnet 4.5
- Structured/lightweight task? Haiku (cost savings)
- Strategic decision? Opus 4.1 (architects only)

---

## Type Distribution Analysis

### Agent Count by Type (119 Total)

| Type | Count | % of Fleet | Primary Model |
|------|-------|-----------|---------------|
| backend-dev | 30 | 25.9% | Sonnet 4.5 |
| researcher | 17 | 14.7% | Sonnet/Haiku |
| deployment-engineer | 15 | 12.9% | Sonnet/Haiku |
| system-architect | 10 | 8.6% | Sonnet 4.5 |
| test-automator | 6 | 5.2% | Sonnet 4.5 |
| planner | 7 | 6.0% | Sonnet/Haiku |
| security-auditor | 7 | 6.0% | Sonnet 4.5 |
| technical-writer | 7 | 6.0% | Haiku |
| reviewer | 3 | 2.6% | Sonnet 4.5 |
| debugger | 2 | 1.7% | Sonnet 4.5 |
| ux-designer | 2 | 1.7% | Sonnet/Haiku |
| python-expert | 2 | 1.7% | Sonnet 4.5 |
| ios-developer | 2 | 1.7% | Sonnet 4.5 |
| mobile-developer | 2 | 1.7% | Sonnet 4.5 |
| coder | 4 | 3.4% | Haiku |

### Model Distribution

| Model | Agent Count | % | Use Case |
|-------|------------|---|----------|
| Sonnet 4.5 | 37 | 31.1% | Complex reasoning, coding, security, architecture |
| Haiku | 81 | 68.1% | Documentation, utilities, structured tasks |
| Opus 4.1 | 1 | 0.8% | Chief Architect leadership |

### Cost Savings

- **Haiku agents**: 81 × $0.08/1k tokens = Cost-effective
- **Sonnet agents**: 37 × $0.60/1k tokens = Complex work
- **Monthly savings**: $600-900 (65-75% reduction)
- **Annual savings**: $7,200-10,800

---

## Implementation Recommendations

### Best Practices for Agent Type Usage

1. **Always use specific types** unless task is genuinely multi-domain
2. **Combine types for complex work** (e.g., backend-dev + security-auditor)
3. **Leverage haiku optimization** for documentation and utilities (68% cost savings)
4. **Reserve Opus for architecture** (Chief Architect leadership only)
5. **Use test-automator for TDD** (write tests before implementation)

### Anti-Patterns to Avoid

1. **Don't use "coder" for specialized tasks** (use backend-dev, security-auditor, etc.)
2. **Don't use Sonnet for simple documentation** (use Haiku technical-writer)
3. **Don't mix architectural review with coding** (use system-architect for design)
4. **Don't skip test-automator in TDD workflow** (ensures test-first methodology)

---

## Recent Changes (Reconciliation v2.1.0)

### What Changed
- **Before**: 129 config entries, 68 "coder" agents, 15 duplicates
- **After**: 119 agents, 4 "coder" agents, 0 duplicates
- **Impact**: 94% reduction in generic roles, improved orchestration accuracy

### Agents Added (6 total)
- Architect Review (reviewer type)
- Flutter Go Reviewer (reviewer type)
- Supabase Schema Architect (system-architect type)
- Supabase Realtime Optimizer (backend-dev type)
- Unused Code Cleaner (coder type, haiku)
- Web Vitals Optimizer (backend-dev type)

### Type Changes Applied
- 72 agents reassigned to correct types
- 39 agents optimized to Haiku model
- All security agents → security-auditor type
- All documentation agents → technical-writer type

### Quality Improvements
- Reduced configuration size by 255 lines (10% reduction)
- Eliminated all duplicates
- Improved agent selection accuracy
- Better cost optimization

---

## Future Enhancements

### Potential New Types (Under Consideration)

1. **database-admin** (DBA specialization)
2. **devops-architect** (Infrastructure design)
3. **ml-engineer** (Dedicated ML/AI type)
4. **frontend-dev** (Web frontend specialization)
5. **data-engineer** (Data pipeline specialization)

### Scaling Strategy

- Current: 119 agents across 13 sections (90 unique agent types)
- Target: 150-200 agents across 18-20 types
- Growth: Add specialized types as needs evolve
- Optimization: Continue haiku adoption for cost-effective agents

---

## Troubleshooting

### Issue: Wrong agent selected for task

**Solution**:
1. Verify task fits intended type description
2. Check if multiple types could apply
3. Create or update `CLAUDE.md` in project for hints
4. Explicitly request agent type in task description

### Issue: Agent quality seems low

**Solution**:
1. Check if agent is using correct model (Sonnet vs Haiku)
2. Verify task complexity matches model capability
3. Review knowledge base for relevant context
4. Consider combining multiple agent types

### Issue: Cost unexpectedly high

**Solution**:
1. Check agent-to-haiku ratio in operations
2. Identify high-token agents (use Haiku where possible)
3. Review task complexity—simplify if possible
4. Batch similar tasks for efficiency

---

## Related Documentation

- [CLAUDE.md](/Users/brent/git/cc-orchestra/CLAUDE.md) - Project configuration
- [TECHNICAL_OVERVIEW.md](TECHNICAL_OVERVIEW.md) - System architecture
- [RECONCILIATION_IMPLEMENTATION_REPORT.md](RECONCILIATION_IMPLEMENTATION_REPORT.md) - Detailed changes
- [README.md](../README.md) - Quick start
- [EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md) - High-level overview

---

**Version**: 2.1.0 (Reconciliation Edition)
**Last Updated**: 2025-11-10
**Status**: Complete and ready for production use
