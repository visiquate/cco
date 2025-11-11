# CLAUDE.md - Project Template

Copy this template to your project root to customize Claude Orchestra behavior for your specific project.

```markdown
# CLAUDE.md

This file provides project-specific guidance to Claude Code when working in this repository.

## Project Overview

[Brief description of your project]

## Claude Orchestra Configuration

### Agent Preferences

**Primary Coding Agents:**
- [ ] Python Expert - [Describe when to use]
- [ ] Swift Expert - [Describe when to use]
- [ ] Go Expert - [Describe when to use]
- [ ] Rust Expert - [Describe when to use]
- [ ] Flutter Expert - [Describe when to use]

**Integration Agents:**
- [ ] API Explorer - [Describe when to use]
- [ ] Salesforce API Expert - [Describe when to use]
- [ ] Authentik API Expert - [Describe when to use]

**Support Agents (usually all needed):**
- [x] Documentation Lead - Always generate docs
- [x] QA Engineer - Always write tests
- [x] Security Auditor - Always security review
- [x] Credential Manager - Always manage credentials
- [x] DevOps Engineer - [Describe deployment needs]

### Custom Trigger Patterns

**Activate Orchestra for:**
- "Update [specific feature in your project]"
- "Add [project-specific pattern]"
- [Add your project-specific triggers]

**Bypass Orchestra for:**
- "Quick fix to [specific area]"
- "Update [specific config file]"
- [Add your project-specific bypass patterns]

### Technology Stack

**Backend:**
- Language: [Python/Go/Rust/etc.]
- Framework: [FastAPI/Gin/Actix/etc.]
- Database: [PostgreSQL/MongoDB/etc.]

**Frontend (if applicable):**
- Framework: [Flutter/React/etc.]
- State Management: [Provider/Redux/etc.]

**DevOps:**
- Containerization: [Docker/Docker Compose]
- Orchestration: [Kubernetes/ECS/etc.]
- CI/CD: [GitHub Actions/GitLab CI/etc.]
- Cloud Provider: [AWS/GCP/Azure]

**Integration APIs:**
- [ ] Salesforce (for [specific use case])
- [ ] Authentik (for [authentication/authorization])
- [ ] [Other APIs your project uses]

### Project-Specific Rules

**Security Requirements:**
- All credentials must use [specific secrets manager]
- Authentication method: [JWT/OAuth2/SAML/etc.]
- [Add security-specific rules]

**Testing Requirements:**
- Minimum test coverage: [percentage]
- Test types required: [unit/integration/e2e]
- [Add testing-specific rules]

**Documentation Requirements:**
- API docs format: [OpenAPI/Swagger/etc.]
- Code comments: [docstrings/JSDoc/etc.]
- [Add documentation-specific rules]

**Deployment Requirements:**
- Deployment target: [AWS ECS/Kubernetes/etc.]
- Environment variables: [location/format]
- Health check endpoint: [/health or custom]
- [Add deployment-specific rules]

### File Organization

```
project-root/
├── src/          # [Description]
├── tests/        # [Description]
├── docs/         # [Description]
├── config/       # [Description]
└── deploy/       # [Description]
```

### Common Commands

```bash
# Development
[List your dev commands]

# Testing
[List your test commands]

# Deployment
[List your deployment commands]

# Database
[List your database commands]
```

### API Integration Details

**Salesforce (if used):**
- Org type: [Production/Sandbox]
- API version: [v59.0/etc.]
- Objects used: [Lead/Contact/Opportunity/etc.]
- OAuth flow: [which flow you use]

**Authentik (if used):**
- Instance URL: [your instance]
- Provider type: [OAuth2/SAML/LDAP]
- Applications: [list your apps]
- Custom flows: [describe any custom flows]

**Other APIs:**
- [API name]: [integration details]

### Agent Coordination Notes

**Architect should:**
- [Project-specific architecture guidelines]
- [Design patterns to follow]
- [Technology decisions already made]

**Coding agents should:**
- [Coding standards specific to this project]
- [File naming conventions]
- [Import organization rules]

**DevOps should:**
- [Deployment procedure specifics]
- [Environment setup requirements]
- [Monitoring and logging setup]

### Known Issues & Workarounds

- [Issue 1]: [Workaround]
- [Issue 2]: [Workaround]

### External Dependencies

- [Dependency 1]: [Version and why]
- [Dependency 2]: [Version and why]

### Environment Variables

```bash
# Required
VARIABLE_NAME=description

