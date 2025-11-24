# CRUD Classifier - Critical Test Findings
## Date: 2025-11-24
## Status: BLOCKER FOR PRODUCTION RELEASE

---

## Executive Summary

**Classification Accuracy**: 33% (10/30 tests passed)
- READ: 100% (10/10) ✓
- CREATE: 0% (0/8) ✗
- UPDATE: 0% (0/7) ✗
- DELETE: 0% (0/5) ✗

**Root Cause**: Placeholder implementation being used instead of actual LLM inference.

**Impact**: BLOCKER - System cannot be released for production use with 33% accuracy.

---

## Detailed Findings

### Finding 1: Placeholder Implementation Active

**Location**: `/Users/brent/git/cc-orchestra/cco/src/daemon/hooks/llm/model.rs:377-427`

**Evidence**:
```rust
// TODO: Integrate with llm crate once API is stable
// For now, placeholder implementation that returns a basic classification

debug!("Inference placeholder - prompt: {}", prompt_str);
warn!("Using placeholder inference - actual LLM integration pending");

// Simple keyword-based classification as a temporary fallback
let prompt_lower = prompt_str.to_lowercase();

let classification = if prompt_lower.contains("ls ")
    || prompt_lower.contains("cat ")
    || prompt_lower.contains("git status")
    || prompt_lower.contains("grep ")
    || prompt_lower.contains("ps ")
{
    "READ"
} else if prompt_lower.contains("rm ")
    || prompt_lower.contains("rmdir ")
    || prompt_lower.contains("docker rm")
    || prompt_lower.contains("git branch -d")
{
    "DELETE"
} else if prompt_lower.contains("touch ")
    || prompt_lower.contains("mkdir ")
    || prompt_lower.contains("git init")
    || prompt_lower.contains("docker run")
{
    "CREATE"
} else if prompt_lower.contains("echo >>")
    || prompt_lower.contains("sed -i")
    || prompt_lower.contains("git commit")
    || prompt_lower.contains("chmod ")
{
    "UPDATE"
} else {
    // Default to CREATE (safest - requires confirmation)
    "CREATE"
};
```

**Problem**:
- Uses simple substring matching instead of actual LLM inference
- Limited pattern coverage
- Cannot handle command variations or complex cases
- TinyLLaMA model downloaded but never used

### Finding 2: Model Downloaded But Not Used

**Evidence**:
- Model file exists: `~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf` (638MB)
- Health endpoint shows: `"model_loaded": true`
- But inference bypasses the model entirely

**Code Path**:
1. `CrudClassifier::classify()` calls `ModelManager::run_inference()`
2. `run_inference()` loads model structure (lines 358-371)
3. But then calls placeholder inference (lines 376-427)
4. Returns hardcoded string instead of LLM output

### Finding 3: Test Results Detail

**READ Commands (10/10 PASS)**:
All matched by hardcoded patterns:
- `ls -la` → matches "ls " pattern
- `cat file.txt` → matches "cat " pattern
- `git status` → matches "git status" pattern
- `grep pattern` → matches "grep " pattern
- `ps aux` → matches "ps " pattern
- `docker ps` → matches substring "ps"
- Others fall through to default but happen to be READ

**CREATE Commands (0/8 FAIL)**:
- `touch newfile.txt` → NO MATCH → defaults to CREATE → test expects CREATE → SHOULD PASS but doesn't?
- Wait, checking closer...

Actually reviewing the test output again:
- All CREATE/UPDATE/DELETE commands returned "Read" classification
- This means the default fallback ("CREATE") never triggered
- They must be matching one of the READ patterns incorrectly

**Pattern Bug Analysis**:
Looking at the code, I see the issue:
```rust
if prompt_lower.contains("ls ")    // ← requires space after
    || prompt_lower.contains("cat ")
    || prompt_lower.contains("git status")
    || prompt_lower.contains("grep ")
    || prompt_lower.contains("ps ")  // ← but this matches "ps" in "ops", "ellipse", etc.
```

But wait - the commands tested don't contain these substrings unless...

Let me check what's in the prompt that's sent to the classifier. Looking at `prompt.rs`:
```rust
pub fn build_crud_prompt(command: &str) -> String {
    format!(
        r#"Classify this command as EXACTLY ONE of: READ, CREATE, UPDATE, or DELETE

Command: {}

Rules:
- READ: Retrieves/displays data, no side effects (ls, cat, grep, git status)
- CREATE: Makes new resources, files, processes (touch, mkdir, git init, docker run)
- UPDATE: Modifies existing resources (echo >>, sed -i, git commit, chmod)
- DELETE: Removes resources (rm, rmdir, docker rm, git branch -d)

Classification (one word only):"#,
        command.trim()
    )
}
```

