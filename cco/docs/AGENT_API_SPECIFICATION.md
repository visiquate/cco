# CCO Agent Definition API Specification

## Overview

The CCO Agent Definition API provides a centralized HTTP service for serving agent configurations to Claude Code and other consumers. This replaces the file-based approach (`~/.claude/agents/*.md`) with a more scalable, maintainable solution that can be compiled directly into the CCO binary.

## Architecture

### Data Flow
```
┌──────────────────┐
│  Claude Code     │
│  (Consumer)      │
└────────┬─────────┘
         │
         │ GET /api/agents
         ▼
┌──────────────────┐
│  CCO Server      │
│  (Provider)      │
├──────────────────┤
│ Embedded Agent   │
│ Definitions      │
│ (Compiled-in)    │
└──────────────────┘
```

### Storage Strategy
- **Compile-Time Embedding**: Agent definitions are embedded into the CCO binary at build time
- **Source Location**: Agent markdown files remain in `~/.claude/agents/` during development
- **Build Process**: `build.rs` reads and embeds agent files into the binary
- **Runtime Serving**: CCO serves embedded definitions via HTTP API

## API Endpoints

### 1. List All Agents
```http
GET /api/agents
```

**Response**: `200 OK`
```json
{
  "version": "2.0.0",
  "totalAgents": 119,
  "modelDistribution": {
    "opus": 1,
    "sonnet": 37,
    "haiku": 81
  },
  "agents": [
    {
      "name": "Chief Architect",
      "type": "chief-architect",
      "model": "opus",
      "category": "leadership",
      "description": "Strategic decision-making and project guidance",
      "capabilities": ["System design", "Architecture decisions", "Agent coordination"],
      "priority": "critical"
    },
    {
      "name": "Python Specialist",
      "type": "python-specialist",
      "model": "haiku",
      "category": "coding",
      "description": "Python development specialist for FastAPI/Flask, Django, ML/AI",
      "specialties": ["FastAPI", "Django", "Data processing", "ML/AI integration"]
    }
    // ... more agents
  ]
}
```

### 2. Get Specific Agent Configuration
```http
GET /api/agents/{agent-type}
```

**Parameters**:
- `agent-type` (string, required): The agent type identifier (e.g., `python-specialist`, `chief-architect`)

**Response**: `200 OK`
```json
{
  "name": "Python Specialist",
  "type": "python-specialist",
  "model": "haiku",
  "category": "coding",
  "description": "Python development specialist for FastAPI/Flask, Django, ML/AI",
  "metadata": {
    "version": "1.0.0",
    "lastUpdated": "2025-11-15T00:00:00Z",
    "tools": ["Read", "Write", "Edit", "Bash"],
    "frontmatter": {
      "name": "python-specialist",
      "description": "Python development specialist...",
      "tools": "Read, Write, Edit, Bash",
      "model": "haiku"
    }
  },
  "content": "You are a Python development specialist focusing on modern Python best practices...\n\n## Specialties\n- **FastAPI/Flask**: REST API development...",
  "specialties": [
    "FastAPI/Flask",
    "Django",
    "Data processing",
    "ML/AI integration",
    "Async/await patterns"
  ],
  "autonomousAuthority": {
    "lowRisk": true,
    "mediumRisk": false,
    "highRisk": false,
    "requiresArchitectApproval": true
  },
  "capabilities": [
    "REST API development",
    "Database integration",
    "Async programming",
    "Testing with pytest",
    "Data pipeline creation"
  ]
}
```

**Error Response**: `404 Not Found`
```json
{
  "error": "Agent not found",
  "type": "unknown-agent-type",
  "availableTypes": ["python-specialist", "swift-specialist", "go-specialist", ...]
}
```

### 3. Get Agent by Category
```http
GET /api/agents/category/{category}
```

**Parameters**:
- `category` (string, required): Agent category (`leadership`, `coding`, `testing`, `security`, `documentation`, `devops`, `research`)

**Response**: `200 OK`
```json
{
  "category": "coding",
  "count": 15,
  "agents": [
    {
      "name": "Python Specialist",
      "type": "python-specialist",
      "model": "haiku"
    },
    {
      "name": "Swift Specialist",
      "type": "swift-specialist",
      "model": "haiku"
    }
    // ... more agents in category
  ]
}
```

### 4. Search Agents
```http
GET /api/agents/search?q={query}
```

