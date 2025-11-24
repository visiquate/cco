# CCO Model Override - Configuration Reference

Complete reference for the `model-overrides.toml` configuration file.

## File Location

```
/Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
```

## Configuration Structure

The configuration file is divided into four main sections:

1. `[overrides]` - Enable/disable and define override rules
2. `[analytics]` - Configure logging and statistics tracking
3. `[per_model_rules]` - (Future) Fine-grained per-model configuration
4. `[override_statistics]` - (Runtime) Statistics populated during operation

## [overrides] Section

This section controls the core override functionality.

### `enabled` - Global Override Switch

**Type:** Boolean
**Default:** `true`
**Description:** Enables or disables all model overrides globally.

**Example:**
```toml
[overrides]
enabled = true  # All rules are active
```

**Impact:**
- When `true`: All rules in the `rules` array are applied
- When `false`: All overrides are disabled (no model rewriting)

**Use Cases:**
- Set to `false` to temporarily disable overrides without removing rules
- Set to `true` to re-enable overrides after testing

### `rules` - Model Override Rules

**Type:** Array of two-element arrays (string pairs)
**Default:** 3 built-in Sonnet→Haiku rules
**Description:** List of model rewrite rules. Each rule maps an original model to a replacement model.

**Format:**
```toml
rules = [
    ["original_model_name", "replacement_model_name"],
    # Add more rules below:
]
```

**Built-in Rules:**
```toml
rules = [
    # Sonnet 4.5 → Haiku 4.5 (75% cost savings)
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # Sonnet 4 → Haiku 4.5 (75% cost savings)
    ["claude-sonnet-4", "claude-haiku-4-5-20251001"],

    # Sonnet 3.5 → Haiku 4.5 (75% cost savings)
    ["claude-sonnet-3.5", "claude-haiku-4-5-20251001"],
]
```

**Rule Mechanics:**

1. Request arrives: `model = "claude-sonnet-4.5-20250929"`
2. CCO checks rules in order
3. First matching rule is applied
4. Model is rewritten: `model = "claude-haiku-4-5-20251001"`
5. Request sent to API with new model

**Adding Custom Rules:**

```toml
rules = [
    # Keep existing rules
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # Add new rules below:
    ["claude-opus-4-1-20250805", "claude-sonnet-4.5-20250929"],
    ["gpt-4-turbo", "gpt-3.5-turbo"],
    ["text-davinci-003", "text-davinci-002"],
]
```

**Model Name Format:**

Model names must match exactly (case-sensitive):

- ✅ `claude-sonnet-4.5-20250929` (correct)
- ❌ `Claude-Sonnet-4.5-20250929` (wrong - capital C)
- ❌ `claude-sonnet-4.5` (wrong - missing date)
- ❌ `sonnet` (wrong - too short)

**Finding Current Model Names:**

