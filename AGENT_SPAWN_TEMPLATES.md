# Agent Spawn Templates - Copy & Paste Ready

Use these templates when spawning agents via the Task tool. **Replace the model from `"sonnet"` to the correct configured model.**

## ✅ Correct Format

```javascript
Task(
  "Description",
  "Full prompt here...",
  "agent-type",
  "correct-model"  // ← Use configured model, NOT "sonnet"
)
```

---

## Your 5 Agents - Ready to Use

### 1. Rust Specialist (Haiku) ✅

**Current (WRONG):**
```javascript
Task(
  "Implement daemon mode",
  "Full prompt...",
  "rust-specialist",
  "sonnet"  // ❌ WRONG
)
```

**Fixed (CORRECT):**
```javascript
Task(
  "Implement daemon mode",
  "You are a Rust development specialist... Full prompt here...",
  "rust-specialist",
  "haiku"  // ✅ CORRECT
)
```

---

### 2. DevOps Engineer (Haiku) ✅

**Current (WRONG):**
```javascript
Task(
  "Implement log rotation",
  "Full prompt...",
  "devops-engineer",
  "sonnet"  // ❌ WRONG
)
```

**Fixed (CORRECT):**
```javascript
Task(
  "Implement log rotation",
  "You are a DevOps engineer... Full prompt here...",
  "devops-engineer",
  "haiku"  // ✅ CORRECT
)
```

---

### 3. Frontend Developer (Haiku) ✅

**Current (WRONG):**
```javascript
Task(
  "Add shutdown button",
  "Full prompt...",
  "frontend-developer",
  "sonnet"  // ❌ WRONG
)
```

**Fixed (CORRECT):**
```javascript
Task(
  "Add shutdown button",
  "You are a frontend developer... Full prompt here...",
  "frontend-developer",
  "haiku"  // ✅ CORRECT
)
```

---

### 4. Test Engineer (Haiku) ✅

**Current (WRONG):**
```javascript
Task(
  "Test daemon functionality",
  "Full prompt...",
  "test-engineer",
  "sonnet"  // ❌ WRONG
)
```

**Fixed (CORRECT):**
```javascript
Task(
  "Test daemon functionality",
  "You are a test engineer... Full prompt here...",
  "test-engineer",
  "haiku"  // ✅ CORRECT
)
```

---

### 5. Documentation Expert (Haiku) ✅

**Current (WRONG):**
```javascript
Task(
  "Update documentation",
  "Full prompt...",
  "documentation-expert",
  "sonnet"  // ❌ WRONG
)
```

**Fixed (CORRECT):**
```javascript
Task(
  "Update documentation",
  "You are a documentation expert... Full prompt here...",
  "documentation-expert",
  "haiku"  // ✅ CORRECT
)
```

---

## Quick Find & Replace

If you have these hardcoded in a script:

```bash
# Find all instances
grep -r "rust-specialist.*sonnet\|devops-engineer.*sonnet\|frontend-developer.*sonnet\|test-engineer.*sonnet\|documentation-expert.*sonnet" .

# Replace (if using sed):
sed -i 's/"rust-specialist", "sonnet"/"rust-specialist", "haiku"/g' *.js
sed -i 's/"devops-engineer", "sonnet"/"devops-engineer", "haiku"/g' *.js
sed -i 's/"frontend-developer", "sonnet"/"frontend-developer", "haiku"/g' *.js
sed -i 's/"test-engineer", "sonnet"/"test-engineer", "haiku"/g' *.js
sed -i 's/"documentation-expert", "sonnet"/"documentation-expert", "haiku"/g' *.js
```

---

## Verify Your Changes

After updating, verify the models are correct:

```bash
# Check rust-specialist model
jq '.codingAgents[] | select(.type=="rust-specialist") | .model' config/orchestra-config.json
# Output should be: "haiku"

# Validate configuration
jq '.codingAgents[] | select(.type=="rust-specialist")' config/orchestra-config.json
# Should display the agent configuration
```

---

## Cost Comparison

Running with **Sonnet** (current):
```
rust-specialist:        ~30k tokens × $0.003/1k = $0.09
devops-engineer:        ~40k tokens × $0.003/1k = $0.12
frontend-developer:     ~44k tokens × $0.003/1k = $0.13
test-engineer:          ~52k tokens × $0.003/1k = $0.16
documentation-expert:   ~72k tokens × $0.003/1k = $0.22
────────────────────────────────────────────────────────
Total per run:          ~238k tokens = ~$0.72
Monthly (50 runs):      ~$36
```

Running with **Haiku** (fixed):
```
rust-specialist:        ~30k tokens × $0.0008/1k = $0.024
devops-engineer:        ~40k tokens × $0.0008/1k = $0.032
frontend-developer:     ~44k tokens × $0.0008/1k = $0.035
test-engineer:          ~52k tokens × $0.0008/1k = $0.042
documentation-expert:   ~72k tokens × $0.0008/1k = $0.058
────────────────────────────────────────────────────────
Total per run:          ~238k tokens = ~$0.191
Monthly (50 runs):      ~$9.55
```

**Monthly savings: ~$26.45 (73% reduction)**

---

## Summary

1. **Find** where you call Task with these agents
2. **Replace** the model from `"sonnet"` to `"haiku"`
3. **Verify** using `jq '.codingAgents[] | select(.type=="<agent-type>")' config/orchestra-config.json`
4. **Enjoy** significant cost savings! 💰
