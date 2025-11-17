# Embedded Agents Quick Start

Get the embedded agents system working in 5 minutes.

## What You Get

- 118 agent definitions embedded in the binary
- No runtime file loading required
- Type-safe Rust interface
- Automatic rebuild on agent file changes

## Files Created/Modified

### Core Files
- `build.rs` (enhanced) - Build script that generates embedded agents
- `src/embedded_agents.rs` (enhanced) - Runtime API module
- `cco/config/agents/*.md` (118 files) - Agent definitions

### Documentation
- `EMBEDDED_AGENTS.md` - Complete usage guide
- `BUILD_RS_IMPLEMENTATION.md` - Technical details
- `BUILD_SUMMARY.md` - Project completion summary

## Quick Build

```bash
cd /Users/brent/git/cc-orchestra/cco

# Build the project
cargo build

# Run tests
cargo test embedded_agents --lib

# Verify installation
bash verify-build.sh
```

## Using Embedded Agents

### Basic Example
```rust
use cco::embedded_agents;

// Get all agents
let agents = embedded_agents::initialize_embedded_agents();
println!("Total agents: {}", agents.len());

// Get agent count
let count = embedded_agents::embedded_agent_count();
println!("Agents: {}", count);

// Get agent names
for name in embedded_agents::embedded_agent_names() {
    println!("- {}", name);
}

// Look up agent model
if let Some(model) = embedded_agents::agent_model("rust-specialist") {
    println!("Model: {}", model);
}
```

### Available Functions

```rust
use cco::embedded_agents::*;

// Create HashMap of all agents
let agents = create_embedded_agents();

// Get helper functions
let count = embedded_agent_count();
let names = embedded_agent_names();
let model = agent_model("agent-name");
let stats = build_stats();

// Access constants directly
println!("Total: {}", EMBEDDED_AGENTS_COUNT);
for name in EMBEDDED_AGENT_NAMES {
    println!("- {}", name);
}
for (name, model) in AGENT_MODELS {
    println!("{}: {}", name, model);
}
println!("{}", BUILD_STATS);
```

## Adding New Agents

### Create Agent File

Create `cco/config/agents/my-agent.md`:

```markdown
---
name: my-agent
model: haiku
description: My agent description
tools: Read, Write, Edit, Bash
---

# My Agent

Detailed capabilities here...
```

### Required Fields
- `name` - Unique identifier (kebab-case)
- `model` - opus, sonnet, or haiku
- `description` - Single-line description
- `tools` - Comma-separated tools

### Rebuild
```bash
cargo build  # Automatically includes new agent
```

## Current Agents

### By Model
- **Opus** (1): Chief Architect
- **Sonnet** (37): Integration specialists, architects
- **Haiku** (80): Language specialists, basic agents

### By Category
- Coding: Python, Rust, Go, Swift, Flutter
- Integration: API, Salesforce, Authentik
- Development: Frontend, Backend, Fullstack
- Infrastructure: DevOps, Database, Cloud
- Support: Documentation, Testing, Security

## Build Output

```
cargo build
   Compiling cco v0.0.0
    ✓ Embedded 118 agents into binary
    ...
   Finished dev [unoptimized + debuginfo] target(s) in 1.23s
```

## Verification

```bash
# Automated verification
bash verify-build.sh

# Manual checks
# Count agents
ls cco/config/agents/*.md | wc -l

# Check format
head -10 cco/config/agents/rust-specialist.md

# View generated code
cat target/debug/build/cco-*/out/agents.rs | head -50
```

## Testing

```bash
# Run all tests
cargo test embedded_agents --lib

# Run with output
cargo test embedded_agents --lib -- --nocapture

# Run specific test
cargo test test_embedded_agents_not_empty --lib
```

## Architecture

```
Agent Files (.md)
    ↓
build.rs parses & validates
    ↓
Generates Rust code
    ↓
Embeds in binary
    ↓
embedded_agents.rs provides API
    ↓
Your application uses agents
```

## Performance

- **Build time**: <1 second (for agent parsing)
- **Binary increase**: ~100-150 KB
- **Runtime lookup**: O(1)
- **Memory**: All static, no heap

## Common Tasks

### Access Agents in Code
```rust
let agents = initialize_embedded_agents();
```

### Get Agent Model
```rust
let model = agent_model("python-specialist").unwrap();
```

### List All Agents
```rust
for name in embedded_agent_names() {
    println!("{}", name);
}
```

### Get Statistics
```rust
println!("Embedded agents: {}", embedded_agent_count());
println!("{}", build_stats());
```

## Troubleshooting

### "No agents embedded"
- Check `cco/config/agents/` exists
- Verify markdown files present
- Review build output

### "Agent not found"
- Check name (case-sensitive)
- Use `embedded_agent_names()` to verify
- Ensure file exists and parsed correctly

### "Invalid model" error
- Model must be: opus, sonnet, haiku
- Check for typos
- Verify YAML syntax

## Documentation

- **Complete Guide**: `EMBEDDED_AGENTS.md`
- **Technical Details**: `BUILD_RS_IMPLEMENTATION.md`
- **Project Summary**: `BUILD_SUMMARY.md`
- **This Guide**: `EMBEDDED_AGENTS_QUICK_START.md`

## Key Files

```
/Users/brent/git/cc-orchestra/cco/
├── build.rs                          # Build script
├── src/embedded_agents.rs            # Runtime module
├── cco/config/agents/                # Agent definitions (118 files)
├── EMBEDDED_AGENTS.md                # Usage guide
├── BUILD_RS_IMPLEMENTATION.md        # Technical details
├── BUILD_SUMMARY.md                  # Project summary
├── EMBEDDED_AGENTS_QUICK_START.md    # This file
└── verify-build.sh                   # Verification script
```

## Next Steps

1. **Build**: `cargo build`
2. **Test**: `cargo test embedded_agents --lib`
3. **Verify**: `bash verify-build.sh`
4. **Add Agents**: Create new `.md` files
5. **Deploy**: Use release binary

---

**Ready to go!** Start with: `cargo build`
