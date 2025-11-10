---
name: project-discovery
description: Comprehensive project requirements discovery through interactive interviews. Mandatory for complex projects, optional for simple tasks. Creates detailed specification document before any implementation begins.
version: 1.0.0
tags: [discovery, requirements, architecture, planning, specification]
---

# Project Discovery Skill

## Overview

This skill guides the Architect through a comprehensive, adaptive interview process to discover all project requirements before any implementation begins. The process is:

- **Adaptive**: Skips irrelevant sections based on project type
- **Interactive**: Back-and-forth conversation with clarification rounds
- **Comprehensive**: 60-80 questions covering all critical aspects
- **Mandatory**: For complex projects (multi-tech, integrations, production systems)
- **Optional**: For simple projects (single-file, basic CRUD, prototypes)

## When to Activate

### Mandatory Activation
- Multi-technology projects (e.g., "Flutter app with Go backend")
- External integrations (Salesforce, Authentik, third-party APIs)
- Production/enterprise systems with security/compliance needs
- Projects requiring deployment infrastructure
- Projects with unclear or incomplete requirements

### Optional Activation
- Simple CRUD applications with clear requirements
- Single-technology prototypes
- Well-defined feature additions to existing projects
- User explicitly provides complete specification

### Skip Discovery
- Single-file changes
- Bug fixes
- Documentation updates
- Configuration changes

## Discovery Process

### Phase 0: Initial Assessment (5 questions)

**Purpose**: Determine project scope and which discovery sections are relevant.

```
Q1: What type of project is this?
   a) Web application (backend + frontend)
   b) Mobile application (iOS/Android/Flutter)
   c) Backend API/service only
   d) Data pipeline/ETL system
   e) Desktop application
   f) Microservices architecture
   g) Integration/sync service
   h) Other: [specify]

Q2: What is the primary purpose in one sentence?
   [Open-ended response]

Q3: Is this a new project or adding to existing codebase?
   a) New project (greenfield)
   b) Adding features to existing project
   c) Refactoring existing project
   d) Integrating with existing systems

Q4: What is the expected complexity level?
   a) Simple (single technology, no integrations, < 1 week)
   b) Medium (2-3 technologies, basic integrations, 1-4 weeks)
   c) Complex (multi-tech, multiple integrations, 1-3 months)
   d) Enterprise (large scale, compliance, > 3 months)

Q5: Do you have a complete written specification already?
   a) Yes, I have detailed specs (can skip most discovery)
   b) Partial specs (need to fill gaps)
   c) No, just a general idea (full discovery needed)
```

**Adaptive Logic**:
- If Q4 = "Simple" AND Q5 = "Yes" → Skip to Phase 7 (Definition of Done)
- If Q1 = "Backend API only" → Skip frontend questions
- If Q1 = "Mobile app" → Focus on mobile-specific questions
- If Q3 = "Adding to existing" → Focus on integration questions

---

### Phase 1: Project Foundation (5-8 questions)

**Activate if**: New project OR complex project

```
Q1.1: Who are the target users?
   a) Internal team/employees only
   b) External customers (B2C)
   c) Enterprise clients (B2B)
   d) Public/open-source
   e) Mixed (internal + external)

Q1.2: What is the expected scale?
   a) Small (< 100 users, development/testing)
   b) Medium (100-10,000 users)
   c) Large (10,000-1M users)
   d) Enterprise (> 1M users, global scale)

Q1.3: What is the timeline/urgency?
   a) Prototype/POC (days to 1 week)
   b) MVP (1-4 weeks)
   c) Production-ready (1-3 months)
   d) Enterprise rollout (> 3 months)

Q1.4: What are the top 3 critical features/requirements?
   [Open-ended: User lists 3 most important things]

Q1.5: What are the main success criteria?
   [Open-ended: How will we know this project succeeded?]

Q1.6: Are there any existing systems this must integrate with?
   a) No external integrations
   b) Yes: [list systems]

Q1.7: What is your budget for external services? (APIs, cloud, etc.)
   a) No budget / free-tier only
   b) Limited ($0-100/month)
   c) Medium ($100-1000/month)
   d) Enterprise (> $1000/month)

Q1.8: Any regulatory/compliance requirements?
   a) None
   b) GDPR (EU data privacy)
   c) HIPAA (healthcare)
   d) SOC2 (security audit)
   e) PCI-DSS (payment processing)
   f) Other: [specify]
```

