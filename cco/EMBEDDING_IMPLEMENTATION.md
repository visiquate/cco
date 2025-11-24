# Implementation Details: Compile-Time Agent Embedding

Technical deep-dive into how CCO implements compile-time agent definition embedding.

## Architecture Components

### 1. build.rs - Build Script

**Location:** `cco/build.rs`

**Purpose:** Runs during compilation to prepare embedded data

**Key Functions:**

```rust
fn main() {
    // Tells Cargo to recompile if these files change
    println!("cargo:rerun-if-changed=../config/");
    println!("cargo:rerun-if-changed=../config/orchestra-config.json");

    // Get git commit
    let git_hash = get_git_hash();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Get build date
    let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Get/set version
    let version = env::var("CCO_VERSION").unwrap_or_else(|_| "2025.11.2".to_string());
    println!("cargo:rustc-env=CCO_VERSION={}", version);

    // Validate configs
    validate_configs();
}

fn get_git_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}

fn validate_configs() {
    let config_paths = vec!["../config/orchestra-config.json"];

    for config_file in config_paths {
        let path = Path::new(config_file);
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Parse JSON to validate syntax
                    if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                        eprintln!("Invalid JSON in {}: {}", config_file, e);
                        panic!("Config validation failed");
                    }
                    println!("cargo:warning=Validated config: {}", config_file);
                }
                Err(e) => {
                    eprintln!("Failed to read {}: {}", config_file, e);
                    panic!("Config file read failed");
                }
            }
        }
    }
}
```

**Environment Variables Set:**

- `GIT_HASH` - Current git commit (short form)
- `BUILD_DATE` - Compilation timestamp
- `CCO_VERSION` - Release version
- `RUST_LINK_ARG=-fPIC` - Position-independent code

### 2. agents_config.rs - Agent Configuration

**Location:** `cco/src/agents_config.rs`

**Purpose:** Defines structures and loading logic for agents

**Key Structures:**

```rust
/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Agent {
    pub name: String,              // "chief-architect"
    pub model: String,             // "opus", "sonnet", "haiku"
    pub description: String,       // Agent description
    pub tools: Vec<String>,        // ["Read", "Write", "Edit"]
}

/// Container for all agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub agents: HashMap<String, Agent>,
}

impl AgentsConfig {
    pub fn new() -> Self {
        Self { agents: HashMap::new() }
    }

    pub fn get(&self, name: &str) -> Option<&Agent> {
        self.agents.get(name)
    }

    pub fn all(&self) -> Vec<Agent> {
        self.agents.values().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}
```

**YAML Frontmatter Parser:**

```rust
/// Simple YAML frontmatter parser
#[derive(Debug, Clone)]
struct FrontmatterData {
    name: Option<String>,
    model: Option<String>,
    description: Option<String>,
    tools: Option<String>,  // Comma-separated
}

/// Parse YAML frontmatter from markdown
fn parse_frontmatter(content: &str) -> Option<FrontmatterData> {
    // Check for opening --- marker
    if !content.starts_with("---") {
        return None;
    }

    // Find closing --- marker
    let rest = &content[3..];
    let closing_pos = rest.find("---")?;
    let yaml_content = &rest[..closing_pos];

    // Parse key: value pairs
    let mut data = FrontmatterData {
        name: None,
        model: None,
        description: None,
        tools: None,
    };

    for line in yaml_content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key: value
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let mut value = line[colon_pos + 1..].trim();

            // Remove quotes if present
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = &value[1..value.len() - 1];
            }

            match key {
                "name" => data.name = Some(value.to_string()),
                "model" => data.model = Some(value.to_string()),
                "description" => data.description = Some(value.to_string()),
                "tools" => data.tools = Some(value.to_string()),
                _ => {}
            }
        }
    }

    Some(data)
}
```

**Agent Loading:**

