# Cross-Repository Usage Guide

## Overview

The Claude Orchestra can be used from **any directory** on your system. You don't need to be in the cc-army repository to deploy agents.

## Quick Start

### 1. From Any Project Directory

```bash
# Navigate to your project
cd ~/git/any-project

# Open Claude Code and describe your task
# The army automatically deploys if complexity matches trigger patterns
```

### 2. Army Auto-Detection

The army is configured in your **global CLAUDE.md** (`~/.claude/CLAUDE.md`) to automatically detect when complex tasks are requested.

**Automatic Activation Examples:**
```
✅ "Build a REST API with authentication and deploy to Docker"
✅ "Create a Flutter app with Go backend and Salesforce integration"
✅ "Set up Authentik OAuth2 for our web application"
✅ "Deploy to AWS ECS with auto-scaling and monitoring"
```

**Automatic Bypass Examples:**
```
❌ "Fix typo in README"
❌ "What does this function do?"
❌ "Run the tests"
❌ "Add a comment to the main function"
```

## How It Works

### Architecture

```
┌─────────────────────────────────────────────────────┐
│  Global CLAUDE.md (~/.claude/CLAUDE.md)             │
│  - Army auto-detection rules                        │
│  - Trigger patterns                                 │
│  - References cc-army config                        │
└─────────────────┬───────────────────────────────────┘
                  │
                  ├─> Detects complex task
                  │
┌─────────────────▼───────────────────────────────────┐
│  Army Config (/Users/brent/git/cc-army/)            │
│  - 14 agent definitions                             │
│  - Agent capabilities                               │
│  - Coordination protocols                           │
└─────────────────┬───────────────────────────────────┘
                  │
                  ├─> Spawns agents via Claude Code Task tool
                  │
┌─────────────────▼───────────────────────────────────┐
│  Your Project Directory (~/git/your-project/)       │
│  - Agents operate HERE                              │
│  - Files created HERE                               │
│  - Coordination via MCP memory                      │
└─────────────────────────────────────────────────────┘
```

### Three-Tier Configuration

1. **Global CLAUDE.md** (`~/.claude/CLAUDE.md`)
   - Auto-detection rules
   - Trigger patterns for army activation
   - References to army config location
   - Applies to ALL projects

2. **Army Config** (`/Users/brent/git/cc-army/config/orchestra-config.json`)
   - 14 agent definitions (types, models, capabilities)
   - Coordination protocols
   - Single source of truth for army

3. **Project CLAUDE.md** (optional: `~/git/your-project/CLAUDE.md`)
   - Project-specific agent preferences
   - Custom trigger patterns
   - Technology stack documentation
   - Overrides for this project only

## Trigger Patterns (Detailed)

### ✅ Activate Army For:

#### 1. Full-Stack Applications
```
"Build a [mobile/web/desktop] app with [backend]"
"Create a full-stack application"
"Develop a Flutter app with Go backend"
"Build an iOS app with Python API"
```

#### 2. Multi-Technology Projects
```
"Build with Python and Go"
"Create microservices in multiple languages"
"Implement React frontend with Rust backend"
```

#### 3. Complex Features
```
"Build authentication with OAuth2"
"Create an API with Salesforce integration"
"Implement real-time chat with websockets"
"Build data pipeline with streaming"
```

#### 4. DevOps & Deployment
```
"Deploy to AWS ECS"
"Set up Kubernetes cluster"
"Containerize with Docker and deploy"
"Create CI/CD pipeline with GitHub Actions"
```

#### 5. Enterprise Integration
```
"Integrate with Salesforce API"
"Set up Authentik authentication"
"Connect to enterprise LDAP"
"Sync data between Salesforce and database"
```

#### 6. Production-Ready Systems
```
"Build with tests, security, and monitoring"
"Create production-ready microservices"
"Implement with comprehensive documentation"
```

### ❌ Bypass Army For:

#### 1. Single-File Changes
```
"Fix typo in README.md"
"Update version in package.json"
"Add comment to main.py"
"Rename variable in utils.js"
```

#### 2. Simple Queries
```
"What does calculateTotal do?"
"Explain this code section"
"How does authentication work?"
"Show me the database schema"
```

#### 3. Basic Operations
```
"Run the tests"
"Check git status"
"List files in src/"
"View application logs"
```

#### 4. Small Additions
```
"Add a print statement"
"Update README with installation steps"
"Add error handling to one function"
"Create .gitignore file"
```

## Project-Specific Customization

### Create Project CLAUDE.md

```bash
# Copy template
cp /Users/brent/git/cc-army/docs/PROJECT_CLAUDE_TEMPLATE.md ~/git/your-project/CLAUDE.md

# Edit for your project
vim ~/git/your-project/CLAUDE.md
```

### Example: Python API Project

