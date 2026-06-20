# AGENTS.md

Guidance for agentic coding agents working in the Claude Orchestra repository.

## What are CCO agents?

CCO ships 117 compiled-in agent definitions. Each definition is a Markdown file in `src/agents/` with a YAML frontmatter block followed by the agent's system prompt. The build script (`build.rs`) reads every `src/agents/*.md` file at compile time and embeds the definitions into the binary — no runtime file access required.

### Frontmatter schema

```yaml
---
description: "One-line summary shown in `cco agents list`."
mode: subagent          # always "subagent" for spawnable agents
model: haiku            # bare tier: haiku | sonnet | opus
temperature: 0.1        # 0.0–1.0
tools:
  read: true
  write: true
  edit: true
  bash: true
  grep: true
  glob: true
---
```

The `model` field is a bare tier name — never a version-pinned model string such as `claude-3-haiku-20240307`. CCO maps the tier to the current-generation model at runtime, so agent definitions do not need to change when Anthropic releases new model versions.

## Model tiers

| Tier | Count | Intended use |
|------|-------|-------------|
| **opus** | 1 | Chief Architect — strategic architecture decisions and cross-agent coordination |
| **sonnet** | 35 | Managers, code reviewers, security auditors, QA, DevOps, system architects, complex coding tasks |
| **haiku** | 81 | Language specialists, documentation writers, utility tasks, research, data analysis |

**Always specify the `model` parameter when spawning agents with the Task tool.** Omitting it causes the subagent to inherit the parent's model (typically opus), which wastes money on work that haiku or sonnet can handle equally well.

```
# Correct — explicit tier
Task(model="haiku", prompt="Refactor this Python module to use dataclasses...")
Task(model="sonnet", prompt="Review the authentication flow for security issues...")
Task(model="opus", prompt="Design the overall migration strategy for...")

# Wrong — model omitted, inherits parent tier
Task(prompt="Format this JSON file...")
```

## Agent roster

### Opus tier (1 agent)

| Agent | Description |
|-------|-------------|
| chief-architect | Strategic architecture leadership and orchestra coordination |

### Sonnet tier (35 agents)

| Agent | Description |
|-------|-------------|
| api-security-audit | API security audit and vulnerability assessment |
| architect-review | Architecture review and design validation |
| architecture-modernizer | Modernize legacy architectures to current patterns |
| backend-architect | Backend system design and API architecture |
| cloud-architect | Cloud infrastructure design and multi-region architecture |
| cloud-migration-specialist | Cloud migration planning and execution |
| code-reviewer | Expert code review for quality, security, and maintainability |
| compliance-specialist | Regulatory compliance and audit preparation |
| comprehensive-researcher | Deep multi-source research and synthesis |
| connection-agent | Cross-domain connection discovery and insight mapping |
| database-architect | Database schema design and normalization |
| deployment-engineer | Deployment automation and release pipeline design |
| error-detective | Root-cause analysis for complex, multi-system errors |
| flutter-go-reviewer | Flutter and Go cross-platform code review |
| graphql-architect | GraphQL schema design and federation architecture |
| graphql-performance-optimizer | GraphQL query optimization and N+1 elimination |
| graphql-security-specialist | GraphQL security audit and introspection hardening |
| incident-responder | Incident triage, runbook creation, and postmortem facilitation |
| legacy-modernizer | Legacy codebase modernization and technical debt reduction |
| load-testing-specialist | Load-test design, execution, and capacity planning |
| mcp-deployment-orchestrator | MCP server deployment and integration orchestration |
| mcp-expert | MCP protocol expertise and tool design |
| mcp-integration-engineer | MCP client and server integration engineering |
| mcp-protocol-specialist | MCP protocol implementation and specification guidance |
| mcp-security-auditor | MCP server security audit |
| mcp-server-architect | MCP server architecture and transport design |
| mcp-testing-engineer | MCP server and tool testing |
| monitoring-specialist | Observability, alerting, and SLO design |
| penetration-tester | Penetration testing and exploit research |
| performance-engineer | System-wide performance profiling and optimization |
| risk-manager | Technical risk assessment and mitigation planning |
| security-auditor | Code security review and OWASP compliance |
| security-engineer | Security engineering and hardening implementation |
| task-decomposition-expert | Break complex tasks into parallelizable sub-tasks |
| tdd-coding-agent | Test-driven development implementation |

