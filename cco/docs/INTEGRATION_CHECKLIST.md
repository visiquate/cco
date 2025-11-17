# Integration Checklist
## Agent Embedding Implementation Tasks

**Target Completion**: 2-3 hours of development time
**Risk Level**: Low (non-breaking changes)

---

## Pre-Implementation Checklist

### Prerequisites ✓
- [ ] Verify 117 agent files exist in `~/.claude/agents/`
- [ ] Confirm all agent files have valid YAML frontmatter
- [ ] Ensure git repository has at least 1MB free space
- [ ] Back up existing agent files
- [ ] Review and approve the recommended implementation approach

### Environment Setup
- [ ] Rust toolchain installed (1.70+)
- [ ] cargo-watch installed (optional, for auto-rebuild)
- [ ] Development environment ready

---

## Phase 1: File Migration (15 minutes)

### 1.1 Create Directory Structure
```bash
- [ ] mkdir -p cco/config/agents
- [ ] mkdir -p cco/docs  # If not exists
```

### 1.2 Copy Agent Files
```bash
- [ ] cp ~/.claude/agents/*.md cco/config/agents/
- [ ] ls cco/config/agents/*.md | wc -l  # Verify 117 files
```

### 1.3 Validate Agent Files
```bash
- [ ] for f in cco/config/agents/*.md; do
        head -1 "$f" | grep -q "^---" || echo "Missing frontmatter: $f"
      done
```

### 1.4 Git Configuration
```bash
- [ ] Ensure config/agents/ is NOT in .gitignore
- [ ] git add cco/config/agents/
- [ ] git status  # Verify 117 new files staged
```

---

## Phase 2: Code Implementation (60 minutes)

### 2.1 Update Dependencies
**File**: `cco/Cargo.toml`
```toml
- [ ] Add to [dependencies]:
      once_cell = "1.19"

- [ ] Add to [build-dependencies]:
      walkdir = "2.4"
```

### 2.2 Enhance Build Script
**File**: `cco/build.rs`
```rust
- [ ] Add use statements:
      use walkdir::WalkDir;

- [ ] Add embed_agent_definitions() function
- [ ] Call embed_agent_definitions() from main()
- [ ] Implement validate_agent_yaml() helper
- [ ] Add rerun-if-changed for config/agents/
```

### 2.3 Update Agent Configuration Module
**File**: `cco/src/agents_config.rs`
```rust
- [ ] Add use statement:
      use once_cell::sync::Lazy;

- [ ] Add include! for generated file:
      include!(concat!(env!("OUT_DIR"), "/embedded_agents.rs"));

- [ ] Create static AGENTS: Lazy<AgentsConfig>
- [ ] Modify load_agents() to return AGENTS.clone()
- [ ] Add parse_embedded_agent() helper
- [ ] Optional: Add development override with CCO_AGENTS_DIR
```

### 2.4 Verify No Changes Needed
**File**: `cco/src/server.rs`
```rust
- [ ] Confirm load_agents() call remains unchanged
- [ ] Verify API endpoints don't need modification
```

---

## Phase 3: Build & Test (45 minutes)

### 3.1 Initial Build
```bash
- [ ] cargo clean
- [ ] cargo build 2>&1 | grep "Embedded"
      # Should see: cargo:warning=Embedded 117 agents
- [ ] Check for build errors
```

### 3.2 Unit Tests
```bash
- [ ] cargo test agents_config
      # All existing tests should pass
- [ ] Add test: test_embedded_agents_count
- [ ] Add test: test_chief_architect_embedded
```

### 3.3 Integration Testing
```bash
- [ ] cargo run -- serve &
- [ ] curl http://localhost:11437/health
- [ ] curl http://localhost:11437/api/agents | jq length
      # Should return 117
- [ ] curl http://localhost:11437/api/agents/chief-architect
      # Should return agent details
```

### 3.4 Binary Verification
```bash
- [ ] cargo build --release
- [ ] ls -lh target/release/cco
      # Note binary size (should be ~15-20MB)
- [ ] strings target/release/cco | grep "chief-architect"
      # Verify agents are embedded
```

### 3.5 Portability Test
```bash
- [ ] cp target/release/cco /tmp/test-cco
- [ ] cd /tmp
- [ ] rm -rf ~/.claude/agents  # BACKUP FIRST!
- [ ] ./test-cco serve &
- [ ] curl http://localhost:11437/api/agents
      # Should still return 117 agents
- [ ] Restore ~/.claude/agents from backup
```

