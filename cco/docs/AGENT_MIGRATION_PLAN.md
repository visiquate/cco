# Agent Definition Migration Plan

## Current State → Target State

### Current State
- **119 agent markdown files** in `~/.claude/agents/`
- **orchestra-config.json** with partial agent metadata
- **File-based loading** by Claude Code
- **Manual synchronization** between files and config

### Target State
- **Embedded agent definitions** in CCO binary
- **HTTP API** serving agent configurations
- **Single source of truth** for agent definitions
- **Automatic synchronization** via build process

## Migration Phases

### Phase 1: Data Preparation (Week 1)

#### 1.1 Audit Existing Agents
```bash
#!/bin/bash
# Inventory all agent files
find ~/.claude/agents -name "*.md" | sort > agent_inventory.txt

# Extract frontmatter from all agents
for file in ~/.claude/agents/*.md; do
  echo "Processing: $file"
  head -n 20 "$file" | grep -E "^(name|model|description|tools):"
done > agent_metadata.csv
```

#### 1.2 Create Agent Registry
```rust
// src/agents/registry.rs
pub struct AgentRegistry {
    agents: HashMap<String, AgentDefinition>,
    categories: HashMap<String, Vec<String>>,
    model_mapping: HashMap<String, String>,
}

impl AgentRegistry {
    pub fn from_embedded() -> Self {
        // Load from compiled-in data
    }
}
```

#### 1.3 Standardize Agent Metadata
- Add missing `category` fields
- Standardize `model` values (opus/sonnet/haiku)
- Ensure consistent `type` identifiers
- Add `priority` levels where appropriate

### Phase 2: Build System Integration (Week 1-2)

#### 2.1 Enhance build.rs
```rust
// build.rs additions
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn parse_agent_frontmatter(content: &str) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();

    // Find frontmatter boundaries
    if lines[0] == "---" {
        for i in 1..lines.len() {
            if lines[i] == "---" {
                break;
            }
            if let Some((key, value)) = lines[i].split_once(": ") {
                metadata.insert(key.to_string(), value.to_string());
            }
        }
    }
    metadata
}

fn generate_embedded_agents() -> Result<(), Box<dyn std::error::Error>> {
    let agents_dir = dirs::home_dir()
        .unwrap()
        .join(".claude")
        .join("agents");

    let mut agents = Vec::new();

    for entry in fs::read_dir(&agents_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "md") {
            let content = fs::read_to_string(&path)?;
            let metadata = parse_agent_frontmatter(&content);

            agents.push(EmbeddedAgent {
                type_id: path.file_stem().unwrap().to_str().unwrap().to_string(),
                content,
                metadata,
            });
        }
    }

    // Generate Rust code
    let out_dir = std::env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("embedded_agents.rs");

    let code = generate_rust_code(&agents);
    fs::write(dest_path, code)?;

    Ok(())
}
```

#### 2.2 Create Agent Module
```rust
// src/agents/mod.rs
mod registry;
mod embedded;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Include generated code
include!(concat!(env!("OUT_DIR"), "/embedded_agents.rs"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub r#type: String,
    pub model: String,
    pub category: String,
    pub description: String,
    pub content: String,
    pub metadata: AgentMetadata,
    pub specialties: Vec<String>,
    pub autonomous_authority: AutonomousAuthority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub version: String,
    pub last_updated: String,
    pub tools: Vec<String>,
    pub frontmatter: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousAuthority {
    pub low_risk: bool,
    pub medium_risk: bool,
    pub high_risk: bool,
    pub requires_architect_approval: bool,
}

pub fn get_all_agents() -> &'static [AgentDefinition] {
    &EMBEDDED_AGENTS
}

pub fn get_agent(type_id: &str) -> Option<&'static AgentDefinition> {
    EMBEDDED_AGENTS.iter().find(|a| a.r#type == type_id)
}
```

### Phase 3: API Implementation (Week 2)

