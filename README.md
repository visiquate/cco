# CCO - Enhancement Layer for Claude Code

CCO enhances Claude Code with intelligent command classification, persistent memory, and multi-agent orchestration—without modifying Claude Code itself.

## What is CCO?

CCO is an optional enhancement layer for Claude Code that adds productivity features through Claude Code's official plugin system. Run `claude` directly for native Claude Code, or `cco` for the enhanced experience.

### Key Features

- **Intelligent Command Classification** - Auto-approves safe READ operations (ls, cat, git status), requires confirmation for CREATE/UPDATE/DELETE
- **Persistent Knowledge** - Context survives conversation compactions through integrated knowledge management
- **Secure Credential Management** - OS keyring integration for secure secret storage
- **Multi-Agent Orchestration** - 117 specialized agents organized in 13 functional categories

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap visiquate/cco
brew install cco
```

### Shell Script (One-Line Install)

```bash
curl -fsSL https://raw.githubusercontent.com/visiquate/cco/main/install.sh | bash
```

### Manual Download

Download the latest release for your platform from [Releases](https://github.com/visiquate/cco/releases):

- macOS (Apple Silicon): `cco-aarch64-apple-darwin.tar.gz`
- macOS (Intel): `cco-x86_64-apple-darwin.tar.gz`
- Linux (x86_64): `cco-x86_64-unknown-linux-gnu.tar.gz`
- Linux (ARM64): `cco-aarch64-unknown-linux-gnu.tar.gz`

Extract and add to PATH:

```bash
tar -xzf cco-*.tar.gz
sudo mv cco /usr/local/bin/
```

## Quick Start

```bash
# Launch enhanced Claude Code
cco

# View available commands
cco help

# Check version
cco --version
```

## CLI Commands

### Core

```bash
cco                    # Launch enhanced Claude Code
cco help               # Show help
cco --version          # Show version
cco health             # Check health status
cco status             # Show running instances
```

### Credentials

```bash
cco credentials store <key> <value>   # Store credential
cco credentials retrieve <key>        # Retrieve credential
cco credentials list                  # List all credentials
cco credentials delete <key>          # Delete credential
```

### Knowledge

```bash
cco knowledge search <query>          # Search knowledge base
cco knowledge store <text>            # Store knowledge entry
cco knowledge stats                   # Show statistics
```

### Orchestra

```bash
cco orchestra stats                   # Show agent statistics
cco orchestra generate <requirement>  # Generate agent spawn instructions
cco orchestra workflow <requirement>  # Generate complete workflow
```

### Classify

```bash
cco classify <command>                # Classify a shell command (READ/CREATE/UPDATE/DELETE)
```

## Agent Architecture

CCO includes 117 specialized agents in 13 categories:

### Model Distribution

- **1 agent** uses Opus (Chief Architect - strategic decisions)
- **35 agents** use Sonnet (managers, reviewers, complex coding)
- **81 agents** use Haiku (basic coders, documentation, utilities)

### Categories

- **Development** (28) - Language specialists, TDD agent
- **Support** (18) - DX optimizer, git flow, utilities
- **Data** (11) - Data engineering, analysis
- **Infrastructure** (10) - DevOps, cloud, Terraform
- **Research** (10) - Technical research, data science
- **Security** (8) - Auditing, compliance
- **Documentation** (7) - Technical writing, API docs
- **Coding** (6) - Code review, refactoring
- **MCP** (6) - MCP protocol specialists
- **AI/ML** (5) - ML engineering, model evaluation
- **Business** (4) - Analysis, product strategy
- **Integration** (3) - API integration
- **Leadership** (1) - Chief Architect

## Requirements

- **Claude Code** - Version 0.8.0+ installed and in PATH
- **OS Support** - macOS, Linux
