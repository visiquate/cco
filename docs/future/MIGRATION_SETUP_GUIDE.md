# Claude Orchestra Migration Setup Guide

Complete step-by-step guide for setting up Claude Orchestra on a new Mac.

**⚠️ POST-MIGRATION UPDATE (2025-11-09)**: MCP servers (claude-flow, ruv-swarm, flow-nexus, agentic-payments) have been **removed** and replaced with the built-in **Knowledge Manager** system. This guide has been updated to reflect the simplified architecture.

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Pre-Migration Backup](#pre-migration-backup)
3. [Core System Setup](#core-system-setup)
4. [~~MCP Server Configuration~~](#mcp-server-configuration-historical) *(Historical - No Longer Needed)*
5. [Claude Orchestra Installation](#claude-army-installation)
6. [Remote LLM Configuration](#remote-llm-configuration)
7. [Knowledge Manager Setup](#knowledge-manager-setup)
8. [File Locations to Preserve](#file-locations-to-preserve)
9. [Testing Checklist](#testing-checklist)
10. [Quick Start Commands](#quick-start-commands)
11. [Troubleshooting](#troubleshooting)

---

## System Requirements

### Node.js and npm
- **Node.js**: v16.0.0 or higher (tested with v24.10.0)
- **npm**: v7.0.0 or higher (tested with v11.6.0)

### Installation via Homebrew (macOS)
```bash
# Install Homebrew if not present
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Node.js (includes npm)
brew install node

# Verify installation
node --version  # Should be v16+
npm --version   # Should be v7+
```

### Git Configuration
```bash
# Verify Git is installed
git --version

# Configure global user (if not already done)
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Set up global .gitignore (CRITICAL for Claude Orchestra)
git config --global core.excludesfile ~/.gitignore_global
```

### Python (Optional - for future features)
```bash
# If Python features are needed
brew install python3

# Verify
python3 --version
```

---

## Pre-Migration Backup

Before migrating, back up these critical files and directories from your current Mac:

### 1. Global Configuration
```bash
# On OLD Mac - backup these files
cp ~/.claude/CLAUDE.md ~/Desktop/backup/
cp ~/.claude/settings.local.json ~/Desktop/backup/
cp ~/.gitignore_global ~/Desktop/backup/

# Create backup archive
cd ~
tar -czf ~/Desktop/claude-army-backup.tar.gz \
  .claude/CLAUDE.md \
  .claude/settings.local.json \
  .gitignore_global
```

### 2. Claude Orchestra Repository
```bash
# On OLD Mac - backup the entire repository
cd ~/git
tar -czf ~/Desktop/cc-army-backup.tar.gz cc-army/

# OR use git to ensure clean state
cd ~/git/cc-army
git status  # Ensure no uncommitted changes
git push origin main  # Push any commits
```

### 3. Knowledge Databases
```bash
# On OLD Mac - backup LanceDB databases
cd ~/git/cc-army
tar -czf ~/Desktop/knowledge-backup.tar.gz data/knowledge/

# Check size
du -sh data/knowledge/
```

### 4. Credentials (SECURE HANDLING)
```bash
# On OLD Mac - backup credentials securely
# DO NOT store in cloud or unencrypted locations
cp /tmp/credentials.json ~/Desktop/backup/credentials.json.backup

# Encrypt if transferring via network
openssl enc -aes-256-cbc -salt \
  -in /tmp/credentials.json \
  -out ~/Desktop/credentials.json.enc
```

---

## Core System Setup

### 1. Create Directory Structure

```bash
# On NEW Mac
mkdir -p ~/git
mkdir -p ~/.claude
mkdir -p ~/Desktop/backup
```

### 2. Install Global Tools

```bash
# Install GitHub CLI (for PR management)
brew install gh

# Install curl (usually pre-installed)
which curl || brew install curl

# Install jq (JSON processing)
brew install jq

# Authenticate with GitHub
gh auth login
```

---

## MCP Server Configuration (HISTORICAL)

**⚠️ HISTORICAL SECTION**: MCP servers are no longer needed. This section is kept for reference only.

### Current Approach (Post-Migration)

Claude Orchestra now uses the **built-in Knowledge Manager** instead of external MCP servers:

```bash
# No MCP server installation needed!

# Knowledge Manager is built into the repository
cd ~/git/cc-army
node src/knowledge-manager.js stats

# See "Knowledge Manager Setup" section below for details
```

**Benefits of removing MCP servers:**
- ✅ Simpler setup (no global npm packages)
- ✅ No Claude Code settings configuration needed
- ✅ Fewer dependencies to manage
- ✅ Same functionality with built-in tools

### Historical MCP Server Information

**The following was previously required but is NO LONGER NEEDED:**

<details>
<summary>Click to view historical MCP setup instructions (not needed anymore)</summary>

#### Understanding MCP Servers (Historical)

Claude Orchestra previously used MCP (Model Context Protocol) servers for agent coordination:
- **claude-flow@alpha**: Core multi-agent workflow orchestration (REMOVED)
- **ruv-swarm**: Enhanced swarm coordination (REMOVED)
- **flow-nexus**: Cloud-based orchestration (REMOVED)
- **agentic-payments**: Payment processing (REMOVED)

#### 1. Install Claude Flow (NO LONGER NEEDED)

```bash
# HISTORICAL - Do not run these commands
npm install -g claude-flow@alpha
npx claude-flow@alpha --version
npx claude-flow@alpha mcp start
```

#### 2. Install Ruv Swarm (NO LONGER NEEDED)

```bash
# HISTORICAL - Do not run these commands
npm install -g ruv-swarm
npx ruv-swarm --version
npx ruv-swarm mcp start
```

#### 3. Configure Claude Code Settings (NO LONGER NEEDED)

```bash
# HISTORICAL - These settings are no longer needed
# Claude Code settings.local.json no longer requires MCP server configuration
```

</details>

### 4. Restore Global CLAUDE.md

```bash
# On NEW Mac - restore from backup
cp ~/Desktop/backup/CLAUDE.md ~/.claude/CLAUDE.md

# OR download latest from repository (if backed up via git)
curl -o ~/.claude/CLAUDE.md https://raw.githubusercontent.com/yourusername/config/main/.claude/CLAUDE.md

# Verify
ls -la ~/.claude/CLAUDE.md
```

### 5. Verify Knowledge Manager Configuration

```bash
# Test Knowledge Manager (replaces MCP verification)
cd ~/git/cc-army
node src/knowledge-manager.js test

# View statistics
node src/knowledge-manager.js stats

# Expected output:
# ✅ Knowledge Manager initialized for repository: cc-army
# ✅ Connected to existing knowledge base
```

---

## Claude Orchestra Installation

### 1. Clone or Restore Repository

**Option A: Clone from GitHub**
```bash
cd ~/git
git clone https://github.com/yourusername/cc-army.git
cd cc-army
```

**Option B: Restore from Backup**
```bash
cd ~/git
tar -xzf ~/Desktop/cc-army-backup.tar.gz
cd cc-army
```

### 2. Install Dependencies

```bash
cd ~/git/cc-army

# Install npm dependencies
npm install

# Verify vectordb (LanceDB) is installed
npm list vectordb
# Should show: vectordb@0.21.2
```

### 3. Verify Installation

```bash
# Check package.json
cat package.json

# Run army orchestrator test
node src/orchestra-conductor.js --help

# Test LLM router
node src/llm-router.js stats

# Test knowledge manager
node src/knowledge-manager.js test
```

### 4. Verify Directory Structure

```bash
cd ~/git/cc-army

# Should have this structure:
tree -L 2 -I node_modules
```

Expected output:
```
.
├── CLAUDE.md
├── README.md
├── config
│   ├── orchestra-config.json
│   └── credential-inventory.json
├── data
│   └── knowledge
├── docs
│   ├── API_INTEGRATION_GUIDE.md
│   ├── ARMY_USAGE_GUIDE.md
│   └── [other docs]
├── package.json
├── package-lock.json
└── src
    ├── orchestra-conductor.js
    ├── credential-manager.js
    ├── knowledge-manager.js
    └── llm-router.js
```

---

## Remote LLM Configuration

### Overview

Claude Orchestra uses **coder.visiquate.com** for specialized coding tasks:
- Architecture/Planning → Claude API (via Claude Code)
- Coding/Implementation → Remote LLM (coder.visiquate.com)

### Available Models

**1. qwen2.5-coder:7b-instruct** (qwen-fast)
- **Size**: 7B parameters
- **Context**: 32k tokens
- **Use Case**: Fast, simple coding tasks
- **Speed**: ~50 tokens/second
- **Best For**: Simple functions, bug fixes, quick implementations

**2. qwen2.5-coder:32b-instruct-q8** (qwen-quality-128k)
- **Size**: 32B parameters (8-bit quantized)
- **Context**: 128k tokens
- **Use Case**: Complex, high-quality coding
- **Speed**: ~20 tokens/second
- **Best For**: Complex algorithms, full-stack features, production code

### Configuration in orchestra-config.json

The LLM routing is already configured in `config/orchestra-config.json`:

```json
{
  "llmRouting": {
    "enabled": true,
    "endpoints": {
      "coding": {
        "enabled": true,
        "type": "ollama",
        "url": "https://coder.visiquate.com",
        "defaultModel": "qwen2.5-coder:32b-instruct",
        "temperature": 0.7,
        "maxTokens": 4096,
        "headers": {},
        "additionalParams": {}
      }
    },
    "rules": {
      "architectureTasks": "claude",
      "codingTasks": "custom-if-enabled",
      "fallbackToClaude": true
    }
  }
}
```

### Model Selection Logic (in llm-router.js)

The router automatically detects Ollama endpoints and uses appropriate format:

```javascript
// Detected automatically based on:
// 1. endpoint.type === 'ollama'
// 2. endpoint.url contains 'ollama'
// 3. endpoint.defaultModel contains 'qwen' or 'llama'

// Ollama API format used:
{
  model: "qwen2.5-coder:32b-instruct",
  prompt: "your coding task",
  stream: false,
  options: {
    temperature: 0.7,
    num_predict: 4096
  }
}
```

### Testing Remote LLM Connection

```bash
cd ~/git/cc-army

# Test routing configuration
node src/llm-router.js stats

# Test routing decisions
node src/llm-router.js route python-expert implement
node src/llm-router.js route system-architect planning

# Test direct LLM call
node src/llm-router.js call-coding-llm "Write a Python hello world function"
```

Expected output for coding task:
```json
{
  "endpoint": "custom",
  "url": "https://coder.visiquate.com",
  "useClaudeCode": false,
  "reason": "Coding tasks routed to custom LLM"
}
```

### Authentication/Credentials

**Current Status**: No authentication required for coder.visiquate.com
- Internal VisiQuate service
- Already accessible from your network
- No API keys needed

**If Authentication is Added Later**:
```bash
# Store API key in credential manager
node src/credential-manager.js store CODER_API_KEY "your-key-here" api_key

# Update orchestra-config.json
# Add: "apiKey": "${CODER_API_KEY}"
```

### Network Requirements

- **Endpoint**: https://coder.visiquate.com
- **Protocol**: HTTPS
- **Port**: 443 (standard HTTPS)
- **Latency**: ~100-200ms (depending on model size)
- **Bandwidth**: Minimal (streaming disabled by default)

### When to Use Which Model

**Use qwen-fast (7B) for:**
- Simple function implementations
- Bug fixes
- Code refactoring
- Unit tests
- Quick prototypes

**Use qwen-quality-128k (32B) for:**
- Complex algorithms
- Full-stack features
- Production-critical code
- Large codebases
- Multi-file implementations

**Use Claude (via Claude Code) for:**
- Architecture decisions
- System design
- Strategic planning
- Requirements analysis
- User requirement clarification

---

## Knowledge Manager Setup

### Understanding Knowledge Manager

The Knowledge Manager uses LanceDB to provide:
- **Persistent Memory**: Survives Claude Code compactions
- **Per-Repository Context**: Each repo has isolated knowledge
- **Semantic Search**: Vector-based knowledge retrieval
- **Automatic Capture**: Pre/post-compaction hooks

### Database Location

```
/Users/brent/git/cc-orchestra/data/knowledge/
├── cc-army/              # Claude Orchestra project knowledge
│   └── army_knowledge/   # LanceDB table
├── statushub/            # StatusHub project knowledge
│   └── army_knowledge/   # LanceDB table
└── [repo-name]/          # Other repositories...
```

### 1. Restore Knowledge Databases

```bash
# On NEW Mac - restore from backup
cd ~/git/cc-army
tar -xzf ~/Desktop/knowledge-backup.tar.gz

# Verify restoration
ls -la data/knowledge/
du -sh data/knowledge/*/
```

### 2. Test Knowledge Manager

```bash
cd ~/git/cc-army

# Run comprehensive test
node src/knowledge-manager.js test

# Expected output:
# 📦 Knowledge Manager initialized for repository: cc-army
# 📁 Database path: /Users/brent/git/cc-orchestra/data/knowledge/cc-army
# ✅ Connected to existing knowledge base for cc-army
#
# 1. Storing test knowledge...
# ✅ Stored knowledge: decision from architect
# ...
# ✅ Test complete!
```

### 3. Verify Per-Repository Isolation

```bash
# Test with different repositories
cd ~/git/statushub
node ~/git/cc-orchestra/src/knowledge-manager.js stats
# Should show: "repository": "statushub"

cd ~/git/slack-broker
node ~/git/cc-orchestra/src/knowledge-manager.js stats
# Should show: "repository": "slack-broker"

cd ~/git/cc-army
node src/knowledge-manager.js stats
# Should show: "repository": "cc-army"
```

### 4. Knowledge Configuration

Configuration is in `config/orchestra-config.json`:

```json
{
  "knowledgeManager": {
    "enabled": true,
    "perRepositoryContext": true,
    "baseDir": "data/knowledge",
    "embeddingDim": 384,
    "autoCapture": {
      "enabled": true,
      "preCompaction": true,
      "postCompaction": true
    }
  }
}
```

### 5. Test Knowledge Operations

```bash
# Store knowledge
node src/knowledge-manager.js store "Test migration knowledge" general

# Search knowledge
node src/knowledge-manager.js search "migration" 5

# View statistics
node src/knowledge-manager.js stats
```

### 6. Migration-Specific Knowledge Backup

If you want to export knowledge for documentation:

```bash
# On OLD Mac (before migration)
cd ~/git/cc-army
node src/knowledge-manager.js stats > ~/Desktop/knowledge-pre-migration.json

# On NEW Mac (after migration)
cd ~/git/cc-army
node src/knowledge-manager.js stats > ~/Desktop/knowledge-post-migration.json

# Compare
diff ~/Desktop/knowledge-pre-migration.json ~/Desktop/knowledge-post-migration.json
```

---

## File Locations to Preserve

### Critical Files and Their Purposes

| File Path | Purpose | Backup Priority | Restore Method |
|-----------|---------|----------------|----------------|
| `~/.claude/CLAUDE.md` | Global Claude Code instructions | CRITICAL | Direct copy |
| `~/.claude/settings.local.json` | Claude Code configuration (MCP no longer needed) | MEDIUM | Direct copy |
| `~/.gitignore_global` | Prevents Claude files from git | HIGH | Direct copy |
| `/Users/brent/git/cc-orchestra/` | Full Claude Orchestra repository | CRITICAL | Git clone or tar |
| `/Users/brent/git/cc-orchestra/config/orchestra-config.json` | Agent and routing config | CRITICAL | Via repository |
| `/Users/brent/git/cc-orchestra/data/knowledge/` | LanceDB knowledge databases | HIGH | Tar archive |
| `/tmp/credentials.json` | Temporary credentials (dev only) | MEDIUM | Encrypted transfer |
| `~/.npmrc` | npm configuration (if customized) | LOW | Direct copy |

### Secure Credential Migration

```bash
# On OLD Mac - encrypt credentials
openssl enc -aes-256-cbc -salt \
  -in /tmp/credentials.json \
  -out ~/Desktop/credentials.json.enc
# Enter encryption password when prompted

# Transfer credentials.json.enc to NEW Mac via secure method
# (USB drive, secure file transfer, etc.)

# On NEW Mac - decrypt credentials
openssl enc -d -aes-256-cbc \
  -in ~/Desktop/credentials.json.enc \
  -out /tmp/credentials.json
# Enter same encryption password

# Set proper permissions
chmod 600 /tmp/credentials.json
```

### Additional Files to Consider

```bash
# SSH keys (for GitHub, servers)
~/.ssh/id_rsa
~/.ssh/id_rsa.pub
~/.ssh/config
~/.ssh/known_hosts

# Git configuration
~/.gitconfig

# Shell configuration (if customized)
~/.zshrc
~/.bash_profile

# Claude Code workspace settings (if project-specific)
[project]/.claude/
```

---

## Testing Checklist

### 1. System Requirements Test

```bash
# ✅ Node.js version
node --version
# Expected: v16.0.0 or higher

# ✅ npm version
npm --version
# Expected: v7.0.0 or higher

# ✅ Git installed
git --version

# ✅ GitHub CLI installed
gh --version

# ✅ Directory structure
ls -la ~/git/cc-army
ls -la ~/.claude
```

### 2. Knowledge Manager Test (Replaces MCP Server Test)

```bash
# ✅ Knowledge Manager works
cd ~/git/cc-army
node src/knowledge-manager.js test

# ✅ Statistics available
node src/knowledge-manager.js stats | jq .

# ✅ Database directory exists
ls -la data/knowledge/

# ✅ Can store and retrieve knowledge
node src/knowledge-manager.js store "Migration test" --type decision --agent architect
node src/knowledge-manager.js search "migration"
```

### 3. Claude Orchestra Installation Test

```bash
cd ~/git/cc-army

# ✅ Dependencies installed
npm list | grep vectordb
# Expected: vectordb@0.21.2

# ✅ Orchestrator works
node src/orchestra-conductor.js --version || echo "Orchestrator exists"

# ✅ LLM Router works
node src/llm-router.js stats

# ✅ Credential Manager works
node src/credential-manager.js list

# ✅ Knowledge Manager works
node src/knowledge-manager.js test
```

### 4. Remote LLM Connection Test

```bash
cd ~/git/cc-army

# ✅ Routing configuration loaded
node src/llm-router.js stats | jq .

# ✅ Architecture tasks route to Claude
node src/llm-router.js route system-architect planning
# Expected: "endpoint": "claude"

# ✅ Coding tasks route to custom LLM
node src/llm-router.js route python-expert implement
# Expected: "endpoint": "custom"

# ✅ Can call remote LLM
node src/llm-router.js call-coding-llm "def hello(): return 'world'"
# Expected: Response from coder.visiquate.com
```

### 5. Knowledge Manager Test

```bash
cd ~/git/cc-army

# ✅ Knowledge database exists
ls -la data/knowledge/cc-army/

# ✅ Can store knowledge
node src/knowledge-manager.js store "Migration test" general

# ✅ Can search knowledge
node src/knowledge-manager.js search "migration" 5

# ✅ Statistics work
node src/knowledge-manager.js stats | jq .

# ✅ Per-repository isolation works
cd ~/git/statushub
node ~/git/cc-orchestra/src/knowledge-manager.js stats | grep repository
# Should show: "repository": "statushub"
```

### 6. Token Savings Verification

```bash
# ✅ Shared memory reduces tokens
# This is verified during actual usage in Claude Code
# Look for memory_store/retrieve operations in agent logs
```

### 7. Full Integration Test

Open Claude Code and test:

```
1. Navigate to a test project
2. Request: "Analyze this codebase architecture"
   - Should use Claude (Opus) via Claude Code

3. Request: "Implement a Python function for JWT validation"
   - Should route to coder.visiquate.com
   - Check llm-router logs for confirmation

4. Check knowledge was stored:
   cd [project-dir]
   node ~/git/cc-orchestra/src/knowledge-manager.js stats
```

---

## Quick Start Commands

### One-Time Setup Script

Create `~/setup-claude-army.sh`:

```bash
#!/bin/bash
set -e

echo "🚀 Claude Orchestra Setup Script"
echo "==========================="

# 1. Install system dependencies
echo "📦 Installing system dependencies..."
which brew || /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install node gh jq

# 2. NO MCP server installation needed (removed in migration)
echo "ℹ️  MCP servers no longer needed - using built-in Knowledge Manager"

# 3. Create directories
echo "📁 Creating directories..."
mkdir -p ~/git ~/.claude ~/Desktop/backup

# 4. Clone repository (if not exists)
if [ ! -d ~/git/cc-army ]; then
  echo "📥 Cloning Claude Orchestra repository..."
  cd ~/git
  git clone https://github.com/yourusername/cc-army.git
  cd cc-army
  npm install
fi

# 5. Verify installation
echo "✅ Verifying installation..."
cd ~/git/cc-army
node src/knowledge-manager.js test

echo ""
echo "✨ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Restore ~/.claude/CLAUDE.md from backup"
echo "2. Restore data/knowledge/ from backup (if migrating)"
echo "3. Run test checklist"
echo "4. Note: MCP server configuration no longer needed!"
```

Make executable and run:
```bash
chmod +x ~/setup-claude-army.sh
~/setup-claude-army.sh
```

### Quick Verification Commands

```bash
# Verify everything is working
cd ~/git/cc-army && \
  echo "Node: $(node --version)" && \
  echo "npm: $(npm --version)" && \
  echo "Dependencies: $(npm list vectordb | grep vectordb)" && \
  node src/llm-router.js stats && \
  node src/knowledge-manager.js stats
  # Note: MCP server checks removed (no longer needed)
```

### Daily Usage Commands

```bash
# Check Claude Orchestra status
alias army-status='cd ~/git/cc-army && node src/llm-router.js stats && node src/knowledge-manager.js stats'

# Search knowledge across all repos
alias army-search='node ~/git/cc-orchestra/src/knowledge-manager.js search'

# View routing configuration
alias army-routing='node ~/git/cc-orchestra/src/llm-router.js stats | jq .'
```

Add to `~/.zshrc` or `~/.bash_profile`:
```bash
# Claude Orchestra aliases
export CLAUDE_ARMY_HOME="$HOME/git/cc-army"
alias army-status='cd $CLAUDE_ARMY_HOME && node src/llm-router.js stats && node src/knowledge-manager.js stats'
alias army-search='node $CLAUDE_ARMY_HOME/src/knowledge-manager.js search'
alias army-routing='node $CLAUDE_ARMY_HOME/src/llm-router.js stats | jq .'
alias army-test='cd $CLAUDE_ARMY_HOME && node src/knowledge-manager.js test'
```

---

## Troubleshooting

### Issue: npm install fails with vectordb

**Symptoms:**
```
npm ERR! Could not resolve dependency:
npm ERR! vectordb@0.21.2
```

**Solution:**
```bash
# Clear npm cache
npm cache clean --force

# Reinstall
rm -rf node_modules package-lock.json
npm install

# If still fails, try installing vectordb explicitly
npm install vectordb@0.21.2

# Check for Apple Silicon vs Intel compatibility
uname -m  # Should show arm64 for Apple Silicon, x86_64 for Intel
```

### Issue: MCP servers not recognized by Claude Code (HISTORICAL - No Longer Applicable)

**⚠️ HISTORICAL**: This issue no longer applies as MCP servers have been removed.

**Current Approach:** If you're experiencing coordination issues, verify Knowledge Manager instead:

```bash
# Verify Knowledge Manager works
cd ~/git/cc-army
node src/knowledge-manager.js test

# Check statistics
node src/knowledge-manager.js stats

# Verify database directory
ls -la data/knowledge/
```

### Issue: Knowledge Manager "database not found"

**Symptoms:**
```
❌ Failed to initialize Knowledge Manager: database not found
```

**Solution:**
```bash
# Create database directory
mkdir -p ~/git/cc-orchestra/data/knowledge

# Run test to initialize
cd ~/git/cc-army
node src/knowledge-manager.js test

# Verify directory was created
ls -la data/knowledge/cc-army/
```

### Issue: Remote LLM connection fails

**Symptoms:**
```
❌ Request failed: connect ETIMEDOUT
```

**Solution:**
```bash
# Test connectivity
curl -v https://coder.visiquate.com

# Check if endpoint is accessible
ping coder.visiquate.com

# Verify routing configuration
node src/llm-router.js stats

# Test with fallback to Claude
# Edit config/orchestra-config.json:
# "fallbackToClaude": true
```

### Issue: Credential Manager permission denied

**Symptoms:**
```
Error: EACCES: permission denied, open '/tmp/credentials.json'
```

**Solution:**
```bash
# Create credentials file with proper permissions
touch /tmp/credentials.json
chmod 600 /tmp/credentials.json

# Test
node src/credential-manager.js list
```

### Issue: Different Node.js version on new Mac

**Symptoms:**
- Incompatibility errors
- Module loading failures

**Solution:**
```bash
# Check current version
node --version

# If too old, update via Homebrew
brew upgrade node

# If too new and causing issues, use nvm for version management
brew install nvm
nvm install 24
nvm use 24

# Reinstall dependencies
cd ~/git/cc-army
rm -rf node_modules package-lock.json
npm install
```

### Issue: Git operations fail

**Symptoms:**
- "Permission denied (publickey)" errors
- Cannot push/pull from GitHub

**Solution:**
```bash
# Restore SSH keys from backup
cp ~/Desktop/backup/.ssh/id_rsa ~/.ssh/
cp ~/Desktop/backup/.ssh/id_rsa.pub ~/.ssh/
chmod 600 ~/.ssh/id_rsa
chmod 644 ~/.ssh/id_rsa.pub

# OR generate new SSH key
ssh-keygen -t rsa -b 4096 -C "your.email@example.com"

# Add to GitHub
cat ~/.ssh/id_rsa.pub
# Copy and add to GitHub → Settings → SSH Keys

# Test connection
ssh -T git@github.com
```

### Issue: Knowledge databases corrupted after migration

**Symptoms:**
- Search returns no results
- Statistics show 0 records
- LanceDB errors

**Solution:**
```bash
# Re-extract from backup
cd ~/git/cc-army
rm -rf data/knowledge/
tar -xzf ~/Desktop/knowledge-backup.tar.gz

# Verify extraction
ls -la data/knowledge/
du -sh data/knowledge/*/

# Run test
node src/knowledge-manager.js test

# If still broken, rebuild from scratch
rm -rf data/knowledge/
node src/knowledge-manager.js test
# This creates a fresh database
```

---

## Post-Migration Verification

After completing all setup steps, run this comprehensive verification:

```bash
#!/bin/bash
echo "🔍 Claude Orchestra Migration Verification"
echo "======================================"

cd ~/git/cc-army

echo ""
echo "1️⃣ System Requirements"
echo "   Node.js: $(node --version)"
echo "   npm: $(npm --version)"
echo "   Git: $(git --version | head -1)"

echo ""
echo "2️⃣ Knowledge Manager (replaces MCP servers)"
node src/knowledge-manager.js stats | jq '.repository'

echo ""
echo "3️⃣ Claude Orchestra Installation"
npm list vectordb | grep vectordb

echo ""
echo "4️⃣ LLM Routing"
node src/llm-router.js route python-expert implement | jq .endpoint

echo ""
echo "5️⃣ Knowledge Manager"
node src/knowledge-manager.js stats | jq '.totalRecords'

echo ""
echo "6️⃣ Configuration Files"
echo "   Global CLAUDE.md: $([ -f ~/.claude/CLAUDE.md ] && echo '✅' || echo '❌')"
echo "   Army config: $([ -f config/orchestra-config.json ] && echo '✅' || echo '❌')"
echo "   Knowledge DB: $([ -d data/knowledge ] && echo '✅' || echo '❌')"

echo ""
echo "✅ Migration verification complete!"
```

Save as `verify-migration.sh`, make executable, and run:
```bash
chmod +x verify-migration.sh
./verify-migration.sh
```

---

## Summary

This guide covered:
- ✅ System requirements and dependencies
- ✅ ~~MCP server installation~~ (No longer needed - replaced by Knowledge Manager)
- ✅ Claude Orchestra repository setup
- ✅ Remote LLM configuration (coder.visiquate.com)
- ✅ Knowledge Manager with LanceDB
- ✅ File backup and restoration procedures
- ✅ Comprehensive testing checklist
- ✅ Troubleshooting common issues

**Key Success Criteria (Post-Migration):**
1. ~~MCP servers installed~~ (No longer needed)
2. Claude Orchestra repository cloned with dependencies installed
3. Remote LLM routing to coder.visiquate.com working
4. Knowledge Manager databases operational
5. Configuration files in place
6. Test checklist passes 100%

**Estimated Setup Time:**
- Fresh installation: 30-45 minutes
- With backups: 15-20 minutes
- Verification: 10 minutes

**Next Steps After Migration:**
1. Test Claude Orchestra with a sample project
2. Verify knowledge retention across sessions
3. Monitor LLM routing decisions
4. Check token savings in practice
5. Update any project-specific configurations

For additional help, refer to:
- `/Users/brent/git/cc-orchestra/docs/ARMY_USAGE_GUIDE.md`
- `/Users/brent/git/cc-orchestra/docs/KNOWLEDGE_MANAGER_GUIDE.md`
- `/Users/brent/git/cc-orchestra/docs/LLM_ROUTING_GUIDE.md`