**Clarification Round**:
After Phase 1, ask 2-3 follow-up questions based on answers:
- If "Enterprise clients" → "What specific enterprise features needed?"
- If "Compliance required" → "Which specific compliance requirements?"
- If "Existing integrations" → "Which systems and what data flows?"

---

### Phase 2: Technology Stack (15-20 questions)

**Activate if**: New project OR unclear tech stack

#### Backend Questions

```
Q2.1: Do you need a backend/API?
   a) Yes, REST API
   b) Yes, GraphQL API
   c) Yes, both REST and GraphQL
   d) No backend needed (frontend only)
   e) Serverless functions only

[If YES to backend:]

Q2.2: Preferred backend language?
   a) Python (FastAPI/Django/Flask)
   b) Go (Gin/Echo/net/http)
   c) Rust (Actix/Rocket)
   d) Node.js/TypeScript
   e) No preference (Architect decides)

Q2.3: Preferred backend framework?
   [Based on Q2.2 answer, list relevant frameworks]
   a) [Language-specific options]
   b) No preference

Q2.4: Database type needed?
   a) PostgreSQL (relational)
   b) MySQL (relational)
   c) MongoDB (document)
   d) Redis (cache/key-value)
   e) Multiple databases
   f) No database / file-based
   g) Not sure (Architect decides)

Q2.5: Database size expectations?
   a) Small (< 1 GB)
   b) Medium (1-100 GB)
   c) Large (100 GB - 1 TB)
   d) Very large (> 1 TB)

Q2.6: Caching layer needed?
   a) Yes, Redis
   b) Yes, Memcached
   c) Yes, in-memory cache
   d) No caching needed
   e) Not sure

Q2.7: Background job processing needed?
   a) Yes (Celery/RQ/etc.)
   b) No
   c) Not sure
```

#### Frontend Questions

**Activate if**: Q1 (Phase 0) = Web app OR Mobile app

```
Q2.8: Frontend type needed?
   a) Web (browser-based)
   b) Mobile (iOS/Android)
   c) Both web and mobile
   d) Desktop application
   e) No frontend (API only)

[If Web:]

Q2.9: Web frontend framework?
   a) React
   b) Vue.js
   c) Vanilla JavaScript (no framework)
   d) Server-side rendered (Jinja/templates)
   e) No preference

Q2.10: State management approach?
   a) Redux/Zustand
   b) Context API
   c) None needed
   d) Not sure

[If Mobile:]

Q2.11: Mobile platform?
   a) Flutter (cross-platform)
   b) React Native (cross-platform)
   c) Native iOS (Swift/SwiftUI)
   d) Native Android (Kotlin)
   e) Both native iOS and Android

Q2.12: Mobile state management?
   a) Provider/Riverpod (Flutter)
   b) Redux/MobX (React Native)
   c) Not sure

Q2.13: Offline support needed?
   a) Yes, must work offline
   b) No, online only
   c) Partial offline capability
```

#### Infrastructure Questions

```
Q2.14: Where will this be deployed?
   a) AWS (Amazon Web Services)
   b) GCP (Google Cloud Platform)
   c) Azure (Microsoft)
   d) On-premise servers
   e) Heroku/Render/Railway
   f) Not sure yet

Q2.15: Containerization approach?
   a) Docker only
   b) Docker + Docker Compose
   c) Kubernetes
   d) AWS ECS/Fargate
   e) No containers (traditional deployment)
   f) Serverless (Lambda/Cloud Functions)

Q2.16: CI/CD pipeline?
   a) GitHub Actions
   b) GitLab CI
   c) Jenkins
   d) CircleCI
   e) No CI/CD needed
   f) Not sure

Q2.17: Infrastructure as Code?
   a) Terraform
   b) CloudFormation
   c) Pulumi
   d) Manual configuration
   e) Not needed
```

