# Agent Type and Role Field Mapping

**Purpose:** Maps all 117 agents to recommended `type` and `role` field values
**Date:** November 15, 2025
**Status:** Ready for Implementation

---

## Quick Reference: Type Categories

| Type | Count | Examples | Model Distribution |
|------|-------|----------|-------------------|
| language-specialist | 10 | rust-specialist, python-specialist, go-specialist | Haiku (10/10) |
| framework-specialist | 8 | nextjs-expert, react-expert, graphql-specialist | Haiku-Sonnet |
| test-engineer | 5 | test-automator, test-engineer, mcp-testing-engineer | Haiku-Sonnet |
| code-reviewer | 3 | code-reviewer, architect-review, debugger | Sonnet (3/3) |
| security-specialist | 8 | security-auditor, penetration-tester, api-security-audit | Sonnet (8/8) |
| devops-engineer | 3 | devops-engineer, deployment-engineer, cloud-architect | Haiku-Sonnet |
| database-specialist | 8 | database-architect, nosql-specialist, database-admin | Haiku-Sonnet |
| documentation-specialist | 6 | technical-writer, documentation-expert, changelog-generator | Haiku (6/6) |
| research-specialist | 10 | technical-researcher, academic-researcher, research-orchestrator | Haiku-Sonnet |
| architect | 1 | chief-architect | Opus (1/1) |
| **TOTAL** | **117** | — | — |

---

## Complete Agent Mapping

### Architecture & Leadership (2 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| chief-architect | opus | architect | Strategic decision-making and orchestra coordination |
| architect-review | sonnet | code-reviewer | Architecture consistency and SOLID principles review |

---

### Programming Languages (10 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| rust-specialist | haiku | language-specialist | Rust systems programming and performance expert |
| rust-pro | haiku | language-specialist | Rust professional and advanced implementation |
| python-specialist | haiku | language-specialist | Python FastAPI, Django, and ML integration specialist |
| python-pro | haiku | language-specialist | Python professional with advanced patterns |
| go-specialist | haiku | language-specialist | Go microservices and cloud-native specialist |
| golang-pro | haiku | language-specialist | Go professional and advanced implementation |
| swift-specialist | haiku | language-specialist | Swift iOS and native development specialist |
| flutter-specialist | haiku | language-specialist | Flutter cross-platform mobile development specialist |
| javascript-pro | haiku | language-specialist | JavaScript professional with advanced patterns |
| typescript-pro | haiku | language-specialist | TypeScript professional with advanced patterns |

---

### Web Frameworks & Frontend (12 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| nextjs-architecture-expert | sonnet | framework-specialist | Next.js architecture and performance optimization |
| react-performance-optimizer | sonnet | framework-specialist | React performance and optimization expert |
| react-performance | sonnet | framework-specialist | React performance optimization specialist |
| graphql-specialist | sonnet | framework-specialist | GraphQL API design and optimization specialist |
| graphql-designer | haiku | framework-specialist | GraphQL schema design and implementation |
| graphql-performance-optimizer | sonnet | framework-specialist | GraphQL performance optimization expert |
| graphql-security-specialist | sonnet | framework-specialist | GraphQL security and vulnerability specialist |
| web-vitals-optimizer | sonnet | framework-specialist | Web Vitals and core metrics optimization |
| frontend-developer | haiku | language-specialist | Frontend development and UI implementation |
| ui-ux-designer | haiku | language-specialist | UI/UX design and user experience specialist |
| web-accessibility-checker | haiku | language-specialist | Web accessibility and WCAG compliance checker |
| supabase-realtime-optimizer | sonnet | framework-specialist | Supabase real-time optimization specialist |

---