**Parameters**:
- `q` (string, required): Search query to match against agent names, types, descriptions, or specialties

**Response**: `200 OK`
```json
{
  "query": "python",
  "results": 3,
  "agents": [
    {
      "name": "Python Specialist",
      "type": "python-specialist",
      "model": "haiku",
      "matchedFields": ["name", "specialties"]
    }
    // ... matching agents
  ]
}
```

### 5. Get Agent Models Configuration
```http
GET /api/agents/models
```

**Response**: `200 OK`
```json
{
  "modelOverrides": {
    "claude-sonnet-4.5-20250929": "claude-haiku-4-5-20251001",
    "claude-opus-4-1-20250805": "claude-sonnet-4.5-20250929"
  },
  "agentModels": {
    "chief-architect": "opus",
    "python-specialist": "haiku",
    "test-engineer": "sonnet"
    // ... all agent type to model mappings
  },
  "statistics": {
    "totalAgents": 119,
    "byModel": {
      "opus": 1,
      "sonnet": 37,
      "haiku": 81
    }
  }
}
```

### 6. Health Check for Agent API
```http
GET /api/agents/health
```

**Response**: `200 OK`
```json
{
  "status": "healthy",
  "agentsLoaded": 119,
  "lastReload": "2025-11-15T10:00:00Z",
  "version": "2.0.0",
  "embeddedAgents": true
}
```

## Data Schema

### Agent Definition Schema (JSON)
```typescript
interface AgentDefinition {
  // Core identification
  name: string;                    // Display name
  type: string;                     // Unique type identifier
  model: "opus" | "sonnet" | "haiku"; // Model assignment
  category: AgentCategory;          // Functional category

  // Description and capabilities
  description: string;              // Brief description
  content: string;                  // Full markdown content
  capabilities: string[];           // List of capabilities
  specialties?: string[];          // Domain specialties

  // Metadata
  metadata: {
    version: string;               // Agent definition version
    lastUpdated: string;           // ISO 8601 timestamp
    tools: string[];               // Available tools
    frontmatter: Record<string, any>; // Original YAML frontmatter
  };

  // Authority and autonomy
  autonomousAuthority?: {
    lowRisk: boolean;
    mediumRisk: boolean;
    highRisk: boolean;
    requiresArchitectApproval?: boolean;
    requiresDocumentation?: boolean;
  };

  // Optional fields
  languages?: string[];            // Programming languages
  frameworks?: string[];           // Frameworks/libraries
  priority?: "critical" | "high" | "medium" | "low";
  fallback?: {
    model: string;
    triggers: string[];
    tokenThreshold?: number;
    automatic?: boolean;
  };
}

type AgentCategory =
  | "leadership"
  | "coding"
  | "testing"
  | "security"
  | "documentation"
  | "devops"
  | "research"
  | "integration"
  | "performance"
  | "support";
```

## Migration Strategy

### Phase 1: Preparation (Current State Analysis)
1. **Inventory existing agents** in `~/.claude/agents/`
2. **Parse frontmatter** from all `.md` files
3. **Map to orchestra-config.json** entries
4. **Identify missing metadata** that needs to be added

### Phase 2: Build Integration
1. **Create `build.rs` enhancements**:
   ```rust
   // build.rs
   fn embed_agent_definitions() {
       let agents_dir = Path::new("~/.claude/agents");
       let mut agents = Vec::new();

       for entry in fs::read_dir(agents_dir)? {
           let path = entry?.path();
           if path.extension() == Some("md") {
               let content = fs::read_to_string(&path)?;
               let parsed = parse_agent_markdown(&content);
               agents.push(parsed);
           }
       }

       // Generate embedded module
       let out_dir = env::var("OUT_DIR")?;
       let dest_path = Path::new(&out_dir).join("agents.rs");
       fs::write(dest_path, generate_agents_module(agents))?;
   }
   ```

2. **Include in binary**:
   ```rust
   // src/agents.rs
   include!(concat!(env!("OUT_DIR"), "/agents.rs"));
   ```

### Phase 3: API Implementation
1. **Add agent routes** to `server.rs`:
   ```rust
   // In run_server function
   .route("/api/agents", get(list_agents))
   .route("/api/agents/health", get(agents_health))
   .route("/api/agents/models", get(agent_models))
   .route("/api/agents/search", get(search_agents))
   .route("/api/agents/category/:category", get(agents_by_category))
   .route("/api/agents/:agent_type", get(get_agent))
   ```

