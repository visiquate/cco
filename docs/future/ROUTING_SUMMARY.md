# Claude Orchestra - Complete Routing Summary

**STATUS: PLANNED - FUTURE HYBRID ROUTING (NOT CURRENTLY IMPLEMENTED)**

This document describes how the 14 agents **would route** to either Claude API or the custom Ollama endpoint at coder.visiquate.com **when ccproxy is deployed**. Currently, all agents use Claude API directly.

## Routing by Agent Type

### 🎯 Routes to CLAUDE API (6 agents)

#### 1. **Chief Architect** (system-architect)
- **Routes to**: Claude API (Opus 4.1)
- **Reason**: Architecture and planning tasks use Claude
- **Responsibilities**:
  - Strategic decision-making
  - Architecture design
  - Technology stack selection
  - Agent coordination
  - Compaction management

#### 2. **API Explorer** (researcher)
- **Routes to**: Claude API (Sonnet 4.5)
- **Reason**: Default routing to Claude (analytical reasoning)
- **Responsibilities**:
  - Explore and document third-party APIs
  - Test API endpoints and authentication
  - Analyze API capabilities
  - Create integration POCs

#### 3. **QA Engineer** (test-automator)
- **Routes to**: Claude API (Sonnet 4.5)
- **Reason**: Default routing to Claude
- **Responsibilities**:
  - Integration testing
  - End-to-end testing
  - Test automation
  - Autonomous test fixing
  - Quality assurance

#### 4. **Security Auditor** (security-auditor)
- **Routes to**: Claude API (Sonnet 4.5)
- **Reason**: Default routing to Claude
- **Responsibilities**:
  - Code security review
  - Vulnerability scanning
  - OWASP compliance
  - Authentication/authorization review
  - Dependency audits

#### 5. **DevOps Engineer** (deployment-engineer)
- **Routes to**: Claude API (Sonnet 4.5)
- **Reason**: Default routing to Claude
- **Responsibilities**:
  - Docker and Kubernetes configuration
  - CI/CD pipeline setup
  - Infrastructure as Code
  - AWS infrastructure
  - Monitoring and logging

#### 6. **API Explorer** (researcher)
- **Routes to**: Claude API (Sonnet 4.5)
- **Reason**: Default routing (research and analysis)
- **Responsibilities**:
  - API exploration and documentation
  - Testing endpoints
  - Integration analysis

---

### 💻 Routes to CUSTOM LLM (coder.visiquate.com) (8 agents)

#### 7. **Python Specialist** (python-expert)
- **Routes to**: Ollama - qwen2.5-coder:32b-instruct
- **Reason**: Coding tasks routed to custom LLM
- **Specialties**: FastAPI/Flask, Django, Data processing, ML/AI integration, Async/await
- **Implementation Model**: Qwen 2.5 Coder (32B parameters)

#### 8. **Swift/iOS Specialist** (ios-developer)
- **Routes to**: Ollama - qwen2.5-coder:32b-instruct
- **Reason**: Coding tasks routed to custom LLM
- **Specialties**: SwiftUI, UIKit, Core Data, Combine, iOS architecture
- **Implementation Model**: Qwen 2.5 Coder (32B parameters)

#### 9. **Go Specialist** (backend-dev)
- **Routes to**: Ollama - qwen2.5-coder:32b-instruct
- **Reason**: Coding tasks routed to custom LLM
- **Specialties**: Microservices, Concurrency, gRPC, Performance optimization
- **Implementation Model**: Qwen 2.5 Coder (32B parameters)

#### 10. **Rust Specialist** (backend-dev)
- **Routes to**: Ollama - qwen2.5-coder:32b-instruct
- **Reason**: Coding tasks routed to custom LLM
- **Specialties**: Systems programming, Memory safety, WebAssembly, Async Rust
- **Implementation Model**: Qwen 2.5 Coder (32B parameters)

#### 11. **Flutter Specialist** (mobile-developer)
- **Routes to**: Ollama - qwen2.5-coder:32b-instruct
- **Reason**: Coding tasks routed to custom LLM
- **Specialties**: Cross-platform mobile, State management, Native integrations
- **Implementation Model**: Qwen 2.5 Coder (32B parameters)

#### 12. **Salesforce API Specialist** (backend-dev)
- **Routes to**: Ollama - qwen2.5-coder:32b-instruct
- **Reason**: Coding tasks routed to custom LLM
- **Specialties**: Salesforce REST/SOAP API, SOQL, Bulk API, OAuth 2.0
- **Implementation Model**: Qwen 2.5 Coder (32B parameters)

#### 13. **Authentik API Specialist** (backend-dev)
- **Routes to**: Ollama - qwen2.5-coder:32b-instruct
- **Reason**: Coding tasks routed to custom LLM
- **Specialties**: OAuth2/OIDC, SAML, User provisioning, MFA
- **Implementation Model**: Qwen 2.5 Coder (32B parameters)