### Database & Data (12 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| database-architect | sonnet | database-specialist | Database architecture and optimization expert |
| database-optimization | haiku | database-specialist | Database query and performance optimization |
| database-optimizer | haiku | database-specialist | Database tuning and optimization specialist |
| nosql-specialist | sonnet | database-specialist | NoSQL database design and implementation |
| supabase-schema-architect | sonnet | database-specialist | Supabase PostgreSQL schema design |
| database-admin | haiku | database-specialist | Database administration and maintenance |
| data-engineer | haiku | database-specialist | Data pipeline and ETL specialist |
| data-scientist | sonnet | database-specialist | Data science and statistical analysis expert |
| data-analyst | haiku | database-specialist | Data analytics and business metrics specialist |
| ml-engineer | sonnet | database-specialist | Machine learning and model implementation |
| ai-engineer | sonnet | database-specialist | AI application and LLM integration specialist |
| mlops-engineer | sonnet | database-specialist | MLOps and machine learning deployment |

---

### Backend & API (15 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| backend-architect | sonnet | architecture-specialist | Backend system architecture and API design |
| backend-specialist | haiku | architecture-specialist | Backend implementation and service development |
| api-explorer | sonnet | integration-specialist | Third-party API exploration and integration |
| api-documenter | haiku | documentation-specialist | OpenAPI/Swagger spec and SDK generation |
| api-designer | haiku | architecture-specialist | REST API and interface design specialist |
| api-security-audit | sonnet | security-specialist | API security and vulnerability assessment |
| rest-api-specialist | haiku | architecture-specialist | REST API design and implementation |
| graphql-designer | haiku | framework-specialist | GraphQL schema design and implementation |
| grpc-specialist | haiku | architecture-specialist | gRPC service design and implementation |
| websocket-specialist | haiku | architecture-specialist | WebSocket and real-time communication specialist |
| microservices-architect | haiku | architecture-specialist | Microservices architecture design |
| event-driven-architect | haiku | architecture-specialist | Event-driven architecture specialist |
| queue-specialist | haiku | architecture-specialist | Message queue and async processing specialist |
| cache-specialist | haiku | architecture-specialist | Caching strategy and implementation specialist |
| connection-agent | haiku | utility | Connection and credential management utility |

---

### Security (8 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| security-auditor | sonnet | security-specialist | Application security and vulnerability expert |
| security-engineer | sonnet | security-specialist | Security engineering and encryption specialist |
| api-security-audit | sonnet | security-specialist | API security and vulnerability assessment |
| penetration-tester | sonnet | security-specialist | Penetration testing and security research |
| compliance-specialist | sonnet | security-specialist | Compliance and regulatory requirement specialist |
| mcp-security-auditor | sonnet | security-specialist | MCP protocol security auditor |
| web-accessibility-checker | haiku | language-specialist | Web accessibility and WCAG compliance |

---

### Testing & Quality (8 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| test-engineer | haiku | test-engineer | Automated testing and QA specialist |
| test-automator | haiku | test-engineer | Test automation and CI/CD integration |
| mcp-testing-engineer | haiku | test-engineer | MCP protocol testing and validation |
| code-reviewer | sonnet | code-reviewer | Code quality and best practices reviewer |
| architect-review | sonnet | code-reviewer | Architecture consistency and SOLID review |
| debugger | sonnet | code-reviewer | Debug issue identification and troubleshooting |
| error-detective | sonnet | code-reviewer | Error analysis and root cause investigation |
| flutter-go-reviewer | sonnet | code-reviewer | Flutter/Go code review specialist |

---

### DevOps & Infrastructure (10 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| devops-engineer | haiku | devops-engineer | DevOps and container orchestration specialist |
| deployment-engineer | sonnet | devops-engineer | Deployment automation and release management |
| cloud-architect | sonnet | devops-engineer | Cloud infrastructure and architecture expert |
| terraform-specialist | haiku | devops-engineer | Terraform and infrastructure-as-code specialist |
| kubernetes-specialist | sonnet | devops-engineer | Kubernetes and container orchestration expert |
| network-engineer | sonnet | devops-engineer | Network architecture and configuration specialist |
| monitoring-specialist | haiku | devops-engineer | Monitoring, logging, and observability specialist |
| performance-engineer | sonnet | performance-specialist | Performance analysis and optimization expert |
| performance-profiler | sonnet | performance-specialist | Performance profiling and benchmarking |
| sre-specialist | haiku | devops-engineer | Site reliability engineering specialist |

