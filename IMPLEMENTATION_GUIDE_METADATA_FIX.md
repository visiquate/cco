# Implementation Guide: Agent Metadata Fix

**Objective:** Fix failing tests #17 and #19 by adding missing `type` and `role` fields
**Status:** Ready for Implementation
**Estimated Time:** 8-11 hours

---

## Change Summary

### What's Changing
- Adding 2 new fields to Agent data structure
- `type` - Agent categorization (required by Test #19)
- `role` - Agent primary role description (required by Test #17)

### Scope of Changes
- 3 code files (Rust source)
- 119 data files (agent definitions)
- 0 breaking changes
- 0 behavior changes (purely additive)

### Impact
- ✅ Fixes Test #17 failure
- ✅ Fixes Test #19 failure
- ✅ Achieves 100% test pass rate
- ✅ Enables "PRODUCTION READY" certification

---

## Step 1: Code Changes (Estimated: 2-3 hours)

### File 1: agents_config.rs - Agent Struct

**Location:** `/Users/brent/git/cc-orchestra/cco/src/agents_config.rs` (Lines 12-23)

**Current Code:**
```rust
/// Agent configuration with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Agent {
    /// Agent identifier (e.g., "chief-architect", "python-specialist")
    pub name: String,
    /// Assigned LLM model (e.g., "opus", "sonnet", "haiku")
    pub model: String,
    /// Description of agent's purpose and capabilities
    pub description: String,
    /// List of tools available to this agent
    pub tools: Vec<String>,
}
```

**Updated Code:**
```rust
/// Agent configuration with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Agent {
    /// Agent identifier (e.g., "chief-architect", "python-specialist")
    pub name: String,
    /// Assigned LLM model (e.g., "opus", "sonnet", "haiku")
    pub model: String,
    /// Agent type/category (e.g., "language-specialist", "security-auditor")
    pub type_: String,
    /// Agent primary role description
    pub role: String,
    /// Description of agent's purpose and capabilities
    pub description: String,
    /// List of tools available to this agent
    pub tools: Vec<String>,
}
```

**Changes:**
- Add `pub type_: String;` (use `type_` because `type` is a keyword)
- Add `pub role: String;`
- Add documentation comments

### File 1b: agents_config.rs - FrontmatterData Struct

**Location:** `/Users/brent/git/cc-orchestra/cco/src/agents_config.rs` (Lines 71-77)

**Current Code:**
```rust
#[derive(Debug, Clone)]
struct FrontmatterData {
    name: Option<String>,
    model: Option<String>,
    description: Option<String>,
    tools: Option<String>, // Will be parsed as comma-separated
}
```

**Updated Code:**
```rust
#[derive(Debug, Clone)]
struct FrontmatterData {
    name: Option<String>,
    model: Option<String>,
    type_: Option<String>,           // NEW
    role: Option<String>,             // NEW
    description: Option<String>,
    tools: Option<String>, // Will be parsed as comma-separated
}
```

**Changes:**
- Add `type_: Option<String>`
- Add `role: Option<String>`

### File 1c: agents_config.rs - parse_frontmatter Function

**Location:** `/Users/brent/git/cc-orchestra/cco/src/agents_config.rs` (Lines 83-137)

**Current Code (initialization section, lines 97-102):**
```rust
    let mut data = FrontmatterData {
        name: None,
        model: None,
        description: None,
        tools: None,
    };
```

**Updated Code:**
```rust
    let mut data = FrontmatterData {
        name: None,
        model: None,
        type_: None,              // NEW
        role: None,                // NEW
        description: None,
        tools: None,
    };
```

**Current Code (match section, lines 126-132):**
```rust
            match key {
                "name" => data.name = Some(value.to_string()),
                "model" => data.model = Some(value.to_string()),
                "description" => data.description = Some(value.to_string()),
                "tools" => data.tools = Some(value.to_string()),
                _ => {}
            }
```

**Updated Code:**
```rust
            match key {
                "name" => data.name = Some(value.to_string()),
                "model" => data.model = Some(value.to_string()),
                "type" => data.type_ = Some(value.to_string()),    // NEW
                "role" => data.role = Some(value.to_string()),      // NEW
                "description" => data.description = Some(value.to_string()),
                "tools" => data.tools = Some(value.to_string()),
                _ => {}
            }
```

**Changes:**
- Add `"type" => data.type_ = Some(value.to_string()),`
- Add `"role" => data.role = Some(value.to_string()),`

### File 1d: agents_config.rs - load_agent_from_file Function

**Location:** `/Users/brent/git/cc-orchestra/cco/src/agents_config.rs` (Lines 146-179)

**Current Code (field extraction, lines 157-160):**
```rust
    // Extract required fields
    let name = frontmatter.name?;
    let model = frontmatter.model?;
    let description = frontmatter.description?;
```

**Updated Code:**
```rust
    // Extract required fields
    let name = frontmatter.name?;
    let model = frontmatter.model?;
    let type_ = frontmatter.type_?;        // NEW
    let role = frontmatter.role?;           // NEW
    let description = frontmatter.description?;
```

**Current Code (Agent construction, lines 173-178):**
```rust
    Some(Agent {
        name,
        model,
        description,
        tools,
    })
```

**Updated Code:**
```rust
    Some(Agent {
        name,
        model,
        type_,          // NEW
        role,            // NEW
        description,
        tools,
    })
```

**Changes:**
- Extract `type_` and `role` from frontmatter
- Include in Agent struct construction

### File 1e: agents_config.rs - Tests

**Location:** `/Users/brent/git/cc-orchestra/cco/src/agents_config.rs` (Tests section)

**Current test_parse_frontmatter_valid (lines 282-301):**
```rust
    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: test-agent
model: sonnet
description: A test agent
tools: Read, Write, Edit
---

# Content here
"#;

        let result = parse_frontmatter(content);
        assert!(result.is_some());

        let data = result.unwrap();
        assert_eq!(data.name, Some("test-agent".to_string()));
        assert_eq!(data.model, Some("sonnet".to_string()));
        assert_eq!(data.description, Some("A test agent".to_string()));
        assert_eq!(data.tools, Some("Read, Write, Edit".to_string()));
    }
```

**Updated:**
```rust
    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: test-agent
model: sonnet
type: test-type
role: Test agent role
description: A test agent
tools: Read, Write, Edit
---

# Content here
"#;

        let result = parse_frontmatter(content);
        assert!(result.is_some());

        let data = result.unwrap();
        assert_eq!(data.name, Some("test-agent".to_string()));
        assert_eq!(data.model, Some("sonnet".to_string()));
        assert_eq!(data.type_, Some("test-type".to_string()));           // NEW
        assert_eq!(data.role, Some("Test agent role".to_string()));      // NEW
        assert_eq!(data.description, Some("A test agent".to_string()));
        assert_eq!(data.tools, Some("Read, Write, Edit".to_string()));
    }
```

**Current test_agents_config_operations (lines 311-326):**
```rust
    #[test]
    fn test_agents_config_operations() {
        let mut config = AgentsConfig::new();

        let agent = Agent {
            name: "test".to_string(),
            model: "sonnet".to_string(),
            description: "Test agent".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string()],
        };

        config.agents.insert("test".to_string(), agent.clone());

        assert_eq!(config.len(), 1);
        assert!(!config.is_empty());
        assert_eq!(config.get("test"), Some(&agent));
    }
```

**Updated:**
```rust
    #[test]
    fn test_agents_config_operations() {
        let mut config = AgentsConfig::new();

        let agent = Agent {
            name: "test".to_string(),
            model: "sonnet".to_string(),
            type_: "test-type".to_string(),                    // NEW
            role: "Test role".to_string(),                     // NEW
            description: "Test agent".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string()],
        };

        config.agents.insert("test".to_string(), agent.clone());

        assert_eq!(config.len(), 1);
        assert!(!config.is_empty());
        assert_eq!(config.get("test"), Some(&agent));
    }
```

**Summary of test changes:**
- Update test YAML to include `type` and `role`
- Add assertions for new fields
- Update Agent construction calls

---

## Step 2: Data File Updates (Estimated: 4-6 hours)

### Overview: 118 Agent Definition Files

**Location:** `/Users/brent/git/cc-orchestra/cco/config/agents/*.md`

**Pattern for each file:**
- Open file
- Add `type: [value]` to YAML frontmatter
- Add `role: [value]` to YAML frontmatter
- Save file

### Template Update Pattern

**Before:**
```yaml
---
name: agent-name
model: [opus|sonnet|haiku]
description: ...
tools: ...
---
```

**After:**
```yaml
---
name: agent-name
model: [opus|sonnet|haiku]
type: [category]
role: [role description]
description: ...
tools: ...
---
```

### Example Updates

#### Example 1: rust-specialist.md

**Current:**
```yaml
---
name: rust-specialist
model: haiku
description: Rust Specialist - Systems programming, memory safety, and high-performance code
tools: Read, Write, Edit, Bash, Test, Performance
---
```

**Updated:**
```yaml
---
name: rust-specialist
model: haiku
type: language-specialist
role: Systems programming and performance expert
description: Rust Specialist - Systems programming, memory safety, and high-performance code
tools: Read, Write, Edit, Bash, Test, Performance
---
```

#### Example 2: chief-architect.md

**Current:**
```yaml
---
name: chief-architect
model: opus
description: Chief Architect - Strategic decision-making and project guidance
tools: Read, Write, Edit, Bash, API, Database, Deploy, Test, Security, Performance
---
```

**Updated:**
```yaml
---
name: chief-architect
model: opus
type: architect
role: Strategic decision-making and orchestra coordination
description: Chief Architect - Strategic decision-making and project guidance
tools: Read, Write, Edit, Bash, API, Database, Deploy, Test, Security, Performance
---
```

#### Example 3: security-auditor.md

**Current:**
```yaml
---
name: security-auditor
description: Review code for vulnerabilities, implement secure authentication...
tools: Read, Write, Edit, Bash
model: sonnet
---
```

**Updated:**
```yaml
---
name: security-auditor
model: sonnet
type: security-specialist
role: Application security and vulnerability expert
description: Review code for vulnerabilities, implement secure authentication...
tools: Read, Write, Edit, Bash
---
```

### Type/Role Value Reference

See `/Users/brent/git/cc-orchestra/AGENT_TYPE_ROLE_MAPPING.md` for complete mapping of all 117 agents.

### Automation Suggestion

Create a script to update all files (pseudocode):

```bash
#!/bin/bash
# For each agent in mapping file
while read agent_name agent_type agent_role; do
  # Read current file
  file="config/agents/${agent_name}.md"

  # Extract metadata lines
  name=$(grep "^name:" "$file")
  model=$(grep "^model:" "$file")
  description=$(grep "^description:" "$file")
  tools=$(grep "^tools:" "$file")

  # Get content after frontmatter
  content=$(sed -n '/^---$/,/^---$/p' "$file" | tail -n +2 | head -n -1)

  # Write updated file
  cat > "$file" <<EOF
---
${name}
${model}
type: ${agent_type}
role: ${agent_role}
${description}
${tools}
---
${content}
EOF
done < agent_mappings.txt
```

---

## Step 3: agents.json Regeneration (Estimated: 1 hour)

### Option A: Automatic Regeneration

If build.rs already generates agents.json from .md files:
```bash
cargo build --release
```

This will automatically regenerate with new fields.

### Option B: Manual Update

If agents.json is maintained separately, update each agent:

**Current format:**
```json
{
  "name": "rust-specialist",
  "model": "haiku",
  "description": "Rust development specialist...",
  "tools": "Read, Write, Edit, Bash",
  "file_path": "cco/config/agents/rust-specialist.md"
}
```

**Updated format:**
```json
{
  "name": "rust-specialist",
  "model": "haiku",
  "type": "language-specialist",
  "role": "Systems programming and performance expert",
  "description": "Rust development specialist...",
  "tools": "Read, Write, Edit, Bash",
  "file_path": "cco/config/agents/rust-specialist.md"
}
```

### Validation Script

```bash
# Verify all agents have type and role
jq '.[] | select(.type == null or .role == null)' config/agents.json

# Should return: empty (no output)
# If agents are returned, those are missing fields
```

---

## Step 4: Build and Test (Estimated: 1-2 hours)

### Build Process

```bash
# Clean previous builds
cargo clean

# Build release version
cargo build --release

# Expected output:
# - Compiling cco v0.0.0 (with new Agent fields)
# - Finished release with 2 new fields
# - Binary should be ~10MB
```

### Verify Embedded Agents

```bash
# Check binary contains agent data
strings ./target/release/cco | grep "type.*language-specialist" | head -5

# Expected: Several matches showing type field is embedded
```

### Run Unit Tests

```bash
cargo test --release

# Expected: All tests pass including:
# - test_parse_frontmatter_valid
# - test_agents_config_operations
# - test_load_agents_from_embedded_agents_have_required_fields
```

### Run E2E Tests

```bash
./cco/comprehensive-e2e-test.sh

# Expected results:
# Test #17 Step 2: PASS (role field present)
# Test #19: PASS (all agents have type field)
# Pass Rate: 28/28 (100%)
# Status: ✅ PRODUCTION READY
```

---

## Step 5: Validation Checklist

Before declaring complete:

### Code Changes
- [ ] Agent struct updated with type_ and role fields
- [ ] FrontmatterData struct updated
- [ ] parse_frontmatter() handles new fields
- [ ] load_agent_from_file() extracts new fields
- [ ] Unit tests updated and passing

### Data Changes
- [ ] All 118 .md files have type field in frontmatter
- [ ] All 118 .md files have role field in frontmatter
- [ ] agents.json updated with type and role for all 117 agents
- [ ] agents.json validates with: `jq . < agents.json > /dev/null`

### Build
- [ ] Clean build succeeds: `cargo build --release`
- [ ] Binary size reasonable (~10MB)
- [ ] Binary contains embedded agents

### Tests
- [ ] Unit tests pass: `cargo test --release`
- [ ] E2E Test #17 passes
- [ ] E2E Test #19 passes
- [ ] Full E2E suite passes: 28/28
- [ ] Test report shows "PRODUCTION READY"

### API Verification
- [ ] Server starts: `./target/release/cco run --port 3000`
- [ ] Health endpoint responds: `curl http://localhost:3000/health`
- [ ] Agent API includes type: `curl http://localhost:3000/api/agents/rust-specialist | jq .type`
- [ ] Agent API includes role: `curl http://localhost:3000/api/agents/rust-specialist | jq .role`

---

## File-by-File Checklist

### Code Files (3 total)

1. **src/agents_config.rs**
   - [ ] Agent struct: add type_ field (String)
   - [ ] Agent struct: add role field (String)
   - [ ] FrontmatterData: add type_ field (Option<String>)
   - [ ] FrontmatterData: add role field (Option<String>)
   - [ ] parse_frontmatter initialization: add type_: None, role: None
   - [ ] parse_frontmatter match: add "type" and "role" cases
   - [ ] load_agent_from_file: extract type_ and role
   - [ ] load_agent_from_file: include in Agent construction
   - [ ] test_parse_frontmatter_valid: add fields to test data and assertions
   - [ ] test_agents_config_operations: add fields to Agent construction

2. **build.rs**
   - [ ] Review (no changes likely needed, just verify it picks up new fields)

3. **Cargo.toml**
   - [ ] No changes (unless versioning desired)

### Data Files (119 total)

4. **config/agents/*.md** (118 files)
   - See mapping document for each agent's type and role
   - Pattern: Add to YAML frontmatter before description field

5. **config/agents.json**
   - [ ] Add "type" field after "model" for each agent
   - [ ] Add "role" field after "type" for each agent
   - [ ] Validate JSON syntax

---

## Troubleshooting

### Issue: Tests still failing after build

**Symptom:** Test #17 or #19 still report failures
**Cause:** Binary not rebuilt or agents not reloaded
**Solution:**
```bash
# Force clean rebuild
cargo clean
cargo build --release

# Verify binary has new fields
strings ./target/release/cco | grep "language-specialist" | wc -l
# Should output: 10+ matches
```

### Issue: "Agent definition incomplete" in Test #17

**Symptom:** Test still shows `.role` is null
**Cause:** Agent .md file missing `role:` field in frontmatter
**Solution:**
- Check frontmatter in agent .md file
- Verify format: `role: [text]`
- Rebuild: `cargo build --release`

### Issue: "Incomplete agents" count not zero in Test #19

**Symptom:** Test shows agents with missing type field
**Cause:** Some agents missing `type:` in frontmatter
**Solution:**
- Use mapping document to verify all agents have type
- Check agents.json for null type values:
  ```bash
  jq '.[] | select(.type == null)' config/agents.json
  ```
- Add missing type fields
- Rebuild

### Issue: JSON parsing error in agents.json

**Symptom:** `jq: parse error` when reading agents.json
**Cause:** Syntax error in JSON file
**Solution:**
- Validate JSON:
  ```bash
  jq . < config/agents.json > /dev/null
  ```
- Check for missing commas between fields
- Verify quotes are balanced
- Regenerate from .md files if needed

---

## Rollback Plan

If issues occur during implementation:

1. **Revert code changes:**
   ```bash
   git checkout src/agents_config.rs
   cargo build --release
   ```

2. **Revert data changes:**
   ```bash
   git checkout config/agents/*.md config/agents.json
   ```

3. **Verify rollback:**
   ```bash
   ./cco/comprehensive-e2e-test.sh
   # Should show same 26/28 as before
   ```

---

## Success Criteria

### Final Verification

When complete, this command should show all agents with type and role:

```bash
curl -s http://localhost:3000/api/agents | jq '.[] | {name, type: .type_, role: .role}' | head -30
```

Output should look like:
```json
{
  "name": "rust-specialist",
  "type": "language-specialist",
  "role": "Systems programming and performance expert"
}
{
  "name": "chief-architect",
  "type": "architect",
  "role": "Strategic decision-making and orchestra coordination"
}
...
```

### Test Results

Final test output should show:
```
Test #17: ✅ PASS - Agent has complete definition
Test #19: ✅ PASS - All agents have complete data
...
Total Tests: 28
Passed: 28
Failed: 0
Pass Rate: 100%

Status: ✅ PRODUCTION READY
```

---

## Summary

**Scope:** 3 code files, 119 data files
**Complexity:** Low (no algorithm changes)
**Risk:** Very low (additive only)
**Impact:** High (fixes test failures)
**Effort:** 8-11 hours
**Value:** Enables production certification

---

**Document Status:** READY FOR IMPLEMENTATION
**Date:** November 15, 2025
**Next Step:** Begin with Step 1 code changes