**Clarification Round**:
- If multiple databases → "How will they be used together?"
- If Kubernetes → "Self-managed or managed service (EKS/GKE)?"
- If serverless → "Which functions and trigger patterns?"

---

### Phase 3: Integration Requirements (10-15 questions)

**Activate if**: Any external integrations mentioned

#### Salesforce Integration

**Activate if**: User mentioned Salesforce

```
Q3.1: Salesforce integration needed?
   a) Yes, heavily used
   b) Yes, limited use
   c) No

[If YES:]

Q3.2: Which Salesforce API?
   a) REST API
   b) SOAP API
   c) Bulk API 2.0
   d) Streaming API
   e) Multiple APIs
   f) Not sure

Q3.3: Which Salesforce objects will you work with?
   [Checklist]
   - [ ] Lead
   - [ ] Contact
   - [ ] Account
   - [ ] Opportunity
   - [ ] Case
   - [ ] Custom objects: [specify]

Q3.4: Salesforce API version?
   a) Latest (v59.0+)
   b) Specific version: [specify]
   c) Not sure

Q3.5: Data flow direction?
   a) Read from Salesforce only
   b) Write to Salesforce only
   c) Bidirectional sync
   d) Real-time events (Platform Events/Change Data Capture)

Q3.6: Salesforce environment?
   a) Production org
   b) Sandbox
   c) Both (separate configs)

Q3.7: Salesforce OAuth setup?
   a) Already have Connected App
   b) Need to create Connected App
   c) Not sure what this means
```

#### Authentik Integration

**Activate if**: User mentioned Authentik OR authentication requirements

```
Q3.8: Authentication method needed?
   a) Authentik (OAuth2/OIDC/SAML)
   b) Custom JWT authentication
   c) Simple username/password
   d) API keys only
   e) No authentication needed

[If Authentik:]

Q3.9: Authentik provider type?
   a) OAuth2/OIDC
   b) SAML 2.0
   c) LDAP
   d) Proxy provider
   e) Multiple providers

Q3.10: Authentik instance location?
   a) Self-hosted: [URL]
   b) Cloud-hosted: [URL]

Q3.11: Which applications in Authentik?
   [List applications that need integration]

Q3.12: User provisioning needed?
   a) Yes, auto-create users from Authentik
   b) Yes, sync users bidirectionally
   c) No, manual user management
   d) Not sure

Q3.13: MFA enforcement?
   a) Required for all users
   b) Optional per user
   c) Required for admins only
   d) Not needed

Q3.14: User attributes to sync?
   [Checklist]
   - [ ] Email
   - [ ] Name (first/last)
   - [ ] Groups/roles
   - [ ] Custom attributes: [specify]
```

#### Other Integrations

```
Q3.15: Payment processing integration?
   a) Stripe
   b) PayPal
   c) Square
   d) Other: [specify]
   e) None needed

Q3.16: Email service integration?
   a) SendGrid
   b) AWS SES
   c) Mailgun
   d) SMTP server
   e) None needed

Q3.17: Other third-party APIs needed?
   [Open-ended: List any other APIs]

Q3.18: Webhooks needed?
   a) Receive webhooks from external services
   b) Send webhooks to external services
   c) Both
   d) None
```

**Clarification Round**:
- If Salesforce sync → "What triggers the sync? Real-time or scheduled?"
- If Authentik → "What happens when user is deleted in Authentik?"
- If payment processing → "One-time payments, subscriptions, or both?"

---

### Phase 4: Security & Compliance (10-15 questions)

**Activate if**: Production system OR sensitive data OR compliance mentioned

#### Authentication & Authorization

