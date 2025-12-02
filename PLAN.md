# Plan: Merge Orchestration Sidecar into Daemon

## Goal
Eliminate the separate orchestration sidecar (port 3001) by merging its functionality into the existing daemon server (random port).

## Current State
- **Daemon**: Random port, handles hooks/classification/knowledge
- **Sidecar**: Fixed port 3001, handles agent coordination/events/context/results
- **Problem**: Two processes, port collision risk, extra lifecycle management

## Target State
- **Single daemon process** with all functionality
- Random port discovery (already works for hooks)
- Unified `/api/orchestration/*` routes

---

## Phase 1: Add Orchestration Routes to Daemon

### 1.1 Create orchestration routes module
- [ ] Create `src/daemon/orchestration_routes.rs`
- [ ] Move handler functions from `src/orchestration/server.rs`
- [ ] Adapt handlers to use `DaemonState` instead of `HandlerState`

### 1.2 Add OrchestrationState to DaemonState
- [ ] Add `orchestration: Option<Arc<OrchestrationState>>` to `DaemonState`
- [ ] Initialize orchestration components in daemon startup
- [ ] Wire up to existing config system

### 1.3 Mount routes in daemon router
- [ ] Add orchestration routes under `/api/orchestration/*` prefix
- [ ] Update health endpoint to include orchestration status
- [ ] Test all 8 endpoints work via daemon port

**Endpoints to migrate:**
| Old Route | New Route |
|-----------|-----------|
| GET `/health` | Merge into daemon `/health` |
| GET `/status` | GET `/api/orchestration/status` |
| GET `/api/context/:issue_id/:agent_type` | GET `/api/orchestration/context/:issue_id/:agent_type` |
| POST `/api/results` | POST `/api/orchestration/results` |
| POST `/api/events/:event_type` | POST `/api/orchestration/events/:event_type` |
| GET `/api/events/wait/:event_type` | GET `/api/orchestration/events/wait/:event_type` |
| POST `/api/agents/spawn` | POST `/api/orchestration/agents/spawn` |
| GET `/api/agents/:agent_id/status` | GET `/api/orchestration/agents/:agent_id/status` |
| DELETE `/api/cache/context/:issue_id` | DELETE `/api/orchestration/cache/context/:issue_id` |

---

## Phase 2: Update Launcher

### 2.1 Remove sidecar lifecycle management
- [ ] Remove `SidecarHandle` and `SIDECAR_HANDLE` global
- [ ] Remove `start_orchestration_sidecar()` call
- [ ] Remove `shutdown_sidecar()` call
- [ ] Remove `set_sidecar_env_vars()` - use daemon URL instead

### 2.2 Update environment variables
- [ ] Update `ORCHESTRATOR_API_URL` to include orchestration routes
- [ ] Remove any `ORCHESTRATION_SIDECAR_*` env vars
- [ ] Verify Claude Code hooks still work

---

## Phase 3: Cleanup

### 3.1 Remove sidecar server code
- [ ] Delete `src/orchestration/server.rs` (handlers moved to daemon)
- [ ] Keep component modules: `event_bus.rs`, `context_injector.rs`, `result_storage.rs`, `knowledge_broker.rs`
- [ ] Update `src/orchestration/mod.rs` to remove server exports

### 3.2 Remove CLI commands
- [ ] Remove `cco orchestration-server` command
- [ ] Update help text

### 3.3 Update documentation
- [ ] Update docs referencing port 3001
- [ ] Update architecture diagrams

---

## Files to Modify

**Create:**
- `src/daemon/orchestration_routes.rs` - new routes module

**Modify:**
- `src/daemon/mod.rs` - add orchestration_routes module
- `src/daemon/server.rs` - add OrchestrationState, mount routes
- `src/commands/launcher.rs` - remove sidecar lifecycle
- `src/orchestration/mod.rs` - remove server re-exports
- `src/main.rs` - remove orchestration-server command

**Delete:**
- `src/orchestration/server.rs` - after handlers migrated

---

## Testing Checklist

- [ ] `cco` launches without sidecar message
- [ ] Daemon `/health` includes orchestration status
- [ ] All orchestration endpoints respond on daemon port
- [ ] Event bus works (publish/subscribe)
- [ ] Context injection works
- [ ] Result storage works
- [ ] No port 3001 in use after launch
- [ ] Claude Code hooks still connect correctly

---

## Rollback Plan

If issues arise:
1. Revert commits
2. Sidecar code preserved in git history
3. Can re-enable by reverting launcher changes