Check the [Anthropic Models API documentation](https://docs.anthropic.com/api/models):

```bash
# Or query via API (requires valid key):
curl -H "x-api-key: $ANTHROPIC_API_KEY" \
  https://api.anthropic.com/v1/models | jq '.models[].id'
```

**Performance Note:**

Rules are evaluated in order. Place most-used rules first for slight performance improvement:

```toml
rules = [
    # Rule 1: Most frequently used override (checked first)
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # Rule 2: Less frequent
    ["claude-opus-4-1-20250805", "claude-sonnet-4.5-20250929"],

    # Rule 3: Rarely used
    ["gpt-4-turbo", "gpt-3.5-turbo"],
]
```

**Disabling Specific Rules:**

Comment out rules to disable them without deleting:

```toml
rules = [
    # This rule is active
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # This rule is disabled
    # ["claude-opus-4-1-20250805", "claude-sonnet-4.5-20250929"],
]
```

## [analytics] Section

Configure logging and statistics tracking.

### `log_overrides` - Console Logging

**Type:** Boolean
**Default:** `true`
**Description:** Logs every model override to the console.

**Example:**
```toml
[analytics]
log_overrides = true
```

**Console Output:**
```
🔄 Model override: claude-sonnet-4.5-20250929 → claude-haiku-4-5-20251001
📝 Processing chat request for model: claude-haiku-4-5-20251001
📊 Override count: 47 (cost saved: $18.50)
```

**Use Cases:**
- Set to `true` for development/testing to see overrides happening
- Set to `false` in production to reduce console noise
- Enable when troubleshooting override issues

### `track_statistics` - Statistics Collection

**Type:** Boolean
**Default:** `true`
**Description:** Tracks override statistics for analytics (queryable via API).

**Example:**
```toml
[analytics]
track_statistics = true
```

**Data Tracked:**
- Total override count
- Overrides per model
- Cost savings by model
- Percentage breakdown

**Impact:**
- Minimal performance overhead
- Essential for dashboard and reports
- Should be `true` in all environments

### `report_format` - Output Format

**Type:** String (`"json"`, `"text"`, or `"silent"`)
**Default:** `"json"`
**Description:** Format for console output messages.

**Examples:**

**JSON format:**
```toml
report_format = "json"

# Output:
# {"event":"model_override","from":"claude-sonnet-4.5","to":"claude-haiku-4-5","timestamp":"2024-11-15T10:30:00Z"}
```

**Text format:**
```toml
report_format = "text"

# Output:
# 🔄 Model override: claude-sonnet-4.5-20250929 → claude-haiku-4-5-20251001
```

**Silent format:**
```toml
report_format = "silent"

# No console output (but statistics still tracked)
```

**Use Cases:**
- `json`: Integration with log aggregation systems
- `text`: Human-readable console output
- `silent`: Production deployments with external logging

## [per_model_rules] Section

Reserved for future use. This section will enable fine-grained configuration per model.

**Future Example (Not Yet Implemented):**
```toml
[per_model_rules]
# Only override during specific times
"claude-sonnet-4.5-20250929" = {
    "override_to" = "claude-haiku-4-5-20251001",
    "time_window" = "business_hours",  # future feature
    "percentage" = 100,
}

# Override only specific percentage of requests
"claude-opus-4-1-20250805" = {
    "override_to" = "claude-sonnet-4.5-20250929",
    "percentage" = 50,  # future feature: 50% of requests
}
```

Currently, all rules apply 100% of the time.

## [override_statistics] Section

This section is populated at runtime by CCO. Do not edit manually.

**Example Content:**
```toml
[override_statistics]
original_model = "claude-sonnet-4.5-20250929"
override_to = "claude-haiku-4-5-20251001"
requests_rewritten = 42
cost_saved = "$18.50"
percentage_saved = "73%"
last_override = "2024-11-15T10:35:22Z"
```

**This is automatically updated by CCO and should not be edited.**

## Complete Example Configuration

Here's a complete, annotated example:

```toml
# CCO Model Override Configuration
# This configuration enables transparent model rewriting at the proxy layer.

[overrides]
# ========================================
# Enable or disable model overrides globally
# Set to false to temporarily disable all overrides
enabled = true

# Model rewrite rules
# Format: ["original_model", "replacement_model"]
#
# The proxy will rewrite any request for the original model to use the replacement.
# Rules are applied in order, with the first match winning.
#
# Current cost savings (Anthropic pricing):
# - Sonnet → Haiku: 73% savings
# - Opus → Sonnet: 90% savings
rules = [
    # ========================================
    # Sonnet → Haiku rewrites (73% cost savings)
    # ========================================
    # Latest Sonnet version
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # Older Sonnet versions (for compatibility)
    ["claude-sonnet-4", "claude-haiku-4-5-20251001"],
    ["claude-sonnet-3.5", "claude-haiku-4-5-20251001"],

    # ========================================
    # Opus → Sonnet rewrites (90% cost savings)
    # Uncomment to enable
    # ========================================
    # ["claude-opus-4-1-20250805", "claude-sonnet-4.5-20250929"],

    # ========================================
    # Other LLM providers (examples)
    # Uncomment to enable
    # ========================================
    # ["gpt-4-turbo", "gpt-3.5-turbo"],
    # ["gpt-4-32k", "gpt-4-turbo"],
    # ["text-davinci-003", "text-davinci-002"],
]

[analytics]
# ========================================
# Logging and analytics configuration
# ========================================

# Log all model overrides to console
# Set to true for development, false for production
log_overrides = true

# Track override statistics (for dashboard and API)
# Keep this true for monitoring and cost analysis
track_statistics = true

# Report format options
# - "json": Machine-readable format for log aggregation
# - "text": Human-readable emoji format
# - "silent": No console output (but stats still tracked)
report_format = "json"

[per_model_rules]
# ========================================
# Per-model rules (future feature)
# Currently not used - all rules apply 100% of the time
# ========================================

# Example (not yet implemented):
# "claude-sonnet-4.5-20250929" = {
#     "override_to" = "claude-haiku-4-5-20251001",
#     "conditions" = {
#         "time_window" = "business_hours"
#     }
# }

[override_statistics]
# ========================================
# Runtime statistics (auto-populated by CCO)
# DO NOT EDIT THIS SECTION
# ========================================
# These are automatically updated by CCO with current override statistics
# Example output:
# original_model = "claude-sonnet-4.5-20250929"
# override_to = "claude-haiku-4-5-20251001"
# requests_rewritten = 42
# cost_saved = "$18.50"
```

## Configuration Best Practices

### 1. Validation

Always validate TOML syntax before deploying:

```bash
# Rust will validate on startup and report errors
./target/release/cco run --port 3000

# Look for error messages like:
# Error: failed to parse model-overrides.toml: ...
```

### 2. Backups

Always backup before making changes:

```bash
cp model-overrides.toml model-overrides.toml.backup
```

### 3. Testing Changes

Test in development first:

```bash
# Test with new config
./target/release/cco run --port 8000

# Make requests and verify overrides are working
curl http://localhost:8000/api/overrides/stats
```

### 4. Gradual Rollout

For new rules, roll out gradually:

```toml
# Phase 1: Monitor without impacting users
# Add new rules but keep log_overrides = true to see impact

# Phase 2: After monitoring, set log_overrides = false
# Continue tracking statistics
```

### 5. Ordering

Place frequently-used rules first for slight performance improvement:

```toml
rules = [
    # Most common override first
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # Less common overrides follow
]
```

### 6. Commenting

Comment your rules to document why they exist:

```toml
rules = [
    # Required: Sonnet is too expensive for development workflows
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # Optional: Opus might be too powerful for simple tasks
    # ["claude-opus-4-1-20250805", "claude-sonnet-4.5-20250929"],
]
```

## Common Configuration Scenarios

### Scenario 1: Maximum Cost Savings

Override all expensive models to cheapest available:

```toml
[overrides]
enabled = true
rules = [
    ["claude-opus-4-1-20250805", "claude-haiku-4-5-20251001"],
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],
    ["gpt-4-turbo", "gpt-3.5-turbo"],
]
```

### Scenario 2: Conservative Approach

Only override Sonnet, preserve Opus for complex tasks:

```toml
[overrides]
enabled = true
rules = [
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],
    # Keep Opus unchanged
]
```

### Scenario 3: Development vs. Production

Use configuration environment variables (advanced):

```bash
# Set different configs per environment
DEV_CONFIG="config/model-overrides.dev.toml"
PROD_CONFIG="config/model-overrides.prod.toml"

./target/release/cco --config $DEV_CONFIG
```

### Scenario 4: Testing Phase

Log everything to verify overrides work:

```toml
[overrides]
enabled = true
rules = [
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],
]

[analytics]
log_overrides = true
report_format = "text"
```

## Troubleshooting Configuration

### Config Not Loaded

**Symptom:** Changes don't take effect after restart.

**Solutions:**

```bash
# 1. Verify file location
ls -la /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# 2. Check syntax errors
./target/release/cco run --port 3000
# Look for: "Error: failed to parse model-overrides.toml"

# 3. Verify permissions
chmod 644 /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# 4. Ensure you restarted CCO
pkill cco
./target/release/cco run --port 3000
```

### Rules Not Matching

**Symptom:** Override never triggers.

**Solutions:**

```bash
# 1. Verify exact model name
echo "Checking model names in config:"
grep "claude-sonnet" /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# 2. Check what models are actually being requested
curl http://localhost:3000/api/overrides/stats | jq

# 3. Verify rules are correctly formatted
# Rules must be: ["original", "replacement"]
# Not: "original" = "replacement"
```

### Invalid TOML Syntax

**Symptom:** CCO won't start with error about invalid TOML.

**Common mistakes:**

```toml
# WRONG: Using = instead of []
overrides = true  # ✗ This is a key=value

# RIGHT:
[overrides]       # ✓ This is a section

# WRONG: Missing quotes
rules = [
    [claude-sonnet, claude-haiku]  # ✗ Missing quotes
]

# RIGHT:
rules = [
    ["claude-sonnet", "claude-haiku"]  # ✓ With quotes
]

# WRONG: Trailing comma
rules = [
    ["claude-sonnet", "claude-haiku"],  # ✗ (sometimes issues)
]

# RIGHT:
rules = [
    ["claude-sonnet", "claude-haiku"]  # ✓ No trailing comma
]
```

Use a TOML validator to check syntax:
https://www.toml-lint.com/

## Next Steps

1. **[User Guide](./MODEL_OVERRIDE_USER_GUIDE.md)** - For using overrides
2. **[Operator Guide](./MODEL_OVERRIDE_OPERATOR_GUIDE.md)** - For deploying and managing
3. **[Cost Analysis](./COST_ANALYSIS.md)** - To calculate savings
4. **[API Documentation](./API.md)** - For monitoring integration

---

**Configuration File Ready?** Use this reference to set up your overrides, then restart CCO to load the new configuration.