#### 3.1 Add API Routes
```rust
// src/server.rs additions

use crate::agents::{get_all_agents, get_agent, AgentDefinition};

// List all agents
async fn list_agents() -> Json<serde_json::Value> {
    let agents = get_all_agents();

    let model_dist = agents.iter().fold(
        HashMap::new(),
        |mut acc, agent| {
            *acc.entry(&agent.model).or_insert(0) += 1;
            acc
        }
    );

    Json(json!({
        "version": "2.0.0",
        "totalAgents": agents.len(),
        "modelDistribution": model_dist,
        "agents": agents
    }))
}

// Get specific agent
async fn get_agent_handler(Path(agent_type): Path<String>) -> Result<Json<AgentDefinition>, StatusCode> {
    match get_agent(&agent_type) {
        Some(agent) => Ok(Json(agent.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// Search agents
async fn search_agents(Query(params): Query<SearchParams>) -> Json<serde_json::Value> {
    let query = params.q.to_lowercase();
    let results: Vec<_> = get_all_agents()
        .iter()
        .filter(|agent| {
            agent.name.to_lowercase().contains(&query) ||
            agent.r#type.contains(&query) ||
            agent.description.to_lowercase().contains(&query) ||
            agent.specialties.iter().any(|s| s.to_lowercase().contains(&query))
        })
        .collect();

    Json(json!({
        "query": params.q,
        "results": results.len(),
        "agents": results
    }))
}

// Get agents by category
async fn agents_by_category(Path(category): Path<String>) -> Json<serde_json::Value> {
    let agents: Vec<_> = get_all_agents()
        .iter()
        .filter(|agent| agent.category == category)
        .collect();

    Json(json!({
        "category": category,
        "count": agents.len(),
        "agents": agents
    }))
}
```

#### 3.2 Update Router
```rust
// In run_server function
let app = Router::new()
    // ... existing routes ...
    // Agent API routes
    .route("/api/agents", get(list_agents))
    .route("/api/agents/health", get(agents_health))
    .route("/api/agents/models", get(agent_models_config))
    .route("/api/agents/search", get(search_agents))
    .route("/api/agents/category/:category", get(agents_by_category))
    .route("/api/agents/:agent_type", get(get_agent_handler))
    // ... rest of configuration
```

### Phase 4: Claude Code Integration (Week 2-3)

#### 4.1 Create API Client
```javascript
// claude-code-integration.js
class CCOAgentClient {
    constructor(apiUrl = process.env.CCO_API_URL || 'http://localhost:3000') {
        this.apiUrl = apiUrl;
        this.cache = new Map();
        this.cacheTimeout = parseInt(process.env.CCO_CACHE_TTL || '3600') * 1000;
    }

    async getAgent(agentType) {
        // Check cache first
        if (this.cache.has(agentType)) {
            const cached = this.cache.get(agentType);
            if (Date.now() - cached.timestamp < this.cacheTimeout) {
                return cached.data;
            }
        }

        try {
            // Try CCO API
            const response = await fetch(`${this.apiUrl}/api/agents/${agentType}`);
            if (response.ok) {
                const agent = await response.json();
                this.cache.set(agentType, {
                    data: agent,
                    timestamp: Date.now()
                });
                return agent;
            }
        } catch (error) {
            console.warn(`CCO API error: ${error.message}`);
        }

        // Fallback to local files
        return this.loadLocalAgent(agentType);
    }

    async loadLocalAgent(agentType) {
        const agentPath = path.join(
            os.homedir(),
            '.claude',
            'agents',
            `${agentType}.md`
        );

        if (fs.existsSync(agentPath)) {
            const content = fs.readFileSync(agentPath, 'utf8');
            return this.parseAgentFile(content, agentType);
        }

        throw new Error(`Agent not found: ${agentType}`);
    }

    parseAgentFile(content, agentType) {
        // Parse frontmatter and content
        const lines = content.split('\n');
        const metadata = {};
        let contentStart = 0;

        if (lines[0] === '---') {
            for (let i = 1; i < lines.length; i++) {
                if (lines[i] === '---') {
                    contentStart = i + 1;
                    break;
                }
                const [key, ...valueParts] = lines[i].split(': ');
                if (key) {
                    metadata[key] = valueParts.join(': ');
                }
            }
        }

        return {
            type: agentType,
            name: metadata.name || agentType,
            model: metadata.model || 'haiku',
            description: metadata.description || '',
            content: lines.slice(contentStart).join('\n'),
            metadata: {
                frontmatter: metadata,
                tools: (metadata.tools || '').split(', '),
            }
        };
    }
}

// Export for use in Claude Code
module.exports = CCOAgentClient;
```

#### 4.2 Environment Setup Script
```bash
#!/bin/bash
# setup-cco-integration.sh

# Check if CCO is running
if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "Starting CCO server..."
    cco run --port 3000 &
    sleep 2
fi

# Set environment variables
export CCO_API_URL="http://localhost:3000"
export CCO_API_TIMEOUT="5000"
export CCO_FALLBACK_TO_FILES="true"
export CCO_CACHE_AGENTS="true"
export CCO_CACHE_TTL="3600"

echo "CCO integration configured:"
echo "  API URL: $CCO_API_URL"
echo "  Fallback enabled: $CCO_FALLBACK_TO_FILES"
echo "  Cache enabled: $CCO_CACHE_AGENTS"
```

### Phase 5: Testing & Validation (Week 3)