```
Q4.1: User roles needed?
   a) Single role (all users same permissions)
   b) Admin and regular users
   c) Multiple roles: [list roles]
   d) Fine-grained permissions (RBAC)

Q4.2: Session management?
   a) JWT tokens
   b) Session cookies
   c) OAuth tokens
   d) Not sure

Q4.3: Token expiration?
   a) Short-lived (15 mins) with refresh
   b) Medium (1 hour)
   c) Long-lived (24+ hours)
   d) Not sure

Q4.4: Multi-factor authentication (MFA)?
   a) Required for all users
   b) Optional per user
   c) Required for sensitive operations
   d) Not needed

Q4.5: Password policy?
   a) Strong (length, complexity, rotation)
   b) Medium (length + complexity)
   c) Weak (length only)
   d) No passwords (OAuth only)
```

#### Data Security

```
Q4.6: Data encryption at rest?
   a) Yes, database encryption required
   b) Yes, file encryption required
   c) Both database and files
   d) Not needed

Q4.7: Data encryption in transit?
   a) HTTPS/TLS for all endpoints
   b) HTTPS for sensitive endpoints only
   c) Not needed

Q4.8: Sensitive data types stored?
   [Checklist]
   - [ ] Passwords (hashed)
   - [ ] PII (personal info)
   - [ ] Payment information
   - [ ] Health information
   - [ ] API keys/secrets
   - [ ] Other: [specify]

Q4.9: Data retention policy?
   a) Keep all data indefinitely
   b) Delete after X days: [specify]
   c) Archive after X days: [specify]
   d) User-controlled deletion
   e) Not sure

Q4.10: Data backup requirements?
   a) Daily automated backups
   b) Weekly backups
   c) Real-time replication
   d) No backups needed
```

#### Credentials & Access

```
Q4.11: Where should credentials be stored?
   a) AWS Secrets Manager
   b) HashiCorp Vault
   c) 1Password
   d) Environment variables
   e) Encrypted config files
   f) Not sure

Q4.12: API keys/credentials needed for:
   [Open-ended: List all services requiring credentials]

Q4.13: Credential rotation frequency?
   a) Every 30 days
   b) Every 90 days
   c) Annually
   d) Never (manual only)
   e) Not sure

Q4.14: Who has access to production credentials?
   a) Only ops team
   b) All developers
   c) Automated systems only
   d) Not defined yet
```

#### Compliance

```
Q4.15: Audit logging required?
   a) Yes, all user actions
   b) Yes, sensitive operations only
   c) No
   d) Not sure

Q4.16: Data residency requirements?
   a) Must stay in specific region: [specify]
   b) No restrictions

Q4.17: Third-party security audits needed?
   a) Yes, SOC 2 compliance
   b) Yes, penetration testing
   c) Yes, other: [specify]
   d) No
```

**Clarification Round**:
- If PII storage → "What is your GDPR/privacy compliance plan?"
- If multiple roles → "Can you describe each role's permissions?"
- If credential rotation → "What's the rotation process?"

---

### Phase 5: Quality Requirements (10 questions)

**Activate if**: Production system OR complex project

#### Testing

```
Q5.1: Required test types?
   [Checklist]
   - [ ] Unit tests
   - [ ] Integration tests
   - [ ] End-to-end (E2E) tests
   - [ ] Performance tests
   - [ ] Security tests
   - [ ] Load tests

Q5.2: Minimum test coverage?
   a) 90%+ (critical system)
   b) 70-90% (production)
   c) 50-70% (standard)
   d) < 50% (MVP/prototype)
   e) No requirement

Q5.3: Test data strategy?
   a) Fixtures/mock data
   b) Database seeds
   c) Production data copy (anonymized)
   d) Not sure

Q5.4: Performance benchmarks?
   a) API response < 200ms
   b) API response < 500ms
   c) API response < 1s
   d) No specific requirement

Q5.5: Load testing requirements?
   a) Yes, must handle X concurrent users: [specify]
   b) Yes, must handle X requests/second: [specify]
   c) No load testing needed
```

#### Documentation