```rust
/// Load all agents from ~/.claude/agents/
pub fn load_agents() -> AgentsConfig {
    let mut config = AgentsConfig::new();

    // Get agents directory
    let agents_dir = match dirs::home_dir() {
        Some(home) => home.join(".claude").join("agents"),
        None => {
            warn!("Could not determine home directory");
            return config;
        }
    };

    // Check directory exists
    if !agents_dir.exists() {
        warn!("Agents directory not found: {:?}", agents_dir);
        return config;
    }

    // Read directory
    let entries = match fs::read_dir(&agents_dir) {
        Ok(e) => e,
        Err(e) => {
            warn!("Failed to read agents directory: {}", e);
            return config;
        }
    };

    // Load each .md file
    let mut loaded_count = 0;
    let mut error_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process .md files
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        if let Some(agent) = load_agent_from_file(&path) {
            config.agents.insert(agent.name.clone(), agent);
            loaded_count += 1;
        } else {
            error_count += 1;
            warn!("Failed to load agent from: {:?}", path);
        }
    }

    info!(
        "✓ Loaded {} agents from {:?} ({} errors)",
        loaded_count, agents_dir, error_count
    );

    config
}

/// Load single agent from file
fn load_agent_from_file(path: &PathBuf) -> Option<Agent> {
    let content = fs::read_to_string(path).ok()?;
    let frontmatter = parse_frontmatter(&content)?;

    let name = frontmatter.name?;
    let model = frontmatter.model?;
    let description = frontmatter.description?;

    let tools = if let Some(tools_str) = frontmatter.tools {
        tools_str
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    Some(Agent {
        name,
        model,
        description,
        tools,
    })
}
```

### 3. server.rs - HTTP API

**Location:** `cco/src/server.rs`

**Purpose:** HTTP endpoints for agent definitions

**Server State:**

```rust
#[derive(Clone)]
pub struct ServerState {
    pub cache: MokaCache,
    pub router: ModelRouter,
    pub analytics: Arc<AnalyticsEngine>,
    pub proxy: Arc<ProxyServer>,
    pub start_time: Instant,
    pub model_overrides: Arc<HashMap<String, String>>,
    pub agent_models: Arc<HashMap<String, String>>,
    pub agents: Arc<AgentsConfig>,  // <-- Embedded agents
}
```

**Agent Loading at Startup:**

```rust
pub async fn run_server(
    host: &str,
    port: u16,
    cache_size: u64,
    cache_ttl: u64,
) -> anyhow::Result<()> {
    // ... other initialization ...

    // Load agents from ~/.claude/agents/
    let agents = Arc::new(load_agents());

    let state = Arc::new(ServerState {
        cache,
        router,
        analytics,
        proxy,
        start_time,
        model_overrides,
        agent_models,
        agents,  // Loaded into state
    });

    // Build router with state
    let app = Router::new()
        .route("/api/agents", get(list_agents))
        .route("/api/agents/:agent_name", get(get_agent))
        // ... other routes ...
        .with_state(state);

    // ... rest of server setup ...
}
```

**Agent Endpoints:**

```rust
/// List all agents
async fn list_agents(State(state): State<Arc<ServerState>>) -> Json<AgentsListResponse> {
    let agents = state.agents.all();
    info!("Returning {} agents", agents.len());
    Json(AgentsListResponse { agents })
}

/// Get specific agent by name
async fn get_agent(
    State(state): State<Arc<ServerState>>,
    AxumPath(agent_name): AxumPath<String>,
) -> Result<Json<Agent>, (StatusCode, Json<AgentNotFoundResponse>)> {
    match state.agents.get(&agent_name) {
        Some(agent) => {
            info!("Agent found: {}", agent_name);
            Ok(Json(agent.clone()))
        }
        None => {
            info!("Agent not found: {}", agent_name);
            Err((
                StatusCode::NOT_FOUND,
                Json(AgentNotFoundResponse {
                    error: format!("Agent not found: {}", agent_name),
                }),
            ))
        }
    }
}
```

## Data Flow

### Build Time

```
1. Developer modifies cco/config/agents/chief-architect.md
                    │
                    ▼
2. Run: cargo build --release
                    │
                    ▼
3. Cargo invokes build.rs
                    │
                    ├─ Watches for changes
                    ├─ Validates JSON configs
                    ├─ Extracts git hash
                    ├─ Sets version
                    └─ Compiles
                    │
                    ▼
4. Rust compiler compiles source code
                    │
                    ├─ agents_config.rs loaded
                    ├─ YAML parser ready
                    ├─ Agent structures defined
                    └─ server.rs prepared
                    │
                    ▼
5. Linker creates binary
                    │
                    ├─ All code compiled in
                    ├─ Version set via env var
                    └─ No external data files
                    │
                    ▼
6. Binary ready: target/release/cco
```

### Runtime