```markdown
# CLAUDE.md

## Claude Orchestra Configuration

### Agent Preferences

**Primary Coding Agents:**
- [x] Python Expert - All backend development
- [ ] Swift Expert - Not needed
- [ ] Go Expert - Not needed
- [ ] Rust Expert - Not needed
- [ ] Flutter Expert - Not needed

**Integration Agents:**
- [ ] API Explorer - Only if integrating new APIs
- [ ] Salesforce API Expert - Not needed
- [x] Authentik API Expert - OAuth2 authentication

**Support Agents:**
- [x] Documentation Lead - Always
- [x] QA Engineer - Always
- [x] Security Auditor - Always
- [x] Credential Manager - Always
- [x] DevOps Engineer - Docker and AWS deployment

### Custom Trigger Patterns

**Activate for:**
- "Update any API endpoint"
- "Add new authentication flow"

**Bypass for:**
- "Quick fix to config"
```

### Example: Flutter Mobile App

```markdown
# CLAUDE.md

## Claude Orchestra Configuration

### Agent Preferences

**Primary Coding Agents:**
- [ ] Python Expert - Not needed
- [ ] Swift Expert - Not needed
- [x] Go Expert - Backend API
- [ ] Rust Expert - Not needed
- [x] Flutter Expert - Mobile development

**Integration Agents:**
- [ ] API Explorer - Not needed
- [ ] Salesforce API Expert - Not needed
- [x] Authentik API Expert - Mobile OAuth2

**Support Agents:**
- [x] All support agents

### Technology Stack

**Frontend:**
- Framework: Flutter 3.x
- State Management: Riverpod

**Backend:**
- Language: Go 1.21
- Framework: Gin

**Authentication:**
- Provider: Authentik
- Flow: OAuth2 PKCE
```

## Agent Selection Logic

### Automatic Selection

Claude Code intelligently selects agents based on:

1. **Technology Keywords**
   - "Python" → Python Expert
   - "Flutter" → Flutter Expert
   - "Go" → Go Expert
   - "Rust" → Rust Expert
   - "Swift"/"iOS" → Swift Expert

2. **Integration Keywords**
   - "Salesforce" → Salesforce API Expert
   - "Authentik"/"OAuth2"/"SAML" → Authentik API Expert
   - "API integration" → API Explorer

3. **Infrastructure Keywords**
   - "Docker"/"Kubernetes"/"AWS" → DevOps Engineer
   - "Deploy"/"CI/CD" → DevOps Engineer

4. **Quality Keywords**
   - "tests"/"testing" → QA Engineer
   - "security"/"audit" → Security Auditor
   - "documentation"/"docs" → Documentation Lead

5. **Always Included**
   - Chief Architect (coordinates everything)
   - Security Auditor (security is always critical)
   - QA Engineer (quality is always critical)

### Manual Override

You can explicitly request specific agents:

```
"Use Python Expert and DevOps Engineer to containerize this application"
"Deploy Salesforce API Expert to integrate with our CRM"
"I need the full army with all 14 agents for this complex project"
```

## Coordination Protocol

### How Agents Coordinate

All agents use **Knowledge Manager** for coordination:

```bash
# Before starting work - retrieve context
node ~/git/cc-army/src/knowledge-manager.js search "architect decisions"
node ~/git/cc-army/src/knowledge-manager.js search "authentication patterns"

# During work - store progress
node ~/git/cc-army/src/knowledge-manager.js store \
  "Edit: auth.py - Implemented JWT authentication" \
  --type edit --agent python-expert

# After completing - notify completion
node ~/git/cc-army/src/knowledge-manager.js store \
  "Task complete: Authentication implemented and tested" \
  --type completion --agent python-expert
```

### Agent Communication Flow

```
1. Architect analyzes task
   └─> Stores architecture in Knowledge Manager

2. Coding agents retrieve architecture from Knowledge Manager
   ├─> Implement features
   └─> Store progress in Knowledge Manager

3. Support agents search Knowledge Manager for completed work
   ├─> QA tests completed features
   ├─> Security audits code
   ├─> Docs documents APIs
   └─> DevOps deploys when ready

4. All agents store status in Knowledge Manager
   └─> Architect reviews and approves
```

## Performance Benefits

### Speed Comparison

**Traditional Sequential Development:**
```
1. Architect designs (1 hour)
2. Developer codes (4 hours)
3. Write tests (1 hour)
4. Security review (1 hour)
5. Write docs (1 hour)
6. DevOps setup (2 hours)

Total: 10 hours
```

**Claude Orchestra Parallel Development:**
```
1. Architect designs (1 hour)
2-7. All agents work in parallel (3 hours)
   - Coding
   - Testing
   - Security
   - Docs
   - DevOps

Total: 4 hours (2.5x faster)
```

### Token Efficiency

- **32% token reduction** via shared memory
- Agents don't repeat context
- Coordination via small memory keys
- Reuse of architecture decisions

## Examples

### Example 1: Simple Python API