```
Q5.6: API documentation format?
   a) OpenAPI/Swagger
   b) Postman collections
   c) Markdown docs
   d) Code comments only
   e) Not needed

Q5.7: User documentation needed?
   a) Yes, comprehensive user guide
   b) Yes, quick start guide
   c) No user docs

Q5.8: Developer documentation?
   a) Yes, comprehensive (architecture, setup, deployment)
   b) Yes, basic (README + setup)
   c) Code comments only
   d) Not needed

Q5.9: Architecture diagrams required?
   a) Yes, detailed (C4 model / UML)
   b) Yes, high-level overview
   c) Not needed
```

#### Code Quality

```
Q5.10: Code quality standards?
   a) Strict (linting, formatting, static analysis)
   b) Standard (linting + formatting)
   c) Basic (formatting only)
   d) No specific standards

Q5.11: Code review process?
   a) Required for all changes
   b) Required for critical code
   c) Optional
   d) Not needed
```

**Clarification Round**:
- If high test coverage → "What's your CI/CD test automation strategy?"
- If performance benchmarks → "How will you monitor performance in production?"

---

### Phase 6: Deployment & Operations (10-15 questions)

**Activate if**: Production deployment OR infrastructure needed

#### Deployment

```
Q6.1: Deployment target environment?
   a) AWS ECS/Fargate
   b) Kubernetes (EKS/GKE/AKS)
   c) AWS Lambda (serverless)
   d) Heroku/Render/Railway
   e) Traditional VMs
   f) Local/development only

Q6.2: Number of environments?
   a) Development only
   b) Dev + Staging
   c) Dev + Staging + Production
   d) Dev + Staging + Production + DR

Q6.3: Deployment frequency?
   a) Continuous (on every merge)
   b) Daily
   c) Weekly
   d) On-demand
   e) Not sure

Q6.4: Deployment strategy?
   a) Blue-green deployment
   b) Canary deployment
   c) Rolling update
   d) Direct replacement
   e) Not sure

Q6.5: Rollback requirements?
   a) Automated rollback on failure
   b) Manual rollback capability
   c) No rollback needed (careful testing)

Q6.6: Zero-downtime requirement?
   a) Yes, no downtime allowed
   b) Short maintenance windows acceptable
   c) Downtime not a concern
```

#### Monitoring & Observability

```
Q6.7: Logging solution?
   a) CloudWatch (AWS)
   b) Stackdriver (GCP)
   c) ELK Stack (Elasticsearch/Logstash/Kibana)
   d) Datadog
   e) Simple file logging
   f) Not sure

Q6.8: Metrics/monitoring?
   a) Prometheus + Grafana
   b) CloudWatch/Stackdriver
   c) Datadog
   d) New Relic
   e) Not needed
   f) Not sure

Q6.9: Alerting needed?
   a) Yes, PagerDuty
   b) Yes, email alerts
   c) Yes, Slack notifications
   d) No alerting
   e) Not sure

Q6.10: What to monitor/alert on?
   [Checklist]
   - [ ] Error rate > X%
   - [ ] Response time > Xms
   - [ ] CPU/memory usage > X%
   - [ ] Failed jobs/tasks
   - [ ] Security events
   - [ ] Custom business metrics
```

#### Reliability

```
Q6.11: Uptime requirements?
   a) 99.99% (< 1 hour downtime/year)
   b) 99.9% (< 9 hours downtime/year)
   c) 99% (< 4 days downtime/year)
   d) Best effort

Q6.12: Auto-scaling needed?
   a) Yes, based on CPU/memory
   b) Yes, based on request rate
   c) Yes, scheduled scaling
   d) No, fixed capacity

Q6.13: Disaster recovery plan?
   a) Full DR with separate region
   b) Backups + restore procedure
   c) Not needed
   d) Not sure

Q6.14: Health check endpoints?
   a) Yes, /health with full checks
   b) Yes, /health basic only
   c) Not needed
```

**Clarification Round**:
- If zero-downtime → "What's your database migration strategy?"
- If auto-scaling → "What are the scaling trigger thresholds?"
- If DR plan → "What's the acceptable RTO/RPO?"

---

### Phase 7: Definition of Done (5-8 questions)

**MANDATORY FOR ALL PROJECTS**