### Haiku tier (81 agents)

| Agent | Description |
|-------|-------------|
| academic-researcher | Academic literature search and citation management |
| agent-overview | Summarize agent capabilities and selection |
| ai-engineer | AI/ML model integration and inference pipeline engineering |
| api-documenter | Write and maintain API reference documentation |
| api-explorer | Explore and map undocumented APIs |
| authentik-api-specialist | Authentik identity provider API integration |
| business-analyst | Business requirements analysis and process modeling |
| changelog-generator | Generate structured changelogs from git history |
| cli-ui-designer | CLI and TUI interface design |
| command-expert | Shell command composition and scripting |
| content-marketer | Content strategy and technical marketing writing |
| context-manager | Conversation context summarization and handoff |
| data-analyst | Data analysis, visualization, and statistical reporting |
| data-engineer | ETL pipeline design and data infrastructure |
| data-scientist | Machine learning workflows and experiment design |
| database-admin | Database administration, backup, and recovery |
| database-optimization | Query optimization and index design |
| database-optimizer | Database performance profiling and tuning |
| debugger | Step-by-step debugging and hypothesis testing |
| dependency-manager | Dependency auditing, updates, and conflict resolution |
| devops-engineer | CI/CD pipeline implementation and cloud operations |
| devops-troubleshooter | DevOps incident diagnosis and remediation |
| document-structure-analyzer | Document structure analysis and information extraction |
| documentation-expert | Technical documentation writing and maintenance |
| dx-optimizer | Developer experience auditing and toolchain improvement |
| fact-checker | Claim verification and source validation |
| flutter-specialist | Flutter UI and state management implementation |
| frontend-developer | Frontend implementation with modern frameworks |
| fullstack-developer | Full-stack feature implementation |
| git-flow-manager | Git workflow management and branch strategy |
| go-specialist | Go systems programming and concurrency patterns |
| golang-pro | Idiomatic Go and standard-library expertise |
| ios-developer | iOS and SwiftUI application development |
| javascript-pro | Modern JavaScript and browser API expertise |
| llms-maintainer | LLM integration maintenance and prompt management |
| markdown-syntax-formatter | Markdown formatting and linting |
| metadata-agent | Metadata extraction and schema mapping |
| ml-engineer | ML model training, evaluation, and deployment |
| mlops-engineer | ML operations, experiment tracking, and model serving |
| mobile-developer | Cross-platform mobile development |
| model-evaluator | LLM evaluation and benchmark design |
| network-engineer | Network configuration and security policy |
| nextjs-architecture-expert | Next.js architecture and server-component design |
| nosql-specialist | NoSQL schema design and query optimization |
| performance-profiler | Low-level performance profiling and flamegraph analysis |
| product-strategist | Product roadmap and feature prioritization |
| project-supervisor-orchestrator | Project workflow coordination |
| prompt-engineer | Prompt design, evaluation, and optimization |
| python-pro | Idiomatic Python and ecosystem expertise |
| python-specialist | FastAPI, Django, data processing, and async Python |
| quant-analyst | Quantitative analysis and financial modeling |
| query-clarifier | Disambiguate underspecified requests before execution |
| react-performance-optimization | React rendering optimization and bundle analysis |
| react-performance-optimizer | React profiling and memoization strategies |
| report-generator | Structured report generation from data or research |
| research-brief-generator | Research brief creation and scoping |
| research-coordinator | Multi-source research coordination |
| research-orchestrator | Research workflow orchestration across agents |
| research-synthesizer | Cross-source research synthesis and summarization |
| review-agent | General-purpose review and critique |
| rust-pro | Idiomatic Rust and standard-library expertise |
| rust-specialist | Rust systems programming, memory safety, and async patterns |
| salesforce-api-specialist | Salesforce API integration and Apex development |
| search-specialist | Search-index design and query relevance tuning |
| shell-scripting-pro | POSIX shell and Bash scripting |
| sql-pro | Advanced SQL, window functions, and query design |
| supabase-realtime-optimizer | Supabase Realtime performance and subscription design |
| supabase-schema-architect | Supabase schema design and RLS policy implementation |
| swift-specialist | Swift language and Apple platform APIs |
| tag-agent | Content tagging and taxonomy management |
| technical-researcher | Technical research and competitive analysis |
| technical-writer | Technical documentation for developer audiences |
| terraform-specialist | Terraform module design and infrastructure-as-code |
| test-automator | Test automation framework design and implementation |
| test-engineer | Unit, integration, and end-to-end test engineering |
| typescript-pro | Advanced TypeScript type system and modern patterns |
| ui-ux-designer | UI/UX design and accessibility review |
| unused-code-cleaner | Dead-code detection and removal |
| url-link-extractor | URL extraction and link validation |
| web-accessibility-checker | WCAG compliance and accessibility audit |
| web-vitals-optimizer | Core Web Vitals measurement and optimization |

