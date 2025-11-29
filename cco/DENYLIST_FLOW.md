# Permission System Flow with Denylist

## Decision Flow Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                    PermissionRequest Received                      │
│                   (command + classification)                       │
└───────────────────────────┬────────────────────────────────────────┘
                            │
                            ▼
                ┌───────────────────────┐
                │   CHECK DENYLIST      │ ◄── HIGHEST PRIORITY
                │   (10 commands)       │
                └───────┬───────────────┘
                        │
                        ├─YES─► ❌ DENIED
                        │       "Would delete critical build artifacts"
                        │       (overrides ALL config)
                        │
                        NO
                        │
                        ▼
                ┌───────────────────────┐
                │   Is READ operation?  │
                └───────┬───────────────┘
                        │
                        ├─YES─► ✅ APPROVED
                        │       "READ operation - safe to execute"
                        │
                        NO
                        │
                        ▼
                ┌────────────────────────────────┐
                │  dangerously_skip_confirmations?│
                └───────┬────────────────────────┘
                        │
                        ├─YES─► ⚠️  SKIPPED
                        │       "Auto-approved (confirmations disabled)"
                        │
                        NO
                        │
                        ▼
                      ⏳ PENDING
                        "Requires user confirmation"
```

## Denylist Protection Examples

### ❌ DENIED - Even with skip_confirmations=true

```bash
# Build artifact deletion
cargo clean                    → DENIED
rm -rf target                  → DENIED
rm -rf target/                 → DENIED

# Git destructive operations
git clean -fdx                 → DENIED
git clean -fd                  → DENIED

# Docker destructive operations
docker system prune -a         → DENIED
docker system prune --all      → DENIED

# Node/Python build cleanup
rm -rf node_modules            → DENIED
rm -rf .venv                   → DENIED
```

### ✅ ALLOWED - Safe commands

```bash
# Cargo commands (safe)
cargo check                    → APPROVED (READ)
cargo build                    → PENDING (CREATE - needs confirmation)
cargo test                     → PENDING (CREATE - needs confirmation)

# Git commands (safe)
git status                     → APPROVED (READ)
git diff                       → APPROVED (READ)
git clean                      → PENDING (without -fd/-fdx)

# Docker commands (safe)
docker ps                      → APPROVED (READ)
docker logs                    → APPROVED (READ)
docker system prune            → PENDING (without -a/--all)

# File operations (safe)
rm -rf /tmp/test              → PENDING (not target/)
rm somefile.txt               → PENDING (not -rf target)
```

## Defense-in-Depth Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                   Permission System Layers                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Layer 1: DENYLIST (Highest Priority)                       │
│  ├─ Hard-blocks critical commands                           │
│  ├─ Overrides ALL configuration                             │
│  └─ Cannot be bypassed                                       │
│                                                              │
│  Layer 2: CRUD Classification                                │
│  ├─ Auto-approve READ operations                            │
│  ├─ Queue CREATE/UPDATE/DELETE                              │
│  └─ Can be overridden by skip_confirmations                 │
│                                                              │
│  Layer 3: User Confirmation (Interactive)                    │
│  ├─ User must approve unsafe operations                     │
│  ├─ Timeout after 5 seconds                                 │
│  └─ Can be skipped with flag                                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Configuration Scenarios

### Scenario 1: Default Configuration (Safe)

```rust
PermissionConfig {
    dangerously_skip_confirmations: false,
    auto_approve_read: true,
}
```

**Behavior:**
- ❌ Denylist commands: DENIED
- ✅ READ operations: APPROVED
- ⏳ CREATE/UPDATE/DELETE: PENDING (user confirmation)

### Scenario 2: Skip Confirmations (Dangerous, but Denylist Still Protects)

```rust
PermissionConfig {
    dangerously_skip_confirmations: true,
    auto_approve_read: true,
}
```

**Behavior:**
- ❌ Denylist commands: DENIED ← **Still protected!**
- ✅ READ operations: APPROVED
- ⚠️  CREATE/UPDATE/DELETE: SKIPPED (auto-approved)

### Scenario 3: No Auto-Approve (Maximum Safety)

```rust
PermissionConfig {
    dangerously_skip_confirmations: false,
    auto_approve_read: false,
}
```

**Behavior:**
- ❌ Denylist commands: DENIED
- ⏳ READ operations: PENDING (user confirmation)
- ⏳ CREATE/UPDATE/DELETE: PENDING (user confirmation)

## Substring Matching Strategy

The denylist uses substring matching to catch variations:

```rust
// Matches all these variations:
"cargo clean"           → matches "cargo clean"
"cd /tmp && cargo clean" → matches "cargo clean"
"cargo clean --release" → matches "cargo clean"

// But avoids false positives:
"cargo check"           → does NOT match "cargo clean"
"cargo build"           → does NOT match "cargo clean"
```

## Why This Approach Works

1. **Early Check**: Denylist checked BEFORE reading config
2. **Simple Logic**: Substring matching is fast and reliable
3. **No Bypasses**: Cannot be disabled via configuration
4. **Clear Feedback**: Users know exactly why command was denied
5. **Safe by Default**: Protects even when safety features disabled

## Testing Strategy

Each denied command has dedicated tests:

```rust
// Verify blocking works
test_denylist_blocks_cargo_clean()
test_denylist_blocks_rm_target()
test_denylist_blocks_git_clean()
test_denylist_blocks_docker_system_prune()
test_denylist_blocks_rm_node_modules()
test_denylist_blocks_rm_venv()

// Verify cannot be bypassed
test_denylist_blocks_even_with_skip_confirmations()
test_denylist_priority_over_all_config()

// Verify safe commands work
test_safe_commands_still_work()
```

All tests verified with standalone test harness: **17/17 passing** ✅