---

### Documentation (7 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| technical-writer | haiku | documentation-specialist | Technical documentation and writing expert |
| documentation-expert | haiku | documentation-specialist | Comprehensive documentation specialist |
| api-documenter | haiku | documentation-specialist | API documentation and SDK generation |
| changelog-generator | haiku | documentation-specialist | Changelog and release notes generator |
| markdown-formatter | haiku | documentation-specialist | Markdown formatting and structure specialist |
| report-generator | haiku | documentation-specialist | Report and document generation specialist |
| content-marketer | haiku | documentation-specialist | Content marketing and copywriting specialist |

---

### Research & Analysis (12 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| technical-researcher | sonnet | research-specialist | Technical research and analysis expert |
| academic-researcher | haiku | research-specialist | Academic research and scholarly sources |
| research-orchestrator | sonnet | research-specialist | Research coordination and synthesis |
| comprehensive-researcher | haiku | research-specialist | Comprehensive research analysis |
| research-brief-generator | haiku | research-specialist | Research brief and summary generation |
| research-coordinator | haiku | research-specialist | Research project coordination |
| research-synthesizer | haiku | research-specialist | Research synthesis and conclusions |
| business-analyst | haiku | research-specialist | Business metrics and analysis specialist |
| query-clarifier | haiku | research-specialist | Query interpretation and clarification |
| search-specialist | haiku | research-specialist | Search strategy and information retrieval |
| fact-checker | haiku | research-specialist | Fact verification and validation specialist |
| model-evaluator | haiku | research-specialist | Model evaluation and benchmarking |

---

### Integration Specialists (6 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| api-explorer | sonnet | integration-specialist | Third-party API exploration and integration |
| salesforce-api-specialist | sonnet | integration-specialist | Salesforce API and CRM integration expert |
| authentik-api-specialist | sonnet | integration-specialist | Authentik OAuth2/OIDC authentication expert |
| supabase-realtime-optimizer | sonnet | integration-specialist | Supabase real-time optimization |
| mcp-integration-engineer | haiku | integration-specialist | MCP protocol integration specialist |
| mcp-expert | sonnet | integration-specialist | MCP protocol and server implementation expert |

---

### Development Utilities (10 agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| tdd-coding-agent | haiku | utility | Test-driven development methodology expert |
| git-flow-manager | haiku | utility | Git workflow and version control specialist |
| dependency-manager | haiku | utility | Dependency management and updates specialist |
| dx-optimizer | haiku | utility | Developer experience optimization specialist |
| credential-manager | haiku | utility | Credential and secrets management specialist |
| command-expert | haiku | utility | Command-line interface and scripting expert |
| shell-scripting-pro | haiku | utility | Shell scripting and system automation |
| sql-pro | haiku | utility | SQL and database query expert |
| prompt-engineer | haiku | utility | Prompt engineering and optimization specialist |
| llms-maintainer | haiku | utility | LLM model maintenance and updates |

---

### Advanced Specialists (15+ agents)