```
Q7.1: What does "done" mean for this project?
   [Open-ended: User describes completion criteria]

Q7.2: Acceptance criteria checklist:
   [Checklist - user specifies what must be true]
   - [ ] All features implemented
   - [ ] Tests passing (specify coverage: ___%)
   - [ ] Security audit complete
   - [ ] Documentation complete
   - [ ] Deployed to production
   - [ ] Performance benchmarks met
   - [ ] Other: [specify]

Q7.3: What is NOT in scope?
   [Open-ended: Explicitly exclude features/requirements]

Q7.4: When should we checkpoint progress?
   a) After each feature complete
   b) After each component complete
   c) After each agent completes work
   d) Only at final completion
   e) Custom: [specify milestones]

Q7.5: How will success be measured?
   [Open-ended: Metrics, KPIs, outcomes]

Q7.6: What could cause this project to fail?
   [Open-ended: Risk factors to monitor]

Q7.7: Who approves final completion?
   a) You (the user)
   b) Architect reviews and approves
   c) Automated tests passing = done
   d) Specific person: [specify]

Q7.8: Post-deployment monitoring period?
   a) Monitor for 24 hours
   b) Monitor for 1 week
   c) Monitor for 1 month
   d) Ongoing monitoring (no specific period)
   e) No monitoring needed
```

---

## Specification Document Generation

After completing discovery, Architect generates a comprehensive specification document.

### Specification Structure