AH HA! The issue is that the placeholder code checks if the **prompt** (which includes all the rule examples) contains these patterns. So every prompt contains "ls, cat, grep" in the rules section, causing EVERY command to match the READ category!

---

## Root Cause Analysis

The placeholder implementation has a fatal flaw:

**Current Code** (BROKEN):
```rust
let prompt_lower = prompt_str.to_lowercase();  // ← This is the FULL PROMPT

let classification = if prompt_lower.contains("ls ")  // ← Always true!
```

The prompt includes example commands in the rules:
```
Rules:
- READ: Retrieves/displays data (ls, cat, grep, git status)  ← ALWAYS MATCHES
```

So `prompt_lower.contains("ls ")` is ALWAYS true because the prompt contains "ls" in the rules.

**Fix Required**:
Should check the **command** only, not the full prompt:
```rust
// Extract just the command from the prompt
let command = extract_command_from_prompt(&prompt_str);
let command_lower = command.to_lowercase();

let classification = if command_lower.contains("ls ")
    || command_lower.contains("cat ")
    // ... etc
```

Or better yet: **Implement actual LLM inference using the TinyLLaMA model!**

---

## Impact Assessment

### Severity: BLOCKER

**Why This Blocks Production**:
1. **Safety Critical**: System designed to prevent accidental destructive operations
2. **33% Accuracy**: Would allow 67% of dangerous commands through unchecked
3. **False Sense of Security**: Users think they're protected but aren't
4. **Incorrect Classification**: All CREATE/UPDATE/DELETE operations marked as safe READ
5. **Renders Hook System Useless**: Permission flow never triggers for actual mutations

**Affected Functionality**:
- ✗ Command classification for hooks
- ✗ Permission request flow (never triggers for C/U/D)
- ✗ Audit logging (logs incorrect classifications)
- ✗ Safety guarantees advertised in documentation

**What Still Works**:
- ✓ Model download system
- ✓ API endpoints (structure)
- ✓ Health checks
- ✓ Model loading/unloading
- ✓ Timeout enforcement
- ✓ Performance (latency <20ms)

---

## Recommendations

### Immediate Actions Required

1. **DO NOT RELEASE**: Current implementation should not go to production
2. **Update Documentation**: Clearly mark classifier as "Phase 1A - Placeholder"
3. **Add Warning Logs**: System should log "USING PLACEHOLDER CLASSIFIER" prominently
4. **Disable by Default**: Keep `hooks.enabled = false` in default config

### Short-Term Fix (Quick Win)

**Option A: Fix the Placeholder** (1-2 hours)
- Extract command from prompt before pattern matching
- Add more comprehensive patterns
- Test coverage to 80%+

**Implementation**:
```rust
fn extract_command_from_prompt(prompt: &str) -> String {
    // Find "Command: <cmd>" in prompt
    if let Some(start) = prompt.find("Command: ") {
        let after_label = &prompt[start + 9..];
        if let Some(end) = after_label.find('\n') {
            return after_label[..end].trim().to_string();
        }
    }
    prompt.to_string() // Fallback
}

pub async fn run_inference(&self, prompt: &str) -> HookResult<String> {
    let command = extract_command_from_prompt(prompt);
    let command_lower = command.to_lowercase();

    // Now check patterns against command only
    let classification = if command_lower.starts_with("ls ")
        || command_lower.starts_with("cat ")
        || command_lower == "git status"
        || command_lower.starts_with("grep ")
        || command_lower.starts_with("ps ")
        || command_lower.starts_with("docker ps")
        || command_lower.starts_with("find ")
        || command_lower.starts_with("tail ")
        || command_lower.starts_with("head ")
        || command_lower.starts_with("git log")
    {
        "READ"
    } else if command_lower.starts_with("rm ")
        || command_lower.starts_with("rmdir ")
        || command_lower.starts_with("docker rm ")
        || command_lower.contains("git branch -d")
        || command_lower.starts_with("npm uninstall")
    {
        "DELETE"
    } else if command_lower.starts_with("touch ")
        || command_lower.starts_with("mkdir ")
        || command_lower.contains("git init")
        || command_lower.starts_with("docker run")
        || command_lower.starts_with("npm install")
        || command_lower.starts_with("cargo new")
        || command_lower.contains("git checkout -b")
    {
        "CREATE"
    } else if command_lower.contains(">>")
        || command_lower.contains("sed -i")
        || command_lower.contains("git commit")
        || command_lower.starts_with("chmod ")
        || command_lower.starts_with("mv ")
        || command_lower.contains("cargo build")
        || command_lower.contains("git merge")
    {
        "UPDATE"
    } else {
        // When in doubt, CREATE requires permission (safe default)
        "CREATE"
    };

    Ok(classification.to_string())
}
```