# Optional
OPTIONAL_VAR=description
```

### Credentials Location

- Development: [where dev credentials are stored]
- Production: [where prod credentials are stored]
- Secrets manager: [AWS Secrets Manager/etc.]

### Contact & Resources

- Project Lead: [Name]
- Documentation: [URL]
- Issue Tracker: [URL]
- CI/CD Dashboard: [URL]

---

**Note:** This file is read by Claude Code and helps configure the Claude Orchestra for this specific project. The global orchestra configuration is at `/Users/brent/git/cc-orchestra/`.
```

## How to Use This Template

1. **Copy to Your Project:**
   ```bash
   cp /Users/brent/git/cc-orchestra/docs/PROJECT_CLAUDE_TEMPLATE.md /path/to/your/project/CLAUDE.md
   ```

2. **Customize Sections:**
   - Fill in project overview
   - Select which agents are needed
   - Define project-specific trigger patterns
   - Document your technology stack
   - Add project-specific rules

3. **Commit to Repository:**
   ```bash
   cd /path/to/your/project
   git add CLAUDE.md
   git commit -m "Add Claude Orchestra configuration"
   git push
   ```

4. **Start Using:**
   - Navigate to your project directory
   - Invoke Claude Code
   - The orchestra will automatically detect your project configuration
   - Orchestra operates in your project directory, not cc-orchestra

## Examples

### Python API Project

```markdown
## Technology Stack

**Backend:**
- Language: Python 3.11
- Framework: FastAPI
- Database: PostgreSQL

**DevOps:**
- Containerization: Docker + Docker Compose
- Orchestration: AWS ECS
- CI/CD: GitHub Actions

### Agent Preferences

**Primary Coding Agents:**
- [x] Python Expert - All backend development

**Support Agents:**
- [x] All support agents (Docs, QA, Security, Credentials, DevOps)
```

### Flutter Mobile App

```markdown
## Technology Stack

**Frontend:**
- Framework: Flutter 3.x
- State Management: Riverpod
- Backend: Go REST API

**Integration APIs:**
- [x] Authentik (for OAuth2 authentication)

### Agent Preferences

**Primary Coding Agents:**
- [x] Flutter Expert - Mobile app development
- [x] Go Expert - Backend API

**Integration Agents:**
- [x] Authentik API Expert - OAuth2 integration

**Support Agents:**
- [x] All support agents
```

### Salesforce Integration Project

```markdown
## Technology Stack

**Backend:**
- Language: Python 3.11
- Framework: FastAPI
- Integration: Salesforce REST API

**Integration APIs:**
- [x] Salesforce (for CRM data sync)

### Agent Preferences

**Primary Coding Agents:**
- [x] Python Expert - Integration service

**Integration Agents:**
- [x] Salesforce API Expert - Salesforce integration
- [x] API Explorer - Explore required endpoints

**Support Agents:**
- [x] All support agents
```

## Benefits of Project-Specific CLAUDE.md

1. **Consistent Agent Selection** - Always spawn the right agents for your project
2. **Custom Trigger Patterns** - Define what "complex" means for your project
3. **Technology Documentation** - Keep stack decisions documented
4. **Onboarding** - New developers (or Claude instances) understand the project faster
5. **Quality Standards** - Enforce project-specific coding and testing standards

## Notes

- The global `~/.claude/CLAUDE.md` provides orchestra auto-detection
- Project-specific `./CLAUDE.md` customizes behavior
- Both files are read by Claude Code
- Project file takes precedence for project-specific settings
- CLAUDE.md files are in `.gitignore_global` and can be committed to repos
