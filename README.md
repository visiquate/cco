# CCO - Enhancement Layer for Claude Code

CCO enhances Claude Code with intelligent command classification, persistent memory, cost monitoring, and multi-agent orchestration—without modifying Claude Code itself.

## What is CCO?

CCO is an optional enhancement layer for Claude Code that adds powerful productivity features through Claude Code's official plugin system. You can use Claude Code in its native form anytime by running `claude` directly, or get the enhanced experience by running `cco`.

### Key Features

- **Intelligent Command Classification** - Auto-approves safe READ operations (ls, cat, git status), maintains security for CREATE/UPDATE/DELETE
- **Persistent Knowledge** - Context survives conversation compactions through integrated knowledge management
- **Real-Time Cost Monitoring** - Track token usage and API costs with built-in dashboard
- **Secure Credential Management** - OS keyring integration (Keychain/Secret Service/DPAPI) with FIPS 140-2 compliant encryption
- **Multi-Agent Orchestration** - 119 specialized agents organized in 13 functional categories with TDD-first methodology

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap visiquate/cco
brew install cco
```

### Shell Script (One-Line Install)

```bash
curl -fsSL https://raw.githubusercontent.com/visiquate/cco-releases/main/install.sh | bash
```

### Manual Download

Download the latest release for your platform from [Releases](https://github.com/visiquate/cco-releases/releases):

- macOS (Apple Silicon): `cco-aarch64-apple-darwin.tar.gz`
- macOS (Intel): `cco-x86_64-apple-darwin.tar.gz`
- Linux (x86_64): `cco-x86_64-unknown-linux-gnu.tar.gz`
- Linux (ARM64): `cco-aarch64-unknown-linux-gnu.tar.gz`
- Windows (x86_64): `cco-x86_64-pc-windows-msvc.zip`

Extract and add to PATH:

```bash
tar -xzf cco-*.tar.gz
sudo mv cco /usr/local/bin/
```

## Quick Start

### Basic Usage

```bash
# Launch enhanced Claude Code
cco

# View available commands
cco help

# Check version
cco --version
```

### Command Classification

CCO automatically approves safe READ operations while maintaining security:

```bash
# Auto-approved (no permission prompt)
ls -la
cat config.json
git status
ps aux | grep node

# Still require confirmation (CREATE/UPDATE/DELETE)
touch newfile.txt
echo "data" > file.txt
rm -rf directory/
git commit -m "message"
```

### Credential Management

```bash
# Store credentials securely
cco credentials store api-key "your-key-here" --service github

# Retrieve credentials
cco credentials retrieve api-key

# List all credentials
cco credentials list
```

### Knowledge Manager

```bash
# Search knowledge base
cco knowledge search "authentication"

# View statistics
cco knowledge stats

# List recent entries
cco knowledge list --limit 20
```

### Cost Monitoring

```bash
# View cost dashboard
cco cost dashboard

# Show session costs
cco cost session

# Export cost data
cco cost export --format json
```

## Agent Architecture

CCO includes 119 specialized agents organized in 13 categories:

### Model Distribution

- **1 agent** uses Opus 4.1 (Chief Architect - strategic decisions)
- **37 agents** use Sonnet 4.5 (managers, reviewers, complex coding)
- **81 agents** use Haiku 4.5 (basic coders, documentation, utilities)

### Agent Categories

1. **Development** - TDD Coding Agent, Language Specialists (Python, Swift, Go, Rust, Flutter)
2. **Infrastructure** - DevOps Engineer, Cloud Architect, Terraform Specialist
3. **Quality Assurance** - Test Engineer, Security Auditor, Performance Engineer
4. **Documentation** - Technical Writer, API Documenter
5. **Research** - Technical Researcher, Data Scientist
6. **Support** - Credential Manager, DX Optimizer, Git Flow Manager

## CLI Command Reference

### Core Commands

```bash
cco                          # Launch enhanced Claude Code
cco help                     # Show help and usage
cco --version               # Show version information
```

### Credential Commands

```bash
cco credentials store <key> <value>     # Store credential
cco credentials retrieve <key>          # Retrieve credential
cco credentials list                    # List all credentials
cco credentials delete <key>            # Delete credential
cco credentials check-rotation          # Check rotation status
```

### Knowledge Manager Commands

```bash
cco knowledge search <query>            # Search knowledge base
cco knowledge store <text>              # Store knowledge entry
cco knowledge list                      # List knowledge entries
cco knowledge stats                     # Show statistics
```

### Cost Monitoring Commands

```bash
cco cost dashboard                      # View cost dashboard
cco cost session                        # Show session costs
cco cost export                         # Export cost data
```

### Orchestra Commands

```bash
cco orchestra                           # View orchestra configuration
cco orchestra conduct <requirement>     # Generate workflow
```

## Performance

- **2.8-4.4x faster** than sequential development with multi-agent orchestration
- **32% token reduction** via shared Knowledge Manager
- **60-80% fewer permission prompts** with intelligent command classification

## Requirements

- **Claude Code** - Latest version (0.8.0+) installed and in PATH
- **Rust** - 1.70+ (for building from source)
- **OS Support** - macOS, Linux, Windows

## Documentation

- [Quick Start Guide](https://github.com/visiquate/cc-orchestra/blob/main/docs/QUICK_START.md)
- [Orchestra Usage Guide](https://github.com/visiquate/cc-orchestra/blob/main/docs/ORCHESTRA_USAGE_GUIDE.md)
- [Agent Architecture](https://github.com/visiquate/cc-orchestra/blob/main/docs/AGENT_ARCHITECTURE.md)
- [Cost Monitoring](https://github.com/visiquate/cc-orchestra/blob/main/docs/COST_MONITORING.md)
- [Credential Security](https://github.com/visiquate/cc-orchestra/blob/main/docs/CREDENTIAL_SECURITY.md)

## Source Code

The source code for CCO is available at [github.com/visiquate/cc-orchestra](https://github.com/visiquate/cc-orchestra).

## License

MIT