```markdown
# [Project Name] - Complete Specification

**Generated**: [Date]
**Architect**: Claude Orchestra Chief Architect
**Discovery Completion**: [Percentage of questions answered]

---

## Executive Summary

### Project Overview
[1-2 paragraph description]

### Target Users
[From Q1.1]

### Expected Scale
[From Q1.2]

### Timeline
[From Q1.3]

### Success Criteria
[From Q7.5]

---

## Technology Stack

### Backend
- **Language**: [From Q2.2]
- **Framework**: [From Q2.3]
- **Database**: [From Q2.4]
- **Caching**: [From Q2.6]
- **Background Jobs**: [From Q2.7]

### Frontend
- **Type**: [From Q2.8]
- **Framework**: [From Q2.9/Q2.11]
- **State Management**: [From Q2.10/Q2.12]
- **Offline Support**: [From Q2.13]

### Infrastructure
- **Cloud Provider**: [From Q2.14]
- **Containerization**: [From Q2.15]
- **CI/CD**: [From Q2.16]
- **IaC**: [From Q2.17]

---

## Architecture Design

### System Architecture
[Architect creates diagram based on answers]

### Component Breakdown
[Based on technology stack and requirements]

### Data Flow
[How data moves through the system]

### Integration Points
[All external systems/APIs]

---

## External Integrations

### Salesforce Integration
[If applicable - from Phase 3]
- **API Type**: [Q3.2]
- **Objects**: [Q3.3]
- **Data Flow**: [Q3.5]
- **Environment**: [Q3.6]
- **OAuth Setup**: [Q3.7]

### Authentik Integration
[If applicable - from Phase 3]
- **Provider Type**: [Q3.9]
- **Instance**: [Q3.10]
- **Applications**: [Q3.11]
- **User Provisioning**: [Q3.12]
- **MFA**: [Q3.13]
- **User Attributes**: [Q3.14]

### Other Integrations
- **Payment**: [Q3.15]
- **Email**: [Q3.16]
- **Third-party APIs**: [Q3.17]
- **Webhooks**: [Q3.18]

---

## Security Architecture

### Authentication
- **Method**: [Q4.1, Q3.8]
- **Session Management**: [Q4.2]
- **Token Expiration**: [Q4.3]
- **MFA**: [Q4.4]
- **Password Policy**: [Q4.5]

### Authorization
- **User Roles**: [Q4.1]
- **Permissions Model**: [Details]

### Data Security
- **Encryption at Rest**: [Q4.6]
- **Encryption in Transit**: [Q4.7]
- **Sensitive Data**: [Q4.8]
- **Retention Policy**: [Q4.9]
- **Backup**: [Q4.10]

### Credentials Management
- **Storage**: [Q4.11]
- **Required Credentials**: [Q4.12]
- **Rotation**: [Q4.13]
- **Access Control**: [Q4.14]

### Compliance
- **Requirements**: [From Q1.8]
- **Audit Logging**: [Q4.15]
- **Data Residency**: [Q4.16]
- **Security Audits**: [Q4.17]

---

## Quality Standards

### Testing
- **Test Types**: [Q5.1]
- **Coverage Target**: [Q5.2]
- **Test Data**: [Q5.3]
- **Performance**: [Q5.4]
- **Load Testing**: [Q5.5]

### Documentation
- **API Docs**: [Q5.6]
- **User Docs**: [Q5.7]
- **Developer Docs**: [Q5.8]
- **Architecture Diagrams**: [Q5.9]

### Code Quality
- **Standards**: [Q5.10]
- **Review Process**: [Q5.11]

---

## Deployment Strategy

### Environments
- **Target**: [Q6.1]
- **Number**: [Q6.2]

### Deployment Process
- **Frequency**: [Q6.3]
- **Strategy**: [Q6.4]
- **Rollback**: [Q6.5]
- **Zero-Downtime**: [Q6.6]

### Monitoring
- **Logging**: [Q6.7]
- **Metrics**: [Q6.8]
- **Alerting**: [Q6.9]
- **Alert Conditions**: [Q6.10]

### Reliability
- **Uptime SLA**: [Q6.11]
- **Auto-Scaling**: [Q6.12]
- **Disaster Recovery**: [Q6.13]
- **Health Checks**: [Q6.14]

---

## Definition of Done

### Completion Criteria
[From Q7.1]

### Acceptance Checklist
[From Q7.2]

### Out of Scope
[From Q7.3]

### Checkpoints
[From Q7.4]

### Success Metrics
[From Q7.5]

### Risk Factors
[From Q7.6]

### Approval Process
[From Q7.7]

### Post-Deployment
[From Q7.8]

---

## Agent Assignments

Based on this specification, the following agents will be deployed:

### Coding Agents
- [ ] **Python Expert**: [Specific responsibilities]
- [ ] **Swift Expert**: [Specific responsibilities]
- [ ] **Go Expert**: [Specific responsibilities]
- [ ] **Rust Expert**: [Specific responsibilities]
- [ ] **Flutter Expert**: [Specific responsibilities]

### Integration Agents
- [ ] **API Explorer**: [APIs to explore]
- [ ] **Salesforce API Expert**: [Salesforce integration tasks]
- [ ] **Authentik API Expert**: [Auth integration tasks]

### Support Agents
- [x] **Documentation Lead**: [Documentation to create]
- [x] **QA Engineer**: [Testing requirements]
- [x] **Security Auditor**: [Security review scope]
- [x] **Credential Manager**: [Credentials to manage]
- [x] **DevOps Engineer**: [Infrastructure to build]

---

## Implementation Timeline

### Phase 1: Foundation (X hours)
- Architecture setup
- Project scaffolding
- CI/CD pipeline

### Phase 2: Core Features (X hours)
- [List core features]

### Phase 3: Integrations (X hours)
- [List integrations]

### Phase 4: Testing & Security (X hours)
- Comprehensive testing
- Security audit
- Performance optimization

### Phase 5: Deployment (X hours)
- Production deployment
- Monitoring setup
- Documentation finalization

**Total Estimated Time**: [Sum of phases]

---

## Memory Checkpoint Plan

### Critical Information to Persist
- Architecture decisions
- Technology stack choices
- Integration configurations
- Security requirements
- API credentials and access methods
- File organization structure
- Test strategies
- Deployment procedures
- Definition of Done criteria

### Checkpoint Triggers
[From Q7.4]

### Memory Keys Structure
```
architect/specification
architect/decisions
project/requirements
project/definition-of-done
{agent-name}/state
{agent-name}/progress
credentials/inventory
credentials/access-methods
integrations/salesforce/config
integrations/authentik/config
```

---

**This specification has been reviewed and approved by the user before implementation begins.**

**User Approval Signature**: ________________
**Date**: ________________
```