```bash
cd ~/git/my-api
# In Claude Code:
"Build a REST API with FastAPI and JWT authentication"

# Army deploys:
- Chief Architect
- Python Expert
- Security Auditor
- QA Engineer
- Documentation Lead
- Credential Manager

# Result: API + Tests + Security audit + Docs (30 mins)
```

### Example 2: Mobile App + Backend

```bash
cd ~/git/mobile-app
# In Claude Code:
"Create a task management app with Flutter and Go backend"

# Army deploys:
- Chief Architect
- Flutter Expert
- Go Expert
- QA Engineer
- Security Auditor
- DevOps Engineer
- Documentation Lead
- Credential Manager

# Result: Full app + Backend + Tests + Docker (2 hours)
```

### Example 3: Salesforce Integration

```bash
cd ~/git/crm-sync
# In Claude Code:
"Sync Salesforce Opportunities to PostgreSQL database"

# Army deploys:
- Chief Architect
- Salesforce API Expert
- Python Expert (integration code)
- API Explorer (explore endpoints)
- QA Engineer
- Security Auditor
- Documentation Lead
- Credential Manager

# Result: Sync service + Tests + Security + Docs (1 hour)
```

### Example 4: Enterprise Auth

```bash
cd ~/git/enterprise-portal
# In Claude Code:
"Add Authentik SAML SSO to our web application"

# Army deploys:
- Chief Architect
- Authentik API Expert
- Python/Go Expert (implementation)
- Security Auditor
- QA Engineer
- Documentation Lead
- Credential Manager

# Result: SAML integration + Tests + Security + Docs (1.5 hours)
```

## Troubleshooting

### Army Not Activating

**Problem:** Simple task being treated as complex

**Solution:**
1. Check global CLAUDE.md trigger patterns
2. Make request more specific: "Quick fix to README" instead of "Update README"
3. Bypass explicitly: "This is a simple change, don't use the army"

### Wrong Agents Selected

**Problem:** Unnecessary agents being spawned

**Solution:**
1. Create project-specific CLAUDE.md
2. Define which agents are needed for your project
3. Add custom bypass patterns

### Agents Creating Files in Wrong Location

**Problem:** Files created in cc-army instead of project

**Solution:**
1. Ensure you're running Claude Code from your project directory
2. Agents inherit cwd from Claude Code session
3. Check `pwd` in Claude Code terminal

### Knowledge Manager Not Working

**Problem:** Agents not coordinating properly

**Solution:**
1. Verify Knowledge Manager is working: `node ~/git/cc-army/src/knowledge-manager.js stats`
2. Check Knowledge Manager directory exists: `ls -la ~/.cc-army-knowledge/`
3. Test Knowledge Manager: `node ~/git/cc-army/src/knowledge-manager.js list --limit 5`

## Best Practices

### 1. Always from Project Root

```bash
# ✅ Good
cd ~/git/your-project
claude code

# ❌ Bad
cd ~/git/your-project/src
claude code  # Agents might create files in src/
```

### 2. Clear Task Description

```bash
# ✅ Good
"Build a REST API with FastAPI, JWT auth, PostgreSQL, and deploy with Docker"

# ❌ Vague
"Make an API"
```

### 3. Project-Specific CLAUDE.md

```bash
# ✅ Good - Customize per project
cp /Users/brent/git/cc-army/docs/PROJECT_CLAUDE_TEMPLATE.md ./CLAUDE.md
# Edit for your project

# ❌ Bad - Rely only on global defaults
```

### 4. Let Architect Lead

```bash
# ✅ Good
"Build [feature]" (let architect design)

# ❌ Micromanaging
"Use Python Expert to create auth.py with these exact functions..."
```

### 5. Knowledge Manager Configuration

```bash
# The Claude Orchestra uses the built-in Knowledge Manager
# No MCP servers required for coordination

# ✅ Good - Knowledge Manager is always available
# Verify it's working:
node ~/git/cc-army/src/knowledge-manager.js stats

# ❌ Bad - Don't try to use external MCP servers for army coordination
# The army is self-contained and uses Knowledge Manager
```

## Reference

- **Army Config**: `/Users/brent/git/cc-army/config/orchestra-config.json`
- **Global CLAUDE.md**: `~/.claude/CLAUDE.md`
- **Project Template**: `/Users/brent/git/cc-army/docs/PROJECT_CLAUDE_TEMPLATE.md`
- **Army Roster**: `/Users/brent/git/cc-army/ORCHESTRA_ROSTER.md`
- **Usage Guide**: `/Users/brent/git/cc-army/docs/ARMY_USAGE_GUIDE.md`
- **API Integration**: `/Users/brent/git/cc-army/docs/API_INTEGRATION_GUIDE.md`

## Support

- **Issues**: Report bugs or request features
- **Documentation**: See docs/ directory for comprehensive guides
- **Examples**: See docs/EXAMPLE_WORKFLOW.md for full workflows