| Agent Name | Model | Type | Role |
|------------|-------|------|------|
| fullstack-developer | sonnet | fullstack-developer | Full-stack application development expert |
| mobile-developer | haiku | language-specialist | Mobile application development specialist |
| ios-developer | haiku | language-specialist | iOS development specialist |
| android-specialist | haiku | language-specialist | Android development specialist |
| design-system-architect | haiku | architecture-specialist | Design system architecture and implementation |
| animation-specialist | haiku | language-specialist | Animation and interactive effects specialist |
| responsive-design-expert | haiku | language-specialist | Responsive design and mobile-first specialist |
| ssr-specialist | haiku | framework-specialist | Server-side rendering specialist |
| static-site-generator | haiku | framework-specialist | Static site generation specialist |
| jamstack-architect | haiku | architecture-specialist | JAMstack architecture specialist |
| cdn-optimizer | haiku | architecture-specialist | CDN and edge optimization specialist |
| image-optimization | haiku | architecture-specialist | Image optimization specialist |
| video-streaming | haiku | architecture-specialist | Video streaming specialist |
| file-storage-specialist | haiku | architecture-specialist | File storage and object storage specialist |
| search-engineer | haiku | architecture-specialist | Search engine design and optimization |
| realtime-engineer | haiku | architecture-specialist | Real-time communication specialist |
| backup-recovery | haiku | devops-engineer | Backup and disaster recovery specialist |
| disaster-recovery | haiku | devops-engineer | Disaster recovery and high availability |
| high-availability | haiku | devops-engineer | High availability and redundancy specialist |
| load-balancing | haiku | devops-engineer | Load balancing and traffic distribution |
| auto-scaling | haiku | devops-engineer | Auto-scaling and capacity management |
| container-orchestration | haiku | devops-engineer | Container orchestration and management |
| service-mesh | haiku | devops-engineer | Service mesh and microservices patterns |
| observability-engineer | haiku | devops-engineer | Observability and monitoring architect |
| logging-specialist | haiku | devops-engineer | Logging and log aggregation specialist |
| metrics-collector | haiku | devops-engineer | Metrics collection and analysis specialist |
| tracing-specialist | haiku | devops-engineer | Distributed tracing specialist |
| alerting-specialist | haiku | devops-engineer | Alerting and incident notification specialist |
| incident-responder | haiku | devops-engineer | Incident response and management specialist |
| chaos-engineer | haiku | devops-engineer | Chaos engineering and resilience testing |
| reliability-engineer | haiku | devops-engineer | Reliability engineering specialist |
| capacity-planner | haiku | devops-engineer | Capacity planning and forecasting specialist |
| cost-optimizer | haiku | devops-engineer | Cost optimization and finops specialist |
| finops-specialist | haiku | devops-engineer | Financial operations and cost management |
| greenfield-architect | haiku | architecture-specialist | Greenfield project architecture specialist |
| legacy-modernizer | haiku | architecture-specialist | Legacy code modernization specialist |
| migration-specialist | haiku | architecture-specialist | Migration and refactoring specialist |
| product-strategist | sonnet | research-specialist | Product strategy and roadmap specialist |
| project-supervisor-orchestrator | haiku | utility | Project supervision and orchestration |
| quant-analyst | haiku | research-specialist | Quantitative analysis specialist |
| unused-code-cleaner | haiku | utility | Code cleanup and refactoring specialist |
| url-link-extractor | haiku | utility | URL and link extraction specialist |
| cloud-migration-specialist | sonnet | architecture-specialist | Cloud migration and strategy expert |
| load-testing-specialist | haiku | test-engineer | Load testing and performance testing specialist |
| risk-manager | haiku | utility | Risk management and assessment specialist |
| document-structure-analyzer | haiku | utility | Document structure analysis specialist |
| metadata-agent | haiku | utility | Metadata management and organization |
| tag-agent | haiku | utility | Tagging and categorization utility |
| context-manager | sonnet | utility | Context preservation and management specialist |
| mcp-protocol-specialist | haiku | integration-specialist | MCP protocol specification specialist |
| mcp-server-architect | haiku | architecture-specialist | MCP server architecture and design |
| mcp-deployment-orchestrator | haiku | devops-engineer | MCP server deployment and orchestration |
| cli-ui-designer | haiku | language-specialist | CLI and terminal UI design specialist |
| architecture-modernizer | sonnet | architecture-specialist | Architecture modernization and evolution |
| review-agent | haiku | utility | General review and quality assurance |
| task-decomposition-expert | haiku | utility | Task decomposition and planning specialist |
| database-admin | haiku | database-specialist | Database administration and operations |
| agent-overview | haiku | utility | Agent system documentation utility |