```
1. User runs: ./cco run --port 3000
                    │
                    ▼
2. Binary loaded into memory
                    │
                    ├─ All compiled code
                    ├─ Version hardcoded
                    └─ Ready to serve
                    │
                    ▼
3. server.rs::run_server() called
                    │
                    ├─ Initialize components
                    ├─ Call load_agents()
                    └─ Create ServerState
                    │
                    ▼
4. load_agents() executes
                    │
                    ├─ Read ~/.claude/agents/*.md
                    ├─ Parse YAML frontmatter
                    ├─ Create Agent structs
                    └─ Store in HashMap
                    │
                    ▼
5. ServerState created with AgentsConfig
                    │
                    ├─ agents HashMap in memory
                    ├─ Ready for lookups
                    └─ O(1) access time
                    │
                    ▼
6. HTTP server starts
                    │
                    ├─ Listen on 127.0.0.1:3000
                    ├─ Register routes
                    └─ Ready for requests
                    │
                    ▼
7. Client requests: GET /api/agents/chief-architect
                    │
                    ├─ Route matches
                    ├─ Handler calls state.agents.get()
                    ├─ HashMap lookup O(1)
                    └─ Returns Agent from memory
                    │
                    ▼
8. Response: Agent definition (instant)
```

## Memory Layout

### Build-Time Data

Agent definitions are not explicitly stored in binary. Instead:

1. **Source Code** - agent loading logic in `agents_config.rs`
2. **Config Files** - `cco/config/orchestra-config.json` validated
3. **Environment Variables** - version, git hash, build date compiled in

### Runtime Memory

```
┌─────────────────────────────────────────┐
│         Loaded Binary (~15-20MB)        │
├─────────────────────────────────────────┤
│ Compiled Code:                          │
│  - main.rs logic                        │
│  - server.rs HTTP handlers              │
│  - agents_config.rs loading functions   │
│  - All dependencies (tokio, axum, etc)  │
│                                         │
│ Embedded Constants:                     │
│  - CCO_VERSION = "2025.11.2"           │
│  - GIT_HASH = "a3f1b2c"                │
│  - BUILD_DATE = "2025-11-15 14:30:45"  │
└─────────────────────────────────────────┘
                    │
                    ▼
        ┌──────────────────────────┐
        │   Server Initialization  │
        ├──────────────────────────┤
        │ load_agents() called      │
        │ Reads from disk:         │
        │ ~/.claude/agents/*.md    │
        └──────────────────────────┘
                    │
                    ▼
        ┌──────────────────────────┐
        │    Runtime Memory        │
        ├──────────────────────────┤
        │ ServerState:             │
        │  cache: MokaCache        │
        │  router: ModelRouter     │
        │  agents: AgentsConfig    │
        │    agents: HashMap<      │
        │      String,             │
        │      Agent               │
        │    > (119 entries)       │
        │                          │
        │ Each Agent:              │
        │  name: "chief-architect" │
        │  model: "opus"           │
        │  description: "..."      │
        │  tools: ["Read", ...]    │
        └──────────────────────────┘
```

## Key Design Decisions

### 1. Why Not Compile Agent Files Into Binary?

**Decision:** Agent definitions are loaded from `~/.claude/agents/` at runtime, NOT compiled into binary.

**Rationale:**

```rust
// Agents are discovered at runtime:
let agents = Arc::new(load_agents());

// This allows:
// 1. Custom agents in ~/.claude/agents/ are included
// 2. Agents can be updated without recompiling
// 3. Smaller binary (no agent content compiled in)
// 4. Faster builds
```

**Trade-off:**

| Approach | Pros | Cons |
|----------|------|------|
| **Runtime loading** | Extensible, fast build, smaller binary | Requires file access at startup |
| **Compile-time** | Single binary, offline | Large binary, slow build, no extension |

### 2. Why HashMap for Agents?

**Decision:** Use `HashMap<String, Agent>` for agent storage

**Rationale:**

```rust
pub struct AgentsConfig {
    pub agents: HashMap<String, Agent>,
}

// Benefits:
// 1. O(1) agent lookup: agents.get("chief-architect")
// 2. Easy iteration: agents.values()
// 3. Fast serialization to JSON for HTTP responses
// 4. Standard Rust pattern, well understood
```

### 3. Why YAML Frontmatter?

**Decision:** Parse YAML frontmatter instead of separate metadata files

**Rationale:**

```markdown
---
name: chief-architect
model: opus
description: Strategic leadership
tools: Read, Write, Edit
---

# Content here...
```

**Benefits:**

1. **Single file per agent** - metadata + documentation together
2. **Standard markdown** - works with any markdown viewer
3. **Easy parsing** - simple key: value pairs, no complex YAML
4. **Self-documenting** - metadata visible at top of file
5. **Extensible** - can add fields without changing code

## Cargo Build System Integration

