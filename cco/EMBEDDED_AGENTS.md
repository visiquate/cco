# Embedded Agents System

## Overview

The CCO binary includes agent definitions that are embedded at compile time via the `build.rs` script. This allows the binary to carry agent metadata without relying on external files or network calls.

## How It Works

### Build-Time Processing

1. **Source Discovery**: `build.rs` looks for markdown files in `cco/config/agents/`
2. **Fallback Source**: If no local files exist, it extracts agent definitions from `../config/orchestra-config.json`
3. **Parsing**: YAML frontmatter is extracted from markdown files:
   ```markdown
   ---
   name: agent-name
   model: haiku|sonnet|opus
   description: Agent description
   tools: Read, Write, Edit, Bash
   ---
   # Rest of markdown content...
   ```
4. **Code Generation**: Rust code is generated with:
   - `create_embedded_agents()` - Creates HashMap of all agents
   - `EMBEDDED_AGENTS_COUNT` - Total count of agents
   - `EMBEDDED_AGENT_NAMES` - Static array of agent names
   - `AGENT_MODELS` - Lookup table of agent to model mappings
   - `BUILD_STATS` - Build-time statistics

5. **Output**: Generated code is written to `target/debug/build/cco-*/out/agents.rs`

## File Structure

```
cco/
├── build.rs                    # Build script (generates embedded agents)
├── src/
│   ├── embedded_agents.rs      # Module that includes generated code
│   ├── agents_config.rs        # Agent struct definition
│   ├── lib.rs                  # Module exports
│   └── ...
├── config/
│   └── agents/                 # Agent markdown files (100+)
│       ├── chief-architect.md
│       ├── rust-specialist.md
│       ├── python-specialist.md
│       ├── api-explorer.md
│       └── ... (100+ more agents)
└── Cargo.toml
```

## Agent Markdown Format

Each agent file must have:

### Required Fields
- `name` - Unique identifier (kebab-case)
- `model` - One of: `opus`, `sonnet`, `haiku`
- `description` - Single-line description
- `tools` - Comma-separated list of available tools

### Example

```markdown
---
name: rust-specialist
model: haiku
description: Rust specialist for systems programming and performance
tools: Read, Write, Edit, Bash, Test, Performance
---

# Rust Specialist

Expert in Rust development...

## Specialties
- Systems programming
- Memory safety
- Performance optimization
```

## Runtime Access

### Module Functions

```rust
use cco::embedded_agents;

// Get all agents as HashMap
let agents = embedded_agents::initialize_embedded_agents();

// Get agent count
let count = embedded_agents::embedded_agent_count();

// Get all agent names
let names = embedded_agents::embedded_agent_names();

// Look up agent model by name
let model = embedded_agents::agent_model("rust-specialist");
assert_eq!(model, Some("haiku"));

// Get build statistics
let stats = embedded_agents::build_stats();
println!("{}", stats);
```

### Static Constants

```rust
use cco::embedded_agents::*;

// Total number of agents
println!("Total agents: {}", EMBEDDED_AGENTS_COUNT);

// List all agent names
for name in EMBEDDED_AGENT_NAMES {
    println!("- {}", name);
}

// Get model for agent
for (agent, model) in AGENT_MODELS {
    println!("{}: {}", agent, model);
}

// Build statistics
println!("{}", BUILD_STATS);
```

## Build Process

### Automatic Triggers

The build script rebuilds when:
- Any `.md` file in `cco/config/agents/` changes
- The `../config/orchestra-config.json` file changes

### Build Output

```
cargo build
   Compiling cco v0.0.0
    ✓ Embedded 120 agents into binary
    ...
   Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

## Model Distribution

The system automatically counts and reports agent distribution:

```
Embedded Agents: 120
  - Opus agents: 1
  - Sonnet agents: 37
  - Haiku agents: 82
```

## Error Handling

### Build-Time Validation

The build script validates:
1. **Required fields** - All files must have name, model, description
2. **Valid models** - Model must be one of: opus, sonnet, haiku
3. **File format** - Must start with `---` YAML frontmatter

### Warnings

Invalid files are skipped with warnings:
```
⚠ Invalid model 'gpt4' for agent 'my-agent', must be opus/sonnet/haiku
⚠ Failed to parse agent from: config/agents/bad-file.md
```

## Integration with agents_config.rs

The `embedded_agents.rs` module works alongside `agents_config.rs`:

- **agents_config.rs**: Runtime loading from `~/.claude/agents/` directory
- **embedded_agents.rs**: Compile-time embedding in binary

This provides fallback behavior:
1. First, try to load from embedded agents
2. Second, fall back to runtime loading from `~/.claude/agents/`

## Testing

### Unit Tests

```bash
cargo test embedded_agents
```

Tests verify:
- Agents are not empty
- All agent names are valid
- All models are valid (opus/sonnet/haiku)
- Agent lookup works correctly
- Build stats are available

### Example Test

```rust
#[test]
fn test_rust_specialist_embedded() {
    let agents = initialize_embedded_agents();
    assert!(agents.contains_key("rust-specialist"));

    let agent = agents.get("rust-specialist").unwrap();
    assert_eq!(agent.model, "haiku");
}
```

## Performance Characteristics

- **Embed time**: < 1 second (build.rs execution)
- **Binary size increase**: ~50-100 KB (for 120 agents)
- **Runtime lookup**: O(n) for iteration, O(1) for static access
- **Memory usage**: All static - no runtime allocation

## Debugging

### View Generated Code

```bash
cat target/debug/build/cco-*/out/agents.rs | head -100
```

### Count Embedded Agents

```bash
cargo build 2>&1 | grep "Embedded"
```

### Check Agent Files

```bash
ls -la cco/config/agents/*.md | wc -l
```

## Future Enhancements

1. **Dynamic Loading** - Load agents at runtime from embedded archive
2. **Caching** - Cache agent metadata after first load
3. **Versioning** - Track agent definition versions
4. **Validation** - Stricter YAML parsing with error reporting
5. **Distribution** - Publish agent definitions in separate crate

## Troubleshooting

### "No agents embedded"

- Check that `cco/config/agents/` exists and contains `.md` files
- Verify markdown files have valid YAML frontmatter
- Check build output for validation errors

### "Agent not found at runtime"

- Verify agent name matches exactly (case-sensitive, kebab-case)
- Check that agent file exists and was parsed
- Use `embedded_agent_names()` to see available agents

### "Invalid model error"

- Ensure model field is one of: `opus`, `sonnet`, `haiku`
- Check for typos (e.g., `opusv4` instead of `opus`)
- Verify YAML syntax is correct

## References

- Build script: `/Users/brent/git/cc-orchestra/cco/build.rs`
- Module: `/Users/brent/git/cc-orchestra/cco/src/embedded_agents.rs`
- Agent files: `/Users/brent/git/cc-orchestra/cco/config/agents/`
- Config fallback: `/Users/brent/git/cc-orchestra/config/orchestra-config.json`
