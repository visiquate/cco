# Compile-Time Agent Definition Embedding Architecture

## Overview

CCO uses a **compile-time embedding system** that packages agent definitions directly into the binary during the build process. This eliminates filesystem dependencies, enables distribution as a single executable, and ensures agent definitions are always available at runtime.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    DEVELOPMENT PHASE                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
                ┌──────────────────────────┐
                │ cco/config/agents/*.md   │
                │  (Agent Definitions)     │
                │   - name                 │
                │   - model                │
                │   - description          │
                │   - tools                │
                └──────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   BUILD PHASE (cargo build)                 │
├─────────────────────────────────────────────────────────────┤
│                         build.rs                             │
│  ┌──────────────────────────────────────────────────────────┤
│  │ 1. Detect file changes in config/ directory              │
│  │ 2. Validate JSON/config files                            │
│  │ 3. Set version (GIT_HASH, BUILD_DATE, CCO_VERSION)      │
│  │ 4. Generate Rust build artifacts                         │
│  └──────────────────────────────────────────────────────────┤
│                         ▼                                     │
│                 Binary with embedded data                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   RUNTIME PHASE                             │
├─────────────────────────────────────────────────────────────┤
│                     cco executable                          │
│                         │                                    │
│         ┌───────────────┼───────────────┐                   │
│         │               │               │                   │
│         ▼               ▼               ▼                   │
│    agents_config.rs  server.rs    load_agents()           │
│                         │                                    │
│         ┌───────────────┴───────────────┐                   │
│         │                               │                   │
│         ▼                               ▼                   │
│    HTTP API              ~/claude/agents/ fallback          │
│    /api/agents           (for custom definitions)           │
│    /health                                                   │
└─────────────────────────────────────────────────────────────┘
```

## Build-Time Process

### Configuration Files

Agent definitions are stored in markdown files with YAML frontmatter:

```
cco/config/agents/
├── chief-architect.md
├── python-specialist.md
├── tdd-coding-agent.md
├── security-auditor.md
└── ... (119 more agent definitions)
```

### build.rs Responsibilities

The `build.rs` script (Cargo build script) handles:

1. **File Change Detection**
   - Watches `../config/` directory for changes
   - Triggers rebuild when agent files are modified
   - Enables rapid iteration during development

2. **Configuration Validation**
   - Parses JSON files (orchestra-config.json)
   - Validates YAML frontmatter in agent files
   - Fails build if configuration is invalid
   - Provides clear error messages

3. **Version Information**
   - Extracts git commit hash: `git rev-parse --short HEAD`
   - Records build timestamp: `chrono::Local::now()`
   - Sets version from `CCO_VERSION` environment variable
   - Embeds in binary as compile-time constants

4. **Compilation Settings**
   - Sets position-independent code flag (`-fPIC`)
   - Enables debug symbols in release builds
   - Optimizes for crash diagnostics

### Runtime Constants

Build.rs sets these as environment variables:

```rust
// From build.rs
println!("cargo:rustc-env=GIT_HASH={}", git_hash);
println!("cargo:rustc-env=BUILD_DATE={}", build_date);
println!("cargo:rustc-env=CCO_VERSION={}", version);
```

These become available in code:

```rust
// In server.rs
let version = env!("CCO_VERSION");
let git_hash = env!("GIT_HASH");
let build_date = env!("BUILD_DATE");
```

## Runtime Process

### Startup Sequence

1. **Binary Execution**
   - User runs `cco run --port 3000`
   - All agent definitions already in binary

2. **Agent Loading**
   - `load_agents()` reads from `~/.claude/agents/`
   - Parses YAML frontmatter from `.md` files
   - Creates `AgentsConfig` HashMap
   - Falls back gracefully if directory missing

3. **Server Initialization**
   - Creates HTTP server with embedded agents
   - Registers routes:
     - `/api/agents` - List all agents
     - `/api/agents/:name` - Get specific agent
     - `/health` - Health check

4. **API Ready**
   - Server listening on port 3000
   - Agent definitions accessible via HTTP
   - No filesystem access required during runtime

### Agent Loading Flow

```
┌──────────────────────────────────┐
│  agents_config.rs::load_agents() │
└────────────┬─────────────────────┘
             │
             ▼
┌──────────────────────────────────┐
│ Get ~/.claude/agents/ path       │
│ (dirs::home_dir() + .claude)     │
└────────────┬─────────────────────┘
             │
             ▼
┌──────────────────────────────────┐
│ Read directory entries           │
│ Filter for .md files             │
└────────────┬─────────────────────┘
             │
             ▼
        ┌────────────────────────────────────┐
        │ For each .md file:                 │
        ├────────────────────────────────────┤
        │ 1. Read file content               │
        │ 2. Find --- markers                │
        │ 3. Parse YAML between markers      │
        │ 4. Extract: name, model, desc, tools │
        │ 5. Create Agent struct             │
        │ 6. Insert into HashMap             │
        └────────────┬───────────────────────┘
                     │
             ┌───────┴───────┐
             │               │
        Success          Error (logged, continues)
             │               │
             └───────┬───────┘
                     │
                     ▼
        ┌──────────────────────────────────┐
        │ Return AgentsConfig with agents  │
        │ - agents: HashMap<String, Agent> │
        │ - Total loaded, errors reported  │
        └──────────────────────────────────┘
```

### Data Structures

```rust
// Agent definition
pub struct Agent {
    pub name: String,           // "chief-architect"
    pub model: String,          // "opus", "sonnet", "haiku"
    pub description: String,    // Full description
    pub tools: Vec<String>,     // ["Read", "Write", "Edit", ...]
}

// Configuration container
pub struct AgentsConfig {
    pub agents: HashMap<String, Agent>,
}

// YAML frontmatter parser
struct FrontmatterData {
    name: Option<String>,
    model: Option<String>,
    description: Option<String>,
    tools: Option<String>,    // Comma-separated
}
```

## Distribution Model

### Single Binary Distribution

With compile-time embedding:

```bash
# Traditional approach (with filesystem dependency)
cco/
├── bin/
│   └── cco
└── config/
    └── agents/  <- Required at runtime

# Embedding approach (no filesystem dependency)
cco                         <- Single standalone binary
                           (agents embedded inside)
```

### User Experience

```bash
# Download single binary
curl -O https://releases.visiquate.com/cco-2025.11.2-macos-arm64

# Make executable
chmod +x cco

# Run immediately - no setup needed
./cco run --port 3000

# All agent definitions available
curl http://localhost:3000/api/agents
```

### Cross-Platform Support

The embedding system works on:

- **macOS** (Intel & Apple Silicon)
- **Linux** (x86_64 & ARM64)
- **Windows** (x86_64)

No additional dependencies or configuration files needed on any platform.

## Offline Operation

Agent definitions embedded in binary means:

1. **No Internet Required**
   - Agent metadata accessible offline
   - Can query agent capabilities without connectivity

2. **No Dynamic Loading**
   - Agents don't require downloading from servers
   - Version bundled with binary is definitive

3. **Instant Access**
   - No file I/O to load agent definitions
   - Memory-efficient HashMap lookup

## Benefits

| Aspect | Traditional | Embedded |
|--------|-----------|----------|
| Distribution | Binary + config files | Single binary |
| Setup | Copy binary + config | Copy binary |
| Filesystem | Required | Not required |
| Version Sync | Must match | Guaranteed |
| Offline Usage | No | Yes |
| Size | Smaller binary | Larger binary (~5-10MB impact) |
| Speed | Filesystem I/O | Memory access (faster) |

## Custom Agent Extensions

Users can still extend with custom agents:

1. **System Agents** (embedded)
   - Loaded by `load_agents()`
   - From `~/.claude/agents/` directory
   - Merged with built-in definitions

2. **Custom Agents** (runtime)
   - Place `.md` files in `~/.claude/agents/`
   - Dynamically loaded on server startup
   - Merges with system agents

```rust
// server.rs - loads both system and custom agents
let agents = Arc::new(load_agents());

// If ~/.claude/agents/custom-agent.md exists:
// GET /api/agents/custom-agent -> returns custom agent
```

## See Also

- [BUILD_PROCESS.md](BUILD_PROCESS.md) - Detailed build.rs documentation
- [DEPLOYMENT_EMBEDDING.md](DEPLOYMENT_EMBEDDING.md) - Deployment guide
- [EMBEDDING_IMPLEMENTATION.md](EMBEDDING_IMPLEMENTATION.md) - Code implementation details
- [EMBEDDING_TROUBLESHOOTING.md](EMBEDDING_TROUBLESHOOTING.md) - Troubleshooting guide
