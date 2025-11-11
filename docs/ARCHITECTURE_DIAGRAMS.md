# Claude Orchestra Architecture Diagrams

This document provides comprehensive visual representations of the Claude Orchestra system architecture, agent coordination, and deployment infrastructure.

## Table of Contents

1. [High-Level System Architecture](#1-high-level-system-architecture) - 117 agents organized by model tier
2. [Agent Coordination Flow](#2-agent-coordination-flow) - Cross-agent communication via Knowledge Manager
3. [Knowledge Manager Architecture](#3-knowledge-manager-architecture) - Persistent memory system
4. [Model Assignment Strategy](#4-model-assignment-strategy) - Opus/Sonnet/Haiku distribution
5. [Autonomous Operation Workflow](#5-autonomous-operation-workflow) - Self-managing workflows
6. [Cross-Repository Deployment](#6-cross-repository-deployment) - Multi-project orchestration
7. [Decision Authority Matrix](#7-decision-authority-matrix) - Risk-based autonomy levels

**Note**: ccproxy model routing (local LLM integration) is a future enhancement pending hardware availability.

---

## 1. High-Level System Architecture

This diagram shows the overall architecture of the Claude Orchestra system with **117 specialized agents** organized by model tier and the Knowledge Manager coordination system.

```mermaid
graph TD
    User[User Request] --> CC[Claude Code]
    CC --> Arch[Chief Architect<br/>Opus 4.1<br/>Strategic Leadership]

    Arch --> Intelligent[Intelligent Managers<br/>77 Agents - Sonnet 4.5]
    Arch --> Basic[Basic Specialists<br/>39 Agents - Haiku 4.5]

    Intelligent --> Reviewers[Code Review & Quality<br/>Reviewers, Debuggers]
    Intelligent --> Security[Security & Compliance<br/>Auditors, Pen Testers]
    Intelligent --> Testing[Testing & QA<br/>TDD, Test Engineers]
    Intelligent --> Architecture[Architecture & Design<br/>Backend, Frontend, Data]
    Intelligent --> Integration[API Integration<br/>Salesforce, Authentik]

    Basic --> Coders[Language Specialists<br/>Python, Swift, Go, Rust, etc.]
    Basic --> Docs[Documentation<br/>Writers, API Docs]
    Basic --> Utils[Utilities<br/>DevOps Tools, Monitoring]
    Basic --> Research[Basic Research<br/>Search, Fact-checking]

    Arch -.-> KM[Knowledge Manager<br/>LanceDB Vector DB<br/>Persistent Memory]
    Intelligent -.-> KM
    Basic -.-> KM

    style Arch fill:#ff9900,stroke:#cc7700,color:#000
    style Intelligent fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Basic fill:#66d9ff,stroke:#3399cc,color:#000
    style KM fill:#ffcc66,stroke:#cc9933,color:#000
```

**Key Components:**
- **Chief Architect** (1 agent): Strategic leadership using Opus 4.1 - architecture design, coordination
- **Intelligent Managers** (77 agents): Complex reasoning using Sonnet 4.5 - code review, security, testing, architecture, API integration
- **Basic Specialists** (39 agents): Simple tasks using Haiku 4.5 - language coding, documentation, utilities, basic research
- **Knowledge Manager**: Persistent memory with LanceDB vector search for cross-agent coordination
- **All agents**: Direct Claude API (ccproxy integration is future enhancement pending hardware)

---

## 2. Agent Coordination Flow

This sequence diagram shows how agents coordinate through the Knowledge Manager from user request to final delivery.

```mermaid
sequenceDiagram
    participant User
    participant CC as Claude Code
    participant Arch as Chief Architect
    participant KM as Knowledge Manager
    participant TDD as TDD Agent
    participant Code as Coding Agents
    participant Creds as Credential Manager
    participant QA as QA Engineer
    participant Sec as Security Auditor
    participant Docs as Docs Lead

    User->>CC: Describe requirement
    CC->>Arch: Analyze & design system
    Arch->>KM: Store architecture decisions
    Note over Arch,KM: Key: architect/decisions

    par Phase 1: Implementation
        Arch->>TDD: Write failing tests first
        TDD->>KM: Search architect/decisions
        TDD->>TDD: Create comprehensive test suite
        TDD->>KM: Store tdd/failing-tests/*

        TDD->>Code: Tests ready
        Code->>KM: Search tdd/failing-tests/*
        Code->>Code: Implement features
        Code->>Code: Run tests until green
        Code->>KM: Store coder/implementation/*

        Arch->>Creds: Manage secrets
        Creds->>KM: Search all agents' needs
        Creds->>Creds: Encrypt & store credentials
        Creds->>KM: Store credentials/inventory
    end

    Note over Code,Creds: Model swap: qwen2.5-coder → qwen-quality-128k

    par Phase 2: Quality Assurance
        Code->>QA: Implementation complete
        QA->>KM: Search tdd/failing-tests/*
        QA->>KM: Search coder/implementation/*
        QA->>QA: Add edge cases & integration tests
        QA->>KM: Store qa/review/*

        Code->>Sec: Security review needed
        Sec->>KM: Search coder/implementation/*
        Sec->>Sec: Vulnerability scan
        Sec->>Sec: OWASP compliance check
        Sec->>KM: Store security/findings/*

        Code->>Docs: Documentation needed
        Docs->>KM: Search architect/decisions
        Docs->>KM: Search coder/implementation/*
        Docs->>Docs: Generate technical docs
        Docs->>KM: Store docs/*
    end

    Arch->>KM: Review all agent outputs
    Arch->>Arch: Final quality check
    Arch->>CC: Synthesis complete
    CC->>User: Deliver solution

    Note over User,Docs: 15 agents, 2 phases, ~70 minutes
```

**Coordination Highlights:**
- All agents query Knowledge Manager before starting work
- Phase 1 runs concurrently (11 agents with 2 models loaded)
- Model swap happens between Phase 1 and Phase 2
- Phase 2 runs concurrently (3 agents with 1 model)
- Architect oversees entire lifecycle

---

## 3. Knowledge Manager Architecture

This diagram illustrates the Knowledge Manager's structure with LanceDB vector database, per-repository isolation, and compaction resilience.

```mermaid
graph TB
    subgraph "Knowledge Manager System"
        KM[Knowledge Manager CLI<br/>knowledge-manager.js]

        subgraph "Storage Layer"
            Lance[LanceDB Vector DB<br/>384-dim embeddings]
            Files[Flat File Store<br/>JSON entries]
        end

        subgraph "Repository Isolation"
            Repo1[Project A<br/>data/knowledge/project-a/]
            Repo2[Project B<br/>data/knowledge/project-b/]
            Repo3[cc-orchestra<br/>data/knowledge/cc-orchestra/]
        end

        subgraph "Knowledge Types"
            Arch[architecture]
            Dec[decision]
            Impl[implementation]
            Config[configuration]
            Cred[credential]
            Issue[issue]
            Pattern[pattern]
            Gen[general]
        end

        subgraph "Agent Integration"
            Pre[Pre-operation:<br/>search & retrieve]
            During[During operation:<br/>store progress]
            Post[Post-operation:<br/>store completion]
        end

        subgraph "Compaction Hooks"
            PreComp[preCompactionHook:<br/>Export critical context]
            PostComp[postCompactionHook:<br/>Restore context]
        end
    end

    KM --> Lance
    KM --> Files
    Lance --> Repo1
    Lance --> Repo2
    Lance --> Repo3

    KM --> Arch
    KM --> Dec
    KM --> Impl
    KM --> Config
    KM --> Cred
    KM --> Issue
    KM --> Pattern
    KM --> Gen

    Agents[All 15 Agents] --> Pre
    Pre --> KM
    Agents --> During
    During --> KM
    Agents --> Post
    Post --> KM

    Compaction[Conversation<br/>Compaction] --> PreComp
    PreComp --> KM
    Compaction --> PostComp
    PostComp --> KM

    style KM fill:#ffcc66,stroke:#cc9933,color:#000
    style Lance fill:#99ccff,stroke:#6699cc,color:#000
    style Files fill:#99ccff,stroke:#6699cc,color:#000
    style Agents fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Compaction fill:#ff6666,stroke:#cc3333,color:#000
```

**Knowledge Manager Features:**
- **Vector Search**: 384-dimensional embeddings for semantic retrieval
- **Per-Repository Context**: Isolated knowledge bases per project
- **Knowledge Types**: 8 categories for structured storage
- **Agent Protocol**: Before/during/after hooks for all agents
- **Compaction Resilience**: Zero data loss across conversation compactions
- **LanceDB Backend**: High-performance vector database
- **Flat File Backup**: JSON entries for portability

---

## 4. Model Assignment Strategy

This diagram shows how the 117 agents are distributed across three Claude models based on role complexity.

```mermaid
graph TD
    subgraph "Claude Orchestra - 117 Agents"
        CC[Claude Code<br/>Spawns all agents in parallel]
    end

    CC --> OpusAgent[Chief Architect<br/>1 Agent<br/>Opus 4.1]
    CC --> SonnetAgents[Intelligent Managers<br/>77 Agents<br/>Sonnet 4.5]
    CC --> HaikuAgents[Basic Specialists<br/>39 Agents<br/>Haiku 4.5]

    OpusAgent --> OpusAPI[Claude Opus 4.1 API<br/>Strategic Leadership<br/>Architecture Design]

    SonnetAgents --> SonnetAPI[Claude Sonnet 4.5 API<br/>Complex Reasoning<br/>Code Review, Security, Testing]

    HaikuAgents --> HaikuAPI[Claude Haiku 4.5 API<br/>Simple Tasks<br/>Language Coding, Docs, Utils]

    SonnetAgents --> ReviewGroup[Code Review & Quality<br/>Reviewers, Debuggers]
    SonnetAgents --> SecGroup[Security & Compliance<br/>Auditors, Pen Testers]
    SonnetAgents --> TestGroup[Testing & QA<br/>TDD, Test Engineers]
    SonnetAgents --> ArchGroup[Architecture & Design<br/>Backend, Frontend, Data]
    SonnetAgents --> IntegGroup[API Integration<br/>Salesforce, Authentik]

    HaikuAgents --> CoderGroup[Language Specialists<br/>Python, Swift, Go, Rust]
    HaikuAgents --> DocGroup[Documentation<br/>Writers, API Docs]
    HaikuAgents --> UtilGroup[Utilities<br/>DevOps Tools, Monitoring]
    HaikuAgents --> ResGroup[Basic Research<br/>Search, Fact-checking]

    style OpusAgent fill:#ff9900,stroke:#cc7700,color:#000
    style SonnetAgents fill:#4a9eff,stroke:#2e6cbb,color:#000
    style HaikuAgents fill:#66d9ff,stroke:#3399cc,color:#000
    style OpusAPI fill:#ffcc99,stroke:#cc9966,color:#000
    style SonnetAPI fill:#99ccff,stroke:#6699cc,color:#000
    style HaikuAPI fill:#99ffff,stroke:#66cccc,color:#000
```

**Model Assignment Criteria:**

| Model | Count | Selection Criteria | Examples |
|-------|-------|-------------------|----------|
| **Opus 4.1** | 1 | Strategic leadership, architecture design, coordination | Chief Architect |
| **Sonnet 4.5** | 77 | Complex reasoning, code review, security analysis, testing strategy, architecture decisions | Code Reviewer, Security Auditor, TDD Agent, Backend Architect, DevOps Engineer |
| **Haiku 4.5** | 39 | Simple coding, documentation, utilities, basic research | Python Specialist, Technical Writer, Git Flow Manager, Search Specialist |

**Cost Optimization:**
- **Current**: All agents use direct Claude API
- **Haiku 4.5**: 33% of agents use most cost-effective model
- **Future**: ccproxy integration for local LLM routing (pending hardware)
  - Potential savings: $300-450/month
  - Target: Mac mini with Ollama for Sonnet/Haiku workloads

---

## 5. Autonomous Operation Workflow

This diagram shows the 8-phase autonomous workflow with checkpoints, heartbeats, error recovery, and compaction management.

```mermaid
graph TD
    Start[User Request] --> Discover[Phase 1: Requirements Discovery<br/>60-80 adaptive questions<br/>Definition of Done]

    Discover --> Spec[Phase 2: Specification Generation<br/>Complete project spec<br/>Store in Knowledge Manager]

    Spec --> Design[Phase 3: Architecture Design<br/>Chief Architect leads<br/>Store decisions in KM]

    Design --> Checkpoint1{Checkpoint 1<br/>30 min}
    Checkpoint1 -->|Continue| Impl[Phase 4: Implementation<br/>Phase 1 agents<br/>TDD-first approach]
    Checkpoint1 -->|User Review| UserReview1[High-risk decision approval]
    UserReview1 --> Impl

    Impl --> Test[Phase 5: Testing<br/>QA Engineer<br/>Edge cases & integration]

    Test --> Checkpoint2{Checkpoint 2<br/>60 min}
    Checkpoint2 -->|Continue| Audit[Phase 6: Security Audit<br/>Security Auditor<br/>OWASP compliance]
    Checkpoint2 -->|User Review| UserReview2[Security findings review]
    UserReview2 --> Audit

    Audit --> Doc[Phase 7: Documentation<br/>Documentation Lead<br/>Technical docs & API reference]

    Doc --> Deploy[Phase 8: Deployment<br/>DevOps Engineer<br/>CI/CD & infrastructure]

    Deploy --> Complete[Delivery]

    subgraph "Autonomous Features"
        Heartbeat[Heartbeat Monitor<br/>Every 10 minutes<br/>Agent health tracking]

        ErrorRecovery[Error Recovery<br/>90%+ autonomous<br/>3 retry attempts]

        ModelFallback[Model Fallback<br/>Opus → Sonnet 4.5<br/>Automatic at 80% tokens]

        CompactionMgmt[Compaction Management<br/>Pre-export critical context<br/>Post-restore from KM]
    end

    Impl -.-> Heartbeat
    Test -.-> Heartbeat
    Audit -.-> Heartbeat
    Doc -.-> Heartbeat
    Deploy -.-> Heartbeat

    Impl -.-> ErrorRecovery
    Test -.-> ErrorRecovery
    Audit -.-> ErrorRecovery

    Design -.-> ModelFallback
    Impl -.-> ModelFallback

    Checkpoint1 -.-> CompactionMgmt
    Checkpoint2 -.-> CompactionMgmt

    style Start fill:#66cc99,stroke:#339966,color:#000
    style Complete fill:#66cc99,stroke:#339966,color:#000
    style Checkpoint1 fill:#ffcc66,stroke:#cc9933,color:#000
    style Checkpoint2 fill:#ffcc66,stroke:#cc9933,color:#000
    style UserReview1 fill:#ff9966,stroke:#cc6633,color:#000
    style UserReview2 fill:#ff9966,stroke:#cc6633,color:#000
    style Heartbeat fill:#99ccff,stroke:#6699cc,color:#000
    style ErrorRecovery fill:#ff6699,stroke:#cc3366,color:#000
    style ModelFallback fill:#cc99ff,stroke:#9966cc,color:#000
    style CompactionMgmt fill:#ffcc99,stroke:#cc9966,color:#000
```

**Autonomous Operation Capabilities:**
- **Target Duration**: 4-8 hours without user intervention
- **Checkpoints**: Every 30-60 minutes for progress tracking
- **Heartbeat**: Every 10 minutes for agent health monitoring
- **Error Recovery**: 90%+ errors handled autonomously (3 retry attempts)
- **Model Fallback**: Automatic Opus → Sonnet 4.5 at 80% token usage
- **Compaction Resilience**: Zero data loss via Knowledge Manager hooks
- **Decision Authority**: Clear matrix for autonomous vs. user approval

---

## 6. Cross-Repository Deployment

This diagram illustrates how the orchestra deploys from a global configuration to work in any project directory with auto-detection.

```mermaid
graph TB
    subgraph "Global Configuration"
        GlobalCLAUDE[~/.claude/CLAUDE.md<br/>Global instructions<br/>117-agent roster<br/>Trigger patterns]

        OrchestraRepo[/Users/brent/git/cc-orchestra/<br/>Orchestra configuration<br/>Knowledge Manager<br/>Agent definitions]

        Config[config/orchestra-config.json<br/>Agent roles & models<br/>ccproxy mappings<br/>Autonomous settings]
    end

    subgraph "Project Directories"
        ProjA[~/git/project-a/<br/>Python API]
        ProjB[~/git/project-b/<br/>Flutter + Go]
        ProjC[~/git/project-c/<br/>Enterprise integration]
    end

    subgraph "Project Customization"
        CustomA[project-a/CLAUDE.md<br/>Tech stack: Python<br/>Agents: Python, QA, Security]

        CustomB[project-b/CLAUDE.md<br/>Tech stack: Flutter + Go<br/>Agents: Flutter, Go, DevOps]

        CustomC[project-c/CLAUDE.md<br/>Tech stack: Salesforce + Authentik<br/>Agents: Selected from 117]
    end

    subgraph "Auto-Detection Logic"
        Trigger[Trigger Pattern Detection<br/>- Full-stack apps<br/>- Multi-technology<br/>- Enterprise integration<br/>- DevOps tasks]

        Bypass[Bypass Patterns<br/>- Single file changes<br/>- Simple queries<br/>- Basic operations]
    end

    User[User Request] --> CC[Claude Code]
    CC --> Trigger
    CC --> Bypass

    Trigger -->|Complex task| GlobalCLAUDE
    GlobalCLAUDE --> OrchestraRepo
    OrchestraRepo --> Config

    Config -->|Deploy agents to| ProjA
    Config -->|Deploy agents to| ProjB
    Config -->|Deploy agents to| ProjC

    ProjA --> CustomA
    ProjB --> CustomB
    ProjC --> CustomC

    CustomA -.->|Override defaults| Config
    CustomB -.->|Override defaults| Config
    CustomC -.->|Override defaults| Config

    KM[Knowledge Manager<br/>Per-repo context<br/>data/knowledge/project-x/]

    ProjA --> KM
    ProjB --> KM
    ProjC --> KM

    Bypass -->|Simple task| Direct[Direct execution<br/>No orchestra deployment]

    style GlobalCLAUDE fill:#ff9900,stroke:#cc7700,color:#000
    style OrchestraRepo fill:#ffcc66,stroke:#cc9933,color:#000
    style Config fill:#ffcc99,stroke:#cc9966,color:#000
    style Trigger fill:#66cc99,stroke:#339966,color:#000
    style Bypass fill:#ff6666,stroke:#cc3333,color:#000
    style KM fill:#99ccff,stroke:#6699cc,color:#000
```

**Cross-Repository Features:**
- **Global Config**: Orchestra configuration lives in `/Users/brent/git/cc-orchestra/`
- **Works Anywhere**: Deploys to current working directory
- **Auto-Detection**: Smart trigger patterns activate orchestra automatically
- **Project Customization**: Local `CLAUDE.md` overrides defaults
- **Per-Repo Context**: Isolated Knowledge Manager databases
- **Consistent Quality**: Same standards across all projects

---

## 7. Agent Phase Architecture

This diagram shows the two-phase execution model with memory management and model loading/unloading strategy.

```mermaid
graph TB
    subgraph "Phase 0: Independent"
        Arch[Chief Architect<br/>claude-opus-4-1<br/>→ claude-sonnet-4-5<br/>Direct Claude API]
    end

    subgraph "Phase 1: Implementation (11 agents, ~25GB)"
        subgraph "qwen2.5-coder:32b-instruct (~20GB)"
            TDD[1. TDD Coding Agent<br/>Test-first specialist]
            Python[2. Python Expert<br/>FastAPI, ML/AI]
            Swift[3. Swift Expert<br/>iOS, SwiftUI]
            Go[4. Go Expert<br/>Microservices]
            Rust[5. Rust Expert<br/>Systems programming]
            Flutter[6. Flutter Expert<br/>Cross-platform mobile]
            API[7. API Explorer<br/>Third-party APIs]
            SF[8. Salesforce API<br/>CRM integration]
            Auth[9. Authentik API<br/>OAuth2/OIDC]
            DevOps[10. DevOps Engineer<br/>Docker, K8s, AWS]
        end

        subgraph "qwen-fast:latest (~5GB)"
            Creds[11. Credential Manager<br/>Secrets management]
        end

        Note1[Both models loaded<br/>simultaneously in memory]
    end

    ModelSwap[Model Swap Event<br/>qwen2.5-coder unloads<br/>qwen-quality-128k loads<br/>~40 seconds]

    subgraph "Phase 2: Quality & Documentation (3 agents, ~35GB)"
        subgraph "qwen-quality-128k:latest (~35GB)"
            QA[13. QA Engineer<br/>Integration & E2E tests<br/>Autonomous fixing]
            Sec[14. Security Auditor<br/>Vulnerability scanning<br/>OWASP compliance]
            Docs[15. Documentation Lead<br/>Technical docs<br/>API reference]
        end

        Note2[Single model<br/>Deep reasoning<br/>128k context]
    end

    User[User Requirement] --> Arch
    Arch -->|Store decisions| KM[Knowledge Manager]

    Arch --> TDD
    TDD --> Python
    TDD --> Swift
    TDD --> Go
    TDD --> Rust
    TDD --> Flutter
    TDD --> SF
    TDD --> Auth

    Arch --> API
    Arch --> DevOps
    Arch --> Creds

    Python -.->|Coordinate via| KM
    Swift -.->|Coordinate via| KM
    Go -.->|Coordinate via| KM
    Rust -.->|Coordinate via| KM
    Flutter -.->|Coordinate via| KM
    API -.->|Coordinate via| KM
    SF -.->|Coordinate via| KM
    Auth -.->|Coordinate via| KM
    DevOps -.->|Coordinate via| KM
    Creds -.->|Coordinate via| KM

    Python --> ModelSwap
    Swift --> ModelSwap
    Go --> ModelSwap
    Rust --> ModelSwap
    Flutter --> ModelSwap
    API --> ModelSwap
    SF --> ModelSwap
    Auth --> ModelSwap
    DevOps --> ModelSwap
    Creds --> ModelSwap

    ModelSwap --> QA
    ModelSwap --> Sec
    ModelSwap --> Docs

    QA -.->|Coordinate via| KM
    Sec -.->|Coordinate via| KM
    Docs -.->|Coordinate via| KM

    QA --> Complete[Delivery]
    Sec --> Complete
    Docs --> Complete

    style Arch fill:#ff9900,stroke:#cc7700,color:#000
    style TDD fill:#66ccff,stroke:#3399cc,color:#000
    style Python fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Swift fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Go fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Rust fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Flutter fill:#4a9eff,stroke:#2e6cbb,color:#000
    style API fill:#4a9eff,stroke:#2e6cbb,color:#000
    style SF fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Auth fill:#4a9eff,stroke:#2e6cbb,color:#000
    style DevOps fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Creds fill:#66d9ff,stroke:#3399cc,color:#000
    style ModelSwap fill:#ff6666,stroke:#cc3333,color:#000
    style QA fill:#9966ff,stroke:#7744cc,color:#000
    style Sec fill:#9966ff,stroke:#7744cc,color:#000
    style Docs fill:#9966ff,stroke:#7744cc,color:#000
    style KM fill:#ffcc66,stroke:#cc9933,color:#000
    style Complete fill:#66cc99,stroke:#339966,color:#000
```

**Phase Execution Details:**
- **Phase 0**: Chief Architect uses Claude API independently
- **Phase 1**: 11 agents run concurrently (10 coding + 1 credentials)
  - 2 models loaded simultaneously: qwen2.5-coder (20GB) + qwen-fast (5GB) = 25GB
  - Duration: ~30 minutes
- **Model Swap**: qwen2.5-coder unloads, qwen-quality-128k loads (~40s)
- **Phase 2**: 3 reasoning agents run concurrently
  - 1 model: qwen-quality-128k (35GB)
  - Duration: ~30 minutes
- **Total Time**: ~70 minutes for full pipeline

---

## 8. Decision Authority Matrix

This flowchart shows the decision-making process with clear authority levels and documentation requirements.

```mermaid
graph TD
    Decision[Agent Makes Decision] --> Assess[Assess Risk Level]

    Assess --> Low{Low Risk?}
    Assess --> Medium{Medium Risk?}
    Assess --> High{High Risk?}

    Low -->|Yes| LowExamples[Examples:<br/>- Code formatting<br/>- Minor version updates<br/>- Test strategies<br/>- File organization]

    LowExamples --> LowExecute[Execute Immediately<br/>No approval needed<br/>No documentation]

    Medium -->|Yes| MediumExamples[Examples:<br/>- Technology choices within stack<br/>- API design<br/>- Database schema<br/>- Security approaches]

    MediumExamples --> ArchitectReview{Architect<br/>Approval?}

    ArchitectReview -->|Approved| MediumExecute[Execute<br/>Document in Knowledge Manager]
    ArchitectReview -->|Rejected| MediumRevise[Revise Approach]
    MediumRevise --> Assess

    High -->|Yes| HighExamples[Examples:<br/>- New external services<br/>- Major architecture changes<br/>- Breaking API changes<br/>- Production deployments]

    HighExamples --> UserCheckpoint[User Checkpoint<br/>Required]

    UserCheckpoint --> UserReview{User<br/>Approval?}

    UserReview -->|Approved| HighExecute[Execute<br/>Full documentation<br/>Store in Knowledge Manager]
    UserReview -->|Rejected| HighRevise[Revise with User Input]
    HighRevise --> Assess

    LowExecute --> Progress[Continue Work]
    MediumExecute --> Progress
    HighExecute --> Progress

    subgraph "Documentation Requirements"
        LowDoc[Low Risk:<br/>No documentation]
        MedDoc[Medium Risk:<br/>Store in Knowledge Manager<br/>Type: decision]
        HighDoc[High Risk:<br/>Detailed documentation<br/>Architecture Decision Record<br/>Rationale & alternatives]
    end

    LowExecute -.-> LowDoc
    MediumExecute -.-> MedDoc
    HighExecute -.-> HighDoc

    subgraph "Autonomous Authority Levels"
        Level1[Level 1: Full Autonomy<br/>All coding agents<br/>Low risk decisions]

        Level2[Level 2: Architect Approval<br/>Integration agents<br/>Medium risk decisions]

        Level3[Level 3: User Approval<br/>Chief Architect<br/>High risk decisions]
    end

    style Low fill:#66cc99,stroke:#339966,color:#000
    style Medium fill:#ffcc66,stroke:#cc9933,color:#000
    style High fill:#ff6666,stroke:#cc3333,color:#000
    style LowExecute fill:#66cc99,stroke:#339966,color:#000
    style MediumExecute fill:#ffcc66,stroke:#cc9933,color:#000
    style HighExecute fill:#ff9966,stroke:#cc6633,color:#000
    style ArchitectReview fill:#ff9900,stroke:#cc7700,color:#000
    style UserCheckpoint fill:#cc3366,stroke:#991144,color:#fff
    style UserReview fill:#cc3366,stroke:#991144,color:#fff
```

**Authority Matrix Summary:**

| Risk Level | Approval Required | Documentation | Examples | Autonomous |
|------------|------------------|---------------|----------|------------|
| **Low** | None | No | Code formatting, minor updates, test strategies | Yes ✅ |
| **Medium** | Architect | Yes (KM) | API design, database schema, tech choices | With approval ⚠️ |
| **High** | User | Yes (Full ADR) | External services, major changes, production deploys | No ❌ |

**Key Principles:**
- **Low Risk**: Agents execute immediately without approval
- **Medium Risk**: Architect reviews and approves before execution
- **High Risk**: User checkpoint required with full documentation
- **Documentation**: Stored in Knowledge Manager for all medium/high risk decisions
- **Escalation**: 3 consecutive errors escalate to next authority level

---

## Summary

These architecture diagrams provide comprehensive visual documentation of the Claude Orchestra system:

1. **High-Level System**: 117 agents, 3 model tiers, Knowledge Manager coordination
2. **Coordination Flow**: TDD-first workflow with parallel execution and checkpoints
3. **Knowledge Manager**: LanceDB vector database with per-repository isolation
4. **ccproxy Routing**: API aliases to local Ollama models with bearer auth
5. **Autonomous Operation**: 8-phase workflow with 4-8 hour target duration
6. **Cross-Repository**: Global config deploys to any project directory
7. **Phase Architecture**: 2-phase execution with memory-aware model loading
8. **Decision Authority**: Clear matrix for autonomous vs. supervised decisions

All diagrams are rendered using Mermaid syntax and can be embedded in any Markdown-compatible documentation system.

---

**Document Version**: 1.0
**Last Updated**: 2025-11-10
**Status**: Complete and current with deployed system