---

## Phase 4: Documentation (20 minutes)

### 4.1 Update README
**File**: `cco/README.md`
```markdown
- [ ] Add section: "Embedded Agent Definitions"
- [ ] Update build instructions
- [ ] Document agent file location change
```

### 4.2 Update Development Guide
```markdown
- [ ] Document how to add new agents
- [ ] Explain CCO_AGENTS_DIR override
- [ ] Add troubleshooting section
```

### 4.3 Create Migration Notes
**File**: `cco/docs/AGENT_EMBEDDING_MIGRATION.md`
```markdown
- [ ] Document what changed
- [ ] Explain benefits
- [ ] Include rollback procedure
```

---

## Phase 5: Final Validation (20 minutes)

### 5.1 Cross-Platform Build Check
```bash
- [ ] Build on macOS (if available)
- [ ] Build on Linux (if available)
- [ ] Verify binary works on both platforms
```

### 5.2 Performance Validation
```bash
- [ ] Measure startup time: time cco --version
- [ ] Check memory usage: ps aux | grep cco
- [ ] Verify API response time < 10ms
```

### 5.3 Error Handling
```bash
- [ ] Test with corrupted agent file
- [ ] Test with missing required fields
- [ ] Verify graceful degradation
```

---

## Phase 6: Deployment (15 minutes)

### 6.1 Version Bump
```bash
- [ ] Update CCO_VERSION in build.rs to 2025.11.3
- [ ] Update version in Cargo.toml metadata
```

### 6.2 Git Commit
```bash
- [ ] git add -A
- [ ] git commit -m "feat: embed agent definitions in binary at compile time

      - Migrate 117 agents from ~/.claude/agents to cco/config/agents
      - Implement compile-time embedding via build.rs
      - Zero runtime file I/O for agent loading
      - Binary now fully self-contained and portable"
```

### 6.3 Testing in Production-like Environment
```bash
- [ ] Build release binary
- [ ] Test in Docker container (optional)
- [ ] Verify all endpoints work
```

---

## Post-Implementation Checklist

### Verification
- [ ] All 117 agents accessible via API
- [ ] Binary works without ~/.claude/agents directory
- [ ] No runtime file I/O for agent loading
- [ ] API backwards compatible
- [ ] Build completes in < 30 seconds
- [ ] Binary size increased by < 1MB

### Documentation
- [ ] README updated
- [ ] Migration guide created
- [ ] API documentation current
- [ ] Changelog updated

### Cleanup (Optional)
- [ ] Remove old file-loading code from agents_config.rs
- [ ] Remove references to ~/.claude/agents in docs
- [ ] Archive old agent directory

---

## Rollback Plan

If issues arise, rollback is simple:

1. **Revert Git Changes**
   ```bash
   git revert HEAD
   git push
   ```

2. **Restore Original Code**
   - Agents will load from ~/.claude/agents/ again
   - No data loss as files still exist

3. **Rebuild**
   ```bash
   cargo build --release
   ```

---

## Success Metrics

### Immediate Success
- ✅ Binary includes all agents
- ✅ API returns 117 agents
- ✅ No file I/O at runtime
- ✅ Build succeeds without warnings

### Long-term Success
- 📊 Reduced deployment complexity
- 📊 Improved startup performance
- 📊 Simplified distribution
- 📊 Better version synchronization

---

## Time Estimate

| Phase | Time | Cumulative |
|-------|------|------------|
| Phase 1: Migration | 15 min | 15 min |
| Phase 2: Implementation | 60 min | 75 min |
| Phase 3: Testing | 45 min | 120 min |
| Phase 4: Documentation | 20 min | 140 min |
| Phase 5: Validation | 20 min | 160 min |
| Phase 6: Deployment | 15 min | 175 min |
| **Total** | **~3 hours** | |

---

## Notes

- The implementation is non-breaking and backwards compatible
- Development can use CCO_AGENTS_DIR for testing
- Binary size increase (~500KB) is negligible
- Compile time increase (~2-3s) is minimal
- All existing tests should continue to pass

---

## Sign-off

- [ ] Developer: Implementation complete
- [ ] Reviewer: Code review passed
- [ ] QA: Testing validated
- [ ] Release: Deployed successfully

**Implementation Date**: _____________
**Completed By**: _____________
**Version Released**: CCO v2025.11.3