## Delegation nudge

The delegation nudge is a soft PostToolUse hook (`cco hook post-tool-use`) that fires when the top-level orchestrator is about to do implementation work that belongs in a lower-cost tier. It is non-blocking — Claude Code receives a warning message, not a hard stop — so the orchestrator can continue if needed.

The nudge fires on two patterns:

1. **Inline implementation** — the orchestrator calls `Bash` with a command that creates or modifies source files (a redirect to `.rs`, `.ts`, `.py`, etc., or commands such as `cargo build`, `npm install`, or `pytest`).
2. **Wrong-tier tool use** — the orchestrator calls `Write` or `Edit` directly on source files rather than delegating to a specialist agent.

The nudge is rate-limited per trigger type so it does not repeat on every tool call in a session. It is silenced entirely when `CCO_BUDGET_GATE=0` is set.

**The nudge only fires for the top-level orchestrator** (where `agent_id` is null in the hook input). Subagents are intentionally exempt — they are already doing delegated work.

## Adding an agent

1. Create `src/agents/my-agent.md` with valid YAML frontmatter and a system prompt body.
2. Set `model` to one of `haiku`, `sonnet`, or `opus` based on task complexity.
3. Run `cargo build` — the build script picks up the new file automatically.
4. Verify with `cco agents list`.

## Build and test commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (or: make release)
make install                   # Build release and install to ~/.local/bin

cargo test --lib               # Library unit tests
cargo test --test '*'          # Integration tests
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all                # Format code
```

## Code style

- Imports: standard library, then external crates, then `crate::`, then `super::` — each group alphabetically sorted, one import per line.
- Naming: `PascalCase` for types and traits; `snake_case` for functions, variables, and fields; `SCREAMING_SNAKE_CASE` for constants; acronyms treated as words (`ApiClient`, not `APIClient`).
- Error handling: `thiserror` for typed errors, `anyhow` for application-level propagation; use `?` throughout; avoid `.unwrap()` outside tests.
- Documentation: rustdoc comments (`///`) on all public items; include examples where useful.
- No `.unwrap()` / `.expect()` in production code paths.
- No clippy warnings (`-D warnings` in CI); no skipping lint with `#[allow(...)]` without a comment explaining why.

## Development workflow

1. Make changes.
2. Run the affected unit test: `cargo test test_name`.
3. Run library tests: `cargo test --lib`.
4. Check formatting: `cargo fmt --all -- --check`.
5. Check lints: `cargo clippy --all-targets --all-features -- -D warnings`.
6. Run full test suite: `make test`.
7. Open a pull request; do not commit directly to `main`.
