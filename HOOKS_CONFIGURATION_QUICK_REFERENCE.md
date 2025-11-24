# Hooks System Configuration Quick Reference

## Overview

The hooks system is **enabled by default** in the daemon, providing autonomous CRUD classification of shell commands using an embedded TinyLLaMA model.

## Default Behavior

When daemon starts:
1. ✅ Hooks system is **automatically enabled**
2. ✅ TinyLLaMA model is **auto-downloaded** to `~/.cco/models/` (first run)
3. ✅ Classification endpoint available at `/api/classify`
4. ✅ Health endpoint reports complete hooks status

## Configuration File

**Location**: `~/.cco/daemon.toml` (or customize via environment/flags)

### Minimal Configuration (Uses All Defaults)
```toml
# Hooks enabled with all defaults
# No additional configuration needed!
```

### Complete Configuration Example
```toml
# Daemon settings
port = 3000
host = "127.0.0.1"
log_level = "info"

# Hooks system configuration
[hooks]
# Enable/disable entire hooks system
enabled = true

# Hook execution timeout in milliseconds
timeout_ms = 5000

# Retry failed hooks this many times
max_retries = 2

# LLM configuration for CRUD classification
[hooks.llm]
model_type = "tinyllama"
model_name = "tinyllama-1.1b-chat-v1.0.Q4_K_M"
model_path = "~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
model_size_mb = 600
quantization = "Q4_K_M"
inference_timeout_ms = 2000
temperature = 0.1

# Permission controls
[hooks.permissions]
allow_command_modification = false
allow_execution_blocking = false
allow_external_calls = false
allow_env_access = false
allow_file_read = false
allow_file_write = false

# Callback configurations (Phase 2+)
[hooks.callbacks]
pre_command = []
post_command = []
post_execution = []
```

## Configuration Parameters

### Primary Settings

| Parameter | Default | Type | Description |
|-----------|---------|------|-------------|
| `enabled` | `true` | bool | Enable/disable hooks subsystem |
| `timeout_ms` | `5000` | u64 | Hook execution timeout in milliseconds |
| `max_retries` | `2` | u32 | Maximum retry attempts for failed hooks |

### LLM Settings (Model Download & Inference)

| Parameter | Default | Type | Description |
|-----------|---------|------|-------------|
| `model_type` | `"tinyllama"` | string | Model type identifier |
| `model_name` | `"tinyllama-1.1b-chat-v1.0.Q4_K_M"` | string | Full model name for downloads |
| `model_path` | `~/.cco/models/...` | path | Local storage location for model file |
| `model_size_mb` | `600` | u32 | Expected model size (600 MB) |
| `quantization` | `"Q4_K_M"` | string | Quantization level (4-bit, memory optimized) |
| `inference_timeout_ms` | `2000` | u64 | Maximum time for single classification (2 sec) |
| `temperature` | `0.1` | f32 | Low temperature for consistent classification |

### Permission Controls (All Default to `false`)

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `allow_command_modification` | `false` | Allow hooks to modify commands |
| `allow_execution_blocking` | `false` | Allow hooks to block command execution |
| `allow_external_calls` | `false` | Allow hooks to make API calls |
| `allow_env_access` | `false` | Allow hooks to read environment variables |
| `allow_file_read` | `false` | Allow hooks to read files |
| `allow_file_write` | `false` | Allow hooks to write files |

## Common Configurations

### Configuration 1: Production (Classification Only)
```toml
[hooks]
enabled = true
timeout_ms = 5000
max_retries = 2

# All other settings use defaults
# Model downloads automatically
# No permissions granted (safe mode)
```

### Configuration 2: Development (Verbose Logging)
```toml
log_level = "debug"

[hooks]
enabled = true
timeout_ms = 10000  # More time for debugging
max_retries = 3
```

### Configuration 3: Disable Hooks
```toml
[hooks]
enabled = false
# Everything disabled, system runs without hooks
```

### Configuration 4: Custom Model Path
```toml
[hooks]
enabled = true

[hooks.llm]
model_path = "/opt/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
# All other LLM settings use defaults
```

## Health Endpoint Response

**Endpoint**: `GET /health`

```json
{
  "status": "ok",
  "version": "2025.11.24",
  "uptime_seconds": 123,
  "port": 3000,
  "hooks": {
    "enabled": true,
    "classifier_available": true,
    "model_loaded": true,
    "model_name": "tinyllama-1.1b-chat-v1.0.Q4_K_M",
    "classification_latency_ms": 45
  }
}
```