#### 14. **Documentation Lead** (coder)
- **Routes to**: Ollama - qwen2.5-coder:32b-instruct
- **Reason**: Coding tasks routed to custom LLM (type='coder')
- **Note**: Uses Haiku model designation but routes to Ollama due to 'coder' type
- **Responsibilities**: README files, API docs, Code comments, Architecture diagrams

⚠️ **Note**: Documentation Lead might benefit from Claude's reasoning for better technical writing. Consider adjusting routing rules if needed.

---

## Routing Statistics

### By Destination:
- **Claude API**: 6 agents (42.9%)
  - 1 Architect (Opus)
  - 5 Support/Analysis agents (Sonnet/Haiku)

- **Custom LLM (Ollama)**: 8 agents (57.1%)
  - 5 Language specialists
  - 2 API integration specialists
  - 1 Documentation specialist

### By Category:
| Category | Claude | Ollama | Total |
|----------|--------|--------|-------|
| **Leadership** | 1 | 0 | 1 |
| **Coding Specialists** | 0 | 5 | 5 |
| **Integration Specialists** | 1 | 2 | 3 |
| **Support Agents** | 4 | 1 | 5 |
| **TOTAL** | **6** | **8** | **14** |

## Routing Logic

### Agent Types that Route to Custom LLM:
```javascript
const codingTypes = [
  'python-expert',      // Python Specialist
  'ios-developer',      // Swift Specialist
  'backend-dev',        // Go, Rust, Salesforce, Authentik specialists
  'mobile-developer',   // Flutter Specialist
  'coder',              // Documentation Lead, Credential Manager
  'frontend-dev'        // (not currently used)
];
```

### Agent Types that Route to Claude:
```javascript
const architectureTypes = [
  'system-architect',   // Chief Architect
  'architecture',       // (not currently used)
  'specification',      // (not currently used)
  'planner'            // (not currently used)
];

// All other types default to Claude:
const otherTypes = [
  'researcher',         // API Explorer
  'test-automator',     // QA Engineer
  'security-auditor',   // Security Auditor
  'deployment-engineer' // DevOps Engineer
];
```

### Task Keywords that Influence Routing:

**Architecture/Planning Keywords** (→ Claude):
- design, architecture, planning
- specification, requirements
- coordination

**Coding Keywords** (→ Custom LLM):
- implement, code, develop
- build, write code, programming

## Cost and Performance Implications

### Claude API Costs (Approximate):
- **Opus**: $15/1M input tokens, $75/1M output tokens
- **Sonnet 4.5**: $3/1M input tokens, $15/1M output tokens
- **Haiku**: $0.25/1M input tokens, $1.25/1M output tokens

### Custom Ollama (coder.visiquate.com):
- **Cost**: $0 (self-hosted)
- **Model**: Qwen 2.5 Coder 32B (Q4_K_M quantization)
- **Performance**: ~14-75 seconds per response (depending on complexity)
- **Capacity**: Limited by server resources

### Estimated Cost Savings:
- **Before routing**: All 14 agents on Claude = ~100% Claude costs
- **After routing**: 8 agents on Ollama = ~57% cost reduction for coding tasks
- **Remaining Claude usage**: Architecture, QA, Security, DevOps (high-value tasks)

## Testing Routing Decisions

### Test Individual Agent:
```bash
node src/llm-router.js route <agent-type> <task-type>

# Examples:
node src/llm-router.js route python-expert implement
node src/llm-router.js route system-architect planning
node src/llm-router.js route ios-developer code
```

### Test All Agents:
```bash
# Run the comprehensive test
/tmp/test-all-routing.sh
```

### Make Live Coding Request:
```bash
node src/llm-router.js call-coding-llm "Your coding prompt here"
```

## Recommendations

### Current Configuration is Optimal For:
✅ **Architecture & Planning** - Claude's strategic reasoning
✅ **Security Reviews** - Claude's comprehensive analysis
✅ **QA & Testing Strategy** - Claude's analytical approach
✅ **DevOps & Infrastructure** - Claude's system-level thinking
✅ **Code Implementation** - Ollama's specialized coding model

### Consider Adjusting If:
- **Documentation Lead**: May benefit from Claude (Haiku) for better technical writing
- **Credential Manager**: Currently routes to Ollama, might be better on Claude for security reasoning
- **API Integration Specialists**: Currently on Ollama, evaluate if Claude's reasoning would help with complex integrations

### To Adjust Routing:
Edit `src/llm-router.js` and modify the agent type arrays:
```javascript
// Move 'coder' type from coding tasks to default (Claude)
const codingTypes = [
  'python-expert',
  'ios-developer',
  'backend-dev',
  'mobile-developer',
  // 'coder',  // Removed - will now route to Claude
  'frontend-dev'
];
```

## Version History
- **v1.0** (2025-01-16): Initial routing implementation
  - 6 agents → Claude API
  - 8 agents → Ollama (coder.visiquate.com)
  - Qwen 2.5 Coder 32B model