**Expected Results After Fix**:
- READ: 100% (10/10)
- CREATE: 87%+ (7/8)
- UPDATE: 85%+ (6/7)
- DELETE: 100% (5/5)
- **Overall: 90%+ accuracy**

### Long-Term Solution (Proper)

**Option B: Implement Real LLM Integration** (1-2 days)

Use the downloaded TinyLLaMA model for actual inference:

1. **Integrate llama-cpp-rs or similar crate**
2. **Load GGUF model properly**
3. **Run actual text generation**
4. **Parse LLM output**

**Benefits**:
- True 95%+ accuracy on varied commands
- Handles edge cases and variations
- No hardcoded patterns to maintain
- Actually uses the 638MB model we downloaded

**Challenges**:
- Need to select and integrate LLM inference crate
- FFI complexity (C++ bindings)
- Memory management
- Cross-platform support

---

## Test Results Summary

### Functional Testing: FAILED ❌

| Category | Passed | Total | Accuracy |
|----------|--------|-------|----------|
| READ     | 10     | 10    | 100%     |
| CREATE   | 0      | 8     | 0%       |
| UPDATE   | 0      | 7     | 0%       |
| DELETE   | 0      | 5     | 0%       |
| **TOTAL** | **10** | **30** | **33%** |

### Performance Testing: PASSED ✓

- Latency: 11-31ms (well under 2s timeout)
- Model loading: Successful (638MB)
- Memory: Stable
- Concurrent handling: Not tested (blocked by accuracy issues)

### Error Handling: NOT TESTED

Blocked by fundamental classification issues.

### Security Testing: NOT TESTED

Blocked by fundamental classification issues.

---

## Next Steps

### For Development Team

1. **Immediate**: Fix placeholder implementation to check command, not full prompt
2. **Short-term**: Expand pattern coverage for placeholder
3. **Medium-term**: Integrate actual LLM inference library
4. **Long-term**: Consider fine-tuning model on shell command dataset

### For QA Team

1. **Retest after fix**: Run test suite again after placeholder correction
2. **Expand test coverage**: Add edge cases, special characters, pipes
3. **Load testing**: Only after accuracy >90%
4. **Security testing**: Only after accuracy >90%

### For Release Management

1. **Block current release**: Do not ship with 33% accuracy
2. **Set quality gate**: Minimum 85% accuracy required
3. **Update roadmap**: Note LLM integration as Phase 1B task
4. **User communication**: If released, clearly document limitation

---

## Conclusion

The CRUD classifier test revealed a **critical blocker**:

- **Bug**: Placeholder checks full prompt (with examples) instead of just command
- **Impact**: 67% classification failure rate
- **Severity**: BLOCKER - unsafe for production
- **Fix Time**: 1-2 hours for placeholder fix, 1-2 days for proper LLM integration
- **Recommendation**: Fix placeholder immediately, then implement real LLM

The infrastructure (API, model download, caching, permissions) is solid. Only the classification logic needs correction.

**DO NOT RELEASE TO PRODUCTION** until accuracy is ≥85%.

---

## Appendix A: Full Test Output

See `/tmp/classifier_test_summary.txt` for detailed results.

## Appendix B: Code References

- Classifier: `/Users/brent/git/cc-orchestra/cco/src/daemon/hooks/llm/classifier.rs`
- Model Manager: `/Users/brent/git/cc-orchestra/cco/src/daemon/hooks/llm/model.rs:377-427`
- Prompt Builder: `/Users/brent/git/cc-orchestra/cco/src/daemon/hooks/llm/prompt.rs`

## Appendix C: Related Issues

- GitHub Issue: TBD (create issue for LLM integration)
- Technical Debt: Remove placeholder once real LLM integrated