### Rerun Triggers

```rust
// build.rs
println!("cargo:rerun-if-changed=../config/");
println!("cargo:rerun-if-changed=../config/orchestra-config.json");
```

**Behavior:**

```
Step 1: Developer modifies agents/chief-architect.md
Step 2: Cargo detects file change in ../config/
Step 3: Cargo reruns build.rs script
Step 4: build.rs validates configs
Step 5: Cargo rebuilds Rust code
Step 6: New binary created with latest definitions
```

### Environment Variable Integration

```rust
// In build.rs
println!("cargo:rustc-env=CCO_VERSION={}", version);

// In source code (agents_config.rs)
let version = env!("CCO_VERSION");  // Compile-time constant
```

**How env!() works:**

1. `println!("cargo:rustc-env=VAR=VALUE")` sets environment variable
2. Compiler sees the variable during compilation
3. `env!("VAR")` macro replaced with string literal
4. No runtime overhead, pure string constant

## Performance Characteristics

### Build Time

```bash
# Full clean build
cargo clean
cargo build --release
# Time: 2-5 minutes (first time)
# Includes: cargo download, dependency compile, CCO compile

# Incremental build after file change
touch cco/config/agents/chief-architect.md
cargo build --release
# Time: 10-30 seconds
# Only recompiles changed modules

# Incremental build with no changes
cargo build --release
# Time: 0.01 seconds
# No recompilation needed
```

### Runtime Performance

```rust
// Agent lookup is O(1) HashMap access
let agent = state.agents.get("chief-architect");
// Time: < 1 microsecond

// List all agents is O(n) iteration
let all = state.agents.all();  // 119 agents
// Time: < 100 microseconds

// Serialize to JSON (Serde)
let json = serde_json::to_string(&agents);
// Time: < 1 millisecond for 119 agents

// HTTP response time
curl http://localhost:3000/api/agents/chief-architect
// Time: < 10 milliseconds (includes network)
```

## Error Handling

### Build-Time Errors

```rust
// Invalid JSON detected
match serde_json::from_str::<serde_json::Value>(&content) {
    Ok(_) => println!("cargo:warning=Validated config"),
    Err(e) => {
        eprintln!("Invalid JSON in {}: {}", config_file, e);
        panic!("Config validation failed");  // Stops build
    }
}
```

### Runtime Errors

```rust
// Agent not found
match state.agents.get(&agent_name) {
    Some(agent) => {
        Ok(Json(agent.clone()))  // Found
    }
    None => {
        Err((
            StatusCode::NOT_FOUND,
            Json(AgentNotFoundResponse {
                error: format!("Agent not found: {}", agent_name),
            }),
        ))
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: test-agent
model: sonnet
description: A test agent
tools: Read, Write, Edit
---

# Content
"#;
        let result = parse_frontmatter(content);
        assert!(result.is_some());
    }

    #[test]
    fn test_agents_config_operations() {
        let mut config = AgentsConfig::new();
        let agent = Agent {
            name: "test".to_string(),
            model: "sonnet".to_string(),
            description: "Test".to_string(),
            tools: vec!["Read".to_string()],
        };

        config.agents.insert("test".to_string(), agent.clone());

        assert_eq!(config.len(), 1);
        assert_eq!(config.get("test"), Some(&agent));
    }
}
```

### Integration Tests

```bash
# Build and test
cargo test --release

# Test agent loading
./target/release/cco run --port 3000 &
sleep 2
curl http://localhost:3000/api/agents | jq '.agents | length'
# Should return 119
```

## See Also

- [EMBEDDING_ARCHITECTURE.md](EMBEDDING_ARCHITECTURE.md) - System overview
- [BUILD_PROCESS.md](BUILD_PROCESS.md) - Build process guide
- [DEPLOYMENT_EMBEDDING.md](DEPLOYMENT_EMBEDDING.md) - Deployment guide
- [EMBEDDING_TROUBLESHOOTING.md](EMBEDDING_TROUBLESHOOTING.md) - Troubleshooting
- [config/agents/README.md](config/agents/README.md) - Agent development

## Summary

CCO's implementation:

1. **build.rs** - Validates configs, sets version, triggers recompilation
2. **agents_config.rs** - Parses YAML, loads from filesystem, provides lookup
3. **server.rs** - Creates HTTP endpoints, serves agents via API
4. **Runtime** - Loads agents on startup, keeps in memory, serves with O(1) access

The key insight: **Metadata travels in source files → validated at build time → loaded at runtime → served from memory**