2. **Implement handlers** using embedded data

### Phase 4: Claude Code Integration
1. **Environment variable**: `CCO_API_URL=http://localhost:3000`
2. **Fallback chain**:
   - Try CCO API first
   - Fall back to local files if API unavailable
   - Use cached version if both unavailable

### Phase 5: Testing & Validation
1. **Unit tests** for agent parsing
2. **Integration tests** for API endpoints
3. **Performance tests** for embedded data access
4. **Compatibility tests** with Claude Code

## Environment Variables

### Required Configuration
```bash
# Claude Code environment
export CCO_API_URL="http://localhost:3000"    # CCO API base URL
export CCO_API_TIMEOUT="5000"                  # API timeout in ms
export CCO_FALLBACK_TO_FILES="true"           # Use local files if API fails
export CCO_CACHE_AGENTS="true"                # Cache agent definitions locally
export CCO_CACHE_TTL="3600"                   # Cache TTL in seconds
```

### Optional Configuration
```bash
# Advanced settings
export CCO_API_KEY=""                         # API key if auth required (future)
export CCO_AGENT_DIR="~/.claude/agents"       # Local agent directory
export CCO_LOG_LEVEL="info"                   # Logging verbosity
```

## Fallback Strategy

### Priority Order
1. **Primary**: CCO API at `$CCO_API_URL/api/agents`
2. **Secondary**: Local files at `~/.claude/agents/*.md`
3. **Tertiary**: Cached definitions from previous successful fetch

### Implementation
```typescript
async function getAgentDefinition(agentType: string): Promise<AgentDefinition> {
  // Try CCO API
  if (process.env.CCO_API_URL) {
    try {
      const response = await fetch(
        `${process.env.CCO_API_URL}/api/agents/${agentType}`,
        { timeout: parseInt(process.env.CCO_API_TIMEOUT || '5000') }
      );
      if (response.ok) {
        const agent = await response.json();
        // Cache for future use
        if (process.env.CCO_CACHE_AGENTS === 'true') {
          await cacheAgent(agentType, agent);
        }
        return agent;
      }
    } catch (error) {
      console.warn(`CCO API unavailable: ${error.message}`);
    }
  }

  // Try local files
  if (process.env.CCO_FALLBACK_TO_FILES === 'true') {
    const filePath = path.join(
      process.env.CCO_AGENT_DIR || '~/.claude/agents',
      `${agentType}.md`
    );
    if (fs.existsSync(filePath)) {
      return parseAgentFile(filePath);
    }
  }

  // Try cache
  const cached = await getCachedAgent(agentType);
  if (cached) {
    console.warn(`Using cached agent definition for ${agentType}`);
    return cached;
  }

  throw new Error(`Agent definition not found: ${agentType}`);
}
```

## Security Considerations

### Access Control
- **Read-only API**: No modification endpoints
- **Rate limiting**: Prevent abuse (100 req/min per IP)
- **CORS headers**: Configured for local development

### Future Enhancements
- **Authentication**: Optional API key support
- **Encryption**: TLS for production deployments
- **Audit logging**: Track agent definition access

## Performance Optimization

### Caching Strategy
- **In-memory cache**: All agents loaded at startup
- **Lazy loading**: Individual agent content loaded on demand
- **Compression**: Gzip compression for large responses

### Resource Usage
- **Memory footprint**: ~10MB for 119 agents
- **Response time**: <10ms for cached responses
- **Startup time**: <100ms to load all agents

## Monitoring & Metrics

### Exposed Metrics
- `cco_agent_api_requests_total`: Total API requests
- `cco_agent_api_errors_total`: API errors by type
- `cco_agent_cache_hits_total`: Cache hit rate
- `cco_agent_response_time_ms`: Response time histogram

### Health Indicators
- Agent count validation
- Memory usage monitoring
- API response time tracking
- Error rate thresholds

## Future Enhancements

### Version 2.1
- **Hot reload**: Reload agent definitions without restart
- **Agent versioning**: Support multiple versions per agent
- **Custom agents**: User-defined agent additions

### Version 2.2
- **Agent marketplace**: Share custom agents
- **Agent analytics**: Usage tracking and optimization
- **Agent testing**: Built-in agent validation

### Version 3.0
- **Agent composition**: Combine multiple agents
- **Agent templates**: Parameterized agent definitions
- **Agent inheritance**: Extend base agent definitions