# Spawn Agents: Agent Selection Guide

## Trigger
`/spawn-agents <requirement or role>`

Helps select the right agents for a task by listing all 117 agents organized by capability.

## Purpose
The Spawn Agents skill provides:
- Complete inventory of all 117 agents
- Organization by role and model tier
- Quick lookup for agent selection
- Model tier explanation
- Combination recommendations

## Model Tiers

| Model | Count | Capability | Ideal For | Speed |
|-------|-------|-----------|-----------|-------|
| **Opus** | 1 | Strategic thinking, complex reasoning | Architecture, coordination, decisions | Slower but most capable |
| **Sonnet** | 35 | Intelligent problem solving, complex coding | Code review, security, debugging, complex tasks | Balanced |
| **Haiku** | 81 | Fast focused execution, simple to moderate coding | Implementation, documentation, utilities, research | Fast |

## Agent Inventory by Category

### Leadership (1 - Opus)
- **Chief Architect**: Strategic decisions, architecture design, orchestration

### Code Review & Quality (8 - Sonnet)
- Code Reviewer, Architect Review, Error Detective, Review Agent, Flutter-Go Reviewer, Test Automator, Test Engineer, Test Engineer

### Backend & API (6 - Sonnet)
- Backend Architect, API Explorer, API Security Audit, API Documenter, API Specialist, GraphQL Architect

### Security (4 - Sonnet)
- Security Auditor, Security Engineer, Penetration Tester, MCP Security Auditor

### Infrastructure & DevOps (8 - Sonnet)
- DevOps Engineer, Cloud Architect, Terraform Specialist, Deployment Engineer, Monitoring Specialist, Network Engineer, Database Admin, Database Architect

### Data & ML (4 - Sonnet)
- Data Engineer, Data Scientist, ML Engineer, MLOps Engineer

### Performance (3 - Sonnet)
- Performance Engineer, Performance Profiler, GraphQL Performance Optimizer

### Documentation & Knowledge (3 - Sonnet)
- Documentation Expert, Research Coordinator, Research Synthesizer

### Language Specialists (15 - Haiku)
- Python, Python Pro, Go, Golang Pro, Rust, Rust Pro, JavaScript Pro, TypeScript Pro, Swift, iOS Developer, Flutter Specialist, Mobile Developer, Shell Scripting Pro, SQL Pro, SQL Pro

### Frontend & Web (8 - Haiku)
- Frontend Developer, React Performance Optimization, React Performance Optimizer, NextJS Architecture Expert, UI/UX Designer, Web Accessibility Checker, Web Vitals Optimizer, CLI/UI Designer

### Framework & Library Specialists (6 - Haiku)
- GraphQL Security Specialist, Supabase Realtime Optimizer, Supabase Schema Architect, Authentik API Specialist, Salesforce API Specialist, MCP Integration Engineer

### Documentation (8 - Haiku)
- Technical Writer, API Documenter, Documentation Expert, Changelog Generator, Markdown Syntax Formatter, Report Generator, Research Brief Generator, Fact Checker

### Analysis & Research (11 - Haiku)
- Technical Researcher, Comprehensive Researcher, Academic Researcher, Data Analyst, Search Specialist, URL Link Extractor, Business Analyst, Product Strategist, Quant Analyst, Risk Manager, Compliance Specialist

### Utilities & Automation (14 - Haiku)
- TDD Coding Agent, Git Flow Manager, DX Optimizer, Query Clarifier, Task Decomposition Expert, Project Supervisor Orchestrator, Context Manager, Metadata Agent, Tag Agent, Debugger, Dependency Manager, Command Expert, Unused Code Cleaner, Connection Agent

### Modernization & Infrastructure (10 - Haiku)
- Legacy Modernizer, Architecture Modernizer, Cloud Migration Specialist, Database Optimizer, LLMs Maintainer, MCP Server Architect, MCP Protocol Specialist, MCP Testing Engineer, MCP Deployment Orchestrator, MCP Expert

### Specialized Roles (9 - Haiku)
- AI Engineer, Prompt Engineer, Model Evaluator, Incident Responder, Load Testing Specialist, Fullstack Developer, Content Marketer, Document Structure Analyzer, MCP Expert

## Quick Lookup by Task Type

**Building a Feature:** TDD Agent → Language Specialist → Code Reviewer → Security Auditor → Technical Writer

**Debugging a Bug:** Error Detective → Debugger → TDD Agent → Language Specialist → Test Engineer

**Refactoring Code:** Architect Review → TDD Agent → Language Specialist → Performance Engineer → Code Reviewer

**Performance Optimization:** Performance Engineer → Performance Profiler → Database Architect → Language Specialist → Test Engineer

**Security Review:** Security Auditor → Security Engineer → Penetration Tester → Code Reviewer → Compliance Specialist

**Building Backend API:** Backend Architect → API Explorer → API Security Audit → Language Specialist → API Documenter

**Building Mobile App:** Frontend Developer / Flutter Specialist → TDD Agent → UI/UX Designer

**Infrastructure/DevOps:** DevOps Engineer → Cloud Architect → Terraform Specialist → Deployment Engineer → Monitoring Specialist

**Database Work:** Database Architect → Database Optimizer → Database Admin → Data Engineer → SQL Pro

**Data Science/ML:** Data Scientist → ML Engineer → Data Engineer → MLOps Engineer → Data Analyst

**Documentation:** Technical Writer → API Documenter → Documentation Expert → Code Reviewer → Fact Checker

**Research/Analysis:** Technical Researcher → Comprehensive Researcher → Academic Researcher → Research Synthesizer → Search Specialist

## Agent Combination Patterns

**Classic Feature Team (5 agents):**
TDD Agent + Language Specialist + Code Reviewer + Security Auditor + Technical Writer

**Full Stack Team (8 agents):**
TDD Agent + Backend Architect + Language Specialist + Frontend Developer + Code Reviewer + Security Auditor + Test Engineer + Technical Writer

**Infrastructure Team (6 agents):**
DevOps Engineer + Cloud Architect + Terraform Specialist + Deployment Engineer + Monitoring Specialist + Network Engineer

## Selection Workflow

1. Identify task type (feature, bug, refactor, infrastructure, etc.)
2. Check quick lookup table for standard team
3. Adjust for specifics (add security for sensitive data, performance if critical)
4. Spawn all agents at once to maximize parallelization
5. Let specialists work independently

## Output Format

Return organized listing with:
- Recommended agent team for the task
- Rationale for each agent selection
- Model tier breakdown
- Parallel execution plan
- Estimated timeline
- Key dependencies between agents
