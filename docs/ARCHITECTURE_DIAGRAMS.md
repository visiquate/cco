# Claude Orchestra Architecture Diagrams

This document provides comprehensive visual representations of the Claude Orchestra system architecture, agent coordination, and deployment infrastructure.

## Table of Contents

1. [High-Level System Architecture](#1-high-level-system-architecture)
2. [Agent Coordination Flow](#2-agent-coordination-flow)
3. [Knowledge Manager Architecture](#3-knowledge-manager-architecture)
4. [ccproxy Model Routing](#4-ccproxy-model-routing)
5. [Autonomous Operation Workflow](#5-autonomous-operation-workflow)
6. [Cross-Repository Deployment](#6-cross-repository-deployment)
7. [Agent Phase Architecture](#7-agent-phase-architecture)
8. [Decision Authority Matrix](#8-decision-authority-matrix)

---

## 1. High-Level System Architecture

This diagram shows the overall architecture of the Claude Orchestra system with 15 specialized agents, the Chief Architect, and the model routing infrastructure through ccproxy.

```mermaid
graph TD
    User[User Request] --> CC[Claude Code]
    CC --> Arch[Chief Architect<br/>Opus 4.1 → Sonnet 4.5<br/>Direct Claude API]

    Arch --> Phase1[Phase 1: Implementation]
    Arch --> Phase2[Phase 2: Quality & Docs]

    Phase1 --> Coding[Coding Agents 1-10<br/>qwen2.5-coder:32b<br/>via claude-3-5-sonnet]
    Phase1 --> Light[Lightweight Agent 11<br/>qwen-fast:latest<br/>via claude-3-haiku]

    Phase2 --> Reasoning[Reasoning Agents 13-15<br/>qwen-quality-128k<br/>via gpt-4]

    Coding --> TDD[1. TDD Coding Agent]
    Coding --> Python[2. Python Expert]
    Coding --> Swift[3. Swift Expert]
    Coding --> Go[4. Go Expert]
    Coding --> Rust[5. Rust Expert]
    Coding --> Flutter[6. Flutter Expert]
    Coding --> API[7. API Explorer]
    Coding --> SF[8. Salesforce API]
    Coding --> Auth[9. Authentik API]
    Coding --> DevOps[10. DevOps Engineer]

    Light --> Creds[11. Credential Manager]

    Reasoning --> QA[13. QA Engineer]
    Reasoning --> Sec[14. Security Auditor]
    Reasoning --> Docs[15. Documentation Lead]

    Coding -.-> KM[Knowledge Manager<br/>LanceDB Vector DB]
    Light -.-> KM
    Reasoning -.-> KM
    Arch -.-> KM

    Coding --> ccproxy[ccproxy @ coder.visiquate.com]
    Light --> ccproxy
    Reasoning --> ccproxy
    ccproxy --> Ollama[Ollama<br/>Mac mini 192.168.9.123]

    style Arch fill:#ff9900,stroke:#cc7700,color:#000
    style Coding fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Light fill:#66d9ff,stroke:#3399cc,color:#000
    style Reasoning fill:#9966ff,stroke:#7744cc,color:#000
    style KM fill:#ffcc66,stroke:#cc9933,color:#000
    style ccproxy fill:#ff6699,stroke:#cc3366,color:#000
    style Ollama fill:#99cc66,stroke:#669933,color:#000
```

**Key Components:**
- **Chief Architect**: Strategic leadership using Claude API (Opus 4.1 with Sonnet 4.5 fallback)
- **Phase 1 Coding Agents**: 10 agents using qwen2.5-coder (32B) for TDD and implementation
- **Phase 1 Lightweight Agent**: 1 agent using qwen-fast (7B) for credential management
- **Phase 2 Reasoning Agents**: 3 agents using qwen-quality-128k (32B) for QA, security, and docs
- **Knowledge Manager**: Persistent memory with LanceDB vector search
- **ccproxy**: LiteLLM proxy routing API calls to local Ollama models

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

## 4. ccproxy Model Routing

This diagram shows how API calls route from Claude Code through ccproxy to Ollama models with bearer token authentication.

```mermaid
graph LR
    subgraph "Claude Code Environment"
        CC[Claude Code]
        Agent1[Agents 1-10<br/>model: sonnet-4.5]
        Agent11[Agent 11<br/>model: haiku]
        Agent13[Agents 13-15<br/>model: gpt-4]
    end

    subgraph "Public Internet"
        CF[Cloudflare Tunnel<br/>coder.visiquate.com]
    end

    subgraph "Mac mini @ 192.168.9.123"
        Traefik[Traefik Reverse Proxy<br/>Port 8080<br/>Bearer Token Auth]

        subgraph "ccproxy (LiteLLM)"
            Proxy[ccproxy<br/>localhost:8081]

            subgraph "API Aliases"
                Alias1[claude-3-5-sonnet]
                Alias2[claude-3-haiku]
                Alias3[gpt-4]
            end

            subgraph "Health Checks"
                HC[Health Checks: DISABLED<br/>Prevents model thrashing]
            end
        end

        subgraph "Ollama @ localhost:11434"
            Model1[qwen2.5-coder:32b<br/>19.8GB<br/>32k context]
            Model2[qwen-fast:latest<br/>4.6GB<br/>32k context]
            Model3[qwen-quality-128k<br/>34.8GB<br/>128k context]
        end

        subgraph "Memory Management"
            Phase1Mem[Phase 1:<br/>Model1 + Model2<br/>= 25GB ✅]
            Phase2Mem[Phase 2:<br/>Model3 only<br/>= 35GB ✅<br/>Model1 auto-unloads]
        end
    end

    Agent1 -->|https://coder.visiquate.com<br/>model: claude-3-5-sonnet| CF
    Agent11 -->|https://coder.visiquate.com<br/>model: claude-3-haiku| CF
    Agent13 -->|https://coder.visiquate.com<br/>model: gpt-4| CF

    CF --> Traefik
    Traefik -->|Bearer Token| Proxy

    Proxy --> Alias1
    Proxy --> Alias2
    Proxy --> Alias3

    Alias1 -.->|routes to| Model1
    Alias2 -.->|routes to| Model2
    Alias3 -.->|routes to| Model3

    Model1 -.-> Phase1Mem
    Model2 -.-> Phase1Mem
    Model3 -.-> Phase2Mem

    Proxy -.-> HC

    style CC fill:#ffcc99,stroke:#cc9966,color:#000
    style Agent1 fill:#4a9eff,stroke:#2e6cbb,color:#000
    style Agent11 fill:#66d9ff,stroke:#3399cc,color:#000
    style Agent13 fill:#9966ff,stroke:#7744cc,color:#000
    style CF fill:#ff9900,stroke:#cc7700,color:#000
    style Traefik fill:#66cc99,stroke:#339966,color:#000
    style Proxy fill:#ff6699,stroke:#cc3366,color:#000
    style Model1 fill:#9966ff,stroke:#7744cc,color:#000
    style Model2 fill:#66d9ff,stroke:#3399cc,color:#000
    style Model3 fill:#cc66ff,stroke:#9933cc,color:#000
    style HC fill:#ff6666,stroke:#cc3333,color:#000
```

**Routing Details:**
- **API Aliases**: ccproxy exposes 3 OpenAI-compatible endpoints
- **Model Mapping**:
  - `claude-3-5-sonnet` → qwen2.5-coder:32b-instruct (Agents 1-10)
  - `claude-3-haiku` → qwen-fast:latest (Agent 11)
  - `gpt-4` → qwen-quality-128k:latest (Agents 13-15)
- **Security**: Bearer token authentication via Traefik
- **Public Access**: Cloudflare tunnel to internal Mac mini
- **Health Checks**: Disabled to prevent model thrashing
- **Memory Strategy**: Phase 1 (25GB) → Phase 2 (35GB)

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
        GlobalCLAUDE[~/.claude/CLAUDE.md<br/>Global instructions<br/>15-agent roster<br/>Trigger patterns]

        ArmyRepo[/Users/brent/git/cc-orchestra/<br/>Army configuration<br/>Knowledge Manager<br/>Agent definitions]

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

        CustomC[project-c/CLAUDE.md<br/>Tech stack: Salesforce + Authentik<br/>Agents: All 15]
    end

    subgraph "Auto-Detection Logic"
        Trigger[Trigger Pattern Detection<br/>- Full-stack apps<br/>- Multi-technology<br/>- Enterprise integration<br/>- DevOps tasks]

        Bypass[Bypass Patterns<br/>- Single file changes<br/>- Simple queries<br/>- Basic operations]
    end

    User[User Request] --> CC[Claude Code]
    CC --> Trigger
    CC --> Bypass

    Trigger -->|Complex task| GlobalCLAUDE
    GlobalCLAUDE --> ArmyRepo
    ArmyRepo --> Config

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

    Bypass -->|Simple task| Direct[Direct execution<br/>No army deployment]

    style GlobalCLAUDE fill:#ff9900,stroke:#cc7700,color:#000
    style ArmyRepo fill:#ffcc66,stroke:#cc9933,color:#000
    style Config fill:#ffcc99,stroke:#cc9966,color:#000
    style Trigger fill:#66cc99,stroke:#339966,color:#000
    style Bypass fill:#ff6666,stroke:#cc3333,color:#000
    style KM fill:#99ccff,stroke:#6699cc,color:#000
```

**Cross-Repository Features:**
- **Global Config**: Army configuration lives in `/Users/brent/git/cc-orchestra/`
- **Works Anywhere**: Deploys to current working directory
- **Auto-Detection**: Smart trigger patterns activate army automatically
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

1. **High-Level System**: 15 agents, 3 model tiers, Knowledge Manager coordination
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
