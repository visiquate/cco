# Hooks Configuration Guide

**Version**: 1.0.0
**Last Updated**: November 17, 2025
**Status**: Complete (Phases 2-5)

## Table of Contents

1. [Overview](#overview)
2. [Configuration File Locations](#configuration-file-locations)
3. [Complete Configuration Structure](#complete-configuration-structure)
4. [Configuration Reference](#configuration-reference)
5. [Environment Variables](#environment-variables)
6. [Example Configurations](#example-configurations)
7. [Validation and Error Handling](#validation-and-error-handling)
8. [Performance Tuning](#performance-tuning)
9. [Troubleshooting Configuration](#troubleshooting-configuration)

## Overview

The hooks system is configured through the CCO orchestrator settings file, which is generated in the system temp directory. Configuration is stored in JSON format and can be customized using environment variables.

**Key Concepts:**
- **Settings file** generated on daemon startup
- **JSON format** with hierarchical structure
- **Environment variable overrides** for each setting
- **Hot reload** not supported (restart daemon to apply changes)
- **Validation** performed on startup

## Configuration File Locations

### Primary Location

The hooks configuration is stored in the main CCO settings file:

**Unix/Linux/macOS:**
```
/tmp/.cco-orchestrator-settings
```

**Windows:**
```
%TEMP%\.cco-orchestrator-settings
```

**Alternative (if main location fails):**
```
$HOME/.cco/settings.json
```

### Minimal Example

Even with no explicit configuration, the system generates defaults:

```bash
cat /tmp/.cco-orchestrator-settings | jq '.hooks'
```

Output:
```json
{
  "hooks": {
    "enabled": true,
    "llm": {
      "model_type": "tinyllama",
      "model_name": "tinyllama-1.1b-chat-v1.0.Q4_K_M",
      "model_path": "~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
      "loaded": false,
      "inference_timeout_ms": 2000,
      "temperature": 0.1,
      "max_tokens": 10,
      "top_k": 40,
      "top_p": 0.95,
      "repeat_penalty": 1.1
    },
    "permissions": {
      "auto_allow_read": true,
      "require_confirmation_cud": true,
      "dangerously_skip_confirmations": false
    },
    "audit": {
      "enabled": true,
      "db_path": "~/.cco/hooks/audit.db",
      "retention_days": 30,
      "auto_cleanup": true
    },
    "active_hooks": ["command_classifier"]
  }
}
```

## Complete Configuration Structure

### Full Schema with All Options

```json
{
  "version": "2025.11.2",
  "orchestration": {
    "enabled": true,
    "api_url": "http://localhost:3000"
  },
  "hooks": {
    "enabled": true,
    "llm": {
      "model_type": "tinyllama",
      "model_name": "tinyllama-1.1b-chat-v1.0.Q4_K_M",
      "model_path": "~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
      "model_size": "600MB",
      "quantization": "Q4_K_M",
      "loaded": false,
      "download_url": "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
      "inference_timeout_ms": 2000,
      "temperature": 0.1,
      "max_tokens": 10,
      "top_k": 40,
      "top_p": 0.95,
      "repeat_penalty": 1.1,
      "cache_embeddings": true
    },
    "permissions": {
      "auto_allow_read": true,
      "require_confirmation_cud": true,
      "dangerously_skip_confirmations": false
    },
    "audit": {
      "enabled": true,
      "db_path": "~/.cco/hooks/audit.db",
      "retention_days": 30,
      "auto_cleanup": true,
      "max_records": 100000
    },
    "active_hooks": [
      "command_classifier"
    ],
    "callbacks": {
      "pre_command": ["classify_crud"],
      "post_command": [],
      "post_execution": []
    },
    "logging": {
      "level": "info",
      "format": "json",
      "output": "file",
      "file_path": "~/.cco/hooks/hooks.log"
    },
    "performance": {
      "cache_decisions": true,
      "cache_ttl_seconds": 3600,
      "max_cache_size": 10000,
      "batch_mode": false,
      "batch_size": 10
    },
    "advanced": {
      "model_fallback_timeout_ms": 5000,
      "pattern_cache_enabled": true,
      "confidence_threshold": 0.5,
      "explain_decisions": true
    }
  }
}
```

## Configuration Reference

### hooks.enabled

**Type**: Boolean
**Default**: true
**Dynamic**: No (requires daemon restart)
**Environment Variable**: `CCO_HOOKS_ENABLED`

Enable or disable the entire hooks system.

```json
{
  "hooks": {
    "enabled": true
  }
}
```

```bash
# Via environment variable
export CCO_HOOKS_ENABLED=false
```

**Values:**
- `true` - Hooks active, commands are classified and confirmed
- `false` - Hooks inactive, commands execute immediately

**When to disable:**
- Testing without hooks interference
- Emergency situations
- Low-resource environments

---

### hooks.llm (Language Model Configuration)

#### model_type

**Type**: String
**Default**: "tinyllama"
**Options**: "tinyllama", "phi2" (future)
**Environment**: `CCO_LLM_MODEL_TYPE`

Which language model to use for classification.

```json
{
  "hooks": {
    "llm": {
      "model_type": "tinyllama"
    }
  }
}
```

---

#### model_name

**Type**: String
**Default**: "tinyllama-1.1b-chat-v1.0.Q4_K_M"
**Environment**: `CCO_LLM_MODEL_NAME`

Specific model variant to use.

```json
{
  "hooks": {
    "llm": {
      "model_name": "tinyllama-1.1b-chat-v1.0.Q4_K_M"
    }
  }
}
```

---

#### model_path

**Type**: String (file path)
**Default**: "~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
**Environment**: `CCO_LLM_MODEL_PATH`

Where the model file is stored locally.

```json
{
  "hooks": {
    "llm": {
      "model_path": "~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
    }
  }
}
```

**Notes:**
- `~` expands to home directory
- Directory created automatically if missing
- Model file (~600MB) downloaded on first use

---

#### inference_timeout_ms

**Type**: Integer (milliseconds)
**Default**: 2000
**Min**: 100
**Max**: 30000
**Environment**: `CCO_LLM_TIMEOUT`

Maximum time to wait for classification response.

```json
{
  "hooks": {
    "llm": {
      "inference_timeout_ms": 2000
    }
  }
}
```

**Tuning Guide:**

| Value | Use Case | Notes |
|-------|----------|-------|
| 500-1000 | Fast systems, SSD storage | May timeout on slower systems |
| 2000 | **Default** | Good balance for most systems |
| 3000-5000 | Older/slower systems | Reduces false timeouts |
| 10000+ | Very slow systems | Long pause between commands |

**What happens on timeout:**
1. Classification is fallback to CREATE (safest)
2. Confirmation requested (extra safe)
3. Error logged
4. User continues normally

---

#### temperature

**Type**: Float (0.0 - 1.0)
**Default**: 0.1
**Environment**: `CCO_LLM_TEMPERATURE`

How creative/random the model's responses are.

```json
{
  "hooks": {
    "llm": {
      "temperature": 0.1
    }
  }
}
```

**Tuning Guide:**

| Value | Behavior | Use Case |
|-------|----------|----------|
| 0.0 | Deterministic | Best for classification (always same result) |
| 0.1 | **Default** | Focused, reliable classifications |
| 0.5 | Balanced | Some variation, still reliable |
| 1.0 | Creative | Random results (bad for classification) |

**Recommendation:** Keep at 0.1 for consistent classification. Only increase if classifications feel too strict.

---

#### max_tokens

**Type**: Integer
**Default**: 10
**Environment**: `CCO_LLM_MAX_TOKENS`

Maximum tokens in classifier response (limit to one word).

```json
{
  "hooks": {
    "llm": {
      "max_tokens": 10
    }
  }
}
```

**Leave at default.** Classification needs only one word (READ/CREATE/UPDATE/DELETE).

---

#### top_k, top_p, repeat_penalty

**Type**: Float
**Default**: top_k=40, top_p=0.95, repeat_penalty=1.1
**Environment**: `CCO_LLM_TOP_K`, `CCO_LLM_TOP_P`, `CCO_LLM_REPEAT_PENALTY`

Advanced LLM sampling parameters.

```json
{
  "hooks": {
    "llm": {
      "top_k": 40,
      "top_p": 0.95,
      "repeat_penalty": 1.1
    }
  }
}
```

**Expert use only.** Default values are tuned for reliable classification. Don't change unless experiencing specific classification issues.

---

### hooks.permissions

#### auto_allow_read

**Type**: Boolean
**Default**: true
**Dynamic**: No (requires daemon restart)
**Environment**: `CCO_AUTO_ALLOW_READ`

Automatically allow READ operations without confirmation.

```json
{
  "hooks": {
    "permissions": {
      "auto_allow_read": true
    }
  }
}
```

**When true (default):**
- `ls`, `cat`, `git status`, etc. execute immediately
- Faster workflow (no interruptions for safe operations)

**When false:**
- All operations require confirmation
- Slower workflow but maximum safety
- For supervised/audited environments only

---

#### require_confirmation_cud

**Type**: Boolean
**Default**: true
**Dynamic**: No (requires daemon restart)
**Environment**: `CCO_REQUIRE_CONFIRMATION_CUD`

Require user confirmation for CREATE/UPDATE/DELETE operations.

```json
{
  "hooks": {
    "permissions": {
      "require_confirmation_cud": true
    }
  }
}
```

**When true (default):**
- CREATE, UPDATE, DELETE prompt for confirmation
- Safe default, protects against mistakes

**When false:**
- C/U/D operations execute immediately (dangerous!)
- Only for automation/CI-CD environments
- Requires thorough review before disabling

---

#### dangerously_skip_confirmations

**Type**: Boolean
**Default**: false
**Dynamic**: No (requires daemon restart)
**Environment**: `CCO_DANGEROUSLY_SKIP_CONFIRMATIONS`

**DANGER:** Override all permission checks and skip confirmations entirely.

```json
{
  "hooks": {
    "permissions": {
      "dangerously_skip_confirmations": false
    }
  }
}
```

**NEVER set to true in production.** Only for:
- Continuous integration/deployment
- Automated testing
- Fully controlled environments

**Consequences of enabling:**
- All operations execute immediately
- No user confirmation
- No safety net
- Audit trail still maintained (cannot disable)

```bash
export CCO_DANGEROUSLY_SKIP_CONFIRMATIONS=true
```

---

### hooks.audit

#### enabled

**Type**: Boolean
**Default**: true
**Dynamic**: No (requires daemon restart)
**Environment**: `CCO_AUDIT_ENABLED`

Enable or disable audit trail logging.

```json
{
  "hooks": {
    "audit": {
      "enabled": true
    }
  }
}
```

**Notes:**
- Audit trail is essential for compliance/security
- Disabling is not recommended
- Audit records are always created even if "disabled" (can't be disabled)

---

#### db_path

**Type**: String (file path)
**Default**: "~/.cco/hooks/audit.db"
**Environment**: `CCO_AUDIT_DB_PATH`

Where audit trail database is stored.

```json
{
  "hooks": {
    "audit": {
      "db_path": "~/.cco/hooks/audit.db"
    }
  }
}
```

**Notes:**
- SQLite database file
- Created automatically if missing
- ~50KB per 1000 decisions

---

#### retention_days

**Type**: Integer
**Default**: 30
**Min**: 1
**Max**: 365
**Environment**: `CCO_AUDIT_RETENTION_DAYS`

How long to keep audit records.

```json
{
  "hooks": {
    "audit": {
      "retention_days": 30
    }
  }
}
```

**Storage implications:**

| Value | Retention Period | Storage (~) |
|-------|------------------|------------|
| 7 | 1 week | 350KB |
| 30 | 1 month | 1.5MB |
| 90 | 3 months | 4.5MB |
| 365 | 1 year | 18MB |

---

#### auto_cleanup

**Type**: Boolean
**Default**: true
**Environment**: `CCO_AUDIT_AUTO_CLEANUP`

Automatically delete old records based on retention_days.

```json
{
  "hooks": {
    "audit": {
      "auto_cleanup": true
    }
  }
}
```

**When true:** Records older than retention_days deleted automatically (daily check)
**When false:** Manual cleanup required (see API reference)

---

### hooks.active_hooks

**Type**: Array of strings
**Default**: ["command_classifier"]
**Environment**: `CCO_ACTIVE_HOOKS` (comma-separated)

Which hook implementations to activate.

```json
{
  "hooks": {
    "active_hooks": [
      "command_classifier"
    ]
  }
}
```

**Currently available:**
- `command_classifier` - CRUD classification (always on for Phase 1C)

**Future (Phases 2-5):**
- `file_access` - Restrict file system access
- `network_access` - Restrict network operations
- `process_control` - Restrict process management
- `custom_rules` - User-defined rules

---

### hooks.logging

#### level

**Type**: String
**Default**: "info"
**Options**: "debug", "info", "warn", "error"
**Environment**: `CCO_LOG_LEVEL`

Logging verbosity.

```json
{
  "hooks": {
    "logging": {
      "level": "info"
    }
  }
}
```

**Levels:**
- `debug` - Very detailed (for troubleshooting)
- `info` - Standard (default)
- `warn` - Only problems (less noise)
- `error` - Only errors (minimum noise)

---

#### output

**Type**: String
**Default**: "file"
**Options**: "file", "stdout", "stderr", "none"
**Environment**: `CCO_LOG_OUTPUT`

Where to send logs.

```json
{
  "hooks": {
    "logging": {
      "output": "file"
    }
  }
}
```

---

### hooks.performance

#### cache_decisions

**Type**: Boolean
**Default**: true
**Environment**: `CCO_CACHE_DECISIONS`

Cache classification results for identical commands.

```json
{
  "hooks": {
    "performance": {
      "cache_decisions": true
    }
  }
}
```

**Benefits:**
- Identical commands classify instantly (cached)
- Reduces model inference load
- ~1KB per cached entry

---

#### cache_ttl_seconds

**Type**: Integer
**Default**: 3600 (1 hour)
**Min**: 60
**Max**: 86400
**Environment**: `CCO_CACHE_TTL`

How long to keep classification cache entries.

```json
{
  "hooks": {
    "performance": {
      "cache_ttl_seconds": 3600
    }
  }
}
```

---

### hooks.advanced

#### confidence_threshold

**Type**: Float (0.0 - 1.0)
**Default**: 0.5
**Environment**: `CCO_CONFIDENCE_THRESHOLD`

Minimum confidence to accept classification (else timeout fallback).

```json
{
  "hooks": {
    "advanced": {
      "confidence_threshold": 0.5
    }
  }
}
```

**Use cases:**
- `0.0` - Accept all classifications
- `0.5` - Default, balanced
- `0.9` - Very strict (many fallbacks)

---

## Environment Variables

### Complete Environment Variable List

| Variable | Type | Default | Section |
|----------|------|---------|---------|
| `CCO_HOOKS_ENABLED` | bool | true | hooks |
| `CCO_LLM_MODEL_TYPE` | string | tinyllama | llm |
| `CCO_LLM_MODEL_NAME` | string | tinyllama-1.1b-chat-v1.0.Q4_K_M | llm |
| `CCO_LLM_MODEL_PATH` | path | ~/.cco/models/... | llm |
| `CCO_LLM_TIMEOUT` | int | 2000 | llm |
| `CCO_LLM_TEMPERATURE` | float | 0.1 | llm |
| `CCO_LLM_MAX_TOKENS` | int | 10 | llm |
| `CCO_AUTO_ALLOW_READ` | bool | true | permissions |
| `CCO_REQUIRE_CONFIRMATION_CUD` | bool | true | permissions |
| `CCO_DANGEROUSLY_SKIP_CONFIRMATIONS` | bool | false | permissions |
| `CCO_AUDIT_ENABLED` | bool | true | audit |
| `CCO_AUDIT_DB_PATH` | path | ~/.cco/hooks/audit.db | audit |
| `CCO_AUDIT_RETENTION_DAYS` | int | 30 | audit |
| `CCO_AUDIT_AUTO_CLEANUP` | bool | true | audit |
| `CCO_ACTIVE_HOOKS` | csv | command_classifier | active_hooks |
| `CCO_LOG_LEVEL` | string | info | logging |
| `CCO_LOG_OUTPUT` | string | file | logging |
| `CCO_CACHE_DECISIONS` | bool | true | performance |
| `CCO_CACHE_TTL` | int | 3600 | performance |

### Setting via Environment Variable

```bash
export CCO_LLM_TIMEOUT=3000
export CCO_AUTO_ALLOW_READ=true
export CCO_AUDIT_RETENTION_DAYS=60

# Then restart daemon to apply changes
cco  # Automatically restarts daemon
```

---

## Example Configurations

### Example 1: Development (Default)

Best for normal development work. READ operations allowed, C/U/D require confirmation.

```json
{
  "hooks": {
    "enabled": true,
    "llm": {
      "model_type": "tinyllama",
      "inference_timeout_ms": 2000,
      "temperature": 0.1
    },
    "permissions": {
      "auto_allow_read": true,
      "require_confirmation_cud": true,
      "dangerously_skip_confirmations": false
    },
    "audit": {
      "enabled": true,
      "retention_days": 30,
      "auto_cleanup": true
    },
    "active_hooks": ["command_classifier"]
  }
}
```

**Characteristics:**
- Safe (confirms destructive operations)
- Fast (skips confirmation for reads)
- Audited (keeps 30 days of history)
- Recommended for most users

---

### Example 2: Production/Audited

Maximum safety with detailed audit trail. All operations confirmed.

```json
{
  "hooks": {
    "enabled": true,
    "llm": {
      "model_type": "tinyllama",
      "inference_timeout_ms": 5000,
      "temperature": 0.1
    },
    "permissions": {
      "auto_allow_read": false,
      "require_confirmation_cud": true,
      "dangerously_skip_confirmations": false
    },
    "audit": {
      "enabled": true,
      "retention_days": 365,
      "auto_cleanup": true
    },
    "active_hooks": ["command_classifier"],
    "logging": {
      "level": "info",
      "output": "file"
    }
  }
}
```

**Characteristics:**
- Very safe (confirms all operations)
- Slower (more confirmation prompts)
- Fully audited (1 year retention)
- For compliance/regulated environments

**Environment setup:**
```bash
export CCO_AUDIT_RETENTION_DAYS=365
export CCO_AUTO_ALLOW_READ=false
export CCO_LLM_TIMEOUT=5000
```

---

### Example 3: CI/CD Pipeline

Automated environment with no user interaction.

```json
{
  "hooks": {
    "enabled": true,
    "llm": {
      "model_type": "tinyllama",
      "inference_timeout_ms": 10000
    },
    "permissions": {
      "auto_allow_read": true,
      "require_confirmation_cud": true,
      "dangerously_skip_confirmations": true
    },
    "audit": {
      "enabled": true,
      "retention_days": 7,
      "auto_cleanup": true
    },
    "active_hooks": ["command_classifier"]
  }
}
```

**DANGER:** This disables confirmations. Only use in fully automated, reviewed environments.

**Environment setup:**
```bash
export CCO_DANGEROUSLY_SKIP_CONFIRMATIONS=true
export CCO_AUDIT_RETENTION_DAYS=7
export CCO_LLM_TIMEOUT=10000
```

---

### Example 4: Low-Resource Systems

Optimized for older/slower machines.

```json
{
  "hooks": {
    "enabled": true,
    "llm": {
      "model_type": "tinyllama",
      "inference_timeout_ms": 5000,
      "temperature": 0.05
    },
    "permissions": {
      "auto_allow_read": true,
      "require_confirmation_cud": true
    },
    "audit": {
      "enabled": true,
      "retention_days": 7,
      "auto_cleanup": true
    },
    "performance": {
      "cache_decisions": true,
      "cache_ttl_seconds": 7200
    }
  }
}
```

**Optimizations:**
- Longer timeout (5s instead of 2s)
- Shorter audit retention (less storage)
- Aggressive caching (fewer model runs)
- Lower temperature for stability

---

### Example 5: Disabled (Normal Mode)

Hooks disabled entirely.

```json
{
  "hooks": {
    "enabled": false
  }
}
```

**Environment setup:**
```bash
export CCO_HOOKS_ENABLED=false
```

---

## Validation and Error Handling

### Validation on Startup

The daemon validates configuration on startup:

```
✓ Hooks enabled
✓ Model path accessible (~/.cco/models/)
✓ Audit database writable (~/.cco/hooks/)
✓ Timeout >= 100ms (2000ms)
✓ Temperature in range 0.0-1.0 (0.1)
✓ Retention days in range 1-365 (30)
✗ ERROR: Invalid auto_allow_read value
```

### Common Configuration Errors

**Error: Invalid JSON syntax**
```
Error: Failed to parse hooks configuration: expected `:` at line 5
Fix: Check JSON syntax, use JSON validator tool
```

**Error: Model path not accessible**
```
Error: Model path not writable: ~/.cco/models/
Fix: Check directory permissions: chmod 755 ~/.cco/models/
```

**Error: Invalid timeout value**
```
Error: inference_timeout_ms must be >= 100 (got 50)
Fix: Set to valid value (e.g., 2000)
```

**Error: Unknown hook type**
```
Error: Unknown hook in active_hooks: 'file_access'
Fix: Only use hooks that exist (currently: command_classifier)
```

---

## Performance Tuning

### Optimize for Speed

Reduce classification time (faster response):

```json
{
  "hooks": {
    "llm": {
      "inference_timeout_ms": 1000,
      "temperature": 0.05
    },
    "performance": {
      "cache_decisions": true,
      "cache_ttl_seconds": 7200,
      "max_cache_size": 50000
    }
  }
}
```

### Optimize for Safety

Increase confidence requirements:

```json
{
  "hooks": {
    "permissions": {
      "auto_allow_read": false
    },
    "advanced": {
      "confidence_threshold": 0.95
    }
  }
}
```

### Optimize for Storage

Reduce audit footprint:

```json
{
  "hooks": {
    "audit": {
      "retention_days": 7,
      "auto_cleanup": true
    }
  }
}
```

---

## Troubleshooting Configuration

### Check Current Configuration

```bash
# View entire settings file
cat /tmp/.cco-orchestrator-settings | jq '.hooks'

# Check specific setting
cat /tmp/.cco-orchestrator-settings | jq '.hooks.permissions'

# Pretty print
cat /tmp/.cco-orchestrator-settings | jq '.'
```

### Verify Configuration Applied

After changing environment variables, restart daemon:

```bash
# Kill existing daemon
pkill cco

# Environment variables are read on daemon startup
export CCO_LLM_TIMEOUT=3000

# Restart daemon
cco  # Starts new daemon with new config
```

### Check if Hooks Are Running

```bash
# Health check
curl http://localhost:3000/health | jq '.hooks'

# Should show:
{
  "enabled": true,
  "classifier_available": true,
  "model_loaded": false
}
```

### Model Download Issues

If model fails to download on first run:

```bash
# Manual download
cd ~/.cco/models/
curl -o tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
  "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"

# Verify file exists
ls -lh ~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf
```

---

**Last Updated**: November 17, 2025
**Version**: 1.0.0
**Status**: Complete for Phases 2-5