---

## Usage Instructions for Architect

### 1. Activate Discovery Skill

When complex project is detected:

```
I'm activating the Project Discovery skill to ensure we have complete requirements
before beginning implementation. This comprehensive interview will take 10-20 minutes
but will save hours of rework later.

Let's begin with Phase 0: Initial Assessment...
```

### 2. Conduct Adaptive Interview

- Ask questions from each phase
- Skip irrelevant sections based on answers
- Ask 2-3 clarifying questions after each phase
- Keep conversational and friendly
- Explain why you're asking when helpful

### 3. Generate Specification

After completing discovery:

```
Thank you for the detailed information. I'm now generating a comprehensive
specification document based on your answers. This will take a moment...

[Generate specification using template above]

Here is the complete specification for your review. Please read through and
confirm approval before we begin implementation.
```

### 4. Store in Memory

Once user approves:

```bash
# Store complete specification
npx claude-flow@alpha memory store \
  --key "architect/specification" \
  --value "$(cat specification.md)"

# Store definition of done
npx claude-flow@alpha memory store \
  --key "project/definition-of-done" \
  --value "$(cat dod.json)"

# Store credentials access methods
npx claude-flow@alpha memory store \
  --key "credentials/access-methods" \
  --value "$(cat credentials-config.json)"
```

### 5. Spawn Agents with Context

All agents receive:
- Link to complete specification in memory
- Their specific responsibilities
- Definition of done criteria
- Checkpoint triggers

```javascript
Task("Python Expert",
  "Retrieve specification from memory: architect/specification.
   Implement backend API according to spec.
   Your responsibilities: [from spec].
   Checkpoint when: [from Q7.4]",
  "python-expert", "sonnet")
```

---

## Compaction Protocol Integration

### Before Compaction

Architect automatically runs:

```bash
# Export all critical state
./scripts/pre-compaction.sh

# Stores:
# - Complete specification
# - All architecture decisions
# - Agent states and progress
# - Credentials and access methods
# - Integration configurations
# - Definition of done
# - Checkpoint history
```

### After Compaction

Architect automatically runs:

```bash
# Restore all critical state
./scripts/post-compaction.sh

# Retrieves from memory:
# - Specification
# - Decisions
# - Agent states
# - All context

# Broadcasts to all agents:
npx claude-flow@alpha hooks notify \
  --message "Session restored. Review memory for project context."
```

All agents retrieve their state and continue seamlessly.

---

## Example Usage

### Complex Project (Full Discovery)

```
User: "Build a mobile app for managing sales leads with Salesforce sync"

Architect: I'm detecting this is a complex project (mobile + integration).
Let me activate the Project Discovery skill to gather complete requirements.

[Runs full discovery - 60-70 questions]
[Generates specification]
[User approves]
[Spawns: Architect, Flutter, Salesforce API, Python, QA, Security, DevOps, Docs, Credentials]

Result: Complete spec, all agents have full context, Definition of Done clear
```

### Simple Project (Quick Start)

```
User: "Create a simple Python script to parse CSV files"

Architect: This appears to be a simple project. Would you like:
a) Full discovery process (comprehensive)
b) Quick start (minimal questions)

User: Quick start

Architect: Great. Just a few questions:
[Asks 5-8 essential questions]
[Creates minimal spec]
[Spawns: Python Expert, QA, Docs]

Result: Fast start, focused implementation
```

---

## Success Metrics

This skill ensures:

✅ **Zero ambiguity** - All requirements captured upfront
✅ **Complete context** - Architect and agents have full picture
✅ **Clear success criteria** - Definition of Done defined
✅ **No rework** - Fewer changes mid-project
✅ **Persistent memory** - Survives compactions
✅ **Credential tracking** - All access methods documented
✅ **Quality standards** - Testing/docs/security requirements clear

---

**End of Project Discovery Skill**
