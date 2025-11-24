# Phase 1A Hooks - Quick Reference

## Test Results

✅ **61 tests passed** (20.22s)
- Config tests: 13 passed
- Executor tests: 18 passed
- Registry tests: 12 passed
- Types tests: 13 passed
- LLM tests: 5 passed

## Key Files

| File | Lines | Purpose |
|------|-------|---------|
| `src/daemon/hooks/types.rs` | 456 | CRUD classification, hook types, payloads |
| `src/daemon/hooks/error.rs` | 237 | Error types and handling |
| `src/daemon/hooks/config.rs` | 417 | Configuration structures |
| `src/daemon/hooks/registry.rs` | 357 | Thread-safe hook storage |
| `src/daemon/hooks/executor.rs` | 529 | Async execution engine |
| `src/daemon/hooks/llm/prompt.rs` | 240 | Prompt engineering |
| `src/daemon/hooks/llm/model.rs` | 315 | Model management |
| `src/daemon/hooks/llm/classifier.rs` | 283 | CRUD classifier |

**Total**: 2,834 lines + tests

## Configuration Example

### TOML (`~/.cco/config.toml`)

```toml
[hooks]
enabled = true
timeout_ms = 5000
max_retries = 2

[hooks.llm]
model_type = "tinyllama"
model_name = "tinyllama-1.1b-chat-v1.0.Q4_K_M"
model_path = "~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
model_size_mb = 600
quantization = "Q4_K_M"
inference_timeout_ms = 2000
temperature = 0.1

[hooks.permissions]
allow_command_modification = false
allow_execution_blocking = false
allow_external_calls = false
```

## API Quick Reference

### Creating Hooks System

```rust
use cco::daemon::hooks::{HookRegistry, HookExecutor};
use std::sync::Arc;

let registry = Arc::new(HookRegistry::new());
let executor = HookExecutor::new(registry.clone());
```

### Registering a Hook

```rust
use cco::daemon::hooks::{HookType, HookPayload, HookResult};

registry.register(HookType::PreCommand, Box::new(|payload: &HookPayload| -> HookResult<()> {
    println!("Command: {}", payload.command);
    Ok(())
}))?;
```

### Executing Hooks

```rust
use cco::daemon::hooks::HookPayload;

let payload = HookPayload::new("ls -la");
executor.execute_hook(HookType::PreCommand, payload).await?;
```

### CRUD Classification

```rust
use cco::daemon::hooks::{CrudClassification, ClassificationResult};

let result = ClassificationResult::new(CrudClassification::Read, 0.95);
println!("Classification: {} (confidence: {})", result.classification, result.confidence);
```

### With Classification

```rust
let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
let payload = HookPayload::with_classification("git status", classification);
executor.execute_hook(HookType::PostCommand, payload).await?;
```

## Hook Types

### PreCommand
- **When**: Before command classification
- **Use**: Preprocessing, security checks, rate limiting
- **Payload**: Command only

### PostCommand
- **When**: After classification, before execution
- **Use**: Validation, permission checks, logging
- **Payload**: Command + classification

### PostExecution
- **When**: After command completes
- **Use**: Result logging, cleanup, metrics
- **Payload**: Command + classification + result

## CRUD Classifications

| Classification | Description | Examples |
|---------------|-------------|----------|
| **READ** | No side effects | `ls`, `cat`, `git status`, `ps` |
| **CREATE** | New resources | `touch`, `mkdir`, `git init`, `docker run` |
| **UPDATE** | Modify existing | `echo >>`, `sed -i`, `git commit`, `chmod` |
| **DELETE** | Remove resources | `rm`, `rmdir`, `docker rm`, `git clean` |

## Error Types

| Error | Retryable | Description |
|-------|-----------|-------------|
| `Timeout` | ✅ Yes | Hook exceeded timeout |
| `ExecutionFailed` | ✅ Yes | Hook returned error |
| `LlmUnavailable` | ✅ Yes | LLM service down |
| `PanicRecovery` | ❌ No | Hook panicked |
| `InvalidConfig` | ❌ No | Bad configuration |
| `MaxRetriesExceeded` | ❌ No | All retries failed |

## Timeout Configuration