---

## Summary Statistics

### By Type Category

```
architect: 1 agent
code-reviewer: 5 agents
database-specialist: 12 agents
devops-engineer: 10 agents
documentation-specialist: 7 agents
framework-specialist: 12 agents
fullstack-developer: 1 agent
integration-specialist: 6 agents
language-specialist: 15 agents
performance-specialist: 2 agents
research-specialist: 12 agents
security-specialist: 7 agents
test-engineer: 5 agents
utility: 16 agents
architecture-specialist: 10 agents
---
TOTAL: 117 agents
```

### By Model Distribution

```
Opus (1):
  - chief-architect

Sonnet (35):
  - architecture-modernizer
  - authentik-api-specialist
  - backend-architect
  - code-reviewer
  - compliance-specialist
  - comprehensive-researcher
  - context-manager
  - data-scientist
  - database-architect
  - deployment-engineer
  - api-security-audit
  - api-explorer
  - architect-review
  - debugger
  - error-detective
  - flutter-go-reviewer
  - fullstack-developer
  - graphql-specialist
  - graphql-performance-optimizer
  - graphql-security-specialist
  - kubernetes-specialist
  - ml-engineer
  - mlops-engineer
  - mcp-expert
  - network-engineer
  - nosql-specialist
  - penetration-tester
  - performance-engineer
  - performance-profiler
  - react-performance
  - research-orchestrator
  - salesforce-api-specialist
  - security-auditor
  - security-engineer
  - technical-researcher
  - terraform-specialist
  - ai-engineer
  - cloud-architect

Haiku (81):
  - academic-researcher
  - agent-overview
  - api-documenter
  - api-designer
  - android-specialist
  - animation-specialist
  - backend-specialist
  - business-analyst
  - cache-specialist
  - changelog-generator
  - cli-ui-designer
  - command-expert
  - connection-agent
  - content-marketer
  - credential-manager
  - data-analyst
  - data-engineer
  - database-admin
  - database-optimization
  - database-optimizer
  - dependency-manager
  - devops-engineer
  - devops-troubleshooter
  - document-structure-analyzer
  - documentation-expert
  - dx-optimizer
  - design-system-architect
  - fact-checker
  - flutter-specialist
  - frontend-developer
  - git-flow-manager
  - go-specialist
  - golang-pro
  - grpc-specialist
  - implementation (add rest)
  - (continued below)
```

---

## Implementation Checklist

### Phase 1: Data Preparation
- [ ] Generate type/role mappings for all 117 agents (this document)
- [ ] Create migration scripts for .md files
- [ ] Create migration script for agents.json
- [ ] Review and validate type taxonomy

### Phase 2: Code Updates
- [ ] Update Agent struct in agents_config.rs
- [ ] Update frontmatter parser in agents_config.rs
- [ ] Update FrontmatterData struct
- [ ] Update YAML parsing logic
- [ ] Add type and role fields to serialization

### Phase 3: Data Updates
- [ ] Update all 118 .md files with type and role fields
- [ ] Regenerate agents.json with type and role fields
- [ ] Verify build.rs generates correct embedded agents
- [ ] Rebuild binary

### Phase 4: Testing
- [ ] Run Test #17 - Verify role field populated
- [ ] Run Test #19 - Verify type field for all agents
- [ ] Run full e2e test suite
- [ ] Verify API responses include new fields

### Phase 5: Validation
- [ ] All 117 agents have type field ✓
- [ ] All 117 agents have role field ✓
- [ ] No null values in required fields
- [ ] Test pass rate: 28/28 (100%)
- [ ] Status: "PRODUCTION READY"

---

**Document Status:** READY FOR IMPLEMENTATION
**Next Step:** Begin Phase 1 data preparation and code updates