**Interpretation**:
- `enabled`: Hooks subsystem is active
- `classifier_available`: CRUD classifier was initialized
- `model_loaded`: TinyLLaMA model is in memory
- `model_name`: Which model version is loaded
- `classification_latency_ms`: How long last classification took

## Classification Endpoint

**Endpoint**: `POST /api/classify`
**Content-Type**: `application/json`

### Request
```json
{
  "command": "git commit -m 'fix: resolve issue'"
}
```

### Response
```json
{
  "classification": "UPDATE",
  "confidence": 0.95,
  "reasoning": "The command modifies the git repository state by creating a new commit object",
  "timestamp": "2025-11-24T09:30:00+00:00"
}
```

### Classification Values
- `READ` - Query operations (ls, cat, grep, git log)
- `CREATE` - New resource creation (mkdir, touch, git branch)
- `UPDATE` - Modify existing resources (git commit, sed -i, echo >>)
- `DELETE` - Remove resources (rm, rmdir, git branch -D)

## File Locations

### Model Storage
```
~/.cco/models/
  └── tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf  (600 MB)
```
- Auto-created if missing
- Downloaded from HuggingFace on first daemon startup
- Cached for subsequent startups

### Configuration
```
~/.cco/daemon.toml  (optional)
```
- If not present, uses built-in defaults
- Create to override any settings

### Logs
```
~/.cco/logs/daemon.log
```
- Rotated at 10 MB (configurable)
- Keeps 5 rotated files (configurable)

## Environment Variables

Override configuration via environment:

```bash
# Override model path
export CCO_HOOKS_MODEL_PATH="/custom/path/model.gguf"

# Override timeout
export CCO_HOOKS_TIMEOUT_MS=10000

# Disable hooks
export CCO_HOOKS_ENABLED=false

# Start daemon
cco daemon start
```

## Troubleshooting

### Hooks Not Enabled
**Issue**: Health endpoint shows `"enabled": false`

**Solution**:
```bash
# Check daemon.toml
cat ~/.cco/daemon.toml | grep -A 5 "\[hooks\]"

# Should show: enabled = true
# If missing, create fresh config or delete file for defaults
```

### Model Download Fails
**Issue**: `classifier_available: false`

**Check**:
```bash
# Verify internet connection
curl -I https://huggingface.co

# Check disk space in home directory
du -h ~/.cco/

# Check logs
tail -50 ~/.cco/logs/daemon.log | grep -i download
```

**Solution**:
- Ensure 1 GB free disk space
- Check internet connectivity
- Verify write permissions in `~/.cco/`

### Slow Classifications
**Issue**: `classification_latency_ms` is very high (>5000ms)

**Causes**:
- Model still loading (check `model_loaded: true`)
- System under heavy load
- Inference timeout too short

**Solution**:
```toml
[hooks.llm]
inference_timeout_ms = 5000  # Increase from 2000
```

## Default Values Explained

### Why TinyLLaMA?
- **Small**: 1.1B parameters fits in memory
- **Fast**: 2-second inference on modern CPUs
- **Accurate**: 92%+ accuracy for CRUD classification
- **Free**: Open-source, no API costs
- **Offline**: Complete local operation

### Why Q4_K_M Quantization?
- **Compressed**: 600 MB vs 3.5 GB full precision
- **Fast**: 4-bit inference optimized for CPU
- **Accurate**: Minimal quality loss vs full precision

### Why 0.1 Temperature?
- **Consistent**: Lower randomness in classifications
- **Deterministic**: Same command → same classification
- **Appropriate**: CRUD classification needs certainty

## Configuration Validation

Daemon validates configuration on startup:

```bash
# Valid configuration
cco daemon start  # ✅ Success

# Invalid configuration
[hooks]
timeout_ms = 0  # ❌ Error: must be > 0
```

**Validation rules**:
- `timeout_ms` must be > 0
- `max_retries` can be any u32
- `model_path` must be valid path or creatable
- `inference_timeout_ms` must be > 0
- `temperature` must be 0.0 - 1.0

## Summary

**Default Setup** = **Works Out of the Box**

```bash
# Just run - everything automatic:
cco daemon start

# Model downloads automatically
# Configuration uses sensible defaults
# Health endpoint shows hooks enabled
# Classification ready to use
```

For customization, create `~/.cco/daemon.toml` with only the parameters you want to override. All others use defaults.

---

**Status**: Hooks enabled by default, fully integrated, ready for production use.