```rust
use std::time::Duration;

// Default timeout (5 seconds)
let executor = HookExecutor::new(registry.clone());

// Custom timeout (10 seconds)
let executor = HookExecutor::with_timeout(registry.clone(), Duration::from_secs(10));

// Custom timeout + retries
let executor = HookExecutor::with_config(
    registry.clone(),
    Duration::from_secs(10),
    5  // max retries
);
```

## Daemon Integration

### Config File
- Location: `~/.cco/config.toml`
- Section: `[hooks]`
- Default: Disabled for security

### Temp Settings
- Location: `/tmp/.cco-orchestrator-settings`
- Format: JSON
- Created: On daemon start
- Cleaned: On daemon stop

### Initialization

```rust
use cco::daemon::{DaemonConfig, DaemonManager};

let config = DaemonConfig::default();
let manager = DaemonManager::new(config);

// Registry and executor are available
let registry = manager.hooks_registry;
let executor = manager.hooks_executor;
```

## Phase 1B Checklist

### To Add (llm crate integration):
- [ ] Add `llm = "0.1"` to Cargo.toml
- [ ] Add `indicatif = "0.17"` for progress bars
- [ ] Implement model download in `model.rs`
- [ ] Implement SHA256 verification
- [ ] Implement GGML inference in `run_inference()`
- [ ] Test with real TinyLLaMA model
- [ ] Update `loaded` flag when model loads

### Current Placeholders:
- `ModelManager::download_model()` - Returns error
- `ModelManager::run_inference()` - Returns error
- `ModelManager::verify_model_hash()` - No-op
- Model storage: `Arc<Mutex<Option<()>>>` (will be `LlmModel`)

## Testing Commands

```bash
# Run all hooks tests
cargo test hooks --lib

# Run specific test module
cargo test hooks::config --lib
cargo test hooks::executor --lib
cargo test hooks::registry --lib

# Run with output
cargo test hooks --lib -- --nocapture

# Check compilation
cargo check
```

## Common Patterns

### Safe Hook Registration

```rust
let hook: Box<dyn Hook> = Box::new(move |payload: &HookPayload| -> HookResult<()> {
    // Hook logic here
    // Return Ok(()) on success
    // Return Err(HookError::...) on failure
    Ok(())
});

registry.register(HookType::PreCommand, hook)?;
```

### Accessing Classification

```rust
if let Some(classification) = &payload.classification {
    match classification.classification {
        CrudClassification::Read => {
            // Safe operation, auto-allow
        }
        CrudClassification::Create |
        CrudClassification::Update |
        CrudClassification::Delete => {
            // Requires confirmation
        }
    }
}
```

### Adding Metadata

```rust
let payload = HookPayload::new("command")
    .with_metadata("user", "alice")
    .with_metadata("session", "12345")
    .with_metadata("source", "api");

if let Some(user) = payload.get_metadata("user") {
    println!("User: {}", user);
}
```

### Custom Context

```rust
use cco::daemon::hooks::HookContext;
use std::collections::HashMap;

let mut metadata = HashMap::new();
metadata.insert("environment".to_string(), "production".to_string());

let context = HookContext::Daemon { metadata };
let payload = HookPayload::new("command").with_context(context);
```

## Performance Notes

- **Hook Execution**: Async, non-blocking
- **Timeout**: 5s default (configurable)
- **Retry Delay**: 100ms between retries
- **Model Inference**: 2s timeout (Phase 1B)
- **Lock Contention**: RwLock minimizes blocking
- **Panic Safe**: Caught with `catch_unwind`

## Security Notes

1. **Default Disabled**: Opt-in activation
2. **Permissions**: All off by default
3. **Timeout**: Prevents runaway hooks
4. **Isolation**: Panics don't crash daemon
5. **Fallback**: CREATE on error (safest)
6. **No Network**: Embedded model only

## Documentation

All public APIs documented with:
- Module-level architecture overview
- Function documentation
- Example code
- Safety notes
- Error conditions

## Version Info

- **Phase**: 1A (Foundation)
- **Status**: Complete
- **Tests**: 61 passing
- **Lines**: 2,834 production + tests
- **Dependencies**: All existing (no new deps yet)
- **Next**: Phase 1B (LLM integration)