#### 5.1 Unit Tests
```rust
// tests/agent_api_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_loading() {
        let agents = get_all_agents();
        assert_eq!(agents.len(), 119);
    }

    #[test]
    fn test_agent_retrieval() {
        let agent = get_agent("python-specialist");
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().model, "haiku");
    }

    #[test]
    fn test_model_distribution() {
        let agents = get_all_agents();
        let opus_count = agents.iter().filter(|a| a.model == "opus").count();
        let sonnet_count = agents.iter().filter(|a| a.model == "sonnet").count();
        let haiku_count = agents.iter().filter(|a| a.model == "haiku").count();

        assert_eq!(opus_count, 1);
        assert_eq!(sonnet_count, 37);
        assert_eq!(haiku_count, 81);
    }
}
```

#### 5.2 Integration Tests
```rust
// tests/integration_tests.rs
#[tokio::test]
async fn test_api_endpoints() {
    let app = create_test_app();

    // Test list agents
    let response = app
        .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test get specific agent
    let response = app
        .oneshot(Request::builder().uri("/api/agents/python-specialist").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test search
    let response = app
        .oneshot(Request::builder().uri("/api/agents/search?q=python").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

#### 5.3 Performance Tests
```rust
// benches/agent_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_agent_loading(c: &mut Criterion) {
    c.bench_function("load all agents", |b| {
        b.iter(|| get_all_agents())
    });
}

fn bench_agent_search(c: &mut Criterion) {
    c.bench_function("search agents", |b| {
        b.iter(|| {
            let agents = get_all_agents();
            agents.iter().filter(|a| a.name.contains("Python")).collect::<Vec<_>>()
        })
    });
}

criterion_group!(benches, bench_agent_loading, bench_agent_search);
criterion_main!(benches);
```

### Phase 6: Deployment (Week 3-4)

#### 6.1 Rollout Strategy
1. **Alpha Testing**: Deploy to development environment
2. **Beta Testing**: Limited rollout to select users
3. **Canary Deployment**: Gradual rollout with monitoring
4. **Full Deployment**: Complete migration

#### 6.2 Monitoring
```rust
// Add metrics to server.rs
static AGENT_API_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "cco_agent_api_requests_total",
        "Total agent API requests",
        &["endpoint", "status"]
    ).unwrap()
});

static AGENT_API_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "cco_agent_api_latency_seconds",
        "Agent API latency",
        &["endpoint"]
    ).unwrap()
});
```

#### 6.3 Rollback Plan
```bash
#!/bin/bash
# rollback.sh

# Revert to file-based agents
unset CCO_API_URL
export CCO_FALLBACK_TO_FILES="true"

echo "Rolled back to file-based agent loading"
echo "Agent files: ~/.claude/agents/"
```

## Timeline

| Week | Phase | Tasks | Deliverables |
|------|-------|-------|-------------|
| 1 | Data Prep | Audit agents, create registry | Agent inventory, metadata spreadsheet |
| 1-2 | Build Integration | Embed agents in binary | Updated build.rs, agent module |
| 2 | API Implementation | Create endpoints | Working API routes |
| 2-3 | Claude Code Integration | Client library | JS client, env setup |
| 3 | Testing | Unit, integration, performance | Test suite, benchmarks |
| 3-4 | Deployment | Rollout, monitoring | Production deployment |

## Success Metrics

### Technical Metrics
- **API response time**: < 10ms for cached responses
- **Memory usage**: < 20MB for agent data
- **Build time impact**: < 5s increase
- **Cache hit rate**: > 90%

### Business Metrics
- **Agent availability**: 99.9% uptime
- **Migration success**: 100% agents migrated
- **User adoption**: 80% using API within 1 month
- **Error rate**: < 0.1% API errors

## Risk Mitigation

### Identified Risks
1. **Build time increase**: Mitigate with incremental compilation
2. **Binary size increase**: Use compression, lazy loading
3. **API availability**: Implement robust fallback chain
4. **Breaking changes**: Version API, maintain compatibility

### Contingency Plans
- **Fallback to files**: Always available as backup
- **Cache persistence**: Store agent cache locally
- **Gradual migration**: Support both methods initially
- **Monitoring alerts**: Immediate notification of issues

## Post-Migration Cleanup

### Week 5+ Tasks
1. **Deprecate file-based loading** (after 30 days)
2. **Remove orchestra-config.json agent entries** (redundant)
3. **Archive original agent files** (for reference)
4. **Update documentation** (reflect new architecture)
5. **Performance optimization** (based on metrics)

## Documentation Updates

### Required Updates
- README.md: New agent API documentation
- ORCHESTRATOR_RULES.md: Update agent invocation
- QUICK_START.md: New setup instructions
- API_DOCUMENTATION.md: Complete API reference

### New Documentation
- AGENT_API_SPECIFICATION.md (this document)
- AGENT_MIGRATION_PLAN.md
- AGENT_CLIENT_GUIDE.md
- AGENT_TROUBLESHOOTING.md